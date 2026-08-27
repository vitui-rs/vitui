//! **F6 input**, ~110 entries — the largest family — expressed by `field`, `button`, `chip`,
//! `select`, `collection` and `slider`.
//!
//! # `slider` is the row §17 froze at Tier 3, and this file is where the column stops lying
//!
//! §14 measured drag capture over the video player's seek bar and concluded *`slider` leaves
//! Tier 3* — so `INVENTORY`'s `built` column read `true` and `MEMBERS` listed the name for two
//! tickets before anything here declared a `slider`. **Nothing could see it**: the `MEMBERS`
//! join compares the module's list against the `families` column and never against the source,
//! and the tier/built gate accepts any row that appears in `MOVED`. Components ticket 33 ships
//! the component *and* the join that would have caught it —
//! `inventory::tests::every_built_row_is_declared_in_the_module_that_homes_it`.
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

use vitui_runtime::keys::{Code, Edge, Pressed};
use vitui_runtime::layout::text::{truncate, width};
use vitui_runtime::overlay::{OverlayOpts, Placement, Z};
use vitui_runtime::{Ctx, CursorShape, Glyph, Id, Interest, Response, Role};

use crate::collect::{CollOpts, CollState, Mode, collection_chorded};
use crate::edit::{Text, WrapKind};
use crate::frame::{Face, face_paint};
use crate::ink::{Direct, Ink};
use crate::keys;
use crate::order::Rows;
use crate::overlay::{Blur, MARK, PopupState, ShellOpts, overlay_with, popup_size};
use crate::scroll::Orient;
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

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// `select` — the owner of a popup, and the second half of §12's two structs
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **What the owner owns and the body never writes.**
///
/// §12's first half of *two structs, one writer each*: this is written only by `select`, from its own
/// input and from the inbox, and [`PopupState`] is written only by the body. Nothing has two writers,
/// which is §7's goal reached one family over.
///
/// **It is `Copy` and [`PopupState`] is not**, and the asymmetry is the mechanism: this crosses the
/// base pass by value, and the body's state crosses the *frame* by `&'f mut`. See
/// [`WhyThePopupIsRequestedLast`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct SelectState {
    /// Whether the popup is standing. **The owner's, and the only thing that requests a layer** —
    /// the census is over the request, so this going false is
    /// [`Dismissal::OwnerStopped`](crate::overlay::Dismissal::OwnerStopped).
    open: bool,
    /// Which option is chosen, as an index into the caller's list.
    chosen: usize,
    /// **Whether the popup has held the keyboard during *this* opening.** The latch a blur clause
    /// needs, and it lives here rather than on [`PopupState`] because **the owner is the only thing
    /// that knows an opening has begun**: [`SelectState::open`] clears it, and an application calling
    /// that verb directly gets the same clearing as a keystroke does.
    ///
    /// Derived from what the body reports and written only here, so §12's one-writer rule is
    /// untouched: the body says *the focus is inside me* and the owner remembers that it once did.
    seated: bool,
}

impl SelectState {
    /// Shut, on the first option.
    #[must_use]
    pub const fn new() -> SelectState {
        SelectState {
            open: false,
            chosen: 0,
            seated: false,
        }
    }

    /// Shut, on `chosen`.
    #[must_use]
    pub const fn at(chosen: usize) -> SelectState {
        SelectState {
            open: false,
            chosen,
            seated: false,
        }
    }

    /// Which option is chosen.
    #[must_use]
    pub const fn chosen(&self) -> usize {
        self.chosen
    }

    /// Whether the popup is standing.
    #[must_use]
    pub const fn is_open(&self) -> bool {
        self.open
    }

    /// **Whether the popup has held the keyboard during this opening.**
    #[must_use]
    pub const fn seated(&self) -> bool {
        self.seated
    }

    /// **Open it.** An application's own gesture — a `Ctrl+Space`, a toolbar button — reaches the
    /// popup this way rather than by faking a click.
    ///
    /// It clears the keyboard latch, and that is what makes a popup **reopenable after a blur**: the
    /// body's last report is still *the focus is not inside me*, and without the clearing the blur
    /// clause fires on the frame after the reopening, before the body has had a frame to hand the
    /// keyboard over.
    pub const fn open(&mut self) {
        self.open = true;
        self.seated = false;
    }

    /// **Shut it**, which stops the request, which is what dismisses the layer.
    pub const fn close(&mut self) {
        self.open = false;
        self.seated = false;
    }
}

/// **Where a popup's size comes from. Three spellings, and two of them are §12's.**
///
/// One field rather than a boolean in a signature, so a reviewer's diff between the shipped build
/// and either refused one is a single line — [`crate::disclose`]'s arrangement one family over.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Sizing {
    /// **The rule.** [`popup_size`] over the options and the room the screen has, so the height is
    /// capped and the body's [`gutter`](crate::overlay::gutter) turns *off the bottom* into *scrollable*.
    #[default]
    ToTheRoom,
    /// **The defect §12 prices at one unreachable row of four.** Sized to the content, uncapped.
    /// [`place`](vitui_runtime::overlay::place) clamps a position and never a size, so a popup taller
    /// than the screen hangs off the bottom edge at its stated size, [`gutter`](crate::overlay::gutter) sees as many rows as
    /// it has content and says *no bar*, and the rows past the edge are drawn, clipped, and reachable
    /// by nothing.
    ToTheContent,
    /// **The defect §12 leads with.** Sized from the drawn extent the body reported last time it
    /// ran. A popup has no frame before the one it opens on, so the extent is zero, so it is granted
    /// zero rows, so it draws nothing, so the extent is zero: granted
    /// [`SPEC_GRANTED_FROM_EXTENT`](crate::overlay::SPEC_GRANTED_FROM_EXTENT) against
    /// [`SPEC_GRANTED`](crate::overlay::SPEC_GRANTED), for ever.
    FromTheDrawnExtent,
}

impl Sizing {
    /// All three, in the order the enum argues them.
    pub const ALL: [Sizing; 3] = [
        Sizing::ToTheRoom,
        Sizing::ToTheContent,
        Sizing::FromTheDrawnExtent,
    ];

    /// The word a report prints it under.
    pub const fn word(self) -> &'static str {
        match self {
            Sizing::ToTheRoom => "to the room",
            Sizing::ToTheContent => "to the content",
            Sizing::FromTheDrawnExtent => "from the drawn extent",
        }
    }
}

/// [`select`]'s options. Spec §1's rule 3: a `Default` struct, never a required builder.
///
/// **`Copy`, and that is load-bearing**: the popup body captures the options *by value*, so a
/// `&'f Self` would be a second `'f` borrow at every call site for no reason.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SelectOpts {
    /// What the shut widget is interested in.
    pub interest: Interest,
    /// The faces the shut widget wears. [`crate::state::press`] reads `hover` once.
    pub faces: Faces,
    /// Where the popup lands against the widget.
    pub placement: Placement,
    /// The band the popup's layer sorts into. [`Z::MENU`] by default; a `select` inside a modal is
    /// nested and takes its parent's band plus [`Z::NESTED`] whatever this says.
    pub z: i32,
    /// How many rows one type-ahead keystroke inside the popup may look at.
    pub search: usize,
}

impl Default for SelectOpts {
    fn default() -> SelectOpts {
        SelectOpts {
            interest: Interest::CLICK.with(Interest::HOVER).with(Interest::FOCUS),
            faces: Faces::default(),
            placement: Placement::BELOW,
            z: Z::MENU,
            search: crate::collect::SEARCH_BUDGET,
        }
    }
}

/// **The key the refused catcher layer is minted under, off the `select`'s own id.**
///
/// Spelled rather than minted from a call site because a `select` requests two layers on that arm and
/// one owner is one layer: shared, the second is inert and `Frame::overlays_merged` counts it.
pub(crate) const CATCHER_KEY: u64 = 2;

/// **Why the popup is requested last, and why that is a borrow error rather than a comment.**
///
/// The body holds `&'f mut PopupState`, and `'f` is the frame call's: the queue holds the body until
/// the satisfy pass, so the borrow lives exactly that long. A caller that reads its own popup state
/// after handing it to [`select`] therefore fails — and **the diagnostic never mentions the
/// overlay**. It arrives at the caller, one level away, on the next ordinary read.
///
/// # The hostile half
///
/// **Protects:** [`select`] and [`PopupState`], written
/// `vitui_components::input::select` and `vitui_components::overlay::PopupState` at the paths the
/// twin uses. The failure is the caller's read, not the call.
///
/// ```compile_fail,E0502
/// use vitui_components::input::{SelectState, select};
/// use vitui_components::overlay::PopupState;
/// use vitui_runtime::ctx::Driver;
///
/// static OPTIONS: [&str; 2] = ["name", "size"];
///
/// fn main() {
///     let mut driver = Driver::headless(40, 12).expect("a sink attaches");
///     let mut st = SelectState::new();
///     let mut popup = PopupState::new();
///     driver.frame(|cx| {
///         let area = cx.area();
///         let _ = select(cx, area, &mut st, &mut popup, &OPTIONS);
///         // The body still holds it, so this is `E0502` — and nothing in the message says
///         // "overlay".
///         let _ = popup.granted();
///     });
/// }
/// ```
///
/// and the twin that names both items by path and does the same thing in the order that works:
/// **read it before the request, never after.**
///
/// ```
/// use vitui_components::input::{SelectState, select};
/// use vitui_components::overlay::PopupState;
/// use vitui_runtime::ctx::Driver;
///
/// static OPTIONS: [&str; 2] = ["name", "size"];
///
/// let mut driver = Driver::headless(40, 12).expect("a sink attaches");
/// let mut st: SelectState = SelectState::new();
/// let mut popup: PopupState = PopupState::new();
/// driver.frame(|cx| {
///     let area = cx.area();
///     // Read first. The granted size is last frame's, which is what it is for.
///     let _ = popup.granted();
///     let _ = select(cx, area, &mut st, &mut popup, &OPTIONS);
/// });
/// ```
#[cfg(doc)]
pub struct WhyThePopupIsRequestedLast;

/// **`select` — a shut face, and a popup that is the owner's.**
///
/// Spec §1's shape, with the one cost §1 records: **an overlay costs two lifetime annotations**, and
/// this is the component §1 measured them on. `popup` and `options` are `'f` because the body
/// captures them and a body is `+ 'f`.
///
/// # The three parties of the rectangle
///
/// The **anchor** is `area`, this component's own, in its own coordinates, during its own call. The
/// **size** comes from [`popup_size`] — a sizing function beside the component, no draw context. The
/// **placement** is the runtime's. The component never hears what it was granted; the *body* does,
/// through `cx.area()`, and that is the only place it is knowable.
///
/// # The choice comes home through the inbox
///
/// The body cannot assign to [`SelectState`]: it does not have one, and it could not be given one,
/// because the owner is still holding it when the body runs. It writes [`PopupState`]'s answer slot
/// and this function takes it **at the top of the frame after**. That is the whole reason there are
/// two structs.
///
/// # Dismissal, and the one that needs a position
///
/// `Esc` and a choice are the body's, and arrive through the inbox. Shutting `open` is
/// [`Dismissal::OwnerStopped`](crate::overlay::Dismissal::OwnerStopped) — the census is over the request, so a layer nobody asked for is
/// gone. And blur is qualified by a **position**: `begin` hands out an optimistic focus a frame
/// before the body can speak, so `focus_left` alone dismisses a popup the pointer is standing on.
/// [`PopupState::over`] is the qualification, and it is a hover and never a press.
///
/// ```
/// use vitui_components::input::{SelectState, select};
/// use vitui_components::overlay::PopupState;
/// use vitui_runtime::ctx::Driver;
/// use vitui_runtime::Rect;
///
/// static OPTIONS: [&str; 3] = ["name", "date modified", "size"];
///
/// let mut driver = Driver::headless(40, 12).expect("a sink attaches");
/// let mut st = SelectState::at(2);
/// let mut popup = PopupState::new();
///
/// // Shut: one hit entry, one tab stop, and no layer at all.
/// driver.frame(|cx| {
///     let _ = select(cx, Rect::new(0, 0, 20, 1), &mut st, &mut popup, &OPTIONS);
/// });
/// assert_eq!(driver.inspect().hits().len(), 1);
/// assert_eq!(driver.inspect().overlays_placed(), 0);
///
/// // Open: the owner's entry, the popup's blur position, and the collection's — three, one layer.
/// st.open();
/// driver.frame(|cx| {
///     let _ = select(cx, Rect::new(0, 0, 20, 1), &mut st, &mut popup, &OPTIONS);
/// });
/// assert_eq!(driver.inspect().overlays_placed(), 1);
/// assert_eq!(driver.inspect().overlays_merged(), 0);
/// // The body reported what it was granted: as wide as the widget, three options tall — never zero.
/// assert_eq!(popup.granted(), (20, 3));
/// ```
#[track_caller]
pub fn select<'f>(
    cx: &mut Ctx<'f, '_>,
    area: Rect,
    st: &mut SelectState,
    popup: &'f mut PopupState,
    options: &'f [&'f str],
) -> Response {
    select_with(cx, area, st, popup, options, &SelectOpts::default())
}

