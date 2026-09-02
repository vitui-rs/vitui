//! **The document: twenty fields, a pasted megabyte, and a 300 → 120 resize.**
//!
//! Components ticket 23. Spec §11, §21. This is the screen `field`'s two scenes are scenes *of*,
//! and it exists for the reason [`crate::listing`] exists one file over: **the axis was established
//! by a defect that passed every gate then in force and looked healthier than the correct build**,
//! and it was caught by an equality against a reference render and by nothing else.
//!
//! | scene | what it decides |
//! |---|---|
//! | twenty fields and a 1 MB pasted textarea | the caret pair and the index: **9 655x on one `Left`** |
//! | a 300 -> 120 resize with a wrap memo | the memo key: **625 rows drawn where 875 are needed** |
//!
//! # The narrow axis here is a cluster, not a byte
//!
//! [`crate::listing`]'s narrow axis is a row running out of columns and [`crate::dense`]'s is a
//! label; this one is **text measurement over the engine's cluster tables**, which is the axis in
//! the one shape where the wrong answer is not a shorter string but a *different* string. Every
//! number and every gate on this page runs over [`crate::clusters::corpus`] — a combining acute, two
//! marks on one base, a ZWJ family, a skin-tone modifier, `U+FE0F`, a regional-indicator pair and
//! CJK — and never over ASCII. That module carries the corpus and the reason it is a value.
//!
//! # Four gates, and *invisible on the rendered screen* turns out to mean two different things
//!
//! §11 states them together:
//!
//! > The caret is always on a cluster boundary · the caret's column equals the engine's tables over
//! > the same prefix · a spliced index equals a rebuilt one · the index's recorded width equals the
//! > width being drawn.
//!
//! and heads them **four gates, none of them visible on the rendered screen**. Measured, that
//! sentence splits in two, and the split is this ticket's finding:
//!
//! - **Gates 1 and 2 are invisible in the strong sense.** The caret is the terminal's cursor and not
//!   a cell, so a caret sitting inside a ZWJ family and a caret sitting on its boundary draw
//!   *exactly the same screen*: [`SURFACE_BLIND`] cells apart, and every counter agrees.
//! - **Gates 3 and 4 are invisible in §21's *other* sense** — the sense the whole scene list is
//!   built on. They change the screen, and **no counter of §20's nine can tell**: the memo-key
//!   defect draws [`STALE_ROWS`] of eighty rows wrong while writing the same cells, making the same
//!   verbs, declaring the same regions and running **faster**. Only an equality against a reference
//!   render reports it, which is exactly what §21 says about three of the four collection axes.
//!
//! [`counters_that_separate`] is that claim as a list rather than as a sentence, run over all four,
//! and register row 69 is where it is filed. **§11's own heading is true and the reading that makes
//! it true is not the obvious one**, which is why this paragraph is here rather than a note.
//!
//! # `graphemes()` is unreachable from this crate
//!
//! §11's mechanism is a caret *moved in cluster steps*, and nothing above `vitui-runtime` can name
//! `vitui_engine::graphemes`. [`crate::clusters::next_cluster`] reconstructs a forward step out of
//! two `truncate` probes; that module has the whole finding, and register row 68 carries it as a
//! [`Barrier`] pointing at the line in the runtime that imports what it does not re-export.
//!
//! **It is recorded, not decided.** The map is closed and this is components ticket 23's to report:
//! whether the runtime grows a cluster step is components architecture's question and
//! `.scratch/vitui-components-impl/issues/24` is the ticket that will have to ask it.
//!
//! # The gate was red for one reason and components 24 inverted it
//!
//! `field` did not exist. [`standing`] is a [`crate::obligations::Verdict`] over one subject,
//! [`subjects_declared`] opens the file the freeze homes it in — `input.rs`, F6 — and
//! [`owed_message`] is the sentence that separates *waiting for its subject* from *the code is
//! wrong*. Ticket 09's criterion 7, inherited whole.
//!
//! **All three rows turned together**, which is what the ticket predicted: there was no second
//! reason here the way the wheel gate is a second reason in [`crate::listing`]. What stood in for
//! `field` was a caret and a wrap index written in this file; both are now
//! [`crate::edit`]'s, this file re-exports them under the names it already used, and
//! [`draw_screen`] draws through [`crate::input::field_into`] — so every figure below is a
//! measurement of the shipped component.
//!
//! [`Barrier`]: crate::gates::Instrument::Barrier

use std::sync::LazyLock;
use std::time::{Duration, Instant};

use vitui_runtime::layout::text::{truncate, width, wrap};
use vitui_runtime::{Ctx, Cursor, Density};

use crate::clusters::{self, Corpus};
use crate::counters::{Allocations, Counter, Counters};
use crate::ink::Ink;
use crate::input::{FieldOpts, field_into};
use crate::obligations::Verdict;
use crate::runner::{Canvas, Diff, Pen, driver_at};
use vitui_runtime::Rect;

// ── the two screens ──────────────────────────────────────────────────────────────────────────────

/// **The width the wrap memo was built at.** §21's own 300, which is also §20's screen width.
pub const WIDE: u16 = 300;

/// **The width after the resize.** §21's own 120.
pub const NARROW: u16 = 120;

/// The viewport's height. Eighty, which is where *69 of 80* is counted.
pub const H: u16 = 80;

/// **How many fields stand on scene 12's screen. Twenty**, which is §21's own count.
///
/// Nineteen one-row inputs and one textarea, because §11's whole headline is that *`input` and
/// `textarea` are one component* — a screen of twenty textareas would be a screen about one of the
/// two spellings.
pub const FIELDS: u16 = 20;

/// **How many bytes are pasted into the textarea. §21's megabyte.**
pub const PASTED: usize = 1_048_576;

/// **How many hard lines the document holds. Six hundred and twenty-five**, which is §21's *625 rows
/// drawn*: at [`WIDE`] every line fits on one visual row.
pub const LINES: usize = 625;

/// **How many visual rows the document needs at [`NARROW`]. Eight hundred and seventy-five**, which
/// is §21's *where 875 are needed*.
pub const WRAPPED: usize = 875;

/// How many lines are longer than [`NARROW`] and no longer than [`WIDE`]. [`WRAPPED`] − [`LINES`].
///
/// Each of them is one row at 300 columns and two at 120, which is the whole of the 625/875
/// arithmetic and the reason the pair is a **construction** rather than a measurement. It is stated
/// that way rather than measured and celebrated: §21's figures are the fixture's parameters here,
/// exactly as [`crate::listing`]'s forty columns are chosen so that *71 of 80 rows* is 2 840 cells.
pub const LONG: usize = WRAPPED - LINES;

/// **How many short lines stand before the first long one. Ten.**
///
/// The one parameter chosen to reproduce §21's third figure, and it is stated as a parameter rather
/// than reported as a measurement.
///
/// The two arms agree on every row before the first line the two widths disagree about. They also
/// agree on that line's **first** row *when the line's greedy break happens to land on the band
/// edge* — a stale row is the whole line truncated at 120 columns and a correct one is the line's
/// first wrapped row, and those are the same cells exactly when the break is at 120. So the first
/// differing row is either `SHORT_HEAD` or `SHORT_HEAD + 1`, decided by one line, and `80 −` it is
/// [`STALE_ROWS`].
///
/// **Eleven was the first value tried and it measured 68 of 80**, because line 11's break lands at
/// exactly 120; ten measures 69, because line 10's falls short. Neither number is about the defect
/// — the direction, the 625/875 pair and the 6 620 cells are the same either way — and both are
/// written down here because *the figure that reproduces and the reason it might not* are the same
/// sentence.
pub const SHORT_HEAD: usize = 10;

/// **Rows of eighty the two arms of the resize disagree about. Sixty-nine**, §21's own number.
pub const STALE_ROWS: usize = 69;

/// Columns a short line occupies, at most. Half of [`NARROW`], so it never wraps at either width.
pub const SHORT_COLUMNS: u16 = 60;

/// Columns a long line occupies, at most.
///
/// Between [`NARROW`] and `2 * NARROW`, so it is exactly two rows at 120 and exactly one at 300.
/// `tests::every_long_line_is_two_rows_at_a_hundred_and_twenty_and_one_at_three_hundred` is what
/// says *exactly* rather than *about*.
pub const LONG_COLUMNS: u16 = 200;

/// **How many splices the greedy-wrap scene runs. Five hundred**, because §11's figure is
/// *499 times in 500* and running fewer is sampling the one population the ticket names.
pub const SPLICES: usize = 500;

/// **Cells the surface differs by when gate 1 or gate 2 is violated. Zero.**
///
/// The caret is the terminal's cursor and not a cell (`Ctx::caret` → `Screen::set_cursor`), so a
/// caret in the wrong place draws the same screen as a caret in the right one. This is the strong
/// reading of §11's *not visible on the rendered screen*, and it is a named constant because a gate
/// asserting `0` without saying which zero it means is the shape §21's first refinement is about.
pub const SURFACE_BLIND: usize = 0;

// ── the document ─────────────────────────────────────────────────────────────────────────────────

/// **The document's hard lines**, every one of them built out of [`crate::clusters::corpus`]'s rota.
///
/// Line `i` opens with its own four-digit index, so no two **lines** share a prefix and a wrong row
/// costs a full row of cells rather than the suffix nobody was looking at. That is
/// [`crate::runner::Fixture::lines`]'s property, restated here because this document's lines vary in
/// length and its generator's do not — and it is the property components ticket 11 found the runner
/// fixture short of, where `(5i + c) mod 26` aliases every thirteenth row.
///
/// # The unit is a hard line and not a visual row, and production ticket 05 measured the difference
///
/// [`LONG`] of these lines wrap to two rows at [`NARROW`], and **a long line's second visual row
/// carries no index**: it is a suffix of [`crate::clusters::ROTA`]'s eight-word rota, so two second
/// rows an even number of rows apart are the same string. Over an eighty-row window compared
/// against the window four hundred rows below it, [`crate::window::ALIASED_ROWS`] of the eighty
/// agree **entirely** — which is the sentence above holding for the unit it names and not for the
/// one a reader of a scrolled screen is counting in.
///
/// It is recorded rather than fixed: §21's own *625 rows drawn where 875 are needed* is a property
/// of this generator, and a generator changed to sharpen a cell count would move a normative figure
/// to win an argument. [`crate::window`] states its scenes in rows for this reason.
pub fn lines() -> Vec<String> {
    LINES_ONCE.clone()
}

/// The document, built once. A `LazyLock` and not a `fn` call per frame because
/// [`line`] verifies its own wrap count as it fills, which is `O(columns^2)` a line, and a
/// six-hundred-line document rebuilt inside a draw would price the fixture rather than the frame.
static LINES_ONCE: LazyLock<Vec<String>> = LazyLock::new(|| {
    let words: Vec<String> = clusters::ROTA.iter().map(|w| w.clusters.concat()).collect();
    (0..LINES).map(|i| line(i, target(i), &words)).collect()
});

