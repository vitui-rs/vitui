//! Fields, buttons, checkboxes, radios, switches, sliders, steppers and selects.
//!
//! The largest family, and the one where a component holds the least: every one of these takes the
//! value it edits as a `&mut` argument, so the state is the caller's and nothing here has to be
//! kept in step with it.
//!
//! # Examples
//!
//! ```
//! use vitui_components::input::{button, checkbox, slider};
//! use vitui_runtime::Rect;
//! use vitui_runtime::ctx::Driver;
//!
//! let mut agreed = false;
//! let mut volume = 0.5;
//! let mut driver = Driver::headless(30, 5).expect("a sink attaches");
//!
//! driver.frame(|cx| {
//!     let pressed = button(cx, Rect::new(0, 0, 12, 1), "Run").clicked;
//!     assert!(!pressed);
//!
//!     // The value is the caller's; the component edits it in place and says what happened.
//!     checkbox(cx, Rect::new(0, 1, 20, 1), "I agree", &mut agreed);
//!     slider(cx, Rect::new(0, 2, 20, 1), &mut volume);
//! });
//! ```
//!
//! # A text field owns a [`Text`], and that is the one exception
//!
//! [`field`] takes `&mut Text` rather than `&mut String`, because a field is a caret, a selection
//! and an undo history as well as a string — and all three have to survive the frame. Everything
//! else in this module takes the plain value.
//!
//! # A focused field consumes the keyboard
//!
//! While a [`field`] holds the focus it takes every text-bearing key, which is what makes typing
//! work and what makes a single-letter quit key stop working. Bind `Ctrl+Q` beside `q` in any
//! application with a field in it.
//!
//! # A slider takes a fraction
//!
//! `0.0..=1.0`, for the reason [`crate::indicate::meter`] gives: converting once at the call site
//! keeps the caller's units out of the component's rounding.
//!
//! # A select opens a popup, and the popup is a collection
//!
//! [`select`] draws the closed control and, when it is open, an overlay whose body is a
//! [`crate::collect::collection`] over the options. The list is the same component a list anywhere
//! else is, which is why it has the same keyboard, the same type-ahead and the same one hit entry.

use vitui_runtime::focus::ScopeKind;
use vitui_runtime::keys::{Code, Edge, Pressed};
use vitui_runtime::layout::rect;
use vitui_runtime::layout::text::{truncate, width};
use vitui_runtime::overlay::{OverlayOpts, Placement, Z};
use vitui_runtime::{Ctx, CursorShape, Glyph, Id, Interest, Response, Role};

use crate::collect::{CollOpts, CollState, Mode, collection_shaped};
use crate::edit::{Text, WrapKind};
use crate::frame::{Face, face_paint};
use crate::ink::{Direct, Ink};
use crate::keys;
use crate::nav::{self, Cursor, TypeAhead};
use crate::order::Rows;
use crate::overlay::{Blur, MARK, PopupState, ShellOpts, overlay_into, popup_size};
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
/// A `Default` struct, never a required builder.
///
/// # It is [`ChipOpts`] under another name today, and that is a finding rather than an oversight
///
/// `button` and `chip` have `constructions: 1` apiece, and *one construction* is exactly
/// what this is: a face out of [`crate::state::press`] with a label written into it. The two entries
/// are two components in the freeze — two demands, two vocabularies, two places in the tree — and
/// **they are not two drawings.** The drawing is one function, `crate::text`'s `face_and_label`,
/// which is why R07's fill-then-draw defect cannot be present in one of them and absent from the
/// other, and why the day one of them grows a real difference there is a struct here to put it in.
///
/// What was expected to be the difference — *a button's label is [`Role::Title`] and a chip's is
/// [`Role::Dim`]* — turned out to be a **defect in both**. See [`ChipOpts`]: a label wearing a role
/// its face does not is repainted by the hover award every frame the pointer rests, for ever. So
/// there is no `label` field here either, and the reason is one rule rather than a
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

/// **A label on a face that reacts, and a tab stop.** The signature, with `Rect` for
/// `Rect`.
///
/// **Hostile axes:** none.
///
/// One rectangle, one face out of [`crate::state::press`] and one tab stop. Nothing about its
/// construction is a function of its width, and it holds no offset and no content of its own.
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
/// crate* — **since lifted**, because `Mouse` and the types needed
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
    /// The two equalities on the component. Swept over the widths where the arithmetic runs out: a
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
    /// pixel correctly**. That is the 110-of-338 defect, and this is the two-line version of
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
/// A `Default` struct, never a required builder.
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

/// **The API that was deleted, kept deleted by a pair.**
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
/// only a gesture makes — and is what undo restores. The difference is **0.0007 µs
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

/// **A text field: one component, one flag.**
///
/// **Hostile axes:** `scrolled`, `shrunk`, `wheeled`, `narrow`.
///
/// All four, and it is the only Tier 1 row that declares all four for its own reasons rather than by
/// inheritance. The narrow scene is the wrap memo at 300 and at 120, 625 rows drawn where
/// 875 are needed; the other three are the caret's, the document's and the offset's.
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

/// **The letters this widget owns as chords: undo and select-all.**
///
/// Named so that the loop's refusal is one list rather than a match arm nobody reads beside the arm
/// that acts. What holds the two together is not co-location — it is
/// `crate::contract`'s sweep, which posts every chord and compares what the widget takes against
/// what it declares: a letter added here and nowhere else answers nothing and reads as a **dead**
/// declaration, and one added below and not here is declined before it arrives.
const OWNED_CHORDS: [char; 2] = ['z', 'a'];

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
    let id = cx.id();
    field_keyed(ink, cx, id, area, st, opts)
}

/// **[`field_into`] under an id its container minted**, which is the seam a [`form`] needs and the
/// only way one can exist.
///
/// [`Ctx::id`] mints from `Location::caller()`, so a container cannot ask *what id would row 7
/// have* — it can only find out by drawing row 7 and reading [`Response::id`] back. A form has to
/// know the answer **before** it draws, because the key that moves the cursor is handed back by the
/// focused field and arrives at the scope's after-the-body moment, one row too late to be the row it
/// names. So the ids are the container's arithmetic: `Id::keyed(form_id, row)`, minted here and
/// hung under the form's own id, which is the rule and [`crate::collect::Cell::id`]'s answer
/// one component over — *every workaround that looks like a hack is the id being opaque*.
///
/// Crate-private, because an id is not part of the component shape: a public one would let an
/// application spell a field's identity, and identity comes from the call site and may never be
/// persisted.
#[track_caller]
pub(crate) fn field_keyed<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    id: Id,
    area: Rect,
    st: &mut Text,
    opts: &FieldOpts,
) -> Response {
    draw_with(ink, cx, id, area, st, opts, defective::Refused::NONE)
}

