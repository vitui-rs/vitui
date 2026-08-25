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
//! # The gate is red, and it is red for one reason
//!
//! `field` does not exist. [`standing`] is a [`crate::obligations::Verdict`] over one subject,
//! [`subjects_declared`] opens the file the freeze homes it in — `input.rs`, F6 — and
//! [`owed_message`] is the sentence that separates *waiting for its subject* from *the code is
//! wrong*. Ticket 09's criterion 7, inherited whole. Inverted by **components 24**.
//!
//! Unlike [`crate::listing`], **all of this ticket's rows turn together**: there is no second reason
//! here the way the wheel gate is a second reason there. What stands in for `field` is a caret and
//! a wrap index written in this file, and every figure below is measured over them.
//!
//! [`Barrier`]: crate::gates::Instrument::Barrier

use std::sync::LazyLock;
use std::time::{Duration, Instant};

use vitui_runtime::layout::text::{truncate, width, wrap};
use vitui_runtime::{Ctx, Cursor, Density, Id, Interest, Role};

use crate::clusters::{self, Corpus, next_cluster};
use crate::counters::{Allocations, Counter, Counters};
use crate::ink::Ink;
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
/// Line `i` opens with its own four-digit index, so no two lines share a prefix and a wrong row
/// costs a full row of cells rather than the suffix nobody was looking at. That is
/// [`crate::runner::Fixture::lines`]'s property, restated here because this document's lines vary in
/// length and its generator's do not — and it is the property components ticket 11 found the runner
/// fixture short of, where `(5i + c) mod 26` aliases every thirteenth row.
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

// ── the wrap index ───────────────────────────────────────────────────────────────────────────────

/// **The wrap index: visual-row starts, the revision it was built at, and the width it was built
/// at.**
///
/// §11 describes it as a `Vec<u32>` of visual-row starts plus a sentinel, keyed on
/// `(revision, width)`. Two departures, both stated rather than slipped in:
///
/// 1. **A row carries its end as well as its start.** A greedy break consumes the space it broke
///    at, so a row's content ends before the next row's start and a sentinel cannot say where. A
///    stand-in that guessed would be measuring the guess.
/// 2. **The width is a field and not only a key.** That is the *point*: §11's fourth gate is *the
///    index's recorded width equals the width being drawn*, and an index that does not record its
///    width has no gate to fail. [`Index::built_at`] is the recording, and
///    [`Defect::MemoKeyedOnRevision`] is what happens when the key drops it.
#[derive(Clone, Debug)]
pub struct Index {
    rows: Vec<(u32, u32)>,
    revision: u64,
    width: u16,
}

impl Index {
    /// **Build the index over `text` at `w` columns.**
    ///
    /// The wrapping is `vitui_runtime::layout::text::wrap` and **not a second greedy wrap written
    /// here**: a stand-in that reimplemented the break rule would make every equality below a
    /// comparison between two copies of one bug. The offsets are recovered from the subslices the
    /// iterator yields, which are slices of `text` itself.
    pub fn build(text: &str, w: u16, revision: u64) -> Index {
        let mut rows = Vec::new();
        let base = text.as_ptr() as usize;
        for hard in text.split('\n') {
            let at = hard.as_ptr() as usize - base;
            let mut any = false;
            for piece in wrap(hard, w) {
                let off = piece.as_ptr() as usize - base;
                rows.push((off as u32, (off + piece.len()) as u32));
                any = true;
            }
            // An empty hard line is one visual row and `wrap` yields nothing for it. A textarea
            // that skipped it would renumber every row below a blank line.
            if !any {
                rows.push((at as u32, at as u32));
            }
        }
        Index {
            rows,
            revision,
            width: w,
        }
    }

    /// How many visual rows. **625 at [`WIDE`] and 875 at [`NARROW`]**, which is §21's pair.
    pub fn rows(&self) -> usize {
        self.rows.len()
    }

    /// Whether it holds no rows. It never does over a non-empty document.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// §11's own shape: the visual-row starts, plus the last row's end as the sentinel.
    pub fn starts(&self) -> Vec<u32> {
        let mut out: Vec<u32> = self.rows.iter().map(|(s, _)| *s).collect();
        if let Some((_, end)) = self.rows.last() {
            out.push(*end);
        }
        out
    }