/// Whether line `i` is one of the [`LONG`] that wrap at [`NARROW`].
pub fn is_long(i: usize) -> bool {
    (SHORT_HEAD..SHORT_HEAD + LONG).contains(&i)
}

/// The column budget line `i` is filled to.
fn target(i: usize) -> u16 {
    if is_long(i) {
        LONG_COLUMNS
    } else {
        SHORT_COLUMNS
    }
}

/// One line: its index, then rota words rotated by the index, until the next one would overflow.
///
/// **It stops on two conditions and the second is the interesting one.** The column budget is the
/// obvious stop; the other is that the line's own wrap count at [`NARROW`] may not exceed the number
/// [`is_long`] promises — one row for a short line, two for a long one. A generator that filled to a
/// column budget and *assumed* the row count is how this fixture first stood up, and it was wrong on
/// sixty-three of six hundred and twenty-five lines, because `layout::text::wrap` had a defect on a
/// row that exactly fills its band. The defect is fixed and the check stays: **a fixture that
/// asserts its own construction is the only kind whose figures mean anything**, and 625/875 is
/// exactly such a figure.
fn line(i: usize, target: u16, words: &[String]) -> String {
    let want = if is_long(i) { 2 } else { 1 };
    let mut out = format!("{i:04}");
    let mut columns = 4u16;
    let mut k = i;
    loop {
        let word = &words[k % words.len()];
        let cost = width(word) + 1;
        if columns + cost > target {
            break;
        }
        let mut candidate = String::with_capacity(out.len() + word.len() + 1);
        candidate.push_str(&out);
        candidate.push(' ');
        candidate.push_str(word);
        if wrap(&candidate, NARROW).count() > want {
            break;
        }
        out = candidate;
        columns += cost;
        k += 1;
    }
    out
}

/// The document as one buffer, which is what a textarea holds. Lines joined by `\n`.
pub fn text() -> String {
    lines().join("\n")
}

/// **A pasted megabyte**, built by repeating the document until it is at least [`PASTED`] bytes.
///
/// A megabyte of one repeated character would answer a different question: the cost §11 prices is
/// *segmenting forward*, and segmentation over one-byte clusters is the cheap case.
pub fn pasted() -> String {
    let one = text();
    let mut out = String::with_capacity(PASTED + one.len());
    while out.len() < PASTED {
        out.push_str(&one);
        out.push('\n');
    }
    out
}

// ── the wrap index and the caret, which are the component's ──────────────────────────────────────

/// **The machine these screens are screens of.** Components ticket 24.
///
/// Ticket 23 wrote the wrap index and the caret pair *here*, beside the gate, because `field` did
/// not exist — and said so in as many words: *what stands in for `field` is a caret and a wrap
/// index written in this file*. Ticket 24 built the component, so they moved into
/// [`crate::edit`] and this module re-exports them under the names its own tests, its scenes and
/// `examples/field_numbers.rs` already used. **Every figure on this page is now a measurement of
/// the shipped machine**, which is the difference between a scene that is stood up and one that is
/// merely green.
pub use crate::edit::{Caret, Index, Text, WrapKind, boundaries, step, step_left, step_right};

/// **What one `Left` at the end of a pasted megabyte costs, both ways.**
///
/// §21's scene 12 in one measurement: *the caret pair and the index — 9 655x on one `Left`*. The
/// index is built **outside** the timing on both arms, because §11's argument is that a `textarea`
/// **has that index already** — timing its construction here would price the wrapping against the
/// caret and answer a question nobody asked.
///
/// A **report**, and gated nowhere: R15's rule, and the caution is earned on this page, because the
/// defective arm of every other scene here is *faster*.
pub fn left_at_the_end(text: &str, index: &Index) -> (Duration, Duration, Caret) {
    // The end of a buffer is a boundary in every document, which is why the pair can be written
    // here at all — `Caret::of` is crate-private and this is the one offset that needs no gesture.
    let end = Caret::of(text.len(), 0);

    let started = Instant::now();
    let without = crate::edit::defective::blind_left(text, end);
    let blind = started.elapsed();

    // **A `Left` at a row start is answered from the row before**, and the index is what makes
    // that one subtraction rather than a search. A pasted megabyte ends in a line break, so the
    // caret's own row starts exactly where the caret is and segmenting forward from there crosses
    // nothing at all — an arm that took `row_of(caret)` unconditionally would answer *the caret did
    // not move*, which looks like a correct `Left` at the end of a document and is not one.
    let row = index.row_of(end.byte());
    let row = if index.row_start(row) >= end.byte() && row > 0 {
        row - 1
    } else {
        row
    };
    let start = index.row_start(row);
    let started = Instant::now();
    let with = step_left(text, end, Caret::of(start, 0));
    let indexed = started.elapsed();

    assert_eq!(
        without.byte(),
        with.byte(),
        "the two arms disagree about where `Left` lands, which is a defect in the instrument"
    );
    (blind, indexed, with)
}

// ── the char caret, as a negative case ───────────────────────────────────────────────────────────

/// **What a `char` caret does to the corpus.** §11's other failure, and it is *faster per step*.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CharWalk {
    /// How many steps a `char` caret takes to cross the corpus.
    pub steps: usize,
    /// How many clusters it crossed.
    pub clusters: usize,
    /// **How many of those steps land inside a cluster.** `steps − clusters`, arithmetic.
    pub inside: usize,
    /// **How many insertions at those positions change the buffer's display width by something
    /// other than what was typed** — the cluster came apart.
    pub width_surprises: usize,
}

/// **Walk the corpus with a `char` caret and count what it breaks.**
///
/// The insertion is one `x`, one column wide. At a cluster boundary the buffer gets exactly one
/// column wider; inside a cluster it does not, and the difference is not a rounding error — a `x`
/// dropped between a base and its combining mark makes two clusters out of one, and a `x` dropped
/// inside a ZWJ family makes as many as five.
pub fn char_walk(corpus: &Corpus) -> CharWalk {
    let text = corpus.text();
    let before = width(text);
    let typed = width("x");
    let mut surprises = 0usize;
    let mut steps = 0usize;

    for (byte, ch) in text.char_indices() {
        let at = byte + ch.len_utf8();
        steps += 1;
        let mut edited = String::with_capacity(text.len() + 1);
        edited.push_str(&text[..at]);
        edited.push('x');
        edited.push_str(&text[at..]);
        if width(&edited) != before + typed {
            surprises += 1;
        }
    }

    CharWalk {
        steps,
        clusters: corpus.len(),
        inside: steps - corpus.len(),
        width_surprises: surprises,
    }
}

// ── the greedy wrap's restart point ──────────────────────────────────────────────────────────────

/// **What five hundred splices said about the restart point.**
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Splices {
    /// How many edits were played. [`SPLICES`].
    pub trials: usize,
    /// How many of them a splice restarted at `row_of(at)` got **wrong**.
    pub at_row_of: usize,
    /// How many of them a splice restarted one row earlier got wrong.
    pub one_row_earlier: usize,
}

/// The document the splice sweep runs over. Short enough that five hundred rebuilds are a test.
///
/// Sixty-four lines of the same rota at forty columns, which is where a greedy break has somewhere
/// to move to: at [`WIDE`] a line is one row and there is no break to invalidate.
pub(crate) fn splice_document() -> String {
    let all = lines();
    all[..64].join("\n")
}

/// The width the splice sweep wraps at. Forty, for [`splice_document`]'s reason.
pub(crate) const SPLICE_W: u16 = 40;