/// [`select`], with the options spelled out.
#[track_caller]
pub fn select_with<'f>(
    cx: &mut Ctx<'f, '_>,
    area: Rect,
    st: &mut SelectState,
    popup: &'f mut PopupState,
    options: &'f [&'f str],
    opts: &SelectOpts,
) -> Response {
    let id = cx.id();
    select_into(&mut Direct, cx, id, area, st, popup, options, opts)
}

/// **[`select`], drawing its shut face through an [`Ink`] and under an id its caller minted.**
///
/// The entry point a gate takes; [`select`] is this with [`Direct`].
///
/// # The popup's body draws through [`Direct`] and not through `ink`
///
/// A body is `FnMut(&mut Ctx<'f, '_>) + 'f`, so a `&mut I` borrowed for this call cannot travel into
/// one. That is not a hole in the seam: a body's `Ctx` is rooted at its own layer, so its writes are
/// in a different coordinate system from the base pass's and a [`Tally`](crate::counters::Tally) that
/// saw both would union two grids — the collision [`crate::frame`] measures at 124 false double
/// writes. The counters an overlay actually moves are the frame's, and those are read from the frame.
#[track_caller]
#[expect(
    clippy::too_many_arguments,
    reason = "the component's own six — a context, an id, a rectangle, the owner's state, the \
              body's state and the option list — plus the options and the `Ink` seam's writer. \
              Folding the first six into a parameter struct would invent a type that exists only \
              to satisfy a lint, and the two states cannot be folded together at all: that is \
              §12's finding"
)]
pub fn select_into<'f, I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'f, '_>,
    id: Id,
    area: Rect,
    st: &mut SelectState,
    popup: &'f mut PopupState,
    options: &'f [&'f str],
    opts: &SelectOpts,
) -> Response {
    select_shaped(
        ink,
        cx,
        id,
        area,
        st,
        popup,
        options,
        opts,
        SelectShape::default(),
    )
}

/// **The four axes `select` and its body can be false on that are not on [`SelectOpts`]**, as one
/// value.
///
/// One struct rather than four booleans in a signature, so a reviewer's diff between the shipped
/// build and any refused one is a single line — [`crate::disclose`]'s arrangement one family over,
/// and its reason: the diff a reviewer would have to catch is the diff the register can point at.
///
/// **`Copy`, because the body captures it.** Every arm below has to reach the popup, and the popup is
/// a closure that outlives the base pass.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct SelectShape {
    /// Where the popup's size comes from.
    pub(crate) sizing: Sizing,
    /// Whether the popup fills its rectangle before it writes its rows.
    pub(crate) fill: Fill,
    /// Whether the popup declares one entry for its collection or one a row.
    pub(crate) regions: PopupRegions,
    /// What a popup with no height does.
    pub(crate) closed: Closed,
    /// What the body holds its list position in.
    pub(crate) holds: Holds,
    /// How the owner qualifies a blur.
    pub(crate) blur: Blur,
}

/// **What an overlay body holds its list position in.**
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Holds {
    /// **The rule.** `&'f mut PopupState` — the caller's own storage, borrowed for the frame.
    #[default]
    ByMutRef,
    /// **The defect.** A `Copy` of the offset, captured by value at request time. The body writes
    /// into a value that dies with the frame, so the owner hands it the same number again next frame:
    /// [`crate::popup::WHEEL_CLICKS`] notches move the offset **0**, and the screen is identical
    /// while it happens.
    CopyOfTheOffset,
}

/// **In what order a popup writes its rectangle.**
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Fill {
    /// **The rule.** Each row is a partition of its own width, written once, through
    /// [`Ink::pad_to`].
    #[default]
    TextFirst,
    /// **The defect.** A fill over the whole rectangle before a single row is written, so every
    /// non-blank cell the rows then write is written twice and re-damaged for as long as the popup
    /// stands: [`crate::popup::FILL_FIRST`] cells a frame.
    FillFirst,
}

/// **How many hit entries a popup's list declares.**
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PopupRegions {
    /// **The rule**, and it is §5's: one entry for the collection, however many rows it has.
    #[default]
    Collection,
    /// **The defect.** One entry and one tab stop a row —
    /// [`crate::popup::PER_ROW_ENTRIES`] of each where the rule spends one.
    PerRow,
}

/// **What a popup granted no height does.**
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Closed {
    /// **The rule.** [`overlay`](crate::overlay::overlay) returns inert on an empty rectangle and the
    /// body is never reached, so nothing is declared: §8's *closed content is not drawn*, one family
    /// over.
    #[default]
    Skip,
    /// **The defect.** The body declares before it looks at what it was granted, so a popup at
    /// `h = 0` still spends its entries and its stops — [`crate::popup::CLOSED_DECLARES`].
    Declare,
}

/// The component, with the four refused spellings threaded in.
#[expect(
    clippy::too_many_arguments,
    reason = "the shipped signature plus the one value that carries every refused spelling, so the \
              arms are one call apart"
)]
fn select_shaped<'f, I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'f, '_>,
    id: Id,
    area: Rect,
    st: &mut SelectState,
    popup: &'f mut PopupState,
    options: &'f [&'f str],
    opts: &SelectOpts,
    shape: SelectShape,
) -> Response {
    if area.is_empty() {
        return Response::inert(id, area);
    }

    // **The inbox, taken first.** The body wrote it on the frame before and this is the only place
    // it is read, which is what makes `SelectState` single-writer. Taking it is the dismissal:
    // `Dismissal::Chose` and `Dismissal::Escape` both arrive here, and the difference between them
    // is whether there is an index in the slot.
    if let Some(chosen) = popup.take() {
        st.chosen = chosen.min(options.len().saturating_sub(1));
        st.close();
        // **On the way out the owner refocuses itself** (§12) — one id it already has, so no id
        // belonging to anybody else is named. It has to: the keyboard is on a list id minted inside a
        // body that will not run again, so left alone the **vanish rule** picks a ring neighbour. On a
        // screen with one widget that is invisible; on a screen with two it is the *other* `select`,
        // which is how this line came to be missing and then found.
        cx.focus(id);
    }

    let resp = cx.interact(id, area, opts.interest);

    // **The shut face**, one role read once. `press` declares the hover award from the same local it
    // returns, so the face drawn and the face awarded are one expression evaluated once.
    let role = crate::state::press_into(ink, cx, area, &resp, &opts.faces);
    let paint = cx.theme().paint(role);
    // **The chevron pair is `ArrowDown`/`ArrowRight`**, which is the same family as the popup's
    // steppers — §16's reason for four arrow ends and not eight.
    let chevron = cx.theme().glyph(if st.is_open() {
        Glyph::ArrowDown
    } else {
        Glyph::ArrowRight
    });
    let label = options.get(st.chosen).copied().unwrap_or("");
    // **The row is a partition of its width**: the chevron and its space, then the label padded out
    // to the end. `pad_to` and not `text` then `run`, so `verbs` cannot separate a full row from a
    // short one — `crate::ink`'s fourth verb and its reason.
    //
    // **And the pad stops where the ellipsis starts.** `glyphs::elide` has already reserved the
    // marker's cell, so padding to the whole width and then writing the marker over the pad's last
    // cell writes that cell **twice** — one cell of every truncated `select`, invisible on the screen
    // and invisible to every counter but the pair. That was in the first draft of this file, and
    // `overlay::tests::a_shut_selects_face_is_a_partition_of_its_rectangle_at_every_width` is what
    // caught it.
    // **Through `crate::glyphs::elided_row_into`, which is where the pad and the marker meet.**
    // It was written out here until components 32, whose `file_picker` transcribed it — two copies
    // of a drawing that had already been wrong once is one copy too many, so the drawing is one
    // function and both callers spend their own prefix before it.
    let _ = ink.text(cx, area.x, area.y, chevron, paint);
    let _ = ink.text(cx, area.x + 1, area.y, " ", paint);
    let _ = crate::glyphs::elided_row_into(
        ink,
        cx,
        area.x + i32::from(MARK),
        area.y,
        label,
        area.w.saturating_sub(MARK),
        paint,
    );

    // **A click on the shut face opens it and takes the focus**; a click on the open face shuts it,
    // which is the same gesture and the same one bool.
    if resp.clicked {
        cx.focus(id);
        if st.is_open() {
            st.close();
        } else {
            st.open();
        }
    }

    // **The owner's keys, in one drain loop, and only while it is shut.** Open, the focus is inside
    // the popup — the trap-less equivalent of a modal — and `next_key` answers nobody else.
    while let Some(k) = cx.next_key(id) {
        if k.kind == Edge::Release {
            continue;
        }
        match k.code {
            Code::Enter | Code::Char(' ') | Code::Down => st.open(),
            // **`Esc` only while it is open**, and the guard is not tidiness: a shut widget that
            // consumes `Esc` takes it away from the application, and an application whose quit key
            // is `Esc` then has none — with nothing on screen to say so. Found by running the
            // application: `Esc` did not quit `console` and no popup was up. Declined instead, it
            // reaches `Driver::unhandled` like any key nobody wanted.
            Code::Escape if st.is_open() => st.close(),
            _ => {
                cx.decline(k);
                break;
            }
        }
    }

    // **The latch, moved forward by what the body reported.** The owner remembers that its popup once
    // held the keyboard; `SelectState::open` clears it, which is what makes a popup reopenable after a
    // blur — the body's last report is still *the focus is not inside me*, and without the clearing
    // the clause below fires on the frame after the reopening, before the body has had a frame to hand
    // the keyboard over. `open` is the owner's own verb, so an application that opens the popup
    // directly gets the same clearing a keystroke does.
    if st.is_open() && popup.inside() {
        st.seated = true;
    }

    // **What a blur is, from the owner's side, is `seated && !inside && !over`** — and
    // `Response::focus_left` is not in it. That is this ticket's application talking: the moment the
    // popup takes the keyboard the owner no longer holds the focus, so it has none to *lose*, and a
    // clause built on the owner's `focus_left` either never fires or fires on the handover itself.
    // §12's *`focus_left` is what an outside click already produces* is true of the **popup's** id
    // and not of its owner's.
    //
    // `seated` is the latch that makes it safe on the frame the popup opens: before the body has run,
    // `inside` is false for the same reason `granted` is `(0, 0)` — nothing has happened yet — and a
    // rule reading `!inside` alone would dismiss every popup on the frame it opened.
    if st.is_open() && st.seated() {
        // **A focus that is still inside the popup is not a blur**, and the arm that forgets this
        // clause is the defect the application found: handing the keyboard to the list reads as the
        // user tabbing away, so the popup dismisses itself on the very next frame with the arrows
        // dead and nothing on screen having gone wrong.
        let handover = match shape.blur {
            Blur::Position | Blur::Press | Blur::Catcher => popup.inside(),
            Blur::PositionAlone => false,
        };
        let qualified = !handover
            && match shape.blur {
                // **The rule**: the position the body reported on the frame it last ran. The catcher
                // qualifies the same way — it exists to make the outside press *explicit*, not to
                // change what a blur means, and what it costs is the press it swallows.
                Blur::Position | Blur::Catcher | Blur::PositionAlone => !popup.over(),
                // **The defect**: a press. `begin`'s optimistic focus arrives a frame ahead of the
                // body, so on the frame that matters there is no press to read and the popup
                // dismisses itself out from under a pointer that is standing on it.
                Blur::Press => !resp.clicked,
            };
        if qualified {
            st.close();
        }
    }

    // **The request is last, and it has to be**: `popup` moves into the body here, and every read
    // above it is a read the borrow checker has already allowed. Written anywhere but last this is
    // `E0502` at the caller — see `WhyThePopupIsRequestedLast`.
    if st.is_open() {
        let room = cx.size();
        let size = match shape.sizing {
            Sizing::ToTheRoom => popup_size(options, room),
            Sizing::ToTheContent => (
                popup_size(options, (room.0, u16::MAX)).0,
                u16::try_from(options.len()).unwrap_or(u16::MAX),
            ),
            Sizing::FromTheDrawnExtent => {
                (popup_size(options, (room.0, u16::MAX)).0, popup.granted().1)
            }
        };
        // **At least as wide as the widget it drops from**, which is the component's and not the
        // sizing function's: a sizing function sees the data and the room and never the anchor, and a
        // popup narrower than the thing it dropped out of reads as a different widget. Capped by the
        // room for `popup_size`'s reason.
        let size = (size.0.max(area.w).min(room.0), size.1);
        let chosen = st.chosen;
        let search = opts.search;
        // **The refused catcher**, and it is a whole second layer: the screen's worth of cells, one
        // entry over all of them, and the press it reports is a press the widget beneath does not
        // get. `Ctx::area` inside a popup's own body is the popup, so this cannot live in the shell
        // — a catcher is the *owner's* second request or it is nothing.
        if shape.blur == Blur::Catcher {
            let screen = cx.area();
            cx.overlay(
                Id::keyed(id, CATCHER_KEY),
                screen,
                OverlayOpts {
                    z: opts.z - 1,
                    placement: Placement::UNDER,
                    ..OverlayOpts::sized(room.0, room.1)
                },
                |cx| {
                    let a = cx.area();
                    let catcher = cx.id();
                    let _ = cx.interact(catcher, a, Interest::CLICK.with(Interest::HOVER));
                },
            );
        }
        let carried = popup.list.offset;
        cx.overlay(
            id,
            area,
            OverlayOpts {
                z: opts.z,
                placement: opts.placement,
                ..OverlayOpts::sized(size.0, size.1)
            },
            move |cx| popup_body(cx, popup, options, chosen, search, shape, carried, id),
        );
    }

    resp
}

