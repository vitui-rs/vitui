//! **The dropped list: a window that goes the wrong way behind a layer, a tail nothing clears, and
//! two wheels that need the layer placed before they can be routed at all.**
//!
//! This is the screen the overlay family's four
//! remaining hostile axes are scenes *of* — [`crate::popup`] carries the family's own screen, the
//! family's five configurations, and the two files divide the way the questions do: that one is
//! **what a layer costs the frame** and this one is **what a windowed list inside one gets wrong**.
//!
//! | scene | what it decides |
//! |---|---|
//! | a `select`'s popup at option [`SCROLLED_TO`] | the window's sign inside a layer |
//! | twenty posted notches over a `select` | three reveals and a fourth defect only this family has |
//! | a picker's listing edited down to [`SHRUNK_TO`] files | the stale tail, behind a layer |
//! | twenty posted notches over a `file_picker` | the same three arms over the other owner |
//!
//! # All four are one question asked twice on each of two components
//!
//! Both overlay owners request a layer and draw **the collection** inside its body — that is stated
//! of the `select`'s popup and of the picker's listing, and both say *reached by calling it*.
//! So the four axes here are `collection`'s four reached through a layer, and what the layer changes
//! is not the arithmetic but **who can see it**:
//!
//! 1. **An overlay body cannot be handed an [`Ink`](crate::ink::Ink).** A body is
//!    `FnMut(&mut Ctx<'f, '_>) + 'f` and a `&mut I` borrowed for the owner's call cannot travel into
//!    one (the fifth component). So no [`Pen`] had ever seen a popup's interior, and the only
//!    picture of one this crate held was `crate::popup::popup_cells_into` — *the same two orders
//!    written where a `Pen` can see them*, which is to say a copy, and `crate::ink`'s own trap says
//!    a gate written against a copy tests the copy.
//! 2. **A notch is routed against the previous frame's hit index, and a layer's entries are only in
//!    it once the layer has been placed.** So the wheel here opens with **two** frames before the
//!    first click rather than `crate::wheel`'s one, which is why these two wheel scenes are here and
//!    not there — see [`WHEELED_SUBJECTS`].
//!
//! # What changed in the shipped code, and it is one line each
//!
//! `crate::input::popup_body` and `crate::files::picker_body` draw their shell and their list
//! through an `I: Ink` instead of through [`Direct`]. Nothing else moves: a
//! `Ctx::overlay` body still cannot capture one, so the seam is reachable only when a caller invokes
//! those two functions **in the base pass** — which is [`crate::popup`]'s own named substitution,
//! inherited whole and for its reason, **over the shipped body rather than over a transcription of
//! it**. It is on both arms of every comparison here, so it is on the side of neither.
//!
//! # The oracle is the tail of the list as a list of its own
//!
//! [`crate::window`]'s, one family over and for its reason. The scroll scene's reference is the
//! option list **from [`SCROLLED_TO`] onwards** drawn at offset 0, so the window arithmetic is
//! `0 + r` on the reference side and `offset + r` on the subject's, and a defect in that arithmetic
//! cannot hide in both. The shrink scene's is a picker built over the short listing from its first
//! frame.
//!
//! **Three things have to agree on both arms or the equality is about something else**, and each was
//! found by watching a comparison fail:
//!
//! - **The tick.** `chosen` is an index into the list the arm was handed, so the subject's is
//!   [`SCROLLED_TO`] and the reference's is 0 — the same visible row.
//! - **The cursor.** `CollState::sel.lead` is an index too, and a reference at lead 0 draws the
//!   cursor face on its **first visible row** while the subject's lead is off screen entirely.
//! - **The focus.** A list that holds the keyboard draws a different face on every cell
//!   ([`crate::state::press_into`]), and a popup takes the keyboard from its owner on the frame
//!   after it opens — so both arms play the owner-seating frame, exactly as
//!   [`crate::window::Play`] does.
//!
//! # The tail below the content is `collection`'s and the pane is not in the picture
//!
//! `picker_body` hands its preview pane a `line` that takes **no ink**, because `file_picker`'s
//! public signature does — `fn(&mut Ctx, Rect, &T, u32)` — so the pane draws through `Direct` and a
//! [`Pen`] sees the shell and the listing and nothing of the pane. That is stated rather than
//! worked around: it is **identical on both arms**, the picker's own shrink surface is the listing's
//! tail, and the pane's two axes have scenes 23 and 24 already. Changing that signature is a public
//! API change and no scenes ticket's.
//!
//! # The picker is driven with a pointer, and that was an open question
//!
//! **An open `file_picker` seats no focus, declares no refusal and answers only on a click.** It is
//! an open question, it was not this screen's to answer: seating a
//! focus there is a keyboard being designed and carries three decisions a scenes ticket has no
//! standing to make — and so [`wheeled_picker`] drives the axis the way a gate can today: a posted
//! notch over the popup's rectangle, with no keystroke in it. **The keyboard half of the wheel rule
//! is therefore asked of `select` and not of the picker**, and that asymmetry is the open question
//! showing through rather than a decision about the axis. The day 23 is answered this scene gains a
//! keyboard arm and needs no rewriting to get one.
//!
//! [`Pen`]: crate::runner::Pen

use vitui_runtime::keys::{Chord, Code};
use vitui_runtime::work::{Cancel, Task, Worker};
use vitui_runtime::{Buttons, Ctx, Density, Driver, Id, Mods, Mouse, MouseKind, Notch, Rect, Role};

use crate::counters::{Allocations, Counter, Counters};
use crate::files::{
    Entry, PickerBody, PickerOpts, PickerShape, PickerState, Preview, defective as files_defective,
    file_picker_into, picker_body,
};
use crate::ink::Direct;
use crate::input::{
    SelectOpts, SelectShape, SelectState, defective as input_defective, popup_body, select_into,
};
use crate::keys;
use crate::overlay::PopupState;
use crate::runner::{Canvas, Diff, Pen, driver_at};

// ── the screen ───────────────────────────────────────────────────────────────────────────────────

/// The rig's width. Wide enough for the picker's listing beside its pane, which is what
/// [`PickerOpts::list_w`] splits.
pub const W: u16 = 60;

/// The rig's height. Twenty-four, so a windowed popup has a window to be wrong about.
pub const H: u16 = 24;

/// **How many options the `select`'s list holds.** Sixty-four, comfortably more than [`H`], so the
/// popup is showing a window and not its whole content.
pub const OPTIONS: usize = 64;

/// **The option the popup's window is scrolled to. Thirty-two**, which is deep enough that the
/// window shares no row with the unscrolled one.
///
/// *At offset 0 every sign agrees* is the runtime's own finding about the scroll axis (architecture
/// 26) and the reason a scrolled scene may not be played near the top.
pub const SCROLLED_TO: usize = 32;

/// **How many files the picker's listing holds** before the shrink. Sixty-four, which is
/// [`OPTIONS`]' own number: both lists are the same length so that the two wheel arms have the same
/// clamp, and a difference between them is the component and not the fixture.
pub const FILES: usize = 64;

/// **How many it holds after. Nine**, which is [`crate::grid::SHRUNK_TO`]'s own number rather than
/// a second one: the content becomes smaller and the rectangle does not move, and a shrink to a
/// *single* row would make [`stale_by_resize`]'s rectangle one row tall, which is a degenerate
/// screen rather than the refused resize.
pub const SHRUNK_TO: usize = 9;

/// **How wide the picker's listing is**, which is [`PickerOpts::default`]'s and not a second number.
pub const LIST_W: u16 = 24;

/// **How many wheel clicks both wheel scenes play. Twenty**, which is the gesture and
/// [`crate::wheel::CLICKS`].
pub const CLICKS: u32 = crate::wheel::CLICKS;

/// **The offset the wheel scenes start from. Ten**, far enough from what a reveal will ask for that
/// the pull is visible: the cursor sits at the top of the list and the window is nowhere near it.
pub const WHEELED_FROM: i32 = 10;

/// **Where twenty clicks leave the window: thirty.** One row a click, which is
/// `vitui_runtime::scroll::Wheel`'s default and the runtime's whole motion model — a notch is intent
/// and the runtime has no standing to multiply somebody's intent by three.
///
/// **It is short of the clamp on purpose.** Both lists hold [`OPTIONS`] rows in an [`H`]-row
/// window, so the largest offset either admits is `64 - 24 = 40`; a run that ended *at* the clamp
/// would report the same number for a wheel that worked and a wheel that overshot, which is
/// `crate::wheel::Subject::axes`' own refusal — *a dead wheel and a meaningless axis are the same
/// number* — arriving on the other side.
pub const MOVED: i32 = WHEELED_FROM + CLICKS as i32;