/// A deterministic step, so that the five hundred are the same five hundred every run.
///
/// A test whose population changes between runs reports a different number each time and can only
/// be believed once. This is xorshift64 with a fixed seed and no dependency.
fn shuffle(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

/// **Five hundred edits, each spliced two ways and compared against a rebuild.**
///
/// §11: *a splice restarted at `row_of(at)` therefore keeps a break the edit invalidated: it agreed
/// with a rebuild 499 times in 500 and the screen looked right. Restarting one row earlier is
/// provably enough, and the defect was caught by an equality against a rebuild and by nothing else.*
///
/// The edits include a **space**, and that is not decoration: the case that moves a break backwards
/// is a word too long for the previous row acquiring a break opportunity inside it, so an edit set
/// with no whitespace in it cannot reach the defect at all.
pub fn splice_sweep() -> Splices {
    let (splices, _) = splice_sweep_with_witness();
    splices
}

/// The five hundred edits, in order. `(at, inserted)` a trial.
///
/// **Deterministic and stated once**, so that [`splice_sweep`] and [`first_naive_failure`] play the
/// same population rather than two that happen to have been written the same way.
fn splice_trials(base: &str) -> Vec<(usize, &'static str)> {
    let inserts = [" ", "x", "e\u{301}", "\u{6F22}", " the "];
    // A document-aware boundary walk. `crate::clusters::boundaries_by_probe` is a *line*
    // instrument — see [`step`] — and this text has newlines in it.
    let boundaries = boundaries(base);
    let mut state = 0x5EED_1234_ABCD_0001u64;
    (0..SPLICES)
        .map(|trial| {
            (
                boundaries[(shuffle(&mut state) as usize) % boundaries.len()],
                inserts[trial % inserts.len()],
            )
        })
        .collect()
}

/// The edited buffer for one trial.
fn edit(base: &str, at: usize, inserted: &str) -> String {
    let mut edited = String::with_capacity(base.len() + inserted.len());
    edited.push_str(&base[..at]);
    edited.push_str(inserted);
    edited.push_str(&base[at..]);
    edited
}

/// **The same five hundred edits, played through [`crate::edit::Text`] rather than through
/// [`Index::spliced`].**
///
/// [`splice_sweep`] asks the *mechanism*: it holds one index and splices it two ways at a restart
/// point the test computes. This asks the **component**: five hundred states, each seated by a
/// gesture and edited with `Text::insert`, and the restart point is the one `Text::edit` chose —
/// [`crate::edit::defective::restarted_at_row_of`] is the only difference between the two arms.
///
/// It exists because row 17 is an equality about the shipped splice and the cheaper sweep is an
/// equality about a splice the test drove. Both populations are [`SPLICES`], so the two numbers are
/// comparable; the component's arm is the one the register cites first.
///
/// The caret is walked to the trial's offset with `Text::right` from the start of its own row,
/// because there is no expression for *put the caret at byte N* (§11) and a gate may not invent
/// one — [`edit_walk`]'s [`Seat::AtByte`] is the door, and it is a defect rather than a fixture.
pub fn component_splice_sweep() -> Splices {
    let base = splice_document();
    let mut out = Splices {
        trials: 0,
        at_row_of: 0,
        one_row_earlier: 0,
    };

    for (at, inserted) in splice_trials(&base) {
        out.trials += 1;
        for naive in [false, true] {
            let mut st = Text::of(base.clone(), WrapKind::Words);
            if naive {
                crate::edit::defective::restarted_at_row_of(&mut st);
            }
            let row = st.index(SPLICE_W).row_of(at);
            st.click(SPLICE_W, row, 0, false);
            while st.caret().byte() < at {
                st.right(SPLICE_W, false);
            }
            st.insert(SPLICE_W, inserted);
            let rebuilt = Index::build(st.text(), SPLICE_W, st.revision());
            if !st.indexed().expect("the edit left one").same_rows(&rebuilt) {
                match naive {
                    true => out.at_row_of += 1,
                    false => out.one_row_earlier += 1,
                }
            }
        }
    }

    out
}

/// [`splice_sweep`], and the first trial the naive restart point got wrong beside it.
///
/// The witness is what the screen next door is drawn from, so the surface figure and the sweep's
/// count are about **the same edit** rather than about two edits chosen separately.
pub fn splice_sweep_with_witness() -> (Splices, Option<(String, usize)>) {
    let base = splice_document();
    let index = Index::build(&base, SPLICE_W, 0);

    let mut at_row_of = 0usize;
    let mut one_row_earlier = 0usize;
    let mut witness = None;

    for (at, inserted) in splice_trials(&base) {
        let edited = edit(&base, at, inserted);
        let rebuilt = Index::build(&edited, SPLICE_W, 1);
        let row = index.row_of(at);
        if !index.spliced(&edited, row, 1).same_rows(&rebuilt) {
            at_row_of += 1;
            if witness.is_none() {
                witness = Some((edited.clone(), at));
            }
        }
        if !index
            .spliced(&edited, row.saturating_sub(1), 1)
            .same_rows(&rebuilt)
        {
            one_row_earlier += 1;
        }
    }

    (
        Splices {
            trials: SPLICES,
            at_row_of,
            one_row_earlier,
        },
        witness,
    )
}

/// **The first of the five hundred edits a splice restarted at `row_of(at)` gets wrong.**
///
/// `(edited, at)`, or `None` if the population cannot reach the defect — which is itself a result
/// and is asserted rather than tolerated.
pub fn first_naive_failure() -> Option<(String, usize)> {
    splice_sweep_with_witness().1
}

/// **One edit constructed to move the break that closes the previous row**, so that the two restart
/// points can be watched disagreeing without waiting for a sweep to find one.
///
/// Returns `(base, edited, at)`. The line is one long word — longer than the band — preceded by a
/// short one, so the greedy wrap closes row 0 early; a space inserted inside the long word gives
/// row 0 something that fits, and the break moves.
pub fn hostile_splice() -> (String, String, usize) {
    let base = "ab cdefghijklmnopqrstuvwxyz".to_string();
    // Inside the long word, far enough in that the fragment before the space still fits row 0.
    let at = "ab cdefghij".len();
    let mut edited = String::with_capacity(base.len() + 1);
    edited.push_str(&base[..at]);
    edited.push(' ');
    edited.push_str(&base[at..]);
    (base, edited, at)
}

/// The width [`hostile_splice`] is wrapped at. Twelve columns: `ab` fits, the long word does not.
pub const HOSTILE_W: u16 = 12;

// ── gate 1, across an edit ───────────────────────────────────────────────────────────────────────

/// **Positions [`edit_walk`] seats a caret at.** Every ninety-seventh `char` boundary of
/// `splice_document`, which is a stride and not a sample of one row.
pub const WALK_STRIDE: usize = 97;

/// **How many positions that stride reaches. A hundred and forty-four.**
///
/// Named because a stride is a construction and the count it produces is a fact about the fixture:
/// a `splice_document` that shrank would leave the script running over three seats and every
/// relation below still true.
pub const WALK_SEATS: usize = 144;

/// **How many of those hundred and forty-four are inside a cluster. Forty-five.**
///
/// The stride walks `char` boundaries, and this document is built out of
/// [`crate::clusters::corpus`]'s rota — so forty-five of the positions a byte-addressed caret would
/// accept are places no gesture can put one. It is the *population* [`Seat::AtByte`] can be wrong
/// at, which is what turns its off-boundary count into an equality rather than a threshold.
pub const WALK_INSIDE: usize = 45;

/// **How a caret is seated between the edits of [`edit_walk`]** — one function, two arms, one value
/// between them, which is this file's rule for a defect (ADR 0026).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Seat {
    /// **The shipped verbs.** A caret is placed by a click's column, a cluster step, `Home` or
    /// `End`, and every one of those lands on a boundary by construction.
    Gesture,
    /// **§11's deleted API, spelled through the one door that exists**:
    /// [`crate::edit::defective::at_byte`] handed to [`Text::set_pos`]. The caret it seats is a
    /// boundary only by luck.
    ///
    /// Its *column* is the engine's tables over the **whole prefix**, and
    /// `crate::edit::defective::at_byte`'s own doc calls that right on gate 2 — which it is on the
    /// one-line corpus that doc was written against and is not here. A caret's column is a *screen*
    /// column, re-seated at every row start, so over a sixty-four-line document a full-prefix column
    /// is a screen column on row 0 and nowhere else: this arm is wrong on gate 1 at the seats inside
    /// a cluster and wrong on gate 2 at every seat but one. The two gates are still separable, and
    /// the arm that shows it is a `Text::edit` that seats a byte-addressed caret after every edit —
    /// which puts 715 columns wrong and leaves gate 1 at zero.
    AtByte,
}

/// **Which verb placed the caret being inspected**, because gate 1 can only fail on two of the
/// three and that is the finding rather than a bookkeeping detail.
///
/// A review of production ticket 04 found the first version of this walk counting one number over
/// all three, and reading a zero as *the shipped verbs are right across an edit* when for one of
/// the three it could not have been anything else.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Placed {
    /// **A gesture, or the seat.** `select_all`, a click, or — on the defective arm — a raw byte
    /// through [`Text::set_pos`]. This is where a caret enters the state from outside, so it is the
    /// only class an *off-boundary* caret can be introduced in.
    Seated,
    /// **`Text::edit`**, which re-seats the pair by walking cluster steps from the start of the
    /// new visual row.
    ///
    /// **Gate 1 cannot fail here and the reason is structural**, not empirical: the walk returns a
    /// member of its own walk, and a checker that walks from byte 0 is no more independent — it
    /// passes through the row start (which is a boundary, and pinned as one) and continues with the
    /// same steps. What *can* fail here is gate 2, and it does: a `Text::edit` that seats a
    /// byte-addressed caret leaves 715 wrong columns and zero off-boundary carets.
    Walked,
    /// **`Text::undo`**, which **restores** the pair rather than recomputing it — §11's `set_pos`
    /// at 0.0007 µs against `set_caret`'s 6 109.
    ///
    /// **This is where gate 1 has teeth across an edit.** A restored pair is only as good as the
    /// buffer it is restored into: it was a boundary in the buffer the edit started from, and the
    /// claim is that undo puts that buffer back. The defective arm demonstrates it without patching
    /// `undo` at all — the third undo of each seat restores the seat's own pair, which on the
    /// `AtByte` arm is inside a cluster.
    Restored,
}

/// What [`edit_walk`] counted.
///
/// The failure columns are counted rather than asserted inside the walk, because a gate that stops
/// at the first bad caret cannot say whether the arm is wrong once or wrong everywhere — and *once
/// in five hundred* is the shape §11 already met on the splice.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct EditWalk {
    /// Carets inspected: every gesture, every edit and every undo of the script, at every position.
    pub inspected: usize,
    /// Of those, how many sat somewhere that is **not** a cluster boundary of the buffer as it then
    /// stood. Recomputed against the edited buffer, never against the original.
    pub off_boundary: usize,
    /// The [`Placed::Seated`] share of [`EditWalk::off_boundary`].
    pub off_boundary_seated: usize,
    /// The [`Placed::Walked`] share, which is **structurally** zero — see [`Placed::Walked`].
    pub off_boundary_walked: usize,
    /// The [`Placed::Restored`] share, which is where gate 1 has teeth across an edit.
    pub off_boundary_restored: usize,
    /// Of those, how many carried a column the engine's tables disagree with over the caret's own
    /// visual row.
    pub wrong_column: usize,
    /// Edits applied — an insert, a backspace and a delete at every position.
    pub edits: usize,
    /// Positions a caret was seated at.
    pub seats: usize,
    /// Of those, how many are inside a cluster of the unedited buffer — the population
    /// [`Seat::AtByte`] can be wrong on gate 1 at, and the number that makes its counts an equality
    /// rather than a threshold.
    pub inside: usize,
    /// **Times an end-of-buffer inspection found the caret at the end it asked for**, counted at
    /// both ends and **never from a fresh state**: a `Text` is constructed with its caret already at
    /// byte 0, so a `Home` counted first is a comparison that cannot fail. `select_all` reaches the
    /// buffer's length, and a click on row 0 column 0 comes back from there — neither end is
    /// `Text::home` or `Text::end`, both of which are the *visual row's* ends.
    pub ends: usize,
    /// **Undos that applied an entry**, which is `Text::undo`'s own answer and not a count of calls.
    /// Three per seat, which is the whole history: an empty ring would make a seat's last three
    /// inspections copies of the one before them, and the third undo is the one that restores the
    /// seat's own pair.
    pub undone: usize,
}

/// **Whether `byte` is a cluster boundary of `buf`, walked from the start of the visual row it
/// falls in** rather than from byte 0.
///
/// The row start is where the walk may begin because the break points are the engine's own:
/// [`Index::build`] wraps with `vitui_runtime::layout::text::wrap`, whose breaks fall on cluster
/// boundaries, so a walk from a row start reaches the same set over that row as a walk from byte 0.
/// [`tests::every_row_start_is_a_cluster_boundary_of_the_whole_buffer`] is that assumption pinned
/// rather than assumed, and it is the reason this is `O(row)`: the same check written as
/// `boundaries(buf).contains(&byte)` is `O(buffer)` at every one of [`edit_walk`]'s inspections and
/// costs twenty seconds against three.
fn on_a_boundary(buf: &str, row_start: usize, byte: usize) -> bool {
    let mut at = row_start;
    while at < byte {
        match step(&buf[at..]) {
            Some(cluster) => at += cluster.len(),
            None => break,
        }
    }
    at == byte
}

