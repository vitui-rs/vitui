//! **A component that reacts** — a label on a face that knows about the pointer and the focus.
//!
//! Everything new here is in four lines of [`pill_into`]: mint an id, declare a rectangle and what
//! it is interested in, read the [`Response`] the frame hands back, and resolve the five facts it
//! carries into **one** paint before a single cell is written.
//!
//! # One paint, resolved first, and never a restyle afterwards
//!
//! The tempting shape is to draw the label and then paint a hover or a focus over it. It works, and
//! it costs a cell twice on every frame a pointer is anywhere near the component. A face is a
//! function of the response, so it is computed once, up front, and every cell is written once.
//!
//! # A colour is not a guarantee, so the component asks
//!
//! On a terminal with no colour at all — or with a theme whose focus role and face role quantise to
//! the same wire colour — a focused pill drawn in `Role::Focus` is a pill drawn in `Role::Face`,
//! and the user cannot tell which one has the keyboard. [`Theme::roles_differ_on_wire`] is the
//! question, asked at draw time, and the answer decides between a paint and a marker glyph. Nothing
//! about that is a fallback for old terminals: the same code takes the marker arm under a
//! monochrome theme on a brand-new one.

use vitui_runtime::layout::text as measure;
use vitui_runtime::{Ctx, Glyph, Interest, Rect, Response, Role};

use crate::ink::{Direct, Ink};

/// The four faces a pill can wear, as roles.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Faces {
    /// At rest.
    pub rest: Role,
    /// Under the pointer.
    pub hover: Role,
    /// While a button is down on it.
    pub active: Role,
    /// While it holds the keyboard.
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

/// [`pill`]'s options.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PillOpts {
    /// The faces, resolved by [`face_of`].
    pub faces: Faces,
    /// What the component declares.
    ///
    /// `HOVER` is not decoration: `hovered` is resolved only for a region that asked for the
    /// pointer, so a pill drawing a hover face without declaring hover has a branch nothing can
    /// take. `FOCUS` costs no tracking at all — it declares a tab stop, and the keyboard is
    /// delivered whatever the mouse is doing.
    pub interest: Interest,
}

impl Default for PillOpts {
    fn default() -> PillOpts {
        PillOpts {
            faces: Faces::default(),
            interest: Interest::CLICK.with(Interest::HOVER).with(Interest::FOCUS),
        }
    }
}

/// **Which face a response is wearing.** Public because a container that draws its own rows wants
/// the same answer without drawing a pill.
///
/// The order is the decision: pressed beats hovered, because a pointer that is pressing is also
/// hovering and the press is the more specific fact.
pub fn face_of(resp: &Response, faces: &Faces) -> Role {
    if resp.pressed {
        faces.active
    } else if resp.hovered {
        faces.hover
    } else {
        faces.rest
    }
}

/// **A label on a face that reacts to the pointer and the keyboard.**
#[track_caller]
pub fn pill(cx: &mut Ctx<'_, '_>, area: Rect, label: &str) -> Response {
    pill_with(cx, area, label, &PillOpts::default())
}

/// [`pill`], with the options spelled out.
#[track_caller]
pub fn pill_with(cx: &mut Ctx<'_, '_>, area: Rect, label: &str, opts: &PillOpts) -> Response {
    pill_into(&mut Direct, cx, area, label, opts)
}

/// [`pill`], drawing through an [`Ink`].
#[track_caller]
pub fn pill_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    label: &str,
    opts: &PillOpts,
) -> Response {
    let id = cx.id();
    let resp = cx.interact(id, area, opts.interest);
    pill_drawn(ink, cx, area, label, &resp, opts);
    resp
}

/// **[`pill`]'s drawing half, with the [`Response`] supplied rather than declared.**
///
/// Two callers want this. A test wants it because a headless driver can be handed a `Response` that
/// says *hovered* without a pointer existing, and that is the only way to stand up the hover screen
/// at all. A container wants it because a list row that has already declared its own region should
/// not declare a second one for the pill inside it — two hit entries under one rectangle
/// double-count it.
pub fn pill_drawn<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    label: &str,
    resp: &Response,
    opts: &PillOpts,
) -> Role {
    if area.is_empty() {
        return opts.faces.rest;
    }

    let base = face_of(resp, &opts.faces);
    // **Ask, rather than assume.** If the two roles come out as the same bytes on this terminal at
    // this theme's tier, the paint carries no information and a glyph has to.
    let paint_says_focus = resp.focused && cx.theme().roles_differ_on_wire(base, opts.faces.focus);
    let face = if paint_says_focus {
        opts.faces.focus
    } else {
        base
    };
    let mark_focus = resp.focused && !paint_says_focus;

    let paint = cx.theme().paint(face);
    let marker = cx.theme().glyph(Glyph::Bullet);
    let ellipsis = cx.theme().glyph(Glyph::Ellipsis);

    // One row carries the label; the rows either side are face. Every cell of the rectangle is
    // written, and none of it twice.
    let label_row = area.y + (i32::from(area.h) - 1) / 2;
    for y in area.y..area.bottom() {
        if y != label_row {
            ink.fill(cx, Rect::new(area.x, y, area.w, 1), " ", paint);
        }
    }

    let lead: u16 = if mark_focus { 2 } else { 0 };
    let room = area.w.saturating_sub(lead);
    let head = measure::truncate(label, room);
    let elided = head.len() < label.len();
    let head = if elided {
        measure::truncate(label, room.saturating_sub(1))
    } else {
        head
    };
    let head_w = measure::width(head) + if elided { 1 } else { 0 };
    let slack = room.saturating_sub(head_w);
    let before = slack / 2;
    let after = slack - before;

    let mut x = area.x;
    if mark_focus {
        ink.text(cx, x, label_row, marker, paint);
        ink.fill(cx, Rect::new(x + 1, label_row, 1, 1), " ", paint);
        x += 2;
    }
    ink.fill(cx, Rect::new(x, label_row, before, 1), " ", paint);
    x += i32::from(before);
    x += i32::from(ink.text(cx, x, label_row, head, paint));
    if elided {
        x += i32::from(ink.text(cx, x, label_row, ellipsis, paint));
    }
    ink.fill(cx, Rect::new(x, label_row, after, 1), " ", paint);

    face
}