/// **Where an unconditional reveal leaves the window: zero.** The pointer is dead.
///
/// It is read **after the settling frame**, which is the whole reason that frame is played. The
/// number one frame earlier is **1**, not 0 — a reveal crosses the frame boundary as sixteen bytes
/// and is read on the frame after, so the defective arm is always one click ahead of its
/// own correction. That one is the runtime's documented residue rather than a softened defect, and
/// it is [`Wheeled::after_last_click`]'s number and not this one.
/// [`crate::wheel::DRAGGED_BACK`] is the same value one component over with the same split behind
/// it.
pub const DRAGGED_BACK: i32 = 0;

/// **The largest offset either list admits at [`H`]. Forty.**
pub const MAX_OFFSET: i32 = OPTIONS as i32 - H as i32;

// ── the ledger ───────────────────────────────────────────────────────────────────────────────────

/// **How many rows of the popup the inverted window is wrong about: twenty-three of
/// [`H`].**
///
/// The primary number, because a row count carries the shape of the failure where a cell count does
/// not — the form. The one row it is right about is the **first**, and it is right by
/// coincidence: at visible row 0 the two arithmetics meet, `offset + 0 == offset - 0`, which is
/// `crate::listing`'s *rows the inverted arm gets right by coincidence* on this component.
pub const INVERTED_ROWS: usize = H as usize - 1;

/// **How many cells of the popup the inverted window is wrong about: forty.**
///
/// **A floor on how wrong the screen is and not a measure of it**, which is
/// [`crate::grid::STALE_CELLS`]'s finding one family over: an option row is `option 0nn` padded out
/// to the popup's width, so two rows of it agree on every cell of the pad and on every cell of the
/// word — the disagreement is the *digits*, and twenty-three wrong rows cost under two cells each.
/// A gate written on cells alone would call a screen wrong in every row a small defect.
pub const INVERTED_CELLS: usize = 40;

/// **What a reserved bar costs a tail-cut oracle: nine cells over nine rows, all of them in one
/// column.**
///
/// [`at_rest_whole`]'s reading, and it is why the equality is over [`window_interior`] —
/// where the reason it cannot be shared is stated, and it is the **extent** and not the offset. The
/// nine is a difference of thumb *lengths*: `24 * 24 / 64` is 9 rows and `24 * 24 / 32` is 18. The
/// *correct* arm's disagreement being **entirely** column `W - 1` is what says the exclusion is the
/// bar and not a convenient blind spot.
pub const BAR_CELLS: usize = 9;

/// **How many rows of the listing the stale tail leaves standing: fifteen.**
///
/// `H - SHRUNK_TO`: the listing is [`H`] rows tall, [`SHRUNK_TO`] of them have content after the
/// shrink, and the refusal leaves the other fifteen carrying what the [`FILES`]-file frame put
/// there.
pub const STALE_ROWS: usize = H as usize - SHRUNK_TO;

/// **How wide a file name is: twelve columns.** `file-0nn.txt`, and
/// `tests::the_fixture_is_the_size_the_scenes_claim` reads it off the fixture rather than trusting
/// this line.
pub const NAME_COLUMNS: usize = 12;

/// **How many cells of it: 180**, which is [`STALE_ROWS`] times [`NAME_COLUMNS`].
///
/// [`INVERTED_CELLS`]' caveat, and here the arithmetic is exact: the rest of a listing row is pad on
/// both arms, so a stale row costs the **name** and not the row. It is a floor for the same reason,
/// and the product is what says so — a gate written on cells alone would call a screen wrong in
/// fifteen of twenty-four rows a 180-cell defect.
pub const STALE_CELLS: usize = STALE_ROWS * NAME_COLUMNS;

/// **What the stale tail costs on the two counters that move: 1 176 writes against 816, and 72
/// verbs against 57.**
///
/// `(rule, refused)` each, and the direction is the whole argument of the axis: **the refusal is
/// cheaper on both and nothing rises.** `distinct` is 600 on both arms — the first frame already
/// touched every cell the listing has — so the pair a reader would reach for cannot separate them
/// either.
pub const STALE_WRITES: (u64, u64) = (1_176, 816);
/// See [`STALE_WRITES`].
pub const STALE_VERBS: (u64, u64) = (72, 57);

/// **How many of the wheel run's frames leave a reveal request behind under the unconditional arm:
/// twenty-one of [`WHEEL_FRAMES`].**
///
/// The two that do not are the two on which the pull had **just landed** — the second opening frame
/// and the settling frame — where the offset is 0 and the cursor is on row 0, so
/// `Area::into_view` answers nothing for a rectangle that is already visible. That is
/// `crate::wheel::leaves_no_request`'s own sentence — *at offset (0, 0) every arm leaves nothing* —
/// arriving as two frames of a run rather than as a parameter.
pub const PULLS: u32 = 21;

/// **How many frames a wheel run plays: twenty-three.** Two to place the layer and fill the index,
/// [`CLICKS`] with a notch on each, and one for the last notch's reveal to land in.
pub const WHEEL_FRAMES: u32 = 2 + CLICKS + 1;

/// **What `Copy`-only body does to twenty notches: `(20, 0)`.**
///
/// `(the rule, the refusal)`, measured through [`crate::overlay::wheeled`] rather than re-driven —
/// see [`wheeled_by_a_copy`]. Twenty and not [`MOVED`] because that gate starts at offset 0 and
/// this module's runs start at [`WHEELED_FROM`]; the number that matters is the **second**, and it
/// is zero.
pub const BY_A_COPY: (i32, i32) = (CLICKS as i32, 0);

// ── the fixture ──────────────────────────────────────────────────────────────────────────────────

/// The option labels, distinct at every row so a window off by one is a different string.
#[must_use]
pub fn option_labels() -> Vec<String> {
    (0..OPTIONS).map(|i| format!("option {i:03}")).collect()
}

/// The file names, distinct at every row for [`option_labels`]'s reason.
#[must_use]
pub fn file_names() -> Vec<String> {
    (0..FILES).map(|i| format!("file-{i:03}.txt")).collect()
}

/// **What a decode produces.** A free function over an identity and never a closure, so
/// the payload is eight bytes and the pane's equality against it can be the pane's.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Doc(u64);

impl Preview for Doc {
    fn shows(&self) -> u64 {
        self.0
    }
    fn extent(&self) -> (u32, u32) {
        (u32::from(W), u32::from(H))
    }
}

/// The decode, as the preview pane requires it: a free function over an identity.
fn decode(id: u64, _cancel: &Cancel) -> Doc {
    Doc(id)
}

/// The pane's row drawer. **It takes no ink**, which is `file_picker`'s public signature and the
/// reason the pane is not in this module's pictures — see the module header.
fn line(cx: &mut Ctx<'_, '_>, row: Rect, _doc: &Doc, i: u32) {
    let paint = cx.theme().paint(Role::Body);
    let _ = cx.text(row.x, row.y, &format!("row {i:03}"), paint);
}

/// The owner both `select` arms spell, so that a failure can name it.
const OWNER: Id = Id::named("dropped.select");
/// The owner both picker arms spell.
const PICKER: Id = Id::named("dropped.picker");

/// **The rectangle the equality is made over: the popup's interior, less the bar's column.**
///
/// **A reserved bar's thumb is a function of the *extent*, and the offset it is handed is a literal
/// zero.** `overlay`'s shell builds `Span { viewport: area.h, extent: rows, offset: 0 }`
/// (`crate::overlay`'s own `draw_with`), so a popup's bar does **not** move as its list scrolls —
/// what makes it differ between these two arms is that the oracle's list is **shorter**. That is
/// unavoidable rather than incidental: cutting the list at [`SCROLLED_TO`] is exactly what makes the
/// oracle independent of the window arithmetic, and the row count is what the cut changes.
///
/// Measured: the thumb is 9 rows of 24 over [`OPTIONS`] rows and 18 over the tail's
/// `OPTIONS - SCROLLED_TO`, so [`BAR_CELLS`] of that column disagree on the **correct** arm.
/// [`at_rest`] is the equality without it and [`at_rest_whole`] reports it, so the exclusion is a
/// number rather than a convenience.
///
/// **The first draft of this paragraph said *offset* and a review caught it**, which is worth
/// keeping because the two readings send a later session to different places: aligning the two arms'
/// offsets changes nothing at all, and the only lever is the extent — which a tail-cut oracle cannot
/// share and still be one.
pub fn window_interior() -> Rect {
    Rect::new(0, 0, W - 1, H)
}