/// **Gate 1 and gate 2 over a script of edits rather than over a construction.**
///
/// The two caret gates were watched over a forward walk of the corpus and over the gestures, and
/// both of those inspect a caret the buffer has not moved under. **An edit moves it**: `Text::edit`
/// re-seats the pair by walking from the start of the *new* visual row, and nothing asked whether
/// that landed on a boundary. This walks the script at every [`WALK_STRIDE`]th `char` boundary of
/// `splice_document` and inspects the caret at both ends of the buffer, after the seat, and after
/// each of an insert, a backspace, a delete and two undos.
///
/// The insert is a base and a combining acute — two `char`s and one cluster — because an insert of
/// one ASCII byte cannot produce a cluster the buffer did not already have.
pub fn edit_walk(seat: Seat) -> EditWalk {
    let base = splice_document();
    let w = SPLICE_W;
    let mut out = EditWalk::default();

    fn inspect(st: &mut Text, w: u16, by: Placed, out: &mut EditWalk) {
        let caret = st.caret();
        let buf = st.text().to_string();
        let row_start = {
            let index = st.index(w);
            index.row_start(index.row_of(caret.byte()))
        };
        out.inspected += 1;
        let off = |by: Placed, out: &mut EditWalk| {
            out.off_boundary += 1;
            match by {
                Placed::Seated => out.off_boundary_seated += 1,
                Placed::Walked => out.off_boundary_walked += 1,
                Placed::Restored => out.off_boundary_restored += 1,
            }
        };

        // **A caret can be off a `char` boundary and not only off a cluster boundary, and the
        // instrument may not fall over on it.** `&buf[..b]` panics inside a code point, so the
        // width comparison below would take this gate down with a slicing message instead of
        // reporting the property it exists for — which is what happened when `Text::undo` was
        // patched to restore the wrong buffer: a pair restored into a shorter buffer landed inside
        // a four-byte emoji. `str::is_char_boundary` is false past the end too, so one test covers
        // a stale pair as well. Such a caret is off a cluster boundary by definition and has no
        // column the engine's tables could agree with, so it counts on both.
        if !buf.is_char_boundary(caret.byte()) {
            off(by, out);
            out.wrong_column += 1;
            return;
        }
        if !on_a_boundary(&buf, row_start, caret.byte()) {
            off(by, out);
        }
        if caret.col() != width(&buf[row_start..caret.byte()]) {
            out.wrong_column += 1;
        }
    }

    let seats: Vec<usize> = (0..base.len())
        .step_by(WALK_STRIDE)
        .filter(|at| base.is_char_boundary(*at))
        .collect();

    let cluster_boundaries = boundaries(&base);
    out.seats = seats.len();
    out.inside = seats
        .iter()
        .filter(|at| !cluster_boundaries.contains(at))
        .count();

    for at in seats {
        let mut st = Text::of(base.clone(), WrapKind::Words);

        // **Both ends of the buffer, and neither of them is `Home` or `End`.**
        //
        // Two traps, both of which a first version of this walk fell into. `Text::home` and
        // `Text::end` are the ends of the caret's own **visual row**, so on row 0 neither is an end
        // of a wrapped document; the two gestures that address the buffer are `select_all`, which
        // clicks the last row at `u16::MAX`, and a click on row 0 column 0. And a `Text` is
        // constructed with its caret already at byte 0 — a `Home` counted first is a comparison
        // between a fresh state and where a fresh state already is, so **the far end is reached
        // first** and byte 0 is the one that has to be come back to.
        st.select_all(w);
        out.ends += usize::from(st.caret().byte() == base.len());
        inspect(&mut st, w, Placed::Seated, &mut out);
        // A click settles the anchor, so this also clears the selection `select_all` left — without
        // it the insert below would replace the whole document and the script would be one edit.
        st.click(w, 0, 0, false);
        out.ends += usize::from(st.caret().byte() == 0);
        inspect(&mut st, w, Placed::Seated, &mut out);

        match seat {
            Seat::Gesture => {
                let (row, col) = {
                    let index = st.index(w);
                    let row = index.row_of(at);
                    (row, width(&base[index.row_start(row)..at]))
                };
                st.click(w, row, col, false);
            }
            Seat::AtByte => st.set_pos(crate::edit::defective::at_byte(&base, at)),
        }
        inspect(&mut st, w, Placed::Seated, &mut out);

        st.insert(w, "e\u{301}");
        out.edits += 1;
        inspect(&mut st, w, Placed::Walked, &mut out);

        st.backspace(w);
        out.edits += 1;
        inspect(&mut st, w, Placed::Walked, &mut out);

        st.delete(w);
        out.edits += 1;
        inspect(&mut st, w, Placed::Walked, &mut out);

        // **Three undos and not two: the whole history.** The third restores the *seat's* own pair
        // into the buffer the seat was made in, which is the one inspection in this script where a
        // caret enters from outside a cluster walk **after** an edit — and on the `AtByte` arm it is
        // inside a cluster.
        for _ in 0..3 {
            out.undone += usize::from(st.undo());
            inspect(&mut st, w, Placed::Restored, &mut out);
        }
    }

    out
}

// ── the four defects, expressed ──────────────────────────────────────────────────────────────────

/// **One of §11's four gates, violated on purpose.**
///
/// Every arm is a build somebody would ship: each is simpler than the correct one, and three of the
/// four are cheaper.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Defect {
    /// **Gate 1.** The caret is placed at a byte offset that is not a cluster boundary — §11's
    /// *there is no such thing as "put the caret at byte N"*, written as the API that has one.
    CaretOffBoundary,
    /// **Gate 2.** The column is `chars().count()` over the prefix rather than the engine's tables:
    /// one column a code point. Right on ASCII, wrong on every cluster in the corpus.
    CaretColumnByChars,
    /// **Gate 3.** The wrap index is spliced from `row_of(at)` rather than one row earlier.
    SpliceRestartedAtRowOf,
    /// **Gate 4.** The wrap memo is keyed on the revision and not on `(revision, width)`, so the
    /// index built at [`WIDE`] is reused at [`NARROW`].
    MemoKeyedOnRevision,
}

impl Defect {
    /// Every defect, in §11's order.
    pub const ALL: [Defect; 4] = [
        Defect::CaretOffBoundary,
        Defect::CaretColumnByChars,
        Defect::SpliceRestartedAtRowOf,
        Defect::MemoKeyedOnRevision,
    ];

    /// The gate it violates, in §11's own words.
    pub fn gate(self) -> &'static str {
        match self {
            Defect::CaretOffBoundary => "the caret is always on a cluster boundary",
            Defect::CaretColumnByChars => {
                "the caret's column equals the engine's tables over the same prefix"
            }
            Defect::SpliceRestartedAtRowOf => "a spliced index equals a rebuilt one",
            Defect::MemoKeyedOnRevision => {
                "the index's recorded width equals the width being drawn"
            }
        }
    }

    /// **Whether the defect changes the rendered surface at all.**
    ///
    /// Two of the four do not, and two do. See this module's header: §11's *none of them visible on
    /// the rendered screen* is true in two different senses and this is the field that separates
    /// them.
    pub fn changes_the_surface(self) -> bool {
        matches!(
            self,
            Defect::SpliceRestartedAtRowOf | Defect::MemoKeyedOnRevision
        )
    }
}

// ── the screen ───────────────────────────────────────────────────────────────────────────────────

/// **One frame of the document screen**, with everything a gate can ask about it.
pub struct Played {
    canvas: Canvas,
    counters: Counters,
    caret: Option<Cursor>,
    rows_drawn: usize,
}

impl Played {
    /// What was drawn.
    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    /// The nine per-frame counters, of which this crate can read eight.
    pub fn counters(&self) -> &Counters {
        &self.counters
    }

    /// **Where the caret went**, as `settle` will hand it to the terminal. Not a cell.
    pub fn caret(&self) -> Option<Cursor> {
        self.caret
    }

    /// How many visual rows the textarea's index believed it had.
    pub fn rows_drawn(&self) -> usize {
        self.rows_drawn
    }
}

/// **Where the caret sits, and it is deliberately not the first cluster of the document.**
///
/// The seat is the boundary before the **second** cluster of more than one code point, which is
/// what makes all three spellings of the column different numbers:
///
/// - the correct column is the engine's tables over the prefix;
/// - the off-boundary column is the tables over one byte more, which lands inside the cluster;
/// - the code-point column is the prefix's `chars().count()`, which is right only while the prefix
///   is ASCII — and a prefix ending at the *first* multi-code-point cluster still is.
///
/// A seat on the first one would leave gate 2 with a defective arm that agrees with the correct
/// one, which is a gate that passes because nothing happened. Only clusters whose first code point
/// is one byte qualify, so that the off-boundary offset is a `char` boundary and a `&str` slice of
/// it does not panic — the caret is wrong, and the instrument is still allowed to be well-formed.
fn caret_seat(corpus: &Corpus) -> usize {
    let mut seen = 0;
    for (i, cluster) in corpus.clusters().iter().enumerate() {
        let head = cluster.chars().next().map(char::len_utf8).unwrap_or(0);
        if cluster.chars().count() > 1 && head == 1 {
            seen += 1;
            if seen == 2 {
                return corpus.boundaries()[i];
            }
        }
    }
    0
}

/// **One arm of one scene: a buffer, an index over it, and where the caret goes.**
///
/// The pair [`screens`] returns is the whole of a scene, and a `Screen` holds its index rather than
/// building one inside the draw for a reason ADR 0026 states one file over: **the correct build and
/// the defective one must be one function with one value between them**, so that a reviewer's diff
/// is the value and not a second drawing path.
pub struct Screen {
    /// The rectangle's width. The height is always [`H`].
    pub w: u16,
    /// **How many one-row inputs stand above the textarea.**
    ///
    /// Nineteen for §21's scene 12 — *twenty fields and a 1 MB pasted textarea* — and **zero** for
    /// scene 13, which is one field: `Content::Fields { fields: 1, bytes: 1 048 576 }`. The two
    /// scenes are two screens and the difference is not decoration: nineteen inputs leave the
    /// textarea sixty-one rows, and §21's *69 of 80* is counted over eighty.
    pub inputs: u16,
    /// **The textarea's state, which is [`crate::edit::Text`] and therefore the component's.**
    ///
    /// Ticket 23 carried a `String`, an `Index` and a caret column here, because there was nothing
    /// to hold them; the four defects were three separate substitutions in this struct. They are
    /// now three settings on one shipped state — [`crate::edit::defective::at_byte`] and
    /// `column_by_chars` for the caret pair, `keyed_on_revision` for the memo key,
    /// `restarted_at_row_of` for the splice — and the screen is drawn by
    /// [`crate::input::field_into`] either way.
    pub st: crate::edit::Text,
}

