//! **The listing: a million rows in a viewport, and the four hostile axes standing on one screen.**
//!
//! This is the screen `collection`'s scenes are
//! scenes *of*, and it exists for the reason [`crate::dense`] exists one file over: **every one of
//! the four axes was established by a defect that passed every gate then in force and looked
//! healthier than the correct build**, and three of the four were caught only by an equality against
//! a reference render.
//!
//! | scene | what it decides |
//! |---|---|
//! | a scrolled collection | the inverted scroll sign: it draws nothing, at a fifth of the cost, and **every counter approves** |
//! | a collection shorter than its viewport | the stale tail: **71 of 80 rows**, the defective build 2.3x faster marking 226x less |
//! | twenty wheel clicks | the unconditional `scroll_into_view`, and it lives in [`crate::wheel`] now |
//! | a narrow collection | truncation, and the one-cell ellipsis inside a row |
//! | 1k / 100k / 1M rows | one store, one window: **identical writes and identical regions** |
//!
//! # Two instruments, because one instrument cannot ask both questions
//!
//! The module is deliberately not one `draw_into` the way [`crate::dense`] is, and the reason is a
//! coordinate system rather than a preference.
//!
//! 1. **The equality scenes go through [`crate::runner`]**, over a [`Fixture`] whose own `offset`
//!    field is the scroll. That is the instrument and it is reused rather than
//!    rebuilt: [`stale`] is `71 of 80 rows` and `2 840` cells because
//!    `runner::tests::the_runner_catches_a_stale_tail_after_content_shrinks_inside_a_rectangle_that_
//!    does_not_move` already reproduced the figure exactly, and a second reproduction of a
//!    number is a second thing to keep in step.
//! 2. **The volume scene goes through a real [`Ctx::scroll_scope`] and [`Ctx::visible_rows`]**,
//!    measured with a [`Tally`] and the frame's own hit index. It cannot go through [`Pen`]: a
//!    scrolled context **is** a content coordinate system, so a listing at offset 1 000 writes at
//!    `y = 1 000` and a [`Canvas`] eighty rows tall discards it. That is the same collision
//!    [`crate::dense::modal_steady`] names for `Ctx::child`, arriving on the one gesture that makes
//!    a collection a collection.
//! # The wheel scene has moved, and the two substitutions that made it move are gone
//!
//! The wheel gate was kept here, with **two stand-ins**. A wheel
//! click could not be posted from this crate — `Driver::post_mouse` takes a `vitui_engine::Mouse`,
//! which was `EngineName { name: "Mouse", reachable_as: None }`, and a `Mouse` needs a `Buttons` and
//! a `MouseKind`, and *neither of those was in `ENGINE_NAMES` at all* — so the click's **delta** was
//! handed to the arithmetic `Response::scrolled` would have delivered it to. And the body it played
//! over was a row loop written beside the gate, because `collection` did not exist yet.
//!
//! **That first sentence is what the re-export acted on**, and the rule it settled
//! is about *construction* rather than about naming precisely because this module found the
//! difference. Components **20** spent it: the gate is [`crate::wheel`], it posts a real notch, it
//! plays over the shipped [`crate::collect::collection`] and over [`crate::scroll::scroll_area`],
//! and it runs the two subjects separately because the blindness is per axis. Two rows
//! went green there.
//!
//! # The gate that stayed here is red for one reason, and it was never the wheel's
//!
//! The four **equality** scenes and the volume scene were red because `collection` did not exist.
//! [`standing`] is a [`Verdict`] over one subject, [`subjects_declared`] opens the file the freeze
//! homes it in, and [`owed_message`] is the sentence that separates *waiting for its subject* from
//! *The code is wrong* — criterion 7, inherited whole. Components **12** inverted them,
//! and the wheel scene stayed red for eight more tickets, which is exactly why a single
//! `inverted_by` on all five would have erased something.
//!
//! # What the volume scene found, and why criterion 6 asks for **regions**
//!
//! The register carries *writes flat 1k -> 1M* as row 4. **That row is green on a build that
//! declares a million hit entries**, and this screen is where that is watched happening: the engine
//! reports a fully clipped verb as **zero columns**, so a listing that iterates its whole content
//! and lets the clip reject the rest writes exactly what the windowed one writes — [`WRITES`], at
//! every volume. What moves is `verbs` and `regions`, and only `regions` is a *frame structure*
//! rather than a count of calls. So the registered equality is
//! **`regions identical at 1k and 1M`**, and it is a different question from row 4 rather than a
//! restatement of it.
//!
//! [`Canvas`]: crate::runner::Canvas
//! [`Ctx::scroll_scope`]: vitui_runtime::Ctx::scroll_scope
//! [`Ctx::visible_rows`]: vitui_runtime::Ctx::visible_rows
//! [`Fixture`]: crate::runner::Fixture
//! [`Pen`]: crate::runner::Pen
//! [`Tally`]: crate::counters::Tally

use std::time::{Duration, Instant};

use vitui_runtime::{Ctx, Density, Interest, Role};

use crate::collect::{CollOpts, CollState, collection_into, defective as coll_defective};
use crate::counters::{Allocations, Counter, Counters, Tally};
use crate::frame::{Face, face_paint};
use crate::ink::{Direct, Ink};
use crate::obligations::Verdict;
use crate::runner::{Diff, Fixture, Pen, compare, defective, play, reference, rows_at_a_time};
use crate::text::{FitOpts, fit_into};
use vitui_runtime::Rect;

// ── the screen ───────────────────────────────────────────────────────────────────────────────────

