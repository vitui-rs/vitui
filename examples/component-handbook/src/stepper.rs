//! **A component with a keyboard**, and the contract it answers written as data.
//!
//! Three things here are decisions rather than style, and each of them has cost somebody a day
//! somewhere:
//!
//! **The bindings are a value, not a `match`.** [`BINDINGS`] is what the component answers, what the
//! help bar renders, and what a test sweeps. A `match` on a key code inside the draw is a fourth
//! copy of the same list that no gate can read, and it goes stale the first time a chord is added.
//!
//! **A chord on a character is not a chord on a key.** One keystroke has four spellings on the
//! wire. A terminal speaking the enhanced keyboard protocol reports the *base key and the shift
//! bit*, so `+` on a US layout arrives as `=` with `SHIFT` — and a binding written on the character
//! `+` never fires, on a modern terminal, while reading perfectly on a legacy one.
//! [`Chord::typed`] is the spelling that compares what the key *produced* and ignores `SHIFT`,
//! because a character that needed shift already says so. Use it for `+`, `?`, `:` and `_`; use
//! [`Chord::new`] and [`Chord::key`] where the shortcut is about *where the key is*.
//!
//! **A key consumed to do nothing is worse than a key not taken.** A stepper already at its
//! maximum that swallows `+` has told the container above it *handled*, and the container — a
//! dialog wanting to close, a pager wanting to page — never sees it. The component owns a key
//! exactly when pressing it would change something, and [`owns`] is that predicate.

use vitui_runtime::keys::{ActionId, Chord, Code, MatchMode, Pressed, write_help};
use vitui_runtime::{Binding, Ctx, Glyph, Interest, Rect, Response, Role};

use crate::ink::{Direct, Ink};
use crate::pill::{Faces, face_of};

/// Step down.
pub const DECREMENT: ActionId = 1;
/// Step up.
pub const INCREMENT: ActionId = 2;
/// Go to the floor.
pub const TO_MIN: ActionId = 3;
/// Go to the ceiling.
pub const TO_MAX: ActionId = 4;

/// **What a stepper answers.** The one list: the drain loop below reads it, [`help`] renders it,
/// and a sweep that presses every chord in it and asks whether anything was taken is the test.
pub const BINDINGS: &[Binding] = &[
    Binding::new(
        &[Chord::new(Code::Left), Chord::key('h'), Chord::typed('-')],
        DECREMENT,
        "Decrease",
    ),
    Binding::new(
        &[Chord::new(Code::Right), Chord::key('l'), Chord::typed('+')],
        INCREMENT,
        "Increase",
    ),
    Binding::new(&[Chord::new(Code::Home)], TO_MIN, "Minimum"),
    Binding::new(&[Chord::new(Code::End)], TO_MAX, "Maximum"),
];

/// The contract as a line of help text. Built **outside** the draw — it allocates, and a frame
/// allocates nothing.
pub fn help() -> String {
    let mut out = String::new();
    for b in BINDINGS {
        if !out.is_empty() {
            out.push_str("   ");
        }
        write_help(&mut out, b);
    }
    out
}

/// [`stepper`]'s options.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StepperOpts {
    /// The floor, inclusive.
    pub min: i32,
    /// The ceiling, inclusive.
    pub max: i32,
    /// How far one press moves it.
    pub step: i32,
    /// The faces of the two arrow cells.
    pub faces: Faces,
    /// What the component declares. `FOCUS` is required for the keyboard to arrive at all: the
    /// frame answers `next_key` only for the focused id.
    pub interest: Interest,
}

impl Default for StepperOpts {
    fn default() -> StepperOpts {
        StepperOpts {
            min: 0,
            max: 100,
            step: 5,
            faces: Faces::default(),
            interest: Interest::CLICK.with(Interest::HOVER).with(Interest::FOCUS),
        }
    }
}

/// **Whether this component would do anything with `action` at `value`.**
///
/// The predicate the drain loop consults before consuming a key. It is public because a container
/// that wants to know what its child will take can ask without pressing anything.
pub fn owns(action: ActionId, value: i32, opts: &StepperOpts) -> bool {
    match action {
        DECREMENT | TO_MIN => value > opts.min,
        INCREMENT | TO_MAX => value < opts.max,
        _ => false,
    }
}

/// Which action a key press is, or `None` for a key this component has never heard of.
fn action_of(key: &Pressed) -> Option<ActionId> {
    BINDINGS
        .iter()
        .find(|b| b.matches(key, MatchMode::Masked))
        .map(Binding::action)
}

