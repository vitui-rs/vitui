//! **The field's window: a scroll it does not follow, a shrink it does not clear, and a pull that
//! fights the wheel.**
//!
//! Production ticket 05. Spec §11, §17 (O5), §21. This is the screen `field`'s other three scenes
//! are scenes *of* — [`crate::document`] carries the fourth, the row 13, and the two files
//! divide the way the axes do: that one is **text measurement** at two widths and this one is
//! **The window over the text**.
//!
//! | scene | what it decides |
//! |---|---|
//! | the document at visual row [`SCROLLED_TO`] | the window's sign, and the caret's row measured from the wrong end |
//! | a megabyte edited down to one line | the stale tail, and the resize spelling that scores it clean |
//! | twenty posted notches over a field | the pull, and the arm that passes a one-directional gate |
//!
//! # All three are one question asked three ways, which is why they are one module
//!
//! `field` owns its offset (§17: `owns_offset`), so the same integer decides all three: where the
//! window sits, what happens to the rows the content has stopped reaching, and what a posted notch
//! is allowed to do to it. The four axes are not a taxonomy — each was established by a defect
//! that **passed every gate then in force and looked healthier than the correct build** — and on
//! this component three of the four land on that one integer. A module per axis would have put the
//! same reference render in three files, which is how three sessions come to disagree about it.
//!
//! # The oracle is the tail of the document as its own field
//!
//! Every one of the three is decided by an equality against a **reference render**, and the
//! reference is the same one in all three: [`tail`] builds a fresh [`crate::edit::Text`] over the
//! *suffix* of the document starting at the hard line that visual row `k` begins, and draws it at
//! offset **0**. So the window arithmetic is `0 + r` on the reference side and `offset + r` on the
//! subject's, and a defect in that arithmetic cannot hide in both.
//!
//! **What it is independent of is the window and not the row drawing**, and that is stated rather
//! than glossed: both sides go through [`crate::input::field_into`], so a defect in `truncate`, in
//! the pad or in the selection painter is invisible to this equality — those are register rows 15
//! to 18's and [`crate::document`]'s, one file over. The trap this file is written against is *the
//! recorder and the defect share a coordinate system*, and it is the **offset** that is the shared
//! coordinate.
//!
//! **The one precondition is that the cut is a row start**, and it is one because a stricter one
//! was invented, measured and deleted: a greedy wrap restarted at *any* visual row start reproduces
//! the document's rows from there on, second rows of wrapped lines included. [`row_start_byte`]
//! carries that measurement, and [`SUFFIX_ROWS`] is asserted beside it as a corroboration that the
//! document has not moved under the constant — never as the reason the oracle is one, because the
//! row count agrees on an inadmissible cut too.
//!
//! # The caret is the one thing the equality cannot see, and it is a defect anyway
//!
//! [`crate::input::defective::CaretRow::Content`] places the caret at the **content** row rather
//! than at the screen row the content was drawn on. The two are the same number at offset 0 and at
//! no other offset — so it is a defect that is right on every field anybody writes a gate for, and
//! wrong the moment a reader scrolls. It costs [`crate::document::SURFACE_BLIND`] cells: the caret
//! is the terminal's cursor and not a cell, so the reference-render equality is **blind to it** and
//! so is every one of the nine counters. [`misplaced_caret`] reads `Frame::caret`, which is the
//! only instrument here that can.
//!
//! That is [`crate::document`]'s own split — *invisible on the rendered screen turns out to mean two
//! different things* — arriving on a third axis, and it is why this module reports two numbers per
//! scene rather than one.
//!
//! # The stale tail is a defect of the second frame given the first
//!
//! [`Play`] draws every step into **one** [`crate::runner::Pen`] and never clears it, which is
//! [`crate::runner::play`]'s own arrangement and for its reason: a cell nobody wrote keeps what was
//! already there, and a runner that started each step from a blank surface would score the stale
//! tail clean. [`stale_by_resize`] is the spelling §21 refuses, kept beside the one it banks so the
//! difference is a number and not a warning.
//!
//! # The notches are posted, not added
//!
//! [`crate::wheel`]'s finding, inherited whole: a click is a [`vitui_runtime::Mouse`] with a
//! [`vitui_runtime::Notch`] on it, posted through `Driver::post_mouse`, resolved against the
//! **previous** frame's hit index and read by the component inside its own draw. A delta handed
//! straight to the arithmetic cannot tell a component that publishes a scrollable region from one
//! that does not — and `field` publishing one is the whole of why it may be wheeled at all.
//!
//! **The three reveal arms are three programs and not two** ([`crate::input::defective::Reveal`]),
//! because a gate written in one direction goes green the moment somebody deletes the call:
//! [`crate::input::defective::Reveal::Never`] moves the window exactly as far as the
//! rule does and loses the keyboard instead. [`wheeled`] reports all three and [`revealed_by_a_key`]
//! is the half that separates the third.

use vitui_runtime::keys::{Chord, Code};
use vitui_runtime::{
    Button, Buttons, Cursor, Density, Driver, Mods, Mouse, MouseKind, Notch, Rect,
};

use crate::counters::{Allocations, Counter, Counters};
use crate::edit::{Text, WrapKind};
use crate::input::FieldOpts;
use crate::input::defective::{CaretRow, Refused, Reveal, Window, field_refused};
use crate::keys;
use crate::runner::{Canvas, Diff, Pen, driver_at};

// ── the screen ───────────────────────────────────────────────────────────────────────────────────

/// The rectangle's width. [`crate::document::NARROW`]'s hundred and twenty, which is the original's
/// narrow width for this component and the width its wrap index is a scene about.
pub const W: u16 = crate::document::NARROW;

/// The rectangle's height. [`crate::document::H`]'s eighty, which is where *n of 80 rows* is
/// counted.
pub const H: u16 = crate::document::H;

/// **How many visual rows the document holds at [`W`].** [`crate::document::WRAPPED`]'s 875.
pub const ROWS: usize = crate::document::WRAPPED;

/// **How many bytes the document holds.** Measured, because `crate::document::text` is built at
/// run time out of a rota of clusters and no `const` can name its length.
///
/// `crate::scenes`'s row for scene 34 states the content it stands up, and a scene stating a size
/// nothing asserts is the failure mode arriving as a number — so the figure lives here, with
/// the rest of this module's ledger, and `tests::the_document_is_the_size_the_scene_claims` is what
/// keeps it honest.
pub const DOCUMENT_BYTES: u64 = 179_885;

/// **The visual row the window is scrolled to. Four hundred**, which is a hard line's first row.
///
/// Deep enough that the window shares no row with the unscrolled one — *at offset 0 every sign
/// agrees*, which is the runtime's own finding about the scroll axis (architecture 26) and the
/// reason a scrolled scene may not be played near the top. It happens to be a hard line's first
/// visual row and nothing rests on that: see [`row_start_byte`], where the precondition that would
/// have needed it was measured and deleted.
pub const SCROLLED_TO: usize = 400;

