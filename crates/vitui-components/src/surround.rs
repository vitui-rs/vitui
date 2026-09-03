//! **What a scroll area writes and its body does not: the tail past the content, and the bands
//! beside it.**
//!
//! Production ticket 09. Spec §9, §17 (O5), §21. This is the screen `scroll_area`'s shrink axis and
//! `sticky`'s scroll axis are scenes *of* — [`crate::area`] carries this family's other four, §21's
//! own scenes 17, 18, 19 and 30, and the two files divide the way the questions do: that one is
//! **how big the parts are** and this one is **what is in them**.
//!
//! | scene | what it decides |
//! |---|---|
//! | an extent shrunk to [`SHORT`] from content row [`FAR`] | the tail, on both of its axes |
//! | five bands over a body at [`FAR`] | the shared axis, and the four ways it is got wrong |
//!
//! # Both are one question and it is *who owes this cell*
//!
//! Spec §2 gives every component every cell of its rectangle, and `scroll_area` is the one place on
//! this map where the rectangle is **partitioned between a component and its caller**: the body gets
//! [`Parts::view`](crate::scroll::Parts::view), and the area keeps the two bars, the corner, the
//! four bands and the tail. So both axes here are the same sentence asked twice —
//!
//! > The range from the extent to the end of the viewport is inside the rectangle, and the component
//! > that owns the rectangle must write it. … A band is a rectangle split that shares one of the two
//! > offsets and pins the other to zero, and it must be a view.
//!
//! — and both are decided by **the one offset the area owns**: the tail begins at
//! `[extent, offset + viewport)`, and a band's translation is
//! [`Shares::of`](crate::scroll::Shares::of) of that same pair. Production 05 put three scenes in
//! one module because one integer decided all three; this is that, at two.
//!
//! # Neither may be played at offset zero, and the existing gates both are
//!
//! *At offset 0 every sign agrees* is the runtime's own finding about this axis (architecture 26),
//! and it is the reason this module exists rather than a criterion it satisfies. Both properties
//! already had an instrument before this ticket and **neither could fail on its own axis**:
//!
//! - `scroll::tests::a_shrunk_extent_leaves_no_unwritten_tail` asks *is every cell of the viewport
//!   written* on **one** frame, over four extents, with no refusal beside it. §17's `shrunk` axis is
//!   *content shrinking inside a rectangle that does not move* — a defect of the second frame given
//!   the first — so a one-frame gate over a state nothing has moved is the trap
//!   `.scratch/vitui-production/README.md` records production 04 finding twice.
//! - `scroll::tests::a_band_is_a_view_and_the_arithmetic_spelling_re_damages_what_is_under_it` is
//!   played at offset **`(0, 0)`**, where four of `crate::scroll::Shared`'s five arms draw the same
//!   screen. It catches the fifth, which is the one it was written for; the scroll axis was
//!   invisible to it.
//!
//! [`AT_REST`] is the control that states the second of those as a number: at `(0, 0)` every
//! refusal here is clean.
//!
//! # Both oracles are the same cut taken twice, and the band's is on the band
//!
//! The tail scene's reference is an area **built over the short content from its first frame**; the
//! band scene's is an area at the **origin whose bands carry the subject's marks** — because a band
//! drawer is handed the offset by its *caller*, so a reference can be drawn at zero while writing
//! the marks of content column [`FAR`]`.0`. Either way the window arithmetic is `0 + i` on the
//! reference side and `offset + i` on the subject's, which is [`crate::window`]'s own oracle and
//! its reason: **a defect in that arithmetic cannot hide in both**.
//!
//! **What the band oracle cannot share is the body and the two thumbs**, and that is a *partition*
//! rather than a judgement: §9's parts tile the rectangle exactly (ADR 0029), so the five bands are
//! disjoint from both. [`body_and_bars`] is the excluded number, measured rather than assumed —
//! where [`crate::dropped::window_interior`] had to cut a column out of the middle of its subject
//! to reach the same position.
//!
//! **Neither oracle can see [`Refused::AnArithmeticBand`]**, whose picture is identical by
//! construction; that arm's detector is the damage ledger and it has had one since components 19.
//! Named beside the other three, because a band gate that plays the translation has not played the
//! clip.
//!
//! # Neither offset answers both halves of the band scene, and that is the measurement
//!
//! A translation wrong by at least the band's own width puts every cell outside the clip and a
//! clipped write lands nothing — so at [`FAR`] all three refusals leave the **header** empty, and
//! its three rows are one empty string. At [`NEAR`] the wrong translation lands *inside* the clip
//! and the three carry three different strings: shifted left by one, blank, and shifted right by
//! twice the offset.
//!
//! And the **counts** separate neither pair: a band wrong by any amount the rota cannot alias is
//! wrong in every cell it has, so [`MISALIGNED_FAR`]'s first and third entries are equal at both
//! offsets. What the counts do say is what only they can — [`GUTTER_CELLS`], the twelve cells that
//! separate the unpinned arm, because a gutter shares **neither** offset and it is the one arm that
//! translates one anyway.
//!
//! So the scene is played at two offsets and stated in three readings, and **each is the half the
//! others cannot see**: *the band is blank* is a class of wrongness a shifted-marks gate never
//! produces, *shifted by twice the offset* is a mechanism a blank band hides, and *the gutter
//! moved* is invisible to both.

use vitui_runtime::{Ctx, Density, Rect, Role};

use crate::counters::{Allocations, Counter, Counters};
use crate::ink::Ink;
use crate::runner::{Canvas, Diff, Pen, driver_at};
use crate::scroll::{
    AreaOpts, AreaShape, AreaState, Band, Hide, Shared, Shares, Tail, defective as refused, parts,
};

// ── the screen ───────────────────────────────────────────────────────────────────────────────────

/// The rig's width. Sixty, which is [`crate::dropped::W`]'s and wide enough for a pinned column
/// beside a body.
pub const W: u16 = 60;

/// The rig's height. Twenty-four, [`crate::dropped::H`]'s, so a header and a footer leave a body
/// with rows to spare.
pub const H: u16 = 24;

/// **The rig, as the pair every screen here is played in.** [`W`] by [`H`].
///
/// A **parameter** of this module's drive loop rather than a constant inside it, which is what lets
/// [`stale_by_resize`] be *the same function* at the short content's own size: §21's refused
/// spelling is the shrink played into a rectangle that has already been resized, and two drive
/// loops differing only in a rig are two places for one to be got wrong.
/// `crate::dropped::picker_screen_in` is that duplication one family over, and a review of this
/// ticket's first draft found this file had copied it.
pub const RIG: (u16, u16) = (W, H);

/// **The content both scenes are played over, in content cells on both axes.**
///
/// Larger than the rig on both axes by a wide margin, so [`FAR`] is admissible under it and neither
/// scene's offset is a clamp reported as a position.
pub const LONG: (u32, u32) = (200, 400);