/// **The rectangle the equality is made over: the picker's listing and nothing else.**
///
/// [`LIST_W`] columns, which is what `picker_body` hands its collection. The pane's columns are
/// excluded because a [`Pen`] cannot see them at all — `file_picker`'s public
/// `line` takes no ink — and the bar's column is inside them. The picker's own shrink surface is
/// the listing's tail, which is what this rectangle is.
pub fn list_rect() -> Rect {
    Rect::new(0, 0, LIST_W, H)
}

// ── scene 39: the popup's window ─────────────────────────────────────────────────────────────────

/// **A `select`'s popup body, drawn into one rectangle in the base pass, over two frames.**
///
/// Two frames and not one: the first seats the owner's focus and the second is the frame on which
/// the popup has taken the keyboard, which is the state an open popup is actually in. A one-frame
/// arm compared against a two-frame one differs on **every cell of the rectangle**, in the paint
/// rather than in the cluster — [`crate::window::Play`]'s finding, one family over.
///
/// `options` is what the arm was handed, so a reference render passes the **tail** of the list;
/// `chosen` and `lead` are indices into that list and are the caller's to keep in step.
fn popup_screen(
    options: &[&str],
    offset: i32,
    chosen: usize,
    lead: usize,
    shape: SelectShape,
) -> Canvas {
    let mut driver = driver_at(W, H, Density::default());
    let mut popup = PopupState::new();
    popup.list.offset = offset;
    popup.list.sel.lead = lead;
    let mut canvas = Canvas::new(W, H);
    for frame in 0..2 {
        let mut pen = Pen::over(canvas);
        let popup = &mut popup;
        driver.frame(|cx| {
            if frame == 0 {
                cx.focus(OWNER);
            }
            let mut child = cx.child(Rect::new(0, 0, W, H));
            popup_body(
                &mut pen, &mut child, popup, options, chosen, 0, shape, offset, OWNER,
            );
        });
        pen.end_frame();
        canvas = pen.into_canvas();
    }
    canvas
}

/// **The correct window against the reference render, over [`window_interior`].** Clean, and it is
/// the arm that says the oracle is an oracle.
///
/// A comparison whose correct arm has never been watched agreeing reports *0 cells over 0 rows* for
/// the same reason a broken one would — `crate::runner::defective`'s own sentence.
#[must_use]
pub fn at_rest() -> Diff {
    against_the_tail(SelectShape::default(), Some(window_interior()))
}

/// **The correct window over the whole rectangle**, which is [`BAR_CELLS`] cells in the bar's own
/// column and nothing anywhere else.
///
/// Reported rather than excluded silently: the difference between this and [`at_rest`] **is** the
/// measurement of what a reserved bar costs a tail-cut oracle, and a reader who is told a column
/// was left out is owed the number.
#[must_use]
pub fn at_rest_whole() -> Diff {
    against_the_tail(SelectShape::default(), None)
}

/// **The inverted window against the reference render.** [`INVERTED_ROWS`] rows of the popup.
#[must_use]
pub fn inverted() -> Diff {
    against_the_tail(
        SelectShape {
            list: inverted_list(),
            ..SelectShape::default()
        },
        Some(window_interior()),
    )
}

/// One arm of the scroll scene, against the option list's own tail drawn at offset 0.
///
/// `over` is the rectangle the equality is made on: [`window_interior`] for the scene's own
/// numbers, `None` for the whole rectangle.
fn against_the_tail(shape: SelectShape, over: Option<Rect>) -> Diff {
    let labels = option_labels();
    let all: Vec<&str> = labels.iter().map(String::as_str).collect();
    let subject = popup_screen(&all, SCROLLED_TO as i32, SCROLLED_TO, SCROLLED_TO, shape);
    let reference = popup_screen(&all[SCROLLED_TO..], 0, 0, 0, SelectShape::default());
    match over {
        Some(r) => subject.cropped(r).diff(&reference.cropped(r)),
        None => subject.diff(&reference),
    }
}

/// The list shape whose window goes the wrong way. See
/// [`crate::collect::defective::inverted_window`].
fn inverted_list() -> crate::collect::CollShape {
    crate::collect::CollShape {
        window: crate::collect::Window::Inverted,
        ..crate::collect::CollShape::RULE
    }
}

// ── scene 41: the picker's tail ──────────────────────────────────────────────────────────────────

/// **A `file_picker`'s body, drawn into one rectangle in the base pass, over a listing that shrinks
/// between the frames.**
///
/// The surface is **not** cleared between frames, which is what makes the shrink axis visible at
/// all: a cell nobody wrote keeps what was already there, and a runner starting each frame from a
/// blank surface would score the stale tail clean. [`crate::runner::play`]'s arrangement.
///
/// `steps` is the listing on each frame, so a reference render passes the short one twice and the
/// subject passes the long one and then the short one.
fn picker_screen(steps: &[usize], shape: PickerShape) -> Canvas {
    let names = file_names();
    let worker = Worker::queueing();
    let task: Task<Doc> = Task::new(&worker);
    let mut body: PickerBody<Doc> = PickerBody::new();
    let mut driver = driver_at(W, H, Density::default());
    let mut canvas = Canvas::new(W, H);
    for (frame, count) in steps.iter().copied().enumerate() {
        let files: Vec<Entry<'_>> = names[..count]
            .iter()
            .enumerate()
            .map(|(i, n)| Entry {
                id: i as u64,
                name: n,
            })
            .collect();
        let mut pen = Pen::over(canvas);
        let body = &mut body;
        let pane_opts = PickerOpts::default().pane;
        driver.frame(|cx| {
            if frame == 0 {
                cx.focus(PICKER);
            }
            let mut child = cx.child(Rect::new(0, 0, W, H));
            picker_body(
                &mut pen, &mut child, PICKER, body, &files, &task, decode, line, LIST_W,
                &pane_opts, shape,
            );
        });
        pen.end_frame();
        canvas = pen.into_canvas();
    }
    canvas
}

/// **The correct build over the same two frames, over [`list_rect`].** Clean, for [`at_rest`]'s
/// reason.
#[must_use]
pub fn shrunk_at_rest() -> Diff {
    against_the_short_listing(PickerShape::default(), Some(list_rect()))
}

/// **The stale tail against a picker built over the short listing.** [`STALE_ROWS`] rows.
#[must_use]
pub fn shrunk() -> Diff {
    against_the_short_listing(omitted_tail(), Some(list_rect()))
}

/// **The correct build over the whole rectangle**, which is the bar's column and the pane's — the
/// second reported for [`at_rest_whole`]'s reason.
#[must_use]
pub fn shrunk_at_rest_whole() -> Diff {
    against_the_short_listing(PickerShape::default(), None)
}

/// One arm of the shrink scene, against a picker that never held the long listing.
fn against_the_short_listing(shape: PickerShape, over: Option<Rect>) -> Diff {
    let subject = picker_screen(&[FILES, SHRUNK_TO], shape);
    let reference = picker_screen(&[SHRUNK_TO, SHRUNK_TO], PickerShape::default());
    match over {
        Some(r) => subject.cropped(r).diff(&reference.cropped(r)),
        None => subject.diff(&reference),
    }
}

/// The list shape that leaves the rows below the content alone. See
/// [`crate::files::defective::a_stale_list_tail`].
fn omitted_tail() -> PickerShape {
    PickerShape {
        list: crate::collect::CollShape {
            tail: crate::collect::Tail::Omitted,
            ..crate::collect::CollShape::RULE
        },
        ..PickerShape::default()
    }
}

/// **The refused spelling, kept as a number rather than as a sentence**: the same refusal played
/// into a rectangle the shrink has already resized.
///
/// The correction to the shrink axis is that *the version written against a terminal resize
/// passes* — a fresh rectangle has nowhere for the residue to survive. Here the rectangle is the
/// short listing's height on both frames, so the refusal draws every row the rectangle has and
/// there is nothing left for a residue to sit in.
#[must_use]
pub fn stale_by_resize() -> Diff {
    let h = SHRUNK_TO as u16;
    let subject = picker_screen_in(&[SHRUNK_TO, SHRUNK_TO], omitted_tail(), h);
    let reference = picker_screen_in(&[SHRUNK_TO, SHRUNK_TO], PickerShape::default(), h);
    let r = Rect::new(0, 0, LIST_W, h);
    subject.cropped(r).diff(&reference.cropped(r))
}