/// The viewport's width. **Forty**, because the stale tail is *71 of 80 rows* and components
/// It was reproduced at 2 840 cells — `71 x 40`. Measuring the same defect at a second width
/// would report a second number for one finding.
pub const W: u16 = 40;
/// The viewport's height. **Eighty**, which is where *71 of 80* comes from.
pub const H: u16 = 80;
/// The width the row truncates at, and the narrow half of the narrow scene.
///
/// Sixteen columns: shorter than [`ROW`]'s thirty-three, so every row elides; and wide enough that
/// the elision is a *row* question rather than a degenerate one — [`crate::glyphs::elide`] reserves
/// one cell whatever the width, and a two-cell rectangle would make the marker the row.
pub const NARROW: u16 = 16;

/// **What a row says. Static, and that is deliberate.**
///
/// Thirty-three columns: longer than [`NARROW`] and shorter than [`W`], which is what puts
/// one-cell ellipsis rule on the screen at the narrow size and off it at the wide one. A row whose
/// length varied with its index would move the write count for a reason that is not the
/// construction — [`crate::dense::widgets`] makes the same choice for the same reason.
pub const ROW: &str = "listing row, ready, nothing wrong";

/// The three volumes stated for it.
pub const VOLUMES: [u64; 3] = [1_000, 100_000, 1_000_000];

/// **How many rows the shrink starts from. Two hundred**, and it is a constant because
/// [`crate::grid`] plays the same shrink one component up and a second literal would be a second
/// home for one number.
///
/// The property it is chosen for is *more than [`H`]*, so every row of the viewport has content on
/// the first frame and the residue the shrink leaves is the whole tail.
pub const FULL_ROWS: usize = 200;

/// **Rect a correct frame writes, at every one of [`VOLUMES`].** Every cell of the viewport, once.
pub const WRITES: u64 = W as u64 * H as u64;

/// **Interactive regions a correct frame declares, at every one of [`VOLUMES`].**
///
/// `1 + 80`: **one hit entry for the collection** — `Response::local` resolves the row by
/// arithmetic, so per-row hover needs no per-row index entry) plus one target on each row that
/// actually drew. A row outside the window has no entry, which is the runtime's half of the rule:
/// state for an undrawn row is state nothing can reach.
pub const REGIONS: usize = 1 + H as usize;

/// **Tab stops a correct frame declares, at every volume. One.**
///
/// > A virtualised collection is one tab stop.
///
/// The ring is built from what drew, so a row outside the window has no entry and no rectangle;
/// mapping a selection index to an offset is the container's job and a different mechanism.
pub const STOPS: usize = 1;

/// Drawing verbs a correct frame makes: one [`Ctx::text`] a visible row.
///
/// [`Ctx::text`]: vitui_runtime::Ctx::text
pub const VERBS: u64 = H as u64;

/// How far the listing is scrolled for the scrolled scene. The offset.
pub const SCROLLED_TO: usize = 60;

/// **Rows the scrolled equality reports different. Seventy-five of eighty, and not seventy-nine.**
///
/// The five that coincide are an artefact of the fixture and are written down rather than tuned
/// away. [`crate::runner::Fixture::lines`] generates column `c` of row `i` as `(5i + c) mod 26`, so
/// two rows agree at one column exactly when they agree at all of them — `i ≡ j (mod 26)`. The
/// inverted arm puts content row `offset − y` where row `offset + y` belongs, and those are
/// congruent when `2y ≡ 0 (mod 26)`, which is **every thirteenth row**: `y = 0, 13, 26, 39, 52`
/// inside the sixty-one rows the arm draws content on at all.
///
/// So the reference-render equality **under-reports this defect by five rows** — and it is still the
/// only instrument in the stack that reports it at all, which is the scene's whole argument rather
/// than an exception to it. See [`counters_approve`].
pub const SCROLLED_ROWS: usize = 75;

/// [`SCROLLED_ROWS`] as cells — `75 x 40`. A wrong row costs exactly [`W`] cells, which is the
/// property `Fixture::lines` is generated to have.
pub const SCROLLED_CELLS: usize = SCROLLED_ROWS * W as usize;

/// Rows the inverted arm gets right by coincidence. See [`SCROLLED_ROWS`].
pub const SCROLLED_ALIASED: usize = H as usize - SCROLLED_ROWS;

/// How many rows are left when the content shrinks inside a rectangle that does not move.
pub const SHRUNK_TO: usize = 9;

/// **Rows that differ after the shrink. The number.**
pub const STALE_ROWS: usize = 71;

/// [`STALE_ROWS`] as cells — `71 x 40`, which is what was measured.
pub const STALE_CELLS: usize = STALE_ROWS * W as usize;

// ── the volume scene: one store, one window ──────────────────────────────────────────────────────

/// **Which listing is drawn**, and the difference between them is one line of the row loop.
///
/// Both are legitimate constructions and the runtime says which is which: a scroll area draws the
/// whole content and lets the clip reject what is off screen, which is right for a form and
/// **887-889x** for a million rows; a virtualised collection asks
/// [`Ctx::visible_rows`](vitui_runtime::Ctx::visible_rows) which rows can be reached and draws
/// those. Choosing the wrong one is *the single most expensive mistake available above this runtime*
/// (`crates/vitui-runtime/src/scroll.rs`), and it is expensive because **both compile and both look
/// right on a thousand rows**.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Volume {
    /// The window and nothing else. Cost proportional to the visible rows.
    Windowed,
    /// The whole content, clipped. Cost proportional to the data.
    WholeContent,
}

impl Volume {
    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Volume::Windowed => "windowed",
            Volume::WholeContent => "whole content",
        }
    }
}