/// **What the extent shrinks to: nine cells on both axes.**
///
/// [`crate::dropped::SHRUNK_TO`]'s number rather than a second one, and **smaller than the viewport
/// on both axes**, which is the only shape a tail can exist in at all: `max_offset` is
/// `extent − viewport`, so at any offset the clamp admits, `offset + viewport ≤ extent` and the
/// range `[extent, offset + viewport)` is empty. A vertical tail therefore requires a content
/// shorter than the viewport, and a horizontal one a content narrower than it.
///
/// **That is a property of the arithmetic and not a weakness of the fixture**, and it is what makes
/// the scene's offset reading interesting: the shrink *clamps the offset to zero*, so the window
/// moves at the same time as the content stops reaching it. See [`SHRUNK_FROM`].
pub const SHORT: (u32, u32) = (9, 9);

/// **The offset both scenes are played at: forty columns and a hundred rows.**
///
/// Non-zero on **both** axes and the two are different, which is one requirement more than
/// non-zero: [`Refused::ATransposedBand`] reads the other field of the pair, and at
/// `offset.0 == offset.1` it draws the correct screen.
pub const FAR: (i32, i32) = (40, 100);

/// **The band scene's second offset: two columns and three rows.**
///
/// Small enough that a translation wrong by [`FAR`] would land outside the clip and one wrong by
/// this lands **inside it, carrying the wrong marks** — which is what separates three refusals that
/// are one number at [`FAR`]. See this module's header.
pub const NEAR: (i32, i32) = (2, 3);

/// **The control: at the origin every refusal draws the correct screen.**
///
/// Not a third arm of the scene but the sentence *a scrolled scene may not be played near the top*
/// as a measurement. `tests::at_the_origin_every_band_refusal_is_clean` is where it is read.
pub const AT_REST: (i32, i32) = (0, 0);

/// How many rows of sticky header the band scene cuts. One.
pub const HEADER: u16 = 1;

/// How many rows of sticky footer. One.
pub const FOOTER: u16 = 1;

/// **How many columns of pinned column. Six**, wide enough for a row mark and a separator and
/// narrow enough to leave the body most of the rig.
pub const PINNED: u16 = 6;

// ── the ledger ───────────────────────────────────────────────────────────────────────────────────

/// **The viewport the tail scene's area hands its body.** Both bars reserved, no bands.
#[must_use]
pub fn tail_view() -> Rect {
    parts(Rect::new(0, 0, W, H), LONG, &tail_opts()).view
}

/// **What the two-axis shrink leaves standing: every row of the viewport, and 1 276 of its cells.**
///
/// `(cells, rows)`, and the row count being **all** of them is the statement rather than a
/// shortcoming of it: a content that has stopped reaching the viewport on both axes leaves nothing
/// on the screen right. The cell count is the informative half here, which inverts
/// [`crate::grid::STALE_CELLS`]'s reading one family over — there a row is mostly pad and the cells
/// undercount the damage; here the body writes every cell it is given, so the count is exact.
pub const STALE: (usize, usize) = (1_276, 23);

/// **What the vertical half of the shrink leaves standing on its own: 826 cells over 14 rows.**
///
/// The rows the content no longer reaches at all. The difference from [`STALE`] is
/// [`tail_into`](crate::scroll)'s **inner** loop — 450 cells in the 9 rows the content still
/// reaches, `[extent.0, offset.0 + view.w)` of each — and it is reported because a gate written on
/// rows alone would score the horizontal half at 9 rows of 23 and a gate written on the two-axis
/// number alone would never say the two loops are two properties.
pub const STALE_DOWN: (usize, usize) = (826, 14);

/// **Where the shrink leaves the offset: the origin**, from [`FAR`].
///
/// Read off the component rather than asserted about the fixture: `max_offset` is recomputed every
/// frame and [`SHORT`] admits no offset at all, so the window moves to `(0, 0)` on the frame the
/// content shrinks. **That is why the scene is played from [`FAR`] and not from the origin** — at
/// the origin the window does not move and the tail is the only thing that changes, which is a
/// weaker screen wearing the same row count.
pub const SHRUNK_FROM: (i32, i32) = FAR;

/// **What the omitted tail costs on the counters that move: 2 880 writes against 1 604, and 111
/// verbs against 88.**
///
/// `(rule, refused)` each, and the direction is the axis's whole argument — **the refusal is
/// cheaper on both and nothing rises.** Cumulative over the two frames, so the rule's 2 880 is two
/// full viewports of 1 440 and the refusal's 1 604 is one of them plus the eighty-one cells the
/// short content still reaches, plus its own eighty-three-cell first-frame difference.
///
/// **`distinct` is 1 440 on both arms** and it is the counter a reader reaches for first: it is
/// cumulative and the first frame already touched every cell the viewport has, which is
/// [`crate::dropped::STALE_WRITES`]' own finding one family over.
pub const STALE_WRITES: (u64, u64) = (2_880, 1_604);
/// See [`STALE_WRITES`].
pub const STALE_VERBS: (u64, u64) = (111, 88);

/// **What `distinct` says about the omitted tail: 1 440 on both arms.** See [`STALE_WRITES`].
pub const STALE_DISTINCT: u64 = 1_440;

/// **How wrong each band refusal is at [`FAR`]: `(cells, rows)` over the five band rectangles**, in
/// [`BANDS`]' order.
///
/// The header and the footer are 53 cells each and the pinned column 126, on **all three** — a
/// band's translation wrong by any amount the rota cannot alias puts every cell of it wrong, and
/// the rota is one cell longer than the rig for exactly that reason. So the count says *the band is
/// wrong* and never *how*.
///
/// **What separates the middle arm is the two gutters, and nothing else on the screen does.** A
/// gutter shares **neither** offset, so `Shares::Neither` maps to a literal zero under the rule,
/// under the transposition and under the inversion alike — only the arm that ignores
/// `Shares` entirely translates one. That is [`GUTTER_CELLS`], and it is
/// exactly the difference between this array's second entry and its first.
pub const MISALIGNED_FAR: [(usize, usize); 3] = [(232, 23), (244, 25), (232, 23)];

/// **How wrong each band refusal is at [`NEAR`]**, in [`MISALIGNED_FAR`]'s order.
///
/// Two of the three are the same pair at both offsets and **the middle one is not**: at a
/// horizontal offset of two the unpinned pinned column's writes land four columns of six inside
/// its own clip, so the band is wrong in 42 cells rather than 126 and the total is 160 against 244.
/// **A single-offset gate would have reported a magnitude that is a property of the fixture**,
/// which is why the scene is played twice and why neither reading is the one it is stated on.
pub const MISALIGNED_NEAR: [(usize, usize); 3] = [(232, 23), (160, 25), (232, 23)];

/// **What the two gutters cost the unpinned band: twelve cells over two rows.**
///
/// `PINNED * 2` — six cells each — and it is the whole of what separates
/// [`Refused::AnUnpinnedBand`] from the other two on this screen. See [`MISALIGNED_FAR`].
pub const GUTTER_CELLS: usize = PINNED as usize * 2;

