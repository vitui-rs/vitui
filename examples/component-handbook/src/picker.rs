//! **A list**, which is where the interesting rules are.
//!
//! # The frame's cost is proportional to what is visible, never to what exists
//!
//! This is the invariant the whole stack is built to keep, and a list is where an author breaks it.
//! The loop below runs over `offset..offset + h` — the rows on screen — and never over `items`. A
//! list of a million rows and a list of ten cost the same frame, and the only thing that makes that
//! true is that nobody iterated the data.
//!
//! It follows that a component may not ask the data anything it cannot ask in `O(1)`. *How tall is
//! row `k`* and *what is the `k`-th visible row* are the two questions, and a caller that sorts,
//! filters or folds owes an index that answers them — built once when the data changes, not once a
//! frame.
//!
//! # The offset is the caller's, and the direction is not a matter of taste
//!
//! Nothing in the runtime stores a scroll position. The component is handed one, moves it, and
//! hands it back, because a component that owned it would have nowhere to keep it and no way to let
//! an application restore a session. The wheel's arithmetic is `offset + resp.scrolled.1`, clamped
//! — a negation there scrolls the wrong way at every offset except zero, where all four spellings
//! agree, which is why it survives so long in a screenshot.
//!
//! # A component declares which way it can still move, not which axis it has
//!
//! [`Scrollable::between`] takes the current offset and the maximum, and answers *the directions
//! that are still available*. A list at its bottom that reports the y axis as movable eats every
//! downward notch, and the pane around it stops scrolling. The signature is what makes that hard to
//! write by accident.

use vitui_runtime::keys::{ActionId, Chord, Code, MatchMode, Pressed};
use vitui_runtime::layout::text as measure;
use vitui_runtime::{Binding, Ctx, Interest, Rect, Response, Role, Scrollable};

use crate::ink::{Direct, Ink};

/// Move the cursor up one row.
pub const UP: ActionId = 1;
/// Move it down one row.
pub const DOWN: ActionId = 2;
/// A screenful up.
pub const PAGE_UP: ActionId = 3;
/// A screenful down.
pub const PAGE_DOWN: ActionId = 4;
/// The first row.
pub const FIRST: ActionId = 5;
/// The last row.
pub const LAST: ActionId = 6;
/// Answer the row the cursor is on.
pub const CHOOSE: ActionId = 7;

/// **What a picker answers.** A vertical list, so it reads the vertical arrows and leaves `←` and
/// `→` alone: a group declares its own axis, and a key a component does not use is a key the
/// container around it can bind.
pub const BINDINGS: &[Binding] = &[
    Binding::new(&[Chord::new(Code::Up), Chord::key('k')], UP, "Up"),
    Binding::new(&[Chord::new(Code::Down), Chord::key('j')], DOWN, "Down"),
    Binding::new(&[Chord::new(Code::PageUp)], PAGE_UP, "Page up"),
    Binding::new(&[Chord::new(Code::PageDown)], PAGE_DOWN, "Page down"),
    Binding::new(&[Chord::new(Code::Home)], FIRST, "First"),
    Binding::new(&[Chord::new(Code::End)], LAST, "Last"),
    Binding::new(&[Chord::new(Code::Enter)], CHOOSE, "Choose"),
];

/// **Everything a picker remembers, which the caller owns.**
///
/// Two numbers. They are here rather than inside the component because there is no *inside*: the
/// runtime keeps no retained structure, and a value that must survive a frame belongs to whoever
/// is calling.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct PickerState {
    /// Which row the keyboard is on.
    pub cursor: usize,
    /// The first visible row.
    pub offset: i32,
}

/// [`picker`]'s options.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PickerOpts {
    /// The row the cursor is on.
    pub selected: Role,
    /// Every other row.
    pub row: Role,
    /// What the component declares, beside the wheel it always takes.
    pub interest: Interest,
}

impl Default for PickerOpts {
    fn default() -> PickerOpts {
        PickerOpts {
            selected: Role::Selection,
            row: Role::Body,
            interest: Interest::CLICK.with(Interest::FOCUS),
        }
    }
}

/// What a picker answered this frame, beside its [`Response`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Picked {
    /// The declaration and the pointer facts.
    pub resp: Response,
    /// The row a click or an `Enter` chose, if one did.
    pub chosen: Option<usize>,
}

fn action_of(key: &Pressed) -> Option<ActionId> {
    BINDINGS
        .iter()
        .find(|b| b.matches(key, MatchMode::Masked))
        .map(Binding::action)
}