/// **The two arms of one gate: correct, then defective.**
///
/// Each defect is expressed on the screen where it is *reachable*, which is not one screen for all
/// four. The two caret gates and the memo key live on the document; the splice lives on the sixty-
/// four-line buffer [`splice_sweep`] plays over, at the first of its five hundred edits the naive
/// restart point gets wrong. **Using the sweep's own witness is the point**: the surface figure and
/// the count are then about one edit rather than about two chosen separately.
///
/// # Panics
///
/// Panics when the five hundred edits reach no naive-splice failure at all. That is not a fixture
/// that needs relaxing — it is the population failing to contain the defect, which is
/// [`crate::obligations::Verdict::of`]'s vacuity refusal arriving on a screen.
pub fn screens(defect: Defect) -> (Screen, Screen) {
    use crate::edit::{Text, WrapKind, defective as bad};

    match defect {
        Defect::CaretOffBoundary | Defect::CaretColumnByChars => {
            let corpus = clusters::corpus();
            let seat = caret_seat(&corpus);
            // **The caret is seated in the corpus and the buffer is the corpus**, so the boundary
            // the seat names is a boundary of the text being drawn. Ticket 23 seated it in the
            // corpus and drew the document, which was honest while the caret was a column handed
            // to `Ctx::caret` and is not once the caret is a **pair inside a buffer**.
            let text = corpus.text().to_string();
            let build = |caret| {
                let mut st = Text::of(text.clone(), WrapKind::Words);
                let _ = st.index(WIDE);
                st.set_pos(caret);
                Screen {
                    w: WIDE,
                    inputs: FIELDS - 1,
                    st,
                }
            };
            let correct = build(bad::at_byte(corpus.text(), seat));
            let broken = build(match defect {
                Defect::CaretOffBoundary => bad::at_byte(corpus.text(), seat + 1),
                _ => bad::column_by_chars(corpus.text(), seat),
            });
            (correct, broken)
        }
        Defect::MemoKeyedOnRevision => {
            let text = text();
            let build = |stale: bool| {
                let mut st = Text::of(text.clone(), WrapKind::Words);
                if stale {
                    bad::keyed_on_revision(&mut st);
                }
                // **The index is built at [`WIDE`] and the screen is drawn at [`NARROW`]**, which
                // is the resize: the correct key misses and rebuilds, and the defective one hits.
                let _ = st.index(WIDE);
                Screen {
                    w: NARROW,
                    // **One field**, which is §21's own content for this scene, and it is what
                    // makes *69 of 80* a count over eighty rows rather than over sixty-one.
                    inputs: 0,
                    st,
                }
            };
            (build(false), build(true))
        }
        Defect::SpliceRestartedAtRowOf => {
            let base = splice_document();
            let (edited, at) = first_naive_failure().expect(
                "five hundred edits reached no naive-splice failure, so this population cannot see \
                 the defect at all",
            );
            let inserted = &edited[at..at + 1];
            let build = |naive: bool| {
                let mut st = crate::edit::Text::of(base.clone(), WrapKind::Words);
                if naive {
                    bad::restarted_at_row_of(&mut st);
                }
                let _ = st.index(SPLICE_W);
                // **The edit is played through the component's own `edit`**, so the two arms differ
                // by the restart point and by nothing else — the buffer they end at is identical.
                let row = st.index(SPLICE_W).row_of(at);
                st.click(SPLICE_W, row, 0, false);
                while st.caret().byte() < at {
                    st.right(SPLICE_W, false);
                }
                st.insert(SPLICE_W, inserted);
                // **The screen is scrolled to the witness, and that is not a convenience.**
                // Sixty-four lines at forty columns is three hundred-odd visual rows and the defect
                // is one of them; a screen that always started at row 0 would score this defect
                // clean **by not looking at it**, which is `Verdict::of`'s vacuity refusal wearing
                // a viewport.
                st.scroll_to((row - 1).saturating_sub(4));
                Screen {
                    w: SPLICE_W,
                    inputs: 0,
                    st,
                }
            };
            (build(false), build(true))
        }
    }
}

/// **Draw one screen: nineteen inputs, one textarea, and a caret.**
///
/// Generic over [`Ink`] so a [`Tally`](crate::counters::Tally) and a [`Pen`] measure the same
/// drawing path rather than a
/// copy of it — [`crate::ink`]'s whole argument.
///
/// # One verb a row, and the two-verb form was measured and refused
///
/// A row is padded to the rectangle's width and written with one `Ctx::text`, the way
/// [`crate::runner::rows_at_a_time`] writes one. The obvious alternative — write the row, then
/// [`Ink::run`] a padding band — is what [`crate::listing`] does and is one allocation cheaper, and
/// it **makes `verbs` separate the two arms of the memo key**: `Ink::run` returns without a verb at
/// a count of zero, so a row that exactly fills its width costs one verb and a row that does not
/// costs two. The stale arm's rows are the *whole line truncated*, so they fill exactly and the
/// correct arm's do not — and `verbs` then reports the defective build as **cheaper**, which is a
/// counter separating the arms in the direction that approves the defect. One verb a row removes it.
///
/// # A textarea that does not seat the focus places no caret at all
///
/// `Ctx::caret` is refused when nothing holds the keyboard — `settle_caret` clears it — so the two
/// caret gates are unaskable until something is focused. This is architecture issue 25's finding
/// arriving on `field`'s own screen: the seating is the component's, written as
/// `if cx.focused().is_none()` rather than as `if !cx.is_focused(id)`, because the second drags the
/// keyboard back the moment the user tabs away.
pub fn draw_screen<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, screen: &mut Screen) -> usize {
    let area = cx.area();
    let w = area.w;
    let corpus = clusters::corpus();
    let head = truncate(corpus.text(), w).to_string();

    // **The one-row inputs across the top, each a shipped `field`.** One call site and a key a row,
    // which is §4's rule: `Ctx::id` mints from `Location::caller()`, so a loop with no key is one
    // widget and the nineteen would merge into one hit entry on a screen that looks correct.
    //
    // **Rectangles and not `Ctx::child`**, which is `CONTEXT.md`'s identity rule read the other
    // way: a container that returns a rectangle preserves its children's identity and one that
    // takes a closure renames them. Nothing here needs a narrower clip, so nothing here narrows.
    let inputs = screen.inputs;
    let opts = FieldOpts::default();
    for y in 0..inputs {
        cx.with_key(u64::from(y), |cx| {
            let mut one = crate::edit::Text::of(head.clone(), crate::edit::WrapKind::Ruler);
            field_into(ink, cx, Rect::new(0, i32::from(y), w, 1), &mut one, &opts);
        });
    }

    // **The textarea below them, drawn by the component from its own state.**
    let rows = area.h - inputs;
    let id = field_into(
        ink,
        cx,
        Rect::new(0, i32::from(inputs), w, rows),
        &mut screen.st,
        &opts,
    )
    .id;
    // **A textarea that does not seat the focus places no caret at all.** `Ctx::caret_with` is
    // refused where nothing holds the keyboard, so the two caret gates are unaskable until
    // something is focused — architecture issue 25's finding arriving on `field`'s own screen. The
    // seating is written `if cx.focused().is_none()` rather than `if !cx.is_focused(id)`, because
    // the second drags the keyboard back the moment the user tabs away.
    if cx.focused().is_none() {
        cx.focus(id);
    }

    screen.st.index(w).rows()
}

/// **Play one frame of a screen.**
///
/// The allocation total is the caller's, for [`crate::runner::Run::counters`]'s reason: a figure
/// defaulted to zero is a counter that prints `0` when it means *nobody counted*.
pub fn play_field(screen: &mut Screen, allocations: Allocations) -> Played {
    let mut driver = driver_at(screen.w, H, Density::default());
    let mut rows_drawn = 0usize;
    // **Two frames, and the second is the one that is read.**
    //
    // Nothing holds the keyboard on the first frame of any program (architecture issue 25), and
    // `Ctx::caret_with` refuses a caret then — so a screen played once has no caret at all and the
    // two caret gates are unaskable. [`draw_screen`] seats the focus at the end of its own draw,
    // which is where an application seats it; the frame after is the first one where the textarea
    // holds the keyboard. That is the same cadence the rest of this crate already runs on — the
    // focus is seated from `Response::id` a frame before the key it enables (components 20) — and
    // it is a property of the runtime rather than of this screen.
    let mut pen = Pen::new(screen.w, H);
    driver.frame(|cx| {
        let _ = draw_screen(&mut pen, cx, screen);
    });
    let mut pen = Pen::new(screen.w, H);
    driver.frame(|cx| rows_drawn = draw_screen(&mut pen, cx, screen));
    let caret = driver.inspect().caret();
    let counters = Counters::of(&driver, pen.tally(), allocations);
    Played {
        canvas: pen.into_canvas(),
        counters,
        caret,
        rows_drawn,
    }
}

/// **The surface a defect changes, against the surface it should have drawn.**
///
/// [`SURFACE_BLIND`] cells for gates 1 and 2 — the caret is not a cell — and a real number for
/// gates 3 and 4.
pub fn surface_diff(defect: Defect) -> Diff {
    let inert = Allocations::over(1, 0);
    let (mut correct, mut broken) = screens(defect);
    play_field(&mut correct, inert)
        .canvas()
        .diff(play_field(&mut broken, inert).canvas())
}

/// **Which of §20's nine counters tell a defective build from a correct one. None of them, for all
/// four gates.**
///
/// Components ticket 11's `counters_that_separate_them`, one component over and over four defects
/// rather than one. An empty answer means *the equality is the only detector there is*; a non-empty
/// one would mean a cheaper gate exists and the scene is optional.
///
/// A counter that is `Reading::Unreachable` on both arms — `marked`, and it will stay that way —
/// contributes nothing either way and is skipped. A counter reachable on one arm and not the other
/// would be a defect in the instrument and is reported as a separation.
pub fn counters_that_separate(
    defect: Defect,
    correct_allocations: Allocations,
    defective_allocations: Allocations,
) -> Vec<Counter> {
    let (mut correct, mut broken) = screens(defect);
    let a = play_field(&mut correct, correct_allocations);
    let b = play_field(&mut broken, defective_allocations);
    Counter::ALL
        .into_iter()
        .filter(|c| {
            let (left, right) = (
                a.counters().get(*c).measured(),
                b.counters().get(*c).measured(),
            );
            left.is_some() != right.is_some() || (left.is_some() && left != right)
        })
        .collect()
}

/// **The resize, as an equality against the correct render.** §21's scene 13.
///
/// *625 rows drawn where 875 are needed*, and on the surface [`STALE_ROWS`] of eighty rows.
pub fn resized() -> Diff {
    surface_diff(Defect::MemoKeyedOnRevision)
}

/// **How many recomputes each memo key costs over the resize.** §21's *1 against 2*.
///
/// The defective key recomputes **once** — the revision did not move, so the resize is a cache hit —
/// and the correct one recomputes **twice**, once per width. §21 states it as a reason the defect
/// looks healthier, and it is: *`recomputes` points the wrong way.*
pub fn recomputes_over_the_resize() -> (usize, usize) {
    let document = text();
    let mut correct = 0usize;
    let mut defective = 0usize;

    let mut correct_key: Option<(u64, u16)> = None;
    let mut defective_key: Option<u64> = None;
    for w in [WIDE, NARROW] {
        if correct_key != Some((0, w)) {
            correct += 1;
            correct_key = Some((0, w));
            let _ = Index::build(&document, w, 0);
        }
        if defective_key != Some(0) {
            defective += 1;
            defective_key = Some(0);
            let _ = Index::build(&document, WIDE, 0);
        }
    }
    (correct, defective)
}