/// [`picker_screen`] in a rectangle of `h` rows, which is what [`stale_by_resize`] needs and the one
/// thing that may stand *beside* the shrink spelling and not instead of it.
fn picker_screen_in(steps: &[usize], shape: PickerShape, h: u16) -> Canvas {
    let names = file_names();
    let worker = Worker::queueing();
    let task: Task<Doc> = Task::new(&worker);
    let mut body: PickerBody<Doc> = PickerBody::new();
    let mut driver = driver_at(W, h, Density::default());
    let mut canvas = Canvas::new(W, h);
    for (frame, count) in steps.iter().copied().enumerate() {
        let files: Vec<Entry<'_>> = names[..count]
            .iter()
            .enumerate()
            .map(|(i, n)| Entry {
                id: i as u64,
                name: n,
            })
            .collect();
        let mut pen = Pen::over(canvas);
        let body = &mut body;
        let pane_opts = PickerOpts::default().pane;
        driver.frame(|cx| {
            if frame == 0 {
                cx.focus(PICKER);
            }
            let mut child = cx.child(Rect::new(0, 0, W, h));
            picker_body(
                &mut pen, &mut child, PICKER, body, &files, &task, decode, line, LIST_W,
                &pane_opts, shape,
            );
        });
        pen.end_frame();
        canvas = pen.into_canvas();
    }
    canvas
}

// ── scenes 40 and 42: the two wheels ─────────────────────────────────────────────────────────────

/// **Which build of the reveal a wheel arm is playing.** [`crate::wheel::Reveal`]'s three, reached
/// through it rather than restated.
pub use crate::wheel::Reveal;

/// **What twenty posted notches did to a popup's offset, and how the keyboard fared.**
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Wheeled {
    /// The offset after the last click's own frame.
    pub after_last_click: i32,
    /// **The offset one frame later**, when the reveal that frame asked for has been applied. The
    /// number the scene is written on.
    pub settled: i32,
    /// How many of the frames left a reveal request behind.
    pub reveals: u32,
}

/// **Twenty posted notches over a `select`'s popup, under one reveal arm.**
///
/// # The layer has to be placed before a notch can be routed, and that is two frames
///
/// `Frame::resolve_wheel` resolves a notch against the **previous** frame's hit index, and a
/// layer's entries only enter that index once the layer has been placed. So the run opens with a
/// pointer position and **two** frames — one that places the layer and one whose index carries it —
/// where [`crate::wheel::wheeled`] opens with one. That is the whole reason these two scenes are in
/// this module and not in that one; see [`WHEELED_SUBJECTS`].
#[must_use]
pub fn wheeled_select(reveal: Reveal) -> Wheeled {
    let labels = option_labels();
    let options: Vec<&str> = labels.iter().map(String::as_str).collect();
    let mut driver = driver_at(W, H, Density::default());
    let mut st = SelectState::new();
    st.open();
    let mut popup = PopupState::new();
    popup.list.offset = WHEELED_FROM;
    let opts = SelectOpts::default();

    let mut reveals = 0;
    driver.post_mouse(at(OVER_THE_POPUP, MouseKind::Move));

    // **Two opening frames and not one.** The first places the layer; the second is the first whose
    // hit index a notch could be resolved against. See [`WHEELED_SUBJECTS`].
    let frame = |driver: &mut Driver,
                 st: &mut SelectState,
                 popup: &mut PopupState,
                 options: &Vec<&str>,
                 reveals: &mut u32| {
        driver.frame(|cx| {
            // **Three named arms and never a shape literal**, which is
            // `crate::wheel::collection_frame`'s arrangement and its reason: a refused build has to
            // differ from the shipped one by **one call**, so a reviewer's diff between them is a
            // single line and the register can point at it. `select_into` is the rule.
            let area = Rect::new(0, 0, W, 1);
            let _ = match reveal {
                Reveal::WhenAsked => {
                    select_into(&mut Direct, cx, OWNER, area, st, popup, options, &opts)
                }
                Reveal::EveryFrame => input_defective::a_popup_revealing_every_frame(
                    &mut Direct,
                    cx,
                    OWNER,
                    area,
                    st,
                    popup,
                    options,
                    &opts,
                ),
                Reveal::Never => input_defective::a_popup_that_never_reveals(
                    &mut Direct,
                    cx,
                    OWNER,
                    area,
                    st,
                    popup,
                    options,
                    &opts,
                ),
            };
        });
        *reveals += u32::from(driver.inspect().into_view().is_some());
    };
    frame(&mut driver, &mut st, &mut popup, &options, &mut reveals);
    frame(&mut driver, &mut st, &mut popup, &options, &mut reveals);
    for _ in 0..CLICKS {
        driver.post_mouse(at(OVER_THE_POPUP, MouseKind::Wheel(Notch::Down)));
        frame(&mut driver, &mut st, &mut popup, &options, &mut reveals);
    }
    let after_last_click = popup.list.offset;
    // **One more frame with no click on it**: the last click's reveal has to have somewhere to land,
    // or the arm is measured a frame before its own defect happens. `crate::wheel::wheeled`'s
    // sentence, unchanged.
    frame(&mut driver, &mut st, &mut popup, &options, &mut reveals);
    Wheeled {
        after_last_click,
        settled: popup.list.offset,
        reveals,
    }
}

/// **Whether a keyboard gesture still brings the cursor into view**, which is the half
/// [`Reveal::Never`] fails.
///
/// Returns the popup's offset after a `Ctrl+Home` posted into a list scrolled a long way from its
/// cursor. The rule and the unconditional arm both follow the keyboard; deleting the call does not,
/// and a gate written on [`wheeled_select`] alone would call that a pass.
///
/// **It is asked of `select` and not of the picker**, which is the keyboard question showing
/// through: an open picker seats no focus, so there is no id a posted key could be routed to. See
/// the module header.
#[must_use]
pub fn revealed_by_a_key(reveal: Reveal) -> i32 {
    let labels = option_labels();
    let options: Vec<&str> = labels.iter().map(String::as_str).collect();
    let mut driver = driver_at(W, H, Density::default());
    let mut st = SelectState::new();
    st.open();
    let mut popup = PopupState::new();
    popup.list.offset = WHEELED_FROM;
    popup.list.sel.lead = OPTIONS - 1;
    let opts = SelectOpts::default();
    for n in 0..4 {
        if n == 2 {
            driver.post_key(keys::press(REVEAL_CHORD));
        }
        let st = &mut st;
        let popup = &mut popup;
        let options = &options;
        driver.frame(|cx| {
            if cx.focused().is_none() {
                cx.focus(OWNER);
            }
            let area = Rect::new(0, 0, W, 1);
            let _ = match reveal {
                Reveal::WhenAsked => {
                    select_into(&mut Direct, cx, OWNER, area, st, popup, options, &opts)
                }
                Reveal::EveryFrame => input_defective::a_popup_revealing_every_frame(
                    &mut Direct,
                    cx,
                    OWNER,
                    area,
                    st,
                    popup,
                    options,
                    &opts,
                ),
                Reveal::Never => input_defective::a_popup_that_never_reveals(
                    &mut Direct,
                    cx,
                    OWNER,
                    area,
                    st,
                    popup,
                    options,
                    &opts,
                ),
            };
        });
    }
    popup.list.offset
}

/// **Twenty posted notches over an open `file_picker`'s listing, under one reveal arm.**
///
/// [`wheeled_select`]'s cadence exactly, one component over. **Driven with a pointer and nothing
/// else**, which is the keyboard question, and it is stated in the module header rather than left
/// for a reader to infer.
#[must_use]
pub fn wheeled_picker(reveal: Reveal) -> Wheeled {
    let names = file_names();
    let files: Vec<Entry<'_>> = names
        .iter()
        .enumerate()
        .map(|(i, n)| Entry {
            id: i as u64,
            name: n,
        })
        .collect();
    let worker = Worker::queueing();
    let task: Task<Doc> = Task::new(&worker);
    let mut body: PickerBody<Doc> = PickerBody::new();
    body.list.offset = WHEELED_FROM;
    let mut st = PickerState::new();
    st.open();
    let opts = PickerOpts::default();
    let mut driver = driver_at(W, H, Density::default());

    let mut reveals = 0;
    driver.post_mouse(at(OVER_THE_LISTING, MouseKind::Move));

    let frame = |driver: &mut Driver,
                 st: &mut PickerState,
                 body: &mut PickerBody<Doc>,
                 files: &Vec<Entry<'_>>,
                 reveals: &mut u32| {
        driver.frame(|cx| {
            // [`wheeled_select`]'s three named arms, one component over. `file_picker_into` is the
            // rule.
            let area = Rect::new(0, 0, W, 1);
            let _ = match reveal {
                Reveal::WhenAsked => file_picker_into(
                    &mut Direct,
                    cx,
                    PICKER,
                    area,
                    st,
                    body,
                    files,
                    &task,
                    decode,
                    line,
                    &opts,
                ),
                Reveal::EveryFrame => files_defective::a_list_revealing_every_frame(
                    &mut Direct,
                    cx,
                    PICKER,
                    area,
                    st,
                    body,
                    files,
                    &task,
                    decode,
                    line,
                    &opts,
                ),
                Reveal::Never => files_defective::a_list_that_never_reveals(
                    &mut Direct,
                    cx,
                    PICKER,
                    area,
                    st,
                    body,
                    files,
                    &task,
                    decode,
                    line,
                    &opts,
                ),
            };
        });
        *reveals += u32::from(driver.inspect().into_view().is_some());
    };
    frame(&mut driver, &mut st, &mut body, &files, &mut reveals);
    frame(&mut driver, &mut st, &mut body, &files, &mut reveals);
    for _ in 0..CLICKS {
        driver.post_mouse(at(OVER_THE_LISTING, MouseKind::Wheel(Notch::Down)));
        frame(&mut driver, &mut st, &mut body, &files, &mut reveals);
    }
    let after_last_click = body.list.offset;
    frame(&mut driver, &mut st, &mut body, &files, &mut reveals);
    Wheeled {
        after_last_click,
        settled: body.list.offset,
        reveals,
    }
}

