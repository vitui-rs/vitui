//! **F1 text**, ~48 entries, expressed by `text`, `chip`, `field` and the [`fit`] helper.
//!
//! The reduction is R1 and R5: markdown needs its own wrap pass and is out of scope, and there
//! is no bidi — a stated non-goal with the engine's tables named as the reason.
//!
//! `field` declares this family and is **not** homed here: its first family is F6, which is where
//! the caret, the wrap index and the keyboard live. Spelled out because `text` and `field` sharing
//! `layout::text` is the reason a reader would expect otherwise.
//!
//! # `fit` is the partition primitive
//!
//! The table gives it one job — *truncation, alignment, padding* — and one shape: **text, then
//! the remainder; there is no verb on it that fills first.** That sentence is the whole helper. A
//! label centred in a rectangle is three writes that touch every cell of its row exactly once — the
//! lead, the text, the trail — and never a fill followed by a draw, which is R07's original defect
//! and 26 of 48 cells on a steady dropdown frame.
//!
//! **Routing through it costs nothing against the discipline**, and that is the measurement spec
//! the table owed rather than a claim about ergonomics: `crate::form` renders one screen twice, once
//! through `fit` and once with the same order written out by hand, and compares them cell for cell.
//! The helper is not a convenience over hand-written correctness — it is the only form of it anybody
//! keeps.

use vitui_runtime::layout::text as measure;
use vitui_runtime::{Ctx, Interest, Paint, Response, Role};

use crate::glyphs::elide;
use crate::ink::{Direct, Ink};
use crate::state::{Faces, press_into};
use vitui_runtime::Rect;
use vitui_runtime::layout::rect;

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &["text", "chip"];

/// The cluster a padding run is made of. One cell, single width, and the only thing `fit` writes
/// that is not the caller's text.
const PAD: &str = " ";

/// Where the text sits in the row it is fitted into.
///
/// Named `Justify` and not `Align` or `Fit`: [`vitui_runtime::layout::Align`] and
/// [`vitui_runtime::layout::Fit`] both exist and mean something else, and a component author reading
/// two `Align`s in one call has to check which crate each came from.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Justify {
    /// Against the left edge, the padding after it. The default.
    #[default]
    Start,
    /// Centred, with the odd cell going to the trailing side — `(w - used) / 2` leads.
    Middle,
    /// Against the right edge, the padding before it.
    End,
}

/// [`fit`]'s options.
///
/// **Options are a `Default` struct, never a required builder**, and every helper
/// `f` has a sibling `f_with` that takes one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FitOpts {
    /// Where the text sits in its row.
    pub justify: Justify,
    /// The role the text is painted in.
    pub role: Role,
    /// The role the padding is painted in.
    ///
    /// **Separate from `role` on purpose.** A chip's label and a chip's face are one rectangle and
    /// two paints, and a caller that had to pass one role would fill the face first to get the
    /// second — which is R07's defect arriving through the helper that exists to prevent it.
    pub pad: Role,
}

impl Default for FitOpts {
    fn default() -> FitOpts {
        FitOpts {
            justify: Justify::Start,
            role: Role::Body,
            pad: Role::Body,
        }
    }
}

/// **Write `s` into the first row of `area`, and return the rows below it.**
///
/// The ninety-per-cent spelling: left-justified, [`Role::Body`] for both halves.
///
/// ```
/// use vitui_runtime::Rect;
/// use vitui_components::text::fit;
/// use vitui_runtime::ctx::Driver;
///
/// let mut driver = Driver::headless(20, 3).expect("a sink attaches");
/// driver.frame(|cx| {
///     let area = cx.area();
///     let rest = fit(cx, area, "one");
///     let rest = fit(cx, rest, "two");
///     // Two rows written, one row returned — and the caller owns it.
///     assert_eq!((rest.y, rest.h), (2, 1));
/// });
/// ```
pub fn fit(cx: &mut Ctx<'_, '_>, area: Rect, s: &str) -> Rect {
    fit_with(cx, area, s, &FitOpts::default())
}

/// [`fit`], with the options spelled out.
pub fn fit_with(cx: &mut Ctx<'_, '_>, area: Rect, s: &str, opts: &FitOpts) -> Rect {
    fit_into(&mut Direct, cx, area, s, opts)
}