/// **What the band oracle leaves out: 1 137 cells over 22 rows at [`FAR`].**
///
/// The body and the two thumbs, which the reference render cannot share because it is drawn at the
/// origin. Reported rather than excluded silently, for [`crate::dropped::BAR_CELLS`]' reason — and
/// it is a **partition** rather than a judgement: §9's parts tile the rectangle exactly (ADR 0029),
/// so not one of these cells is inside a band.
pub const BODY_AND_BARS: (usize, usize) = (1_137, 22);

/// **What a band refusal costs on the counters: 1 440 writes and 1 440 distinct against 1 208, with
/// the verbs identical at 294.**
///
/// `(rule, refused)`, at [`FAR`]. **The refusal is cheaper again** — a clipped write lands nothing,
/// so a band drawn outside its own band is a band that costs less — but unlike the tail this axis
/// *is* visible to a counter, and that is worth stating rather than glossing: `writes` and
/// `distinct` both move. What no counter says is **which** cells, and at [`NEAR`] the same refusal
/// is 1 396 against 1 440 — a 3% difference on a screen that is wrong in 232 of its 244 band cells.
pub const BAND_WRITES: (u64, u64) = (1_440, 1_208);
/// See [`BAND_WRITES`]. The same pair at [`NEAR`], where the difference is 3%.
pub const BAND_WRITES_NEAR: (u64, u64) = (1_440, 1_396);
/// **What the verbs say: nothing — 294 at [`FAR`] and 293 at [`NEAR`], identical on both arms of
/// each.**
///
/// Two numbers because the *rule's* own count moves with the offset and not because the refusal
/// does: at [`NEAR`] the vertical thumb sits against the top of its track, so `bar` writes no
/// stripe above it — spec §3's *thumb, then the track above and below*, with one of the two empty.
/// A reader could take 294 for a property of the screen; it is a property of where the thumb is.
pub const BAND_VERBS: (u64, u64) = (294, 293);

// ── the fixture ──────────────────────────────────────────────────────────────────────────────────

/// **The column marks, one per content column, distinct over any [`W`] consecutive columns.**
///
/// A rota of length 61 — one more than the rig — so that no two columns of one screen carry the same
/// mark and a band off by `k` is a different string for every `k` the screen can hold. A rota of
/// length 10 would make an off-by-ten band identical to the rule.
const COLUMN_ROTA: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-";

/// **The row marks**, [`COLUMN_ROTA`]'s rule on the other axis and a **different** alphabet, so
/// that a band reading the wrong axis of the offset is a different *alphabet* and not only a
/// different index.
const ROW_ROTA: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ9876543210.";

/// The mark content column `c` carries.
fn column_mark(c: i32) -> char {
    // **`expect` and not `unwrap_or(0)`**: `rem_euclid` of a positive modulus is non-negative, so
    // the fallback is unreachable — and a fallback that silently answers *the mark of column zero*
    // would make a wrong index look like a correct mark, which is the one thing this fixture exists
    // to make impossible.
    let i = usize::try_from(c.rem_euclid(COLUMN_ROTA.len() as i32)).expect("non-negative");
    char::from(COLUMN_ROTA[i])
}

/// The mark content row `r` carries.
fn row_mark(r: i32) -> char {
    let i = usize::try_from(r.rem_euclid(ROW_ROTA.len() as i32)).expect("non-negative");
    char::from(ROW_ROTA[i])
}

/// The options the tail scene's area is drawn with. **No bands**, so the shrink is the only thing
/// on the screen.
fn tail_opts() -> AreaOpts {
    AreaOpts {
        hide: Hide::Never,
        ..AreaOpts::default()
    }
}

/// The options the band scene's area is drawn with. All four of §9's bands stand.
fn band_opts() -> AreaOpts {
    AreaOpts {
        hide: Hide::Never,
        header: HEADER,
        footer: FOOTER,
        pinned: PINNED,
        ..AreaOpts::default()
    }
}

/// **The body both scenes draw**, at content coordinates, virtualised and bounded by the extent.
///
/// One row a verb, and **every cell carries the mark of its own content column with the row's mark
/// in front of it** — so a cell says which content coordinate it came from on both axes, and a
/// window off by one on either is a different string. That is what the tail scene's equality is
/// made over; the band scene crops it away.
///
/// Bounded by the **extent** and not by the viewport, which is a body's whole contract here: the
/// cells past it are the area's tail and writing them from inside the body would write them twice.
fn body_into<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, extent: (u32, u32)) {
    let paint = cx.theme().paint(Role::Body);
    let rows = cx.visible_rows();
    let cols = cx.visible_cols();
    let last_row = i32::try_from(extent.1).unwrap_or(i32::MAX);
    let last_col = i32::try_from(extent.0).unwrap_or(i32::MAX);
    let right = cols.end.min(last_col);
    if right <= cols.start {
        return;
    }
    for r in rows.start..rows.end.min(last_row) {
        let mut line = String::new();
        line.push(row_mark(r));
        for c in cols.start + 1..right {
            line.push(column_mark(c));
        }
        let _ = ink.text(cx, cols.start, r, &line, paint);
    }
}

/// **The band drawer**: it writes at content coordinate `place` the marks of content coordinate
/// `mark`.
///
/// Two coordinates and not one, and that is the whole of what makes the oracle one — see
/// [`band_screen`]. On every shipped path they are the same pair; the reference render passes the
/// origin and the subject's marks.
///
/// The offset is the **caller's**, which is what a band drawer is handed in every application on
/// this map: `Ctx::visible_rows` is the scroll scope's and a band is a `child` plus a `scrolled`,
/// so the range a band covers is arithmetic the caller already owns. See
/// [`crate::scroll::sticky`]'s own documentation for the coordinates.
///
/// # One cell a verb, and that is a finding rather than a style
///
/// [`Pen`] records a verb at the column it was **asked** for plus the context's origin, and the
/// engine reports how many columns *landed* — so a run that starts left of its clip is recorded
/// **shifted by the discarded prefix**, which is `Pen`'s own documented caveat (ADR 0022's
/// clamp-and-discard) and `Tally`'s. Every refusal here is a band drawing outside its own band, so
/// the caveat is not hypothetical: written as one run a row, a transposed header at [`NEAR`]
/// reported **one cell in the gutter beside it** — a cell the screen does not have, produced by the
/// recorder. Per cell the write either lands whole or is discarded whole, which is
/// [`crate::runner::reference`]'s own arrangement and its reason.
fn band_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    b: Band,
    place: (i32, i32),
    mark: (i32, i32),
) {
    let paint = cx.theme().paint(Role::Title);
    match b.shares() {
        Shares::X => {
            for i in 0..i32::from(b.rect.w) {
                let glyph = column_mark(mark.0 + i);
                let _ = ink.text(cx, place.0 + i, 0, &glyph.to_string(), paint);
            }
        }
        Shares::Y => {
            for i in 0..i32::from(b.rect.h) {
                let glyph = row_mark(mark.1 + i);
                for x in 0..i32::from(b.rect.w) {
                    let _ = ink.text(cx, x, place.1 + i, &glyph.to_string(), paint);
                }
            }
        }
        // **A gutter shares neither**, so it carries neither mark: the same glyph on both arms is
        // what says its cells are the ones no translation reaches.
        Shares::Neither => {
            for i in 0..i32::from(b.rect.h) {
                for x in 0..i32::from(b.rect.w) {
                    let _ = ink.text(cx, x, i, "+", paint);
                }
            }
        }
    }
}

