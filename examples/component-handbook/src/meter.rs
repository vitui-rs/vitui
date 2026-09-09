//! **A component that draws and nothing else** — the shape everything else here is a variation on.
//!
//! It is one horizontal bar with a percentage beside it. Nothing about it is interactive, and it is
//! still worth writing as a component rather than as four lines at the call site, because four
//! things have to be true of it and none of them is obvious:
//!
//! 1. **It writes every cell of the rectangle it was handed, exactly once.** A component that
//!    writes part of its rectangle leaves whatever was there before — which on a screen that
//!    re-lays out is the *previous* content, not the background. A component that writes a cell
//!    twice pays for it on every frame for ever.
//! 2. **It names roles, never colours.** The palette is the theme's business, and a colour named
//!    here would survive a theme change.
//! 3. **It takes its glyphs from the theme too**, so that the same call draws a solid bar on a
//!    terminal that has one and an ASCII bar on a terminal that does not.
//! 4. **It declares its rectangle**, even though it wants no events, so that a container above can
//!    find it and so that the frame's region count is the truth.

use vitui_runtime::{Ctx, Glyph, Interest, Rect, Response, Role};

use crate::ink::{Direct, Ink};

/// How a [`meter`] looks. Every field is a role or a flag; there is no colour here to set.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MeterOpts {
    /// The part of the bar that is full.
    pub full: Role,
    /// The part that is not.
    pub track: Role,
    /// The percentage beside it.
    pub label: Role,
    /// Whether to write that percentage at all. A meter three columns wide has no room for it.
    pub show_percent: bool,
}

impl Default for MeterOpts {
    fn default() -> MeterOpts {
        MeterOpts {
            full: Role::Ok,
            track: Role::Dim,
            label: Role::Body,
            show_percent: true,
        }
    }
}

/// **A bar filled to `ratio`, with the percentage beside it.** `ratio` is clamped to `0.0..=1.0`.
///
/// The ninety-per-cent spelling: three arguments, sensible defaults, no options struct in sight.
#[track_caller]
pub fn meter(cx: &mut Ctx<'_, '_>, area: Rect, ratio: f32) -> Response {
    meter_with(cx, area, ratio, &MeterOpts::default())
}

/// [`meter`], with the options spelled out.
#[track_caller]
pub fn meter_with(cx: &mut Ctx<'_, '_>, area: Rect, ratio: f32, opts: &MeterOpts) -> Response {
    meter_into(&mut Direct, cx, area, ratio, opts)
}

/// [`meter`], drawing through an [`Ink`] so a counter can see the verbs.
///
/// The third spelling is what a gate takes. It is the only one with a body; the other two are one
/// line each, which is what keeps the three from drifting apart.
#[track_caller]
pub fn meter_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    ratio: f32,
    opts: &MeterOpts,
) -> Response {
    // **The id is minted here and nowhere else.** `Ctx::id` is `Location::caller()`, and
    // `#[track_caller]` forwards it through this function and its two wrappers — so all three
    // spellings of one call site are one widget, and two call sites are two.
    let id = cx.id();
    // Declared even though it wants nothing: the region index is what a container above reads, and
    // a component missing from it is a hole in every count taken over the frame.
    let resp = cx.interact(id, area, Interest::NONE);
    if area.is_empty() {
        return resp;
    }

    let percent = (ratio.clamp(0.0, 1.0) * 100.0).round() as u16;
    let full_paint = cx.theme().paint(opts.full);
    let track_paint = cx.theme().paint(opts.track);
    let label_paint = cx.theme().paint(opts.label);
    // `&'static str`, so it outlives the borrow of the context that follows.
    let thumb = cx.theme().glyph(Glyph::Thumb);
    let track = cx.theme().glyph(Glyph::Track);

    // The bar sits on the middle row. Everything else is the padding this component owes.
    let bar_row = area.y + (i32::from(area.h) - 1) / 2;
    for y in area.y..area.bottom() {
        if y != bar_row {
            ink.fill(cx, Rect::new(area.x, y, area.w, 1), " ", track_paint);
        }
    }

    // Four columns for ` 100%`, taken off the right before anything is drawn — a bar that
    // computes its own width from what is left cannot overrun the label.
    let label_w: u16 = if opts.show_percent && area.w > 6 {
        5
    } else {
        0
    };
    let bar_w = area.w - label_w;
    let filled = (u32::from(bar_w) * u32::from(percent) / 100) as u16;

    ink.fill(cx, Rect::new(area.x, bar_row, filled, 1), thumb, full_paint);
    ink.fill(
        cx,
        Rect::new(area.x + i32::from(filled), bar_row, bar_w - filled, 1),
        track,
        track_paint,
    );

    if label_w > 0 {
        // `Ctx::stage` formats into a buffer the frame owns and the ink's `blit` writes it, so a
        // percentage costs no allocation. A `format!` here would allocate once a frame for ever,
        // and the frame's allocation gate is a count of zero.
        let _ = cx.stage(format_args!("{percent:>4}%"));
        ink.blit(cx, area.x + i32::from(bar_w), bar_row, label_paint);
    }

    resp
}