/// **The three axes `field` can be false on that are not on [`FieldOpts`]**, threaded here so the
/// shipped build and every refused one are one function with one value between them.
///
/// It was one argument and one axis until the hostile-axis scenes needed the other two:
/// [`defective::Window`] is the window arithmetic — `scrolled` and `shrunk` axes, whose two
/// refusals are the *inverted sign* and *stopping at the last content row* — and
/// [`defective::CaretRow`] is where the caret's row is measured from. They arrive as a
/// [`defective::Refused`] rather than as three parameters because they are three
/// [`defective::Window`]-shaped values a reader transposes; `crate::wheel::Play` is the same answer
/// one module over, for the same reason.
#[track_caller]
fn draw_with<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    id: Id,
    area: Rect,
    st: &mut Text,
    opts: &FieldOpts,
    refused: defective::Refused,
) -> Response {
    // **The id is the caller's** — minted by `field_into` from `Ctx::id` (ADR 0027, outside every
    // closure) or by a container that owes its rows' identities, which is [`field_keyed`].
    if area.is_empty() {
        return Response::inert(id, area);
    }
    let w = area.w;
    let rows = area.h;

    // **One hit entry for the widget.** *79 regions against 621*: the alternative is one
    // region per visible cluster, which is `Regions::PerCluster` and is what a field that wanted a
    // per-cluster hover would write.
    let mut resp = cx.interact(id, area, opts.interest);

    // **A click places the caret by a column, which is a boundary by construction.**
    // `Response::local` is the press's position inside the widget's own rectangle, so the row is a
    // row of the viewport and the column is a screen column — neither is a byte offset, and that is
    // the whole of *there is no such thing as "put the caret at byte N"* on the pointer path.
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
        // **A chord is not a motion and is not an edit**, and this guard is components ticket 38's
        // finding rather than a tidy-up. Without it the arms beneath read `k.code` alone, so
        // `Ctrl+Left` moved the caret **one cluster** — the widget swallowed the accelerator *and*
        // did the wrong thing with it, which is worse than either half: *a chord pressed into
        // every focusable types nothing* is green either way, because a caret move types nothing.
        // Word motion is exactly what a user pressing `Ctrl+Left` means, and it is
        // `crate::contract::ABSENT`'s one row — the engine exports no word iterator — so the honest
        // answer is to hand the key back rather than to answer it with a cluster.
        //
        // **Shift is not in `SIGNIFICANT` and that is the whole point of the predicate**:
        // a capital is what Shift is for, and `Shift+Left` is this widget's own selection step.
        // The two chords the widget owns are read below, in their own arm; every other one is the
        // application's. One list and not two, so a third owned chord is one edit.
        if keys::is_chord(&k) && !matches!(k.code, Code::Char(c) if OWNED_CHORDS.contains(&c)) {
            cx.decline(k);
            break;
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
                let before = st.caret();
                st.step_row(w, up, shift);
                // **A cursor key this widget cannot act on belongs to whatever is above it.** A
                // one-row `input` — the flag at `WrapKind::Ruler`, which is most fields anybody
                // writes — has no row to step to, and a field that consumed `Up`/`Down` anyway
                // leaves every container above it **deaf**: a `crate::input::form` is a `Group`
                // whose `crate::nav::cursor` never sees an arrow, on a screen that renders
                // perfectly. That is *declared and consumed nothing*
                // arriving on the keyboard axis, and it is why the answer is the caret's own
                // position rather than a flag: the same line is right at the top of a textarea and
                // at the bottom of one.
                st.caret() != before
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
    // The finding — *a body dead downward is alive sideways* — answered by construction
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
    // **The face is resolved before a cell is written and never restyled after**, and
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
    if refused.regions == defective::Regions::PerCluster {
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
    // **How many rows of the rectangle are written, and it is all of them.** The shipped answer
    // ignores the content's length: a row past the last one draws its pad, because the previous
    // frame's content is what is there otherwise. [`defective::Window::ContentRowsOnly`] is the
    // obvious saving, and it is the `shrunk` axis — the stale tail, from the direction no counter
    // watches, and *cheaper* on every one of them.
    let painted = match refused.window {
        defective::Window::ContentRowsOnly => u16::try_from(index.rows().saturating_sub(offset))
            .unwrap_or(rows)
            .min(rows),
        _ => rows,
    };
    for r in 0..painted {
        let y = area.y + i32::from(r);
        // **The window's sign**, and `defective::Window::Inverted` is the `scrolled` axis: the
        // content is at `offset + r` and the refusal reads `offset - r`, which was found three
        // times independently and which every counter in the stack approved of.
        let row = match refused.window {
            defective::Window::Inverted => offset.saturating_sub(usize::from(r)),
            _ => offset + usize::from(r),
        };
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

    // **The caret's row is a row of the viewport, and `defective::CaretRow::Content` is the row of
    // the content.** The loop above recorded the first; the refusal below is what somebody writes
    // while looking at that loop, because the content row is the number already in hand. They are
    // the same number at offset 0 and at **no other offset** — so it is right on every field a gate
    // is written over and wrong the moment a reader scrolls — and the difference is invisible to
    // every instrument that reads cells: the caret is the terminal's cursor, so the two screens are
    // `crate::document::SURFACE_BLIND` cells apart and *the equality against a reference render
    // cannot see it either*. `crate::window::misplaced_caret` reads `Frame::caret`, which is the
    // only instrument that can.
    let caret_at = match refused.caret {
        defective::CaretRow::Content => Some((
            area.x + i32::from(caret.col().min(w.saturating_sub(1))),
            area.y + i32::try_from(caret_row).unwrap_or(i32::MAX),
        )),
        defective::CaretRow::Viewport => caret_at,
    };
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

/// **The refused region count, built so a gate can watch it.** One entry per visible cluster.
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
// `select` — the owner of a popup, and the second half of the two structs
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **What the owner owns and the body never writes.**
///
/// The first half of *two structs, one writer each*: this is written only by `select`, from its own
/// input and from the inbox, and [`PopupState`] is written only by the body. Nothing has two writers,
/// which is the goal reached one family over.
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
    /// Derived from what the body reports and written only here, so the one-writer rule is
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

/// **Where a popup's size comes from. Three spellings, two of them refused.**
///
/// One field rather than a boolean in a signature, so a reviewer's diff between the shipped build
/// and either refused one is a single line — [`crate::disclose`]'s arrangement one family over.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Sizing {
    /// **The rule.** [`popup_size`] over the options and the room the screen has, so the height is
    /// capped and the body's [`gutter`](crate::overlay::gutter) turns *off the bottom* into *scrollable*.
    #[default]
    ToTheRoom,
    /// **The defect that costs one unreachable row of four.** Sized to the content, uncapped.
    /// [`place`](vitui_runtime::overlay::place) clamps a position and never a size, so a popup taller
    /// than the screen hangs off the bottom edge at its stated size, [`gutter`](crate::overlay::gutter) sees as many rows as
    /// it has content and says *no bar*, and the rows past the edge are drawn, clipped, and reachable
    /// by nothing.
    ToTheContent,
    /// **The first defect.** Sized from the drawn extent the body reported last time it
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

/// [`select`]'s options. rule 3: a `Default` struct, never a required builder.
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
/// **Hostile axes:** `scrolled`, `wheeled`.
///
/// The popup's list is a windowed [`crate::collect::collection`] and a notch moves the offset it
/// owns. The other two are not this component's: the shut face elides at every width through the
/// one elision drawing this crate has, and the popup's extent is a fixpoint rather than a residue.
///
/// The shape, with the one recorded cost: **an overlay costs two lifetime annotations**, and
/// this is the component they were measured on. `popup` and `options` are `'f` because the body
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
    /// **What the popup's own list refuses**, which is `collection`'s vocabulary and not a second
    /// one.
    ///
    /// It is [`crate::collect::TableShape`]'s `coll` field one family over and
    /// for its reason: the popup's list is *the collection over the option list*,
    /// so the three axes a windowed list can be wrong on are `collection`'s, reached by calling it.
    /// A second vocabulary here would be a second answer to *what is a stale tail*.
    ///
    /// [`crate::collect::TableShape`]: crate::collect
    pub(crate) list: crate::collect::CollShape,
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
    /// **The rule**, and it is the collection's: one entry for it, however many rows it has.
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
    /// body is never reached, so nothing is declared: *closed content is not drawn*, one family
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
        // **On the way out the owner refocuses itself** — one id it already has, so no id
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
    // steppers — the reason for four arrow ends and not eight.
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
    // **And the rows below the face, because the face is the rectangle** — the second half, which
    // components 40 found unmet here and in `crate::files::file_picker` and nowhere else on the
    // freeze: twenty-six of the twenty-eight write every cell of any rectangle they are handed, and
    // the two that did not are the two overlay owners. **A remainder cannot be named in a
    // `Response`** — the rule's third clause says *the cells it does not write are named in its
    // return value*, and the return value here is the runtime's own type, so writing them is the
    // only reachable answer rather than the chosen one.
    //
    // **The paint is the face's and not `Role::Body`**, because `press_into` declares the hover
    // award over the whole of `area`: a rectangle a component paints one row of and awards all of
    // is a rectangle whose hover repaints cells nobody wrote.
    crate::text::pad_rows(
        ink,
        cx,
        Rect::new(area.x, area.y + 1, area.w, area.h.saturating_sub(1)),
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
        // **A chord belongs to the application**, and components ticket 38 found this
        // loop reading `k.code` alone: `Ctrl+Down` and `Alt+Enter` opened the list, so a shut
        // `select` ate every accelerator built on the four keys it owns — silently, on a screen
        // where nothing had visibly happened.
        if keys::is_chord(&k) {
            cx.decline(k);
            break;
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
    // *`focus_left` is what an outside click already produces* is true of the **popup's** id
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
            move |cx| {
                popup_body(
                    &mut Direct,
                    cx,
                    popup,
                    options,
                    chosen,
                    search,
                    shape,
                    carried,
                    id,
                )
            },
        );
    }

    resp
}

/// **The popup's body: the shell, then the collection over the option list.**
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
              and the owner's id — which it needs in order to hand the keyboard over exactly once, \
              plus the `Ink` seam's writer. The extras exist so the refused arms are one field \
              apart rather than two bodies"
)]
pub(crate) fn popup_body<I: Ink>(
    ink: &mut I,
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
        // **`cx.id()` and not a named root**, and the difference is the sentence arriving
        // inside a body: two `select`s standing at once are two overlays whose bodies are one
        // function, so a named root gives both popups the same ids, `Ctx::interact` makes the
        // second claim **inert**, and the frame reports one popup's worth of entries where two
        // declared. Here the id stack is rooted at the *owner*, so one source line mints two ids.
        let blur = cx.id();
        let _ = cx.interact(blur, area, Interest::HOVER);
        let list = cx.id();
        let _ = cx.interact(list, area, Interest::CLICK.with(Interest::FOCUS));
    }

    // **The shell goes through the seam and not through `Direct`**, which is the one
    // change to this function. `overlay_with` hard-codes [`Direct`], so everything a popup drew was
    // invisible to a [`Pen`](crate::runner::Pen) and the only picture of a popup's interior this
    // crate had was `crate::popup::popup_cells_into` — *the same two orders written where a `Pen`
    // can see them*, which is to say a copy, and `crate::ink`'s own trap says a gate written
    // against a copy tests the copy. Threaded, the shipped body **is** the drawing a scene compares,
    // and `crate::dropped` is what compares it.
    //
    // **What it does not reach is a `Ctx::overlay` body**, and that is unchanged: a body is
    // `FnMut(&mut Ctx<'f, '_>) + 'f` and a `&mut I` borrowed for the call cannot travel into one
    // (the fifth component, components 26's `'f`). So a `Pen` reaches this function only when
    // a caller invokes it **in the base pass** — `crate::popup`'s own named substitution, and it is
    // on both arms of every comparison there.
    //
    // **The shell's id, minted here rather than inside `overlay_with`.** One source line, and it
    // still mints two ids for two open popups: the id stack is rooted at the *owner*, which is the
    // sentence the `Closed::Declare` branch above already rests on.
    let shell_id = cx.id();
    let shell = overlay_into(
        ink,
        cx,
        shell_id,
        area,
        rows,
        &ShellOpts::default(),
        |ink: &mut I, cx: &mut Ctx<'_, '_>, interior: Rect| {
            // **The refused fill**, before a single row is written. Every non-blank cell the rows then
            // write is written twice and re-damaged for as long as the popup stands.
            if shape.fill == Fill::FillFirst {
                let paint = cx.theme().paint(Role::Body);
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
            let mut mine = |k: &Pressed, cursor: usize| {
                // **A chord belongs to the application**, here as much as at the owner.
                // Components ticket 38 found this closure reading `k.code` alone: `Ctrl+Enter`
                // committed and `Alt+Esc` dismissed, so an open popup ate every accelerator built
                // on its own two keys — and unlike the owner's, this one is *inside* a trapless
                // overlay, where the application has no other reader.
                if keys::is_chord(k) {
                    return false;
                }
                match k.code {
                    Code::Enter => {
                        answer = Some(cursor);
                        true
                    }
                    Code::Escape => {
                        answer = Some(chosen);
                        true
                    }
                    _ => false,
                }
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
            let list_resp = collection_shaped(
                ink,
                cx,
                interior,
                list,
                &opts,
                Rows::of(options.len()),
                |buf, range: core::ops::Range<usize>| {
                    range.into_iter().find(|&i| options[i].starts_with(buf))
                },
                |ink: &mut I, cx: &mut Ctx<'_, '_>, r: Rect, i: usize, face: Face| {
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
                shape.list,
            );
            // **A click on a row is a choice**, and it is the collection's own response rather
            // than a second hit entry per row: *one hit entry per collection* is what keeps
            // the closed popup at `crate::popup::CLOSED_DECLARES` and not thirteen entries more.
            //
            // The **edge** and not `Response::clicked`: a row selects on the press, so by the time a
            // click has completed the collection has already moved its cursor and a release-driven
            // choice arrives a frame late.
            if list_resp.press_began {
                answer = Some(list.sel.lead);
            }
            // **The popup takes the keyboard from its owner, exactly once.** `if cx.is_focused(owner)`
            // and never `if !cx.is_focused(list)`: the second drags the keyboard back every frame the
            // user has tabbed away, which is the refused spelling one family over.
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
// `checkbox`, `radio` and `switch` — the three Tier 2 toggles, and one machine between them
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **Which of the three toggles a call is**, and the whole difference between them.
///
/// The freeze has three rows here and all three get one sentence — *[`crate::state::press`]
/// plus a [`Glyph`] pair plus a [`Role`]* — so this is [`crate::disclose::Collapses`]'s arrangement
/// and [`crate::overlay::FAMILY`]'s: **one machine and three configurations**, with a table beside it
/// that a test iterates rather than a paragraph a reader is trusted with. Three arms and three rows,
/// and no fourth on either side.
///
/// # The mark is not a pair, and the freeze is what says so
///
/// The backlog's sentence reads *a `Glyph` pair*, and the freeze's `glyphs` column — which is a
/// value and therefore the authority — gives `checkbox` exactly [`Glyph::Tick`], `radio` exactly
/// [`Glyph::Bullet`] and `switch` **nothing at all**. There is no pair to draw, because the *off*
/// half of a toggle cannot be a glyph: `CONTEXT.md` defines a glyph as a lookup with **no spelling
/// blank**, and off is blank. So each mark is one glyph and one absence, and the absence is painted
/// in the face like every other cell of the widget.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Toggle {
    /// **A checkbox.** [`Glyph::Tick`] when on, a blank cell when off. The default.
    #[default]
    Check,
    /// **A standalone radio button.** [`Glyph::Bullet`] when on, a blank cell when off.
    ///
    /// **A radio *set* is not this**: a set is
    /// [`crate::collect::collection`] at [`Mode::Options`] — *exactly one, and it can never become
    /// zero* — which is why `crate::INVENTORY`'s `radio` row carries no composition edge. The row is
    /// the widget and the set is a different call, and `tests::a_radio_set_is_a_collection_and_not_a_second_store`
    /// is the gate that keeps the two apart.
    Radio,
    /// **A switch.** No glyph at all: the knob is a blank cell inside the face and the state is
    /// carried by [`ToggleOpts::words`] and by where the knob sits.
    ///
    /// See [`ToggleOpts::words`] for why the one row of the three with an empty `glyphs` column is
    /// also the one whose state survives a terminal with no colour *and* no repertoire.
    Switch,
}

impl Toggle {
    /// All three, which is what the tables and the sweeps iterate.
    pub const ALL: [Toggle; 3] = [Toggle::Check, Toggle::Radio, Toggle::Switch];

    /// The freeze's id for this configuration.
    pub const fn id(self) -> &'static str {
        match self {
            Toggle::Check => "checkbox",
            Toggle::Radio => "radio",
            Toggle::Switch => "switch",
        }
    }

    /// **The glyph its mark is spelled with, or `None` when it has none.**
    ///
    /// The join `tests::the_three_toggles_demand_exactly_what_the_freeze_says_they_do` runs this
    /// against `crate::INVENTORY`'s `glyphs` column in both directions, so a mark that grew a second
    /// glyph without the freeze moving fails here rather than on a screen.
    pub const fn mark(self) -> Option<Glyph> {
        match self {
            Toggle::Check => Some(Glyph::Tick),
            Toggle::Radio => Some(Glyph::Bullet),
            Toggle::Switch => None,
        }
    }
}

/// The three toggles' options.
///
/// A `Default` struct, never a required builder.
///
/// # There is no `label` role here, and that is the partition rule rather than a simplification
///
/// [`ChipOpts`] carries the finding: *a label wearing a role its face does not is repainted by the
/// hover award every frame the pointer rests, for ever.* The label, the mark and the padding are one
/// paint — the face [`crate::state::press`] returned — which is why [`crate::text::FitOpts`] carries
/// two roles and why both of them are given the same value here.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ToggleOpts {
    /// Which of the three this is.
    pub kind: Toggle,
    /// The four faces, handed to [`crate::state::press`] whole.
    pub faces: Faces,
    /// Where the label sits in the row beside the mark.
    pub justify: Justify,
    /// **What a [`Toggle::Switch`] writes in its knob field, on and off.**
    ///
    /// Ignored by the other two, which spell their state with [`Toggle::mark`].
    ///
    /// # The one row of the three with no glyph is the one whose state always survives
    ///
    /// The whole subject is what a distinction costs at a lower rung: a checkbox and a radio carry
    /// theirs on the **glyph** axis, which is exactly what `GlyphSet::Ascii` narrows — `✓` becomes
    /// `x` and `•` becomes `*`, both still present, both still one cell. A switch carries its state
    /// on **three** axes and only one of them is the palette: the two words, the side the knob sits
    /// on, and the face. `tests::a_switch_states_itself_at_every_rung_and_a_checkbox_needs_its_glyph`
    /// is the count, and it is the reason the empty `glyphs` column in the freeze is not a hole.
    pub words: (&'static str, &'static str),
    /// What it declares.
    ///
    /// `Interest::SCROLL` is not in it, and the reason is: a widget that declares the
    /// wheel and consumes nothing is worse than one that declares nothing at all, because it is the
    /// topmost region over its rectangle and an enclosing `crate::scroll::scroll_area` never sees the
    /// notch either.
    pub interest: Interest,
}

impl ToggleOpts {
    /// **How many cells the mark field is**, the label excluded.
    ///
    /// Two for a mark and the gap after it. For a switch it is the wider of the two words plus that
    /// same gap, which is what makes the knob's travel a property of [`ToggleOpts::words`] rather
    /// than a number written twice.
    pub fn field(&self) -> u16 {
        match self.kind {
            Toggle::Check | Toggle::Radio => 2,
            Toggle::Switch => width(self.words.0)
                .max(width(self.words.1))
                .saturating_add(1),
        }
    }
}

impl Default for ToggleOpts {
    fn default() -> ToggleOpts {
        ToggleOpts {
            kind: Toggle::Check,
            faces: Faces::default(),
            justify: Justify::Start,
            words: ("on", "off"),
            interest: Interest::CLICK.with(Interest::HOVER).with(Interest::FOCUS),
        }
    }
}

/// **A box with a tick in it, a label beside it, and the caller's own `bool`.**
///
/// **Hostile axes:** none.
///
/// Its whole cross-frame fact is the caller's `&mut bool`. The mark is one cell out of the theme's
/// table and the label elides through [`crate::text::fit`], which is `text`'s own flag.
///
/// The signature is written out by name — `fn(&mut Ctx, Rect, &str, &mut bool) -> Response` —
/// and rule 2 is what the `&mut bool` is: *the widget's own value, never application data*.
///
/// ```
/// use vitui_components::input::checkbox;
/// use vitui_runtime::Rect;
/// use vitui_runtime::ctx::Driver;
///
/// let mut driver = Driver::headless(20, 1).expect("a sink attaches");
/// let mut wrap = false;
/// driver.frame(|cx| {
///     let resp = checkbox(cx, Rect::new(0, 0, 20, 1), "wrap", &mut wrap);
///     // Rule 4: a `Response`, and nothing has happened on a frame with no input.
///     assert!(!resp.changed);
/// });
/// assert!(!wrap);
/// ```
#[track_caller]
pub fn checkbox(cx: &mut Ctx<'_, '_>, area: Rect, label: &str, on: &mut bool) -> Response {
    toggle_with(cx, area, label, on, &ToggleOpts::default())
}

/// **A standalone radio button.** [`checkbox`] with [`Toggle::Radio`]'s mark.
///
/// **Hostile axes:** none.
///
/// [`checkbox`]'s, for [`checkbox`]'s reason: one machine, one `&mut bool`, one cell of mark.
///
/// A radio *set* is [`crate::collect::collection`] at [`Mode::Options`] and is a different call
/// entirely — see [`Toggle::Radio`]. **A standalone radio is the caller's own `bool`**, and two of
/// them side by side are two `bool`s the caller keeps exclusive; nothing here does that for anybody.
///
/// ```
/// use vitui_components::input::radio;
/// use vitui_runtime::Rect;
/// use vitui_runtime::ctx::Driver;
///
/// let mut driver = Driver::headless(20, 2).expect("a sink attaches");
/// let (mut ascending, mut descending) = (true, false);
/// driver.frame(|cx| {
///     let up = radio(cx, Rect::new(0, 0, 20, 1), "ascending", &mut ascending);
///     let down = radio(cx, Rect::new(0, 1, 20, 1), "descending", &mut descending);
///     // Two call sites, so two ids, so two targets. Exclusivity is the caller's.
///     assert_ne!(up.id, down.id);
/// });
/// assert!(ascending && !descending);
/// ```
#[track_caller]
pub fn radio(cx: &mut Ctx<'_, '_>, area: Rect, label: &str, on: &mut bool) -> Response {
    toggle_with(
        cx,
        area,
        label,
        on,
        &ToggleOpts {
            kind: Toggle::Radio,
            ..ToggleOpts::default()
        },
    )
}

/// **A switch.** [`checkbox`] with a knob that travels and two words instead of a glyph.
///
/// **Hostile axes:** none.
///
/// [`checkbox`]'s. The knob's travel is a property of [`ToggleOpts::words`] and not of the
/// rectangle's width, which is what makes the field a derived number rather than a second
/// construction.
///
/// **It is the one row of the twenty-nine whose state survives ASCII *and* no colour**, because it
/// is carried on three axes and only one of them is the palette: the two words, the side the knob
/// sits on, and the face. A checkbox and a radio differ in exactly **1** cell between on and off and
/// that cell came out of the theme's glyph table; a switch differs in **3** and none of them did.
///
/// ```
/// use vitui_components::input::switch;
/// use vitui_runtime::Rect;
/// use vitui_runtime::ctx::Driver;
///
/// let mut driver = Driver::headless(24, 1).expect("a sink attaches");
/// let mut wrap = true;
/// driver.frame(|cx| {
///     let resp = switch(cx, Rect::new(0, 0, 24, 1), "wrap", &mut wrap);
///     assert!(!resp.changed, "nothing has happened on a frame with no input");
/// });
/// assert!(wrap);
/// ```
#[track_caller]
pub fn switch(cx: &mut Ctx<'_, '_>, area: Rect, label: &str, on: &mut bool) -> Response {
    toggle_with(
        cx,
        area,
        label,
        on,
        &ToggleOpts {
            kind: Toggle::Switch,
            ..ToggleOpts::default()
        },
    )
}

/// The three, with the options spelled out: one options sibling for all of them at once.
#[track_caller]
pub fn toggle_with(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    label: &str,
    on: &mut bool,
    opts: &ToggleOpts,
) -> Response {
    toggle_into(&mut Direct, cx, area, label, on, opts)
}

/// **The three, drawing through an [`Ink`] so a counter can see every cell.**
///
/// The entry point a gate takes; [`checkbox`], [`radio`] and [`switch`] are this with [`Direct`] and
/// one field of [`ToggleOpts`] between them.
///
/// # What it composes, and it introduces nothing else
///
/// [`crate::state::press`] for the face and the deferred hover award, `Theme::glyph` for the mark,
/// [`crate::text::fit_into`] for the label's row and `text`'s own `pad_rows` for the rows it does
/// not take. There is no state here at all: a toggle's whole cross-frame fact is the caller's
/// `&mut bool`, and `crate::composed`'s scan is what keeps it that way.
#[track_caller]
pub fn toggle_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    label: &str,
    on: &mut bool,
    opts: &ToggleOpts,
) -> Response {
    // **The id, taken outside every closure**, and `#[track_caller]` all the way up so
    // that two toggles at two call sites are two widgets — `crate::media::player` shipped both
    // halves of that wrong once, on a screen that rendered perfectly.
    let id = cx.id();
    if area.is_empty() {
        return Response::inert(id, area);
    }
    let mut resp = cx.interact(id, area, opts.interest);
    let before = *on;

    // A click flips it, and so does `Space` or `Enter` while it holds the focus. **One drain loop**:
    // `Ctx::decline` hands a key back *and ends the level's turn at the queue*
    // (`crate::collect`'s finding, components 17), so a widget that read some keys here and the rest
    // afterwards would get nothing after the first refusal.
    if resp.clicked {
        *on = !*on;
    }
    while let Some(k) = cx.next_key(id) {
        if k.kind == Edge::Release {
            continue;
        }
        if !keys::is_chord(&k) && matches!(k.code, Code::Char(' ') | Code::Enter) {
            *on = !*on;
            continue;
        }
        cx.decline(k);
        break;
    }

    // **The runtime never sets this** — `Response::changed` is a component's own convention.
    resp.changed = *on != before;
    toggle_drawn(ink, cx, area, label, *on, &resp, opts);
    resp
}

/// **The drawing half, with the [`Response`] supplied rather than declared.** Returns the face.
///
/// Public for [`chip_drawn`]'s reason: a container that already holds an id draws the widget inside
/// a region it declared itself, and a gate that cannot hand a component a `Response` cannot play the
/// hovered frame at all.
pub fn toggle_drawn<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    label: &str,
    on: bool,
    resp: &Response,
    opts: &ToggleOpts,
) -> Role {
    // **The face and the award are one expression evaluated once** — `crate::state::press_into`'s
    // own criterion 1, and the cells are the whole widget's because the whole widget is one face.
    let face = crate::state::press_into(ink, cx, area, resp, &opts.faces);
    let paint = cx.theme().paint(face);

    // The mark and the label sit on the middle row, and the rows either side are face. Exactly
    // `crate::text::face_and_label`'s split, which is what makes a toggle read as a chip with a
    // mark in front of it.
    let (above, rest) = rect::split_at_v(area, area.h.saturating_sub(1) / 2);
    crate::text::pad_rows(ink, cx, above, paint);
    let (row, below) = rect::split_at_v(rest, 1);
    let field = opts.field().min(row.w);
    let (mark_cells, label_cells) = rect::split_at_h(row, field);
    mark_into(ink, cx, mark_cells, on, face, opts);
    crate::text::fit_into(
        ink,
        cx,
        label_cells,
        label,
        &crate::text::FitOpts {
            justify: opts.justify,
            // **Both roles are the face.** See [`ToggleOpts`]'s own note.
            role: face,
            pad: face,
        },
    );
    crate::text::pad_rows(ink, cx, below, paint);
    face
}

/// **The mark field: one glyph and a blank, or a word that has travelled.**
///
/// The one place the three configurations differ in what they *write*, and it is two verbs — which
/// is why they are one component and not three.
fn mark_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    cells: Rect,
    on: bool,
    face: Role,
    opts: &ToggleOpts,
) {
    if cells.is_empty() {
        return;
    }
    match opts.kind.mark() {
        // A mark and the gap after it, and the *off* half is the same blank the gap is — see
        // [`Toggle`]: a glyph has no blank spelling, so off cannot be one.
        Some(glyph) => {
            let paint = cx.theme().paint(face);
            let mark = if on { cx.theme().glyph(glyph) } else { " " };
            ink.run(cx, cells.x, cells.y, mark, 1, paint);
            ink.run(cx, cells.x + 1, cells.y, " ", cells.w - 1, paint);
        }
        // **The word, on the side its state puts it**, which is the knob travelling and the state
        // being spelled at the same time and in the same two verbs. `fit` is the helper that writes
        // a line into a row and pads the remainder, so there is no second justification arithmetic
        // here either.
        None => {
            let (word, justify) = if on {
                (opts.words.0, Justify::End)
            } else {
                (opts.words.1, Justify::Start)
            };
            // **The gap is split off first and is never the knob's travel.** Justified inside the
            // whole field, the *on* word slides right until it touches the label, and a switch whose
            // knob has arrived reads as one word with the label glued to it.
            let (word_cells, gap) = rect::split_at_h(cells, cells.w.saturating_sub(1));
            crate::text::fit_into(
                ink,
                cx,
                word_cells,
                word,
                &crate::text::FitOpts {
                    justify,
                    role: face,
                    pad: face,
                },
            );
            crate::text::pad_rows(ink, cx, gap, cx.theme().paint(face));
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// `slider` — the drag capture, and the row §17 froze at Tier 3
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **How many steps the keyboard divides the track into. A hundred.**
///
/// See [`SliderOpts::steps`] for why the keyboard's unit is an integer and the pointer's is not.
pub const SLIDER_STEPS: u32 = 100;

/// **How many of [`SLIDER_STEPS`] a page is. Ten.**
pub const SLIDER_PAGE: u32 = 10;

/// [`slider`]'s options.
///
/// A `Default` struct, never a required builder.
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
    /// **The award is over the thumb and not over the track**, which is the relation rather
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
    /// The row reads *a slider's value is not an offset and no wheel event moves it*; components
    /// the other half is a widget that declares the wheel and consumes
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
/// Drag capture was measured over the video player's seek bar, and it wrote the
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
/// // A press jumps to where it landed, at the measured fraction.
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
/// slider that reports a change on **every** frame for ever, which is the one thing
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
/// It is the same boundary from the other side: there, a container could not read `←`
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

/// **A value on a track, dragged from where the pointer is and stepped by the keyboard.**
/// Tier 3 row, and the mechanism under it is [`grab`].
///
/// **Hostile axes:** none.
///
/// The track is the rectangle, the value is a fraction of it and the thumb is derived — nothing here
/// is a window onto content larger than what it was handed, and the grab is
/// [`vitui_runtime::Response::local`] over [`vitui_runtime::Response::rect`] and nothing else.
///
/// `value` is a fraction of the track, `0.0..=1.0`. **It is not a range**, and that is
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
    // **The id, taken outside every closure**, and `#[track_caller]` all the way up so
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
/// order is the same order, which is why that helper has one job: *the bar every scrollable
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
/// mean inventing one out of the track's own width, which is the shape the unit rule exists to
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

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// `form` — R3 example: `field` + `nav::cursor` + the focus ring the draw builds
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **Everything a form keeps across frames, and it is [`crate::nav::cursor`]'s own parameter.**
///
/// One field, and the type is exactly as big as the thing `nav::cursor` cannot borrow from the ring:
/// the buffer that has to survive a frame. There is **no cursor here** — the cursor is the focus,
/// which is the runtime's, read back with [`Ctx::is_focused`](vitui_runtime::Ctx::is_focused) during
/// the draw; and there is no selection, no per-row slot and no geometry.
/// `tests::a_form_adds_nothing_to_what_nav_cursor_already_needs` is that as a `size_of`, which is
/// what turns *no new mechanism* from a claim into a number.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct FormState {
    /// The type-ahead buffer and the one deadline it owes. [`crate::nav::TypeAhead`], because a
    /// second buffer would be a second place the one-second window is written down.
    pub ahead: TypeAhead,
}

impl FormState {
    /// A form nobody has typed into.
    pub fn new() -> FormState {
        FormState::default()
    }
}

/// [`form`]'s options.
///
/// A `Default` struct, never a required builder.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FormOpts {
    /// The role a label is drawn in.
    pub label: Role,
    /// How wide the label column is, or `0` to derive it from the widest label. Capped at half the
    /// form's width, because a label column wider than its fields is a form with no fields in it.
    pub label_w: u16,
    /// How many cells sit between a label and its field.
    pub gap: u16,
    /// How many rows one entry takes. One is an `input`; more is a `textarea` — the one flag, and
    /// the form does not have a second opinion about it.
    pub rows: u16,
    /// The role the gap, the label column's padding and the tail below the last entry are drawn in.
    pub pad: Role,
    /// **Whether the form is one tab stop.**
    ///
    /// `true` by default, which is the placement decision rather than this component's taste:
    /// *`nav::cursor`'s placement is the decision, not its contents* — a `Group` collapses the walk
    /// onto the first entry and the rest stay in the ring, so `Tab` reaches the form and the arrows
    /// move inside it.
    ///
    /// **It is also the only way a form can have arrows at all**, and that is a fact about the
    /// runtime rather than a preference. A container receives the keys its children hand back only
    /// through [`Ctx::scope`](vitui_runtime::Ctx::scope)'s after-the-body moment, and
    /// [`ScopeKind`] has three arms: a `Group`, a modal's `Trap`, and the code editor's `Isolated`.
    /// So `false` is a form of *n* tab stops whose arrow keys do nothing, and both arms are measured
    /// — see `tests::a_grouped_form_is_one_tab_stop_and_an_ungrouped_one_is_every_field`.
    pub group: bool,
    /// What each field is. [`FieldOpts`], forwarded whole — a form has no field of its own.
    pub field: FieldOpts,
}

impl Default for FormOpts {
    fn default() -> FormOpts {
        FormOpts {
            label: Role::Dim,
            label_w: 0,
            gap: 1,
            rows: 1,
            pad: Role::Body,
            group: true,
            field: FieldOpts::default(),
        }
    }
}

/// **What a form is made of, as a value**, so that *no new mechanism* is enumerable rather than
/// asserted.
///
/// Three names, one per clause of R3 sentence — *`form` is `field` + `nav::cursor` + the focus
/// ring the draw builds* — and `tests::a_form_is_the_three_things_r3_says_it_is` reads them out of
/// this file. A fourth entry here would be a fourth clause in the spec.
pub const FORM_IS: [&str; 3] = ["field", "nav::cursor", "the focus ring"];

/// **A column of labelled fields, one tab stop, and the arrows moving inside it.**
///
/// **Hostile axes:** none.
///
/// The fields are [`field`]s and each carries its own four. What is left over is a column of
/// rectangles and one tab stop, and neither is a function of the width or a window onto anything.
///
/// R3 in its own words: *a composition of shipped components with no new mechanism.* The
/// fields are [`field`], the navigation is [`crate::nav::cursor`], and what says where the keyboard
/// is is the focus ring the draw builds — [`FormState`] is one type-ahead buffer and nothing else.
///
/// # The labels arrive as their own slice, and that is a count rather than a shape
///
/// [`crate::nav::cursor`] takes `&[&str]`. A form written over a slice of records — a label and a
/// `Text` in one struct, which is the obvious spelling — has to **build** that slice every frame,
/// and a `Vec<&str>` a frame is one allocation a frame against a budget of zero. The
/// refused shape is kept runnable at [`defective::form_collecting_labels`] and priced by
/// `tests::a_form_allocates_nothing_and_the_record_shaped_spelling_allocates_a_frame`.
///
/// # Two entries that do not fit are two entries nobody drew
///
/// Density is theme data and it changes rectangles, so the same form inside the same panel
/// stands a different number of fields at `Compact` and at `Cosy`. That is **reported rather than
/// hidden**: the count that falls off the bottom is `labels.len()` minus the hit entries the frame
/// declared, and `tests::the_same_form_stands_fewer_fields_at_cosy_and_neither_writes_a_cell_twice`
/// is where both columns are printed.
///
/// ```
/// use vitui_components::edit::Text;
/// use vitui_components::input::{FormState, form};
/// use vitui_runtime::ctx::Driver;
///
/// let labels = ["name", "email", "role"];
/// let mut texts = [Text::input(), Text::input(), Text::input()];
/// let mut st = FormState::new();
/// let mut driver = Driver::headless(40, 6).expect("a sink attaches");
/// driver.frame(|cx| {
///     let area = cx.area();
///     let resp = form(cx, area, &mut st, &labels, &mut texts);
///     // Rule 4: a `Response` back, and nothing has happened on a frame with no input.
///     assert!(!resp.changed);
/// });
/// let frame = driver.inspect();
/// // Three fields, three hit entries, three ring entries — and **one** tab stop, because a form is
/// // a `Group` and that is what `nav::cursor` is for.
/// assert_eq!(frame.hits().len(), 3);
/// assert_eq!(frame.ring().len(), 3);
/// assert_eq!(frame.stop_count(), 1);
/// ```
#[track_caller]
pub fn form(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    st: &mut FormState,
    labels: &[&str],
    texts: &mut [Text],
) -> Response {
    form_with(cx, area, st, labels, texts, &FormOpts::default())
}

/// [`form`], with the options spelled out.
#[track_caller]
pub fn form_with(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    st: &mut FormState,
    labels: &[&str],
    texts: &mut [Text],
    opts: &FormOpts,
) -> Response {
    form_into(&mut Direct, cx, area, st, labels, texts, opts)
}

/// **[`form`], drawing through an [`Ink`] so a counter can see every cell.**
///
/// The entry point a gate takes; [`form`] is this with [`Direct`].
#[track_caller]
pub fn form_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    st: &mut FormState,
    labels: &[&str],
    texts: &mut [Text],
    opts: &FormOpts,
) -> Response {
    // **The id, taken outside every closure**, and it is also the base every row's id is
    // keyed from — see [`field_keyed`].
    let id = cx.id();
    let mut resp = Response::inert(id, area);
    if area.is_empty() {
        return resp;
    }
    let n = labels.len().min(texts.len());
    let row_h = opts.rows.max(1);
    let shown = n.min(usize::from(area.h / row_h));

    let label_w = match opts.label_w {
        0 => labels[..shown]
            .iter()
            .map(|l| width(l))
            .max()
            .unwrap_or(0)
            .min(area.w / 2),
        w => w.min(area.w),
    };

    let pad = cx.theme().paint(opts.pad);
    let (rows, tail) = rect::split_at_v(
        area,
        row_h.saturating_mul(u16::try_from(shown).unwrap_or(u16::MAX)),
    );

    // **The body, and it is one closure so that the two scope arms cannot become two drawings.**
    let draw = |cx: &mut Ctx<'_, '_>, ink: &mut I, texts: &mut [Text]| -> bool {
        let mut changed = false;
        for i in 0..shown {
            let row = Rect::new(
                rows.x,
                rows.y + i32::from(row_h) * i32::try_from(i).unwrap_or(0),
                rows.w,
                row_h,
            );
            let (label, rest) = rect::split_at_h(row, label_w);
            let (gap, entry) = rect::split_at_h(rest, opts.gap);
            crate::text::fit_into(
                ink,
                cx,
                label,
                labels[i],
                &crate::text::FitOpts {
                    justify: Justify::Start,
                    role: opts.label,
                    pad: opts.pad,
                },
            );
            // A label column taller than one row, and the gap between it and the field, are the
            // form's own cells: the rule is that the owner of a rectangle writes all of it.
            crate::text::pad_rows(ink, cx, rect::shrink(label, 0, 1, 0, 0), pad);
            crate::text::pad_rows(ink, cx, gap, pad);
            let fid = form_row_id(id, i);
            changed |= field_keyed(ink, cx, fid, entry, &mut texts[i], &opts.field).changed;
        }
        changed
    };
    resp.changed = match opts.group {
        // The scope's id is the form's own: `scope` roots no identity of its own, and it
        // is what makes the after-the-body moment deliver this form's declined keys to this form.
        true => cx.scope(id, ScopeKind::Group, |cx| draw(cx, ink, texts)),
        false => draw(cx, ink, texts),
    };
    crate::text::pad_rows(ink, cx, tail, pad);

    // **The keyboard, after the body**, which is the only moment a container can have one: a field
    // that declined a key ends the level's turn at the queue, and `Ctx::scope` resumes it when the
    // routing target moves outward. Ungrouped there is no scope, `next_key` answers `None`, and this
    // loop does not run — see [`FormOpts::group`].
    // **The cursor is carried through the loop and not read once before it.** `Ctx::next_key` can
    // answer more than one key in a frame — the queue closes on a *decline*, not on a take — and a
    // `cur` computed once would compute the second arrow from the position the first one left
    // behind: two `Down`s in one batch would move one row. `collection`'s own drain loop rebuilds
    // its `Cursor` inside the loop for the same reason.
    let mut at = focused_row(cx, id, shown);
    while let Some(k) = cx.next_key(id) {
        let cur = Cursor {
            at,
            len: shown,
            page: shown.max(1),
        };
        match nav::cursor(cx, id, cur, &mut st.ahead, &k, &labels[..shown]) {
            Some(to) => {
                // **The row's id is arithmetic and not a lookup**, which is the whole reason
                // `field_keyed` exists: a container cannot ask what id a row it has already drawn
                // would have.
                at = to;
                cx.focus(form_row_id(id, to));
                resp.changed = true;
            }
            None => cx.decline(k),
        }
    }
    resp
}

