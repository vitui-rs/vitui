//! **A component that opens a layer** — the one shape whose state cannot all live in one place.
//!
//! # An overlay body is a closure the frame keeps, so it cannot borrow the caller's frame locals
//!
//! `Ctx::overlay` takes a body that is `FnMut(&mut Ctx) + 'f`, and it runs in a **second pass**,
//! after the base pass has finished and every context in that subtree has been dropped. So the body
//! cannot capture anything borrowed for the duration of the owner's call — including the `&mut`
//! state the owner itself is holding.
//!
//! The way out is not a cleverer lifetime. It is to notice that an owner and its popup are two
//! things with two lifetimes and to give them two states: [`MenuState`] belongs to the owner and
//! says whether the menu is open, [`MenuPopup`] belongs to the body and carries what the body
//! decided. The application owns both and passes one into each. The answer therefore arrives **one
//! frame after the click**, which is not a defect to hide: the frame that opens a layer is the
//! frame that draws it, and the frame that reads what the user did in it is the next one.
//!
//! # Every row in the body needs an id of its own, and a loop does not give it one
//!
//! `Ctx::id` is `Location::caller()`. A loop drawing ten rows from one source line mints **one** id
//! ten times, so ten rows share a hover, a press and a focus — and the screen looks perfect while
//! the wrong row answers. [`Ctx::with_key`] is what separates them, and the key is the caller's own
//! stable number for the row, never the loop index if the rows can be reordered.

use vitui_runtime::layout::text as measure;
use vitui_runtime::overlay::OverlayOpts;
use vitui_runtime::{Ctx, Glyph, Interest, Rect, Response, Role};

use crate::ink::{Direct, Ink};
use crate::pill::{Faces, face_of};

/// **The owner's state:** whether the menu is showing.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct MenuState {
    /// Whether the layer is open.
    pub open: bool,
}

/// **The body's state:** where its cursor is, and what it answered.
///
/// A separate value from [`MenuState`] because the body outlives the owner's call — see this
/// module's header. The application reads `chosen` after the frame and takes it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct MenuPopup {
    /// The row under the pointer, kept so the highlight survives a frame with no motion on it.
    pub cursor: usize,
    /// What the body answered, if it answered.
    pub chosen: Option<usize>,
}

/// [`menu`]'s options.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MenuOpts {
    /// The faces of the button.
    pub faces: Faces,
    /// How wide the layer is. There is no measure pass in the overlay path, so a size is stated.
    pub width: u16,
}

impl Default for MenuOpts {
    fn default() -> MenuOpts {
        MenuOpts {
            faces: Faces::default(),
            width: 18,
        }
    }
}

/// **A button that drops a list under itself.**
///
/// `popup` and `items` are borrowed for the frame, because the body the frame keeps is what reads
/// them. `chosen` is answered on the frame **after** the click that chose it.
#[track_caller]
pub fn menu<'f>(
    cx: &mut Ctx<'f, '_>,
    area: Rect,
    label: &str,
    st: &mut MenuState,
    popup: &'f mut MenuPopup,
    items: &'f [&'f str],
) -> Response {
    menu_with(cx, area, label, st, popup, items, &MenuOpts::default())
}

/// [`menu`], with the options spelled out.
#[track_caller]
pub fn menu_with<'f>(
    cx: &mut Ctx<'f, '_>,
    area: Rect,
    label: &str,
    st: &mut MenuState,
    popup: &'f mut MenuPopup,
    items: &'f [&'f str],
    opts: &MenuOpts,
) -> Response {
    let id = cx.id();

    // Last frame's answer, read before the body takes the state again. This is the whole of the
    // one-frame round trip, written out.
    if popup.chosen.is_some() {
        st.open = false;
    }

    let resp = cx.interact(
        id,
        area,
        Interest::CLICK.with(Interest::HOVER).with(Interest::FOCUS),
    );
    if resp.clicked {
        st.open = !st.open;
    }

    draw_button(&mut Direct, cx, area, label, &resp, st.open, opts);

    if st.open && !items.is_empty() {
        let height = u16::try_from(items.len()).unwrap_or(u16::MAX);
        let rest = cx.theme().paint(Role::Face);
        let hot = cx.theme().paint(Role::Selection);
        // The body is queued here and runs after the base pass. Everything it needs is either
        // `Copy` and moved in, or borrowed for `'f`.
        cx.overlay(
            id,
            area,
            OverlayOpts::sized(opts.width, height),
            move |cx| {
                let layer = cx.area();
                for (i, item) in items.iter().enumerate() {
                    let row = Rect::new(0, i as i32, layer.w, 1);
                    // **One key per row.** Without this every row is the same widget.
                    let hit = cx.with_key(i as u64, |cx| {
                        let rid = cx.id();
                        cx.interact(rid, row, Interest::CLICK.with(Interest::HOVER))
                    });
                    if hit.hovered {
                        popup.cursor = i;
                    }
                    if hit.clicked {
                        popup.chosen = Some(i);
                    }
                    let paint = if popup.cursor == i { hot } else { rest };
                    let head = measure::truncate(item, layer.w);
                    let n = cx.text(row.x, row.y, head, paint).cells;
                    cx.fill(
                        Rect::new(row.x + i32::from(n), row.y, layer.w - n, 1),
                        " ",
                        paint,
                    );
                }
            },
        );
    }

    resp
}

/// The shut face: a label with a chevron on the right.
fn draw_button<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    label: &str,
    resp: &Response,
    open: bool,
    opts: &MenuOpts,
) {
    if area.is_empty() {
        return;
    }
    let face = if open {
        opts.faces.active
    } else {
        face_of(resp, &opts.faces)
    };
    let paint = cx.theme().paint(face);
    let chevron = cx.theme().glyph(if open {
        Glyph::ArrowUp
    } else {
        Glyph::ArrowDown
    });

    let row = area.y;
    for y in (area.y + 1)..area.bottom() {
        ink.fill(cx, Rect::new(area.x, y, area.w, 1), " ", paint);
    }
    let room = area.w.saturating_sub(2);
    let head = measure::truncate(label, room);
    let n = ink.text(cx, area.x, row, head, paint);
    ink.fill(
        cx,
        Rect::new(area.x + i32::from(n), row, room - n, 1),
        " ",
        paint,
    );
    if area.w >= 2 {
        ink.fill(
            cx,
            Rect::new(area.x + i32::from(area.w) - 2, row, 1, 1),
            " ",
            paint,
        );
        ink.text(cx, area.x + i32::from(area.w) - 1, row, chevron, paint);
    }
}