/// **[`fit`], drawing through an [`Ink`] so a counter can see the verbs.**
///
/// The entry point a gate takes; `fit` is this with [`Direct`]. There is no second implementation —
/// [`crate::ink`]'s header is why, and it is the same argument `crate::counters` makes one file
/// over: a gate written against a copy of the code tests the copy.
///
/// # What it writes, and in what order
///
/// The first row of `area`, left to right and never twice: the lead padding, the head of the text,
/// the one-cell ellipsis if there is one, and the trail padding. Every one of the four is skipped
/// when it is empty, which is what keeps `verbs` honest — a label that exactly fills its row is one
/// verb and not three.
///
/// # The one-cell ellipsis is enforced here
///
/// `fit` is where truncation is decided, so it is where the rule has to hold: [`elide`] reserves
/// **one** cell for the marker and `Theme::glyph` guarantees every spelling is one cell wide.
/// A three-cell `...` where one was reserved has `writes`, `verbs` and `marked` identical either
/// way — the defect has no signature at any counter and only the surface disagrees — which is why
/// the rule lives in a call rather than in a comment.
pub fn fit_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    s: &str,
    opts: &FitOpts,
) -> Rect {
    let (band, rest) = rect::split_at_v(area, 1);
    if band.is_empty() {
        return rest;
    }
    // `theme()` borrows the environment and not the context, so the paints outlive the verbs below.
    let theme = cx.theme();
    let text_paint = theme.paint(opts.role);
    let pad_paint = theme.paint(opts.pad);

    let w = band.w;
    let (head, marker) = elide(theme, s, w);
    let head_w = measure::width(head);
    let mark_w = measure::width(marker);
    let used = head_w + mark_w;
    debug_assert!(
        used <= w,
        "`elide` returned {used} cells for a {w}-cell row: the one-cell ellipsis rule has moved"
    );
    let slack = w - used;
    let lead = match opts.justify {
        Justify::Start => 0,
        Justify::Middle => slack / 2,
        Justify::End => slack,
    };
    let trail = slack - lead;

    let y = band.y;
    let x = band.x;
    ink.run(cx, x, y, PAD, lead, pad_paint);
    if head_w > 0 {
        ink.text(cx, x + i32::from(lead), y, head, text_paint);
    }
    if mark_w > 0 {
        ink.text(cx, x + i32::from(lead + head_w), y, marker, text_paint);
    }
    ink.run(cx, x + i32::from(lead + used), y, PAD, trail, pad_paint);
    rest
}

// ── `text` and `chip`, the two components this module homes ──────────────────────────────────────
//
// **Spec §1's four rules, with one substitution, and the substitution is stated rather than
// silent.** Rule 1 is `fn(&mut Ctx, Rect, …) -> Response`, and `Rect` **cannot be named in this
// package at all**: it is `vitui_engine::Rect`, named by twenty-seven of the runtime's public
// declarations and re-exported by none of them, and constraint C6 says this crate's
// `[dependencies]` table is `vitui-runtime` and nothing else. So every component here takes
// [`Rect`], which is this crate's own rectangle in the `Ctx`'s own coordinates, swept operator for
// operator against `vitui_runtime::layout::rect`. See [`vitui_runtime::layout::rect`]'s header for the four
// candidate answers and why this is the one. **The rule is obeyed, not ignored** — a reader coming
// from spec §1 is looking at `fn(&mut Ctx, Rect, …) -> Response`.
//
// The other three rules are as written: data by shared reference, options a `Default` struct with an
// `_with` sibling, and a `Response` back even from a pure drawer.
//
// # Why a component writes **all** of its rectangle and `fit` returns the rest
//
// §2 states the rule in two halves — *the owner of a rectangle writes all of it; a component handed
// a rectangle writes all of that* — and the two halves land on two different items. [`fit`] is a
// **helper**: it writes one row and returns the rows below it, so a caller can chain it down a
// rectangle it owns. [`text`] is a **component**: it was handed a rectangle and it writes every cell
// of it, padding the rows its one line does not take. A component that wrote only its first row
// would leave the rest carrying whatever was there before, which is §2's *no cell never* and the
// defect C11's sentinel counted at 9 956 cells of 53 280.
//
// [`crate::structure::panel`] is the exception the rule names: it *returns* the interior it did not
// write, because §2's own sentence is *the cells it does not write are named in its return value*.

/// [`text`]'s options.
///
/// A `Default` struct, never a required builder.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TextOpts {
    /// Where the line sits in its row.
    pub justify: Justify,
    /// The role the line is painted in.
    pub role: Role,
    /// The role the padding — the rest of the row, and every row the line does not take — is painted
    /// in.
    pub pad: Role,
    /// **Whether to declare a region at all**, and what it asks for.
    ///
    /// `None` is the default and means *no hit entry*: a label is a pure drawer, and the `Response`
    /// it returns is [`Response::inert`]'s. `Some(interest)` declares a region — a clickable heading, a
    /// footer that opens a log. See [`Response::inert`] for why `None` and `Some(Interest::NONE)` are
    /// two different statements and neither is the other spelled differently.
    pub interest: Option<Interest>,
}

impl Default for TextOpts {
    fn default() -> TextOpts {
        TextOpts {
            justify: Justify::Start,
            role: Role::Body,
            pad: Role::Body,
            interest: None,
        }
    }
}