    /// The revision it was built at.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// **The width it was built at**, which §11's fourth gate compares against the width being
    /// drawn.
    pub fn built_at(&self) -> u16 {
        self.width
    }

    /// The row `byte` falls in. `O(log rows)`.
    pub fn row_of(&self, byte: usize) -> usize {
        match self
            .rows
            .binary_search_by(|(s, _)| (*s as usize).cmp(&byte))
        {
            Ok(i) => i,
            Err(0) => 0,
            Err(i) => i - 1,
        }
    }

    /// Where row `r` starts, in bytes.
    pub fn row_start(&self, r: usize) -> usize {
        self.rows.get(r).map(|(s, _)| *s as usize).unwrap_or(0)
    }

    /// Row `r`'s content.
    pub fn row_text<'a>(&self, text: &'a str, r: usize) -> &'a str {
        match self.rows.get(r) {
            Some((s, e)) => &text[*s as usize..*e as usize],
            None => "",
        }
    }

    /// **Rebuild the tail from row `restart`, keeping the rows before it.** §10's splice.
    ///
    /// The restart point is the whole question — see [`splice_sweep`] — so it is an argument rather
    /// than a policy, and the two spellings the scene compares are `row_of(at)` and one row earlier.
    ///
    /// It rewraps to the end of the document rather than stopping when it resynchronises. That is an
    /// upper bound on the work and **exact on the answer**, which is the half this scene is about: a
    /// splice and a rebuild can only disagree about the rows the splice kept.
    pub fn spliced(&self, edited: &str, restart: usize, revision: u64) -> Index {
        let from = self.row_start(restart.min(self.rows.len()));
        let head: Vec<(u32, u32)> = self.rows[..restart.min(self.rows.len())].to_vec();
        let tail = Index::build(&edited[from..], self.width, revision);
        let mut rows = head;
        rows.extend(
            tail.rows
                .iter()
                .map(|(s, e)| (*s + from as u32, *e + from as u32)),
        );
        Index {
            rows,
            revision,
            width: self.width,
        }
    }

    /// Whether two indices describe the same rows.
    pub fn same_rows(&self, other: &Index) -> bool {
        self.rows == other.rows
    }

    /// The first row two indices disagree about, if any.
    pub fn first_disagreement(&self, other: &Index) -> Option<usize> {
        self.rows
            .iter()
            .zip(other.rows.iter())
            .position(|(a, b)| a != b)
            .or_else(|| {
                (self.rows.len() != other.rows.len())
                    .then_some(self.rows.len().min(other.rows.len()))
            })
    }
}

// ── the caret ────────────────────────────────────────────────────────────────────────────────────

/// **The caret: a `(byte, column)` pair.** §11's first sentence, as a type.
///
/// Not a byte offset whose column is computed where it is needed. `display_width(&s[..caret])` is
/// correct on every cluster and is **2 988 µs a frame at 1 MB against 80.12** — thirty budgets, on a
/// screen where nothing happened.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Caret {
    /// Where it is, in bytes.
    pub byte: usize,
    /// What column it is at, carried alongside and moved by the width of the cluster crossed.
    pub col: u16,
}

/// How far [`step`] looks for a line break before handing the rest to
/// [`crate::clusters::next_cluster`]. Sixty-four bytes.
///
/// Every cluster in [`crate::clusters::ROTA`] is at most twenty-five bytes, and the bound is what
/// keeps a step `O(1)`: a `find` for the next break over the remainder of a pasted megabyte would
/// make crossing the buffer `O(bytes^2)`, which is a fixture that cannot run rather than a slow one.
const LOOKAHEAD: usize = 64;