/// **The family's own fourth wheel defect**, which is neither of the reveal arms: the body holds its
/// list position in a `Copy` of the offset, writes into a value that dies with the frame, and is
/// handed the same number again next frame.
///
/// The literal `Copy`-only body, and it is reached through
/// [`crate::overlay::wheeled`] rather than re-driven here — that gate has existed since components
/// 26 and a second drive loop beside it would be a second answer to one question. What this module
/// adds is that it is **named beside the three reveal arms**, because a wheel gate that plays the
/// reveal and not the `Holds` has not played this family's own defect.
#[must_use]
pub fn wheeled_by_a_copy() -> (i32, i32) {
    (
        crate::overlay::wheeled(true),
        crate::overlay::wheeled(false),
    )
}

/// The chord both keyboard halves post. [`crate::wheel::REVEAL_CHORD`], reached rather than
/// restated.
pub const REVEAL_CHORD: Chord = Chord::new(Code::Home).ctrl();

/// A pointer event at `x`, halfway down the rig.
///
/// **`x` is a parameter and it is the whole of one debugging session.** A `select`'s popup is as
/// wide as the widget it drops from, so the middle of the rig is inside its list; a picker's popup
/// is the room and its **listing is only [`LIST_W`] columns of it**, so the middle of the rig is in
/// the *preview pane*. `Frame::chain_target` finds the innermost hit that is both over and admits
/// the direction, and the pane admits one too — so twenty notches over the middle of a picker moved
/// the listing **0**, which is exactly the reading a dead wheel gives. Measured before it was
/// understood, which is why the two call sites name their column.
fn at(x: u16, kind: MouseKind) -> Mouse {
    Mouse {
        x,
        y: H / 2,
        kind,
        buttons: Buttons::NONE,
        mods: Mods::NONE,
        at: std::time::Instant::now(),
    }
}

/// The column a notch is posted over a `select`'s popup, which is as wide as the rig.
const OVER_THE_POPUP: u16 = W / 2;

/// The column a notch is posted over a picker's **listing**, which is [`LIST_W`] of the rig.
const OVER_THE_LISTING: u16 = LIST_W / 2;

// ── what the counters say about all four ─────────────────────────────────────────────────────────

/// **Which scene a counter comparison is played on.**
///
/// A named argument and not a `match` on the refusal, which is [`crate::window::On`]'s own refusal
/// and its reason: read that way, a refusal the dispatch did not enumerate falls into the other
/// scene's play silently.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum On {
    /// The scrolled popup: at [`SCROLLED_TO`], two frames.
    TheWindow,
    /// The shrunk listing: edited down to [`SHRUNK_TO`], two frames.
    TheTail,
}

/// **Every counter this crate can read, over the rule and over the refusal.** Returns
/// `(rule, refused)`.
///
/// The scenes exist because the pairs are *indistinguishable*, and a report printing only the diff
/// would leave a reader to take on trust that the refused build looked healthier. This is the half
/// that stops that being trust.
#[must_use]
pub fn counters_approve(on: On, allocations: Allocations) -> (Counters, Counters) {
    match on {
        On::TheWindow => {
            let labels = option_labels();
            let all: Vec<&str> = labels.iter().map(String::as_str).collect();
            (
                popup_counters(&all, SelectShape::default(), allocations),
                popup_counters(
                    &all,
                    SelectShape {
                        list: inverted_list(),
                        ..SelectShape::default()
                    },
                    allocations,
                ),
            )
        }
        On::TheTail => (
            picker_counters(PickerShape::default(), allocations),
            picker_counters(
                PickerShape {
                    list: crate::collect::CollShape {
                        tail: crate::collect::Tail::Omitted,
                        ..crate::collect::CollShape::RULE
                    },
                    ..PickerShape::default()
                },
                allocations,
            ),
        ),
    }
}

/// **Which of the nine counters tell a refused build from the rule.**
///
/// [`crate::window::counters_that_separate`]'s shape, one family over. An empty answer means *the
/// equality against a reference render is the only detector there is*.
#[must_use]
pub fn counters_that_separate(on: On, allocations: Allocations) -> Vec<Counter> {
    let (a, b) = counters_approve(on, allocations);
    Counter::ALL
        .into_iter()
        .filter(|c| {
            let (left, right) = (a.get(*c).measured(), b.get(*c).measured());
            left.is_some() != right.is_some() || (left.is_some() && left != right)
        })
        .collect()
}

/// The popup's counters over one shape, on the frame the window is drawn.
fn popup_counters(options: &[&str], shape: SelectShape, allocations: Allocations) -> Counters {
    let mut driver = driver_at(W, H, Density::default());
    let mut popup = PopupState::new();
    popup.list.offset = SCROLLED_TO as i32;
    popup.list.sel.lead = SCROLLED_TO;
    let mut pen = Pen::new(W, H);
    for frame in 0..2 {
        let popup = &mut popup;
        driver.frame(|cx| {
            if frame == 0 {
                cx.focus(OWNER);
            }
            let mut child = cx.child(Rect::new(0, 0, W, H));
            popup_body(
                &mut pen,
                &mut child,
                popup,
                options,
                SCROLLED_TO,
                0,
                shape,
                SCROLLED_TO as i32,
                OWNER,
            );
        });
    }
    Counters::of(&driver, pen.tally(), allocations)
}

/// The picker's counters over one shape, across the shrink.
fn picker_counters(shape: PickerShape, allocations: Allocations) -> Counters {
    let names = file_names();
    let worker = Worker::queueing();
    let task: Task<Doc> = Task::new(&worker);
    let mut body: PickerBody<Doc> = PickerBody::new();
    let mut driver = driver_at(W, H, Density::default());
    let mut pen = Pen::new(W, H);
    for (frame, count) in [FILES, SHRUNK_TO].into_iter().enumerate() {
        let files: Vec<Entry<'_>> = names[..count]
            .iter()
            .enumerate()
            .map(|(i, n)| Entry {
                id: i as u64,
                name: n,
            })
            .collect();
        let body = &mut body;
        let pane_opts = PickerOpts::default().pane;
        driver.frame(|cx| {
            if frame == 0 {
                cx.focus(PICKER);
            }
            let mut child = cx.child(Rect::new(0, 0, W, H));
            picker_body(
                &mut pen, &mut child, PICKER, body, &files, &task, decode, line, LIST_W,
                &pane_opts, shape,
            );
        });
    }
    Counters::of(&driver, pen.tally(), allocations)
}

// ── the subjects, and the scan that says whether they are here ───────────────────────────────────

/// **What the four screens in this module are screens of, between them.** Two, and **no scene
/// claims both** — see [`SELECT_SCREENS`] and [`PICKER_SCREENS`], which is what
/// `crate::scenes::Scene::stands` is populated from.
///
/// It is written out and not reached, because there is no list to reach:
/// [`crate::popup::SUBJECTS`] is `["select", "overlay"]` — the overlay *family*'s two rows — and
/// this is one of those plus a component from another family. **The first draft of this doc claimed
/// it was popup's list "plus the picker, reached rather than restated"**, which was wrong twice
/// over: `overlay` is not here and nothing was reached. Caught by a review.
pub const SUBJECTS: &[&str] = &["select", "file_picker"];