/// **How many rows the reference render holds.** [`ROWS`] − [`SCROLLED_TO`].
///
/// The precondition that makes the oracle admissible, and it is asserted before a cell is compared:
/// a greedy wrap of the suffix agrees with the whole document's wrap from [`SCROLLED_TO`] on, or
/// the two renders are of two different documents and the equality is about the fixture.
pub const SUFFIX_ROWS: usize = ROWS - SCROLLED_TO;

/// **Where the caret is seated, as a visual row of the document.**
///
/// Five rows into the window, so that both arms of [`misplaced_caret`] place a caret at all: a
/// caret outside the window is `None` on the correct build, which is the right answer and a weaker
/// statement than two carets at two rows.
pub const CARET_ROW: usize = SCROLLED_TO + 5;

/// Which column the caret is seated at. Three, and every row of the document is longer.
pub const CARET_COL: u16 = 3;

/// **How many clicks the wheel scene posts.** [`crate::wheel::CLICKS`]'s twenty, which is the original's
/// number for the axis.
pub const CLICKS: u32 = crate::wheel::CLICKS;

/// **Where the wheel scene starts.** [`SCROLLED_TO`] − [`CLICKS`], so that the settled window is
/// [`SCROLLED_TO`] and the two scenes share one reference render.
///
/// The landing row is what has to be a hard line's, not the start: the equality is asked of the
/// screen the clicks came to rest on. Sharing the oracle is the point — the scrolled scene decides
/// *The window at rest* and this one decides *the window a notch put there*, against one answer.
pub const WHEELED_FROM: usize = SCROLLED_TO - CLICKS as usize;

/// **What the megabyte is edited down to.** The *edited down to one line*.
pub const SHRUNK_LINE: &str = "one line, inside a rectangle that did not move";

/// **How many visual rows [`SHRUNK_LINE`] takes at [`W`]. One**, which is the whole point of the
/// line: it is shorter than the rectangle is wide, so the shrink leaves seventy-nine rows with
/// nothing in them.
pub const SHRUNK_ROWS: u16 = 1;

// ── the ledger ───────────────────────────────────────────────────────────────────────────────────
//
// **Every number this module is gated or reported on has one home and it is here** — the runtime's
// ledger rule, which `crate::document` and `crate::state` already inherit. Every figure is a count
// over a deterministic screen, so none carries a machine.

/// **Rows the inverted window draws wrong. Seventy-nine of eighty.**
///
/// Row 0 is the one it gets right — `offset - 0` is `offset` — and the *rows* are the number that
/// carries the shape here, which is [`crate::runner::Diff`]'s own reason for reporting two: *6 662
/// cells over 80 rows is a whole screen wrong, 6 662 over 9 is a band.*
pub const INVERTED_ROWS: usize = H as usize - 1;

/// **Cells the inverted window and the reference disagree on. 5 891 of 9 600.**
///
/// **Well under `INVERTED_ROWS * W`, and the gap is the fixture rather than the defect.**
/// `crate::runner::Fixture::lines` makes rows that share no column for exactly this reason — *a
/// wrong row costs exactly `w` cells* — and this scene is played over the document, which is
/// prose: two rows of English agree wherever their words do. The cell count is therefore a floor on
/// how wrong the screen is and never a measure of it, which is why [`INVERTED_ROWS`] is the number
/// this scene is stated in.
pub const INVERTED_CELLS: usize = 5_891;

/// **Rows left standing when the megabyte becomes one line. Seventy-nine of eighty**, which is
/// the stale tail on this component.
pub const STALE_ROWS: usize = H as usize - 1;

/// **Cells the stale tail leaves wrong. 5 519 of 9 600.** See [`INVERTED_CELLS`]: the shortfall is
/// prose agreeing with prose, not the tail being partly clean.
pub const STALE_CELLS: usize = 5_519;

/// **The visual row a mid-line cut is measured at. Eleven** — the first row of the document whose
/// start is not a hard line's, which is the second row of the first line long enough to wrap.
pub const MISCUT_ROW: usize = 11;

/// **Rows a suffix cut at [`MISCUT_ROW`] re-wraps to. 864**, which is [`ROWS`] − [`MISCUT_ROW`] and
/// therefore exactly what the document holds from that row on.
///
/// **It is not a number about the window** — the window is [`H`] rows and the oracle holds
/// [`SUFFIX_ROWS`] — and saying so was a false sentence in four places before a review caught it.
/// What it is about is the *row count as a precondition*: the count agrees on a cut that was
/// expected to change it, so a count cannot tell an admissible cut from an inadmissible one. See
/// [`row_start_byte`], where that measurement deleted a precondition rather than corroborating one.
pub const MISCUT_ROWS_AGREE: usize = ROWS - MISCUT_ROW;

/// **Where the caret goes on the correct build**, as a row of the screen. Five.
pub const CARET_SCREEN_ROW: u16 = (CARET_ROW - SCROLLED_TO) as u16;

/// **Where the settled window sits after [`CLICKS`] posted notches under the rule.**
/// [`SCROLLED_TO`].
pub const WHEELED_TO: usize = SCROLLED_TO;

/// **Where it sits under an unconditional reveal. Zero** — dragged back to the caret's row, on
/// every frame, for ever.
pub const DRAGGED_BACK: usize = 0;

/// **Rows the pull leaves disagreeing with the oracle. Forty-five of eighty.**
///
/// Not seventy-nine, and the shortfall is the **fixture** rather than the defect — see
/// [`ALIASED_ROWS`], which is that shortfall as a measurement instead of as an excuse.
pub const PULLED_ROWS: usize = 45;

/// **Cells the pull leaves disagreeing with the oracle. 864 of 9 600.**
///
/// Nineteen cells a wrong row, which is [`INVERTED_CELLS`]'s shortfall an order worse, and it is
/// the reason this scene is stated in rows: a reader whose viewport will not stay where they put it
/// is looking at an entirely wrong screen, and the cell count says *9% of it*.
pub const PULLED_CELLS: usize = 864;

/// **How many cells the rectangle holds. 9 600**, and it is here because three sentences quoted it
/// and nothing asserted it.
pub const CELLS: usize = W as usize * H as usize;

/// **Cells an oracle drawn without the pointer on it disagrees with the subject about. All of
/// them.**
///
/// The hover face is part of the screen: [`crate::state::press_into`] resolves *pressed beats
/// hovered beats focused* and the paint lands on the cell, so a wheel scene whose subject has a
/// pointer over it and whose reference does not reports the **whole rectangle** — in the paint and
/// never in the cluster. It was a sentence in three files with no instrument behind it until a
/// review said so; [`pointless_oracle`] is the instrument.
pub const NO_POINTER_CELLS: usize = CELLS;

/// **Distinct cells touched, on both arms of the shrink.** [`CELLS`], and the equality is the
/// finding rather than the number: the counter is cumulative over the play and the first frames
/// already touched every cell, so *how many cells were ever written* cannot fall when a later frame
/// stops writing some of them.
pub const SHRUNK_DISTINCT: u64 = CELLS as u64;