/// **The forward cluster step a *document* needs**, which is not quite the one a line needs.
///
/// [`crate::clusters::next_cluster`] probes with `truncate`, and `truncate` is greedy over
/// zero-width clusters — **a line break is zero columns wide**, so a probe for one column at
/// `"a\nb"` answers `"a\n"`, which is two clusters in one answer. That is the same precondition the
/// corpus asserts for itself, arriving on the one string a `textarea` actually holds.
///
/// So the break is handled here and the rest is delegated: CRLF is one cluster (UAX #29), a lone
/// line feed or carriage return is one, and everything else is probed inside a window that stops at
/// the next break. **This is the second half of the `graphemes()` finding** — see this module's
/// header — and it is the half a component ticket would have discovered by shipping a caret that
/// walked off the end of a line.
pub fn step(text: &str) -> Option<&str> {
    let bytes = text.as_bytes();
    match bytes.first() {
        None => return None,
        Some(b'\r') => {
            return Some(if bytes.get(1) == Some(&b'\n') {
                &text[..2]
            } else {
                &text[..1]
            });
        }
        Some(b'\n') => return Some(&text[..1]),
        Some(_) => {}
    }
    let mut bound = text.len().min(LOOKAHEAD);
    while !text.is_char_boundary(bound) {
        bound -= 1;
    }
    let head = &text[..bound];
    let stop = head
        .as_bytes()
        .iter()
        .position(|b| *b == b'\n' || *b == b'\r')
        .unwrap_or(bound);
    next_cluster(&text[..stop])
}

/// **Every cluster boundary of a document**, walked with [`step`].
pub fn boundaries(text: &str) -> Vec<usize> {
    let mut out = vec![0usize];
    let mut at = 0usize;
    while let Some(cluster) = step(&text[at..]) {
        at += cluster.len();
        out.push(at);
    }
    out
}

/// **`Left`, from a boundary already known.**
///
/// `from` must be a cluster boundary at column `from_col` and no further right than `caret`. That
/// precondition is §11's whole complexity argument: `Graphemes` is a forward iterator, so the
/// boundary *before* an offset can only be found by segmenting forward from one already known, **and
/// which one you have decides the complexity.** With no index the only one you have is byte 0.
pub fn left_from(text: &str, caret: Caret, from: usize, from_col: u16) -> Caret {
    let mut at = Caret {
        byte: from,
        col: from_col,
    };
    let mut prev = at;
    while at.byte < caret.byte {
        let Some(cluster) = step(&text[at.byte..]) else {
            break;
        };
        prev = at;
        at = Caret {
            byte: at.byte + cluster.len(),
            // **Saturating, and the saturation is the argument.** A caret's column is a *screen*
            // column, reset at every row start; an arm that segments from byte 0 of a megabyte is
            // accumulating something that is not a column at all, which is §11's own point about
            // `display_width(&s[..caret])`. What the blind arm is compared on is the byte.
            col: at.col.saturating_add(width(cluster)),
        };
    }
    prev
}