/// **What one frame of the listing turned out to be**, returned rather than printed.
///
/// A number only a report prints is a number no gate can read — [`crate::dense::Shape`]'s rule, and
/// it is the reason criterion 5's *identical writes and identical regions* is an assertion here
/// rather than three lines of a table somebody compares by eye.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Shape {
    /// Columns the engine reported written.
    pub writes: u64,
    /// Distinct cells touched.
    pub distinct: u64,
    /// Drawing calls made.
    pub verbs: u64,
    /// Entries in the runtime's hit index.
    pub regions: usize,
    /// Entries in the focus ring.
    pub stops: usize,
    /// **Rows the body actually iterated.** The mechanism, where the three counters above are the
    /// symptom — `crates/vitui-runtime/src/scroll.rs` counts what each body iterated rather than
    /// timing it, *because the count is the mechanism and the timing is the weather*.
    pub iterated: u64,
}

/// **Draw one frame of the listing at `rows` rows, scrolled to `offset`.**
///
/// The rectangle is the whole context, so there is one scroll area and it is the screen.
///
/// # It is the component now, and that is what turned four of these scenes green
///
/// This was a stand-in row loop, because [`crate::collect::collection`]
/// did not exist. It does (components 12), so the stand-in is gone: **both arms are the component**
/// — [`Volume::Windowed`] is [`collection_into`] and [`Volume::WholeContent`] is
/// [`crate::collect::defective::whole_content`], which is that function with one value changed —
/// and what belongs to this screen is the row drawer, the geometry and the counting.
///
/// The row signature is `(cx, rect, index, Face)`, with a fifth argument the [`Ink`] seam adds
/// so a gate can see the cells; the shipped four are what [`crate::collect::collection`] hands a
/// component author.
///
/// Generic over [`Ink`] so that a [`Tally`] and a [`Direct`] measure **the same drawing path**
/// rather than a copy of it — [`crate::ink`]'s whole argument, and the reason [`volume_over`] and
/// [`volume_cost`] can report a count and a cost about one function.
///
/// [`Direct`]: crate::ink::Direct
/// [`Ink`]: crate::ink::Ink
/// [`collection_into`]: crate::collect::collection_into
pub fn draw_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    kind: Volume,
    rows: u64,
    offset: i32,
) -> u64 {
    let view = cx.area();
    // **`Rows::of` and not a stated revision** (an earlier pass): this screen has no order behind it,
    // so its positions cannot be permuted and it is never told they went stale. `Rows::of` carries
    // `Revision::UNKNOWN`, which never matches, and the component's comparison is guarded on
    // `is_known()`.
    let len = crate::order::Rows::of(usize::try_from(rows).unwrap_or(usize::MAX));
    let opts = CollOpts::default();
    let mut state = CollState::new();
    state.offset = offset;

    let mut iterated = 0u64;
    // **The listing has no search**, which is not a hole: type-ahead is *the caller's search behind
    // a budget*, and this screen's caller is a measurement with no labels in it.
    // `crate::collect::type_ahead_cost` is where the budget is priced.
    let find = |_: &str, _: std::ops::Range<usize>| None;
    let row = |ink: &mut I, cx: &mut Ctx<'_, '_>, r: Rect, i: usize, face: Face| {
        iterated += 1;
        // A row's own target, per target and for visible rows only. The `WholeContent` arm
        // declares one for every row of the content, which is the defect: `Ctx::declare` does
        // not clip, so an off-screen declaration is a real entry in a real index.
        cx.with_key(i as u64, |cx| {
            let id = cx.id();
            let _ = cx.interact(id, r, Interest::CLICK);
            let paint = face_paint(cx.theme(), face);
            let written = ink.text(cx, r.x, r.y, ROW, paint);
            // The trailing pad, so the row is a **partition** of its width and not a prefix of
            // it. `Ink::run` and not a `str::repeat`: under `Direct` it stages once and
            // blits once, which is the verb a padding band is supposed to be and costs the
            // frame no allocation.
            let _ = ink.run(
                cx,
                r.x + i32::from(written),
                r.y,
                " ",
                W.saturating_sub(written),
                paint,
            );
        });
    };

    let _ = match kind {
        Volume::Windowed => collection_into(ink, cx, view, &mut state, &opts, len, find, row),
        Volume::WholeContent => {
            coll_defective::whole_content(ink, cx, view, &mut state, &opts, len, find, row)
        }
    };
    iterated
}

/// The largest offset `rows` rows admit in an [`H`]-row viewport. *Content minus viewport*, floored.
pub fn max_offset(rows: u64) -> i32 {
    i32::try_from(rows.saturating_sub(u64::from(H))).unwrap_or(i32::MAX)
}

/// **The listing at one volume, as a shape.** Criterion 5's instrument.
///
/// One warm-up frame and one measured one, because the five frame structures take their allocation
/// on the first frame that needs one and keep it — the same warming `tests/budget.rs` does before it
/// opens its window, and the reason a cold frame is not a frame.
pub fn volume(kind: Volume, rows: u64) -> Shape {
    volume_over(kind, rows, 1).0
}

/// [`volume`], over `frames` measured frames, with the **per-frame** time beside it.
///
/// **The duration is a report and never a gate.** R15's rule, inherited unchanged: *a gate is a
/// count, a ratio, an equality or a compile outcome; a timing is a report* — and the reason is on
/// this page, because three of the four hostile axes were **faster**.
///
/// The driver is attached **outside** the measurement and one frame is drawn before the clock
/// starts. Both are stated rather than assumed: `Driver::headless` plus `Theme::authored` is tens of
/// microseconds of one-off work, and a figure that carried it would be a report about attaching a
/// terminal.
///
/// # Panics
///
/// Panics on zero frames. A per-frame figure over no frames is a division by zero dressed as a
/// measurement.
pub fn volume_over(kind: Volume, rows: u64, frames: u32) -> (Shape, Duration) {
    assert!(frames > 0, "a per-frame figure needs a frame");
    let mut driver = crate::runner::driver_at(W, H, Density::default());
    let mut tally = Tally::new();
    let mut iterated = 0;

    // The warm-up, and its tally is thrown away: `writes` and `verbs` are per-frame counters.
    driver.frame(|cx| iterated = draw_into(&mut tally, cx, kind, rows, 0));

    let mut elapsed = Duration::ZERO;
    let mut shape = Shape::default();
    for _ in 0..frames {
        let mut frame_tally = Tally::new();
        let started = Instant::now();
        driver.frame(|cx| iterated = draw_into(&mut frame_tally, cx, kind, rows, 0));
        elapsed += started.elapsed();
        let frame = driver.inspect();
        shape = Shape {
            writes: frame_tally.writes(),
            distinct: frame_tally.distinct(),
            verbs: frame_tally.verbs(),
            regions: frame.hits().len(),
            stops: frame.stop_count(),
            iterated,
        };
    }
    (shape, elapsed / frames)
}