/// **What scenes 39 and 40 stand up: a `select`, and nothing else is on those screens.**
///
/// `crate::scenes::Scene::stands` is *the components that are on the screen*, and this module's
/// `popup_screen` and [`wheeled_select`] draw one component. A list of both owners here would inflate
/// criterion 2's `scenes_for` enumeration in both directions, which is the *generous join* the
/// field's own documentation forbids by name — and the first draft of these four scenes did exactly
/// that, on all four rows. Caught by a review.
pub const SELECT_SCREENS: &[&str] = &["select"];

/// **What scenes 41 and 42 stand up: a `file_picker`.** See [`SELECT_SCREENS`].
pub const PICKER_SCREENS: &[&str] = &["file_picker"];

/// **The components a posted wheel notch is played over here.** Two, and they are scenes 40's and
/// 42's.
///
/// [`crate::window::WHEELED_SUBJECTS`]' counterpart and a third source for
/// `crate::obligations`'s wheel-pair join, which read [`crate::wheel::Subject::ALL`] alone until
/// one pass and those two until another. **A gate's subject list is derived from the gate
/// that plays them**, and the join is the union — written out by hand on either side, a subject
/// escapes both halves while both stay green.
///
/// # Why these two are not `crate::wheel::Subject` arms
///
/// That module's `Run` opens a run with a pointer position and **one** frame. An overlay's entries
/// are in the hit index only once its layer has been placed, and a notch is resolved against the
/// previous frame's index — so a popup needs **two** opening frames, and a drive loop with one
/// cadence cannot have both. It is the reason with a different mechanism behind it:
/// there, a `field` consumed its notch inside its own draw; here, the notch cannot be routed at all
/// until the layer exists.
pub const WHEELED_SUBJECTS: &[&str] = &["select", "file_picker"];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Axis, INVENTORY};

    /// **The oracle is an oracle: the correct window agrees with the option list's own tail
    /// everywhere inside the popup, and the whole-rectangle reading is the bar and nothing else.**
    ///
    /// Both halves, because a comparison whose correct arm has never been watched agreeing reports
    /// *0 cells over 0 rows* for the same reason a broken one would — and because the second half is
    /// what makes [`window_interior`]'s exclusion a measurement rather than a convenience.
    #[test]
    fn the_correct_window_is_the_option_lists_own_tail_and_the_bar_is_the_whole_difference() {
        at_rest().assert_clean("the popup's window against the option list's tail");

        let whole = at_rest_whole();
        assert_eq!(
            (whole.cells, whole.rows),
            (BAR_CELLS, BAR_CELLS),
            "the correct arm's whole-rectangle disagreement is the reserved bar's thumb, which a \
             tail-cut oracle cannot share: `overlay`'s gutter hands the bar an extent and an \
             offset, and the oracle's are the tail's"
        );
        assert_eq!(
            whole.first.map(|(x, _)| x),
            Some(W - 1),
            "and every one of them is in the bar's own column — if the first disagreement were \
             anywhere else the exclusion would be hiding something"
        );

        // **Both arms really do reserve a bar**, asked of `overlay`'s own gutter rather than
        // assumed. Without this the exclusion could go quietly weaker: a popup that stopped
        // reserving one would leave `window_interior` cutting a column of **content** off the
        // equality, and the scene would report a smaller number for a reason nothing recorded.
        assert!(
            crate::overlay::gutter((W, H), OPTIONS as u32).bar,
            "the subject's popup reserves no bar, so `window_interior` is excluding content"
        );
        assert!(
            crate::overlay::gutter((W, H), (OPTIONS - SCROLLED_TO) as u32).bar,
            "the oracle's popup reserves no bar, so the two arms differ about the column for a \
             second reason and the number above is not the thumb"
        );
        assert_eq!(
            window_interior().w,
            W - 1,
            "a bar is one column, which is what `overlay`'s interior subtracts"
        );
    }

    /// **The scrolled popup: the inverted window is [`INVERTED_ROWS`] of [`H`] rows, and no counter says
    /// so.**
    ///
    /// The two halves are the axis's whole argument: the equality against a reference render is the
    /// **only** detector, and the row count says the failure is the screen rather than a band.
    #[test]
    fn the_inverted_popup_window_is_twenty_three_of_twenty_four_rows_and_every_counter_approves() {
        let diff = inverted();
        assert_eq!(
            (diff.cells, diff.rows),
            (INVERTED_CELLS, INVERTED_ROWS),
            "the inverted window inside a layer"
        );
        assert_eq!(
            diff.first.map(|(_, y)| y),
            Some(1),
            "the first row it is wrong about is the **second**: at visible row 0 the two \
             arithmetics meet, which is `crate::listing`'s rows-right-by-coincidence on this \
             component"
        );

        // **The cell count is a floor**, and the arithmetic says why rather than a comment: an
        // option row is a word and a pad, and the pad agrees.
        assert!(
            INVERTED_CELLS < INVERTED_ROWS * usize::from(W - 1),
            "a cell count is a floor on how wrong the screen is, not a measure of it"
        );

        let separated = counters_that_separate(On::TheWindow, Allocations::over(2, 0));
        assert_eq!(
            separated,
            Vec::<Counter>::new(),
            "a counter separates the inverted window, so the scene is optional and this assertion \
             is what says so — it draws *less*, which is why every counter in the stack approved of \
             it three times independently"
        );
    }

    /// **The shrunk listing: the stale tail is [`STALE_ROWS`] of [`H`] rows, it is cheaper on
    /// every counter that moves, and the refused spelling scores clean.**
    #[test]
    fn the_pickers_stale_tail_is_fifteen_of_twenty_four_rows_and_the_resize_spelling_misses_it() {
        shrunk_at_rest().assert_clean("the correct listing against one built over the short one");

        let diff = shrunk();
        assert_eq!(
            (diff.cells, diff.rows),
            (STALE_CELLS, STALE_ROWS),
            "the stale tail behind a layer"
        );
        assert_eq!(
            diff.first.map(|(_, y)| y),
            Some(SHRUNK_TO as u16),
            "the first row it is wrong about is the first row the content no longer reaches"
        );

        // **The refused spelling, as a number.** Played into a rectangle the shrink has already
        // resized, the same refusal is clean: a fresh rectangle has nowhere for the residue to
        // survive, so the arm that leaves fifteen rows standing draws the same screen as the rule.
        stale_by_resize().assert_clean(
            "the resize spelling scores the stale tail clean, which is why §21 refuses it",
        );
    }

    /// **The stale tail is cheaper on both counters that move and nothing rises**, which is what
    /// makes the scene the only detector there is.
    #[test]
    fn the_stale_tail_is_cheaper_on_the_counters_that_move_and_invisible_to_the_rest() {
        let (rule, bad) = counters_approve(On::TheTail, Allocations::over(2, 0));
        assert_eq!(
            (
                rule.get(Counter::Writes).measured(),
                bad.get(Counter::Writes).measured()
            ),
            (Some(STALE_WRITES.0), Some(STALE_WRITES.1))
        );
        assert_eq!(
            (
                rule.get(Counter::Verbs).measured(),
                bad.get(Counter::Verbs).measured()
            ),
            (Some(STALE_VERBS.0), Some(STALE_VERBS.1))
        );
        assert!(
            STALE_WRITES.1 < STALE_WRITES.0 && STALE_VERBS.1 < STALE_VERBS.0,
            "the refused build is cheaper on both, which is the axis's whole argument: no counter \
             in the stack disapproves of a screen that is wrong in fifteen of its rows"
        );

        // **`distinct` cannot separate them**, which is the counter a reader reaches for first: the
        // first frame already touched every cell the listing has, and it is cumulative.
        assert_eq!(
            rule.get(Counter::Distinct).measured(),
            bad.get(Counter::Distinct).measured(),
            "`distinct` is cumulative and the first frame touched every cell, so the pair a reader \
             would reach for is blind here"
        );
        assert_eq!(
            counters_that_separate(On::TheTail, Allocations::over(2, 0)),
            vec![Counter::Writes, Counter::Verbs],
            "and those two are the whole list — both in the refusal's favour"
        );
    }

    /// **Scenes 40 and 42: twenty posted notches settle both owners' windows at [`MOVED`], an
    /// unconditional reveal drags both to [`DRAGGED_BACK`], and deleting the call passes the wheel
    /// half.**
    ///
    /// # Every number is asserted **equal to the other component's** rather than to a constant
    /// twice
    ///
    /// The popup's list is *the collection over the option list*, and the
    /// picker's body is *the shell, then a collection beside a preview pane*. Both say **reached by
    /// calling it**, and nothing had ever asked either one a wheel question. Asked as *`select`
    /// equals `file_picker`, arm for arm*, an owner that grew an offset or a reveal of its own fails
    /// here rather than reporting three constants written twice — which is the shape on
    /// the pair `table`/`collection`, arriving on two components that are siblings rather than one
    /// built on the other.
    #[test]
    fn twenty_posted_notches_settle_both_owners_and_an_unconditional_reveal_drags_both_back() {
        for reveal in [Reveal::WhenAsked, Reveal::EveryFrame, Reveal::Never] {
            assert_eq!(
                wheeled_select(reveal),
                wheeled_picker(reveal),
                "the two overlay owners disagree about the wheel on the `{}` arm, so one of them \
                 has a row axis that is not `collection`'s",
                reveal.word()
            );
        }

        let free = wheeled_select(Reveal::WhenAsked);
        assert_eq!(free.settled, MOVED, "twenty clicks, twenty rows");
        assert_eq!(free.after_last_click, MOVED);
        assert_eq!(free.reveals, 0, "nothing asked, so nothing was requested");
        // **The run ends short of the clamp, and the clamp is asked of the component rather than
        // computed here.** `CollState::max_offset` is the expression `collection` itself applies, so
        // a run that ended *at* it would report the same number for a wheel that worked and a wheel
        // that overshot — `crate::wheel::Subject::axes`' refusal from the other side.
        let clamp = crate::collect::CollState::max_offset(OPTIONS, H);
        assert_eq!(clamp, MAX_OFFSET, "the fixture's clamp is the component's");
        assert!(
            free.settled < clamp,
            "the run ended at the clamp, where a wheel that worked and one that overshot report \
             the same number"
        );

        let dragged = wheeled_select(Reveal::EveryFrame);
        assert_eq!(
            dragged.settled, DRAGGED_BACK,
            "an unconditional `scroll_into_view` and the pointer is dead — inside a layer this \
             time, which is the one place the four resolved tickets' defect had never been played"
        );
        assert_eq!(
            dragged.after_last_click, 1,
            "one click ahead of its own correction, which is ADR 0015's documented residue seen \
             from the other side rather than a softened defect"
        );
        assert_eq!(
            dragged.reveals, PULLS,
            "twenty-one of the twenty-three frames asked; the two that did not are the two on \
             which the pull had just landed"
        );

        // **The third arm, and it is why a one-directional gate is not a gate.** Deleting the call
        // passes the wheel half exactly.
        let deleted = wheeled_select(Reveal::Never);
        assert_eq!(
            (deleted.settled, deleted.reveals),
            (MOVED, 0),
            "deleting the reveal moves the window exactly as far as the rule does, so a gate \
             written on the wheel alone calls it a pass"
        );
    }

    /// **The half [`Reveal::Never`] fails: a keyboard gesture still brings the cursor into view.**
    ///
    /// **Asked of `select` and not of the picker**, which is the keyboard question showing
    /// through rather than a decision about the axis — an open `file_picker` seats no focus, so
    /// there is no id a posted key could be routed to. See this module's header.
    #[test]
    fn deleting_the_reveal_loses_the_keyboard_and_only_the_select_can_be_asked() {
        assert_eq!(
            revealed_by_a_key(Reveal::WhenAsked),
            0,
            "`Ctrl+Home` moved the cursor to the top and the window followed it"
        );
        assert_eq!(
            revealed_by_a_key(Reveal::EveryFrame),
            0,
            "the unconditional arm follows the keyboard too, which is why the wheel half cannot \
             be the whole gate"
        );
        assert_eq!(
            revealed_by_a_key(Reveal::Never),
            WHEELED_FROM,
            "and deleting the call leaves the window where it was with the cursor off screen: the \
             way to pass a one-directional gate, and it is not a fix"
        );
    }

    /// **The family's own fourth wheel defect, named beside the three reveal arms.**
    ///
    /// A wheel gate that plays the reveal has not played this: the literal `Copy`-only body moves
    /// the offset **0** in twenty notches, on a screen that is identical while it happens. Reached
    /// through [`crate::overlay::wheeled`] rather than re-driven, because that gate has existed
    /// since components 26 and a second drive loop beside it would be a second answer to one
    /// question.
    #[test]
    fn the_copy_only_body_is_a_fourth_wheel_defect_and_neither_reveal_arm_is_it() {
        assert_eq!(wheeled_by_a_copy(), BY_A_COPY);
        assert_eq!(
            BY_A_COPY.1, 0,
            "the body writes into a value that dies with the frame and the owner hands it the same \
             number again"
        );
        // **And it is not either reveal arm**: the `Copy` body's offset never moves at all, where
        // the unconditional reveal's moves and is pulled back. Two mechanisms, one number on the
        // settled reading, and only one of them is reachable through `crate::collect`.
        assert_ne!(
            wheeled_select(Reveal::EveryFrame).after_last_click,
            BY_A_COPY.1,
            "the two defects differ one frame before they settle, which is the only reading that \
             separates them"
        );
    }

    /// **Criterion 6, from the gate's side: the freeze declares the axis for every pair this module
    /// claims.**
    ///
    /// `crate::obligations` asserts the other direction — O5 holds a pair for every subject a
    /// posted notch is played over. The two are not the same question and neither implies the
    /// other.
    #[test]
    fn the_freeze_declares_all_four_axes_this_module_claims() {
        let claims: [(&str, Axis); 4] = [
            ("select", Axis::Scrolled),
            ("select", Axis::Wheeled),
            ("file_picker", Axis::Shrunk),
            ("file_picker", Axis::Wheeled),
        ];
        for (id, axis) in claims {
            let component = INVENTORY
                .iter()
                .find(|c| c.id == id)
                .unwrap_or_else(|| panic!("`{id}` is in the freeze"));
            assert!(
                component.declares(axis),
                "this module claims `{id}` is {} and the freeze does not",
                axis.name()
            );
        }

        // **And `select` declares neither of the other two**, which is why this module has two
        // scenes for it and not four: the shut face elides at every width through the one elision
        // this crate has, and a popup's extent is a fixpoint rather than a residue.
        let select = INVENTORY.iter().find(|c| c.id == "select").expect("frozen");
        assert!(!select.declares(Axis::Shrunk) && !select.declares(Axis::Narrow));
        // The picker declares `scrolled` too and scene 25 is its evidence — an earlier pass's, and
        // the memo-key rule rather than a window.
        let picker = INVENTORY
            .iter()
            .find(|c| c.id == "file_picker")
            .expect("frozen");
        assert!(picker.declares(Axis::Scrolled) && !picker.declares(Axis::Narrow));
    }

    /// **The four screens stand on two declared components**, which is criterion 8's verdict from
    /// the direction that can go quietly wrong.
    ///
    /// **Two scans and not one**, and that is a review finding on this test's first draft: it read
    /// [`crate::popup::subjects_declared`] alone and asserted `select`, so `file_picker`'s
    /// declaration was **never scanned** — and this test is one of only two instruments pinning
    /// scenes 41 and 42, both of which are the picker's. The picker is homed in another family, so
    /// its scan is [`crate::preview::subjects_declared`]'s. Both are reached rather than copied: a
    /// third scan written here would be a third answer to *is this component declared*, and
    /// `crate::window::the_three_screens_stand_on_a_declared_field` is the same arrangement one
    /// family over.
    #[test]
    fn the_four_screens_stand_on_two_declared_components() {
        assert!(
            crate::popup::subjects_declared().contains(&"select"),
            "`select` is not declared where the freeze homes it, so scenes 39 and 40 are waiting \
             for their subject rather than measuring one"
        );
        assert!(
            crate::preview::subjects_declared().contains(&"file_picker"),
            "`file_picker` is not declared where the freeze homes it, so scenes 41 and 42 are \
             waiting for their subject rather than measuring one"
        );
        assert_eq!(
            SUBJECTS, WHEELED_SUBJECTS,
            "the two lists happen to be the same population today and they are two facts: four \
             screens stand these components up between them, and two of the four post a notch"
        );
        // **And no scene claims both**, which is `Scene::stands`' own rule — *the components that
        // are on the screen* — and the join `scenes_for` enumerates.
        assert_eq!(
            [SELECT_SCREENS, PICKER_SCREENS].concat(),
            SUBJECTS,
            "the per-scene lists and the module's population have parted"
        );
        for id in SUBJECTS {
            assert!(
                INVENTORY.iter().any(|c| c.id == *id && c.built),
                "`{id}` is not built, so a scene of it is not evidence of anything"
            );
        }
    }

    /// **The fixture is the size the scenes claim**, so a scene stating a content nothing asserts
    /// cannot drift — the failure mode arriving as a number.
    #[test]
    fn the_fixture_is_the_size_the_scenes_claim() {
        assert_eq!(option_labels().len(), OPTIONS);
        assert_eq!(file_names().len(), FILES);
        // **A file name's width is read off the fixture**, because `STALE_CELLS` is a product over
        // it: a name that grew a digit would make that constant a number nobody had measured.
        for name in file_names() {
            assert_eq!(name.chars().count(), NAME_COLUMNS);
        }
        assert!(
            NAME_COLUMNS < usize::from(LIST_W),
            "a name as wide as the listing would make a stale row cost the whole row, and the \
             floor would stop being a floor"
        );
        assert_eq!(
            OPTIONS, FILES,
            "one length, so the two clamps are one number"
        );
        assert!(
            OPTIONS > usize::from(H),
            "a list that fits its window has no window to be wrong about"
        );
        assert!(
            SHRUNK_TO < usize::from(H) && SHRUNK_TO > 1,
            "the shrink leaves a tail, and it leaves a rectangle `stale_by_resize` can be played in"
        );
        assert_eq!(STALE_ROWS, usize::from(H) - SHRUNK_TO);
        assert_eq!(WHEEL_FRAMES, 23);
    }

    /// **An open `file_picker` answers the keyboard, and the arm that shipped answered none of it.**
    ///
    /// The keyboard question, and the population it repairs is *the whole keyboard*: until it was
    /// answered `picker_body` seated no focus and declared no [`Refusal`](crate::collect::Refusal),
    /// so a picker could only be used with a mouse. **It rendered perfectly**, which is why nothing
    /// caught it — every other gate over this component drives it with a pointer or asserts about
    /// the pane, and a key that is never answered leaves no mark on a canvas. So this is the one
    /// instrument in the file that is not a screen comparison, and the defective arm is
    /// [`crate::files::defective::a_popup_with_no_keyboard`]: the same function with one field
    /// changed, which is this crate's rule about second implementations.
    ///
    /// Four properties, one per decision the issue left open plus the wake:
    ///
    /// - **the cursor moves and the type-ahead seeks**, which is the focus being seated on the
    ///   *list* — the first decision, and the pane is not a candidate because its document is
    ///   a function of that cursor;
    /// - **`Enter` answers the cursor's file**, which is the same expression a click answers with,
    ///   so one meaning has two triggers;
    /// - **`Esc` closes and leaves `chosen` alone**, which is how a dismissal restores here — the
    ///   second decision, and it is not `select`'s, because a picker's `chosen` starts empty and
    ///   *answer what was already there* would say nothing;
    /// - **nothing is declined on the seated arm**, and every key is declined on the other, which is
    ///   what *pointer-only* means as a number.
    #[test]
    fn an_open_picker_answers_the_keyboard_and_the_arm_that_shipped_answered_none_of_it() {
        /// `(cursor, chosen, still open, keys the application saw)`.
        fn drive(seated: bool, chords: &[Chord]) -> (usize, Option<u64>, bool, usize) {
            const NAMES: [&str; 4] = ["alpha", "beta", "gamma", "delta"];
            let files: Vec<Entry<'_>> = NAMES
                .iter()
                .enumerate()
                .map(|(i, n)| Entry {
                    id: i as u64,
                    name: n,
                })
                .collect();
            let worker = Worker::queueing();
            let task: Task<Doc> = Task::new(&worker);
            let mut body: PickerBody<Doc> = PickerBody::new();
            let mut st = PickerState::new();
            st.open();
            let opts = PickerOpts::default();
            let mut driver = driver_at(W, H, Density::default());
            let mut declined = 0;

            let frame = |driver: &mut Driver,
                         st: &mut PickerState,
                         body: &mut PickerBody<Doc>,
                         files: &Vec<Entry<'_>>,
                         seat: bool,
                         declined: &mut usize| {
                driver.frame(|cx| {
                    if seat {
                        cx.focus(PICKER);
                    }
                    let area = Rect::new(0, 0, W, 1);
                    let _ = if seated {
                        file_picker_into(
                            &mut Direct,
                            cx,
                            PICKER,
                            area,
                            st,
                            body,
                            files,
                            &task,
                            decode,
                            line,
                            &opts,
                        )
                    } else {
                        files_defective::a_popup_with_no_keyboard(
                            &mut Direct,
                            cx,
                            PICKER,
                            area,
                            st,
                            body,
                            files,
                            &task,
                            decode,
                            line,
                            &opts,
                        )
                    };
                });
                *declined += driver.unhandled().len();
            };

            // **Two opening frames**, which is `crate::wheel`'s own finding one family over: the
            // owner is focused on the first, the body takes the keyboard on it, and `next_key`
            // answers the *previous* frame's focus — so the first key can only land on the third.
            frame(&mut driver, &mut st, &mut body, &files, true, &mut declined);
            frame(
                &mut driver,
                &mut st,
                &mut body,
                &files,
                false,
                &mut declined,
            );
            for &c in chords {
                driver.post_key(keys::press(c));
                frame(
                    &mut driver,
                    &mut st,
                    &mut body,
                    &files,
                    false,
                    &mut declined,
                );
            }
            // **The delivery frame the body asked for.** A body answers through the inbox and the
            // owner reads the slot at the *top* of the next frame, so the frame that carries the
            // keystroke is never the frame that acts on it — which is why `picker_body` calls
            // `Ctx::request_frame` when either slot fills, and why an application that parks on
            // `Driver::wait` would otherwise see the choice land on whatever input happened next.
            frame(
                &mut driver,
                &mut st,
                &mut body,
                &files,
                false,
                &mut declined,
            );
            (body.list.sel.lead, st.chosen(), st.is_open(), declined)
        }

        // ── the cursor moves, and a letter seeks ──────────────────────────────────────────────
        let (lead, chosen, open, declined) = drive(true, &[Chord::new(Code::Down)]);
        assert_eq!(lead, 1, "`Down` moves the picker's cursor");
        assert_eq!(chosen, None, "and chooses nothing on its own");
        assert!(open, "and leaves the popup standing");
        assert_eq!(declined, 0, "and the application never saw the key");

        let (lead, ..) = drive(true, &[Chord::typed('g')]);
        assert_eq!(
            lead, 2,
            "the type-ahead finds `gamma`, which is the list's own `find`"
        );

        // ── `Enter` answers the cursor's file, and the owner takes it ─────────────────────────
        let (_, chosen, open, _) = drive(true, &[Chord::new(Code::Down), Chord::new(Code::Enter)]);
        assert_eq!(chosen, Some(1), "`Enter` answers the file under the cursor");
        assert!(
            !open,
            "and the owner closes the popup when it takes the answer"
        );

        // ── `Esc` closes and restores by saying nothing ───────────────────────────────────────
        let (_, chosen, open, _) = drive(true, &[Chord::new(Code::Down), Chord::new(Code::Escape)]);
        assert_eq!(
            chosen, None,
            "`Esc` leaves `chosen` exactly as it was, which is the restore"
        );
        assert!(!open, "and closes the popup");

        // ── and the arm that shipped answers none of it ───────────────────────────────────────
        //
        // **The keys do not vanish — they stay with the *owner*.** Nothing seats a focus inside the
        // body, so the owner's own drain loop is still what `next_key` answers, and its four binds
        // go on meaning *open* and *close*: `Down` and `Enter` are `st.open()` on a picker that is
        // already open, which is a no-op, and only the letter is declined. Those four are exactly
        // `contract::OVERLAY_OWNER`, and they are the 8 in `PICKER_IS_MISSING`'s *35 against 8*.
        let keys = [
            Chord::new(Code::Down),
            Chord::typed('g'),
            Chord::new(Code::Enter),
        ];
        let (lead, chosen, open, declined) = drive(false, &keys);
        assert_eq!(lead, 0, "pointer-only: the cursor never moved");
        assert_eq!(chosen, None, "pointer-only: `Enter` chooses no file");
        assert!(open, "pointer-only: and nothing here shut the popup");
        assert_eq!(
            declined, 1,
            "only the letter reaches the application: the other two are the *owner's* binds \
             answered at the owner, which is what makes the gap 27 rather than 35"
        );

        // And `Esc` on that arm is the owner's *close it*, not the body's *cancel* — the same
        // keystroke, a different reader, which is the half of the defect a spelling count cannot
        // see and the reason the seated arm asserts `chosen` and not just `open`.
        let (_, chosen, open, _) = drive(false, &[Chord::new(Code::Escape)]);
        assert!(
            !open,
            "the owner closes on `Esc` whether the body has a keyboard or not"
        );
        assert_eq!(chosen, None, "and chooses nothing either way");
    }
}