// ── scene 43: the tail ───────────────────────────────────────────────────────────────────────────

/// **A scroll area over `steps` extents, one a frame, into one surface that is never cleared.**
///
/// [`crate::runner::play`]'s arrangement and its reason: a cell nobody wrote keeps what was already
/// there, and a runner starting each frame from a blank surface scores the stale tail clean. The
/// offset is carried across the frames by the component's own state, which is what makes
/// [`SHRUNK_FROM`] a reading rather than a parameter.
///
/// **`rig` is a parameter and it is why there is one drive loop here and not two**: [`RIG`] on the
/// scene's own arms and the short content's own size on [`stale_by_resize`]'s, which is §21's
/// refused spelling and the one thing it says may stand *beside* the shrink rather than instead of
/// it. See [`RIG`].
fn tail_screen(
    steps: &[(u32, u32)],
    from: (i32, i32),
    shape: AreaShape,
    rig: (u16, u16),
) -> (Canvas, AreaState) {
    let (w, h) = rig;
    let mut driver = driver_at(w, h, Density::default());
    let mut st = AreaState { offset: from };
    let opts = tail_opts();
    let mut canvas = Canvas::new(w, h);
    for extent in steps.iter().copied() {
        let mut pen = Pen::over(canvas);
        let st = &mut st;
        driver.frame(|cx| {
            let id = cx.id();
            let _ = refused::area_shaped(
                &mut pen,
                cx,
                id,
                Rect::new(0, 0, w, h),
                st,
                &opts,
                extent,
                |_ink, _cx, _b| {},
                |ink, cx| body_into(ink, cx, extent),
                shape,
            );
        });
        pen.end_frame();
        canvas = pen.into_canvas();
    }
    (canvas, st)
}

/// **The correct build against an area that never held the long content.** Clean, and it is the arm
/// that says the oracle is an oracle.
///
/// A comparison whose correct side has never been watched agreeing reports *0 cells over 0 rows* for
/// the same reason a broken one would — `crate::runner::defective`'s own sentence.
#[must_use]
pub fn shrunk_at_rest() -> Diff {
    against_the_short_content(Refused::Nothing, SHORT)
}

/// **The tail omitted, on both of its axes.** [`STALE`].
#[must_use]
pub fn shrunk() -> Diff {
    against_the_short_content(Refused::TheTail, SHORT)
}

/// **The tail omitted with only the vertical axis shrunk.** [`STALE_DOWN`], and the difference from
/// [`shrunk`] is `tail_into`'s inner loop.
#[must_use]
pub fn shrunk_down() -> Diff {
    against_the_short_content(Refused::TheTail, (LONG.0, SHORT.1))
}

/// One arm of the tail scene, against an area built over the short content from its first frame.
///
/// **The reference is played from the same [`FAR`]** and not from the origin: its own clamp puts it
/// wherever the shrunk extent admits, which is the offset the subject arrives at. Handing the
/// reference a zero would make the two arms differ about the offset as well as about the tail.
fn against_the_short_content(refused: Refused, short: (u32, u32)) -> Diff {
    let (subject, _) = tail_screen(&[LONG, short], FAR, refused.shape(), RIG);
    let (reference, _) = tail_screen(&[short, short], FAR, AreaShape::RULE, RIG);
    let r = tail_view();
    subject.cropped(r).diff(&reference.cropped(r))
}

/// **The spelling §21 refuses, kept as a number rather than as a sentence**: the same refusal played
/// into a rectangle the shrink has already resized.
///
/// §21's own correction to this axis is that *the version written against a terminal resize passes*
/// — a fresh rectangle has nowhere for the residue to survive. Here the rig is the short content's
/// own size on both frames, so the refusal draws every cell the rectangle has and there is nothing
/// left for a residue to sit in.
#[must_use]
pub fn stale_by_resize() -> Diff {
    let rig = (SHORT.0 as u16 + 1, SHORT.1 as u16 + 1);
    let subject = tail_screen(&[SHORT, SHORT], AT_REST, Refused::TheTail.shape(), rig).0;
    let reference = tail_screen(&[SHORT, SHORT], AT_REST, AreaShape::RULE, rig).0;
    subject.diff(&reference)
}

/// **Where the shrink left the offset.** [`SHRUNK_FROM`]'s counterpart, read off the component.
#[must_use]
pub fn offset_after_the_shrink() -> (i32, i32) {
    tail_screen(&[LONG, SHORT], FAR, AreaShape::RULE, RIG)
        .1
        .offset
}

// ── scene 44: the bands ──────────────────────────────────────────────────────────────────────────

/// **How far the bands and the reference render disagree**, over the five band rectangles.
///
/// A pair and not a [`Diff`], because the five bands are five rectangles and [`Canvas::diff`] is
/// one comparison over one surface: the crop is applied five times and the readings are added.
/// That is the one comparison this crate has, used five times — not a second answer to *do these
/// two screens agree*.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Misaligned {
    /// How many cells of the five band rectangles differ.
    pub cells: usize,
    /// How many rows of them carry at least one.
    pub rows: usize,
}

impl Misaligned {
    /// The pair, so a scene can be stated in `(cells, rows)` like every other one on this map.
    #[must_use]
    pub const fn pair(self) -> (usize, usize) {
        (self.cells, self.rows)
    }

    /// Whether the bands agree with the reference everywhere.
    #[must_use]
    pub const fn clean(self) -> bool {
        self.cells == 0
    }
}

/// **One frame of the band scene: an area at `place`, whose bands carry the marks of content
/// `mark`.**
///
/// The two arguments are what make the oracle one. A band drawer is handed the offset by its
/// **caller** — `Ctx::visible_rows` is the scroll scope's and a band is a `child` plus a
/// `scrolled`, so the range a band covers is arithmetic the caller already owns — so a reference
/// render can be drawn at the origin while writing the marks of content column [`FAR`]`.0`. The
/// rule places content column `mark.0` at the band's own first cell and so does the reference at
/// the origin, and **a defect in that arithmetic cannot hide in both**: it is
/// [`crate::window::tail`]'s oracle, one family over, with the cut on the *band* instead of on the
/// document.
fn band_screen(place: (i32, i32), mark: (i32, i32), shape: AreaShape) -> Canvas {
    let mut driver = driver_at(W, H, Density::default());
    let mut st = AreaState { offset: place };
    let opts = band_opts();
    let mut pen = Pen::new(W, H);
    let st = &mut st;
    driver.frame(|cx| {
        let id = cx.id();
        let _ = refused::area_shaped(
            &mut pen,
            cx,
            id,
            Rect::new(0, 0, W, H),
            st,
            &opts,
            LONG,
            |ink, cx, b| band_into(ink, cx, b, place, mark),
            |ink, cx| body_into(ink, cx, LONG),
            shape,
        );
    });
    pen.end_frame();
    pen.into_canvas()
}