/// **A form's row id, derived from the form's own.**
///
/// `Id::keyed(form, row)`, which is the rule — a component's children hang from its own id —
/// and it is **public** because an application has a use for it that nothing else does:
/// *nothing holds the focus until an application says so*. A program that
/// opens on a form and wants the keyboard in its first field has to name that field, and
/// [`Ctx::id`](vitui_runtime::Ctx::id) mints from `Location::caller()` — there is no other way to
/// ask.
///
/// It does not weaken *identity comes from the call site and may never be persisted*: the
/// `form` argument is a [`Response::id`] read back from the frame that just drew, so the identity is
/// still the call site's and this is arithmetic over it.
///
/// ```
/// use vitui_components::edit::Text;
/// use vitui_components::input::{FormState, form, form_row_id};
/// use vitui_runtime::ctx::Driver;
///
/// let labels = ["name", "email"];
/// let mut texts = [Text::input(), Text::input()];
/// let mut st = FormState::new();
/// let mut driver = Driver::headless(30, 4).expect("a sink attaches");
/// driver.frame(|cx| {
///     let resp = form(cx, cx.area(), &mut st, &labels, &mut texts);
///     // The application seats the keyboard, because nothing else will.
///     if cx.focused().is_none() {
///         cx.focus(form_row_id(resp.id, 0));
///     }
/// });
/// assert!(driver.inspect().focused().is_some());
/// ```
pub fn form_row_id(form: Id, row: usize) -> Id {
    Id::keyed(form, u64::try_from(row).unwrap_or(u64::MAX))
}