// ── the corpus figures, measured and remembered ──────────────────────────────────────────────────

/// **Clusters in the corpus. 167**, and §11 remembers 166. See [`crate::clusters`].
pub const CORPUS_CLUSTERS: usize = 167;
/// **Code points in the corpus, which is how many steps a `char` caret takes. 251**, against §11's
/// 217.
pub const CORPUS_CHARS: usize = 251;
/// **Steps landing inside a cluster. 84**, against §11's 51 — and it is
/// `CORPUS_CHARS − CORPUS_CLUSTERS` exactly, which is the half of §11's figure that reproduces.
pub const CORPUS_INSIDE: usize = CORPUS_CHARS - CORPUS_CLUSTERS;

/// §11's remembered cluster count.
pub const REMEMBERED_CLUSTERS: usize = 166;
/// §11's remembered char-step count.
pub const REMEMBERED_STEPS: usize = 217;
/// §11's remembered inside-landing count.
pub const REMEMBERED_INSIDE: usize = 51;
/// §11's remembered count of insertions that change the width by something other than what was
/// typed.
pub const REMEMBERED_SURPRISES: usize = 15;
/// §11's remembered `Left` ratio at 1 MB: 3 161.68 µs against 0.327.
pub const REMEMBERED_LEFT_RATIO: f64 = 9_655.0;
/// §11's remembered splice agreement: 499 of 500.
pub const REMEMBERED_SPLICE_AGREEMENTS: usize = 499;

// ── the subject, and the scan that says whether it is here ───────────────────────────────────────

/// **The component these two scenes are scenes of, and it is not declared yet.**
pub const SUBJECTS: [&str; 1] = ["field"];

/// Where [`SUBJECTS`] is declared, as `(module file, the declaration)`.
///
/// The home is the freeze's, joined through [`crate::Family`]: `field`'s first family is `F6Input`,
/// whose module is `input.rs`. A component is `fn(&mut Ctx, Rect, …) -> Response` (spec §1, rule 1),
/// so the thing to look for is a public function of the component's own name in its own family's
/// module.
pub const DECLARATIONS: [(&str, &str); 1] = [("input.rs", "pub fn field(")];

/// **Which of [`SUBJECTS`] this crate actually declares. Today: none.**
///
/// A source scan and not a `use`, for [`crate::dense::subjects_declared`]'s reason: *the item does
/// not exist* has no expression, and a `compile_fail` fence would pass today and pass again the day
/// somebody renames the module. The predicate is `crate::dense::declares` and it is shared rather
/// than copied.
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

/// **Whether the document stands on its subject, as a verdict rather than as a sentence.**
pub fn standing() -> Verdict {
    let declared = subjects_declared();
    Verdict::of(
        SUBJECTS.len(),
        SUBJECTS.len() - declared.len(),
        "`field` is not declared in this crate, so what stands on the document is a caret and a \
         wrap index written in `crate::document` and not the component. The two screens, the four \
         gates, the cluster corpus, the 500 splices and the 625/875 resize are measured and green; \
         what is missing is the subject",
        "components 24",
    )
}

/// **The sentence a scene waiting for its subject fails with**, or `None` once it is declared.
///
/// # This is criterion 8, and it is the distinction the whole ticket rests on
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
         components are undeclared — {}. This is not a defect in the screen. The document is drawn, \
         its caret is placed, all four of spec §11's gates run over a cluster corpus that is not \
         ASCII, the resize is {STALE_ROWS} of {H} rows against a correct render and five hundred \
         splices are played — see `crate::document::tests`. Inverted by `components 24`",
        owed.len(),
        SUBJECTS.len(),
        owed.join(", "),
    ))
}