/// **The five band rectangles at this rig, in draw order.**
#[must_use]
pub fn band_rects() -> Vec<Rect> {
    parts(Rect::new(0, 0, W, H), LONG, &band_opts())
        .bands()
        .map(|b| b.rect)
        .collect()
}

/// **The bands of an area at [`FAR`] against the bands of one at the origin carrying the same
/// marks.**
///
/// `over` is what the equality is made on: the five band rectangles for the scene's own numbers,
/// `None` for the whole rig — which is [`BODY_AND_BARS`], reported rather than assumed for
/// [`crate::dropped::at_rest_whole`]'s reason.
#[must_use]
pub fn aligned(offset: (i32, i32), refused: Refused) -> Misaligned {
    let subject = band_screen(offset, offset, refused.shape());
    let reference = band_screen(AT_REST, offset, AreaShape::RULE);
    let mut cells = 0usize;
    let mut rows = 0usize;
    for r in band_rects() {
        let diff = subject.cropped(r).diff(&reference.cropped(r));
        cells += diff.cells;
        rows += diff.rows;
    }
    Misaligned { cells, rows }
}

/// **What the band oracle cannot share, and it is the body and the two bars.**
///
/// The reference is drawn at the origin, so its body draws content `[0, view)` where the subject's
/// draws content `[offset, offset + view)`, and its thumbs sit at the start of their tracks. Both
/// are outside the five band rectangles by construction — §9's parts tile the rectangle exactly
/// (ADR 0029) — which is why the exclusion here is a *partition* and not a judgement, where
/// [`crate::dropped::window_interior`]'s had to cut a column out of the middle of its own subject.
///
/// It is measured rather than argued: a reader told a region was left out is owed the number.
#[must_use]
pub fn body_and_bars(offset: (i32, i32)) -> Diff {
    let subject = band_screen(offset, offset, AreaShape::RULE);
    let reference = band_screen(AT_REST, offset, AreaShape::RULE);
    subject.diff(&reference)
}

/// **What this screen refuses, in the screen's own vocabulary.**
///
/// [`crate::grid::TailShape`]'s arrangement and its recorded reason rather than a copy of
/// `crate::scroll`'s policy value: *the component's is crate-private and this module's is a
/// screen's vocabulary, spelled the way §21's rows spell it.* The mapping is `Refused::shape` and
/// it is the one place the two vocabularies meet.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refused {
    /// **The rule.** The shipped `scroll_area`.
    Nothing,
    /// **§17's `shrunk` axis**: the cells past the extent left alone.
    TheTail,
    /// **§17's `scrolled` axis**: a band whose shared axis is the other one.
    ATransposedBand,
    /// A band that pins neither axis, so both of its coordinates translate.
    AnUnpinnedBand,
    /// A band whose translation carries §17's inverted sign.
    AnInvertedBand,
    /// A band that moves its origin by arithmetic instead of narrowing a clip. **The picture is
    /// identical**, which is why it is not one of [`BANDS`].
    AnArithmeticBand,
}

impl Refused {
    /// The component's own policy value. The one place the screen's vocabulary meets it.
    fn shape(self) -> AreaShape {
        let band = |band| AreaShape {
            band,
            ..AreaShape::RULE
        };
        match self {
            Refused::Nothing => AreaShape::RULE,
            Refused::TheTail => AreaShape {
                tail: Tail::Omitted,
                ..AreaShape::RULE
            },
            Refused::ATransposedBand => band(Shared::Transposed),
            Refused::AnUnpinnedBand => band(Shared::Both),
            Refused::AnInvertedBand => band(Shared::Inverted),
            Refused::AnArithmeticBand => band(Shared::Arithmetic),
        }
    }

    /// The word a report prints.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Refused::Nothing => "nothing",
            Refused::TheTail => "the tail",
            Refused::ATransposedBand => "a transposed band",
            Refused::AnUnpinnedBand => "an unpinned band",
            Refused::AnInvertedBand => "an inverted band",
            Refused::AnArithmeticBand => "an arithmetic band",
        }
    }
}

/// **The three band refusals the equality can see, in the order [`MISALIGNED_FAR`] states them.**
///
/// [`Refused::AnArithmeticBand`] is the fourth and it is not here: its picture is identical by
/// construction, so its detector is the damage ledger. Named beside the three rather than folded
/// in with them — see this module's header.
pub const BANDS: [Refused; 3] = [
    Refused::ATransposedBand,
    Refused::AnUnpinnedBand,
    Refused::AnInvertedBand,
];

// ── what the counters say about both ─────────────────────────────────────────────────────────────

/// **Which scene a counter comparison is played on.**
///
/// A named argument and never a `match` on the refusal, which is [`crate::window::On`]'s refusal and
/// its reason: read that way, a refusal the dispatch did not enumerate falls into the other scene's
/// play silently.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum On {
    /// Scene 43: the extent shrunk to [`SHORT`] over two frames.
    TheTail,
    /// Scene 44: five bands at one offset, one frame.
    TheBands {
        /// Which offset. **A field and not two variants**, because the counters' answer is
        /// different at [`FAR`] and at [`NEAR`] and that difference is the reading.
        at: (i32, i32),
    },
}

/// **Every counter this crate can read, over the rule and over the refusal.** Returns
/// `(rule, refused)`.
///
/// The scenes exist because the pairs are *indistinguishable*, and a report printing only the diff
/// would leave a reader to take on trust that the refused build looked healthier.
#[must_use]
pub fn counters_approve(on: On, allocations: Allocations) -> (Counters, Counters) {
    match on {
        On::TheTail => (
            tail_counters(AreaShape::RULE, allocations),
            tail_counters(Refused::TheTail.shape(), allocations),
        ),
        On::TheBands { at } => (
            band_counters(at, AreaShape::RULE, allocations),
            band_counters(at, Refused::AnInvertedBand.shape(), allocations),
        ),
    }
}

/// **Which of §20's nine counters tell a refused build from the rule.**
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

/// The tail scene's counters over one shape, across the shrink.
fn tail_counters(shape: AreaShape, allocations: Allocations) -> Counters {
    let mut driver = driver_at(W, H, Density::default());
    let mut st = AreaState { offset: FAR };
    let opts = tail_opts();
    let mut pen = Pen::new(W, H);
    for extent in [LONG, SHORT] {
        let st = &mut st;
        driver.frame(|cx| {
            let id = cx.id();
            let _ = refused::area_shaped(
                &mut pen,
                cx,
                id,
                Rect::new(0, 0, W, H),
                st,
                &opts,
                extent,
                |_ink, _cx, _b| {},
                |ink, cx| body_into(ink, cx, extent),
                shape,
            );
        });
    }
    Counters::of(&driver, pen.tally(), allocations)
}