/// **What the frame costs a component**, drawn through [`Direct`] rather than through a [`Tally`].
///
/// The one number on this page that is comparable with the original's 63.87 µs, and it is separate from
/// [`volume_over`]'s for a reason worth the second function: a `Tally` keeps a `BTreeSet` of every
/// cell it sees, so a figure taken with one in the loop is **a report about the instrument**. The
/// two run the identical [`draw_into`], which is [`crate::ink`]'s whole argument — a gate written
/// against a copy of the code tests the copy.
///
/// A report and never a gate.
///
/// # Panics
///
/// [`volume_over`]'s, unchanged.
///
/// [`Direct`]: crate::ink::Direct
pub fn volume_cost(kind: Volume, rows: u64, frames: u32) -> Duration {
    assert!(frames > 0, "a per-frame figure needs a frame");
    let mut driver = crate::runner::driver_at(W, H, Density::default());
    let mut ink = Direct;
    driver.frame(|cx| {
        let _ = draw_into(&mut ink, cx, kind, rows, 0);
    });
    let started = Instant::now();
    for _ in 0..frames {
        driver.frame(|cx| {
            let _ = draw_into(&mut ink, cx, kind, rows, 0);
        });
    }
    started.elapsed() / frames
}

/// **The shape at every one of [`VOLUMES`]**, which is what the equality is asserted over.
pub fn across_volumes(kind: Volume) -> Vec<(u64, Shape)> {
    VOLUMES.into_iter().map(|n| (n, volume(kind, n))).collect()
}

// ── the equality scenes, over the runner ──────────────────────────────────────

/// The listing's content at `rows` rows: [`W`] by [`H`], one [`ROW`] a row.
///
/// [`crate::runner::Fixture::lines`] is deliberately not used. Its rows are a per-column generator
/// so that *a wrong row costs exactly `w` cells*, which is what makes the scrolled and stale-tail
/// numbers read as `m * w`; this listing's rows are the screen's, and the two are used for the two
/// different questions below.
pub fn content(rows: usize) -> Fixture {
    Fixture::of(W, H, vec![String::from(ROW); rows])
}

/// The fixture the two reproduced axes are measured on.
///
/// [`crate::runner::Fixture::lines`] at [`W`] by [`H`]: no two rows share a column, so a wrong row
/// costs exactly forty cells and *n cells over m rows* reads as `m * 40`. **That is the property
/// the 71-of-80 is stated in**, and changing the fixture changes the number without changing the
/// defect.
pub fn lines(rows: usize) -> Fixture {
    Fixture::lines(W, H, rows)
}

/// **The scrolled collection, as an equality against a reference render.** Criterion 2.
///
/// Not a count: the inverted sign draws nothing and **every counter approves of it**, which is
/// [`counters_approve`]'s half of the same scene. The equality is the only detector, and it is
/// the `reference` — one cell at a time, no runs, no offset it did not compute
/// itself.
pub fn scrolled() -> Diff {
    let fx = lines(VOLUMES[0] as usize).scrolled_to(SCROLLED_TO);
    compare(reference, defective::inverted_scroll, &[fx])
}

/// **Every counter this crate can read, over the two arms of [`scrolled`].**
///
/// Returns `(correct, defective)`. The scene exists because the pair is *indistinguishable*: a
/// report that printed only the diff would leave the reader to take on trust that the defective
/// build looked healthier, and this is the half that stops that being trust.
///
/// # The allocation totals are the caller's, and that is not a hole
///
/// [`crate::runner::Run::counters`]'s own arrangement, for its reason: `vitui-alloc-probe` is a
/// dev-dependency and a library cannot install a global allocator on a consumer's behalf. There is
/// no default, because *a figure defaulted to zero is a counter that prints `0` when it means nobody
/// counted*. A caller with no probe hands both arms the same value, which makes that column inert in
/// the comparison rather than false; `examples/listing_numbers.rs` has the probe and hands in two
/// measured ones.
pub fn counters_approve(correct: Allocations, defective: Allocations) -> (Counters, Counters) {
    let fx = lines(VOLUMES[0] as usize).scrolled_to(SCROLLED_TO);
    (
        play(rows_at_a_time, std::slice::from_ref(&fx)).counters(correct),
        play(defective::inverted_scroll, &[fx]).counters(defective),
    )
}

/// **Which of the nine counters tell the two arms of [`scrolled`] apart. None of them.**
///
/// Criterion 2's argument as a list rather than as a sentence: an empty answer means *the equality
/// against a reference render is the only detector there is*, and a non-empty one would mean a
/// cheaper gate exists and this scene is optional.
///
/// A counter that is [`Reading::Unreachable`] on both arms — `marked`, and it will stay that way —
/// contributes nothing either way and is skipped rather than silently counted as agreement. A
/// counter reachable on one arm and not the other would be a defect in the instrument and is
/// reported as a separation.
///
/// [`Reading::Unreachable`]: crate::counters::Reading::Unreachable
pub fn counters_that_separate_them(correct: Allocations, defective: Allocations) -> Vec<Counter> {
    let (a, b) = counters_approve(correct, defective);
    Counter::ALL
        .into_iter()
        .filter(|c| {
            let (left, right) = (a.get(*c).measured(), b.get(*c).measured());
            left.is_some() != right.is_some() || (left.is_some() && left != right)
        })
        .collect()
}