/// **One line of text, filling the rectangle it was handed.** The pure drawer.
///
/// **Hostile axes:** `narrow`.
///
/// The one-cell marker rule only fires at a width where the label truncates: a
/// three-cell `...` where one cell was reserved moves **468 cells over 78 rows** with writes, verbs
/// and marked identical at 23 402 / 2 163 / 0 either way.
///
/// ```
/// use vitui_runtime::Rect;
/// use vitui_components::text::text;
/// use vitui_runtime::ctx::Driver;
///
/// let mut driver = Driver::headless(20, 2).expect("a sink attaches");
/// driver.frame(|cx| {
///     let resp = text(cx, cx.area(), "a label");
///     // A pure drawer still answers, which is rule 4 — and it declared nothing, so nothing
///     // happened to it.
///     assert!(!resp.clicked && !resp.hovered && !resp.focused);
/// });
/// ```
#[track_caller]
pub fn text(cx: &mut Ctx<'_, '_>, area: Rect, s: &str) -> Response {
    text_with(cx, area, s, &TextOpts::default())
}

/// [`text`], with the options spelled out.
#[track_caller]
pub fn text_with(cx: &mut Ctx<'_, '_>, area: Rect, s: &str, opts: &TextOpts) -> Response {
    text_into(&mut Direct, cx, area, s, opts)
}

/// **[`text`], drawing through an [`Ink`] so a counter can see the verbs.**
///
/// The entry point a gate takes; `text` is this with [`Direct`]. See [`crate::ink`] for why the seam
/// exists rather than a second implementation.
///
/// # `#[track_caller]` is on the component and not inside it
///
/// [`Ctx::id`] carries the attribute, so the id it mints is the **caller's** call site. A component
/// that draws one widget therefore has to carry it too, or every call in an application collides on
/// the line inside this file — which is `Ctx::id`'s own documented trap, stated there as *put
/// `#[track_caller]` on a function that draws one widget, and not on one that draws several*.
#[track_caller]
pub fn text_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    s: &str,
    opts: &TextOpts,
) -> Response {
    let id = cx.id();
    let resp = match opts.interest {
        Some(interest) => cx.interact(id, area, interest),
        None => Response::inert(id, area),
    };
    let rest = fit_into(
        ink,
        cx,
        area,
        s,
        &FitOpts {
            justify: opts.justify,
            role: opts.role,
            pad: opts.pad,
        },
    );
    // **The rows the line did not take are this component's too.** `fit` hands them back because it
    // is a helper; `text` was handed the rectangle, so it owns them.
    let pad = cx.theme().paint(opts.pad);
    pad_rows(ink, cx, rest, pad);
    resp
}

/// [`chip`]'s options.
///
/// The options struct, and [`Faces`] is inside it rather than beside it for [`crate::state::press`]'s
/// reason: the face drawn and the face awarded have to come out of **one** value.
/// # There is no `label` role here, and its absence cost two cells a frame to find
///
/// The obvious third field is *the role the label is painted in*, and a stand-in screen
/// used one: `Role::Dim` on the chip, `Role::Title` on the button. **It is a defect, and it is
/// the rule firing:**
///
/// > A restyle is free only when the component's own next draw already produces the value the
/// > restyle produced.
///
/// The deferred hover award restyles the chip's rectangle to the hover face. A cell already carrying
/// that face is left alone; a cell carrying `Role::Dim` **is not**, so the award repaints it, the
/// next draw writes `Dim` back, and the award repaints it again — for as long as the pointer rests.
/// A two-column label costs **2 cells a frame, for ever**, and it is invisible on any screen with no
/// pointer on it, which is every screen this crate can build. `crate::dense`'s 338-region screen
/// scored a clean 0 with the defect on it, because `Response::hovered` is false everywhere there.
///
/// So the label wears the face, which is what [`crate::state::press`]'s *one role, so the two cannot
/// disagree* means when it reaches the component: **there is no parameter for a second paint.**
/// `tests::a_hovered_chip_re_damages_its_own_eight_cells_and_not_the_screen` is the eight cells that
/// number would otherwise have been ten.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ChipOpts {
    /// The four faces, handed to [`crate::state::press`] whole.
    pub faces: Faces,
    /// Where the label sits on the face.
    pub justify: Justify,
    /// What the chip declares.
    ///
    /// [`Interest::HOVER`] is not decoration: `press` returns [`Faces::hover`] when
    /// `Response::hovered` is true, and `hovered` is resolved only for a region that asked for the
    /// pointer. A chip drawing a hover face without declaring hover is a branch nothing can take.
    pub interest: Interest,
}

impl Default for ChipOpts {
    fn default() -> ChipOpts {
        ChipOpts {
            faces: Faces::default(),
            justify: Justify::Middle,
            interest: Interest::CLICK.with(Interest::HOVER).with(Interest::FOCUS),
        }
    }
}