/// **Fail with the subject that is missing, the file it belongs in, and the ticket.**
///
/// # Panics
///
/// Panics while [`SUBJECTS`] is undeclared, which is **today**. Components ticket 24 inverts it.
pub fn assert_stands_up(scene: &str) {
    if let Some(message) = owed_message(&subjects_declared(), scene) {
        panic!("{message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::counters::Tally;

    /// The allocation column, inert: both arms are handed the same value, which makes it neutral in
    /// a comparison rather than false. `examples/field_numbers.rs` has the probe.
    fn inert() -> Allocations {
        Allocations::over(1, 0)
    }

    /// **The document is 625 lines, 625 rows at 300 columns and 875 at 120.** §21's own pair.
    #[test]
    fn the_document_is_six_hundred_and_twenty_five_rows_wide_and_eight_hundred_and_seventy_five_narrow()
     {
        let document = text();
        assert_eq!(lines().len(), LINES);
        assert_eq!(Index::build(&document, WIDE, 0).rows(), LINES);
        assert_eq!(Index::build(&document, NARROW, 0).rows(), WRAPPED);
        assert_eq!(WRAPPED - LINES, LONG);
    }

    /// **Every long line is exactly two rows at 120 and exactly one at 300**, which is what makes
    /// the pair above arithmetic rather than a coincidence.
    #[test]
    fn every_long_line_is_two_rows_at_a_hundred_and_twenty_and_one_at_three_hundred() {
        for (i, line) in lines().iter().enumerate() {
            let narrow = wrap(line, NARROW).count();
            let wide = wrap(line, WIDE).count();
            assert_eq!(wide, 1, "line {i} does not fit on one row at {WIDE}");
            assert_eq!(
                narrow,
                if is_long(i) { 2 } else { 1 },
                "line {i} wraps into {narrow} rows at {NARROW}"
            );
        }
    }

    /// **No two lines share a prefix**, so a wrong row costs a whole row of cells.
    ///
    /// Components ticket 11 found the runner's own fixture short of this — `(5i + c) mod 26`
    /// aliases every thirteenth row and the equality under-reports by five. A four-digit index at
    /// the head of every line is the fix, and this is the test that says it worked.
    #[test]
    fn no_two_lines_of_the_document_share_a_prefix() {
        let all = lines();
        let mut heads: Vec<&str> = all.iter().map(|l| &l[..4]).collect();
        heads.sort_unstable();
        heads.dedup();
        assert_eq!(heads.len(), LINES);
    }

    /// **The corpus figures, measured against §11's remembered ones.**
    ///
    /// Two of the three do not reproduce and the third is not a measurement at all. The
    /// constants are asserted so that a corpus edited without the report being re-read fails here
    /// rather than in a paragraph.
    #[test]
    fn the_corpus_figures_are_asserted_as_measured_and_not_as_remembered() {
        let corpus = clusters::corpus();
        assert_eq!(corpus.len(), CORPUS_CLUSTERS);
        assert_eq!(corpus.chars(), CORPUS_CHARS);
        assert_eq!(corpus.inside(), CORPUS_INSIDE);

        // The relation §11's third figure is, and it holds for any string at all.
        assert_eq!(corpus.inside(), corpus.chars() - corpus.len());
        assert_eq!(
            REMEMBERED_INSIDE,
            REMEMBERED_STEPS - REMEMBERED_CLUSTERS,
            "§11's own three figures satisfy the same relation, which is what says it is arithmetic"
        );
    }

    /// **A `char` caret is faster per step and it lands inside clusters.** §11's negative case.
    #[test]
    fn a_char_caret_lands_inside_clusters_and_changes_widths_nobody_typed() {
        let corpus = clusters::corpus();
        let walk = char_walk(&corpus);
        assert_eq!(walk.steps, CORPUS_CHARS);
        assert_eq!(walk.clusters, CORPUS_CLUSTERS);
        assert_eq!(walk.inside, CORPUS_INSIDE);
        assert!(
            walk.width_surprises > 0,
            "an insertion sweep that surprises nobody is a sweep over ASCII"
        );
        assert!(
            walk.width_surprises <= walk.inside,
            "a boundary insertion cannot change the width by anything but what was typed"
        );

        // **The other direction, and it is the whole reason the corpus is a value.** Over a corpus
        // of nothing but ASCII the same sweep reports **nothing at all** — no step lands inside a
        // cluster and no insertion surprises anybody — so a gate run on ASCII is a gate that has
        // said nothing while reporting `ok`.
        let ascii = char_walk(&clusters::ascii_corpus());
        assert_eq!(ascii.inside, 0, "every ASCII cluster is one code point");
        assert_eq!(
            ascii.width_surprises, 0,
            "on ASCII an inserted column is always exactly one column"
        );
        assert_eq!(
            ascii.steps, ascii.clusters,
            "a char caret and a cluster caret are the same caret on ASCII, which is why the wrong \
             one ships"
        );
    }

    /// **Every visual row start is a cluster boundary of the whole buffer**, which is what lets
    /// [`on_a_boundary`] begin its walk at a row start instead of at byte 0.
    ///
    /// One full [`boundaries`] walk of the walk's own document, against every row start the index
    /// declares. Without it the cheap check would be resting on the thing it is checking: a wrap
    /// that broke inside a cluster would move a row start off a boundary and every caret measured
    /// from that start would still read as on one.
    #[test]
    fn every_row_start_is_a_cluster_boundary_of_the_whole_buffer() {
        let base = splice_document();
        let index = Index::build(&base, SPLICE_W, 0);
        let all = boundaries(&base);
        assert!(index.rows() > 1, "one row makes this vacuous");
        for r in 0..index.rows() {
            let start = index.row_start(r);
            assert!(
                all.contains(&start),
                "row {r} starts at byte {start}, which is inside a cluster"
            );
        }

        // Watched failing on the one thing that would invalidate it: a walk begun at a byte that is
        // **not** a boundary reports the caret it was asked about as off one.
        let inside = (0..base.len())
            .find(|b| base.is_char_boundary(*b) && !all.contains(b))
            .expect("this document is built out of clusters");
        assert!(!on_a_boundary(&base, inside, inside + 1));
        assert!(on_a_boundary(&base, 0, *all.last().expect("non-empty")));
    }

    /// **Gate 1 and gate 2 at both ends of the buffer and across an edit**, which is the half of
    /// §11's first two gates a construction-time walk cannot reach.
    ///
    /// The two caret gates next door inspect a caret over a buffer that does not move: a forward
    /// `step_right` walk of the corpus, and the gestures. **An edit moves the buffer under the
    /// caret** — and §11's own two caret defects arrived there. [`edit_walk`] plays a script at each
    /// of [`WALK_SEATS`] positions: both ends of the buffer, the seat, an insert of one cluster
    /// spelled as two `char`s, a backspace, a delete, and then **three** undos, which is the whole
    /// history back to the buffer the seat was made in.
    ///
    /// # The three producers are counted apart, and that is the finding
    ///
    /// A review of this ticket's first version found one number over all three and a zero being read
    /// as *the shipped verbs are right across an edit*, when for one of the three it could not have
    /// been anything else. [`Placed`] carries the split:
    ///
    /// - **[`Placed::Walked`] is structurally zero.** `Text::edit` re-seats the pair by walking
    ///   cluster steps from the new row's start, so it returns a member of its own walk — and a
    ///   checker walking from byte 0 is no more independent, because it passes through that row
    ///   start and continues with the same steps. Gate 1 asks nothing here; **gate 2 does**, and a
    ///   `Text::edit` that seats a byte-addressed caret leaves 715 wrong columns with the
    ///   off-boundary count still at zero.
    /// - **[`Placed::Restored`] is where gate 1 has teeth.** `Text::undo` restores the pair rather
    ///   than recomputing it (§11: 0.0007 µs against 6 109), so a restored pair is only as good as
    ///   the buffer it is restored into. The defective arm demonstrates it **without patching
    ///   `undo`**: the third undo puts the seat's own pair back, and on the `AtByte` arm that pair
    ///   is inside a cluster — [`WALK_INSIDE`] of them.
    /// - **[`Placed::Seated`]** is where a caret enters from outside, and the other
    ///   [`WALK_INSIDE`].
    #[test]
    fn the_caret_is_on_a_boundary_at_both_ends_and_after_every_edit() {
        let shipped = edit_walk(Seat::Gesture);
        assert_eq!(shipped.seats, WALK_SEATS, "the stride's population");
        assert_eq!(shipped.inside, WALK_INSIDE);
        assert_eq!(
            shipped.ends,
            2 * WALK_SEATS,
            "both ends of the buffer were reached at every seat, and neither is `Home` or `End`: \
             those are the ends of a visual row, so `select_all` reaches the buffer's length and a \
             click on row 0 column 0 comes back from there"
        );
        assert_eq!(shipped.inspected, 9 * WALK_SEATS, "the script's length");
        assert_eq!(
            shipped.edits,
            3 * WALK_SEATS,
            "an insert, a backspace and a delete at every seat"
        );
        assert_eq!(
            shipped.undone,
            3 * WALK_SEATS,
            "and three undos that each applied an entry, which is the whole history — an empty \
             ring would make a seat's last three inspections copies of the one before them"
        );
        assert_eq!(
            (
                shipped.off_boundary,
                shipped.off_boundary_seated,
                shipped.off_boundary_walked,
                shipped.off_boundary_restored
            ),
            (0, 0, 0, 0),
            "gate 1 across an edit: {} of {} carets the shipped verbs produced were inside a \
             cluster",
            shipped.off_boundary,
            shipped.inspected
        );
        assert_eq!(
            shipped.wrong_column, 0,
            "gate 2 across an edit: {} carried a column the engine's tables disagree with over \
             their own visual row",
            shipped.wrong_column
        );

        // **The deleted API, watched failing both gates over the same script.**
        let bad = edit_walk(Seat::AtByte);
        assert_eq!(
            (
                bad.seats,
                bad.inside,
                bad.inspected,
                bad.edits,
                bad.ends,
                bad.undone
            ),
            (
                shipped.seats,
                shipped.inside,
                shipped.inspected,
                shipped.edits,
                shipped.ends,
                shipped.undone
            ),
            "the two arms differ by the seat and by nothing else — every count that is not a \
             failure count, including both ends reached and all three undos applied, because the \
             failure counts below are read off this arm"
        );
        assert_eq!(
            bad.off_boundary_seated, WALK_INSIDE,
            "one per seat that is inside a cluster, which is where a caret enters from outside"
        );
        assert_eq!(
            bad.off_boundary_restored, WALK_INSIDE,
            "and one per seat again from the third undo, which restores that same pair into the \
             buffer it was made in — this is the observation gate 1 has after an edit, and it is \
             `Text::undo` restoring a pair rather than recomputing one"
        );
        assert_eq!(
            bad.off_boundary_walked, 0,
            "**structural, not a pass.** `Text::edit` walks cluster steps from the new row's start \
             and returns a member of its own walk; nothing a checker can do from outside separates \
             that from a correct one, which is why gate 2 is the one that fires here"
        );
        assert_eq!(bad.off_boundary, 2 * WALK_INSIDE);
        assert_eq!(
            bad.wrong_column,
            2 * (WALK_SEATS - 1),
            "every seat but the one on row 0, twice: once at the seat and once when the third undo \
             restores it. `at_byte`'s column is the tables over the **whole** prefix, and a \
             caret's column is a screen column re-seated at every row start"
        );
    }

    /// **Gate 1: the caret is always on a cluster boundary**, and the defective arm is not.
    #[test]
    fn the_caret_is_always_on_a_cluster_boundary_and_the_off_boundary_arm_is_not() {
        let corpus = clusters::corpus();
        let boundary = caret_seat(&corpus);
        assert!(corpus.is_boundary(boundary));
        assert!(
            !corpus.is_boundary(boundary + 1),
            "the seat is a cluster of more than one code point, so the byte after it is inside one"
        );

        // Every caret a cluster step can reach is a boundary, over the whole corpus.
        let mut at = Caret::default();
        while at.byte() < corpus.text().len() {
            assert!(
                corpus.is_boundary(at.byte()),
                "byte {} is inside a cluster",
                at.byte()
            );
            at = step_right(corpus.text(), at);
        }
        assert_eq!(at.byte(), corpus.text().len());
    }

    /// **Gate 2: the caret's column equals the engine's tables over the same prefix**, and the
    /// per-code-point arm does not.
    #[test]
    fn the_carets_column_is_the_engines_tables_and_counting_code_points_is_not() {
        let corpus = clusters::corpus();
        let mut at = Caret::default();
        while at.byte() < corpus.text().len() {
            assert_eq!(
                at.col(),
                width(&corpus.text()[..at.byte()]),
                "the carried column and the tables disagree at byte {}",
                at.byte()
            );
            at = step_right(corpus.text(), at);
        }
        assert_eq!(at.col(), corpus.width());

        // The defective spelling, watched being wrong: one column a code point.
        let end = corpus.text().len();
        assert_ne!(
            corpus.text().chars().count() as u16,
            width(corpus.text()),
            "counting code points agrees with the tables on ASCII and this corpus is not ASCII"
        );
        assert_eq!(corpus.column_of(end), Some(corpus.width()));
    }

    /// **Gate 3: a spliced index equals a rebuilt one, and the restart point is the whole
    /// question.**
    ///
    /// Both directions on the sweep's own first failing edit, so this test and the population next
    /// door are about the same thing rather than about two cases chosen separately.
    #[test]
    fn a_splice_restarted_at_the_row_of_the_edit_keeps_a_break_the_edit_invalidated() {
        let base = splice_document();
        let index = Index::build(&base, SPLICE_W, 0);
        let (edited, at) = first_naive_failure().expect("the population reaches the defect");
        let rebuilt = Index::build(&edited, SPLICE_W, 1);
        let row = index.row_of(at);
        assert!(
            row > 0,
            "an edit in the first row has no previous break to move"
        );

        let naive = index.spliced(&edited, row, 1);
        assert!(
            !naive.same_rows(&rebuilt),
            "the witness edit does not move a break, so it is not the case §11 describes"
        );
        assert_eq!(
            naive.first_disagreement(&rebuilt),
            Some(row - 1),
            "the row the naive splice keeps and should not is the one before the edit's"
        );

        let earlier = index.spliced(&edited, row - 1, 1);
        assert!(
            earlier.same_rows(&rebuilt),
            "restarting one row earlier is provably enough and this edit says otherwise"
        );
    }

    /// **Five hundred splices, because §11's figure is 499 of 500 and running fewer samples it.**
    #[test]
    fn five_hundred_splices_separate_the_two_restart_points() {
        let sweep = splice_sweep();
        assert_eq!(sweep.trials, SPLICES);
        assert_eq!(
            sweep.one_row_earlier, 0,
            "restarting one row earlier disagreed with a rebuild {} times, which contradicts §11's \
             *provably enough*",
            sweep.one_row_earlier
        );
        assert!(
            sweep.at_row_of > 0,
            "five hundred edits found no case where `row_of(at)` keeps an invalidated break, so \
             this population cannot see the defect at all"
        );
    }

    /// **Gate 4: the index's recorded width equals the width being drawn**, and the memo keyed on
    /// the revision alone does not.
    #[test]
    fn a_memo_keyed_on_the_revision_alone_draws_an_index_built_at_another_width() {
        let document = text();
        let correct = Index::build(&document, NARROW, 0);
        assert_eq!(correct.built_at(), NARROW);
        assert_eq!(correct.rows(), WRAPPED);

        let stale = Index::build(&document, WIDE, 0);
        assert_eq!(stale.built_at(), WIDE);
        assert_eq!(stale.rows(), LINES);
        assert_ne!(
            stale.built_at(),
            NARROW,
            "the fourth gate is exactly this inequality, and it is invisible in the row count \
             until the rows are compared"
        );
    }

    /// **Row 17's equality over the shipped component, at the sweep's own population of five
    /// hundred.**
    ///
    /// [`five_hundred_splices_separate_the_two_restart_points`] is an equality about
    /// [`Index::spliced`] at a restart point the test computes; this is the same five hundred edits
    /// played through `Text::insert`, where the restart point is the one `Text::edit` chose. The
    /// naive arm is [`crate::edit::defective::restarted_at_row_of`] — one field on the state, so the
    /// two arms are one drawing path with one value between them.
    ///
    /// **The two sweeps report the same three numbers**, which is what makes the cheap one a
    /// legitimate stand-in for the expensive one rather than a second population that happens to
    /// agree. It is not an equality between two derivations of one declaration: the restart point is
    /// spelled `i.row_of(at).saturating_sub(1)` inside `Text::edit` and `row.saturating_sub(1)`
    /// inside [`splice_sweep`], in two files, and a change to either would separate them here.
    ///
    /// **Nine of five hundred and not one.** §11 remembers *499 times in 500*
    /// ([`REMEMBERED_SPLICE_AGREEMENTS`]) and this population reaches the defect nine times; the
    /// figure is reported as measured beside the remembered one in `examples/field_numbers.rs` and
    /// asserted here as a floor rather than as a number, because the count is a property of the
    /// five hundred insertions and not of the mechanism.
    #[test]
    fn the_components_five_hundred_splices_agree_with_a_rebuild_and_the_naive_point_does_not() {
        let component = component_splice_sweep();
        assert_eq!(component.trials, SPLICES, "the population, stated");
        assert_eq!(
            component.one_row_earlier, 0,
            "the shipped restart point disagreed with a rebuild {} times in {SPLICES}, which \
             contradicts §11's *provably enough*",
            component.one_row_earlier
        );
        assert!(
            component.at_row_of > 0,
            "five hundred edits through `Text::insert` found no case where the naive restart point \
             keeps an invalidated break, so this population cannot see the defect at all"
        );
        assert_eq!(
            component,
            splice_sweep(),
            "the component's splice and the mechanism's are the same function over this population"
        );
    }

    /// **Gate 4 over the shipped draw: the index the frame drew with was built at the frame's own
    /// width.**
    ///
    /// The gate next door is an *inequality* between two builds — two indexes, one at 300 and one at
    /// 120, neither of them drawn — and the surface figure below it is a count of differing rows.
    /// Neither states the property §11 states, which is about **the width being drawn**: a frame at
    /// `w` columns holds an index built at `w`. Read off the state after
    /// [`crate::input::field_into`] has drawn it, so the width it is compared against is the
    /// rectangle the component was handed and not a number the test chose.
    ///
    /// Both arms enter the frame holding an index built at [`WIDE`] and draw at [`NARROW`], which is
    /// the resize. The correct key misses and rebuilds; the defective one **hits**, and what it
    /// draws with was built at the width before the resize.
    #[test]
    fn the_index_the_field_drew_with_was_built_at_the_width_it_drew_at() {
        let (mut correct, mut stale) = screens(Defect::MemoKeyedOnRevision);
        for arm in [&correct, &stale] {
            assert_eq!(arm.w, NARROW, "the screen is drawn after the resize");
            assert_eq!(
                arm.st.indexed().expect("built before the frame").built_at(),
                WIDE,
                "and enters the frame holding an index built before it, which is what makes this a \
                 resize rather than a first frame"
            );
        }

        let _ = play_field(&mut correct, inert());
        assert_eq!(
            correct.st.indexed().expect("the draw built one").built_at(),
            correct.w,
            "the shipped key is `(revision, width)`, so the frame's own width is in it"
        );
        assert_eq!(correct.st.indexed().expect("built").rows(), WRAPPED);

        // **Watched failing on the same frame**, which is the direction that matters: the defect is
        // not a build that is absent, it is a build that is *there* and answers the wrong width.
        let _ = play_field(&mut stale, inert());
        assert_eq!(
            stale
                .st
                .indexed()
                .expect("the draw hit the memo")
                .built_at(),
            WIDE,
            "the memo keyed on the revision alone is a hit on a resize, so gate 4 is the one \
             equality that separates the two — and `recomputes` reports the defect as the cheaper \
             build"
        );
        assert_ne!(stale.st.indexed().expect("built").built_at(), stale.w);
        assert_eq!(stale.st.indexed().expect("built").rows(), LINES);
    }

    /// **The resize on the surface: 69 of 80 rows.** §21's own figure, and criterion 5.
    #[test]
    fn the_resize_differs_on_sixty_nine_of_eighty_rows() {
        let diff = resized();
        assert_eq!(diff.over, (NARROW, H));
        assert_eq!(
            diff.rows, STALE_ROWS,
            "the resize differs on {} of {H} rows",
            diff.rows
        );
        assert_eq!(
            diff.first.map(|(_, y)| y),
            Some(SHORT_HEAD as u16 + 1),
            "the first differing row is the row after the first long line's first row, which both \
             arms draw the same because that line's greedy break falls short of the band"
        );
        assert_eq!(
            usize::from(H) - usize::from(diff.first.map(|(_, y)| y).unwrap_or(0)),
            STALE_ROWS,
            "the count is `80 - the first differing row` and nothing else, which is what makes it \
             a construction rather than a coincidence"
        );
    }

    /// **`recomputes` points the wrong way: 1 against 2.** §21's own figure, and the reason the
    /// scene may not be gated on it.
    #[test]
    fn the_defective_memo_recomputes_once_and_the_correct_one_twice() {
        let (correct, defective) = recomputes_over_the_resize();
        assert_eq!((defective, correct), (1, 2));
    }

    /// **No counter of §20's nine separates a defective build from a correct one, for any of the
    /// four gates.**
    ///
    /// This is register row 69 and it is the ticket's headline finding: §11's *four gates, none of
    /// them visible on the rendered screen* is true, and for two of the four it is true in the sense
    /// the whole scene list is built on — the screen changes and **no counter can say so**.
    #[test]
    fn no_counter_separates_a_field_that_violates_one_of_the_four_gates_from_one_that_does_not() {
        for defect in Defect::ALL {
            let separating = counters_that_separate(defect, inert(), inert());
            assert!(
                separating.is_empty(),
                "`{}` is separated by {:?}, so a cheaper gate than the equality exists",
                defect.gate(),
                separating.iter().map(|c| c.word()).collect::<Vec<_>>()
            );
        }
    }

    /// **Two of the four are invisible on the surface as well, and two are not.**
    ///
    /// §11's heading reads as one claim and measures as two. This is the split, asserted.
    #[test]
    fn the_two_caret_gates_change_no_cell_and_the_two_index_gates_change_many() {
        for defect in Defect::ALL {
            let diff = surface_diff(defect);
            if defect.changes_the_surface() {
                assert!(
                    diff.cells > 0,
                    "`{}` changes no cell, so `changes_the_surface` is wrong about it",
                    defect.gate()
                );
            } else {
                assert_eq!(
                    diff.cells,
                    SURFACE_BLIND,
                    "`{}` moved {} cells, and the caret is not a cell",
                    defect.gate(),
                    diff.cells
                );
            }
        }
    }

    /// **The caret defects are visible to exactly one instrument: the caret itself.**
    ///
    /// The surface is identical and every counter agrees, so a gate that did not read
    /// `Frame::caret` would have nothing at all to look at.
    #[test]
    fn the_caret_is_where_the_two_caret_defects_are_visible_and_it_is_not_a_cell() {
        for defect in [Defect::CaretOffBoundary, Defect::CaretColumnByChars] {
            let (mut good, mut bad) = screens(defect);
            let correct = play_field(&mut good, inert());
            let broken = play_field(&mut bad, inert());
            assert_eq!(
                correct.canvas().diff(broken.canvas()).cells,
                SURFACE_BLIND,
                "`{}` is visible on the surface",
                defect.gate()
            );
            assert!(
                correct.caret().is_some(),
                "the frame placed no caret at all, which is what an unfocused textarea does"
            );
            assert_ne!(
                correct.caret(),
                broken.caret(),
                "`{}` did not move the caret, so its seat is somewhere the defect cannot be seen",
                defect.gate()
            );
        }
    }

    /// **The screen is a screen: twenty fields, a hit entry each, and every cell of it written.**
    #[test]
    fn the_document_screen_stands_twenty_fields_and_writes_every_cell_once() {
        let (mut correct, _) = screens(Defect::CaretOffBoundary);
        let mut driver = driver_at(WIDE, H, Density::default());
        let mut tally = Tally::new();
        driver.frame(|cx| {
            let _ = draw_screen(&mut tally, cx, &mut correct);
        });
        let frame = driver.inspect();
        assert_eq!(frame.hits().len(), usize::from(FIELDS));
        assert_eq!(tally.writes(), u64::from(WIDE) * u64::from(H));
        assert_eq!(
            tally.distinct(),
            tally.writes(),
            "the screen is a partition of its rectangle and not a prefix of it"
        );
    }

    /// **`Left` at the end of a pasted megabyte, both ways.** A report, asserted only as a
    /// direction.
    ///
    /// The **ratio** is what §11 states and a ratio of timings is a report; what is gated is that
    /// the indexed arm segments over one row and the blind arm over the whole buffer, which is a
    /// count.
    #[test]
    fn left_at_the_end_of_a_megabyte_costs_one_row_with_an_index_and_the_buffer_without_one() {
        let document = pasted();
        assert!(document.len() >= PASTED);
        let index = Index::build(&document, WIDE, 0);
        let (blind, indexed, caret) = left_at_the_end(&document, &index);

        // The mechanism, as a count rather than as a stopwatch: how far each arm had to segment.
        let row = index.row_of(document.len());
        let from_the_row = document.len() - index.row_start(row);
        assert!(
            from_the_row < document.len() / 1_000,
            "the last row is {from_the_row} bytes of {}, which is not a row",
            document.len()
        );
        assert!(caret.byte() < document.len());
        assert!(
            indexed <= blind,
            "the indexed arm took {indexed:?} against the blind arm's {blind:?}"
        );
    }

    /// **The document stands on its declared subject**, which is what components ticket 24
    /// inverted.
    ///
    /// It was `the_document_is_red_because_field_is_not_declared` and it panicked on purpose for
    /// one ticket. What changed is not the screen: every figure below was measured before `field`
    /// existed and reproduces through it. What changed is that the three scenes are now
    /// measurements of the **component** rather than of a caret and a wrap index written beside
    /// the gate — which is the difference between a scene that is stood up and one that is green.
    #[test]
    fn the_document_stands_on_its_declared_subject() {
        assert_stands_up("twenty fields and a 1 MB pasted textarea");
        assert_eq!(subjects_declared(), vec!["field"]);
        standing().assert_met("the document");
    }

    /// **The waiting message separates *unimplemented* from *wrong*.** Criterion 8, both ways.
    #[test]
    fn the_waiting_message_separates_unimplemented_from_wrong() {
        let owed = owed_message(&[], "a scene").expect("nothing is declared today");
        assert!(owed.contains("waiting for its subject rather than failing"));
        assert!(owed.contains("This is not a defect in the screen"));
        assert!(owed.contains("src/input.rs"));
        assert!(owed.contains("pub fn field("));
        assert!(owed.contains("components 24"));

        // The other direction: once the subject is declared the sentence is gone rather than
        // softened.
        assert_eq!(owed_message(&["field"], "a scene"), None);
    }

    /// **The subject scan finds a declaration when there is one**, so the red row above is a fact
    /// about `input.rs` and not about the scanner.
    #[test]
    fn the_subject_scan_finds_a_declaration_when_there_is_one() {
        // **The fixture is interpolated and not written out**, and this is the third time a source
        // scan in this crate has matched itself. `keys::tests::every_text_bearing_name_is_in_the_
        // freeze` scans every `.rs` file here for the declaration below to check that row 5's standing
        // is still over a sink rather than over a component — and a test that spelled the hostile
        // line as a literal *is* that line. `crate::frame`'s deleted `focus_ring` scanner and
        // `gates`'s counting-allocator scan both learned it the same way.
        let (verb, name) = ("pub fn ", SUBJECTS[0]);
        let declaration = format!("{verb}{name}(");
        assert_eq!(declaration, DECLARATIONS[0].1);
        assert!(crate::dense::declares(
            &format!("{declaration}cx: &mut Ctx, area: Rect) -> Response {{"),
            &declaration
        ));
        assert!(!crate::dense::declares(
            &format!("// {declaration} is what ticket 24 will write"),
            &declaration
        ));
        assert_eq!(subjects_declared(), vec![SUBJECTS[0]]);

        // `Met` over **one** and never over nothing, which is `Verdict::of`'s vacuity refusal on a
        // population of one: an equality between two things that do not exist holds, and the
        // population here is `SUBJECTS` and is not empty.
        match standing() {
            Verdict::Met { over } => assert_eq!(over, 1),
            Verdict::Unmet { why, .. } => panic!("`field` is declared and the verdict says: {why}"),
        }
    }
}