/// **`Right`, one cluster.** Cheap from anywhere, which is why §11 is about `Left`.
pub fn right(text: &str, caret: Caret) -> Caret {
    match step(&text[caret.byte..]) {
        Some(cluster) => Caret {
            byte: caret.byte + cluster.len(),
            col: caret.col.saturating_add(width(cluster)),
        },
        None => caret,
    }
}

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
    let end = Caret {
        byte: text.len(),
        col: 0,
    };

    let started = Instant::now();
    let without = left_from(text, end, 0, 0);
    let blind = started.elapsed();

    // **A `Left` at a row start is answered from the row before**, and the index is what makes
    // that one subtraction rather than a search. A pasted megabyte ends in a line break, so the
    // caret's own row starts exactly where the caret is and segmenting forward from there crosses
    // nothing at all — an arm that took `row_of(caret)` unconditionally would answer *the caret did
    // not move*, which looks like a correct `Left` at the end of a document and is not one.
    let row = index.row_of(end.byte);
    let row = if index.row_start(row) >= end.byte && row > 0 {
        row - 1
    } else {
        row
    };
    let start = index.row_start(row);
    let started = Instant::now();
    let with = left_from(text, end, start, 0);
    let indexed = started.elapsed();

    assert_eq!(
        without.byte, with.byte,
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
fn splice_document() -> String {
    let all = lines();
    all[..64].join("\n")
}

/// The width the splice sweep wraps at. Forty, for [`splice_document`]'s reason.
const SPLICE_W: u16 = 40;

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
    /// The first visual row drawn. Zero for both scenes; see [`screens`] for the one arm that is
    /// not.
    pub offset: usize,
    /// What the textarea holds.
    pub text: String,
    /// The wrap index it draws from.
    pub index: Index,
    /// The column the caret is placed at, on row 0.
    pub caret_col: u16,
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
    match defect {
        Defect::CaretOffBoundary | Defect::CaretColumnByChars => {
            let text = text();
            let index = Index::build(&text, WIDE, 0);
            let corpus = clusters::corpus();
            let seat = caret_seat(&corpus);
            let correct = width(&corpus.text()[..seat]);
            let broken = match defect {
                Defect::CaretOffBoundary => width(&corpus.text()[..seat + 1]),
                _ => u16::try_from(corpus.text()[..seat].chars().count()).unwrap_or(u16::MAX),
            };
            (
                Screen {
                    w: WIDE,
                    inputs: FIELDS - 1,
                    offset: 0,
                    text: text.clone(),
                    index: index.clone(),
                    caret_col: correct,
                },
                Screen {
                    w: WIDE,
                    inputs: FIELDS - 1,
                    offset: 0,
                    text,
                    index,
                    caret_col: broken,
                },
            )
        }
        Defect::MemoKeyedOnRevision => {
            let text = text();
            (
                Screen {
                    w: NARROW,
                    // **One field**, which is §21's own content for this scene, and it is what
                    // makes *69 of 80* a count over eighty rows rather than over sixty-one.
                    inputs: 0,
                    offset: 0,
                    text: text.clone(),
                    // Keyed on `(revision, width)`: an index at the width about to be drawn at.
                    index: Index::build(&text, NARROW, 0),
                    caret_col: 0,
                },
                Screen {
                    w: NARROW,
                    inputs: 0,
                    offset: 0,
                    // Keyed on the revision alone: the revision did not move, so the resize is a
                    // cache hit and what comes back was built at `WIDE`.
                    index: Index::build(&text, WIDE, 0),
                    text,
                    caret_col: 0,
                },
            )
        }
        Defect::SpliceRestartedAtRowOf => {
            let base = splice_document();
            let index = Index::build(&base, SPLICE_W, 0);
            let (edited, at) = first_naive_failure().expect(
                "five hundred edits reached no naive-splice failure, so this population cannot see \
                 the defect at all",
            );
            let row = index.row_of(at);
            // **The screen is scrolled to the witness, and that is not a convenience.** Sixty-four
            // lines at forty columns is three hundred-odd visual rows and the defect is one of
            // them; a screen that always started at row 0 would score this defect clean **by not
            // looking at it**, which is `Verdict::of`'s vacuity refusal wearing a viewport.
            let offset = (row - 1).saturating_sub(4);
            (
                Screen {
                    w: SPLICE_W,
                    inputs: 0,
                    offset,
                    index: Index::build(&edited, SPLICE_W, 1),
                    text: edited.clone(),
                    caret_col: 0,
                },
                Screen {
                    w: SPLICE_W,
                    inputs: 0,
                    offset,
                    index: index.spliced(&edited, row, 1),
                    text: edited,
                    caret_col: 0,
                },
            )
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
pub fn draw_screen<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, screen: &Screen) -> usize {
    let area = cx.area();
    let w = area.w;
    let body = cx.theme().paint(Role::Body);
    let corpus = clusters::corpus();
    let head = padded(truncate(corpus.text(), w), w);

    // The one-row inputs across the top. Each is a field and each declares one hit entry.
    let inputs = screen.inputs;
    for y in 0..inputs {
        cx.with_key(u64::from(y), |cx| {
            let id = cx.id();
            let _ = cx.interact(id, Rect::new(0, i32::from(y), w, 1), Interest::CLICK);
            let _ = ink.text(cx, 0, i32::from(y), &head, body);
        });
    }

    // The textarea below them, drawn from its wrap index.
    let id = Id::named("textarea");
    let rows = area.h - inputs;
    let _ = cx.interact(
        id,
        Rect::new(0, i32::from(inputs), w, rows),
        Interest::CLICK.with(Interest::FOCUS),
    );
    if cx.focused().is_none() {
        cx.focus(id);
    }

    for r in 0..rows {
        let y = i32::from(inputs) + i32::from(r);
        // The stale arm's rows are up to `WIDE` columns long and the rectangle is `w`, so what
        // reaches the surface is the truncation — which is why the first differing row is the row
        // *after* the first long line and not that line itself.
        let line = padded(
            truncate(
                screen
                    .index
                    .row_text(&screen.text, screen.offset + r as usize),
                w,
            ),
            w,
        );
        let _ = ink.text(cx, 0, y, &line, body);
    }

    // **The caret, which is not a cell.** Gates 1 and 2 differ here and nowhere else.
    cx.caret(i32::from(screen.caret_col), 0);

    screen.index.rows()
}

/// `s`, padded with spaces to `w` columns. A row is a **partition** of its width, never a prefix.
fn padded(s: &str, w: u16) -> String {
    let mut out = String::from(s);
    for _ in width(s)..w {
        out.push(' ');
    }
    out
}

/// **Play one frame of a screen.**
///
/// The allocation total is the caller's, for [`crate::runner::Run::counters`]'s reason: a figure
/// defaulted to zero is a counter that prints `0` when it means *nobody counted*.
pub fn play_field(screen: &Screen, allocations: Allocations) -> Played {
    let mut driver = driver_at(screen.w, H, Density::default());
    let mut pen = Pen::new(screen.w, H);
    let mut rows_drawn = 0usize;
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
    let (correct, broken) = screens(defect);
    play_field(&correct, inert)
        .canvas()
        .diff(play_field(&broken, inert).canvas())
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
    let (correct, broken) = screens(defect);
    let a = play_field(&correct, correct_allocations);
    let b = play_field(&broken, defective_allocations);
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
        while at.byte < corpus.text().len() {
            assert!(
                corpus.is_boundary(at.byte),
                "byte {} is inside a cluster",
                at.byte
            );
            at = right(corpus.text(), at);
        }
        assert_eq!(at.byte, corpus.text().len());
    }

    /// **Gate 2: the caret's column equals the engine's tables over the same prefix**, and the
    /// per-code-point arm does not.
    #[test]
    fn the_carets_column_is_the_engines_tables_and_counting_code_points_is_not() {
        let corpus = clusters::corpus();
        let mut at = Caret::default();
        while at.byte < corpus.text().len() {
            assert_eq!(
                at.col,
                width(&corpus.text()[..at.byte]),
                "the carried column and the tables disagree at byte {}",
                at.byte
            );
            at = right(corpus.text(), at);
        }
        assert_eq!(at.col, corpus.width());

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
            let (good, bad) = screens(defect);
            let correct = play_field(&good, inert());
            let broken = play_field(&bad, inert());
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
        let (correct, _) = screens(Defect::CaretOffBoundary);
        let mut driver = driver_at(WIDE, H, Density::default());
        let mut tally = Tally::new();
        driver.frame(|cx| {
            let _ = draw_screen(&mut tally, cx, &correct);
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
        assert!(caret.byte < document.len());
        assert!(
            indexed <= blind,
            "the indexed arm took {indexed:?} against the blind arm's {blind:?}"
        );
    }

    /// **The document is red because `field` is not declared.**
    ///
    /// # Panics
    ///
    /// Panics today, on purpose, and components ticket 24 inverts it.
    #[test]
    #[should_panic(expected = "waiting for its subject")]
    fn the_document_is_red_because_field_is_not_declared() {
        assert_stands_up("twenty fields and a 1 MB pasted textarea");
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
        assert_eq!(subjects_declared(), Vec::<&str>::new());

        // `Unmet` over **one** and not `Met` over nothing, which is `Verdict::of`'s vacuity refusal
        // on a population of one.
        match standing() {
            Verdict::Unmet {
                over,
                failing,
                why,
                inverted_by,
            } => {
                assert_eq!((over, failing), (1, 1));
                assert!(why.contains("`field` is not declared"));
                assert_eq!(inverted_by, "components 24");
            }
            Verdict::Met { .. } => panic!("`field` is not declared and the verdict says it is"),
        }
    }
}