/// **The collection shorter than its viewport, asserted on the rendered surface.** Criterion 3.
///
/// The spelling of the shrink axis — *content shrinking inside a rectangle that does not
/// move* — and the rectangle is [`W`] by [`H`] on both steps. The version written against a terminal
/// resize passes on all twelve panels, because a fresh `Surface` has nowhere for the residue to
/// survive; [`stale_by_resize`] is that spelling, kept beside this one so the difference is a number.
pub fn stale() -> Diff {
    let full = lines(FULL_ROWS);
    let shrunk = full.shrunk_to(SHRUNK_TO);
    compare(reference, defective::stale_tail, &[full, shrunk])
}

/// **The same defect, spelled as a terminal resize — and it scores clean.**
///
/// The shrink gate written this way is refused, and this is the refusal as a value rather
/// than as a sentence.
pub fn stale_by_resize() -> Diff {
    compare(
        reference,
        defective::stale_tail,
        &[Fixture::lines(W, SHRUNK_TO as u16, SHRUNK_TO).resized(W, SHRUNK_TO as u16)],
    )
}

/// **The narrow collection: every row truncates, and the marker is one cell.** Criterion 1's fourth
/// axis column.
///
/// Green at [`W`] and red at [`NARROW`], which is the shape the overlap has and the one register
/// row 43 gates. The two painters below are the arms.
pub fn narrow(width: u16) -> Diff {
    let fx = content(H as usize).resized(width, H);
    compare(narrowed, does_not_narrow, &[fx])
}

/// **The correct arm: every row goes through [`crate::text::fit`].**
///
/// `fit` is where truncation is decided, so it is where the one-cell rule holds: `elide` reserves
/// exactly one cell for the marker and `Theme::glyph` guarantees every spelling of it is one cell
/// wide.
pub fn narrowed(pen: &mut Pen, cx: &mut Ctx<'_, '_>, fx: &Fixture) {
    let (w, h) = fx.size();
    let opts = FitOpts::default();
    for y in 0..h {
        fit_into(
            pen,
            cx,
            Rect::new(0, i32::from(y), w, 1),
            row_line(fx, y),
            &opts,
        );
    }
}

/// **The untruncated text of screen row `y`**, which is what makes the narrow scene a scene.
///
/// [`crate::runner::Fixture::row_text`] truncates to the fixture's own width before it returns —
/// correct for the runner's per-cell oracle, and fatal here: a row already cut to sixteen columns
/// has nothing left for `fit` to elide, so both arms would write sixteen letters and the equality
/// would report **0 cells over 0 rows** on the axis it exists to catch. Watched doing exactly that.
///
/// The listing's rows are all [`ROW`] by construction, so what the fixture is asked for is the one
/// thing it can answer without cutting: *is there a row here at all*.
fn row_line(fx: &Fixture, y: u16) -> &'static str {
    if fx.offset() + usize::from(y) < fx.len() {
        ROW
    } else {
        ""
    }
}

/// **The defective arm: the row is written whole and the clip decides where it stops.**
///
/// Identical to [`narrowed`] at [`W`], where nothing truncates. At [`NARROW`] the last cell of every
/// row carries a letter where the marker belongs, and the caller's own overrun is visible to the
/// engine as a short report — `asked - reported`, which is `crate::gates::REGISTER`'s row 40.
pub fn does_not_narrow(pen: &mut Pen, cx: &mut Ctx<'_, '_>, fx: &Fixture) {
    let body = cx.theme().paint(Role::Body);
    let (w, h) = fx.size();
    for y in 0..h {
        let written = pen.text(cx, 0, i32::from(y), row_line(fx, y), body);
        if written < w {
            let pad = " ".repeat(usize::from(w - written));
            pen.text(cx, i32::from(written), i32::from(y), &pad, body);
        }
    }
}

// ── the subject, and the scan that says whether it is here ───────────────────────────────────────

/// **The component these five scenes are scenes of, and it is not declared yet.**
///
/// One subject and not four, which is [`Verdict::of`]'s vacuity refusal doing its job on a
/// population of one: *no component exists* is `Unmet` over one rather than `Met` over nothing.
pub const SUBJECTS: [&str; 1] = ["collection"];

/// Where [`SUBJECTS`] is declared, as `(module file, the declaration)`.
///
/// The home is the freeze's, joined through [`crate::Family`]: `collection`'s first family is
/// `F7Collections`, whose module is `collect.rs`. A component is `fn(&mut Ctx, Rect, …) -> Response`
/// with [`Rect`] in `Rect`'s place, so the thing to look for is a public function
/// of the component's own name in its own family's module.
pub const DECLARATIONS: [(&str, &str); 1] = [("collect.rs", "pub fn collection(")];

/// **Which of [`SUBJECTS`] this crate actually declares. Today: none.**
///
/// A source scan and not a `use`, for [`crate::dense::subjects_declared`]'s reason: *the item does
/// not exist* has no expression, and a `compile_fail` fence would pass today and pass again the day
/// somebody renames the module. The predicate is `crate::dense::declares` and it is shared rather
/// than copied — **one definition of *a line that is not a comment***, which is the only thing that
/// makes *fires in both directions* mean anything.
pub fn subjects_declared() -> Vec<&'static str> {
    let mut out = Vec::new();
    for (subject, (file, declaration)) in SUBJECTS.into_iter().zip(DECLARATIONS) {
        let path = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).join(file);
        let source = std::fs::read_to_string(&path).unwrap_or_default();
        if crate::dense::declares(&source, declaration) {
            out.push(subject);
        }
    }
    out
}