/// **Rows of the pull's screen that agree with the oracle *entirely*. Thirty-five of eighty.**
///
/// **The fixture's own stated property does not reach this comparison, and that is the finding.**
/// [`crate::document::lines`] says *line `i` opens with its own four-digit index, so no two lines
/// share a prefix and a wrong row costs a full row of cells* — [`crate::runner::Fixture::lines`]'s
/// property, restated one file over. It is true of the document's **hard lines** and not of its
/// **visual rows**: [`crate::document::LONG`] of the six hundred and twenty-five lines wrap to two
/// rows at [`W`], and a long line's *second* visual row carries no index at all. It is a suffix of
/// [`crate::clusters::ROTA`]'s eight-word rota, so two second rows an even number of rows apart are
/// **The same string** — and every offset in this module is an even number of rows.
///
/// Thirty-five is that, counted.
///
/// **It is the measurement and explains the number**, and saying it explained scene
/// 34's was a false sentence a review caught. Scene 34 compares row `400 - r` against row `400 + r`
/// and reports 79 rows apart, so *these* thirty-five rows are not among its agreements at all — the
/// same mechanism is at work there and it is a different pair of screens, which is why
/// [`INVERTED_CELLS`] gives its shortfall the general reason and not this count.
///
/// The scenes are stated in rows because of it, the sentence in [`crate::document::lines`] was
/// corrected to say which unit it is about, and **the fixture is not bent to make either number
/// nicer**: the original's 625/875 is a property of this generator, and a generator changed to sharpen a
/// cell count would move a normative figure to win an argument.
pub const ALIASED_ROWS: usize = 35;

// ── the document, and the oracle over its tail ───────────────────────────────────────────────────

/// **The byte the buffer is cut at to make the oracle**: the start of visual row `row` at [`W`].
///
/// # The cut may be any row start, and that was measured rather than assumed
///
/// This began as `hard_line_row` — *the first visual row at or after `from` that a hard line
/// begins* — on the reasoning that a greedy break depends on the text after it, so a suffix cut in
/// the middle of a wrapped line would re-wrap differently. **It does not.** Cut at [`MISCUT_ROW`],
/// the second visual row of the first line long enough to wrap, the suffix re-wraps to
/// [`MISCUT_ROWS_AGREE`] rows — exactly what the document holds from that row on — and its first
/// twenty rows are that row's first twenty **string for string**. A greedy wrap only ever looks
/// forward, so restarting it at a row start restarts it at a decision it was going to make anyway.
///
/// So the precondition is *the cut is a row start*, which is what [`crate::edit::Index::row_start`]
/// answers by construction, and the hard-line search was deleted after being watched holding on
/// both kinds of cut. `tests::a_suffix_cut_at_any_row_start_reproduces_the_window` is the
/// measurement, and it is kept because the invented precondition is the kind a later session
/// re-invents.
///
/// # Panics
///
/// Panics when `row` is not a row of the document at [`W`]. `Index::row_start` answers 0 out of
/// range, and an oracle silently cut at byte 0 is an oracle comparing the window against the top of
/// the document — which is exactly the defect the pull arm is about, arriving in the
/// instrument.
pub fn row_start_byte(row: usize) -> usize {
    let mut st = Text::of(crate::document::text(), WrapKind::Words);
    let rows = st.index(W).rows();
    assert!(
        row < rows,
        "visual row {row} is past the document's {rows} at {W} columns, and `Index::row_start` \
         answers 0 out of range — so the oracle would be cut at the top of the document"
    );
    st.index(W).row_start(row)
}

/// **The reference render's state: the document's tail as a field of its own, at offset 0.**
///
/// # What the precondition is, and what it is not
///
/// It is **the cut is a row start** — see [`row_start_byte`], where the stricter one this began
/// with was measured and refused. It is **not** the row count: a suffix cut in the middle of a
/// wrapped line re-wraps to exactly the same number of rows as one cut at a hard line, so a count
/// cannot tell an admissible cut from an inadmissible one. The count is asserted below as a
/// corroboration that the document has not moved under the constant, and never as the reason the
/// oracle is one.
///
/// # Panics
///
/// Panics when the suffix does not re-wrap to [`SUFFIX_ROWS`] rows, which means the document or
/// [`SCROLLED_TO`] has moved and the reference is a render of something else.
pub fn tail() -> Text {
    let byte = row_start_byte(SCROLLED_TO);
    let text = crate::document::text();
    let mut st = Text::of(text[byte..].to_string(), WrapKind::Words);
    let rows = st.index(W).rows();
    assert_eq!(
        rows, SUFFIX_ROWS,
        "the suffix re-wraps to {rows} rows where the window is showing {SUFFIX_ROWS} of them, so \
         the reference render is a render of a different document"
    );
    st
}

/// **The subject's state: the whole document, scrolled to [`SCROLLED_TO`], caret seated in the
/// window.**
pub fn scrolled() -> Text {
    let mut st = Text::of(crate::document::text(), WrapKind::Words);
    let _ = st.index(W);
    // **The caret is placed by a click and not by a byte**, which is §11's rule on the pointer
    // path: a column is a boundary by construction. It is called outside a frame, so nothing has
    // asked for a reveal and the offset below is the one the screen is drawn at.
    st.click(W, CARET_ROW, CARET_COL, false);
    st.scroll_to(SCROLLED_TO);
    st
}

/// The megabyte, at offset 0, so that every row of the rectangle has content on the first frame.
pub fn pasted() -> Text {
    let mut st = Text::of(crate::document::pasted(), WrapKind::Words);
    let _ = st.index(W);
    st
}

// ── one play loop, and every arm goes through it ─────────────────────────────────────────────────

/// **One field, drawn over a sequence of frames into a surface that is not cleared between them.**
///
/// [`crate::runner::play`]'s arrangement, with a [`Text`] where that one has a
/// [`crate::runner::Fixture`]: the surface persisting across frames is the mechanism the shrink
/// axis is about, and the step list is a closure per frame because two of the three scenes change
/// the state between frames rather than the content of a fixture.
///
/// # The warm frame is the runtime's cadence and not this instrument's arrangement
///
/// Nothing holds the keyboard on the first frame of any program (runtime architecture 25), and the
/// face a field draws in depends on whether it is focused ([`crate::state::press_into`]) — so a
/// reference render played one frame and a subject played two would differ on **every cell of the
/// rectangle**, in the paint rather than in the cluster. Every play here opens with one frame whose
/// only job is to seat the focus, exactly as [`crate::document::play_field`] does and for the same
/// reason.
pub struct Play {
    driver: Driver,
    pen: Pen,
    st: Text,
    opts: FieldOpts,
    refused: Refused,
    h: u16,
}

impl Play {
    /// A play over `st` in an [`H`]-row rectangle, with every refusal in `refused`.
    pub fn new(st: Text, refused: Refused) -> Play {
        Play::in_rows(st, refused, H)
    }

    /// **A play in a rectangle of `h` rows**, which is what [`stale_by_resize`] needs and the one
    /// thing §21 says may stand *beside* the shrink spelling and not instead of it.
    pub fn in_rows(st: Text, refused: Refused, h: u16) -> Play {
        Play {
            driver: driver_at(W, h, Density::default()),
            pen: Pen::new(W, h),
            st,
            opts: FieldOpts::default(),
            refused,
            h,
        }
    }

    /// The same play with a stated reveal arm.
    pub fn revealing(mut self, reveal: Reveal) -> Play {
        self.opts.reveal = reveal;
        self
    }