/// **The popup's body: the shell, then §5's collection over the option list.**
///
/// The one function every arm of the family goes through, so a gate playing a refused spelling is
/// playing the shipped drawing path with one field of [`SelectShape`] changed.
///
/// It writes [`PopupState`] and nothing else writes it. Two things are reported rather than asked for
/// — what it was granted, and whether the pointer was over it — because both are facts only the body
/// can know and the owner needs one frame later.
#[expect(
    clippy::too_many_arguments,
    reason = "the body's own five, the shape, the offset the refused `Holds` arm carries by value, \
              and the owner's id — which it needs in order to hand the keyboard over exactly once. \
              The extras exist so the refused arms are one field apart rather than two bodies"
)]
fn popup_body(
    cx: &mut Ctx<'_, '_>,
    popup: &mut PopupState,
    options: &[&str],
    chosen: usize,
    search: usize,
    shape: SelectShape,
    carried: i32,
    owner: Id,
) {
    let area = cx.area();
    let rows = u32::try_from(options.len()).unwrap_or(u32::MAX);
    let mut answer = None;
    let mut inside = false;

    // **The defect that declares before it looks.** `overlay` returns inert on an empty rectangle,
    // so the shipped arm below is never reached at `h = 0` and nothing is spent; this one spends the
    // entries first and looks afterwards. Both arms then fall through to the shell, which is what
    // makes the difference one branch rather than two bodies.
    if shape.closed == Closed::Declare {
        // **`cx.id()` and not a named root**, and the difference is §12's own sentence arriving
        // inside a body: two `select`s standing at once are two overlays whose bodies are one
        // function, so a named root gives both popups the same ids, `Ctx::interact` makes the
        // second claim **inert**, and the frame reports one popup's worth of entries where two
        // declared. Here the id stack is rooted at the *owner*, so one source line mints two ids.
        let blur = cx.id();
        let _ = cx.interact(blur, area, Interest::HOVER);
        let list = cx.id();
        let _ = cx.interact(list, area, Interest::CLICK.with(Interest::FOCUS));
    }

    let shell = overlay_with(
        cx,
        area,
        rows,
        &ShellOpts::default(),
        &mut |cx, interior| {
            // **The refused fill**, before a single row is written. Every non-blank cell the rows then
            // write is written twice and re-damaged for as long as the popup stands.
            if shape.fill == Fill::FillFirst {
                let paint = cx.theme().paint(Role::Body);
                let mut ink = Direct;
                for row in 0..interior.h {
                    let _ = ink.run(
                        cx,
                        interior.x,
                        interior.y + i32::from(row),
                        " ",
                        interior.w,
                        paint,
                    );
                }
            }
            // **The refused region spelling**: one entry and one stop a row, where §5 spends one for the
            // whole collection. The rows are still drawn by the collection below, so the two arms draw
            // exactly the same cells and only the counts differ.
            if shape.regions == PopupRegions::PerRow {
                // Rooted at `cx.id()` for the reason above, then keyed per row: one entry and one stop
                // a row, **on top of** the entry the collection below declares for all of them.
                let root = cx.id();
                for i in 0..interior.h {
                    let r = Rect::new(interior.x, interior.y + i32::from(i), interior.w, 1);
                    let _ = cx.interact(
                        Id::keyed(root, u64::from(i)),
                        r,
                        Interest::CLICK.with(Interest::FOCUS),
                    );
                }
            }
            let opts = CollOpts {
                mode: Mode::Single,
                search,
                tail: Role::Body,
            };
            // **First refusal, inside the one drain loop.** `Ctx::decline` hands a key back *and ends the
            // level's turn at the queue*, so a popup that read `Enter` before the collection would leave
            // it nothing and one that read it after would find the queue closed. `Esc` and `Enter` are
            // the popup's two, and `crate::nav::step` owns the rest.
            let mut mine = |k: &Pressed, cursor: usize| match k.code {
                Code::Enter => {
                    answer = Some(cursor);
                    true
                }
                Code::Escape => {
                    answer = Some(chosen);
                    true
                }
                _ => false,
            };
            let filled = shape.fill == Fill::FillFirst;
            // **What the list position lives in.** The shipped arm hands the collection the caller's own
            // store; the refused one hands it a scratch seeded from a `Copy` of the offset, which the
            // wheel then moves and the frame then drops.
            let mut scratch = CollState::new();
            scratch.offset = carried;
            let list = if shape.holds == Holds::ByMutRef {
                &mut popup.list
            } else {
                &mut scratch
            };
            let list_resp = collection_chorded(
                &mut Direct,
                cx,
                interior,
                list,
                &opts,
                Rows::of(options.len()),
                |buf, range: core::ops::Range<usize>| {
                    range.into_iter().find(|&i| options[i].starts_with(buf))
                },
                |ink: &mut Direct, cx: &mut Ctx<'_, '_>, r: Rect, i: usize, face: Face| {
                    let paint = face_paint(cx.theme(), face);
                    let mark = if i == chosen {
                        cx.theme().glyph(Glyph::Tick)
                    } else {
                        " "
                    };
                    let room = r.w.saturating_sub(MARK);
                    let (shown, tail) = crate::glyphs::elide(cx.theme(), options[i], room);
                    let ell = width(tail);
                    let head = ink.text(cx, r.x, r.y, mark, paint);
                    let _ = ink.text(cx, r.x + i32::from(head), r.y, " ", paint);
                    // **`pad_to` and not `text` then `run`.** A row is a partition of its width, and
                    // written as two verbs the counter separates a full row from a short one — which
                    // on the fill-first arm reports the *defect* as cheaper. Under a fill the pad has
                    // already been written, so the row writes its glyphs and nothing else.
                    //
                    // **And the pad stops where the ellipsis starts**, or the last cell of a
                    // truncated row is written twice — §2, on the one cell nobody looks at.
                    let body = room.saturating_sub(ell);
                    if filled {
                        let _ = ink.text(cx, r.x + i32::from(MARK), r.y, shown, paint);
                    } else {
                        let _ = ink.pad_to(cx, r.x + i32::from(MARK), r.y, shown, body, paint);
                    }
                    if ell > 0 {
                        let _ = ink.text(
                            cx,
                            r.x + i32::from(MARK) + i32::from(body),
                            r.y,
                            tail,
                            paint,
                        );
                    }
                },
                &mut mine,
            );
            // **A click on a row is a choice**, and it is the collection's own press edge rather than a
            // second hit entry per row: §5's *one hit entry per collection* is what keeps the closed
            // popup at `crate::popup::CLOSED_DECLARES` and not thirteen entries more.
            //
            // The **edge** and not `Response::clicked`: a row selects on the press, so by the time a
            // click has completed the collection has already moved its cursor and a release-driven
            // choice arrives a frame late.
            if list.press_edge() {
                answer = Some(list.sel.lead);
            }
            // **The popup takes the keyboard from its owner, exactly once.** `if cx.is_focused(owner)`
            // and never `if !cx.is_focused(list)`: the second drags the keyboard back every frame the
            // user has tabbed away, which is architecture issue 25's refused spelling one family over.
            // The list is what the arrows, the type-ahead and `Esc` are addressed to, and it is seated
            // a frame before the key it enables — `next_key` answers the *previous* frame's focus.
            if cx.is_focused(owner) {
                cx.focus(list_resp.id);
            }
            inside = cx.is_focused(list_resp.id);
        },
    );
    // **Reported, never asked for.** A popup sized from this is `(20, 0)` for ever.
    //
    // **`local` and not `hovered`**, and §12 says *a position* for exactly this reason:
    // `Response::hovered` is `hover_guess`, resolved from the **previous** frame's hit index, and the
    // frame that matters is the one the layer was placed on — the frame the optimistic focus arrives.
    // `local` is this frame's containment in this widget's own coordinates, computed from the pointer
    // that travelled down the `Ctx`, and it is exact on the first frame the popup exists.
    popup.saw((area.w, area.h), shell.response.local.is_some(), inside);
    if let Some(at) = answer {
        popup.answer(at);
        // **A body that answers through the inbox owes the frame that delivers it**, and this line is
        // the application's second finding. The owner reads the slot at the top of the *next* frame,
        // and an application parks on `Driver::wait` — so without a wake the choice lands on
        // whatever input happens next, which for the keystroke that made it means **never**. It is
        // `Driver::unhandled`'s own finding (C25) one layer over: the two-phase protocol puts the
        // answer a frame away, and a frame away is only a frame if somebody asks for it.
        //
        // Conditional, and that is the whole of why it is not a wake loop: a frame with nothing in
        // the slot asks for nothing, and the frame that delivers empties it.
        cx.request_frame();
    }
}

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// `slider` — spec §14's drag capture, and the row §17 froze at Tier 3
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **How many steps the keyboard divides the track into. A hundred.**
///
/// See [`SliderOpts::steps`] for why the keyboard's unit is an integer and the pointer's is not.
pub const SLIDER_STEPS: u32 = 100;

/// **How many of [`SLIDER_STEPS`] a page is. Ten.**
pub const SLIDER_PAGE: u32 = 10;

/// [`slider`]'s options.
///
/// Spec §1's rule 3: a `Default` struct, never a required builder.
///
/// # `orient` is the one field whose default is not the shared enum's
///
/// [`Orient`]'s own default is [`Orient::Vertical`], and its reason is written on it: *most content
/// is longer than it is wide*. That is a **scrollbar's** reason and a slider has no content at all —
/// a volume, a gamma, a seek position. So this default is [`Orient::Horizontal`] and the enum's is
/// left alone, because the day the two disagree the one to change is the caller that reads
/// `Orient::default()` for a slider.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SliderOpts {
    /// Which way it runs. Horizontal by default — see this type's own note.
    pub orient: Orient,
    /// The thumb's four faces, handed to [`crate::state::press`] whole.
    ///
    /// **The award is over the thumb and not over the track**, which is ADR 0026's relation rather
    /// than a decoration: a restyle is free only where the component's own next draw already
    /// produces what the restyle produced, and a hover face laid over the whole rectangle would
    /// wipe the [`SliderOpts::done`]/[`SliderOpts::rest`] distinction one frame and redraw it the
    /// next.
    pub faces: Faces,
    /// The stretch of track behind the thumb.
    pub done: Role,
    /// The stretch of track ahead of it.
    pub rest: Role,
    /// **How many steps the keyboard divides the track into**, and therefore what one arrow is
    /// worth: `1 / steps`.
    ///
    /// **It is a count and not a `f32` step, and that is the whole of this component's arithmetic
    /// finding.** A step added to a `f32` accumulates: fifty `Right`s from zero reach
    /// **0.4999998** rather than `0.5`, which on a 300-cell track is the thumb at column **149
    /// where 150 is the middle**; a hundred reach **0.99999934**, so a slider driven only from the
    /// keyboard **never reaches its own maximum** while the screen shows it at the far end. The two
    /// defects hide each other — the middle one is visible on the screen and right in the value's
    /// last digits, the end one is visible in the value and invisible on the screen. Stepping an
    /// integer index and dividing once makes both exact.
    /// [`defective::float_stepped`] is the spelling this replaced.
    pub steps: u32,
    /// How many of [`SliderOpts::steps`] `PageUp` and `PageDown` move over.
    pub page: u32,
    /// What it declares.
    ///
    /// **`Interest::SCROLL` is not in it, and its absence is a decision rather than an omission.**
    /// §17's row reads *a slider's value is not an offset and no wheel event moves it*; components
    /// ticket 20's finding is the other half — a widget that declares the wheel and consumes
    /// nothing is worse than one that declares nothing at all, because it is the topmost region
    /// over its rectangle and an enclosing `scroll_area` never sees the notch either.
    pub interest: Interest,
}