/// **Which row holds the keyboard**, or `0` when the form does not.
///
/// Read off the ring rather than stored: the focus is the runtime's and a second copy of it here is
/// a second thing to be wrong. `0` for *nobody*, because a cursor has to be somewhere and the first
/// row is where `Tab` would land.
fn focused_row(cx: &Ctx<'_, '_>, id: Id, shown: usize) -> usize {
    (0..shown)
        .find(|&i| cx.is_focused(form_row_id(id, i)))
        .unwrap_or(0)
}

/// **Every spelling of `field`, of `select` and of `slider` that is refused, kept runnable.**
pub mod defective {
    /// **How many regions the widget declares.**
    ///
    /// *A text widget declares 79 regions against 621 for one per visible cluster.* Both arms
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

    /// **Every refusal `draw_with` threads, as one value.**
    ///
    /// Three axes, and a struct rather than three parameters for [`crate::wheel::Play`]'s reason:
    /// [`Window::Inverted`] and [`Window::ContentRowsOnly`] are two arms of one field and
    /// [`CaretRow::Content`] is a second field, and three enums side by side in a call are exactly
    /// the arguments a reader transposes. [`Refused::NONE`] is what the shipped `field` passes.
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
    pub struct Refused {
        /// How many hit entries the widget declares.
        pub regions: Regions,
        /// How the window over the content is computed.
        pub window: Window,
        /// Where the caret's row is measured from.
        pub caret: CaretRow,
    }