/// The band scene's counters over one shape, on the one frame it is played on.
fn band_counters(at: (i32, i32), shape: AreaShape, allocations: Allocations) -> Counters {
    let mut driver = driver_at(W, H, Density::default());
    let mut st = AreaState { offset: at };
    let opts = band_opts();
    let mut pen = Pen::new(W, H);
    let st = &mut st;
    driver.frame(|cx| {
        let id = cx.id();
        let _ = refused::area_shaped(
            &mut pen,
            cx,
            id,
            Rect::new(0, 0, W, H),
            st,
            &opts,
            LONG,
            |ink, cx, b| band_into(ink, cx, b, at, at),
            |ink, cx| body_into(ink, cx, LONG),
            shape,
        );
    });
    Counters::of(&driver, pen.tally(), allocations)
}

// ── the subjects, and the scan that says whether they are here ───────────────────────────────────

/// **What the two screens in this module are screens of, between them.** Two, and **no scene claims
/// both**.
///
/// [`crate::dropped::SUBJECTS`]' arrangement and its reason: `crate::scenes::Scene::stands` is *the
/// components that are on the screen*, so a list of both on both rows inflates criterion 2's
/// enumeration in both directions — the *generous join* the field's own documentation forbids by
/// name.
pub const SUBJECTS: &[&str] = &["scroll_area", "sticky"];

/// **What scene 43 stands up: a `scroll_area`, and no band is on that screen** — the tail scene's
/// options cut none.
pub const TAIL_SCREEN: &[&str] = &["scroll_area"];