/// **A number with two arrows, driven by the pointer or the keyboard.**
///
/// `value` is the caller's, borrowed for the call. The component holds nothing across frames —
/// there is nowhere for it to hold anything, and that is the point rather than a limitation.
/// `Response::changed` is `true` on the frame the value moved.
#[track_caller]
pub fn stepper(cx: &mut Ctx<'_, '_>, area: Rect, value: &mut i32) -> Response {
    stepper_with(cx, area, value, &StepperOpts::default())
}

/// [`stepper`], with the options spelled out.
#[track_caller]
pub fn stepper_with(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    value: &mut i32,
    opts: &StepperOpts,
) -> Response {
    stepper_into(&mut Direct, cx, area, value, opts)
}

/// [`stepper`], drawing through an [`Ink`].
#[track_caller]
pub fn stepper_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    value: &mut i32,
    opts: &StepperOpts,
) -> Response {
    let id = cx.id();
    let mut resp = cx.interact(id, area, opts.interest);

    // ── The pointer ──────────────────────────────────────────────────────────────────────────
    //
    // One rectangle was declared, so *where* the click landed is read off the response rather than
    // from a second region: `local` is the pointer in this component's own coordinates.
    if resp.clicked
        && let Some((lx, _)) = resp.local
    {
        let on_right = lx >= i32::from(area.w) / 2;
        let action = if on_right { INCREMENT } else { DECREMENT };
        if owns(action, *value, opts) {
            *value = apply(action, *value, opts);
            resp.changed = true;
        }
    }

    // ── The keyboard ─────────────────────────────────────────────────────────────────────────
    //
    // `next_key` answers only the focused id, so this loop runs on exactly one component a frame.
    // A key this component does not own goes straight back with `decline`, which reopens the queue
    // for whoever is above.
    while let Some(key) = cx.next_key(id) {
        match action_of(&key) {
            Some(action) if owns(action, *value, opts) => {
                *value = apply(action, *value, opts);
                resp.changed = true;
            }
            // Both arms decline: a chord we know but cannot act on, and a chord we have never
            // heard of, are the same thing from the container's point of view.
            _ => cx.decline(key),
        }
    }

    draw(ink, cx, area, *value, &resp, opts);
    resp
}

/// What an action does to a value. Separated out so the drain loop reads as a routing table.
fn apply(action: ActionId, value: i32, opts: &StepperOpts) -> i32 {
    let next = match action {
        DECREMENT => value - opts.step,
        INCREMENT => value + opts.step,
        TO_MIN => opts.min,
        TO_MAX => opts.max,
        _ => value,
    };
    next.clamp(opts.min, opts.max)
}

/// `[ < ]  42  [ > ]`, on one row, with every cell of the rectangle written once.
fn draw<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    value: i32,
    resp: &Response,
    opts: &StepperOpts,
) {
    if area.is_empty() {
        return;
    }
    let face = face_of(resp, &opts.faces);
    // The arrows go dim at the ends, which is the same fact `owns` reports to the container: a
    // control that cannot move should not look as though it can.
    let left = if value > opts.min {
        face
    } else {
        Role::Disabled
    };
    let right = if value < opts.max {
        face
    } else {
        Role::Disabled
    };

    let body = cx.theme().paint(if resp.focused {
        Role::Focus
    } else {
        Role::Body
    });
    let left_paint = cx.theme().paint(left);
    let right_paint = cx.theme().paint(right);
    let arrow_left = cx.theme().glyph(Glyph::ArrowLeft);
    let arrow_right = cx.theme().glyph(Glyph::ArrowRight);

    let row = area.y + (i32::from(area.h) - 1) / 2;
    for y in area.y..area.bottom() {
        if y != row {
            ink.fill(cx, Rect::new(area.x, y, area.w, 1), " ", body);
        }
    }
    if area.w < 5 {
        ink.fill(cx, Rect::new(area.x, row, area.w, 1), " ", body);
        return;
    }

    ink.text(cx, area.x, row, arrow_left, left_paint);
    let right_x = area.x + i32::from(area.w) - 1;
    ink.text(cx, right_x, row, arrow_right, right_paint);

    let inner = Rect::new(area.x + 1, row, area.w - 2, 1);
    let n = cx.stage(format_args!("{value}"));
    let slack = inner.w.saturating_sub(n);
    let before = slack / 2;
    ink.fill(cx, Rect::new(inner.x, row, before, 1), " ", body);
    let wrote = ink.blit(cx, inner.x + i32::from(before), row, body);
    let used = before + wrote;
    ink.fill(
        cx,
        Rect::new(inner.x + i32::from(used), row, inner.w - used, 1),
        " ",
        body,
    );
}