    impl Refused {
        /// **The shipped build**, which is what `field_into` and `field_keyed` pass.
        ///
        /// A `const` and not `Default::default()` at the call site, so that the one place the
        /// shipped component names a value from this module names *this* one and a reader grepping
        /// for the refusals finds it.
        pub const NONE: Refused = Refused {
            regions: Regions::Widget,
            window: Window::Offset,
            caret: CaretRow::Viewport,
        };

        /// The shipped build with one window arithmetic substituted.
        pub const fn windowed(window: Window) -> Refused {
            Refused {
                window,
                ..Refused::NONE
            }
        }

        /// The shipped build with the caret measured from somewhere else.
        pub const fn carets(caret: CaretRow) -> Refused {
            Refused {
                caret,
                ..Refused::NONE
            }
        }
    }

    /// **How the window over the content is computed**, which is `scrolled` and `shrunk` axes
    /// in one enum.
    ///
    /// One enum for the two because both refusals are one expression of the shipped loop and
    /// because the pair is the finding: they are the two directions a window can be wrong, one of
    /// them *drawing the wrong content* and the other *drawing none* — and every counter of
    /// nine approves of both.
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
    pub enum Window {
        /// **The rule.** `offset + r`, over every row of the rectangle.
        #[default]
        Offset,
        /// **The `scrolled` defect.** `offset - r`, which is the inverted sign — *12.21 µs /
        /// 3 058 writes against 62.96 / 20 418, and faster.* Found three times independently.
        Inverted,
        /// **The `shrunk` defect.** Stop at the last content row instead of clearing the rest of
        /// the rectangle, so what stays on screen is the previous frame: the stale tail, *71 of
        /// 80 rows* one component over, and the arm that is **cheaper on every counter**.
        ContentRowsOnly,
    }

    /// **Where the caret's row is measured from.**
    ///
    /// Two arms, and the defect is the one somebody writes while looking at the row loop: the loop
    /// already knows the content row, so placing the caret at it reads as the simpler line. It is
    /// right at offset 0 and wrong at every other offset, and **it is invisible to a cell-for-cell
    /// equality**, because the caret is the terminal's cursor and not a cell.
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
    pub enum CaretRow {
        /// **The rule.** The screen row the content row was drawn on.
        #[default]
        Viewport,
        /// **The defect.** The content row itself, unscrolled.
        Content,
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
    /// One argument, three arms, and the two that are not [`Sizing::ToTheRoom`](super::Sizing::ToTheRoom) are refusals:
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
    /// The literal `Copy`-only body, one family over: [`crate::popup::WHEEL_CLICKS`] notches move
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

    /// **A `select` whose popup asks to be brought into view on every frame.** The `wheeled` axis
    /// on this component, and the arm `CONTEXT.md` forbids by name.
    ///
    /// The **second** wheel defect this component can make and the first that is not its own:
    /// [`a_copy_of_the_offset`] is `Copy`-only body, where the notch lands and the write dies;
    /// this one is `collection`'s unconditional reveal, where the notch lands, the write survives
    /// and the pull undoes it before anything draws. Both move the offset **0** in twenty clicks
    /// and both leave the screen identical, and a gate that plays one has not played the other.
    /// [`a_popup_that_never_reveals`] is the third arm.
    #[expect(
        clippy::too_many_arguments,
        reason = "the shipped signature plus the one arm, so the two are one call apart"
    )]
    pub fn a_popup_revealing_every_frame<'f, I: crate::ink::Ink>(
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
                list: crate::collect::CollShape {
                    reveal: crate::collect::Reveal::EveryFrame,
                    ..crate::collect::CollShape::RULE
                },
                ..Default::default()
            },
        )
    }

    /// **A `select` whose popup never asks to be brought into view.** The way to pass a wheel gate
    /// written in one direction, and it is not a fix.
    ///
    /// [`a_popup_revealing_every_frame`]'s third arm, for `crate::wheel`'s reason: deleting the call
    /// passes the loud half and loses the keyboard.
    #[expect(
        clippy::too_many_arguments,
        reason = "the shipped signature plus the one arm, so the two are one call apart"
    )]
    pub fn a_popup_that_never_reveals<'f, I: crate::ink::Ink>(
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
                list: crate::collect::CollShape {
                    reveal: crate::collect::Reveal::Never,
                    ..crate::collect::CollShape::RULE
                },
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
    /// *one hit entry per collection* from the other side:
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
    /// Paired with [`Sizing::FromTheDrawnExtent`](super::Sizing::FromTheDrawnExtent) this is
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
        let id = cx.id();
        super::draw_with(
            ink,
            cx,
            id,
            area,
            st,
            opts,
            Refused {
                regions,
                ..Refused::NONE
            },
        )
    }

    /// **[`crate::input::field_into`] with every refusal stated**, which is the entry
    /// [`crate::window`]'s three scenes take.
    ///
    /// It is a second entry rather than a widening of [`field_regions`], because that one is
    /// `crate::contract`'s and `crate::gates`'s already and its argument is the question those ask.
    /// Both are one call into `super::draw_with`, so there is one drawing path and not two —
    /// `crate::ink`'s rule, and the reason a `Refused` exists at all.
    #[track_caller]
    pub fn field_refused<I: crate::ink::Ink>(
        ink: &mut I,
        cx: &mut vitui_runtime::Ctx<'_, '_>,
        area: vitui_runtime::Rect,
        st: &mut crate::edit::Text,
        opts: &super::FieldOpts,
        refused: Refused,
    ) -> vitui_runtime::Response {
        let id = cx.id();
        super::draw_with(ink, cx, id, area, st, opts, refused)
    }

    // ── `form`'s one refused spelling ────────────────────────────────────────────────────────────

    /// **A form whose labels are collected every frame**, which is what a slice of records costs.
    ///
    /// [`crate::nav::cursor`] takes `&[&str]`. A form written over a slice of *records* — a label
    /// and a [`crate::edit::Text`] in one struct, which reads better and is the shape a reviewer
    /// expects — cannot hand it one without building it, and a `Vec<&str>` a frame is **one
    /// allocation a frame** against a budget of zero.
    ///
    /// The two arms differ by the one line below, and the picture is **identical** either way —
    /// which is why the allocation window is the only instrument that can tell them apart, exactly
    /// as it was for `crate::media::player`'s chapter list and for `crate::structure::rule`'s
    /// caption.
    #[track_caller]
    pub fn form_collecting_labels<I: crate::ink::Ink>(
        ink: &mut I,
        cx: &mut vitui_runtime::Ctx<'_, '_>,
        area: vitui_runtime::Rect,
        st: &mut super::FormState,
        labels: &[&str],
        texts: &mut [crate::edit::Text],
        opts: &super::FormOpts,
    ) -> vitui_runtime::Response {
        // **The one line.** Everything else is the shipped component.
        let collected: Vec<&str> = labels.to_vec();
        super::form_into(ink, cx, area, st, &collected, texts, opts)
    }

    // ── `slider`'s two refused spellings ─────────────────────────────────────────────────────────

    /// **What one arrow adds to a slider.**
    ///
    /// One argument, two arms, and the second is the arithmetic defect rather than the prose.
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
    /// construction here that needs the fact whose absence was measured.**
    ///
    /// [`grab`](super::grab) is `Response::local` over `Response::rect` and nothing else. Two thumbs
    /// on one track need a third answer that neither field carries: *which* thumb. Every way to get
    /// it is one of two:
    ///
    /// 1. **Remember which one the press landed nearer** — a press origin, which is exactly the
    ///    fifth cross-frame fact the headline is that drag capture does not need. It works, and it
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
    /// right. The defect, arriving inside the gate for it.
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
    /// *A text widget declares 79 regions against 621 for one per visible cluster.* Both arms
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

    /// **A focused field inside a scrolled form still places its caret**, which is runtime
    /// it was asked of the caller rather than of `Ctx::caret` on
    /// its own.
    ///
    /// `Ctx::caret_with` bounds-checks against `Ctx::area`, and inside a [`Ctx::scroll_scope`] that
    /// rectangle used to name content rows `0..h` while the window sat at `offset..offset + h`: so
    /// a caret at a **visible** content row was silently dropped, and a form inside a scroll area is
    /// the shape `scroll_scope`'s own rustdoc names as what it is right for. The runtime's gate
    /// asserts the rectangle and calls `cx.caret` by hand; this one puts a real `field` there,
    /// because a gate exercises a component where its author put it and an application puts it
    /// somewhere else.
    ///
    /// The field is on the first row the window shows, so its caret is the same root cell at both
    /// offsets — which is what makes the pair an equality rather than two unrelated numbers.
    #[test]
    fn a_focused_field_inside_a_scrolled_form_still_places_its_caret() {
        // **One call site, drawn twice**, for `one_field`'s reason: `Ctx::id` is
        // `Location::caller()`, so a field declared at two source lines is two widgets and the seat
        // would name one that is no longer drawing.
        fn one_scrolled_field(
            driver: &mut Driver,
            offset: i32,
            st: &mut Text,
            opts: &FieldOpts,
            seat: Option<vitui_runtime::Id>,
        ) -> vitui_runtime::Id {
            let form = vitui_runtime::Id::named("form");
            let view = Rect::new(0, 0, 20, 8);
            let mut id = None;
            driver.frame(|cx| {
                if let Some(seat) = seat {
                    cx.focus(seat);
                }
                cx.scroll_scope(form, view, (0, offset), (0, 1_000), |cx| {
                    // The field sits on the first content row the window is showing, so the caret
                    // it places lands on the screen's own first row whatever the offset is.
                    id = Some(field_with(cx, Rect::new(0, offset, 20, 1), st, opts).id);
                });
            });
            id.expect("a field answers with its id")
        }

        for offset in [0, 100] {
            let mut st = Text::of("hello".into(), WrapKind::Ruler);
            let mut driver = Driver::headless(20, 8).expect("a sink attaches");
            let opts = FieldOpts::default();

            // Nothing holds the keyboard on the first frame, and `Ctx::caret_with` refuses a caret
            // then — the runtime's own refusal, asserted rather than worked around.
            let id = one_scrolled_field(&mut driver, offset, &mut st, &opts, None);
            assert!(
                driver.inspect().caret().is_none(),
                "offset {offset}: unfocused, so no caret"
            );

            one_scrolled_field(&mut driver, offset, &mut st, &opts, Some(id));
            let caret = driver
                .inspect()
                .caret()
                .unwrap_or_else(|| panic!("offset {offset}: focused, so a caret"));
            assert_eq!(
                (caret.x, caret.y),
                (0, 0),
                "offset {offset}: the caret is on the row the window shows"
            );
        }
    }

    /// **A field writes a partition of its whole rectangle**, at every size the arithmetic runs out
    /// at. The two equalities on the component.
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
    /// most, split at the selection's two edges — so it is the only place the partition equalities can
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
    /// A field declares `Interest::SCROLL` and owns its offset, so a notch it did not consume
    /// is worse than one it never declared: it is the topmost region over its rectangle, so an
    /// enclosing `scroll_area` would never see the notch either. That defect
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
        // The arrangement and not a delta added to the offset, which has no second axis to
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
    /// *at cap (256 entries, 25 600 B) 160.67 µs / 0 allocs, emptied 160.71 µs / 0 allocs,
    /// same document, same screen*. The timing is the report; what is gated is that **every one of
    /// the readable counters is the same number** on the two arms, which is the sentence *the
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
    /// It is priced at *82.4 µs / 0 marked / 19 634 writes / 631 verbs / 79 regions / 0 merges /
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
    /// The 110-of-338 defect, at an amplitude of two: without the attribute the second field
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

    /// **The screen the drag is played on: `300` wide**, so the span is **299** and the two
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
    /// The shape is the four rules, and the half a test can reach is that `slider`,
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

        // **And the seam is deterministic**, which is all a second `Pen` can say. This assertion was
        // labelled *the `_into` seam is not the shipped draw* and could not have been: arms 0 and 1
        // draw through `Direct`, only arm 2's canvas is captured, and `Direct` writes into the
        // engine where **nothing reads a cell back** — so the equality compared
        // `slider_into` with `slider_into` and held whatever `slider` did. Found by the review of
        // components ticket 34, which had copied the shape into two more components.
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
            "two runs of the seam drew different pictures"
        );

        // **The claim the surface cannot make, made from the source instead**: the three spellings
        // are one body, which is a scan for the two calls that route into it.
        let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/input.rs"))
            .expect("this file");
        let section = crate::composed::section(
            &source,
            "// `slider` — the drag capture, and the row §17 froze at Tier 3",
        );
        assert!(!section.is_empty());
        for owed in [
            "slider_with(cx, area, value, &SliderOpts::default())",
            "slider_into(&mut Direct, cx, area, value, opts)",
        ] {
            assert!(
                crate::dense::declares(section, owed),
                "`{owed}` is not in the shipped slider, so the three spellings are not one body"
            );
        }
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
            .find("// `slider` — the drag capture")
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
    /// The three fractions, and the cadence had to be written around exactly as
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
    /// [`crate::media::player::scrub`] is where drag capture was measured, and it has one axis
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
    /// The two equalities. The values are chosen to put the thumb against both ends and in the
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
    /// ([`crate::counters::Counters::marked`]), so the reachable form is: carry the
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
    /// the warming discipline — *every allocation window in this workspace warms
    /// with two identical frames* — arriving on a counter rather than on an allocator, and the honest
    /// shape is the sequence: a gate that averaged it away would also hide a thumb that really did
    /// re-paint once a frame.
    ///
    /// What the 0 after it is evidence *of* is the relation — *a restyle is free only where
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
    /// the same boundary from the other side, where a container could not read `←` and
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
    /// *a chord types nothing*, on a component that reads the keyboard without a key map. The
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
    /// on **every** frame for ever — which is the one thing *zero marked on a steady
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
    /// — so the arrows are dead until either the caller focuses the slider
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
    /// The row reads *a slider's value is not an offset and no wheel event moves it*, and
    /// That finding is why this has to be a declaration rather than a missing
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
        // fifth cross-frame fact the headline is that drag capture does not need.
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