impl Default for SliderOpts {
    fn default() -> SliderOpts {
        SliderOpts {
            orient: Orient::Horizontal,
            faces: Faces::default(),
            done: Role::Ok,
            rest: Role::Dim,
            steps: SLIDER_STEPS,
            page: SLIDER_PAGE,
            interest: Interest::CLICK
                .with(Interest::DRAG)
                .with(Interest::HOVER)
                .with(Interest::FOCUS),
        }
    }
}

/// **Where along a slider's track the pointer is, while it is held — and nothing else.**
///
/// `None` when nothing is being dragged. `Some(0.0..=1.0)` on the press frame and on every frame the
/// button is held, which is what makes a press *jump* rather than start a delta.
///
/// # It is [`media::player::scrub`](crate::media::player::scrub) with an axis, and the two are gated
/// equal
///
/// Components ticket 30 measured drag capture over the video player's seek bar and wrote the
/// sentence this component exists to wrap: **the value is `Response::local` divided by
/// `Response::rect`**, with no press origin, no stored anchor and no *was I dragging last frame*.
/// That function reads `local.0` against `rect.w` and has no second axis, because a seek bar has no
/// second axis. This one takes the axis, and `tests::the_grab_is_the_chrome_s_own_on_the_axis_it_has`
/// sweeps the two against each other rather than trusting the transcription.
///
/// # The vertical arm is inverted and not transposed
///
/// A vertical slider's value grows **towards the top** — it is a volume, not an offset — so the
/// reading is `1 - local.1 / rect.h`. Transposing alone would give a slider whose value falls as the
/// pointer rises, which is a `scroll_area`'s convention arriving in a component that has no content
/// to scroll.
///
/// ```
/// use vitui_components::input::grab;
/// use vitui_components::scroll::Orient;
/// use vitui_runtime::{Id, Rect, Response};
///
/// let mut resp = Response::inert(Id::from_raw(1), Rect::new(0, 0, 300, 1));
/// // Nothing held: no answer, which is not the same as an answer of zero.
/// assert_eq!(grab(&resp, Orient::Horizontal), None);
///
/// resp.pressed = true;
/// resp.local = Some((20, 0));
/// // A press jumps to where it landed. §14's own fraction.
/// assert_eq!(grab(&resp, Orient::Horizontal), Some(20.0 / 299.0));
/// ```
#[must_use]
pub fn grab(resp: &Response, orient: Orient) -> Option<f32> {
    if !resp.pressed {
        return None;
    }
    let (lx, ly) = resp.local?;
    match orient {
        Orient::Horizontal => Some(along(lx, resp.rect.w)),
        // **Up is more.** See this function's own note.
        Orient::Vertical => Some(1.0 - along(ly, resp.rect.h)),
    }
}

/// One axis of [`grab`]: a local coordinate over the span it was resolved against.
///
/// The `max` pair is [`crate::media::player::scrub`]'s, character for character: a one-cell track
/// has a span of one rather than a division by zero, and an empty one is never reached because the
/// component returns before it.
fn along(local: i32, extent: u16) -> f32 {
    let span = i32::from(extent).max(1) - 1;
    let span = span.max(1);
    local.clamp(0, span) as f32 / span as f32
}

/// Which of [`SLIDER_STEPS`] a value sits on. **The keyboard's own unit.**
fn on_grid(value: f32, steps: u32) -> u32 {
    (fraction(value) * steps as f32).round() as u32
}

/// A value clamped into `0.0..=1.0`, with a non-finite one read as zero.
///
/// The guard is not defensive tidiness: `f32::NAN.clamp(0.0, 1.0)` is `NAN`, `NAN != NAN`, and
/// [`Response::changed`] is an inequality — so a caller who once handed in a `NaN` would own a
/// slider that reports a change on **every** frame for ever, which is the one thing spec §20's
/// *zero marked on a steady frame* cannot survive.
fn fraction(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// **The new value a cursor key moves a slider to**, or `None` when the key is not the slider's.
///
/// # `Up` is paired with `Right`, which is why [`crate::nav::step`] is not what this calls
///
/// `nav::step` is the cursor-key helper this crate already ships, and it pairs `Up` with `Left`
/// because it moves an **index into a list** and a list's index grows downward. A slider's value
/// grows *upward*: `Up` is more. The two therefore disagree at exactly four of the eight cursor
/// codes — `Up`, `Down`, `PageUp`, `PageDown` — and agree at `Left`, `Right`, `Home` and `End`,
/// which `tests::the_slider_s_pairing_is_not_a_list_s_and_the_disagreement_is_four_codes` counts.
///
/// It is components ticket 17's finding on the other side: there, a container could not read `←`
/// and `→` through `nav::step` because that helper reads them *as* `↑`/`↓`. Here the same pairing
/// is right for a list and wrong for a value.
///
/// **Both arrows act at both orientations**, so there is no orientation branch here at all. A
/// vertical slider that ignored `Right` would be a slider a user cannot find the keys for.
///
/// # The two refusals are the same two, read from the same place
///
/// A release is not a gesture — the engine pushes kitty flag 31 and bit 2 of that is *report event
/// types*, so on a terminal that speaks the protocol every arrow arrives twice — and a chord is not
/// a cursor key, which is [`crate::keys::is_chord`]. Both are stated here rather than inherited,
/// because the helper that states them is the one this function cannot call.
#[must_use]
pub fn stepped(k: &Pressed, value: f32, opts: &SliderOpts) -> Option<f32> {
    if k.kind == Edge::Release || keys::is_chord(k) {
        return None;
    }
    let steps = opts.steps.max(1);
    let page = opts.page.clamp(1, steps);
    let at = on_grid(value, steps);
    let moved = match k.code {
        // **More.** `Up` with `Right`, which is the pairing `nav::step` cannot express.
        Code::Right | Code::Up => at.saturating_add(1).min(steps),
        Code::Left | Code::Down => at.saturating_sub(1),
        Code::PageUp => at.saturating_add(page).min(steps),
        Code::PageDown => at.saturating_sub(page),
        Code::Home => 0,
        Code::End => steps,
        // `Code` is `#[non_exhaustive]`, so this arm is required rather than tidy.
        _ => return None,
    };
    // **One division, at the end.** Every intermediate value is an integer, so a hundred `Right`s
    // land on `steps / steps` and a round trip returns to `0 / steps` — both exactly.
    Some(moved as f32 / steps as f32)
}

/// **A value on a track, dragged from where the pointer is and stepped by the keyboard.** §17's
/// Tier 3 row, and the mechanism under it is [`grab`].
///
/// `value` is a fraction of the track, `0.0..=1.0`. **It is not a range**, and that is spec §9's
/// unit rule rather than a simplification: a component that owned a minimum and a maximum would own
/// a *unit*, and the one thing no layer of this library does for its caller is decide what a number
/// means. A slider over `0..=11` is `*v = f * 11.0` at the call site, in the caller's own unit.
///
/// ```
/// use vitui_components::input::slider;
/// use vitui_runtime::Rect;
/// use vitui_runtime::ctx::Driver;
///
/// let mut driver = Driver::headless(20, 1).expect("a sink attaches");
/// let mut volume = 0.5f32;
/// driver.frame(|cx| {
///     let resp = slider(cx, Rect::new(0, 0, 20, 1), &mut volume);
///     // Rule 4: a `Response` back, and nothing has happened to it on a frame with no input.
///     assert!(!resp.changed);
/// });
/// // One region and one tab stop, because `Interest::FOCUS` is in the default.
/// assert_eq!(driver.inspect().hits().len(), 1);
/// assert_eq!(driver.inspect().stop_count(), 1);
/// ```
#[track_caller]
pub fn slider(cx: &mut Ctx<'_, '_>, area: Rect, value: &mut f32) -> Response {
    slider_with(cx, area, value, &SliderOpts::default())
}

/// [`slider`], with the options spelled out.
#[track_caller]
pub fn slider_with(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    value: &mut f32,
    opts: &SliderOpts,
) -> Response {
    slider_into(&mut Direct, cx, area, value, opts)
}

/// **[`slider`], drawing through an [`Ink`] so a counter can see every cell.**
///
/// The entry point a gate takes; [`slider`] is this with [`Direct`].
#[track_caller]
pub fn slider_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    value: &mut f32,
    opts: &SliderOpts,
) -> Response {
    slider_shaped(ink, cx, area, value, opts, defective::Stepping::Grid)
}

/// **The one axis a slider can be false on that is not on [`SliderOpts`]**, threaded here so the
/// shipped build and the refused one are one function with one value between them.
#[track_caller]
fn slider_shaped<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    value: &mut f32,
    opts: &SliderOpts,
    stepping: defective::Stepping,
) -> Response {
    // **The id, taken outside every closure** (ADR 0027), and `#[track_caller]` all the way up so
    // that two sliders at two call sites are two widgets. `crate::media::player` shipped both
    // halves of that wrong once, on a screen that rendered perfectly.
    let id = cx.id();
    if area.is_empty() {
        return Response::inert(id, area);
    }
    let mut resp = cx.interact(id, area, opts.interest);
    // **The value as it *arrived*, unsanitised**, because that is what `Response::changed` is a
    // comparison against. Clamped first, a caller handing in `1.5` gets `changed == false` for ever
    // while the component silently rewrites the value under it — the clamp is a change and the
    // component is the one making it. Raw, it is reported exactly once, because the sanitised value
    // is written back.
    let before = *value;

    // **The grab, and there is no `cx.focus` beside it.** A press on a region that declared
    // `Interest::FOCUS` is *awarded* the focus by the runtime, from the index that has just drawn —
    // `crate::disclose`'s own finding. A component calling `focus` here would be a second producer
    // of the same award.
    let mut moved = grab(&resp, opts.orient).unwrap_or_else(|| fraction(before));

    // ── the keyboard, in one drain loop ──────────────────────────────────────────────────────────
    //
    // **One loop and not two.** `Ctx::decline` hands a key back *and ends the level's turn at the
    // queue* (`crate::collect`'s finding, components 17), so a widget that read some keys here and
    // the rest after would get nothing after the first refusal.
    while let Some(k) = cx.next_key(id) {
        // A release is dropped rather than declined — nobody wants one. `stepped` refuses it too;
        // the difference is that a declined release would end this loop and cost the arrow behind
        // it.
        if k.kind == Edge::Release {
            continue;
        }
        let next = match stepping {
            defective::Stepping::Grid => stepped(&k, moved, opts),
            defective::Stepping::Float => defective::float_stepped(&k, moved, opts),
        };
        if let Some(v) = next {
            moved = v;
            continue;
        }
        // **Not this widget's key.** The queue is ordered and shared, so a widget that declined *k*
        // and then took *k+1* would leave the level above seeing the two the wrong way round.
        cx.decline(k);
        break;
    }

    *value = fraction(moved);
    // **The runtime never sets this** — `Response::changed` is a component's own convention. See
    // `before` above for why the comparison is against the raw arrival and not the sanitised one.
    resp.changed = *value != before;
    draw_slider(ink, cx, area, *value, &resp, opts);
    resp
}

/// **The track: the thumb, then the stretch behind it, then the stretch ahead.** Returns the thumb.
///
/// # Thumb first, and it is [`crate::scroll::bar`]'s rule reached rather than restated
///
/// The verbs are [`crate::scroll::stripe`]'s — the same helper the scrollbar draws through — and the
/// order is the same order, which spec §3 gives that helper one job for: *the bar every scrollable
/// draws*, thumb before track. Everybody writes the groove and then the thumb on it; that reads
/// correctly, costs one verb fewer, and writes every cell of the thumb **twice**.
///
/// # Why `bar` itself is not called, and the difference is a count
///
/// A bar has **two** parts and a slider has **three**. [`crate::scroll::BarOpts`] carries one
/// `track` role, and a slider's groove is two — [`SliderOpts::done`] behind the thumb and
/// [`SliderOpts::rest`] ahead of it — so the shipped `bar` cannot express the picture at all. The
/// second half of the reason is the unit: a [`Span`](crate::scroll::Span) is three counts of
/// **content cells**, and a slider has no content and therefore no extent. Reaching `bar` would
/// mean inventing one out of the track's own width, which is the shape §9's unit rule exists to
/// refuse.
///
/// What *is* shared is the arithmetic's answer, and it is gated rather than asserted:
/// `tests::the_thumb_sits_where_scroll_thumb_would_put_it` sweeps this position against
/// [`crate::scroll::thumb`] at `Span { viewport: 1, extent: length, offset: at }` over every width
/// a track can have.
fn draw_slider<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    value: f32,
    resp: &Response,
    opts: &SliderOpts,
) -> Rect {
    let length = match opts.orient {
        Orient::Vertical => area.h,
        Orient::Horizontal => area.w,
    };
    let at = thumb_cell(value, length);
    // **The thumb's place in *track* order**, which is the value's own index one way round and its
    // complement the other. Everything below this line is orientation-free.
    let k = match opts.orient {
        Orient::Horizontal => at,
        Orient::Vertical => length - 1 - at,
    };
    let thumb_cells = crate::scroll::band(area, opts.orient, k, 1);

    // **The face and the award are one expression evaluated once** (`crate::state::press_into`'s own
    // criterion), and the cells are the **thumb's**. See [`SliderOpts::faces`].
    let face = crate::state::press_into(ink, cx, thumb_cells, resp, &opts.faces);

    let theme = cx.theme();
    let thumb_paint = theme.paint(face);
    let behind = theme.paint(opts.done);
    let ahead = theme.paint(opts.rest);
    let rule = theme.glyph(match opts.orient {
        Orient::Horizontal => Glyph::HLine,
        Orient::Vertical => Glyph::VLine,
    });
    let thumb = theme.glyph(Glyph::Thumb);
    // Behind the thumb is *below* it on a vertical track, because up is more.
    let (low, high) = match opts.orient {
        Orient::Horizontal => (behind, ahead),
        Orient::Vertical => (ahead, behind),
    };

    crate::scroll::stripe(ink, cx, area, opts.orient, k, 1, thumb, thumb_paint);
    crate::scroll::stripe(ink, cx, area, opts.orient, 0, k, rule, low);
    crate::scroll::stripe(
        ink,
        cx,
        area,
        opts.orient,
        k + 1,
        length - k - 1,
        rule,
        high,
    );
    thumb_cells
}