/// **A vertical list of labels with a cursor, a wheel and a keyboard.**
#[track_caller]
pub fn picker(cx: &mut Ctx<'_, '_>, area: Rect, items: &[&str], st: &mut PickerState) -> Picked {
    picker_with(cx, area, items, st, &PickerOpts::default())
}

/// [`picker`], with the options spelled out.
#[track_caller]
pub fn picker_with(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    items: &[&str],
    st: &mut PickerState,
    opts: &PickerOpts,
) -> Picked {
    picker_into(&mut Direct, cx, area, items, st, opts)
}

/// [`picker`], drawing through an [`Ink`].
#[track_caller]
pub fn picker_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    items: &[&str],
    st: &mut PickerState,
    opts: &PickerOpts,
) -> Picked {
    let id = cx.id();
    let rows = i32::from(area.h);
    // `extent - viewport`, floored at zero. It is a *bound on the offset* and not the content
    // height: the same quantity, so they are computed in one place.
    let max = (items.len() as i32 - rows).max(0);
    st.offset = st.offset.clamp(0, max);
    if !items.is_empty() {
        st.cursor = st.cursor.min(items.len() - 1);
    }

    // The declaration says where it can still go, from where it currently is.
    let resp = cx.scrollable(
        id,
        area,
        opts.interest,
        Scrollable::between((0, st.offset), (0, max)),
    );

    // ── The wheel ────────────────────────────────────────────────────────────────────────────
    st.offset = (st.offset + resp.scrolled.1).clamp(0, max);

    // ── The pointer ──────────────────────────────────────────────────────────────────────────
    let mut chosen = None;
    if let Some((_, ly)) = resp.local
        && resp.clicked
    {
        let row = st.offset + ly;
        if row >= 0 && (row as usize) < items.len() {
            st.cursor = row as usize;
            chosen = Some(st.cursor);
        }
    }

    // ── The keyboard ─────────────────────────────────────────────────────────────────────────
    while let Some(key) = cx.next_key(id) {
        match action_of(&key) {
            Some(CHOOSE) if !items.is_empty() => chosen = Some(st.cursor),
            Some(action) if !items.is_empty() => {
                let last = items.len() - 1;
                let page = (rows - 1).max(1) as usize;
                st.cursor = match action {
                    UP => st.cursor.saturating_sub(1),
                    DOWN => (st.cursor + 1).min(last),
                    PAGE_UP => st.cursor.saturating_sub(page),
                    PAGE_DOWN => (st.cursor + page).min(last),
                    FIRST => 0,
                    LAST => last,
                    _ => st.cursor,
                };
            }
            // A key this component has no use for goes back, and so does every key at all when the
            // list is empty: a cursor with nowhere to go owns nothing.
            _ => cx.decline(key),
        }
    }

    // **Keep the cursor on screen, by moving the offset and not the cursor.** The runtime has a
    // two-frame route for this — `Ctx::request_into_view` leaves a request and the offset's owner
    // takes it with `take_into_view` on the next frame — which is what a *scope* whose offset lives
    // somewhere else has to use. Here the component was handed the offset, so it can do the
    // arithmetic now and the reveal lands on this frame.
    let cursor = st.cursor as i32;
    if cursor < st.offset {
        st.offset = cursor;
    } else if cursor >= st.offset + rows {
        st.offset = cursor - rows + 1;
    }
    st.offset = st.offset.clamp(0, max);

    draw(ink, cx, area, items, st, &resp, opts);
    Picked { resp, chosen }
}

/// **The visible rows and nothing else.**
fn draw<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    items: &[&str],
    st: &PickerState,
    resp: &Response,
    opts: &PickerOpts,
) {
    let row_paint = cx.theme().paint(opts.row);
    let sel_paint = cx.theme().paint(if resp.focused {
        opts.selected
    } else {
        Role::Dim
    });

    for i in 0..i32::from(area.h) {
        let y = area.y + i;
        let index = st.offset + i;
        let row = Rect::new(area.x, y, area.w, 1);
        let Some(label) = usize::try_from(index).ok().and_then(|k| items.get(k)) else {
            // Past the end of the data. It is still this component's rectangle, so it is still this
            // component's to paint: a row left alone keeps whatever the previous frame put there.
            ink.fill(cx, row, " ", row_paint);
            continue;
        };
        let paint = if usize::try_from(index) == Ok(st.cursor) {
            sel_paint
        } else {
            row_paint
        };
        let head = measure::truncate(label, area.w);
        let n = ink.text(cx, row.x, y, head, paint);
        ink.fill(
            cx,
            Rect::new(row.x + i32::from(n), y, area.w - n, 1),
            " ",
            paint,
        );
    }
}