/// **What scene 44 stands up: both**, because a band exists only inside the area that cuts it.
///
/// The one row of the two that names two components, and it is not a generous join: §9's rule is
/// that *`scroll_area` calls `sticky` for all four of its bands and nothing else in this crate opens
/// a band of its own*, so a `sticky` screen **is** an area screen. The pair it claims for O5 is
/// `sticky`'s alone — see `crate::scenes`.
pub const BAND_SCREEN: &[&str] = &["scroll_area", "sticky"];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Axis, INVENTORY};

    /// **The oracle is an oracle: an area built over the short content agrees with one that shrank
    /// to it, everywhere inside the viewport.**
    ///
    /// And the offset is read off the component beside it, because [`SHRUNK_FROM`]'s whole point is
    /// that the window moves at the same time as the content stops reaching it.
    #[test]
    fn the_correct_area_agrees_with_one_built_over_the_short_content() {
        shrunk_at_rest().assert_clean("the shrunk area against one built over the short content");
        assert_eq!(
            offset_after_the_shrink(),
            AT_REST,
            "the shrink clamps the offset to the origin, which is what makes the window move at \
             the same time as the tail appears — a scene played from the origin has only the tail"
        );
        assert_ne!(
            SHRUNK_FROM, AT_REST,
            "a shrink scene played at the origin is played where every sign agrees"
        );
    }

    /// **Scene 43: the tail omitted is [`STALE`], the vertical half of it is [`STALE_DOWN`], and
    /// the resize spelling scores it clean.**
    #[test]
    fn the_omitted_tail_is_every_row_of_the_viewport_and_the_resize_spelling_misses_it() {
        let diff = shrunk();
        assert_eq!((diff.cells, diff.rows), STALE, "the tail on both axes");

        let down = shrunk_down();
        assert_eq!(
            (down.cells, down.rows),
            STALE_DOWN,
            "the tail's outer loop alone: the rows the content no longer reaches"
        );
        assert!(
            down.cells < diff.cells && down.rows < diff.rows,
            "the horizontal half of the tail costs {} cells in {} rows more, and it is \
             `tail_into`'s inner loop — a gate on either number alone cannot say the two loops are \
             two properties",
            diff.cells - down.cells,
            diff.rows - down.rows
        );

        // **§21's refused spelling, as a number.** Played into a rectangle the shrink has already
        // resized, the same refusal is clean.
        stale_by_resize().assert_clean(
            "the resize spelling scores the omitted tail clean, which is why §21 refuses it",
        );
    }

    /// **The omitted tail is cheaper on both counters that move and nothing rises**, which is what
    /// makes the scene the only detector there is.
    #[test]
    fn the_omitted_tail_is_cheaper_on_the_counters_that_move() {
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
             in the stack disapproves of a screen that is wrong in every row it has"
        );
        // **`distinct` cannot separate them**, which is the counter a reader reaches for first: it
        // is cumulative and the first frame already touched every cell the viewport has.
        assert_eq!(
            (
                rule.get(Counter::Distinct).measured(),
                bad.get(Counter::Distinct).measured()
            ),
            (Some(STALE_DISTINCT), Some(STALE_DISTINCT)),
            "`distinct` is cumulative and the first frame touched every cell, so the pair a \
             reader would reach for is blind here"
        );
        assert_eq!(
            counters_that_separate(On::TheTail, Allocations::over(2, 0)),
            vec![Counter::Writes, Counter::Verbs],
            "and those two are the whole list — both in the refusal's favour"
        );
    }

    /// **Scene 44: a band and the reference agree about where a content coordinate is, and every
    /// translation refusal breaks it.**
    ///
    /// > That is what makes a title and the cells under it unable to disagree about where a column
    /// > is: they are the same number. (§9, on [`crate::scroll::sticky`])
    #[test]
    fn the_bands_agree_with_the_reference_and_every_translation_refusal_disagrees() {
        assert!(
            aligned(FAR, Refused::Nothing).clean(),
            "the shipped bands disagree with an area at the origin carrying the same marks, so \
             the oracle is about the fixture: {:?}",
            aligned(FAR, Refused::Nothing)
        );

        let far: Vec<(usize, usize)> = BANDS
            .into_iter()
            .map(|band| aligned(FAR, band).pair())
            .collect();
        assert_eq!(far, MISALIGNED_FAR.to_vec());

        // **The two gutters are the whole of what separates the unpinned arm**, and the arithmetic
        // says so rather than a comment: a gutter shares neither offset, so three of the five arms
        // map it to a literal zero and one does not.
        assert_eq!(
            far[1].0 - far[0].0,
            GUTTER_CELLS,
            "the unpinned band's extra cells are the two gutters and nothing else"
        );
        assert_eq!(far[1].1 - far[0].1, 2, "and they are two rows");

        // **The excluded region is measured**, so a reader told the body and the bars were left
        // out is owed the number.
        let outside = body_and_bars(FAR);
        assert_eq!((outside.cells, outside.rows), BODY_AND_BARS);
        let (fx, fy) = outside.first.expect("the excluded region is not empty");
        let (fx, fy) = (i32::from(fx), i32::from(fy));
        for rect in band_rects() {
            let inside = fx >= rect.x && fx < rect.right() && fy >= rect.y && fy < rect.bottom();
            assert!(
                !inside,
                "the excluded region's first cell is inside a band, so the exclusion is not the \
                 partition it claims to be"
            );
        }
    }

    /// **The counts say a band is wrong and never how, the strings say which, and neither offset
    /// answers both halves.**
    ///
    /// A transposed band and an inverted one report the **same pair** at both offsets: a
    /// translation wrong by any amount the rota cannot alias puts every cell of the band wrong, and
    /// the rota is one cell longer than the rig so that nothing aliases. So the second half of this
    /// scene is a distinctness assertion over the drawn header row — and it is asked at [`NEAR`],
    /// because at [`FAR`] all three refusals draw **nothing at all** and `row_text` answers one
    /// empty string for the three of them.
    ///
    /// **Both readings are asserted**, because each is the half the other cannot see: *the band is
    /// blank* is a whole class of wrongness a shifted-marks gate never produces, and *the marks are
    /// shifted by twice the offset* is a mechanism a blank band hides.
    #[test]
    fn the_three_refusals_are_two_numbers_and_three_strings_and_neither_offset_says_both() {
        assert_eq!(
            MISALIGNED_FAR[0], MISALIGNED_FAR[2],
            "the transposed and the inverted band are one number, which is what the strings below \
             exist to separate"
        );

        let header = band_rects()
            .into_iter()
            .find(|r| r.h == 1 && r.w > PINNED)
            .expect("the header is a one-row band wider than the pinned column");
        let row = |offset, refused: Refused| {
            band_screen(offset, offset, refused.shape())
                .cropped(header)
                .row_text(0)
        };

        // **At `FAR` the three are one string and it is empty.** A translation wrong by more than
        // the band's own width puts every cell outside the clip, and a clipped write lands nothing.
        let far: std::collections::BTreeSet<String> =
            BANDS.into_iter().map(|band| row(FAR, band)).collect();
        assert_eq!(
            far,
            std::collections::BTreeSet::from([String::new()]),
            "a refusal drew something at {FAR:?}, so the *blank band* class this offset exists to \
             produce is not what is being measured"
        );

        // **At `NEAR` they are three**, and none of them is the rule's.
        let near: Vec<String> = BANDS.into_iter().map(|band| row(NEAR, band)).collect();
        let distinct: std::collections::BTreeSet<&String> = near.iter().collect();
        assert_eq!(
            distinct.len(),
            3,
            "two refusals drew the same header row, so this screen cannot tell the two mechanisms \
             apart at all: {near:?}"
        );
        let correct = row(NEAR, Refused::Nothing);
        assert!(
            !distinct.contains(&correct),
            "a refusal drew the correct header row, which would make its own pair a coincidence"
        );
        assert!(
            near[1].is_empty() && !near[0].is_empty() && !near[2].is_empty(),
            "the unpinned band is the one that is still blank at {NEAR:?} — it translates the \
             *pinned* axis of a one-row band, and three rows is more than one: {near:?}"
        );
    }

    /// **The same refusal is 244 cells wrong at one offset and 160 at another**, which is why the
    /// scene is played twice and why neither reading is the one it is stated on.
    ///
    /// At [`NEAR`] the unpinned pinned column's writes land four columns of [`PINNED`] inside its
    /// own clip, so the band is wrong in 42 cells rather than 126. **A single-offset gate would
    /// have reported a magnitude that is a property of the fixture.**
    #[test]
    fn the_unpinned_band_is_a_different_number_at_a_different_offset() {
        assert!(
            aligned(NEAR, Refused::Nothing).clean(),
            "the rule is clean at both offsets or the comparison is about the fixture"
        );
        let near: Vec<(usize, usize)> = BANDS
            .into_iter()
            .map(|band| aligned(NEAR, band).pair())
            .collect();
        assert_eq!(near, MISALIGNED_NEAR.to_vec());
        assert_ne!(
            near[1], MISALIGNED_FAR[1],
            "the unpinned band is the one arm whose magnitude moves with the offset"
        );
        assert_eq!(
            (near[0], near[2]),
            (MISALIGNED_FAR[0], MISALIGNED_FAR[2]),
            "and the other two do not, because a band wrong by more than nothing is wrong in \
             every cell it has"
        );
    }

    /// **A band refusal is visible to two counters and the tail is not**, which is worth stating
    /// rather than glossing.
    ///
    /// A clipped write lands nothing, so a band drawn outside its own band **costs less** — the
    /// refused build looks healthier here as it does on every one of these axes, and unlike
    /// [`On::TheTail`] the difference is one a counter can see. What no counter says is *which*
    /// cells: at [`NEAR`] the same refusal is 1 396 against 1 440, a 3% reading on a screen that is
    /// wrong in 232 of its band cells.
    #[test]
    fn a_band_refusal_is_cheaper_and_two_counters_can_see_it() {
        for (on, expected, verbs) in [
            (On::TheBands { at: FAR }, BAND_WRITES, BAND_VERBS.0),
            (On::TheBands { at: NEAR }, BAND_WRITES_NEAR, BAND_VERBS.1),
        ] {
            let (rule, bad) = counters_approve(on, Allocations::over(1, 0));
            assert_eq!(
                (
                    rule.get(Counter::Writes).measured(),
                    bad.get(Counter::Writes).measured()
                ),
                (Some(expected.0), Some(expected.1)),
                "{on:?}"
            );
            assert!(expected.1 < expected.0, "the refusal is cheaper: {on:?}");
            assert_eq!(
                (
                    rule.get(Counter::Verbs).measured(),
                    bad.get(Counter::Verbs).measured()
                ),
                (Some(verbs), Some(verbs)),
                "the verbs are identical, which is what makes `writes` a difference in what \
                 *landed* rather than in what was asked for: {on:?}"
            );
            assert_eq!(
                counters_that_separate(on, Allocations::over(1, 0)),
                vec![Counter::Writes, Counter::Distinct],
                "{on:?}"
            );
        }
        assert!(
            BAND_WRITES.0 - BAND_WRITES.1 > (BAND_WRITES_NEAR.0 - BAND_WRITES_NEAR.1) * 4,
            "the counter's reading at the two offsets differs by more than fourfold on one \
             refusal, so the number is the fixture's and the equality is the gate"
        );
    }

    /// **The control, and it is the reason this module exists rather than a criterion it
    /// satisfies**: at the origin every band refusal draws the correct screen.
    #[test]
    fn at_the_origin_every_band_refusal_is_clean() {
        for band in BANDS {
            assert!(
                aligned(AT_REST, band).clean(),
                "the `{}` band is wrong at {FAR:?} and right at the origin, which is the \
                 runtime's *at offset 0 every sign agrees* — and the gate this crate had for this \
                 component was played there",
                band.word()
            );
        }
        // **The control is the origin, asserted where it can fail**: not `AT_REST == (0, 0)`,
        // which is the constant's own literal, but that the *runtime's* own clamp puts an area
        // handed the origin there — which is what makes the arm a control and not a coincidence.
        assert_eq!(
            crate::scroll::max_offset(SHORT, (tail_view().w, tail_view().h)),
            (0, 0),
            "the shrunk content admits no offset at all, which is why the control and the clamp \
             are the same point"
        );
    }

    /// **The fourth refusal is the clip and no equality can see it**, which is why it is named here
    /// rather than measured here.
    ///
    /// `crate::scroll`'s arithmetic arm draws an identical picture by construction — the overrun lands on the
    /// neighbour, which draws afterwards and wins — so the detector is the damage ledger and
    /// `crate::scroll::tests::a_band_is_a_view_and_the_arithmetic_spelling_re_damages_what_is_\
    /// under_it` is it. Reached rather than re-driven, for `crate::dropped::wheeled_by_a_copy`'s
    /// reason: that gate has existed since components 19 and a second drive loop beside it would be
    /// a second answer to one question.
    #[test]
    fn the_arithmetic_band_is_a_fourth_refusal_and_the_equality_is_blind_to_it() {
        assert!(
            aligned(FAR, Refused::AnArithmeticBand).clean(),
            "the arithmetic band draws the correct screen, which is the whole of why it survived: \
             a band gate that plays the translation has not played the clip"
        );
        // **A membership test and not `BANDS.len() == 3`**, which is a `[Refused; 3]` and cannot
        // fail without a compile error — the trap `CLAUDE.md` names, found by a review of this
        // ticket's first draft. What can fail is the fourth arm *joining* the three, which is the
        // edit this assertion exists to stop: an arm whose picture is identical by construction
        // would report `(0, 0)` beside three real numbers and read as a passing equality.
        assert!(
            !BANDS.contains(&Refused::AnArithmeticBand),
            "the arithmetic band is in the equality's population, and it draws the correct screen \
             — so it would report clean beside three real numbers and read as a pass"
        );
    }

    /// **Criterion: the freeze declares the axis for every pair this module claims.**
    ///
    /// `crate::obligations` asserts the other direction. The two are not the same question and
    /// neither implies the other.
    #[test]
    fn the_freeze_declares_both_axes_this_module_claims() {
        for (id, axis) in [("scroll_area", Axis::Shrunk), ("sticky", Axis::Scrolled)] {
            let component = INVENTORY
                .iter()
                .find(|c| c.id == id)
                .unwrap_or_else(|| panic!("`{id}` is in the freeze"));
            assert!(
                component.declares(axis),
                "this module claims `{id}` is {} and the freeze does not",
                axis.name()
            );
            assert!(
                component.built,
                "`{id}` is played through the shipped component, so the freeze has to say it is \
                 built"
            );
        }

        // **And `sticky` declares one axis and only one**, which is §9's own rule rather than an
        // omission: a band shares one of the two offsets and pins the other, so there is no second
        // axis for it to be wrong on, and it declares nothing — so no notch reaches it and its
        // extent is its caller's.
        let sticky = INVENTORY.iter().find(|c| c.id == "sticky").expect("frozen");
        assert!(!sticky.declares(Axis::Shrunk));
        assert!(!sticky.declares(Axis::Wheeled));
        assert!(!sticky.declares(Axis::Narrow));
    }

    /// **The two screens stand on declared components**, which is criterion 8's verdict from the
    /// direction that can go quietly wrong.
    ///
    /// [`crate::area::subjects_declared`] is reached rather than copied — a second scan written here
    /// would be a second answer to *is `sticky` declared*, and `crate::window`'s and
    /// `crate::dropped`'s rows use the same arrangement.
    #[test]
    fn the_two_screens_stand_on_declared_components() {
        let declared = crate::area::subjects_declared();
        for id in SUBJECTS {
            assert!(
                declared.contains(id),
                "`{id}` is not declared where the freeze homes it, so these scenes are waiting \
                 for their subject rather than measuring one"
            );
        }
        // **The two `stands` lists are joined against the options that decide them**, which is
        // what makes this a gate rather than a literal compared with itself: `Scene::stands` is
        // *the components that are on the screen*, and whether a `sticky` is on one is decided by
        // whether the area cuts a band. A review of this ticket's first draft found
        // `assert_eq!(TAIL_SCREEN, &["scroll_area"])` here — the constant's own literal.
        for (screen, opts) in [(TAIL_SCREEN, tail_opts()), (BAND_SCREEN, band_opts())] {
            let cuts_a_band = opts.header > 0 || opts.footer > 0 || opts.pinned > 0;
            assert_eq!(
                screen.contains(&"sticky"),
                cuts_a_band,
                "a scene claims `sticky` and its area cuts no band, or cuts one and does not \
                 claim it — `{screen:?}` against header {}, footer {}, pinned {}",
                opts.header,
                opts.footer,
                opts.pinned
            );
            assert!(
                screen.contains(&"scroll_area"),
                "every screen in this module is an area's"
            );
        }
        assert_eq!(
            BAND_SCREEN, SUBJECTS,
            "the band screen carries the module's whole population, and the tail screen is the \
             half of it that cuts no band"
        );
    }

    /// **The fixture is the shape the scenes claim**, so a scene stating a content nothing asserts
    /// cannot drift — §21's own failure mode arriving as a number.
    #[test]
    fn the_fixture_is_the_shape_the_scenes_claim() {
        let view = tail_view();
        assert!(
            SHORT.0 < u32::from(view.w) && SHORT.1 < u32::from(view.h),
            "a tail exists only where the content is smaller than the viewport: `max_offset` is \
             `extent - viewport`, so at any admissible offset the range past the extent is empty"
        );
        let max = parts(Rect::new(0, 0, W, H), LONG, &tail_opts()).max_offset(LONG);
        assert!(
            FAR.0 <= max.0 && FAR.1 <= max.1,
            "the offset both scenes are played at has to be one the long content admits, or it is \
             a clamp reported as a position: {FAR:?} against {max:?}"
        );
        assert_ne!(
            FAR.0, FAR.1,
            "a transposed band is correct where the two offsets are equal"
        );
        assert_ne!(NEAR, AT_REST);
        assert_ne!(NEAR, FAR);
        assert!(
            i32::from(HEADER) < NEAR.1,
            "a one-row band pushed off its own row by `Shared::Both` has to be pushed *out*, or \
             the arm is clean for a reason nothing recorded"
        );

        // **The two rotas are long enough that no two cells of one screen alias**, which is what
        // makes a band off by `k` a different string for every `k` this rig can hold.
        assert!(COLUMN_ROTA.len() > usize::from(W));
        assert!(ROW_ROTA.len() > usize::from(H));
        assert_eq!(
            COLUMN_ROTA.len(),
            ROW_ROTA.len(),
            "one length, so a mark's index is the only thing that distinguishes the two alphabets"
        );
        assert_ne!(
            COLUMN_ROTA, ROW_ROTA,
            "two alphabets, so a band reading the wrong axis of the offset is a different \
             alphabet and not only a different index"
        );
        assert_eq!(STALE_DOWN.0, STALE_DOWN.1 * usize::from(tail_view().w));
        assert_eq!(STALE.1, usize::from(view.h));
    }
}