    /// **One frame.** The field fills the rectangle, and the focus is seated the way an application
    /// seats it — `if cx.focused().is_none()`, which is the refusal of a
    /// runtime that seats the first stop.
    pub fn frame(&mut self) {
        let area = Rect::new(0, 0, W, self.h);
        let pen = &mut self.pen;
        let st = &mut self.st;
        let opts = &self.opts;
        let refused = self.refused;
        self.driver.frame(|cx| {
            let resp = field_refused(pen, cx, area, st, opts, refused);
            if cx.focused().is_none() {
                cx.focus(resp.id);
            }
        });
    }

    /// The frame that seats the focus, then `frames` more.
    pub fn played(mut self, frames: u32) -> Play {
        for _ in 0..=frames {
            self.frame();
        }
        self
    }

    /// Post one wheel click down over the middle of the rectangle.
    pub fn click(&mut self) {
        self.driver.post_mouse(Mouse {
            x: W / 2,
            y: self.h / 2,
            kind: MouseKind::Wheel(Notch::Down),
            buttons: Buttons::NONE,
            mods: Mods::NONE,
            at: std::time::Instant::now(),
        });
    }

    /// Give the pointer a position, so that a hit can be `over` at all.
    pub fn point(&mut self) {
        self.driver.post_mouse(Mouse {
            x: W / 2,
            y: self.h / 2,
            kind: MouseKind::Move,
            buttons: Buttons::NONE,
            mods: Mods::NONE,
            at: std::time::Instant::now(),
        });
    }

    /// Post one key press, which is a gesture that asks for a reveal.
    pub fn press(&mut self, chord: Chord) {
        self.driver.post_key(keys::press(chord));
    }

    /// Press the left button over the pointer's position, and release it.
    pub fn tap(&mut self) {
        for kind in [MouseKind::Down(Button::Left), MouseKind::Up(Button::Left)] {
            self.driver.post_mouse(Mouse {
                x: W / 2,
                y: self.h / 2,
                kind,
                buttons: Buttons::NONE,
                mods: Mods::NONE,
                at: std::time::Instant::now(),
            });
        }
    }

    /// The state, so a caller can edit it between frames.
    pub fn state(&mut self) -> &mut Text {
        &mut self.st
    }

    /// Where the window sits.
    pub fn offset(&self) -> usize {
        self.st.offset()
    }

    /// **Where the caret went**, as `settle` will hand it to the terminal. Not a cell.
    pub fn caret(&self) -> Option<Cursor> {
        self.driver.inspect().caret()
    }

    /// The nine per-frame counters, of which this crate can read eight.
    pub fn counters(&self, allocations: Allocations) -> Counters {
        Counters::of(&self.driver, self.pen.tally(), allocations)
    }

    /// The surface, as the terminal holds it after the last frame.
    pub fn canvas(self) -> Canvas {
        self.pen.into_canvas()
    }
}

// ── scene 34: the window's sign, and the caret's row ─────────────────────────────────────────────

/// **The correct window against the reference render.** Clean, and it is the arm that says the
/// oracle is an oracle.
///
/// A comparison whose correct arm has never been watched agreeing reports *0 cells over 0 rows* for
/// the same reason a broken one would — `crate::runner::defective`'s own sentence, and §21 has the
/// finding three times from the other direction.
pub fn at_rest() -> Diff {
    against_the_tail(Refused::NONE, 1)
}

/// **The inverted window against the reference render.** [`INVERTED_CELLS`] cells over
/// [`INVERTED_ROWS`] rows.
pub fn inverted() -> Diff {
    against_the_tail(Refused::windowed(Window::Inverted), 1)
}

/// **The caret measured from the content row, against the reference render.**
/// [`crate::document::SURFACE_BLIND`] cells — the equality is blind to it, and so is every counter.
pub fn caret_at_the_content_row() -> Diff {
    against_the_tail(Refused::carets(CaretRow::Content), 1)
}

/// One arm of the scrolled scene, compared against [`tail`] drawn for the same number of frames.
fn against_the_tail(refused: Refused, frames: u32) -> Diff {
    let subject = Play::new(scrolled(), refused).played(frames).canvas();
    let reference = Play::new(tail(), Refused::NONE).played(frames).canvas();
    subject.diff(&reference)
}

/// **Where the caret lands under each build**, as a screen row.
///
/// Returns `(the rule, the refusal)`. The rule is [`CARET_SCREEN_ROW`]; the refusal is the content
/// row, which on an eighty-row screen is off the rectangle entirely — so the two readings are *a
/// caret five rows down* and *no caret at all*, on two screens that are cell for cell identical.
///
/// **This is the one thing in this module no equality against a reference render can see**, which
/// is [`crate::document`]'s split — *invisible on the rendered screen means two different things* —
/// arriving on the scroll axis.
pub fn misplaced_caret() -> (Option<u16>, Option<u16>) {
    let rule = Play::new(scrolled(), Refused::NONE).played(1);
    let refusal = Play::new(scrolled(), Refused::carets(CaretRow::Content)).played(1);
    (rule.caret().map(|c| c.y), refusal.caret().map(|c| c.y))
}

// ── scene 35: the stale tail ─────────────────────────────────────────────────────────────────────

/// **The megabyte edited down to one line, in a rectangle that does not move**, against a field
/// freshly built over that one line.
///
/// Two frames into one surface: the first draws the megabyte, the second draws what is left after
/// [`Text::select_all`] and an insert. The spelling of the axis, and the rectangle is [`W`]
/// by [`H`] on both frames.
pub fn shrunk(refused: Refused) -> Diff {
    let reference = Play::new(one_line(), Refused::NONE).played(1);
    shrunk_play(refused).canvas().diff(&reference.canvas())
}

/// **The same defect spelled as a terminal resize — and it scores clean.**
///
/// §21 refuses to bank the shrink gate written this way, and this is the refusal as a number rather
/// than as a sentence: a fresh rectangle has nowhere for the residue to survive, so the arm that
/// leaves seventy-nine rows standing draws the same screen as the rule.
///
/// It is spelled the way [`crate::listing::stale_by_resize`] spells it one component over: the
/// **rectangle** becomes the content's height, which is what a terminal resize down to the shrunk
/// document gives you. The refusal then draws every row the rectangle has, because the rectangle
/// has exactly the rows the content has — and there is nothing left for a residue to sit in.
///
/// **The first spelling of this was wrong and is worth recording.** Written as *the same
/// [`H`]-row play, started at the one line*, it reported the tail dirty rather than clean: the
/// refusal writes one row and leaves seventy-nine **untouched**, and an untouched cell is
/// [`crate::runner::Canvas`]'s own value and not a blank — *modelling it as a space would make the
/// two arms agree on exactly the cells the shrink axis is about.* That reading is a fact about a
/// surface nobody has drawn on, not about a resize.
pub fn stale_by_resize(refused: Refused) -> Diff {
    let rows = SHRUNK_ROWS;
    let subject = Play::in_rows(one_line(), refused, rows).played(1);
    let reference = Play::in_rows(one_line(), Refused::NONE, rows).played(1);
    subject.canvas().diff(&reference.canvas())
}