/// **Whether the listing stands on its subject, as a verdict rather than as a sentence.**
///
/// **`Met` over one since components 12**, which declared `collection` in
/// `crates/vitui-components/src/collect.rs` and rewrote [`draw_into`] to draw through it. It was
/// `Unmet { over: 1, failing: 1 }` for exactly one ticket, and the failing sentence below is kept
/// live rather than deleted — [`owed_message`] builds it over any declaration list, so the
/// distinction criterion 7 is about survives the crate arriving on the other side of it.
pub fn standing() -> Verdict {
    let declared = subjects_declared();
    Verdict::of(
        SUBJECTS.len(),
        SUBJECTS.len() - declared.len(),
        "`collection` is not declared in this crate, so what stands on the listing is a stand-in \
         row loop and not the component. The screen, its 81 regions, its equality against a \
         reference render at three offsets, its stale tail at 71 of 80 rows and its identical \
         writes and regions at 1k / 100k / 1M are measured and green; what is missing is the \
         subject",
    )
}

/// **The sentence a scene waiting for its subject fails with**, or `None` once it is declared.
///
/// Separated from [`assert_stands_up`] because the message is the mechanism and the panic is only
/// how it is delivered — [`crate::dense::owed_message`]'s arrangement, and it takes the declaration
/// list as an argument for the same reason: the day the crate is on the other side of it, the
/// hostile case is still one call away.
///
/// # This is criterion 7, and it is the distinction the whole ticket rests on
///
/// A scene that fails because it is unimplemented and a scene that fails because the code is wrong
/// are **the same failure** unless the message separates them. This one names the subject, the file
/// it belongs in, the declaration to look for, and the ticket — and says in as many words that it is
/// not a defect in the screen.
pub fn owed_message(declared: &[&str], scene: &str) -> Option<String> {
    if declared.len() == SUBJECTS.len() {
        return None;
    }
    let owed: Vec<String> = SUBJECTS
        .into_iter()
        .zip(DECLARATIONS)
        .filter(|(id, _)| !declared.contains(id))
        .map(|(id, (file, declaration))| format!("`{id}` (`src/{file}`: `{declaration}…)`)"))
        .collect();
    Some(format!(
        "{scene} is not standing, and it is waiting for its subject rather than failing: {} of {} \
         components are undeclared — {}. This is not a defect in the screen. The listing is drawn, \
         its regions are counted, its equality against a reference render holds at three offsets, \
         its stale tail is 71 of 80 rows and its writes and regions are identical at 1 000, \
         100 000 and 1 000 000 — see `crate::listing::tests`. Inverted by `components 12`",
        owed.len(),
        SUBJECTS.len(),
        owed.join(", "),
    ))
}