/// **Which cell of a track of `length` a value's thumb sits on**, counted from the value's own zero.
///
/// `length` is at least one, which the component's `is_empty` guard is what guarantees.
fn thumb_cell(value: f32, length: u16) -> u16 {
    let span = i32::from(length).max(1) - 1;
    let span = span.max(1);
    let at = (fraction(value) * span as f32).round() as u16;
    at.min(length - 1)
}

/// **Every spelling of `field`, of `select` and of `slider` that is refused, kept runnable.**
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

    /// **A `select` whose popup is sized from something other than the room the screen has.**
    ///
    /// One argument, three arms, and the two that are not [`Sizing::ToTheRoom`](super::Sizing::ToTheRoom) are §12's:
    /// [`Sizing::ToTheContent`](super::Sizing::ToTheContent) leaves
    /// [`SPEC_UNREACHABLE`](crate::overlay::SPEC_UNREACHABLE) of
    /// [`SHORT_OPTIONS`](crate::overlay::SHORT_OPTIONS) rows reachable by nothing, and
    /// [`Sizing::FromTheDrawnExtent`](super::Sizing::FromTheDrawnExtent) is granted
    /// [`SPEC_GRANTED_FROM_EXTENT`](crate::overlay::SPEC_GRANTED_FROM_EXTENT) for ever.
    #[expect(
        clippy::too_many_arguments,
        reason = "the shipped signature plus the one arm, so the two are one call apart"
    )]
    pub fn sized<'f, I: crate::ink::Ink>(
        ink: &mut I,
        cx: &mut vitui_runtime::Ctx<'f, '_>,
        id: vitui_runtime::Id,
        area: vitui_runtime::Rect,
        st: &mut super::SelectState,
        popup: &'f mut crate::overlay::PopupState,
        options: &'f [&'f str],
        opts: &super::SelectOpts,
        sizing: super::Sizing,
    ) -> vitui_runtime::Response {
        super::select_shaped(
            ink,
            cx,
            id,
            area,
            st,
            popup,
            options,
            opts,
            super::SelectShape {
                sizing,
                ..Default::default()
            },
        )
    }

    /// **A `select` whose body holds its list position in a `Copy` of the offset.**
    ///
    /// §7's literal `Copy`-only body, one family over: [`crate::popup::WHEEL_CLICKS`] notches move
    /// the offset **0**, the screen is identical while it happens, and every counter agrees.
    #[expect(
        clippy::too_many_arguments,
        reason = "the shipped signature plus the one arm, so the two are one call apart"
    )]
    pub fn a_copy_of_the_offset<'f, I: crate::ink::Ink>(
        ink: &mut I,
        cx: &mut vitui_runtime::Ctx<'f, '_>,
        id: vitui_runtime::Id,
        area: vitui_runtime::Rect,
        st: &mut super::SelectState,
        popup: &'f mut crate::overlay::PopupState,
        options: &'f [&'f str],
        opts: &super::SelectOpts,
    ) -> vitui_runtime::Response {
        super::select_shaped(
            ink,
            cx,
            id,
            area,
            st,
            popup,
            options,
            opts,
            super::SelectShape {
                holds: super::Holds::CopyOfTheOffset,
                ..Default::default()
            },
        )
    }

    /// **A `select` that qualifies a blur by something other than a position.**
    ///
    /// [`Blur::Position`](crate::overlay::Blur::Position) is the rule;
    /// [`Blur::Press`](crate::overlay::Blur::Press) asks for an edge that has not happened yet on the
    /// frame that matters, and [`Blur::Catcher`](crate::overlay::Blur::Catcher) reports it from a
    /// second layer that swallows it.
    #[expect(
        clippy::too_many_arguments,
        reason = "the shipped signature plus the one arm, so the two are one call apart"
    )]
    pub fn blurred<'f, I: crate::ink::Ink>(
        ink: &mut I,
        cx: &mut vitui_runtime::Ctx<'f, '_>,
        id: vitui_runtime::Id,
        area: vitui_runtime::Rect,
        st: &mut super::SelectState,
        popup: &'f mut crate::overlay::PopupState,
        options: &'f [&'f str],
        opts: &super::SelectOpts,
        blur: crate::overlay::Blur,
    ) -> vitui_runtime::Response {
        super::select_shaped(
            ink,
            cx,
            id,
            area,
            st,
            popup,
            options,
            opts,
            super::SelectShape {
                blur,
                ..Default::default()
            },
        )
    }

    /// **A `select` whose popup fills its rectangle before it writes a row.**
    ///
    /// The cells are identical and one verb a row cheaper; what moves is the re-damage —
    /// [`crate::popup::FILL_FIRST`] cells on every steady frame the popup stands.
    #[expect(
        clippy::too_many_arguments,
        reason = "the shipped signature plus the one arm, so the two are one call apart"
    )]
    pub fn fill_first<'f, I: crate::ink::Ink>(
        ink: &mut I,
        cx: &mut vitui_runtime::Ctx<'f, '_>,
        id: vitui_runtime::Id,
        area: vitui_runtime::Rect,
        st: &mut super::SelectState,
        popup: &'f mut crate::overlay::PopupState,
        options: &'f [&'f str],
        opts: &super::SelectOpts,
    ) -> vitui_runtime::Response {
        super::select_shaped(
            ink,
            cx,
            id,
            area,
            st,
            popup,
            options,
            opts,
            super::SelectShape {
                fill: super::Fill::FillFirst,
                ..Default::default()
            },
        )
    }

    /// **A `select` whose popup declares one hit entry and one tab stop a row.**
    ///
    /// §5's *one hit entry per collection* from the other side:
    /// [`crate::popup::PER_ROW_ENTRIES`] of each where the rule spends one, over cells that are
    /// identical.
    #[expect(
        clippy::too_many_arguments,
        reason = "the shipped signature plus the one arm, so the two are one call apart"
    )]
    pub fn per_row<'f, I: crate::ink::Ink>(
        ink: &mut I,
        cx: &mut vitui_runtime::Ctx<'f, '_>,
        id: vitui_runtime::Id,
        area: vitui_runtime::Rect,
        st: &mut super::SelectState,
        popup: &'f mut crate::overlay::PopupState,
        options: &'f [&'f str],
        opts: &super::SelectOpts,
    ) -> vitui_runtime::Response {
        super::select_shaped(
            ink,
            cx,
            id,
            area,
            st,
            popup,
            options,
            opts,
            super::SelectShape {
                regions: super::PopupRegions::PerRow,
                ..Default::default()
            },
        )
    }

    /// **A `select` whose popup declares before it looks at what it was granted.**
    ///
    /// Paired with [`Sizing::FromTheDrawnExtent`](super::Sizing::FromTheDrawnExtent) this is §12's
    /// closed-popup case: `h = 0`, eighty rows of screen identical either way, and
    /// [`crate::popup::CLOSED_DECLARES`] spent on entries nobody can reach.
    #[expect(
        clippy::too_many_arguments,
        reason = "the shipped signature plus the one arm, so the two are one call apart"
    )]
    pub fn declares_at_zero<'f, I: crate::ink::Ink>(
        ink: &mut I,
        cx: &mut vitui_runtime::Ctx<'f, '_>,
        id: vitui_runtime::Id,
        area: vitui_runtime::Rect,
        st: &mut super::SelectState,
        popup: &'f mut crate::overlay::PopupState,
        options: &'f [&'f str],
        opts: &super::SelectOpts,
        sizing: super::Sizing,
    ) -> vitui_runtime::Response {
        super::select_shaped(
            ink,
            cx,
            id,
            area,
            st,
            popup,
            options,
            opts,
            super::SelectShape {
                sizing,
                closed: super::Closed::Declare,
                ..Default::default()
            },
        )
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

    // ── `slider`'s two refused spellings ─────────────────────────────────────────────────────────

    /// **What one arrow adds to a slider.**
    ///
    /// One argument, two arms, and the second is §14's arithmetic defect rather than §14's prose.
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
    pub enum Stepping {
        /// **The rule.** An integer index on a grid of [`SliderOpts::steps`](super::SliderOpts::steps),
        /// divided once at the end.
        #[default]
        Grid,
        /// **The defect.** `1 / steps` added to the value, which accumulates — see
        /// [`float_stepped`].
        Float,
    }

    /// **A slider whose arrow adds `1 / steps` to the value.**
    ///
    /// The two magnitudes, both measured in `crate::input::tests`:
    ///
    /// | from `0.0` | grid | float |
    /// |---|---|---|
    /// | fifty `Right`s | `0.5` | **`0.4999998`** — thumb at column **149 of a 300-cell track where 150 is the middle** |
    /// | a hundred `Right`s | `1.0` | **`0.99999934`** — a slider that never reaches its own maximum |
    /// | a hundred up, a hundred down | `0.0` | **`1.4901161e-8`** |
    ///
    /// **The two hide each other**, which is why this shipped in three prototypes before anybody
    /// looked: at the ends the value is wrong and the screen is right — `round(0.99999934 × 299)` is
    /// `299`, the last cell — and in the middle the screen is wrong and the value looks right to
    /// every digit a status row prints.
    #[must_use]
    pub fn float_stepped(
        k: &vitui_runtime::keys::Pressed,
        value: f32,
        opts: &super::SliderOpts,
    ) -> Option<f32> {
        if k.kind == vitui_runtime::keys::Edge::Release || crate::keys::is_chord(k) {
            return None;
        }
        let steps = opts.steps.max(1);
        let step = 1.0 / steps as f32;
        let page = f32::from(u16::try_from(opts.page.clamp(1, steps)).unwrap_or(u16::MAX)) * step;
        use vitui_runtime::keys::Code;
        let moved = match k.code {
            Code::Right | Code::Up => value + step,
            Code::Left | Code::Down => value - step,
            Code::PageUp => value + page,
            Code::PageDown => value - page,
            Code::Home => 0.0,
            Code::End => 1.0,
            _ => return None,
        };
        Some(moved.clamp(0.0, 1.0))
    }

    /// **[`slider`](super::slider) on the float step**, so the two arms are one call apart.
    #[track_caller]
    pub fn float_stepped_into<I: crate::ink::Ink>(
        ink: &mut I,
        cx: &mut vitui_runtime::Ctx<'_, '_>,
        area: vitui_runtime::Rect,
        value: &mut f32,
        opts: &super::SliderOpts,
    ) -> vitui_runtime::Response {
        super::slider_shaped(ink, cx, area, value, opts, Stepping::Float)
    }

    /// **Why there is no range slider, and the shape that would avoid the missing fact.**
    ///
    /// Ticket criterion 4 is *a range slider works on the same mechanism, or is explicitly not
    /// shipped with the reason recorded*. It is not shipped, and the reason is that **it is the one
    /// construction on this map that needs the fact §14 measured the absence of.**
    ///
    /// [`grab`](super::grab) is `Response::local` over `Response::rect` and nothing else. Two thumbs
    /// on one track need a third answer that neither field carries: *which* thumb. Every way to get
    /// it is one of two:
    ///
    /// 1. **Remember which one the press landed nearer** — a press origin, which is exactly the
    ///    fifth cross-frame fact §14's headline is that drag capture does not need. It works, and it
    ///    makes the range slider a component with a mechanism no other component here has.
    /// 2. **Derive the boundary from the two values**, so nothing is stored. This is [`Range`], and
    ///    it does not work: because the boundary moves with the values it separates, **one
    ///    continuous drag moves both of them**. Measured, from `(0.2, 0.8)` with the pointer moving
    ///    `0.3 → 0.6`: the low thumb goes `0.2 → 0.3` and then the *high* thumb is yanked
    ///    `0.8 → 0.6` — a jump of **0.2 on a thumb nobody touched**, in the middle of a gesture that
    ///    never released.
    ///
    /// Two thumbs as **two widgets** is not a third answer: `Response::local` is in the coordinates
    /// of the rectangle it was declared with, so a one-cell thumb's own `local` is one cell wide and
    /// says nothing about the track — and two overlapping regions resolve the pointer to the topmost
    /// one, so a thumb and its track cannot both have it.
    ///
    /// **Recorded rather than decided here**: which of the two a range slider should cost is a
    /// question for the ticket that builds one, and neither answer is a new *mechanism* — 1 is a
    /// stored `bool` and 2 is arithmetic. What this file settles is that the shipped `slider` reads
    /// two fields, and a second thumb is not free.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct WhyThereIsNoRangeSlider;

    /// Which thumb of a [`Range`] a pointer is moving.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Which {
        /// The lower of the two.
        Low,
        /// The upper of the two.
        High,
    }

    /// **The range slider that stores nothing, and therefore moves two values with one drag.**
    ///
    /// See [`WhyThereIsNoRangeSlider`]. Kept runnable rather than described, because *the boundary
    /// is derived so there is nothing to keep in sync* is a sentence that reads like a design and
    /// costs a defect.
    #[derive(Clone, Copy, PartialEq, Debug)]
    pub struct Range {
        /// The lower value.
        pub low: f32,
        /// The upper value.
        pub high: f32,
    }

    impl Range {
        /// Which thumb a pointer at `at` moves, decided by the midpoint of the two values.
        #[must_use]
        pub fn which(self, at: f32) -> Which {
            if at < (self.low + self.high) / 2.0 {
                Which::Low
            } else {
                Which::High
            }
        }

        /// Move whichever thumb [`Range::which`] names to `at`. **The frame of a held drag.**
        pub fn drag(&mut self, at: f32) -> Which {
            let which = self.which(at);
            match which {
                Which::Low => self.low = at,
                Which::High => self.high = at,
            }
            which
        }
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

#[cfg(test)]
mod slider_tests {
    use super::defective::{Range, Stepping, Which, float_stepped, float_stepped_into};
    use super::*;
    use crate::counters::Tally;
    use crate::runner::{Canvas, Pen};
    use vitui_runtime::ctx::Driver;
    use vitui_runtime::keys::Pressed;
    use vitui_runtime::{Button, Buttons, Id, Mods, Mouse, MouseKind, Notch};

    /// **The screen the drag is played on: `300` wide**, so the span is **299** and §14's two
    /// fractions are the fractions it wrote down.
    const W: u16 = 300;

    /// The row the track is on.
    const ROW: u16 = 0;

    /// A pointer event at `x` on the track's row.
    fn at(x: u16, kind: MouseKind) -> Mouse {
        Mouse {
            x,
            y: ROW,
            kind,
            buttons: Buttons::NONE,
            mods: Mods::NONE,
            at: std::time::Instant::now(),
        }
    }

    /// A key press with no modifiers.
    fn key(code: Code) -> Pressed {
        crate::keys::press_with(code, Mods::NONE)
    }

    /// The **release** edge of `code`. `crate::keys::press_with` only builds a press, and the
    /// release is half of what `stepped` refuses.
    fn release(code: Code) -> Pressed {
        Pressed {
            kind: Edge::Release,
            ..crate::keys::press_with(code, Mods::NONE)
        }
    }

    /// A tally over one frame of `f`, on a `w` by `h` sink.
    fn tallied(w: u16, h: u16, f: impl FnOnce(&mut Tally, &mut Ctx<'_, '_>)) -> Tally {
        let mut driver = Driver::headless(w, h).expect("a sink cannot fail to attach");
        let mut tally = Tally::new();
        driver.frame(|cx| f(&mut tally, cx));
        tally
    }

    /// The eight cursor codes a slider and a list both read.
    const CURSOR: [Code; 8] = [
        Code::Left,
        Code::Right,
        Code::Up,
        Code::Down,
        Code::PageUp,
        Code::PageDown,
        Code::Home,
        Code::End,
    ];

    /// **Criterion 1: three spellings, one function, and rule 4 comes back from all of them.**
    ///
    /// The shape is spec §1's four rules, and the half a test can reach is that `slider`,
    /// `slider_with` and `slider_into` are the *same* draw: the options struct is a `Default` and
    /// never a required builder, so the ninety-per-cent spelling has to be reachable without naming
    /// one — and the `_into` seam has to draw the same cells, or every gate below is measuring a
    /// second implementation.
    #[test]
    fn the_three_spellings_are_one_component_and_each_answers_with_a_response() {
        let opts = SliderOpts::default();
        let mut painted: Vec<Canvas> = Vec::new();
        for arm in 0..3u8 {
            let mut driver = Driver::headless(40, 1).expect("a sink attaches");
            let mut pen = Pen::over(Canvas::new(40, 1));
            let mut value = 0.375f32;
            let mut changed = true;
            driver.frame(|cx| {
                let area = cx.area();
                let resp = match arm {
                    // Rule 3: the ninety-per-cent spelling names no options at all.
                    0 => slider(cx, area, &mut value),
                    1 => slider_with(cx, area, &mut value, &opts),
                    _ => slider_into(&mut pen, cx, area, &mut value, &opts),
                };
                // Rule 4: a `Response`, and nothing has happened on a frame with no input.
                assert_eq!(resp.rect.w, 40);
                changed = resp.changed;
            });
            assert!(!changed, "arm {arm} reported a change nobody made");
            // Rule 2: the state is the caller's `&mut f32` and the component did not move it.
            assert_eq!(value, 0.375);
            if arm == 2 {
                painted.push(pen.into_canvas());
            }
        }

        // And the seam draws what `Direct` draws: the same frame through a `Pen` and through the
        // shipped path, compared cell for cell.
        let mut driver = Driver::headless(40, 1).expect("a sink attaches");
        let mut pen = Pen::over(Canvas::new(40, 1));
        let mut value = 0.375f32;
        driver.frame(|cx| {
            let area = cx.area();
            slider_into(&mut pen, cx, area, &mut value, &opts);
        });
        assert_eq!(
            pen.into_canvas().diff(&painted.remove(0)).cells,
            0,
            "the `_into` seam is not the shipped draw"
        );
    }

    /// **Criterion 2, the source half: the component stores no press origin and no drag-phase
    /// field.**
    ///
    /// A **scan** and not a type assertion, because what is being refused has no expression: *there
    /// is no field called `anchor`* cannot be written as Rust, and a `compile_fail` naming a type
    /// that was never declared passes today and passes again the day somebody declares it under
    /// another name. The predicate is `crate::dense::declares`, which is one definition of *a line
    /// that is not a comment* and is shared with every other scan in this crate.
    ///
    /// **The positive half is in the same test**, because a scan for absences alone goes green when
    /// the whole section is deleted: the four things it must *not* mint are checked beside the three
    /// it must have.
    #[test]
    fn the_slider_mints_no_press_origin_and_no_drag_phase_field() {
        let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/input.rs"))
            .expect("this file");
        // Only the slider's own half of the file: `select` legitimately holds a popup's state.
        let from = source
            .find("// `slider` — spec §14's drag capture")
            .expect("the slider's own banner");
        let to = source[from..]
            .find("#[cfg(test)]")
            .map_or(source.len(), |at| from + at);
        // **The slice ends at the test module**, or every positive needle below is satisfied by
        // this test's own source and the scan is watching itself from the other direction.
        let slider_half = &source[from..to];

        // **The needles are assembled from fragments, because a scanner looking for a literal
        // contains that literal.** `crate::frame`'s own first run reported the module it defends,
        // and this one reported `press_origin` on its own line in this test. Concatenating is the
        // established answer: the fragments are not the string, so the file does not match itself.
        let forbidden = [
            format!("press{}origin", "_"),
            format!("anc{}hor", ""),
            format!("was{}dragging", "_"),
            format!("struct Slider{}", "State"),
        ];
        for forbidden in &forbidden {
            assert!(
                !crate::dense::declares(slider_half, forbidden),
                "the slider mints `{forbidden}`, and §14's claim is that the grab needs no fifth \
                 cross-frame fact"
            );
        }
        for owed in [
            "pub fn grab(resp: &Response, orient: Orient) -> Option<f32>",
            "let (lx, ly) = resp.local?;",
            "pub fn slider(cx: &mut Ctx<'_, '_>, area: Rect, value: &mut f32) -> Response",
        ] {
            assert!(
                crate::dense::declares(slider_half, owed),
                "`{owed}` is not in the shipped slider, so the scan above is watching an empty \
                 file"
            );
        }
        // And the state a caller holds is four bytes with no slot beside it — `crate::disclose`'s
        // `Collapse` pair from the other end, where the 4 is live and the 48 is the slot.
        assert_eq!(std::mem::size_of::<f32>(), 4);
    }

    /// **Criterion 3: press `20/299`, move `60/299`, release unchanged — through the shipped
    /// component, from a posted pointer.**
    ///
    /// §14's own three fractions, and the cadence had to be written around exactly as
    /// `crate::media::player`'s did: `Response::pressed` is `frame.grab == id` and the grab is
    /// awarded at `end` from the index that has just drawn, so the frame that *delivers* the `Down`
    /// reads `false`. A gate playing one frame a phase would have measured the cadence and called
    /// it the mechanism.
    #[test]
    fn a_posted_press_jumps_a_posted_move_carries_and_the_release_moves_nothing() {
        let mut driver = Driver::headless(W, 1).expect("a sink attaches");
        let mut value = 0.0f32;
        let mut changed = false;
        let frame = |driver: &mut Driver, value: &mut f32, changed: &mut bool| {
            driver.frame(|cx| {
                let area = cx.area();
                *changed = slider(cx, area, value).changed;
            });
        };

        // The pointer arrives, and nothing is held.
        driver.post_mouse(at(20, MouseKind::Move));
        frame(&mut driver, &mut value, &mut changed);
        assert_eq!(value, 0.0, "a hover moved the value");

        // The press. Two frames, because the grab is awarded at the end of the one that delivers it.
        driver.post_mouse(at(20, MouseKind::Down(Button::Left)));
        frame(&mut driver, &mut value, &mut changed);
        frame(&mut driver, &mut value, &mut changed);
        assert!(
            (value - 20.0 / 299.0).abs() < 1e-6,
            "a press jumped to {value} rather than to 20/299"
        );

        // The move carries it, without a press origin anywhere.
        driver.post_mouse(at(60, MouseKind::Move));
        frame(&mut driver, &mut value, &mut changed);
        let moved = value;
        assert!(
            (moved - 60.0 / 299.0).abs() < 1e-6,
            "a move carried to {moved} rather than to 60/299"
        );
        assert!(changed, "the component did not report the move");

        // The release ends the scrub and moves nothing.
        driver.post_mouse(at(60, MouseKind::Up(Button::Left)));
        frame(&mut driver, &mut value, &mut changed);
        frame(&mut driver, &mut value, &mut changed);
        assert_eq!(value, moved, "the release moved the value");
        assert!(!changed, "a released slider still reports a change");
    }

    /// **The grab is the chrome's own on the axis the chrome has**, swept rather than transcribed.
    ///
    /// [`crate::media::player::scrub`] is where §14 measured drag capture, and it has one axis
    /// because a seek bar has one. This component's [`grab`] takes the axis, and the horizontal arm
    /// must be that function — checked over every width a track can have and every local position
    /// including both overruns, because a transcription that agreed at `20/299` and disagreed at the
    /// ends would pass criterion 3 and be wrong everywhere else.
    #[test]
    fn the_grab_is_the_chromes_own_on_the_axis_it_has() {
        for w in [1u16, 2, 3, 20, 40, 299, 300, 1_000] {
            for local in [-9i32, -1, 0, 1, 7, 20, 60, 149, 150, 298, 299, 300, 4_000] {
                let mut resp = Response::inert(Id::from_raw(1), Rect::new(0, 0, w, 1));
                resp.pressed = true;
                resp.local = Some((local, 0));
                assert_eq!(
                    grab(&resp, Orient::Horizontal),
                    crate::media::player::scrub(&resp),
                    "{w} wide at {local}: the slider's grab is not the chrome's"
                );
            }
        }
        // And neither answers at all while nothing is held, which is not the same as answering zero.
        let resp = Response::inert(Id::from_raw(1), Rect::new(0, 0, 300, 1));
        assert_eq!(grab(&resp, Orient::Horizontal), None);
        assert_eq!(grab(&resp, Orient::Vertical), None);
    }

    /// **The vertical arm is inverted and not transposed: up is more.**
    ///
    /// A vertical slider's value grows towards the top. Transposing alone gives a slider whose value
    /// *falls* as the pointer rises, which is a `scroll_area`'s convention arriving in a component
    /// with no content to scroll — and it is invisible in a screenshot, because the thumb is in the
    /// right place for the wrong value on exactly one frame and in the wrong place ever after.
    #[test]
    fn the_vertical_arm_is_inverted_and_not_transposed() {
        let mut resp = Response::inert(Id::from_raw(1), Rect::new(0, 0, 1, 11));
        resp.pressed = true;
        // The top row is the maximum and the bottom row is the minimum.
        resp.local = Some((0, 0));
        assert_eq!(grab(&resp, Orient::Vertical), Some(1.0));
        resp.local = Some((0, 10));
        assert_eq!(grab(&resp, Orient::Vertical), Some(0.0));
        resp.local = Some((0, 5));
        assert_eq!(grab(&resp, Orient::Vertical), Some(0.5));

        // And the transpose — the spelling that reads the axis and forgets the direction — is the
        // complement of it at every row but the middle.
        let mut disagreed = 0;
        for y in 0..11i32 {
            resp.local = Some((0, y));
            let shipped = grab(&resp, Orient::Vertical).expect("held");
            let transposed = 1.0 - shipped;
            if (shipped - transposed).abs() > f32::EPSILON {
                disagreed += 1;
            }
        }
        assert_eq!(disagreed, 10, "eleven rows, and only the middle one agrees");
    }

    /// **Criterion 5: `writes == distinct == every cell`, at both orientations and every shape the
    /// arithmetic runs out on.**
    ///
    /// Spec §2's two equalities. The values are chosen to put the thumb against both ends and in the
    /// middle, because the two stretches either side of it are what a `0` count in [`crate::scroll::stripe`]
    /// makes disappear.
    #[test]
    fn a_slider_writes_a_partition_of_its_whole_rectangle() {
        for orient in [Orient::Horizontal, Orient::Vertical] {
            for (w, h) in [
                (1u16, 1u16),
                (2, 1),
                (1, 2),
                (3, 3),
                (20, 1),
                (1, 20),
                (40, 3),
            ] {
                for value in [0.0f32, 0.001, 0.5, 0.999, 1.0, f32::NAN, -2.0, 7.0] {
                    let opts = SliderOpts {
                        orient,
                        ..SliderOpts::default()
                    };
                    let mut v = value;
                    let tally = tallied(w, h, |tally, cx| {
                        slider_into(tally, cx, Rect::new(0, 0, w, h), &mut v, &opts);
                    });
                    assert_eq!(
                        tally.writes(),
                        tally.distinct(),
                        "{orient:?} {w}x{h} at {value}: a cell written twice"
                    );
                    assert_eq!(
                        tally.distinct(),
                        u64::from(w) * u64::from(h),
                        "{orient:?} {w}x{h} at {value}: the slider does not cover its own track"
                    );
                    assert_eq!(
                        tally.asked(),
                        tally.reported(),
                        "{orient:?} {w}x{h} at {value}: a verb left the rectangle"
                    );
                }
            }
        }
    }

    /// **Criterion 5, the steady half: a slider nobody touches changes 0 cells a frame — and the
    /// warm-up is two frames rather than one, because the hover arrives a frame after the pointer.**
    ///
    /// `marked` is the engine's own counter and is unreachable from this crate
    /// ([`crate::counters::Counters::marked`]), so the reachable form is register row 48's: carry the
    /// surface across frames with [`Pen::over`] and count the cells whose *value* changed.
    ///
    /// # The warming discipline, on the hover axis
    ///
    /// The sequence is `[40, 1, 0, 0, …]` and **the 1 is not slack**. `Response::hovered` is
    /// `frame.hover_guess == id`, resolved from the index the *previous* frame built, so the frame
    /// the pointer arrives on draws the thumb at rest and declares an award nothing applies; the
    /// frame after it draws the thumb hovered and applies the award to the same value. One cell
    /// changes once, and then nothing changes ever again.
    ///
    /// Written as a single total over 59 frames this read **1** and looked like a defect. It is
    /// components ticket 22's warming discipline — *every allocation window in this workspace warms
    /// with two identical frames* — arriving on a counter rather than on an allocator, and the honest
    /// shape is the sequence: a gate that averaged it away would also hide a thumb that really did
    /// re-paint once a frame.
    ///
    /// What the 0 after it is evidence *of* is ADR 0026's relation — *a restyle is free only where
    /// the component's own next draw already produces the value the restyle produced* — and it is why
    /// [`SliderOpts::faces`] awards over the thumb and not over the track. Awarded over the whole
    /// rectangle the award paints 40 cells the next draw immediately contradicts, and the steady
    /// figure is the width of the track for as long as a pointer rests on it.
    #[test]
    fn a_slider_nobody_touches_changes_nothing_after_its_first_two_frames() {
        const FRAMES: usize = 60;
        const W_TRACK: u64 = 40;
        let mut driver = Driver::headless(40, 1).expect("a sink attaches");
        let mut value = 0.375f32;
        let mut canvas = Canvas::new(40, 1);
        let mut per_frame: Vec<u64> = Vec::with_capacity(FRAMES);
        // The pointer rests on the track for the whole run, so the hover award is declared on every
        // frame — which is the arm that is *not* free when the award and the draw disagree.
        driver.post_mouse(at(14, MouseKind::Move));
        for _ in 0..FRAMES {
            let mut pen = Pen::over(canvas);
            driver.frame(|cx| {
                let area = cx.area();
                slider_into(&mut pen, cx, area, &mut value, &SliderOpts::default());
            });
            pen.end_frame();
            canvas = pen.into_canvas();
            per_frame.push(canvas.take_repaints());
        }
        assert_eq!(
            per_frame[0], W_TRACK,
            "the first frame paints every cell of the track"
        );
        assert_eq!(
            per_frame[1], 1,
            "the hover arrives a frame after the pointer, and it is the thumb's one cell"
        );
        assert_eq!(
            per_frame[2..].iter().sum::<u64>(),
            0,
            "a slider nobody touched re-painted cells over {} settled frames: {:?}",
            FRAMES - 2,
            &per_frame[2..]
        );
        // And the value nobody moved is the value it started at.
        assert_eq!(value, 0.375);
    }

    /// **Criterion 6: the thumb sits where [`crate::scroll::thumb`] would put it, at every width.**
    ///
    /// The bar is drawn thumb-first through the scrollbar's own [`crate::scroll::stripe`], and `bar`
    /// itself is not called for two reasons that are both counts — three stretches against two, and
    /// a [`Span`](crate::scroll::Span) that is content cells where a slider has no content. What is
    /// shared is the *answer*, and this is the sweep that keeps it shared: a one-cell thumb at
    /// `Span { viewport: 1, extent: length, offset: at }` is the same cell this component draws it
    /// on, over every track a terminal can hold.
    #[test]
    fn the_thumb_sits_where_scroll_thumb_would_put_it() {
        for length in 1u16..=200 {
            let span = i32::from(length).max(1) - 1;
            let span = span.max(1);
            for at in 0..=u32::from(length.saturating_sub(1)).max(1) {
                let value = at as f32 / span as f32;
                let ours = super::thumb_cell(value, length);
                let (start, len) = crate::scroll::thumb(
                    length,
                    crate::scroll::Span {
                        viewport: 1,
                        extent: u32::from(length),
                        offset: at,
                    },
                );
                assert_eq!(len, 1, "a {length}-cell track's thumb is one cell");
                assert_eq!(
                    ours, start,
                    "{length} cells at step {at}: the slider puts the thumb on {ours} and \
                     `scroll::thumb` on {start}"
                );
            }
        }
    }

    /// **The keyboard's grid is exact and the float step is not — and the two defects hide each
    /// other.**
    ///
    /// [`SliderOpts::steps`]'s own note as three numbers. The middle one is the interesting one: at
    /// fifty `Right`s the *value* looks right to every digit a status row prints and the **thumb is
    /// one cell short of the middle of a 300-cell track**; at a hundred the thumb is in exactly the
    /// right place and the value **never reaches its own maximum**, so a caller testing `v == 1.0`
    /// has a branch that can never be taken.
    #[test]
    fn fifty_rights_land_on_the_middle_and_the_float_step_lands_one_cell_short() {
        let opts = SliderOpts::default();
        let press = |mut v: f32, code: Code, n: u32, arm: Stepping| {
            for _ in 0..n {
                v = match arm {
                    Stepping::Grid => stepped(&key(code), v, &opts),
                    Stepping::Float => float_stepped(&key(code), v, &opts),
                }
                .expect("a cursor key is the slider's");
            }
            v
        };

        // Fifty `Right`s: the middle, and one cell short of it.
        let grid = press(0.0, Code::Right, 50, Stepping::Grid);
        let float = press(0.0, Code::Right, 50, Stepping::Float);
        assert_eq!(grid, 0.5);
        assert_eq!(float, 0.4999998);
        assert_eq!(super::thumb_cell(grid, W), 150);
        assert_eq!(
            super::thumb_cell(float, W),
            149,
            "the float step's thumb is not one cell short of the middle"
        );

        // A hundred: the maximum, and a value that never gets there.
        assert_eq!(press(0.0, Code::Right, 100, Stepping::Grid), 1.0);
        assert_eq!(press(0.0, Code::Right, 100, Stepping::Float), 0.99999934);
        // And here the screen agrees with the defect, which is the half that hid it.
        assert_eq!(super::thumb_cell(0.99999934, W), 299);
        assert_eq!(super::thumb_cell(1.0, W), 299);

        // The round trip. The grid returns to zero exactly; the float does not, and the thumb is on
        // the same cell either way.
        let there_and_back = |arm: Stepping| {
            let up = press(0.0, Code::Right, 100, arm);
            press(up, Code::Left, 100, arm)
        };
        assert_eq!(there_and_back(Stepping::Grid), 0.0);
        assert_eq!(there_and_back(Stepping::Float), 1.4901161e-8);
        assert_eq!(super::thumb_cell(1.4901161e-8, W), 0);

        // `Home` and `End` are exact on both arms, which is why neither is where the defect shows.
        assert_eq!(stepped(&key(Code::Home), 0.7, &opts), Some(0.0));
        assert_eq!(stepped(&key(Code::End), 0.3, &opts), Some(1.0));

        // A page is ten steps and it clamps rather than wrapping.
        assert_eq!(stepped(&key(Code::PageUp), 0.0, &opts), Some(0.1));
        assert_eq!(stepped(&key(Code::PageDown), 0.05, &opts), Some(0.0));
        assert_eq!(stepped(&key(Code::PageUp), 0.98, &opts), Some(1.0));
    }

    /// **A slider's pairing is not a list's, and the disagreement is exactly four of the eight
    /// cursor codes.**
    ///
    /// [`crate::nav::step`] pairs `Up` with `Left`, because it moves an index into a list and a
    /// list's index grows downward. A slider's value grows *upward*. So the two agree at `Left`,
    /// `Right`, `Home` and `End` and disagree at `Up`, `Down`, `PageUp` and `PageDown` — which is
    /// components ticket 17's finding from the other side, where a container could not read `←` and
    /// `→` through that helper because it reads them *as* `↑` and `↓`.
    ///
    /// The count is the assertion. A slider that called `nav::step` would be right for four keys and
    /// silently backwards for four, on a screen where the thumb visibly moves either way.
    #[test]
    fn the_sliders_pairing_is_not_a_lists_and_the_disagreement_is_four_codes() {
        let opts = SliderOpts {
            steps: 10,
            page: 3,
            ..SliderOpts::default()
        };
        // A list of eleven entries paging by three is the same grid: index `i` is value `i / 10`.
        let cursor = crate::nav::Cursor {
            at: 5,
            len: 11,
            page: 3,
        };
        let mut disagreed: Vec<Code> = Vec::new();
        for code in CURSOR {
            let k = key(code);
            let ours = stepped(&k, 0.5, &opts).expect("a cursor key is the slider's");
            let list =
                crate::nav::step(&k, cursor).expect("a cursor key is a list's") as f32 / 10.0;
            if (ours - list).abs() > f32::EPSILON {
                disagreed.push(code);
            }
        }
        assert_eq!(
            disagreed,
            vec![Code::Up, Code::Down, Code::PageUp, Code::PageDown],
            "the two helpers disagree somewhere other than on the vertical pairing"
        );
    }

    /// **A chord moves a slider nothing, and a release is not a step.**
    ///
    /// §21's *a chord types nothing*, on a component that reads the keyboard without a key map. The
    /// two refusals are `crate::keys::is_chord` and `Edge::Release`, and they are the two
    /// [`crate::nav::step`] exists for — restated here rather than inherited, because the helper that
    /// states them is the one this component cannot call.
    ///
    /// **Shift passes**, for [`crate::keys::SIGNIFICANT`]'s reason: `Ctrl+Right` is an accelerator
    /// and `Shift+Right` is a bigger nudge somebody may want, so the mask is `CTRL | ALT` and Shift
    /// is deliberately outside it.
    #[test]
    fn a_chord_moves_a_slider_nothing_and_a_release_is_not_a_step() {
        let opts = SliderOpts::default();
        for code in CURSOR {
            for modifier in [Mods::CTRL, Mods::ALT, Mods::CTRL.with(Mods::SHIFT)] {
                let k = crate::keys::press_with(code, modifier);
                assert_eq!(
                    stepped(&k, 0.5, &opts),
                    None,
                    "{code:?} with {modifier:?} moved the slider, so the application's \
                     accelerator is gone"
                );
            }
            assert_eq!(
                stepped(&release(code), 0.5, &opts),
                None,
                "{code:?}'s release moved the slider, so every arrow moves it twice on a terminal \
                 that reports event types"
            );
        }
        // Shift is not in the mask.
        assert_eq!(
            stepped(
                &crate::keys::press_with(Code::Right, Mods::SHIFT),
                0.5,
                &opts
            ),
            Some(0.51)
        );
        // And a key that is nobody's cursor key is declined rather than swallowed.
        assert_eq!(stepped(&key(Code::Char('k')), 0.5, &opts), None);
    }

    /// **A value handed in outside `0.0..=1.0` is reported changed once and then never again.**
    ///
    /// `Response::changed` is a comparison against the value **as it arrived**, and getting that
    /// wrong is silent in both directions. Compared against the *sanitised* value a caller handing in
    /// `1.5` sees `changed == false` for ever while the component rewrites the value under it — the
    /// clamp is a change and the component is the one making it. Compared against the raw value it is
    /// one change, because the sanitised value is written back on the same frame.
    ///
    /// **`NaN` is the case that makes it load-bearing rather than tidy.** `f32::NAN.clamp(0.0, 1.0)`
    /// is `NaN` and `NaN != NaN`, so a component that kept a non-finite value would report a change
    /// on **every** frame for ever — which is the one thing spec §20's *zero marked on a steady
    /// frame* cannot survive. `fraction` reads a non-finite value as zero, and the zero is written
    /// back.
    #[test]
    fn a_value_from_outside_the_range_is_reported_changed_once_and_then_never() {
        for (handed, settles_to) in [(1.5f32, 1.0f32), (-2.0, 0.0), (f32::NAN, 0.0)] {
            let mut driver = Driver::headless(40, 1).expect("a sink attaches");
            let mut value = handed;
            let mut seen: Vec<bool> = Vec::new();
            for _ in 0..3 {
                driver.frame(|cx| {
                    let area = cx.area();
                    seen.push(slider(cx, area, &mut value).changed);
                });
            }
            assert_eq!(
                seen,
                vec![true, false, false],
                "handed {handed}: the clamp is a change and it happens once"
            );
            assert_eq!(value, settles_to, "handed {handed}");
        }
    }

    /// **The keyboard reaches the component, and it reaches it through the focus a press awards.**
    ///
    /// `Ctx::next_key` answers only the focused id, and nothing seats a focus until something asks
    /// (architecture issue 25) — so the arrows are dead until either the caller focuses the slider
    /// or a press does. The press is the one this component relies on: a region that declared
    /// `Interest::FOCUS` is *awarded* the focus by the runtime at `end`, so there is no `cx.focus`
    /// call here to be a second producer of it.
    #[test]
    fn an_arrow_reaches_a_focused_slider_and_the_press_is_what_focused_it() {
        let mut driver = Driver::headless(40, 1).expect("a sink attaches");
        let mut value = 0.5f32;
        let mut focused = false;
        let mut changed = false;
        // **One call site, and that is not tidiness.** `Ctx::id` mints from `Location::caller()` and
        // `#[track_caller]` carries the *caller's* line, so a test that calls the component from two
        // places is testing two widgets: the press focuses the first id, the next frame draws the
        // second, the focused id has not drawn, and the **vanish rule clears the focus**. Written
        // that way this test failed at *the release took the focus away again* — a symptom three
        // mechanisms away from its cause. Components ticket 24 met the same trap with the caret as
        // the instrument; here it is the focus.
        let frame =
            |driver: &mut Driver, value: &mut f32, focused: &mut bool, changed: &mut bool| {
                driver.frame(|cx| {
                    let area = cx.area();
                    let resp = slider(cx, area, value);
                    *focused = resp.focused;
                    *changed = resp.changed;
                });
            };

        // Frame one: nothing holds the focus, so the arrow goes nowhere. Nothing seats a focus until
        // something asks — runtime architecture issue 25.
        driver.post_key(key(Code::Right));
        frame(&mut driver, &mut value, &mut focused, &mut changed);
        assert!(!focused);
        assert_eq!(value, 0.5, "an unfocused slider read the keyboard");

        // **A press, and it is two frames rather than one.** The pointer is resolved against the
        // index the *previous* frame built and both the grab and the focus are awarded at `end`, so
        // the frame that delivers the `Down` reads neither. A gate playing one frame a phase
        // measures the cadence and calls it the mechanism.
        driver.post_mouse(at(0, MouseKind::Down(Button::Left)));
        for _ in 0..2 {
            frame(&mut driver, &mut value, &mut focused, &mut changed);
        }
        assert!(
            focused,
            "a press on a `FOCUS` region was not awarded the focus; value={value}"
        );
        driver.post_mouse(at(0, MouseKind::Up(Button::Left)));
        frame(&mut driver, &mut value, &mut focused, &mut changed);
        assert!(focused, "the release took the focus away again");

        // And now the arrow lands. The press put the value at the left end.
        assert_eq!(value, 0.0);
        driver.post_key(key(Code::Right));
        frame(&mut driver, &mut value, &mut focused, &mut changed);
        assert_eq!(value, 0.01, "one step of a hundred");
        assert!(changed);
    }

    /// **A slider declares no wheel, and a notch over it is not consumed.**
    ///
    /// §17's row reads *a slider's value is not an offset and no wheel event moves it*, and
    /// components ticket 20's finding is why that has to be a declaration rather than a missing
    /// branch: a widget that declares `Interest::SCROLL` and consumes nothing is **worse** than one
    /// that declares nothing at all, because it is the topmost region over its rectangle and an
    /// enclosing `scroll_area` never sees the notch either.
    #[test]
    fn a_slider_declares_no_wheel_and_a_notch_over_it_moves_nothing() {
        assert!(
            !SliderOpts::default()
                .interest
                .contains(vitui_runtime::Interest::SCROLL),
            "the slider declared the wheel and has nothing to do with it"
        );

        let mut driver = Driver::headless(40, 1).expect("a sink attaches");
        let mut value = 0.5f32;
        let mut scrolled = (0, 0);
        for _ in 0..2 {
            driver.post_mouse(at(20, MouseKind::Wheel(Notch::Down)));
            driver.frame(|cx| {
                let area = cx.area();
                let resp = slider(cx, area, &mut value);
                scrolled = resp.scrolled;
            });
        }
        assert_eq!(value, 0.5, "a notch moved a value that is not an offset");
        assert_eq!(
            scrolled,
            (0, 0),
            "the notch was delivered to a widget that never asked for it"
        );
    }

    /// **Two sliders at two call sites are two widgets**, which is what `#[track_caller]` buys and
    /// what its absence silently costs.
    ///
    /// `Ctx::id` mints from `Location::caller()`, so a component without the attribute reports *its
    /// own* line for every call in the application: the second slider merges into the first, gets an
    /// inert `Response` and no hit entry, and **the screen still renders correctly** — both thumbs in
    /// the right place, neither of them draggable. `crate::media` shipped that twice, once quietly on
    /// a picture and once loudly on the chrome's own seek bar.
    #[test]
    fn two_sliders_at_two_call_sites_are_two_widgets() {
        let mut driver = Driver::headless(24, 2).expect("a sink attaches");
        let mut a = 0.25f32;
        let mut b = 0.75f32;
        driver.frame(|cx| {
            slider(cx, Rect::new(0, 0, 24, 1), &mut a);
            slider(cx, Rect::new(0, 1, 24, 1), &mut b);
        });
        assert_eq!(driver.inspect().hits().len(), 2);
        assert_eq!(driver.inspect().stop_count(), 2);

        // The same two from **one** call site are one widget unless the caller keys them, which is
        // the merge policy and not an error.
        let mut driver = Driver::headless(24, 2).expect("a sink attaches");
        driver.frame(|cx| {
            for row in 0..2i32 {
                slider(cx, Rect::new(0, row, 24, 1), &mut a);
            }
        });
        assert_eq!(driver.inspect().hits().len(), 1);
        let mut driver = Driver::headless(24, 2).expect("a sink attaches");
        driver.frame(|cx| {
            for row in 0..2i32 {
                cx.with_key(row as u64, |cx| {
                    slider(cx, Rect::new(0, row, 24, 1), &mut a)
                });
            }
        });
        assert_eq!(driver.inspect().hits().len(), 2, "keyed, they are two");
    }

    /// **Criterion 4: one drag of a derived-boundary range moves two values, and that is why there
    /// is no range slider.**
    ///
    /// See [`super::defective::WhyThereIsNoRangeSlider`]. Two thumbs need a third answer neither
    /// `Response::local` nor `Response::rect` carries — *which* thumb — and the spelling that avoids
    /// storing it derives the boundary from the two values it separates. Because the boundary moves
    /// with them, a pointer that crosses it mid-gesture **abandons the thumb it was dragging and
    /// yanks the other one**: measured from `(0.2, 0.8)` with the pointer going `0.3 → 0.6`, the high
    /// thumb jumps `0.8 → 0.6`, a jump of `0.2` on a thumb nobody touched, in a drag that never
    /// released.
    #[test]
    fn one_drag_of_a_derived_boundary_range_moves_two_values() {
        let mut range = Range {
            low: 0.2,
            high: 0.8,
        };
        // The press lands left of the midpoint, so it takes the low thumb — correctly.
        assert_eq!(range.drag(0.3), Which::Low);
        assert_eq!((range.low, range.high), (0.3, 0.8));

        // The same held drag, carried right. The boundary moved with the value it separates.
        assert_eq!(range.drag(0.6), Which::High);
        assert_eq!(
            (range.low, range.high),
            (0.3, 0.6),
            "the drag was supposed to be moving the low thumb"
        );
        // Two values moved in one gesture, and the second moved by a fifth of the track.
        assert!((0.8f32 - range.high).abs() > 0.19);

        // **The stored answer works**, which is what makes this a cost rather than an impossibility:
        // one `Which` kept across frames and the same two drags move one value. That `Which` is the
        // fifth cross-frame fact §14's headline is that drag capture does not need.
        let mut range = Range {
            low: 0.2,
            high: 0.8,
        };
        let held = range.which(0.3);
        for at in [0.3f32, 0.6] {
            match held {
                Which::Low => range.low = at,
                Which::High => range.high = at,
            }
        }
        assert_eq!((range.low, range.high), (0.6, 0.8));
    }

    /// **The float arm is a component and not only a function**, so the two spellings are one call
    /// apart and a reviewer's diff between them is a field.
    #[test]
    fn the_two_stepping_arms_are_one_component_with_one_value_between_them() {
        let opts = SliderOpts::default();
        let run = |arm: Stepping| {
            let mut driver = Driver::headless(W, 1).expect("a sink attaches");
            let mut value = 0.0f32;
            let frame = |driver: &mut Driver, value: &mut f32| {
                driver.frame(|cx| {
                    let area = cx.area();
                    match arm {
                        Stepping::Grid => slider_with(cx, area, value, &opts),
                        Stepping::Float => float_stepped_into(&mut Direct, cx, area, value, &opts),
                    };
                });
            };
            // **Seat the focus with a press at the left end**, which takes a warm frame to build the
            // index the pointer is resolved against and two more for the award. See
            // `an_arrow_reaches_a_focused_slider_and_the_press_is_what_focused_it`.
            frame(&mut driver, &mut value);
            driver.post_mouse(at(0, MouseKind::Down(Button::Left)));
            frame(&mut driver, &mut value);
            frame(&mut driver, &mut value);
            driver.post_mouse(at(0, MouseKind::Up(Button::Left)));
            frame(&mut driver, &mut value);
            assert_eq!(value, 0.0, "the press did not land at the left end");
            for _ in 0..50 {
                driver.post_key(key(Code::Right));
                frame(&mut driver, &mut value);
            }
            value
        };
        assert_eq!(run(Stepping::Grid), 0.5);
        assert_eq!(run(Stepping::Float), 0.4999998);
    }
}