/// **A label on a face that reacts.** One rectangle, one paint out of [`crate::state::press`], and
/// every cell written exactly once.
///
/// **Hostile axes:** `narrow`.
///
/// The partition rule names the defect in the axis's own word: *a chip that does not narrow, whose
/// label runs into its sibling's rectangle* — **432 cells re-damaged every steady frame**.
///
/// ```
/// use vitui_runtime::Rect;
/// use vitui_components::text::chip;
/// use vitui_runtime::ctx::Driver;
///
/// let mut driver = Driver::headless(20, 1).expect("a sink attaches");
/// driver.frame(|cx| {
///     let resp = chip(cx, Rect::new(2, 0, 12, 1), "degraded, retrying");
///     // It declared a region, and the label was narrowed into it rather than into its neighbour.
///     assert_eq!((resp.rect.w, resp.rect.h), (12, 1));
/// });
/// ```
#[track_caller]
pub fn chip(cx: &mut Ctx<'_, '_>, area: Rect, label: &str) -> Response {
    chip_with(cx, area, label, &ChipOpts::default())
}

/// [`chip`], with the options spelled out.
#[track_caller]
pub fn chip_with(cx: &mut Ctx<'_, '_>, area: Rect, label: &str, opts: &ChipOpts) -> Response {
    chip_into(&mut Direct, cx, area, label, opts)
}

/// **[`chip`], drawing through an [`Ink`] so a counter can see the verbs.**
#[track_caller]
pub fn chip_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    label: &str,
    opts: &ChipOpts,
) -> Response {
    let id = cx.id();
    let resp = cx.interact(id, area, opts.interest);
    chip_drawn(ink, cx, area, label, &resp, opts);
    resp
}

/// **[`chip`]'s drawing half, with the [`Response`] supplied rather than declared.** Returns the
/// face it drew.
///
/// # This is public because the pointer could not be driven from this crate at all
///
/// **The barrier this section rests on is lifted** — `Mouse` and the
/// three types needed to build one are reachable now. The reasoning is kept because it is why this
/// door is public; the door is not withdrawn, because a container that already holds an id still
/// needs it.
///
/// As written: `Driver::post_mouse` takes a `vitui_engine::Mouse`, and
/// `crates/vitui-runtime/src/line.rs` filed it `EngineName { name: "Mouse", reachable_as: None }`;
/// `Ctx::interact` reads `hovered` off
/// `frame.hover_guess`, which nothing here can set. So *the frame one chip is hovered* —
/// **8 cells against 6 662, 833×** — has no gesture to play, and the only way to stand it up is to
/// hand the component the `Response` the runtime would have handed it.
///
/// [`crate::state`] made exactly this substitution one ticket earlier, at [`press_into`], and named
/// it on the number it produces. This is that substitution moved **up to the component**, so the
/// eight cells are `chip`'s number and not its construction's — see
/// `tests::a_hovered_chip_re_damages_its_own_eight_cells_and_not_the_screen`.
///
/// It is also the entry point a container with an id it already holds wants: a collection row
/// declares its own region and draws a chip inside it.
pub fn chip_drawn<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    label: &str,
    resp: &Response,
    opts: &ChipOpts,
) -> Role {
    let face = press_into(ink, cx, area, resp, &opts.faces);
    face_and_label(ink, cx, area, label, face, opts.justify);
    face
}

/// **The one order a chip is written in: the label's row through [`fit`], the rest as face.**
///
/// Shared by [`chip`] and by [`crate::input::button`], because they are the same construction with
/// different defaults — the freeze gives each of them `constructions: 1`, and that is what
/// *one construction* means. **There is no fill here**: the face is the padding role handed to
/// `fit`, which is why [`FitOpts`] carries two roles and not one.
pub(crate) fn face_and_label<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    label: &str,
    face: Role,
    justify: Justify,
) {
    let paint = cx.theme().paint(face);
    // The label sits on the middle row of a taller rectangle, and the rows either side are face.
    let (above, rest) = rect::split_at_v(area, area.h.saturating_sub(1) / 2);
    pad_rows(ink, cx, above, paint);
    let below = fit_into(
        ink,
        cx,
        rest,
        label,
        &FitOpts {
            // **Both roles are the face**, which is the whole of [`ChipOpts`]'s note: a label in a
            // second paint is a cell the hover award repaints every frame for ever. `FitOpts`
            // carrying two roles is what lets one value fill both.
            justify,
            role: face,
            pad: face,
        },
    );
    pad_rows(ink, cx, below, paint);
}

/// Write every row of `cells` as padding. **Runs and never `Ctx::fill`** — see [`crate::ink`]:
/// `fill` returns `()`, so a filled cell is modelled rather than reported and the pair stops being
/// a comparison between two sources.
pub(crate) fn pad_rows<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, cells: Rect, st: Paint) {
    if cells.is_empty() {
        return;
    }
    let x = cells.x;
    for r in 0..cells.h {
        ink.run(cx, x, cells.y + i32::from(r), PAD, cells.w, st);
    }
}

/// **The two components written the priced way, kept because a gate nobody has watched
/// fail is not a gate.**
///
/// `pub` for the reason [`crate::frame::defective`] and [`crate::state::defective`] are: an
/// instrument crate's fixtures are part of the instrument, and a gate validated only against a
/// correct build reports zero for the same reason a broken one would. Each of these is **the correct
/// component with one thing changed**, so the diff a reviewer would have to catch is the diff the
/// register can point at.
pub mod defective {
    use super::{
        ChipOpts, Ctx, Ink, PAD, Paint, Rect, Response, TextOpts, face_and_label, pad_rows,
        press_into, rect,
    };