/// The one line the megabyte is edited down to, as a field of its own.
fn one_line() -> Text {
    Text::of(SHRUNK_LINE.to_string(), WrapKind::Words)
}

// ── scene 36: the pull ───────────────────────────────────────────────────────────────────────────

/// **What [`CLICKS`] posted notches did to the window**, under one reveal arm.
///
/// The run opens with a frame that has no click on it — a notch is resolved against the *previous*
/// frame's hit index, and nothing is `over` until the pointer has a position — and closes with one
/// more, because *twenty wheel clicks move the window twenty* is a statement about where the screen
/// came to rest.
pub fn wheeled(reveal: Reveal) -> usize {
    wheeled_play(reveal).offset()
}

/// **The wheel scene's play.** One function for [`wheeled`]'s reason and for [`shrunk_play`]'s: the
/// offset reading and the screen reading have to be two readings of one run, or the number and the
/// picture come to describe two different sequences of clicks.
pub fn wheeled_play(reveal: Reveal) -> Play {
    let mut play = Play::new(wheel_state(), Refused::NONE).revealing(reveal);
    play.point();
    play.frame();
    play.frame();
    for _ in 0..CLICKS {
        play.click();
        play.frame();
    }
    play.frame();
    play
}

/// **The settled screen after the notches, against the reference render.**
///
/// Clean under the rule, because the window the clicks reached is [`SCROLLED_TO`] and that is the
/// row the oracle is cut at. A pull reports the whole screen.
///
/// # The reference render has the pointer on it too, and that is not tidiness
///
/// **The hover face is part of the screen.** [`crate::state::press_into`] resolves *pressed beats
/// hovered beats focused* and the paint is on the cell, so a wheel scene whose subject has a
/// pointer over it and whose oracle does not disagrees on **9 600 of 9 600 cells** — measured — in
/// the paint and never in the cluster. That is the whole screen reported wrong for a reason that
/// has nothing to do with the window, which is a defect in the instrument of exactly the kind
/// the caret half is about from the other side.
pub fn wheeled_screen(reveal: Reveal) -> Diff {
    wheeled_play(reveal)
        .canvas()
        .diff(&pointed_at_tail().canvas())
}

/// **The wheel scene against an oracle with no pointer on it**, which is [`NO_POINTER_CELLS`].
///
/// The instrument behind a sentence that stood in three files with none. It is a defect *in the
/// instrument* rather than in the component, so it is measured here and not filed as a scene.
pub fn pointless_oracle() -> Diff {
    let reference = Play::new(tail(), Refused::NONE).played(1);
    wheeled_play(Reveal::WhenAsked)
        .canvas()
        .diff(&reference.canvas())
}

/// **The wheel scene's oracle**: [`tail`] with the pointer on it, for [`wheeled_screen`]'s reason.
pub fn pointed_at_tail() -> Play {
    let mut reference = Play::new(tail(), Refused::NONE);
    reference.point();
    reference.frame();
    reference.frame();
    reference
}

/// **Whether a keyboard gesture still brings the caret into view**, which is the half
/// [`Reveal::Never`] fails.
///
/// Returns the window's offset after a `Right` posted into a field whose caret is a long way below
/// it. The rule and the unconditional arm both follow the caret; deleting the call does not, and a
/// gate written only on [`wheeled`] would call that a pass.
pub fn revealed_by_a_key(reveal: Reveal) -> usize {
    let mut st = Text::of(crate::document::text(), WrapKind::Words);
    let _ = st.index(W);
    st.click(W, CARET_ROW, CARET_COL, false);
    st.scroll_to(0);
    let mut play = Play::new(st, Refused::NONE).revealing(reveal);
    play.frame();
    play.frame();
    play.press(Chord::new(Code::Right));
    play.frame();
    play.offset()
}

/// The wheel scene's state: the document, at [`WHEELED_FROM`], with the caret at the top.
///
/// **The caret is where an unconditional reveal will drag the window to**, and it is left at byte 0
/// deliberately: the defect is not *the window moves oddly*, it is *the window goes back to the
/// caret and the reader cannot get away from it*.
fn wheel_state() -> Text {
    let mut st = Text::of(crate::document::text(), WrapKind::Words);
    let _ = st.index(W);
    st.scroll_to(WHEELED_FROM);
    st
}

// ── what the counters say about all three ────────────────────────────────────────────────────────

/// **Which of the nine counters tell a refused build from the rule.**
///
/// [`crate::listing::counters_that_separate_them`]'s shape, one component over and over three
/// refusals rather than one. An empty answer means *the equality against a reference render is the
/// only detector there is*; a non-empty one would mean a cheaper gate exists and the scene is
/// optional.
///
/// A counter that is [`Reading::Unreachable`] on both arms — `marked`, and it will stay that way —
/// contributes nothing either way and is skipped. A counter reachable on one arm and not the other
/// would be a defect in the instrument and is reported as a separation.
///
/// # The allocation totals are the caller's, and that is not a hole
///
/// [`crate::runner::Run::counters`]'s arrangement, for its reason: `vitui-alloc-probe` is a
/// dev-dependency and a library cannot install a global allocator on a consumer's behalf. A caller
/// with no probe hands both arms the same value, which makes that column inert rather than false.
///
/// [`Reading::Unreachable`]: crate::counters::Reading::Unreachable
pub fn counters_that_separate(
    on: On,
    refused: Refused,
    rule_allocations: Allocations,
    refused_allocations: Allocations,
) -> Vec<Counter> {
    let (a, b) = counters_approve(on, refused, rule_allocations, refused_allocations);
    Counter::ALL
        .into_iter()
        .filter(|c| {
            let (left, right) = (a.get(*c).measured(), b.get(*c).measured());
            left.is_some() != right.is_some() || (left.is_some() && left != right)
        })
        .collect()
}

/// **Every counter this crate can read, over the rule and over one refusal.** Returns
/// `(rule, refused)`.
///
/// The scenes exist because the pairs are *indistinguishable*, and a report printing only the diff
/// would leave the reader to take on trust that the refused build looked healthier. This is the
/// half that stops that being trust — and on the shrink axis it is stronger than *the same*: the
/// stale tail writes seventy-nine fewer rows and its every counter is **better**.
pub fn counters_approve(
    on: On,
    refused: Refused,
    rule_allocations: Allocations,
    refused_allocations: Allocations,
) -> (Counters, Counters) {
    let (rule, bad) = match on {
        On::TheShrink => (shrunk_play(Refused::NONE), shrunk_play(refused)),
        On::TheWindow => (
            Play::new(scrolled(), Refused::NONE).played(1),
            Play::new(scrolled(), refused).played(1),
        ),
    };
    (
        rule.counters(rule_allocations),
        bad.counters(refused_allocations),
    )
}

/// **Which scene a counter comparison is played on.**
///
/// A named argument and **not** a `match` on the refusal, which is what this was: read that way, a
/// refusal the dispatch did not enumerate falls into the other scene's play silently, and the
/// answer is then a comparison of two screens nobody asked about. A caller naming the screen cannot
/// make that mistake without writing it down.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum On {
    /// Scene 34: the document at [`SCROLLED_TO`], one frame.
    TheWindow,
    /// Scene 35: the megabyte edited down to one line, three frames into one surface.
    TheShrink,
}