#[cfg(test)]
mod toggle_tests {
    use super::*;
    use crate::INVENTORY;
    use crate::chart::raster::RUNGS;
    use crate::counters::Tally;
    use crate::runner::{Canvas, Pen};
    use vitui_runtime::ctx::Driver;
    use vitui_runtime::theme::{CATPPUCCIN_MOCHA, Density, Theme};
    use vitui_runtime::{Button, Buttons, ColorDepth, Mods, Mouse, MouseKind};

    /// A pointer event at `(x, y)`.
    fn at(x: u16, y: u16, kind: MouseKind) -> Mouse {
        Mouse {
            x,
            y,
            kind,
            buttons: Buttons::NONE,
            mods: Mods::NONE,
            at: std::time::Instant::now(),
        }
    }

    /// A tally over one frame of `f`, on a `w` by `h` sink.
    fn tallied(w: u16, h: u16, f: impl FnOnce(&mut Tally, &mut Ctx<'_, '_>)) -> Tally {
        let mut driver = Driver::headless(w, h).expect("a sink cannot fail to attach");
        let mut tally = Tally::new();
        driver.frame(|cx| f(&mut tally, cx));
        tally
    }

    /// The options a `Toggle` gets when nothing else is said.
    fn opts(kind: Toggle) -> ToggleOpts {
        ToggleOpts {
            kind,
            ..ToggleOpts::default()
        }
    }

    /// **Criterion 1: three spellings, one machine, and rule 4 comes back from all of them.**
    ///
    /// `checkbox`'s signature is written out by name, so the half a test can reach is that the
    /// named spelling, the `_with` sibling and the `_into` seam are the *same* draw — or every gate
    /// below is measuring a second implementation.
    #[test]
    fn the_three_toggles_are_one_machine_and_each_answers_with_a_response() {
        for kind in Toggle::ALL {
            let mut painted: Vec<Canvas> = Vec::new();
            for arm in 0..3u8 {
                let mut driver = Driver::headless(24, 1).expect("a sink attaches");
                let mut pen = Pen::over(Canvas::new(24, 1));
                let mut on = true;
                let mut changed = true;
                driver.frame(|cx| {
                    let area = cx.area();
                    let resp = match (arm, kind) {
                        // Rule 3: the ninety-per-cent spelling names no options at all.
                        (0, Toggle::Check) => checkbox(cx, area, "wrap", &mut on),
                        (0, Toggle::Radio) => radio(cx, area, "wrap", &mut on),
                        (0, Toggle::Switch) => switch(cx, area, "wrap", &mut on),
                        (1, _) => toggle_with(cx, area, "wrap", &mut on, &opts(kind)),
                        _ => toggle_into(&mut pen, cx, area, "wrap", &mut on, &opts(kind)),
                    };
                    assert_eq!(resp.rect.w, 24);
                    changed = resp.changed;
                });
                assert!(!changed, "{kind:?} arm {arm} reported a change nobody made");
                // Rule 2: the state is the caller's `&mut bool` and nothing moved it.
                assert!(on, "{kind:?} arm {arm} flipped the caller's value");
                if arm == 2 {
                    painted.push(pen.into_canvas());
                }
            }

            // **And the seam is deterministic**, which is all a second `Pen` can say: `Direct`
            // writes into the engine and *nothing reads a cell back*, so there is no
            // surface to compare the shipped path against. Two `Pen`s compared is one path
            // compared with itself, and that is what the assertion below is — kept as the
            // determinism check it actually is rather than as the equality it was labelled.
            let mut driver = Driver::headless(24, 1).expect("a sink attaches");
            let mut pen = Pen::over(Canvas::new(24, 1));
            let mut on = true;
            driver.frame(|cx| {
                let area = cx.area();
                toggle_into(&mut pen, cx, area, "wrap", &mut on, &opts(kind));
            });
            assert_eq!(
                pen.into_canvas().diff(&painted.remove(0)).cells,
                0,
                "{kind:?}: two runs of the seam drew different pictures"
            );
        }

        // **The claim the surface cannot make, made from the source instead.** *The three named
        // spellings are the same draw* is exactly *they route into one body*, and that is a scan:
        // each of the three must reach `toggle_with`, and none of them may draw anything itself.
        //
        // Written as a `Pen` comparison it was **vacuous** — arms 0 and 1 draw through `Direct` and
        // only arm 2's canvas was captured, so the equality compared `toggle_into` with
        // `toggle_into` and held whatever `checkbox` did. Found by review, and the same shape is in
        // `crate::structure`'s rule and in this file's own slider.
        let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/input.rs"))
            .expect("this file");
        let toggles = crate::composed::section(
            &source,
            "// `checkbox`, `radio` and `switch` — the three Tier 2 toggles",
        );
        assert!(!toggles.is_empty());
        for owed in [
            "pub fn checkbox(cx: &mut Ctx<'_, '_>, area: Rect, label: &str, on: &mut bool) -> Response",
            "pub fn radio(cx: &mut Ctx<'_, '_>, area: Rect, label: &str, on: &mut bool) -> Response",
            "pub fn switch(cx: &mut Ctx<'_, '_>, area: Rect, label: &str, on: &mut bool) -> Response",
            "toggle_with(cx, area, label, on, &ToggleOpts::default())",
            "toggle_into(&mut Direct, cx, area, label, on, opts)",
        ] {
            assert!(
                crate::dense::declares(toggles, owed),
                "`{owed}` is not in the shipped toggles, so the three spellings are not one body"
            );
        }
        // **Three calls into the body and no fourth**, so a spelling that grew its own draw is a
        // count rather than a reading.
        assert_eq!(
            toggles.matches("toggle_with(").count(),
            4,
            "the three named spellings call `toggle_with` once each, and `toggle_with` declares              itself"
        );
    }