/// **Fail with the subject that is missing, the file it belongs in, and the ticket.**
///
/// # Panics
///
/// Panics while [`SUBJECTS`] is undeclared, which is **today**.
pub fn assert_stands_up(scene: &str) {
    if let Some(message) = owed_message(&subjects_declared(), scene) {
        panic!("{message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vitui_runtime::layout::text::width;

    /// **Criterion 5: identical writes and identical regions at 1 000, 100 000 and 1 000 000.**
    ///
    /// The equality is over the whole [`Shape`] except the one field that is *allowed* to move —
    /// there is none — because a gate that compared writes alone would be green on the arm below.
    /// The timing is taken and thrown away here; `examples/listing_numbers.rs` prints it, and
    /// rule is why: **a timing is a report**.
    #[test]
    fn the_listing_writes_and_declares_the_same_at_a_thousand_rows_and_at_a_million() {
        let across = across_volumes(Volume::Windowed);
        assert_eq!(across.len(), 3);
        let first = across[0].1;
        assert_eq!(first.writes, WRITES, "every cell of the viewport, once");
        assert_eq!(first.regions, REGIONS, "one collection and eighty rows");
        assert_eq!(
            first.stops, STOPS,
            "a virtualised collection is one tab stop"
        );
        assert_eq!(first.verbs, VERBS * 2, "a row and its trailing pad");
        assert_eq!(first.distinct, WRITES, "and it is a partition");
        assert_eq!(first.iterated, u64::from(H));
        for (rows, shape) in &across {
            assert_eq!(
                *shape, first,
                "the listing at {rows} rows is not the listing at {} rows",
                across[0].0
            );
        }
    }

    /// **The other direction, and it is where criterion 6's `regions` earns its place.**
    ///
    /// A listing that iterates its whole content writes **exactly what the windowed one writes** —
    /// the engine reports a fully clipped verb as zero columns — so *writes
    /// flat 1k -> 1M*, is green on it. What is not green is the hit index: `Ctx::declare` does not
    /// clip, so a row declared off screen is a real entry, and the count is the content's.
    #[test]
    fn a_listing_that_iterates_its_whole_content_writes_the_same_and_declares_a_thousand_times_more()
     {
        let windowed = volume(Volume::Windowed, VOLUMES[0]);
        let whole = volume(Volume::WholeContent, VOLUMES[0]);

        assert_eq!(
            whole.writes, windowed.writes,
            "the write count cannot see this defect, which is why the equality is on `regions`"
        );
        assert_eq!(
            whole.distinct, windowed.distinct,
            "and neither can `distinct`"
        );
        assert_eq!(whole.stops, windowed.stops, "and neither can the ring");

        assert_eq!(whole.iterated, VOLUMES[0], "it iterated the content");
        assert_eq!(whole.regions, VOLUMES[0] as usize + 1);
        assert_eq!(windowed.regions, REGIONS);
        assert!(whole.regions > windowed.regions * 12);

        // And the ratio grows with the data, which is the mechanism rather than the instance.
        let bigger = volume(Volume::WholeContent, VOLUMES[1]);
        assert_eq!(bigger.regions, VOLUMES[1] as usize + 1);
        assert_eq!(bigger.writes, windowed.writes);
        assert_eq!(
            bigger.regions - whole.regions,
            (VOLUMES[1] - VOLUMES[0]) as usize,
            "the index grows by exactly the rows that were added"
        );
    }

    /// **Criterion 2: the scrolled collection is caught by an equality and by nothing else.**
    ///
    /// Both halves, because the contrast is the scene's entire argument. The equality refuses —
    /// **seventy-five of eighty rows** carry a different row — and **not one of the nine counters
    /// separates the arms**: the defective build makes the same number of calls, writes the same
    /// number of columns and touches the same cells, so there is no counting gate anywhere in the
    /// stack that could have been on the wrong side of it. The one that would have caught it is
    /// `marked`, and it is `Unreachable` from this crate and will stay that way.
    #[test]
    fn the_scrolled_listing_is_refused_by_the_equality_and_approved_by_every_counter() {
        let diff = scrolled();
        assert_eq!(
            (diff.cells, diff.rows),
            (SCROLLED_CELLS, SCROLLED_ROWS),
            "seventy-five of eighty rows carry a different row, and the five that do not are the \
             fixture's generator repeating every twenty-sixth row: {diff}"
        );
        assert_eq!(
            H as usize - diff.rows,
            SCROLLED_ALIASED,
            "the equality under-reports this defect by five rows and is still the only instrument \
             that reports it"
        );
        assert_eq!(diff.first, Some((0, 1)), "row 0 is right by coincidence");
        assert!(!diff.clean());

        // **Every one of the nine, and not a chosen three.** The allocation column is inert here
        // by construction — the same figure on both arms — because this binary installs no probe;
        // `examples/listing_numbers.rs` does and hands in two measured ones.
        let inert = Allocations::over(1, 0);
        assert_eq!(
            counters_that_separate_them(inert, inert),
            Vec::<Counter>::new(),
            "a counter told the two arms apart, which would make the equality optional. Criterion \
             2's whole argument is that none of them can"
        );
        let (correct, broken) = counters_approve(inert, inert);
        assert_eq!(
            correct.get(Counter::Writes).measured(),
            Some(WRITES),
            "and both wrote the whole viewport rather than agreeing by drawing nothing"
        );
        assert_eq!(broken.get(Counter::Writes).measured(), Some(WRITES));
        assert_eq!(
            correct.get(Counter::Marked).measured(),
            None,
            "`marked` is the counter that would have caught it, and it is unreachable from here"
        );
        assert_eq!(correct.reachable(), 8);

        // The correct painter over the same offset is clean, which is the other direction: the
        // equality is not simply reporting that two things are two things.
        let fx = lines(VOLUMES[0] as usize).scrolled_to(SCROLLED_TO);
        assert!(compare(reference, rows_at_a_time, &[fx]).clean());
    }

    /// **Criterion 3: the stale tail is 71 of 80 rows, asserted on the rendered surface.**
    ///
    /// The number, reached through the instrument. The resize
    /// spelling beside it scores the same painter clean, which is the reason for refusing to
    /// bank that gate.
    #[test]
    fn the_stale_tail_is_seventy_one_of_eighty_rows_and_the_resize_spelling_misses_it() {
        let diff = stale();
        assert_eq!(diff.rows, STALE_ROWS, "§21's own number: {diff}");
        assert_eq!(diff.cells, STALE_CELLS, "2 840 cells over 71 rows");
        assert_eq!(diff.first, Some((0, SHRUNK_TO as u16)));

        // **The defective build looks healthier**, measured on the frame the defect is in.
        let shrunk = lines(FULL_ROWS).shrunk_to(SHRUNK_TO);
        let correct = play(rows_at_a_time, std::slice::from_ref(&shrunk));
        let broken = play(defective::stale_tail, std::slice::from_ref(&shrunk));
        assert_eq!(
            (correct.tally().writes(), broken.tally().writes()),
            (WRITES, SHRUNK_TO as u64 * W as u64)
        );
        assert!(broken.tally().writes() * 8 < correct.tally().writes());

        assert!(
            stale_by_resize().clean(),
            "the resize spelling scores the same defect clean, which is why §21 keeps both"
        );
    }

    /// **Criterion 1's fourth axis column: the narrow listing, green wide and red narrow.**
    ///
    /// The narrow size is a different construction and not a clipped one, and the cell it differs in
    /// is the last of every row: `fit` puts the one-cell marker there and the arm that writes its
    /// row whole puts a letter there.
    #[test]
    fn a_listing_row_narrows_through_fit_and_the_ellipsis_is_one_cell() {
        assert!(
            width(ROW) > NARROW && width(ROW) < W,
            "the row has to truncate at one size and fit at the other or the scene asks nothing"
        );

        let wide = narrow(W);
        assert!(wide.clean(), "nothing truncates at {W} columns: {wide}");

        let tight = narrow(NARROW);
        assert_eq!(
            (tight.cells, tight.rows),
            (H as usize, H as usize),
            "one cell a row — the last one: {tight}"
        );
        assert_eq!(tight.first, Some((NARROW - 1, 0)));

        // And the marker really is one cell: what `fit` wrote in the last column is the theme's
        // ellipsis, and it is one column wide.
        let run = play(narrowed, &[content(H as usize).resized(NARROW, H)]);
        let last = run
            .canvas()
            .get(NARROW - 1, 0)
            .expect("the row is a partition of its width");
        assert_eq!(width(&last.cluster), 1, "§16's one-cell ellipsis rule");
        let head = vitui_runtime::layout::text::truncate(ROW, NARROW);
        assert!(
            !head.ends_with(&last.cluster),
            "the last cell carries the marker and not the row's own {}th character, which is what \
             the defective arm puts there",
            NARROW
        );
        // The other arm, on the same cell, so this is a comparison rather than an inspection.
        let broken = play(does_not_narrow, &[content(H as usize).resized(NARROW, H)]);
        let their_last = broken
            .canvas()
            .get(NARROW - 1, 0)
            .expect("the clipped row still writes its last column");
        assert_eq!(
            their_last.cluster,
            head[head.len() - 1..],
            "a row written whole and clipped ends in a letter"
        );
        assert_eq!(
            run.canvas().written(),
            usize::from(NARROW) * usize::from(H),
            "and it is still a partition"
        );
    }

    /// **The listing stands on `collection`, and the verdict says so.**
    ///
    /// The exact set, in both directions: one subject, declared, and the verdict is `Met` over one
    /// rather than `Met` over nothing — [`Verdict::of`]'s vacuity refusal doing the job it was
    /// written for on the day the population stopped being empty.
    ///
    /// **This test is the inversion of `the_listing_is_red_because_collection_is_not_declared`**,
    /// which was pinned red and which `crate::gates::REGISTER` and four of
    /// `crate::scenes`' five standings named. Changing it back is a deliberate edit in all three
    /// places — and the fifth standing, the wheel gate's, was **never** one of them: it was red
    /// because of the *defect*, it stayed red for eight tickets after this one went green, and
    /// components 20 stood it up in [`crate::wheel`].
    #[test]
    fn the_listing_stands_on_the_collection_it_is_a_screen_of() {
        assert_eq!(
            subjects_declared(),
            SUBJECTS.to_vec(),
            "`collection` has stopped being declared where the freeze homes it. That is not a \
             defect in the screen: `crate::listing::DECLARATIONS` names the file and the signature \
             it is looked for at"
        );
        let verdict = standing();
        assert!(verdict.met());
        match verdict {
            Verdict::Met { over } => assert_eq!(over, 1, "one subject, and it is here"),
            Verdict::Unmet { over, failing, .. } => {
                unreachable!("{failing} of {over} undeclared, which the assertion above caught")
            }
        }
        // And the scenes do not panic any more, which is the whole ticket in one call.
        assert_stands_up("a scrolled collection");

        // **Both arms of `draw_into` are the component**, which is what makes the four equalities
        // above claims about `collection` rather than about a stand-in row loop. Read out of the
        // source, because *this function calls that one* has no expression a test can write.
        let source = include_str!("listing.rs");
        for call in ["collection_into(ink", "coll_defective::whole_content(ink"] {
            assert!(
                crate::dense::declares(source, call),
                "`draw_into` no longer calls `{call}`, so the scenes have stopped standing on the \
                 component even though the subject scan still finds it"
            );
        }
    }

    /// **The waiting message says which failure it is**, which is the distinction criterion 7 is
    /// about.
    ///
    /// Fired in both directions over a declaration list rather than over the crate, so that the day
    /// `collection` is declared this half is still live — `crate::dense::owed_message`'s arrangement
    /// and its reason: a message no test can read is a message that rots.
    #[test]
    fn the_waiting_message_separates_unimplemented_from_wrong() {
        let message = owed_message(&[], "a scrolled collection").expect("no subject is declared");
        assert!(
            message.contains("waiting for its subject rather than failing"),
            "{message}"
        );
        assert!(
            message.contains("This is not a defect in the screen"),
            "{message}"
        );
        assert!(message.contains("components 12"), "{message}");
        assert!(message.contains("src/collect.rs"), "{message}");
        assert!(message.contains("pub fn collection("), "{message}");
        assert!(message.contains("71 of 80"), "{message}");

        // The other direction: with the subject declared there is no message at all.
        assert_eq!(owed_message(&SUBJECTS, "a scrolled collection"), None);
    }

    /// **The subject scan finds a declaration when there is one.**
    ///
    /// The half that makes [`subjects_declared`] a scan rather than a `false` — the same
    /// both-directions rule `crate::dense` applies to its four, over the one predicate both share.
    #[test]
    fn the_subject_scan_finds_a_declaration_when_there_is_one() {
        for (_, declaration) in DECLARATIONS {
            assert!(crate::dense::declares(
                &format!("// a comment\n{declaration}cx: &mut Ctx) {{}}\n"),
                declaration
            ));
            assert!(
                !crate::dense::declares(
                    &format!("// {declaration}…) is components 12's\n"),
                    declaration
                ),
                "a mention in a comment is not a declaration"
            );
        }
        // And the file the freeze homes it in exists, so a rename is a failing test here rather
        // than a scan that quietly answers `false` for ever.
        for (file, _) in DECLARATIONS {
            let path =
                std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).join(file);
            assert!(path.is_file(), "{}", path.display());
        }
    }

    /// **The freeze homes `collection` where [`DECLARATIONS`] looks for it.**
    ///
    /// The join is `crate::Family::members`, and asserting it here is what stops the scan pointing
    /// at a file the inventory has stopped naming.
    #[test]
    fn the_freeze_homes_collection_in_the_file_the_scan_opens() {
        let component = crate::INVENTORY
            .iter()
            .find(|c| c.id == SUBJECTS[0])
            .expect("`collection` is in the freeze");
        let home = component.families[0];
        assert_eq!(home.module(), Some("collect"));
        assert!(home.members().contains(&SUBJECTS[0]));
        assert_eq!(DECLARATIONS[0].0, "collect.rs");

        // Three of the four axes are literally this component's defects and the fourth is not:
        // a row truncates through `text::fit`, which is `text`'s flag. The narrow scene therefore
        // *stands* `collection` up and claims no `(collection, narrow)` pair — see `crate::scenes`.
        assert!(component.declares(crate::Axis::Scrolled));
        assert!(component.declares(crate::Axis::Shrunk));
        assert!(component.declares(crate::Axis::Wheeled));
        assert!(!component.declares(crate::Axis::Narrow));
    }
}