/// **The shrink scene's play**, up to and including the frame after the edit.
///
/// One function and not two: [`shrunk`] and [`counters_approve`] both need this exact sequence, and
/// two copies of it would let the cell count and its counter argument come to describe two
/// different plays.
pub fn shrunk_play(refused: Refused) -> Play {
    let mut play = Play::new(pasted(), refused);
    play.frame();
    play.frame();
    play.state().select_all(W);
    play.state().insert(W, SHRUNK_LINE);
    play.frame();
    play
}

// ── the subject, and the scan that says whether it is here ───────────────────────────────────────

/// **What these three screens are screens of.** [`crate::document::SUBJECTS`], reached through this
/// alias so the two files cannot drift about which component `field` is.
pub const SUBJECTS: &[&str] = &crate::document::SUBJECTS;

/// **The components a posted wheel notch is played over here.** One, and it is scene 36's.
///
/// [`crate::wheel::Subject::ALL`]'s counterpart, and it is a second const rather than a reuse of
/// [`SUBJECTS`] for that value's own reason: `crate::obligations`'s wheel-pair join reads it as a
/// **population**, and a fourth scene in this module over some other component would grow
/// `SUBJECTS` and silently widen a join that is meant to enumerate. Three scenes stand `field` up
/// and one of them posts a notch; those are two facts and this is the second one.
pub const WHEELED_SUBJECTS: &[&str] = &["field"];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Axis, INVENTORY};

    /// **The window sits inside the document and the oracle holds the rows it is showing**, both
    /// computed off the document rather than typed.
    ///
    /// The two sides are two derivations: one is the greedy wrap of six hundred and twenty-five
    /// hard lines at a hundred and twenty columns, the other is three numbers in this file.
    #[test]
    fn the_window_sits_inside_the_document_and_the_oracle_holds_its_rows() {
        let mut st = Text::of(crate::document::text(), WrapKind::Words);
        assert_eq!(st.index(W).rows(), ROWS);
        assert!(
            SCROLLED_TO + H as usize <= ROWS,
            "the window hangs off the end"
        );
        assert_eq!(tail().index(W).rows(), SUFFIX_ROWS);

        // **The oracle is cut where the window starts**, which is the one precondition left.
        assert_eq!(
            row_start_byte(SCROLLED_TO),
            st.index(W).row_start(SCROLLED_TO)
        );
    }

    /// **`row_start_byte` refuses a row the document does not have**, because `Index::row_start`
    /// answers 0 out of range and an oracle silently cut at byte 0 is the pull defect arriving in
    /// the instrument.
    #[test]
    #[should_panic(expected = "is past the document's")]
    fn a_window_past_the_last_row_is_refused() {
        let _ = row_start_byte(ROWS);
    }

    /// **A suffix cut at any row start reproduces the window, and this is the measurement that
    /// deleted a precondition.**
    ///
    /// The oracle began with a hard-line search, on the reasoning that a greedy break depends on
    /// the text after it. Watched on the case it was written to refuse — [`MISCUT_ROW`], the second
    /// visual row of the first line long enough to wrap — the suffix re-wraps to
    /// [`MISCUT_ROWS_AGREE`] rows, which is exactly what the document holds from that row on,
    /// **and its first twenty rows are that row's first twenty string for string.** A greedy wrap
    /// only looks forward.
    ///
    /// Both halves are asserted, and the count first, because the count is the half a reader
    /// reaches for and the half that proves nothing: it agrees on the mis-cut suffix too.
    #[test]
    fn a_suffix_cut_at_any_row_start_reproduces_the_window() {
        let text = crate::document::text();
        let mut whole = Text::of(text.clone(), WrapKind::Words);
        let index = whole.index(W).clone();
        // A visual row whose start is **not** a hard line's: the second row of a wrapped line.
        let mut mid = None;
        for r in 1..index.rows() {
            let byte = index.row_start(r);
            if byte > 0 && text.as_bytes()[byte - 1] != b'\n' {
                mid = Some((r, byte));
                break;
            }
        }
        let (row, byte) = mid.expect("a wrapped line's second visual row");
        assert_eq!(row, MISCUT_ROW);
        let mut cut = Text::of(text[byte..].to_string(), WrapKind::Words);
        let cut_rows = cut.index(W).rows();
        assert_eq!(
            (cut_rows, index.rows() - row),
            (MISCUT_ROWS_AGREE, MISCUT_ROWS_AGREE),
            "the row count no longer agrees on a mid-line cut, which is the reading the deleted \
             precondition was written against"
        );

        // **And the rows themselves**, which is the half that decides it.
        let cut_index = cut.index(W).clone();
        let cut_text = cut.text().to_string();
        let apart = (0..20)
            .filter(|k| index.row_text(&text, row + k) != cut_index.row_text(&cut_text, *k))
            .count();
        assert_eq!(
            apart, 0,
            "a suffix cut inside a wrapped line disagrees with the window, so `row_start_byte` \
             owes the hard-line search after all and this module's oracle is not admissible at \
             every row"
        );

        // And the row start the oracle is actually cut at is a row start of the document.
        assert_eq!(row_start_byte(SCROLLED_TO), index.row_start(SCROLLED_TO));
    }

    /// **Scene 34: the window's sign, decided by an equality against the reference render.**
    ///
    /// The correct arm agrees everywhere and the inverted one is [`INVERTED_CELLS`] cells over
    /// [`INVERTED_ROWS`] rows. Both halves, because a comparison whose correct arm has never
    /// been watched agreeing reports *0 cells over 0 rows* for the same reason a broken one would.
    #[test]
    fn the_inverted_window_is_the_whole_screen_and_the_rule_agrees_with_the_tail() {
        at_rest().assert_clean("scene 34, the rule");

        let diff = inverted();
        assert_eq!(
            (diff.cells, diff.rows),
            (INVERTED_CELLS, INVERTED_ROWS),
            "the inverted window: {diff}"
        );
        // **Row 0 is the one row the refusal gets right** — `offset - 0` is `offset` — which is
        // why the rows apart are one fewer than the rectangle. Asserted as the row itself and not
        // as `H - INVERTED_ROWS == 1`, which is an equality between two derivations of one
        // declaration and cannot fail.
        assert_eq!(diff.first.map(|(_, y)| y), Some(1));
        let subject = Play::new(scrolled(), Refused::windowed(Window::Inverted))
            .played(1)
            .canvas();
        let reference = Play::new(tail(), Refused::NONE).played(1).canvas();
        assert_eq!(subject.row_text(0), reference.row_text(0), "row 0");
        assert_ne!(subject.row_text(1), reference.row_text(1), "row 1");
    }

    /// **The other half: the caret, which no equality here can see.**
    ///
    /// [`crate::document::SURFACE_BLIND`] cells apart and the caret five rows out of eighty. The
    /// two readings are asserted together, because *the surface is identical* is what makes the
    /// caret reading the only instrument there is.
    #[test]
    fn the_caret_measured_from_the_content_row_is_invisible_to_every_cell() {
        let diff = caret_at_the_content_row();
        assert_eq!(
            diff.cells,
            crate::document::SURFACE_BLIND,
            "the caret is the terminal's cursor and not a cell: {diff}"
        );

        let (rule, refusal) = misplaced_caret();
        assert_eq!(rule, Some(CARET_SCREEN_ROW), "the rule");
        assert_eq!(
            refusal, None,
            "the content row is 405 on an eighty-row screen, so the refusal places no caret at all \
             — and the screen it draws is cell for cell the correct one"
        );
    }

    /// **Scene 35: the stale tail is seventy-nine of eighty rows, and the resize spelling misses
    /// it.**
    #[test]
    fn the_stale_tail_is_seventy_nine_of_eighty_rows_and_the_resize_spelling_misses_it() {
        shrunk(Refused::NONE).assert_clean("scene 35, the rule");

        let diff = shrunk(Refused::windowed(Window::ContentRowsOnly));
        assert_eq!(
            (diff.cells, diff.rows),
            (STALE_CELLS, STALE_ROWS),
            "the stale tail: {diff}"
        );
        // **The one row it gets right is the one the shrunk content still reaches**, and it is
        // asserted as that row rather than as `STALE_ROWS == H - 1`, which restates the constant's
        // own declaration and cannot fail.
        assert_eq!(diff.first.map(|(_, y)| y), Some(1));
        assert_eq!(diff.first.map(|(_, y)| y), Some(1));

        // **And the spelling §21 refuses, as a number.** The same refusal, played over a surface
        // that has never held the megabyte, is clean — which is why the shrink axis may not be
        // gated by a resize.
        assert!(
            stale_by_resize(Refused::windowed(Window::ContentRowsOnly)).clean(),
            "the resize spelling caught the stale tail, so §21's correction has stopped being \
             about anything"
        );
        // **Clean over two surfaces that hold something**, which is the half a `clean()` owes: a
        // comparison of two blank surfaces is clean for the reason a correct one is, and this
        // module says so about `at_rest` two scenes up. The resize spelling is watched **dirty**
        // over the one refusal it can still see — the inverted window, which is wrong on the only
        // row a one-row rectangle has — so the arm is a comparison and not a pair of blanks.
        let dirty = stale_by_resize(Refused::carets(CaretRow::Content));
        assert_eq!(
            dirty.cells,
            crate::document::SURFACE_BLIND,
            "the caret is not a cell here either"
        );
        let drawn = Play::in_rows(one_line(), Refused::NONE, SHRUNK_ROWS).played(1);
        assert!(
            drawn.canvas().row_text(0).contains("one line"),
            "the resize spelling is comparing two surfaces nobody drew on"
        );
    }

    /// **The argument: the stale tail is cheaper on the two counters that move and invisible
    /// to the rest.**
    ///
    /// Not *the counters agree* — this is the axis where the refusal is measurably **better**, and
    /// the direction is the finding: a report that only printed the diff would leave the reader to
    /// take on trust that the defective build looked healthier.
    ///
    /// **`distinct` is the one worth separating**, and it was measured rather than assumed: it is
    /// 9 600 on both arms. The counter is cumulative over the play and the first two frames already
    /// touched every cell of the rectangle, so *how many cells were ever written* cannot fall when
    /// a later frame stops writing some of them — which is `crate::gates`'s own trap, **an output
    /// counter is blind to work that produces no output**, met from the other end: here the work is
    /// omitted and the output stays.
    #[test]
    fn the_stale_tail_is_cheaper_on_the_counters_that_move_and_invisible_to_the_rest() {
        let inert = Allocations::over(1, 0);
        let (rule, bad) = counters_approve(
            On::TheShrink,
            Refused::windowed(Window::ContentRowsOnly),
            inert,
            inert,
        );
        // **The partition is over `Counter::ALL` and not over a list typed here**, which is what
        // makes *cheaper on every counter that moves* a sentence with a gate under it: a counter
        // left out of both arms of a hand-written pair is a counter nobody looked at, and this
        // test's whole claim is about the counters as a population.
        let mut cheaper = Vec::new();
        let mut equal = Vec::new();
        let mut dearer = Vec::new();
        let mut unreachable = Vec::new();
        for counter in Counter::ALL {
            match (rule.get(counter).measured(), bad.get(counter).measured()) {
                (Some(a), Some(b)) if b < a => cheaper.push(counter),
                (Some(a), Some(b)) if b > a => dearer.push(counter),
                (Some(_), Some(_)) => equal.push(counter),
                _ => unreachable.push(counter),
            }
        }
        assert_eq!(
            cheaper,
            vec![Counter::Writes, Counter::Verbs],
            "the counters the stale tail moves, and it moves both of them **down**"
        );
        assert_eq!(
            dearer,
            Vec::new(),
            "a counter the stale tail makes worse would make this axis gateable by a count"
        );
        assert_eq!(
            equal,
            vec![
                Counter::Distinct,
                Counter::Regions,
                Counter::TabStops,
                Counter::Merges,
                Counter::ContentLayers,
                Counter::Allocations,
            ],
            "the counters blind to it: `distinct` because it is cumulative and the first frames \
             already touched every cell, the hit index and the tab ring and the merge count \
             because the widget is one widget either way, no overlay stands over any of it, and \
             the allocation totals are the caller's and were handed in equal"
        );
        assert_eq!(
            unreachable,
            vec![Counter::Marked],
            "`marked` and only `marked`"
        );

        // **And `distinct`'s number**, because *9 600 on both arms* was a sentence in three files
        // with nothing asserting it. It is the whole rectangle, which is the point: the counter
        // cannot fall.
        assert_eq!(
            rule.get(Counter::Distinct).measured(),
            Some(SHRUNK_DISTINCT)
        );
        assert_eq!(bad.get(Counter::Distinct).measured(), Some(SHRUNK_DISTINCT));
    }

    /// **The hover face is part of the screen**, which is a defect in the instrument and had three
    /// sentences and no gate until a review said so.
    ///
    /// An oracle drawn without the pointer on it disagrees with the subject about
    /// [`NO_POINTER_CELLS`] cells — the whole rectangle — in the paint and never in the cluster.
    /// Both halves are asserted: the count, and that **every row still reads the same text**, which
    /// is what makes it a paint difference rather than a window one.
    #[test]
    fn an_oracle_without_the_pointer_on_it_disagrees_about_the_whole_rectangle() {
        let diff = pointless_oracle();
        assert_eq!(
            (diff.cells, diff.rows),
            (NO_POINTER_CELLS, H as usize),
            "the pointer's own face: {diff}"
        );

        let subject = wheeled_play(Reveal::WhenAsked).canvas();
        let unpointed = Play::new(tail(), Refused::NONE).played(1).canvas();
        for y in 0..H {
            assert_eq!(
                subject.row_text(y),
                unpointed.row_text(y),
                "row {y} differs in its text, so this is not only the hover face"
            );
        }
        // And with the pointer on it the same comparison is clean, which is `wheeled_screen`'s.
        wheeled_screen(Reveal::WhenAsked).assert_clean("the pointed oracle");
    }

    /// **The argument: no counter separates the inverted window or the misplaced caret from
    /// the rule.**
    ///
    /// An empty answer means the equality is the only detector there is. Both refusals, because
    /// they are blind in two different senses — one changes the screen and no counter reports it,
    /// the other changes nothing a cell can hold.
    #[test]
    fn no_counter_separates_the_inverted_window_or_the_misplaced_caret() {
        let inert = Allocations::over(1, 0);
        for refused in [
            Refused::windowed(Window::Inverted),
            Refused::carets(CaretRow::Content),
        ] {
            assert_eq!(
                counters_that_separate(On::TheWindow, refused, inert, inert),
                Vec::new(),
                "a counter told {refused:?} from the rule, so this scene is optional"
            );
        }
    }

    /// **Scene 36: twenty posted notches move the window twenty, and an unconditional reveal moves
    /// it none.**
    ///
    /// Three arms and not two, because [`Reveal::Never`] moves the window exactly as far as the
    /// rule does — a gate written only on this number calls deleting the call a pass.
    #[test]
    fn twenty_posted_notches_settle_the_window_and_an_unconditional_reveal_drags_it_back() {
        assert_eq!(wheeled(Reveal::WhenAsked), WHEELED_TO, "the rule");
        assert_eq!(
            wheeled(Reveal::EveryFrame),
            DRAGGED_BACK,
            "the pull: the reader cannot get away from the caret"
        );
        assert_eq!(
            wheeled(Reveal::Never),
            WHEELED_TO,
            "deleting the call passes the loud half"
        );

        // **And the half that separates the third**: a keyboard gesture still brings the caret into
        // view under the rule and under the pull, and never under `Never`.
        assert!(revealed_by_a_key(Reveal::WhenAsked) > 0, "the rule");
        assert!(revealed_by_a_key(Reveal::EveryFrame) > 0, "the pull");
        assert_eq!(
            revealed_by_a_key(Reveal::Never),
            0,
            "the caret is off the window and stays there, which is what deleting the call costs"
        );
    }

    /// **Scene 36 on the screen, and not only on the offset.**
    ///
    /// The settled window is [`SCROLLED_TO`], which is the row the oracle is cut at — so the same
    /// reference render decides this scene and scene 34. The pull reports the whole screen.
    #[test]
    fn the_settled_screen_is_the_tail_and_the_pull_is_the_top_of_the_document() {
        wheeled_screen(Reveal::WhenAsked).assert_clean("scene 36, the rule");

        let diff = wheeled_screen(Reveal::EveryFrame);
        assert_eq!(
            (diff.cells, diff.rows),
            (PULLED_CELLS, PULLED_ROWS),
            "the pull: {diff}"
        );
    }

    /// **The thirty-five rows the pull gets away with, counted — and where they come from.**
    ///
    /// [`ALIASED_ROWS`] as a measurement rather than as an excuse, and in two independent
    /// derivations: the rows two screens four hundred apart agree on, and the rows of that screen
    /// that are a long line's **second** visual row. The two are the same set, which is what makes
    /// the explanation an explanation.
    ///
    /// It is here rather than in [`crate::document`] because it is a fact about *this comparison*
    /// and not about the document: the sentence that turned out to be about hard lines rather than
    /// visual rows has been corrected there, and this is the number that corrected it.
    #[test]
    fn the_rows_the_pull_gets_away_with_are_the_fixtures_and_not_the_defects() {
        let text = crate::document::text();
        let mut st = Text::of(text.clone(), WrapKind::Words);
        let index = st.index(W).clone();

        // The rows the top of the document and the window four hundred rows down agree on.
        let agree: Vec<usize> = (0..H as usize)
            .filter(|j| index.row_text(&text, *j) == index.row_text(&text, SCROLLED_TO + *j))
            .collect();
        assert_eq!(agree.len(), ALIASED_ROWS);
        assert_eq!(PULLED_ROWS + ALIASED_ROWS, H as usize);

        // **And every one of them is a row that carries no index**, which is the other derivation:
        // a row whose text does not start with its line's four digits is a long line's second one.
        for j in &agree {
            let row = index.row_text(&text, *j);
            assert!(
                !row.starts_with(|c: char| c.is_ascii_digit()),
                "row {j} agrees with the row four hundred below it and *does* open with an index, \
                 so the aliasing is not the second-row one this number is explained by: {row:?}"
            );
        }
        // The converse, so the set is characterised and not merely covered: no row that opens with
        // an index aliases.
        let indexed = (0..H as usize)
            .filter(|j| {
                index
                    .row_text(&text, *j)
                    .starts_with(|c: char| c.is_ascii_digit())
            })
            .count();
        assert_eq!(indexed, PULLED_ROWS);
    }

    /// **The document is the size scene 34 claims**, and the megabyte is a megabyte.
    #[test]
    fn the_document_is_the_size_the_scene_claims() {
        assert_eq!(crate::document::text().len() as u64, DOCUMENT_BYTES);
        assert!(crate::document::pasted().len() >= crate::document::PASTED);
    }

    /// **Every component a notch is posted over here is a row of the freeze that declares the
    /// wheeled axis**, which is what makes `crate::obligations`'s join over this list a query.
    ///
    /// Not `WHEELED_SUBJECTS == ["field"]`, which restates the constant's own declaration two lines
    /// up and cannot fail. Both sides here cross a file: one is `crate::document`'s subject list
    /// and the other is the freeze.
    #[test]
    fn every_component_a_notch_is_posted_over_declares_the_wheeled_axis() {
        assert_eq!(WHEELED_SUBJECTS, SUBJECTS);
        for id in WHEELED_SUBJECTS {
            let component = INVENTORY
                .iter()
                .find(|c| c.id == *id)
                .unwrap_or_else(|| panic!("`{id}` is not a row of the freeze"));
            assert!(
                component.declares(Axis::Wheeled),
                "a notch is posted over `{id}` and the freeze does not declare it wheeled, so \
                 scene 36 claims a pair for an axis nothing sets"
            );
        }
    }

    /// **Every pair these three scenes claim is an axis the freeze sets for `field`.**
    ///
    /// The other direction of `crate::scenes`'s own check, asked where the scenes are played: a
    /// scene covering an axis the freeze does not set inflates O5 silently.
    #[test]
    fn the_freeze_declares_all_three_axes_for_field() {
        let field = INVENTORY
            .iter()
            .find(|c| c.id == "field")
            .expect("`field` is a row of the freeze");
        for axis in [Axis::Scrolled, Axis::Shrunk, Axis::Wheeled] {
            assert!(
                field.declares(axis),
                "the freeze does not set `{}` on `field`",
                axis.name()
            );
        }
        assert_eq!(SUBJECTS, ["field"]);
    }

    /// **The subject is declared where the freeze homes it**, which is what makes these screens
    /// stood up rather than rehearsed.
    ///
    /// [`crate::document::subjects_declared`] opens `input.rs` and reads `pub fn field(` there, so
    /// the day the component moves this fails and the scenes' standing is a deliberate edit.
    #[test]
    fn the_three_screens_stand_on_a_declared_field() {
        assert_eq!(crate::document::subjects_declared(), SUBJECTS.to_vec());
        crate::document::assert_stands_up("production 05's three scenes");
    }
}