    /// **A `text` that does not narrow**, whose line runs into its neighbour's rectangle.
    ///
    /// Green at 300×80 and red at 120×40, which is the whole reason [`crate::dense`]'s equality runs
    /// at two sizes.
    pub fn text_that_does_not_narrow<I: Ink>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        s: &str,
        opts: &TextOpts,
    ) -> Response {
        let id = cx.id();
        let resp = match opts.interest {
            Some(interest) => cx.interact(id, area, interest),
            None => Response::inert(id, area),
        };
        let theme = cx.theme();
        let (text, pad) = (theme.paint(opts.role), theme.paint(opts.pad));
        let rest = overrunning(ink, cx, area, s, text, pad);
        pad_rows(ink, cx, rest, pad);
        resp
    }

    /// **A `chip` that does not narrow.** The third instance: 54 chips of 222 overrun six
    /// columns each into the column beside them, and the neighbour writes them back every frame —
    /// `54 × 6 = 324` on [`crate::dense`]'s screen.
    pub fn chip_that_does_not_narrow<I: Ink>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        label: &str,
        opts: &ChipOpts,
    ) -> Response {
        let id = cx.id();
        let resp = cx.interact(id, area, opts.interest);
        let face = press_into(ink, cx, area, &resp, &opts.faces);
        let theme = cx.theme();
        let (text, pad) = (theme.paint(face), theme.paint(face));
        let rest = overrunning(ink, cx, area, label, text, pad);
        pad_rows(ink, cx, rest, pad);
        resp
    }

    /// **A `chip` that fills its face before drawing its label.** R07's original defect: the screen
    /// is right on every frame and the label's own cells are written twice, every frame, for ever.
    pub fn chip_that_fills_its_face<I: Ink>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        label: &str,
        opts: &ChipOpts,
    ) -> Response {
        let id = cx.id();
        let resp = cx.interact(id, area, opts.interest);
        let face = press_into(ink, cx, area, &resp, &opts.faces);
        let paint = cx.theme().paint(face);
        pad_rows(ink, cx, area, paint);
        face_and_label(ink, cx, area, label, face, opts.justify);
        resp
    }

    /// **Write `s` without truncating it, pad what is left of the row, and return the rows below.**
    ///
    /// The one branch the two arms above share. The extent is the engine's: [`Ink::text`] returns
    /// how many columns landed after clipping, so a string longer than its rectangle is reported at
    /// its full width and the padding run is empty. What it wrote into is the **neighbour's**
    /// rectangle, and the neighbour writes it back on the next frame — which is what makes an
    /// unnarrowed label re-damage rather than merely look wrong.
    fn overrunning<I: Ink>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        s: &str,
        text: Paint,
        pad: Paint,
    ) -> Rect {
        let (band, rest) = rect::split_at_v(area, 1);
        if band.is_empty() {
            return rest;
        }
        let (x, y) = (band.x, band.y);
        let used = ink.text(cx, x, y, s, text);
        ink.run(
            cx,
            x + i32::from(used),
            y,
            PAD,
            band.w.saturating_sub(used),
            pad,
        );
        rest
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::counters::Tally;
    use crate::runner::{Canvas, Pen, driver_at};
    use vitui_runtime::Density;
    use vitui_runtime::ctx::Driver;

    /// A tally over one frame of `f`, on a `w` by `h` sink.
    fn tallied(w: u16, h: u16, f: impl FnOnce(&mut Tally, &mut Ctx<'_, '_>)) -> Tally {
        let mut driver = Driver::headless(w, h).expect("a sink cannot fail to attach");
        let mut tally = Tally::new();
        driver.frame(|cx| f(&mut tally, cx));
        tally
    }

    /// **`fit` writes its row exactly once and nothing else** — the first equality, on the
    /// helper it is stated about, and the second equality restricted to the band.
    ///
    /// Swept over every width and every justification, because the interesting cases are the ones
    /// where the arithmetic runs out: a row narrower than its text, a row exactly as wide, and a row
    /// one cell wide where the ellipsis has nowhere to go.
    #[test]
    fn fit_writes_a_partition_of_its_row_at_every_width_and_justification() {
        for justify in [Justify::Start, Justify::Middle, Justify::End] {
            for w in 0u16..24 {
                for label in ["", "a", "hello", "a rather longer label than fits"] {
                    let opts = FitOpts {
                        justify,
                        ..FitOpts::default()
                    };
                    let tally = tallied(w.max(1), 3, |tally, cx| {
                        let area = Rect::new(0, 0, w, 1);
                        let rest = fit_into(tally, cx, area, label, &opts);
                        assert!(rest.is_empty(), "a one-row area leaves no remainder");
                    });
                    assert_eq!(
                        tally.writes(),
                        tally.distinct(),
                        "{justify:?} `{label}` at {w}: a cell was written twice"
                    );
                    assert_eq!(
                        tally.distinct(),
                        u64::from(w),
                        "{justify:?} `{label}` at {w}: the row is not covered"
                    );
                    assert_eq!(
                        tally.reported(),
                        tally.writes(),
                        "{justify:?} `{label}` at {w}: a write the engine did not report"
                    );
                }
            }
        }
    }

    /// **The remainder is the rest of the rectangle, and it is untouched.**
    ///
    /// The first half of criterion 1: `fit` returns what it did not write, and a chain of them
    /// partitions a rectangle into rows.
    #[test]
    fn fit_returns_the_rows_it_did_not_write() {
        let tally = tallied(12, 5, |tally, cx| {
            let area = cx.area();
            let opts = FitOpts::default();
            let mut rest = area;
            for line in ["one", "two"] {
                rest = fit_into(tally, cx, rest, line, &opts);
            }
            assert_eq!((rest.x, rest.y, rest.w, rest.h), (0, 2, 12, 3));
        });
        assert_eq!(tally.writes(), 24, "two rows of twelve");
        assert_eq!(tally.distinct(), 24);
        for y in 2..5 {
            for x in 0..12 {
                assert!(
                    !tally.touched(x, y),
                    "({x}, {y}) is the caller's and was written"
                );
            }
        }
    }

    /// **A chain that runs off the bottom keeps returning empty rectangles in the right place**, so
    /// a caller with more content than room writes nothing outside its own rectangle.
    #[test]
    fn a_chain_that_runs_out_of_rows_writes_nothing_more() {
        let tally = tallied(8, 2, |tally, cx| {
            let opts = FitOpts::default();
            let mut rest = cx.area();
            for i in 0..6 {
                rest = fit_into(tally, cx, rest, &format!("row {i}"), &opts);
                if i >= 2 {
                    assert!(rest.is_empty());
                    assert_eq!(
                        rest.y, 2,
                        "the empty remainder stays where the split left it"
                    );
                }
            }
        });
        assert_eq!(
            tally.writes(),
            16,
            "two rows of eight and not one cell more"
        );
        // Two verbs a row — `row 0` is five columns and three of padding trail it — and **four
        // calls past the bottom add none**, which is the half this test is about.
        assert_eq!(
            tally.verbs(),
            4,
            "a call onto an empty rectangle is not a verb"
        );
    }

    /// **The one-cell ellipsis, enforced where truncation is decided** — criterion 8.
    ///
    /// Two claims, and the second cannot be made from `glyphs` alone: the
    /// marker is one cell, *and* the head plus the marker never exceed the row. A three-cell marker
    /// would leave `writes`, `verbs` and `distinct` unmoved inside a narrowed context and only the
    /// surface would disagree.
    #[test]
    fn a_truncated_label_ends_in_exactly_one_cell_of_ellipsis() {
        let mut driver = Driver::headless(40, 1).expect("a sink cannot fail to attach");
        driver.frame(|cx| {
            let theme = cx.theme();
            for w in 1u16..20 {
                let (head, marker) = elide(theme, "a label that is far too long", w);
                assert_eq!(
                    measure::width(marker),
                    1,
                    "the marker is not one cell at {w}"
                );
                assert!(
                    measure::width(head) + measure::width(marker) <= w,
                    "head plus marker overruns a {w}-cell row"
                );
            }
            // Nothing is cut, so nothing is marked.
            let (head, marker) = elide(theme, "short", 20);
            assert_eq!((head, marker), ("short", ""));
        });
    }

    /// **Centring puts the odd cell on the trailing side**, which is a decision and not an accident:
    /// a column of centred labels of mixed parity has to agree with itself, and `(w - used) / 2`
    /// truncating is what makes it.
    #[test]
    fn centring_leads_with_the_floor_and_trails_with_the_remainder() {
        let tally = tallied(9, 1, |tally, cx| {
            let opts = FitOpts {
                justify: Justify::Middle,
                ..FitOpts::default()
            };
            fit_into(tally, cx, Rect::new(0, 0, 9, 1), "abcd", &opts);
        });
        assert_eq!(tally.writes(), 9);
        assert_eq!(tally.verbs(), 3, "lead, text, trail");
        // (9 - 4) / 2 == 2 leading, so the text occupies columns 2..6 and three cells trail.
        for x in 0..9 {
            assert!(tally.touched(x, 0));
        }
    }

    // ── components ticket 10: `text` and `chip` ──────────────────────────────────────────────────

    /// The corpus every partition sweep below runs over: a rectangle narrower than its label, one
    /// exactly as wide, one a single cell, and one two rows taller than a line.
    const SIZES: [(u16, u16); 8] = [
        (1, 1),
        (2, 1),
        (5, 1),
        (12, 1),
        (12, 3),
        (12, 4),
        (40, 1),
        (40, 6),
    ];

    /// The labels the sweeps use. Two of them are wider than every rectangle above but the last.
    const LABELS: [&str; 4] = [
        "",
        "on",
        "degraded, retrying",
        "an unusually long metric name",
    ];

    /// **Criterion 2, behavioural half: `text` writes each cell of its rectangle exactly once, and
    /// writes every one of them.**
    ///
    /// The two equalities, on the component rather than on the helper: `writes == distinct` (no
    /// cell twice) and `distinct == w × h` (no cell never). The second is the one [`fit`] cannot
    /// make — it returns the rows it did not write, because it is a helper — and it is why `text`
    /// pads them.
    #[test]
    fn text_writes_a_partition_of_its_whole_rectangle() {
        for (w, h) in SIZES {
            for label in LABELS {
                for justify in [Justify::Start, Justify::Middle, Justify::End] {
                    let opts = TextOpts {
                        justify,
                        ..TextOpts::default()
                    };
                    let tally = tallied(w, h, |tally, cx| {
                        text_into(tally, cx, Rect::new(0, 0, w, h), label, &opts);
                    });
                    let cells = u64::from(w) * u64::from(h);
                    assert_eq!(
                        tally.writes(),
                        tally.distinct(),
                        "{w}x{h} {justify:?} `{label}`: {} cells written twice",
                        tally.writes() - tally.distinct()
                    );
                    assert_eq!(
                        tally.distinct(),
                        cells,
                        "{w}x{h} {justify:?} `{label}`: the rectangle is not covered, and a cell \
                         nobody writes keeps what was there before"
                    );
                    assert_eq!(
                        tally.reported(),
                        tally.writes(),
                        "{w}x{h}: a write the engine did not report"
                    );
                    assert_eq!(
                        tally.asked(),
                        tally.reported(),
                        "{w}x{h}: a verb was clipped"
                    );
                }
            }
        }
    }

    /// **Criterion 1: `text` declares nothing by default, and a region when it is asked for one.**
    ///
    /// Rule 4 is *return `Response`, even from a pure drawer*, and [`Response::inert`]'s header is why
    /// that is not `Interest::NONE`: `NONE` is a hit entry, and 222 of them is the difference
    /// between [`crate::dense`]'s 338 regions and 560.
    #[test]
    fn text_takes_a_hit_entry_only_when_its_options_ask_for_one() {
        let mut driver = Driver::headless(20, 1).expect("a sink attaches");
        driver.frame(|cx| {
            let resp = text(cx, Rect::new(0, 0, 20, 1), "a label");
            assert_eq!(
                (resp.rect.w, resp.rect.h),
                (20, 1),
                "it names its rectangle"
            );
            assert!(!resp.hovered && !resp.focused && !resp.clicked);
        });
        assert_eq!(
            driver.inspect().hits().len(),
            0,
            "a pure drawer is not a region"
        );

        let mut driver = Driver::headless(20, 1).expect("a sink attaches");
        driver.frame(|cx| {
            text_with(
                cx,
                Rect::new(0, 0, 20, 1),
                "a heading",
                &TextOpts {
                    interest: Some(Interest::CLICK),
                    ..TextOpts::default()
                },
            );
        });
        assert_eq!(driver.inspect().hits().len(), 1);
        assert_eq!(
            driver.inspect().stop_count(),
            0,
            "and clickable is not focusable"
        );
    }

    /// **Criterion 2 and criterion 3: `chip` writes a partition of its rectangle and narrows into
    /// it.**
    ///
    /// The narrowing is the second equality doing the work: a chip whose label ran into its
    /// sibling's rectangle would report `distinct > w × h`, because [`crate::counters::Tally`] unions
    /// the spans the verbs actually asked for. That is the same 324 cells
    /// [`crate::dense::CHIP_NOT_NARROWED`] measures as re-damage on a screen — **two instruments,
    /// two halves, and neither sees both** (see `crate::dense`).
    #[test]
    fn chip_writes_a_partition_of_its_rectangle_and_narrows_into_it() {
        for (w, h) in SIZES {
            for label in LABELS {
                let opts = ChipOpts::default();
                let tally = tallied(w.max(2), h, |tally, cx| {
                    chip_into(tally, cx, Rect::new(0, 0, w, h), label, &opts);
                });
                let cells = u64::from(w) * u64::from(h);
                assert_eq!(
                    tally.writes(),
                    tally.distinct(),
                    "{w}x{h} `{label}`: a chip cell written twice, which is R07's own defect"
                );
                assert_eq!(
                    tally.distinct(),
                    cells,
                    "{w}x{h} `{label}`: the chip does not cover its own face"
                );
                assert_eq!(
                    tally.asked(),
                    tally.reported(),
                    "{w}x{h}: a verb was clipped"
                );
            }
        }
    }

    /// **The two defective chips, watched failing the two gates above.**
    ///
    /// *A gate nobody has watched fail is not a gate*, and the two failures are **different**: the
    /// filled face fails `writes == distinct` and covers exactly the right cells; the unnarrowed
    /// label passes `writes == distinct` and covers **too many**. One gate would have caught one of
    /// them.
    #[test]
    fn a_chip_that_fills_its_face_and_one_that_does_not_narrow_fail_two_different_gates() {
        let opts = ChipOpts::default();
        let label = "degraded, retrying";
        let area = Rect::new(0, 0, 12, 1);

        let filled = tallied(24, 1, |tally, cx| {
            defective::chip_that_fills_its_face(tally, cx, area, label, &opts);
        });
        assert!(
            filled.writes() > filled.distinct(),
            "the filled face wrote no cell twice, so R07's defect is not what this fixture is"
        );
        assert_eq!(
            filled.distinct(),
            (u64::from(area.w) * u64::from(area.h)),
            "and it covers exactly the chip, which is why *no cell never* cannot see it"
        );
        assert_eq!(
            filled.writes() - filled.distinct(),
            u64::from(area.w),
            "the double-written cells are the label's — this one elides to exactly the chip's \
             twelve columns, so the fill buys nothing at all and every cell of it is written twice"
        );

        let overrun = tallied(24, 1, |tally, cx| {
            defective::chip_that_does_not_narrow(tally, cx, area, label, &opts);
        });
        assert_eq!(
            overrun.writes(),
            overrun.distinct(),
            "the unnarrowed label writes no cell twice, which is why the pair alone misses it"
        );
        assert!(
            overrun.distinct() > (u64::from(area.w) * u64::from(area.h)),
            "the label stayed inside the chip, so this fixture is no longer the defect it is kept as"
        );
        assert_eq!(
            overrun.distinct() - (u64::from(area.w) * u64::from(area.h)),
            u64::from(vitui_runtime::layout::text::width(label)) - u64::from(area.w),
            "and what it overran by is exactly the label's overhang into its neighbour"
        );
    }

    /// **Criterion 7: the frame a chip is hovered re-damages the chip and nothing else — 8 cells.**
    ///
    /// > | | naive | correct |
    /// > |---|---|---|
    /// > | C01, the frame one chip is hovered | **6 662 damaged** | **8** — 833× |
    ///
    /// **The 8 reproduces exactly, because the number *is* the chip's width** — two documents
    /// both state it that way, it is reproduced at [`crate::state::CHIP`], and this is the
    /// same eight cells measured through the component instead of through its construction.
    ///
    /// # The pointer is fabricated, and that is named rather than hidden
    ///
    /// `Driver::post_mouse` takes a `vitui_engine::Mouse` and `crates/vitui-runtime/src/line.rs`
    /// files it `reachable_as: None`, so **no gesture can be played from this crate**. What is
    /// supplied instead is the `Response` the runtime would have supplied — [`chip_drawn`]'s whole
    /// reason for being public — and [`Pen::end_frame`] applies the award where `Driver::frame`
    /// applies it. The same two substitutions, one layer up.
    ///
    /// # What the other column would be on this map
    ///
    /// The 6 662 is C01's prototype screen clearing every frame. On [`crate::dense`]'s screen the
    /// same spelling is [`crate::dense::CLEARED_EVERY_FRAME`] — **9 024** — so the ratio here is
    /// 1 128× rather than 833×. The magnitude is the screen's and the direction is the rule's; both
    /// columns are printed by `examples/primitive_numbers.rs` rather than reconciled.
    #[test]
    fn a_hovered_chip_re_damages_its_own_eight_cells_and_not_the_screen() {
        let cells = crate::state::chip();
        assert_eq!(
            (u64::from(cells.w) * u64::from(cells.h)),
            8,
            "the number is the chip's width"
        );

        let mut driver = driver_at(crate::state::W, crate::state::H, Density::Compact);
        let mut canvas = Canvas::new(crate::state::W, crate::state::H);
        let mut changed = Vec::new();
        // Four frames at rest, then one with the pointer on it, then one more resting there.
        for hovered in [false, false, false, false, true, true] {
            let mut pen = Pen::over(canvas);
            driver.frame(|cx| {
                let opts = ChipOpts::default();
                let id = cx.id();
                let mut resp = cx.interact(id, cells, opts.interest);
                resp.hovered = hovered;
                chip_drawn(&mut pen, cx, cells, crate::state::LABEL, &resp, &opts);
            });
            pen.end_frame();
            canvas = pen.into_canvas();
            changed.push(canvas.take_repaints());
        }

        assert_eq!(
            changed[0], 8,
            "the chip appearing: eight cells that were nobody's"
        );
        assert_eq!(
            &changed[1..4],
            &[0, 0, 0],
            "a chip nobody is pointing at re-damages nothing"
        );
        assert_eq!(
            changed[4], 8,
            "the frame the pointer arrives: the chip's own eight cells and not one more"
        );
        assert_eq!(
            changed[5], 0,
            "and the frame after it, which is `press` collapsing the two statements — the face \
             drawn already *is* the face awarded, so the restyle writes what is there"
        );
    }
}