    /// **Criterion 7: every one of the three writes a partition of its whole rectangle.**
    ///
    /// The two equalities, swept where the arithmetic runs out — a one-cell rectangle, a rectangle
    /// narrower than the mark field, a label wider than the room it has, and a rectangle three rows
    /// tall so that the rows either side of the label's are somebody's too.
    #[test]
    fn a_toggle_writes_a_partition_of_its_whole_rectangle() {
        for kind in Toggle::ALL {
            for (w, h) in [
                (1u16, 1u16),
                (2, 1),
                (3, 1),
                (4, 2),
                (12, 1),
                (24, 3),
                (1, 5),
            ] {
                // **And at an origin that is not the screen's.** This is the arm this crate has now
                // had to add three times: `crate::collect`'s table drew its header from `x = 0`
                // rather than from the band it was handed, and every gate passed, because every
                // gate played at the origin — where the two agree.
                for (ox, oy) in [(0i32, 0i32), (5, 2)] {
                    for on in [false, true] {
                        for label in ["", "wrap", "a label far wider than this toggle is"] {
                            let mut value = on;
                            let at = Rect::new(ox, oy, w, h);
                            let (sw, sh) = (w + ox as u16, h + oy as u16);
                            let tally = tallied(sw, sh, |tally, cx| {
                                toggle_into(tally, cx, at, label, &mut value, &opts(kind));
                            });
                            assert_eq!(
                                tally.writes(),
                                tally.distinct(),
                                "{kind:?} {w}x{h}@{ox},{oy} on={on} `{label}`: a cell twice"
                            );
                            assert_eq!(
                                tally.distinct(),
                                u64::from(w) * u64::from(h),
                                "{kind:?} {w}x{h}@{ox},{oy} on={on} `{label}`: not a partition"
                            );
                            assert_eq!(
                                tally.asked(),
                                tally.reported(),
                                "{kind:?} {w}x{h}@{ox},{oy} on={on} `{label}`: a verb left"
                            );
                            // And the cells it wrote are **its own**, which `distinct` alone cannot
                            // say: the same count lands on the same number one column over.
                            for y in oy..oy + i32::from(h) {
                                for x in ox..ox + i32::from(w) {
                                    assert!(
                                        tally.touched(x, y),
                                        "{kind:?} {w}x{h}@{ox},{oy}: ({x}, {y}) unwritten"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// **Criterion 10, the glyph half: what the three demand is what the freeze says they demand.**
    ///
    /// Both directions, because a mark that grew a second glyph without the freeze moving and a
    /// freeze that grew a glyph the mark never draws are the same drift with the sign flipped.
    #[test]
    fn the_three_toggles_demand_exactly_what_the_freeze_says_they_do() {
        for kind in Toggle::ALL {
            let row = INVENTORY
                .iter()
                .find(|c| c.id == kind.id())
                .expect("the freeze has a row for each");
            let demanded: Vec<Glyph> = kind.mark().into_iter().collect();
            assert_eq!(
                row.glyphs,
                &demanded[..],
                "`{}`'s demand set and its mark disagree",
                kind.id()
            );
            assert_eq!(row.constructions, 1, "`{}` is one construction", kind.id());
        }
        // **`switch` is the row with the empty column**, and that is the claim the next test is
        // about rather than an omission.
        assert!(Toggle::Switch.mark().is_none());
    }

    /// **Three components at once: which of them still states itself with no glyph and no
    /// colour.**
    ///
    /// A checkbox and a radio carry their state on the **glyph** axis — `✓` becomes `x`, `•` becomes
    /// `*`, both still one cell and both still present, which is what *no spelling blank*
    /// buys. A switch carries its on three axes and only one of them is the palette: the two words,
    /// the side the knob sits on, and the face.
    ///
    /// So the count is **1 of 3**: at `GlyphSet::Ascii` the switch's two states differ in cells that
    /// carry no glyph at all, and the other two differ in exactly one cell whose content came out of
    /// the theme's table. The freeze's empty `glyphs` column for `switch` is that fact, and this is
    /// the number underneath it.
    #[test]
    fn a_switch_states_itself_at_every_rung_and_a_checkbox_needs_its_glyph() {
        let mut glyphless = 0usize;
        for kind in Toggle::ALL {
            // **The rungs come from `chart::raster::RUNGS` rather than being named here**, which is
            // that constant's own purpose: the exception for naming a repertoire is worth exactly
            // one file, and this is not it.
            for set in RUNGS {
                let painted = |on: bool| {
                    let mut driver = Driver::headless(12, 1).expect("a sink attaches");
                    driver.set_theme(
                        Theme::authored(&CATPPUCCIN_MOCHA, set, Density::default())
                            .resolve(ColorDepth::TrueColor),
                    );
                    let mut pen = Pen::over(Canvas::new(12, 1));
                    let mut value = on;
                    driver.frame(|cx| {
                        let area = cx.area();
                        toggle_into(&mut pen, cx, area, "x", &mut value, &opts(kind));
                    });
                    pen.into_canvas()
                };
                let off = painted(false);
                let on = painted(true);
                let differ = off.diff(&on).cells;
                assert!(
                    differ > 0,
                    "{kind:?} at {set:?}: the two states are the same picture"
                );
                if kind == Toggle::Switch {
                    // Two words of different widths and a knob that moved: more than the one cell a
                    // mark costs, and not one of them out of the glyph table.
                    assert!(
                        differ >= 2,
                        "{kind:?} at {set:?}: a switch differs in {differ} cells"
                    );
                } else {
                    assert_eq!(
                        differ, 1,
                        "{kind:?} at {set:?}: a mark is one cell and this is {differ}"
                    );
                }
            }
            if kind.mark().is_none() {
                glyphless += 1;
            }
        }
        assert_eq!(glyphless, 1, "one of the three states itself with no glyph");
    }

    /// **A click flips it, `Space` flips it, and a chord flips nothing.**
    ///
    /// The click is two frames, because the runtime sets `Response::clicked` on the release and the
    /// award is resolved from the index that has just drawn — `crate::media::player`'s cadence, met
    /// again. The chord is *a chord types nothing* on a component whose key map is two codes.
    #[test]
    fn a_click_flips_it_and_space_flips_it_and_a_chord_flips_nothing() {
        let mut driver = Driver::headless(12, 1).expect("a sink attaches");
        let mut on = false;
        let frame = |driver: &mut Driver, on: &mut bool| {
            driver.frame(|cx| {
                let area = cx.area();
                checkbox(cx, area, "x", on);
            });
        };

        // **A warm frame builds the index the pointer is resolved against** — nothing can be over a
        // region that has not been declared yet — and the `Move` is what puts the pointer
        // there, because the runtime reads its position once a frame before it walks the batch. A
        // `Down` at a position nothing has moved to is a press over nothing, which is
        // `crate::collect`'s own finding.
        frame(&mut driver, &mut on);
        driver.post_mouse(at(0, 0, MouseKind::Move));
        frame(&mut driver, &mut on);
        driver.post_mouse(at(0, 0, MouseKind::Down(Button::Left)));
        frame(&mut driver, &mut on);
        frame(&mut driver, &mut on);
        assert!(!on, "a press alone flipped it");
        // **And the release is a frame late in exactly the same way.** The frame that carries the
        // `Up` still reads `pressed`; `clicked` arrives on the one after it. A gate playing one
        // frame a phase would have measured the cadence and called it the mechanism —
        // `crate::media::player`'s finding, and `crate::input`'s slider met it one component ago.
        driver.post_mouse(at(0, 0, MouseKind::Up(Button::Left)));
        frame(&mut driver, &mut on);
        assert!(
            !on,
            "the frame carrying the release is not the frame of the click"
        );
        frame(&mut driver, &mut on);
        assert!(on, "a click did not flip it");

        // The press seated the focus, so the keyboard reaches it.
        driver.post_key(crate::keys::press_with(Code::Char(' '), Mods::NONE));
        frame(&mut driver, &mut on);
        assert!(!on, "`Space` did not flip it");

        driver.post_key(crate::keys::press_with(Code::Char(' '), Mods::CTRL));
        frame(&mut driver, &mut on);
        assert!(
            !on,
            "a chord flipped it, and a chord is not this widget's key"
        );

        driver.post_key(crate::keys::press_with(Code::Enter, Mods::NONE));
        frame(&mut driver, &mut on);
        assert!(on, "`Enter` did not flip it");
    }

    /// **Criterion 6: a radio *set* is `collection` at `Mode::Options`, not a second selection
    /// store.**
    ///
    /// An earlier draft spelled the mode `Mode::Radio`; it shipped as [`Mode::Options`] — *exactly
    /// one, and it can never become zero*, which is the whole difference from [`Mode::Single`]. The
    /// name is recorded rather than changed: a mode called `Radio` would be the thirteen match arms
    /// wearing one component's name.
    ///
    /// Two halves, because either alone is satisfiable by the wrong thing: the **set** really does
    /// hold exactly one at every gesture a set can be given, and the **standalone** row holds no
    /// selection at all — its whole cross-frame fact is the caller's `&mut bool`, which is four
    /// bytes plus nothing.
    #[test]
    fn a_radio_set_is_a_collection_and_not_a_second_store() {
        let mut st = CollState::new();
        st.sel.select_only(0);
        for gesture in [
            crate::collect::Gesture::Plain(2),
            crate::collect::Gesture::Toggle(2),
            crate::collect::Gesture::Nothing,
            crate::collect::Gesture::All,
            crate::collect::Gesture::Extend(3),
        ] {
            crate::collect::apply(Mode::Options, &mut st.sel, 4, gesture);
            assert_eq!(
                st.sel.count(),
                1,
                "{gesture:?}: a set at `Mode::Options` is exactly one and never zero"
            );
        }

        // And the standalone widget has no store to be a second of.
        assert_eq!(std::mem::size_of::<bool>(), 1);
        let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/input.rs"))
            .expect("this file");
        let toggles = crate::composed::section(
            &source,
            "// `checkbox`, `radio` and `switch` — the three Tier 2 toggles",
        );
        assert!(!toggles.is_empty());
        assert!(
            !crate::dense::declares(toggles, "CollState"),
            "the standalone radio holds a collection's selection store"
        );
    }

    /// **Criterion 7, the steady half: a toggle nobody touches changes 0 cells a frame.**
    ///
    /// `marked` is the engine's own counter and unreachable from this crate, so the reachable form
    /// is: carry the surface across frames with [`Pen::over`] and count the cells
    /// whose value changed.
    ///
    /// The sequence is `[n, 0, 0, …]` and there is **no hover frame in it**, because the pointer is
    /// never posted — which is the difference from `crate::input`'s slider, whose own steady figure
    /// reads `[40, 1, 0, …]` for the frame the hover arrives on.
    #[test]
    fn a_toggle_nobody_touches_changes_nothing_after_its_first_frame() {
        const FRAMES: usize = 20;
        for kind in Toggle::ALL {
            let mut driver = Driver::headless(24, 1).expect("a sink attaches");
            let mut pen = Pen::over(Canvas::new(24, 1));
            let mut on = true;
            let mut changed = Vec::new();
            for _ in 0..FRAMES {
                let before = pen.canvas().clone();
                driver.frame(|cx| {
                    let area = cx.area();
                    toggle_into(&mut pen, cx, area, "wrap", &mut on, &opts(kind));
                });
                pen.end_frame();
                changed.push(before.diff(pen.canvas()).cells);
            }
            assert_eq!(changed[0], 24, "{kind:?}: the first frame writes the row");
            assert!(
                changed[1..].iter().all(|&c| c == 0),
                "{kind:?}: a settled toggle changes cells — {changed:?}"
            );
        }
    }
}

#[cfg(test)]
mod form_tests {
    use super::*;
    use crate::counters::Tally;
    use crate::edit::Text;
    use vitui_runtime::Density;
    use vitui_runtime::ctx::Driver;
    use vitui_runtime::keys::{Chord, Code};

    /// The form every measurement below is taken over. Six labelled fields, and the widest label is
    /// what the label column derives from.
    const LABELS: [&str; 6] = ["name", "email", "role", "team", "location", "pronouns"];

    fn texts() -> [Text; 6] {
        [
            Text::input(),
            Text::input(),
            Text::input(),
            Text::input(),
            Text::input(),
            Text::input(),
        ]
    }

    /// **One frame of the form, from one call site**, seating the focus where asked.
    ///
    /// `form` is `#[track_caller]`, so a test that wrote the call twice would draw **two** forms:
    /// the second frame mints a different id, every row's id is keyed from it, and the id the first
    /// frame focused would not have drawn. The defect, met
    /// with a slider.
    fn frame(
        driver: &mut Driver,
        st: &mut FormState,
        texts: &mut [Text],
        area: Rect,
        opts: &FormOpts,
    ) -> Id {
        let mut id = None;
        driver.frame(|cx| id = Some(form_with(cx, area, st, &LABELS, texts, opts).id));
        id.expect("a frame ran")
    }

    /// **The partition rule over a container: every cell of the rectangle, exactly once**, at every size, at both
    /// scope arms, and at every entry height.
    ///
    /// The interesting sizes are the ones where the entries do **not** divide the height: what is
    /// left is the form's own tail, and leaving it out is the stale-tail axis on a screen with no
    /// list on it.
    #[test]
    fn a_form_writes_every_cell_of_its_rectangle_exactly_once() {
        for (w, h) in [(1, 1), (12, 1), (40, 7), (40, 6), (30, 20), (9, 5), (80, 3)] {
            for rows in [1u16, 2, 3] {
                for group in [true, false] {
                    for label_w in [0u16, 3, 200] {
                        let opts = FormOpts {
                            rows,
                            group,
                            label_w,
                            ..FormOpts::default()
                        };
                        let mut st = FormState::new();
                        let mut fields = texts();
                        // **Two cells of margin on every side**, because a component drawn at the
                        // screen's own edge is clipped by the screen and a verb that runs past its
                        // rectangle costs nothing a counter can see. `pagination` was writing 5
                        // cells into a 4-cell strip under a sweep that had none.
                        let mut driver =
                            Driver::headless(w + 4, h + 4).expect("a sink cannot fail to attach");
                        let mut tally = Tally::new();
                        driver.frame(|cx| {
                            form_into(
                                &mut tally,
                                cx,
                                Rect::new(2, 2, w, h),
                                &mut st,
                                &LABELS,
                                &mut fields,
                                &opts,
                            );
                        });
                        let cells = u64::from(w) * u64::from(h);
                        assert_eq!(
                            tally.writes(),
                            tally.distinct(),
                            "{w}x{h} rows={rows} group={group} label_w={label_w}: {} cells twice",
                            tally.writes() - tally.distinct()
                        );
                        assert_eq!(
                            tally.distinct(),
                            cells,
                            "{w}x{h} rows={rows} group={group} label_w={label_w}: not covered"
                        );
                    }
                }
            }
        }
    }

    /// **A field declines a cursor key it could not act on**, which is what makes a container above
    /// it able to hear one at all.
    ///
    /// The defect found in code that was already green, and it is the
    /// *declared and consumed nothing* shape on the keyboard axis. The one flag makes
    /// `input` and `textarea` one component, so `field` reads `Up`/`Down` as a caret row step — and
    /// a one-row `input` has no row to step to. It consumed the key anyway, and what that cost is
    /// invisible on the field: a `form` is a `Group` whose `nav::cursor` **never saw an arrow**, on
    /// a screen that rendered perfectly.
    ///
    /// Both directions, because a field that declined *every* arrow would be a textarea nobody can
    /// move the caret down inside.
    #[test]
    fn a_field_declines_a_cursor_key_it_could_not_act_on() {
        /// One frame, **from one call site**, drawing a field of `h` rows inside a scope that can
        /// hear what the field hands back. Answers the field's id and how many keys reached the
        /// level above — and the one call site is the whole reason this is a function: written
        /// twice, the second frame mints a different id and the planted focus reaches nobody.
        fn frame(driver: &mut Driver, st: &mut Text, h: u16) -> (Id, usize) {
            let sink = Id::named("outer");
            let (mut id, mut heard) = (None, 0);
            driver.frame(|cx| {
                cx.scope(sink, ScopeKind::Group, |cx| {
                    id = Some(field(cx, Rect::new(0, 0, 20, h), st).id);
                });
                while let Some(k) = cx.next_key(sink) {
                    heard += 1;
                    cx.decline(k);
                }
            });
            (id.expect("a frame ran"), heard)
        }

        // A one-row input: `Down` has nowhere to go, so the level above hears it.
        let mut one = Text::input();
        one.insert(20, "hello");
        let mut driver = Driver::headless(20, 4).expect("a sink cannot fail to attach");
        let (id, _) = frame(&mut driver, &mut one, 1);
        driver.plant(None, Some(id), None);
        driver.post_key(crate::keys::press(Chord::new(Code::Down)));
        assert_eq!(
            frame(&mut driver, &mut one, 1).1,
            1,
            "a one-row input swallowed a `Down` it could not act on, which is what makes every \
             container above it deaf"
        );

        // A textarea with somewhere to go keeps it — the same line, the other answer.
        let mut many = Text::textarea();
        many.insert(20, "one\ntwo\nthree");
        // The caret is placed by a **gesture**, which is the rule and the only way to place one:
        // row 0, column 0, so there is a row below it.
        many.click(20, 0, 0, false);
        let mut driver = Driver::headless(20, 4).expect("a sink cannot fail to attach");
        let (id, _) = frame(&mut driver, &mut many, 3);
        driver.plant(None, Some(id), None);
        driver.post_key(crate::keys::press(Chord::new(Code::Down)));
        assert_eq!(
            frame(&mut driver, &mut many, 3).1,
            0,
            "a textarea handed back a `Down` that moved its caret"
        );

        // And at the bottom of that same textarea it hands it back, because the answer is the
        // caret's own position rather than a flag on the kind.
        many.click(20, 2, 0, false);
        driver.plant(None, Some(id), None);
        driver.post_key(crate::keys::press(Chord::new(Code::Down)));
        assert_eq!(frame(&mut driver, &mut many, 3).1, 1);
    }

    /// **Criterion 4: a form introduces no mechanism `field`, `nav::cursor` and the ring do not
    /// already have** — and the strongest form of that is a `size_of`.
    ///
    /// [`FormState`] is exactly the one thing `nav::cursor` cannot borrow from the ring: the buffer
    /// that has to survive a frame. No cursor (the cursor is the focus), no selection, no per-row
    /// slot, no geometry. A field added here would move this number.
    #[test]
    fn a_form_adds_nothing_to_what_nav_cursor_already_needs() {
        assert_eq!(size_of::<FormState>(), size_of::<crate::nav::TypeAhead>());
        // And the cursor really is read back rather than kept: a fresh state and a state that has
        // been driven around the form are the same value once the buffer has lapsed.
        assert_eq!(FormState::new(), FormState::default());
    }

    /// **Criterion 4, the other half: the three names R3 gives, read out of the source.**
    ///
    /// A scan for absences alone goes green when the section is deleted, so both halves run — the
    /// three calls that must be there and the mechanisms that must not. The needles are the ones
    /// [`crate::composed`] runs over the whole Tier 2 table; what is here is the join between them
    /// and [`FORM_IS`], so that a fourth clause cannot arrive in one of the two places alone.
    #[test]
    fn a_form_is_the_three_things_r3_says_it_is() {
        assert_eq!(FORM_IS.len(), 3);
        let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/input.rs"))
            .expect("this file is here");
        let section = crate::composed::section(
            &source,
            "// `form` — R3 example: `field` + `nav::cursor` + the focus ring the draw builds",
        );
        assert!(!section.is_empty());
        for used in [
            "field_keyed(",
            "nav::cursor(",
            "ScopeKind::Group",
            "cx.is_focused(",
        ] {
            assert!(
                crate::dense::declares(section, used),
                "a form no longer reaches `{used}`"
            );
        }
        for minted in [
            "struct FormCursor",
            "CollState",
            "Selection",
            "cx.interact(",
            "nav::step(",
            "keys::text(",
        ] {
            assert!(
                !crate::dense::declares(section, minted),
                "a form mints `{minted}`, and §18's R3 is *no new mechanism*"
            );
        }
    }

    /// **A form is one tab stop and its fields are every ring entry — and ungrouped it is the other
    /// way round, with the arrows dead.**
    ///
    /// *`nav::cursor`'s placement is the decision, not its contents.* Both arms are
    /// measured because the difference is not cosmetic: a container receives the keys its children
    /// hand back **only** through a scope's after-the-body moment, and `ScopeKind` has three arms of
    /// which the other two are a modal and a code editor. So *no group* is not *a form without a
    /// group scope* — it is a form whose arrow keys reach nothing at all.
    #[test]
    fn a_grouped_form_is_one_tab_stop_and_an_ungrouped_one_is_every_field() {
        let area = Rect::new(0, 0, 40, 6);
        let mut counts = Vec::new();
        for group in [true, false] {
            let opts = FormOpts {
                group,
                ..FormOpts::default()
            };
            let mut st = FormState::new();
            let mut fields = texts();
            let mut driver = Driver::headless(40, 6).expect("a sink cannot fail to attach");
            let id = frame(&mut driver, &mut st, &mut fields, area, &opts);
            let f = driver.inspect();
            counts.push((f.ring().len(), f.stop_count(), f.tab_walk().count()));

            // The focus starts on the first row, and `Down` moves it — or does not.
            driver.plant(None, Some(Id::keyed(id, 0)), None);
            driver.post_key(crate::keys::press(Chord::new(Code::Down)));
            frame(&mut driver, &mut st, &mut fields, area, &opts);
            let moved = driver.inspect().focused() == Some(Id::keyed(id, 1));
            assert_eq!(
                moved, group,
                "group={group}: `Down` moved the focus to row 1 == {moved}, and it must be {group}"
            );
        }
        // Six ring entries either way; one tab stop grouped and six ungrouped, and the walk is the
        // stops.
        assert_eq!(counts, vec![(6, 1, 1), (6, 6, 6)]);
    }

    /// **Two arrows in one batch move two rows**, which is what carrying the cursor through the
    /// drain loop buys.
    ///
    /// `Ctx::next_key` closes the level's queue on a **decline**, not on a take, so a form can be
    /// handed several keys in one frame — a terminal folds an ordinary run into one batch, which is
    /// `route::batch_len`'s whole job. A `Cursor` read once before the loop computes the second
    /// arrow from the position the first one left behind, and two `Down`s move **one** row on a
    /// screen where nothing else is wrong.
    #[test]
    fn two_arrows_in_one_batch_move_two_rows() {
        let area = Rect::new(0, 0, 40, 6);
        let opts = FormOpts::default();
        let mut st = FormState::new();
        let mut fields = texts();
        let mut driver = Driver::headless(40, 6).expect("a sink cannot fail to attach");
        let id = frame(&mut driver, &mut st, &mut fields, area, &opts);

        driver.plant(None, Some(form_row_id(id, 0)), None);
        driver.post_key(crate::keys::press(Chord::new(Code::Down)));
        driver.post_key(crate::keys::press(Chord::new(Code::Down)));
        frame(&mut driver, &mut st, &mut fields, area, &opts);
        assert_eq!(
            driver.inspect().focused(),
            Some(form_row_id(id, 2)),
            "two `Down`s in one batch moved one row, which is a `Cursor` read once before the drain \
             loop"
        );
    }

    /// **Criterion 5: the walk repeats no id, and reaches every stop unless a trap is standing** —
    /// the third refinement, run over a form.
    ///
    /// The existing instrument (`tests/gates.rs`) runs it over a fixture of bare `interact` calls.
    /// This one runs it over the component named as that reduction's own example, which is the population
    /// the obligation is about — and the ungrouped arm is the one that makes it say something, since
    /// a grouped form is one stop and *reaches every stop* is nearly free at one.
    #[test]
    fn the_walk_over_a_form_repeats_no_id_and_reaches_every_stop_unless_a_trap_is_standing() {
        let area = Rect::new(0, 0, 40, 6);
        let opts = FormOpts {
            group: false,
            ..FormOpts::default()
        };
        let mut st = FormState::new();
        let mut fields = texts();
        let mut driver = Driver::headless(40, 6).expect("a sink cannot fail to attach");
        frame(&mut driver, &mut st, &mut fields, area, &opts);

        let f = driver.inspect();
        let walk: Vec<Id> = f.tab_walk().collect();
        let stops: Vec<Id> = f.stop_ids().collect();
        let traps: Vec<Id> = f.trap_scopes().collect();
        let mut seen = walk.clone();
        seen.sort_by_key(|id| id.raw());
        let before = seen.len();
        seen.dedup();
        assert_eq!(before - seen.len(), 0, "the walk repeats an id");
        assert!(traps.is_empty(), "no trap is standing");
        assert_eq!(walk.len(), stops.len(), "{} of {}", walk.len(), stops.len());
        assert_eq!(walk.len(), 6);

        // **The exception, named rather than excused.** A modal over the form is a `Trap`, and the
        // walk correctly stops inside it — with `Frame::trap_scopes` as the only thing that can say
        // so.
        let mut st = FormState::new();
        let mut fields = texts();
        let mut driver = Driver::headless(40, 8).expect("a sink cannot fail to attach");
        driver.frame(|cx| {
            form_with(cx, area, &mut st, &LABELS, &mut fields, &opts);
            cx.scope(Id::named("confirm"), ScopeKind::Trap, |cx| {
                let ok = Id::named("ok");
                let _ = cx.interact(ok, Rect::new(0, 6, 10, 1), Interest::FOCUS);
                cx.focus(ok);
            });
        });
        // The second frame is the one the trap stands on: a trap that stood *last* frame is what
        // refuses delivery outside itself.
        driver.frame(|cx| {
            form_with(cx, area, &mut st, &LABELS, &mut fields, &opts);
            cx.scope(Id::named("confirm"), ScopeKind::Trap, |cx| {
                let ok = Id::named("ok");
                let _ = cx.interact(ok, Rect::new(0, 6, 10, 1), Interest::FOCUS);
            });
        });
        let f = driver.inspect();
        let walk = f.tab_walk().count();
        let stops = f.stop_ids().count();
        assert_eq!(f.trap_scopes().count(), 1, "the trap names itself");
        assert!(
            walk < stops,
            "the walk reached {walk} of {stops} stops with a trap standing, which is the whole \
             exception §21 refuses to loosen the gate for"
        );
        assert_eq!((walk, stops), (1, 7));
    }

    /// **Criterion 6: a chord pressed into every focusable in a form types nothing.**
    ///
    /// The row 5, over a real form rather than over the seven **sinks** `crate::keys` stands it on
    /// — which is what that module's own note says it is waiting for: *none of the seven exists in
    /// this crate*, and one of the seven is `form`. Every field is focused in turn, `Ctrl+S` is
    /// pressed into it, and the two halves are one statement: nothing lands in the buffer **and** the
    /// key reaches the application.
    #[test]
    fn a_chord_pressed_into_every_focusable_in_a_form_types_nothing() {
        let area = Rect::new(0, 0, 40, 6);
        let opts = FormOpts::default();
        let mut st = FormState::new();
        let mut fields = texts();
        let mut driver = Driver::headless(40, 6).expect("a sink cannot fail to attach");
        let id = frame(&mut driver, &mut st, &mut fields, area, &opts);

        let mut reached = 0;
        for row in 0..LABELS.len() {
            driver.plant(None, Some(Id::keyed(id, row as u64)), None);
            driver.post_key(crate::keys::press(Chord::key('s').ctrl()));
            frame(&mut driver, &mut st, &mut fields, area, &opts);
            reached += driver.unhandled().len();
        }
        assert_eq!(
            reached,
            LABELS.len(),
            "a chord did not reach the application"
        );
        for (i, t) in fields.iter().enumerate() {
            assert!(
                t.text().is_empty(),
                "row {i} holds {:?} after six accelerators",
                t.text()
            );
        }

        // And the same drive with a **letter** does type, so the gate above is not measuring a form
        // that has stopped accepting anything.
        driver.plant(None, Some(Id::keyed(id, 0)), None);
        driver.post_key(crate::keys::press(Chord::key('s')));
        frame(&mut driver, &mut st, &mut fields, area, &opts);
        assert_eq!(fields[0].text(), "s");
    }

    /// **Criterion 7: `Compact` against `Cosy` on the same form** — both counts reported, neither
    /// writes a cell twice, and the widget count that falls off the bottom is stated.
    ///
    /// Density is theme data and it changes rectangles, and [`crate::frame::block`] is where that
    /// lands — so the form is drawn inside a panel, which is the only way a density can
    /// reach it at all. What falls off the bottom is `LABELS.len()` minus the hit entries the frame
    /// declared, which is a **count of drawn fields** rather than an inference from a height.
    #[test]
    fn the_same_form_stands_fewer_fields_at_cosy_and_neither_writes_a_cell_twice() {
        let measure = |density: Density| -> (u64, u64, usize) {
            let mut driver = crate::runner::driver_at(30, 8, density);
            let mut st = FormState::new();
            let mut fields = texts();
            let mut tally = Tally::new();
            driver.frame(|cx| {
                let area = cx.area();
                let interior = crate::frame::block_into(
                    &mut tally,
                    cx,
                    area,
                    &crate::frame::BlockOpts {
                        title: " who ",
                        ..crate::frame::BlockOpts::default()
                    },
                );
                form_into(
                    &mut tally,
                    cx,
                    interior,
                    &mut st,
                    &LABELS,
                    &mut fields,
                    &FormOpts::default(),
                );
            });
            (
                tally.writes(),
                tally.distinct(),
                driver.inspect().hits().len(),
            )
        };

        let (cw, cd, compact) = measure(Density::Compact);
        let (yw, yd, cosy) = measure(Density::Cosy);
        assert_eq!(cw, cd, "Compact wrote {} cells twice", cw - cd);
        assert_eq!(yw, yd, "Cosy wrote {} cells twice", yw - yd);
        assert_eq!((cw, yw), (240, 240), "both densities cover the same screen");
        // **The number that is stated rather than hidden.** One cell of padding against two, on both
        // edges, is two rows of interior — and two rows of a one-row entry is two fields.
        assert_eq!((compact, cosy), (4, 2));
        assert_eq!(LABELS.len() - cosy, 4);
        assert_eq!(compact - cosy, 2, "two rows of padding is two fields");
    }

    /// **Two forms on one screen are two sets of ids and merge nothing**, on a component
    /// whose children's ids are its own arithmetic.
    ///
    /// This is the half `field_keyed` makes possible and dangerous in the same move: every row's id
    /// is `Id::keyed(form_id, row)`, so a form that forgot `#[track_caller]` would give two forms
    /// one base and **twelve widgets six ids**, with the second form's fields inert on a screen that
    /// renders perfectly.
    #[test]
    fn two_forms_on_one_screen_are_two_sets_of_ids_and_merge_nothing() {
        let mut st = (FormState::new(), FormState::new());
        let mut a = texts();
        let mut b = texts();
        let mut driver = Driver::headless(40, 12).expect("a sink cannot fail to attach");
        let mut ids = (None, None);
        driver.frame(|cx| {
            ids.0 = Some(form(cx, Rect::new(0, 0, 40, 6), &mut st.0, &LABELS, &mut a).id);
            ids.1 = Some(form(cx, Rect::new(0, 6, 40, 6), &mut st.1, &LABELS, &mut b).id);
        });
        assert_ne!(ids.0, ids.1, "two forms are one form");
        let f = driver.inspect();
        assert_eq!(f.hits().len(), 12, "twelve fields, twelve regions");
        assert_eq!(f.ids().merges(), 0);
    }
}
