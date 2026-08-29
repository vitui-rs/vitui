//! **The text machine: a buffer, a caret pair, an anchored selection, an undo ring and a wrap
//! index.**
//!
//! Components ticket 24. Spec §11. This is what [`crate::input::field`] is, and
//! [`crate::document`] — the screens ticket 23 built beside it — now draws *through* it rather
//! than through a caret and an index written next to the gate.
//!
//! # §11's headline, and it is the reason every type here is one type
//!
//! > **Every backward question about text is O(prefix) unless an index already knows the answer,
//! > and the index that answers them is the one the wrapping already needs.**
//!
//! `Graphemes` is a forward iterator, so the boundary *before* an offset can only be found by
//! segmenting forward from one already known — **and which one you have decides the complexity**.
//! A `textarea` has a wrap index because it wraps; an `input` keeps the *same* index under a
//! different break rule ([`WrapKind::Ruler`]) whose only job is that a boundary is never more than
//! one window behind the caret. That is why [`WrapKind`] is a flag on one component and not two
//! components, and it is §5's `Mode` in a fourth family.
//!
//! # Three things here are absences, and each is gated as one
//!
//! **There is no `Text::set_caret(byte)`.** [`Caret`]'s fields are private and it is produced only
//! by a gesture — [`Caret::HOME`], a cluster step, a click's column, or a pair some earlier state
//! stored. An arbitrary offset does not land on a boundary, and everything downstream preserves
//! the bad caret faithfully. [`defective::at_byte`] is the door a gate takes and the only one there
//! is; [`crate::input`] carries the `compile_fail` pair that names the absent API by path.
//!
//! **There is no run-list selection.** A selection is one anchored range in *bytes*
//! ([`Text::selection`]). §11's cost argument is not contiguity — it is the **unit**: a run list
//! addresses cluster indices and a buffer addresses bytes, so every gesture pays a prefix walk.
//! [`defective::RunList`] is that spelling, kept runnable and priced.
//!
//! **There is no block selection.** It is a rectangle over *visual* rows, which neither the run
//! list nor the anchored range expresses, and it belongs to whoever writes the code editor.
//!
//! # The bound on the ring is where the silent defect lives
//!
//! A ring of sixteen entries after sixty-four edits runs sixteen steps, returns `true` every time,
//! and the document is **not** back to the original — indistinguishable from success unless the
//! ring says so. So crossing the bound sets [`Ring::truncated`], and there are **two** bounds
//! rather than one, entries *and* bytes, because one large paste is one entry.

use std::collections::VecDeque;

use vitui_runtime::layout::text::{truncate, width, wrap};

use crate::clusters::next_cluster;

// ── the break rule ───────────────────────────────────────────────────────────────────────────────

/// **Which break rule the wrap index is built under.** §11's one flag.
///
/// `input` and `textarea` are one component and this is the whole of the difference. Not two
/// functions and not two states: the index, the caret, the selection and the ring are identical
/// under both, and what changes is where a row ends.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum WrapKind {
    /// **A textarea.** Greedy word wrap, hard lines broken at `\n`.
    #[default]
    Words,
    /// **An input.** Hard breaks every `w` columns, no word breaks, no newlines.
    ///
    /// Its only job is that **a boundary is never more than one window behind the caret**, which is
    /// what makes `Left` at the end of a pasted megabyte cost one window rather than the buffer. An
    /// input holds no line breaks, so a `\n` that arrives is a cluster like any other and never a
    /// row boundary — which is also why this arm cannot be written as `Words` at a very large
    /// width: at `w = u16::MAX` a megabyte is one row and the caret is back to O(prefix).
    Ruler,
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
///    [`crate::document::Defect::MemoKeyedOnRevision`] is what happens when the key drops it.
#[derive(Clone, Debug)]
pub struct Index {
    rows: Vec<(u32, u32)>,
    revision: u64,
    width: u16,
    kind: WrapKind,
}

impl Index {
    /// **Build the index over `text` at `w` columns.**
    ///
    /// The wrapping is `vitui_runtime::layout::text::wrap` and **not a second greedy wrap written
    /// here**: a stand-in that reimplemented the break rule would make every equality below a
    /// comparison between two copies of one bug. The offsets are recovered from the subslices the
    /// iterator yields, which are slices of `text` itself.
    pub fn build(text: &str, w: u16, revision: u64) -> Index {
        Index::build_with(text, w, revision, WrapKind::Words)
    }

    /// [`Index::build`], under a stated break rule.
    ///
    /// **The two arms share every line below this function**, which is §11's one flag as code: the
    /// caret, the selection, the ring and the splice do not know which one built the rows.
    pub fn build_with(text: &str, w: u16, revision: u64, kind: WrapKind) -> Index {
        let mut rows = Vec::new();
        match kind {
            WrapKind::Words => {
                let base = text.as_ptr() as usize;
                for hard in text.split('\n') {
                    let at = hard.as_ptr() as usize - base;
                    let mut any = false;
                    for piece in wrap(hard, w) {
                        let off = piece.as_ptr() as usize - base;
                        rows.push((off as u32, (off + piece.len()) as u32));
                        any = true;
                    }
                    // An empty hard line is one visual row and `wrap` yields nothing for it. A
                    // textarea that skipped it would renumber every row below a blank line.
                    if !any {
                        rows.push((at as u32, at as u32));
                    }
                }
            }
            // **The ruler, and `truncate` is the break rule rather than a second one written
            // here.** `truncate(rest, w)` is the longest prefix that fits `w` columns over the
            // engine's own tables, which is exactly *a hard break every `w` columns* — and it is
            // the same function [`crate::text::fit`] clips with, so a row's break and a row's
            // truncation cannot disagree.
            WrapKind::Ruler => {
                let mut at = 0usize;
                while at < text.len() {
                    let piece = truncate(&text[at..], w.max(1));
                    // A cluster wider than the whole band would answer with nothing and loop for
                    // ever. One cluster a row is the only answer that makes progress, and it is
                    // what a one-column input showing a CJK glyph is.
                    let took = match piece.len() {
                        0 => step(&text[at..]).map_or(text.len() - at, str::len),
                        n => n,
                    };
                    rows.push((at as u32, (at + took) as u32));
                    at += took;
                }
                if rows.is_empty() {
                    rows.push((0, 0));
                }
            }
        }
        Index {
            rows,
            revision,
            width: w,
            kind,
        }
    }

    /// The break rule it was built under.
    pub fn kind(&self) -> WrapKind {
        self.kind
    }

    /// How many visual rows. **625 at [`crate::document::WIDE`] and 875 at
    /// [`crate::document::NARROW`]**, which is §21's pair.
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

    /// **How many bytes row `r`'s content is**, which is not the distance to the next row's start.
    pub fn row_len(&self, r: usize) -> usize {
        self.rows.get(r).map_or(0, |(s, e)| (*e - *s) as usize)
    }

    /// **How far row `r` can hold a caret**, which is *not* where its content stops and *not* where
    /// the next row starts.
    ///
    /// `vitui_runtime::layout::text::wrap` trims each piece, so the rows **do not tile the
    /// buffer**: the whitespace a greedy break consumed, and any trailing whitespace at the end of
    /// a hard line, belong to no row's *content*. They are still places a caret goes — typing a
    /// space at the end of a line puts it there — so they belong to the row before the gap.
    ///
    /// Two subtractions, and each is a case the click sweep found:
    ///
    /// 1. **A hard break is stripped**, or `End` on a line would land on the row below it.
    /// 2. **A row followed by another stops one cluster short of that row's start.** That byte is
    ///    the next row's own column 0 and [`Index::row_of`] says so, so a walk that landed on it
    ///    would leave the caret carrying one row's column while the draw put it on another's. The
    ///    last row has no such neighbour and its bound is the end of the buffer, which is how `End`
    ///    on `"hi "` reaches byte 3 and `Ctrl+A` selects the space.
    pub fn row_bound(&self, text: &str, r: usize) -> usize {
        let start = self.row_start(r);
        let Some((next, _)) = self.rows.get(r + 1) else {
            return text.len();
        };
        let next = *next as usize;
        let head = &text[..next];
        let stripped = if head.ends_with("\r\n") {
            next - 2
        } else if head.ends_with('\n') || head.ends_with('\r') {
            next - 1
        } else {
            next
        };
        if stripped < next {
            return stripped.max(start);
        }
        // No break to strip, so the gap runs right up to the next row's first byte: stop one
        // cluster short of it.
        step_left(text, Caret::of(next, 0), Caret::of(start, 0))
            .byte
            .max(start)
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
    /// The restart point is the whole question — see [`crate::document::splice_sweep`] — so it is an argument rather
    /// than a policy, and the two spellings the scene compares are `row_of(at)` and one row earlier.
    ///
    /// It rewraps to the end of the document rather than stopping when it resynchronises. That is an
    /// upper bound on the work and **exact on the answer**, which is the half this scene is about: a
    /// splice and a rebuild can only disagree about the rows the splice kept.
    pub fn spliced(&self, edited: &str, restart: usize, revision: u64) -> Index {
        // **Clamped to the last row and not to `rows.len()`.** `Index::row_start` answers 0 out of
        // range, so a restart *at* the length would keep every row as the head and rebuild the tail
        // from byte 0 — a doubled index rather than a panic. `Text::edit` never passes that value
        // and this function is `pub`.
        let restart = restart.min(self.rows.len().saturating_sub(1));
        let from = self.row_start(restart);
        let head: Vec<(u32, u32)> = self.rows[..restart].to_vec();
        let tail = Index::build_with(&edited[from..], self.width, revision, self.kind);
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
            kind: self.kind,
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
///
/// # The fields are private, and that is the API consequence §11 asks for
///
/// > **There is no such thing as "put the caret at byte N."**
///
/// A caret is placed by a gesture — a click's column, a cluster step, [`Caret::HOME`], or a pair
/// some earlier state stored — and **each of those lands on a boundary by construction**. An
/// arbitrary offset does not, and everything downstream preserves the bad caret faithfully. A
/// `pub byte` is that offset with three extra keystrokes, so the fields are read through
/// [`Caret::byte`] and [`Caret::col`] and written by [`step_right`], [`step_left`],
/// [`Text::click`], [`Text::home`], [`Text::end`] and [`Text::set_pos`] and by nothing else.
///
/// [`defective::at_byte`] is the door a gate takes. It is `pub` inside a module named for what it
/// is, because a defect nobody can build is a gate nobody can watch fire.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, PartialOrd, Ord)]
pub struct Caret {
    byte: usize,
    col: u16,
}

impl Caret {
    /// **The start of the buffer**, which is the one caret that is a boundary in every document.
    pub const HOME: Caret = Caret { byte: 0, col: 0 };

    /// Where it is, in bytes.
    pub fn byte(self) -> usize {
        self.byte
    }

    /// What column it is at, carried alongside and moved by the width of the cluster crossed.
    pub fn col(self) -> u16 {
        self.col
    }

    /// **The pair, from parts.** Crate-private, and that is the deletion: a caller outside this
    /// crate has no expression for *the caret is at byte N*, so the only carets that exist are the
    /// ones a gesture made — and [`defective::at_byte`] is the one door a gate takes through.
    pub(crate) const fn of(byte: usize, col: u16) -> Caret {
        Caret { byte, col }
    }
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
/// `from` must be a cluster boundary no further right than `caret`, carrying the column it sits
/// at. That
/// precondition is §11's whole complexity argument: `Graphemes` is a forward iterator, so the
/// boundary *before* an offset can only be found by segmenting forward from one already known, **and
/// which one you have decides the complexity.** With no index the only one you have is byte 0.
pub fn step_left(text: &str, caret: Caret, from: Caret) -> Caret {
    step_left_counted(text, caret, from, &mut 0)
}

/// **[`step_left`], with the clusters it walked counted.**
///
/// The runtime's scene 19 arrangement one crate down — `chunked.get(i, &mut hops)` — and it is here
/// for that scene's reason: *a step count is the same number on every machine*, where the
/// microseconds §11 states are three orders apart between a debug binary and a release one and
/// would be three different orders somewhere else. [`crate::volume`] is what reads it, and
/// [`defective::blind_left`] is the arm it separates: **one `Left` at the end of a pasted megabyte
/// is one cluster from the row start and a million from byte 0**, which is the whole of §11's
/// complexity argument as a count rather than as a clock.
///
/// `step_left` forwards to it with a throwaway, so there is one loop and not two — a counted copy
/// of a walk is a copy, and this crate has already found what a second transcription of one drawing
/// costs.
pub fn step_left_counted(text: &str, caret: Caret, from: Caret, steps: &mut u64) -> Caret {
    let mut at = from;
    let mut prev = at;
    while at.byte < caret.byte {
        *steps += 1;
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
pub fn step_right(text: &str, caret: Caret) -> Caret {
    match step(&text[caret.byte..]) {
        Some(cluster) => Caret {
            byte: caret.byte + cluster.len(),
            col: caret.col.saturating_add(width(cluster)),
        },
        None => caret,
    }
}

// ── the undo ring ────────────────────────────────────────────────────────────────────────────────

/// **One entry of the undo ring: `(at, removed, inserted)` plus the caret pair and the anchor the
/// edit started from.**
///
/// §11's own record. The caret pair is stored rather than recomputed for the reason the pair exists
/// at all: `set_pos(byte, col)` is **0.0007 µs** and `set_caret(byte)` — recovering the column by
/// segmenting the prefix — is **6 109 µs** on a 1 MB line. Undo restores the pair.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Entry {
    at: usize,
    removed: String,
    inserted: String,
    caret: Caret,
    anchor: usize,
}

impl Entry {
    /// Where the edit happened, in bytes.
    pub fn at(&self) -> usize {
        self.at
    }

    /// What it took out.
    pub fn removed(&self) -> &str {
        &self.removed
    }

    /// What it put in.
    pub fn inserted(&self) -> &str {
        &self.inserted
    }

    /// **What this entry costs the ring**, which is the second of the two bounds.
    ///
    /// The two strings and the fixed part. A `size_of` would report the two `String` headers and
    /// none of their bytes, which is exactly the accounting that makes one large paste look free.
    pub fn bytes(&self) -> usize {
        self.removed.len() + self.inserted.len() + FIXED
    }
}

/// The fixed part of an [`Entry`]: the offset, the pair, the anchor and the two string headers.
const FIXED: usize = std::mem::size_of::<usize>() * 4 + std::mem::size_of::<Caret>() + 32;

/// **How many entries the ring holds by default. 256**, which is the cap §11 prices the frame at.
pub const RING_ENTRIES: usize = 256;

/// **How many bytes the ring holds by default. 25 600**, §11's second bound.
///
/// Two bounds rather than one, **because one large paste is one entry**: a ring bounded only on
/// entries holds a megabyte per paste and 256 of them, and a ring bounded only on bytes drops a
/// hundred keystrokes to make room for one.
pub const RING_BYTES: usize = 25_600;

/// **The undo ring: bounded twice, coalescing, and reporting truncation.**
///
/// # `truncated` is the whole point of the type
///
/// A ring of sixteen entries after sixty-four edits runs sixteen steps, **returns `true` every
/// time**, and the document is not back to the original. That is indistinguishable from success
/// unless the ring says so — an undo that ran out of history and an undo that finished both answer
/// *yes, I undid something*. [`Ring::truncated`] is the difference, and
/// `crate::edit::tests::sixty_four_edits_through_a_sixteen_entry_ring` is it watched happening.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Ring {
    entries: VecDeque<Entry>,
    bytes: usize,
    max_entries: usize,
    max_bytes: usize,
    truncated: bool,
    coalesces: bool,
}

impl Default for Ring {
    fn default() -> Ring {
        Ring::bounded(RING_ENTRIES, RING_BYTES)
    }
}

impl Ring {
    /// A ring with both bounds spelled out.
    pub fn bounded(max_entries: usize, max_bytes: usize) -> Ring {
        Ring {
            entries: VecDeque::new(),
            bytes: 0,
            max_entries,
            max_bytes,
            truncated: false,
            coalesces: true,
        }
    }

    /// The same ring with coalescing off, which is the arm §11's **2.4×** is measured against.
    pub fn uncoalesced(mut self) -> Ring {
        self.coalesces = false;
        self
    }

    /// How many entries it holds.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether it holds none.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// **How many bytes it holds**, counting the strings and not only their headers.
    pub fn bytes(&self) -> usize {
        self.bytes
    }

    /// **Whether either bound has ever dropped an entry**, which is what makes a failed undo
    /// distinguishable from a finished one.
    pub fn truncated(&self) -> bool {
        self.truncated
    }

    /// Whether it coalesces.
    pub fn coalesces(&self) -> bool {
        self.coalesces
    }

    /// **Record an edit**, coalescing it into the last entry where it continues one.
    ///
    /// The rule is *an insertion of no line break, at exactly where the last insertion ended, into
    /// an entry that removed nothing and has not already reached a word break*. Anything else opens
    /// an entry: a deletion, a replacement, a paste that lands elsewhere, and a newline.
    ///
    /// **The word break is the rule and not a refinement of it.** §11 gives the reason in the same
    /// sentence as the ratio — *undoing a sentence is 414 presses instead of 1 012* — and a run
    /// that closed at nothing but a line break would make undoing a sentence **one** press, which
    /// is not an undo history, it is a checkpoint. Whitespace joins the run it ends, so `the ` is
    /// one entry and the `q` after it opens the next.
    ///
    /// It is not a time win and is not claimed as one.
    fn record(&mut self, entry: Entry) {
        if self.coalesces
            && entry.removed.is_empty()
            && !entry.inserted.contains('\n')
            && let Some(last) = self.entries.back_mut()
            && last.removed.is_empty()
            && last.at + last.inserted.len() == entry.at
            && !last.inserted.ends_with(char::is_whitespace)
        {
            self.bytes += entry.inserted.len();
            last.inserted.push_str(&entry.inserted);
            self.evict();
            return;
        }
        self.bytes += entry.bytes();
        self.entries.push_back(entry);
        self.evict();
    }

    /// Drop from the front until both bounds hold, recording that it happened.
    fn evict(&mut self) {
        while self.entries.len() > self.max_entries
            || (self.bytes > self.max_bytes && self.entries.len() > 1)
        {
            let Some(dropped) = self.entries.pop_front() else {
                break;
            };
            self.bytes -= dropped.bytes();
            self.truncated = true;
        }
    }

    /// The entries, oldest first.
    pub fn entries(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter()
    }
}

// ── the component's state ────────────────────────────────────────────────────────────────────────

/// **What `field` keeps: a buffer, a revision, the caret pair, the selection anchor, the undo ring
/// and the wrap memo.**
///
/// One type for `input` and `textarea`, and the only thing that separates them is [`WrapKind`].
///
/// # The revision does not move when the caret does
///
/// §11 states it as a measurement — **200 recomputes at 306.51 µs/key against 1 at 1.587** — and it
/// is a rule about which field the bump lives on: [`Text::revision`] moves in [`Text::edit`] and in
/// nothing else. A caret step, a selection, a click and a resize all leave it alone, so the wrap
/// memo is a hit on every frame where nothing was typed.
#[derive(Clone, Debug)]
pub struct Text {
    buf: String,
    revision: u64,
    caret: Caret,
    anchor: usize,
    kind: WrapKind,
    index: Option<Index>,
    ring: Ring,
    recomputes: u64,
    offset: usize,
    shape: Shape,
}

/// **The two ways a `Text` can be false that are not on any option**, as one value.
///
/// Both are §11 gates, both are cheaper than the correct build, and both are *keys* rather than
/// behaviours — which is why they are a field here and not a second machine written beside the
/// gate. A reviewer's diff between the shipped build and either refusal is one enum variant, and
/// the code that reads them is the code that ships. [`defective::keyed_on_revision`] and
/// [`defective::restarted_at_row_of`] are the only ways to set them.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct Shape {
    key: Key,
    restart: Restart,
}

/// What the wrap memo is keyed on.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum Key {
    /// **The rule.** `(revision, width)`.
    #[default]
    RevisionAndWidth,
    /// **The defect.** The revision alone, so a resize is a cache hit.
    Revision,
}

/// Where a splice restarts.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum Restart {
    /// **The rule.** One row before the edit's own.
    #[default]
    OneRowEarlier,
    /// **The defect.** The edit's own row, which keeps a break the edit invalidated.
    AtRowOf,
}

impl Default for Text {
    fn default() -> Text {
        Text::textarea()
    }
}

impl Text {
    /// **An `input`**: one row, [`WrapKind::Ruler`].
    pub fn input() -> Text {
        Text::of(String::new(), WrapKind::Ruler)
    }

    /// **A `textarea`**: greedy word wrap, hard lines at `\n`.
    pub fn textarea() -> Text {
        Text::of(String::new(), WrapKind::Words)
    }

    /// A field over content it already has.
    pub fn of(buf: String, kind: WrapKind) -> Text {
        Text {
            buf,
            revision: 0,
            caret: Caret::HOME,
            anchor: 0,
            kind,
            index: None,
            ring: Ring::default(),
            recomputes: 0,
            offset: 0,
            shape: Shape::default(),
        }
    }

    /// What it holds.
    pub fn text(&self) -> &str {
        &self.buf
    }

    /// How many bytes it holds.
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Whether it holds nothing.
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// The break rule.
    pub fn kind(&self) -> WrapKind {
        self.kind
    }

    /// **The revision, which moves on an edit and on nothing else.**
    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// The caret.
    pub fn caret(&self) -> Caret {
        self.caret
    }

    /// **The selection, as one anchored range in bytes.** Empty when the anchor is at the caret.
    pub fn selection(&self) -> std::ops::Range<usize> {
        let (a, b) = (
            self.anchor.min(self.caret.byte),
            self.anchor.max(self.caret.byte),
        );
        a..b
    }

    /// What the selection covers.
    pub fn selected(&self) -> &str {
        &self.buf[self.selection()]
    }

    /// The undo ring.
    pub fn ring(&self) -> &Ring {
        &self.ring
    }

    /// The first visual row drawn. `field` owns its offset (§17's `owns_offset`).
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// **How many times the wrap memo has recomputed.** §11's *1 against 200*, as a counter.
    pub fn recomputes(&self) -> u64 {
        self.recomputes
    }

    /// **Restore a caret pair somebody else stored.** The undo path, and §11's `set_pos`.
    ///
    /// It takes a [`Caret`] rather than a `(usize, u16)` and that is the whole safety: a `Caret`
    /// is produced only by a gesture, so a pair restored here was a boundary when it was made.
    pub fn set_pos(&mut self, at: Caret) {
        self.caret = at;
        self.anchor = at.byte;
    }

    /// Give the ring different bounds.
    pub fn with_ring(mut self, ring: Ring) -> Text {
        self.ring = ring;
        self
    }

    // ── the index, memoised ──────────────────────────────────────────────────────────────────────

    /// **The wrap index at `w`, keyed on `(revision, width)`.**
    ///
    /// Both halves of the key, and the second one is §11's fourth gate: *the index's recorded width
    /// equals the width being drawn*. A memo keyed on the revision alone is a **hit** on a resize —
    /// the revision did not move — and what comes back was built at the old width.
    /// [`defective::stale_at`] is that key, and `crate::document`'s scene 13 is it drawn.
    pub fn index(&mut self, w: u16) -> &Index {
        // **The defect the key carries is in [`Text::index_of`]**, which this delegates to, so the
        // memo key is written once: with `Key::Revision` the width is not in it, a resize is a
        // **hit**, and what comes back was built at the old width — `recomputes` then reports the
        // defect as the cheaper build.
        let buf = std::mem::take(&mut self.buf);
        let _ = self.index_of(&buf, w);
        self.buf = buf;
        self.index.as_ref().expect("just built")
    }

    /// The index it has, without building one.
    pub fn indexed(&self) -> Option<&Index> {
        self.index.as_ref()
    }

    // ── the gestures ─────────────────────────────────────────────────────────────────────────────

    /// **`Right`, one cluster — and the column is re-seated on the row it lands in.**
    ///
    /// The byte is one forward step and costs nothing, which is why §11 is about `Left`. The
    /// **column** is not: a caret's column is a *screen* column, reset at every row start, and a
    /// greedy break consumes the space it broke at — so a step that crossed a row boundary and kept
    /// accumulating would report a column no cell is at. It is re-read from the caret's own row
    /// start, which the index makes one row rather than one prefix. That is §11's headline arriving
    /// on the cheap direction: *the index that answers them is the one the wrapping already needs.*
    pub fn right(&mut self, w: u16, extend: bool) {
        let to = step_right(&self.buf, self.caret).byte();
        self.caret = self.walked_to(w, to);
        self.settle(extend);
    }

    /// **`Left`, one cluster — and the index is what makes it one row rather than one buffer.**
    ///
    /// §11's headline measurement: **3 161.68 µs with no index against 0.327 µs with one**. The
    /// boundary before the caret is found by segmenting forward from the start of the caret's own
    /// visual row, and a caret sitting *on* a row start is answered from the row before — a `Left`
    /// at the end of a document that ends in a line break would otherwise answer *the caret did not
    /// move*, which looks exactly like a correct `Left` at the end of a document.
    ///
    /// With no index there is one boundary known and it is byte 0, which is
    /// [`defective::blind_left`].
    pub fn left(&mut self, w: u16, extend: bool) {
        let from = self.row_before(w);
        self.caret = step_left(&self.buf, self.caret, from);
        self.settle(extend);
    }

    /// The caret's own row start as a pair, stepping back one row when the caret sits on one.
    fn row_before(&mut self, w: u16) -> Caret {
        let byte = self.caret.byte;
        let index = self.index(w);
        let row = index.row_of(byte);
        let row = if index.row_start(row) >= byte && row > 0 {
            row - 1
        } else {
            row
        };
        Caret {
            byte: index.row_start(row),
            col: 0,
        }
    }

    /// **`Home`**: the start of the caret's visual row.
    pub fn home(&mut self, w: u16, extend: bool) {
        let at = self.caret.byte;
        let index = self.index(w);
        let byte = index.row_start(index.row_of(at));
        self.caret = Caret { byte, col: 0 };
        self.settle(extend);
    }

    /// **`End`**: the end of the caret's visual row, walked in cluster steps from its start.
    pub fn end(&mut self, w: u16, extend: bool) {
        let at = self.caret.byte;
        let (from, to) = self.row_span(w, |index| index.row_of(at));
        let mut at = Caret { byte: from, col: 0 };
        while at.byte < to {
            at = step_right(&self.buf, at);
        }
        self.caret = at;
        self.settle(extend);
    }

    /// **`Up` and `Down`: the same column on the row either side.**
    ///
    /// A visual row and not a hard line, which is the whole reason the index carries visual rows:
    /// in a textarea `Down` from the first half of a wrapped line lands in its second half, and a
    /// caret that stepped by hard lines would jump a screenful.
    ///
    /// The target column is the caret's own, and the landing is a [`Text::click`] on it — so the
    /// caret lands on a boundary by construction rather than at a byte the arithmetic produced.
    pub fn step_row(&mut self, w: u16, up: bool, extend: bool) {
        let col = self.caret.col;
        let at = self.caret.byte;
        let row = self.index(w).row_of(at);
        let to = if up {
            row.saturating_sub(1)
        } else {
            (row + 1).min(self.index(w).rows().saturating_sub(1))
        };
        self.click(w, to, col, extend);
    }

    /// **A click, which places the caret by a column and never by an offset.**
    ///
    /// `row` is a visual row and `col` a column inside it; the caret lands on the last boundary at
    /// or before that column, which is a boundary by construction — §11's first gesture.
    pub fn click(&mut self, w: u16, row: usize, col: u16, extend: bool) {
        let (from, to) = self.row_span(w, |index| row.min(index.rows().saturating_sub(1)));
        let mut at = Caret { byte: from, col: 0 };
        while at.byte < to {
            let next = step_right(&self.buf, at);
            if next.col > col {
                break;
            }
            at = next;
        }
        self.caret = at;
        self.settle(extend);
    }

    /// **Where one visual row starts and how far a caret on it may go**, in bytes.
    ///
    /// [`Index::row_bound`] and **not** `row_start + row_len`: the rows do not tile the buffer, so
    /// a walk that stopped at the content's end could not reach a trailing space — `End` on `"hi "`
    /// would land at byte 2 and `Ctrl+A` would select `"hi"`.
    fn row_span(&mut self, w: u16, pick: impl Fn(&Index) -> usize) -> (usize, usize) {
        let buf = std::mem::take(&mut self.buf);
        let index = self.index_of(&buf, w);
        let r = pick(index);
        let start = index.row_start(r);
        let to = index.row_bound(&buf, r);
        self.buf = buf;
        (start, to)
    }

    /// [`Text::index`] over a buffer the caller is holding, which is how a walk reads the index and
    /// the bytes at once without copying either.
    fn index_of(&mut self, buf: &str, w: u16) -> &Index {
        let fresh = match (&self.index, self.shape.key) {
            (Some(i), Key::RevisionAndWidth) => i.revision() == self.revision && i.built_at() == w,
            (Some(i), Key::Revision) => i.revision() == self.revision,
            (None, _) => false,
        };
        if !fresh {
            self.recomputes += 1;
            self.index = Some(Index::build_with(buf, w, self.revision, self.kind));
        }
        self.index.as_ref().expect("just built")
    }

    /// Put the anchor where the caret is unless the gesture extends the selection.
    fn settle(&mut self, extend: bool) {
        if !extend {
            self.anchor = self.caret.byte;
        }
    }

    /// **Select everything.** The anchor at 0 and the caret at the end, both boundaries.
    ///
    /// The end of a buffer is a boundary in every document, and the column it sits at is the one a
    /// walk of its own row gives — which is [`Text::end`] on the last row, reached through the
    /// gesture rather than by writing a pair.
    pub fn select_all(&mut self, w: u16) {
        self.anchor = 0;
        self.caret = Caret::HOME;
        let last = self.index(w).rows().saturating_sub(1);
        self.click(w, last, u16::MAX, true);
    }

    // ── the edits ────────────────────────────────────────────────────────────────────────────────

    /// **Replace the selection with `s`.** The one edit verb, and the only place the revision moves.
    ///
    /// Every other edit is this: typing is a replacement of an empty selection, `Backspace` is a
    /// replacement of the cluster before the caret with nothing, `Delete` is the cluster after.
    /// One verb rather than four is what makes *the ring records `(at, removed, inserted)`* one
    /// statement instead of four that can drift.
    pub fn edit(&mut self, w: u16, s: &str) {
        let range = self.selection();
        let at = range.start;
        let removed = self.buf[range.clone()].to_string();
        let before = self.caret;
        let anchor = self.anchor;

        // **The restart point is computed against the index of the buffer *before* the edit**, and
        // it is one row earlier than the edit's own. See [`Index::spliced`] and
        // [`defective::splice_at_row_of`]: a greedy break depends on the text after it, so an
        // insertion at an offset greater than a break can move that break — and the break that
        // closed the row the edit landed in is the one the naive point keeps.
        let restart = self
            .index
            .as_ref()
            .filter(|i| i.revision() == self.revision && i.built_at() == w)
            .map(|i| match self.shape.restart {
                Restart::OneRowEarlier => i.row_of(at).saturating_sub(1),
                Restart::AtRowOf => i.row_of(at),
            });

        self.buf.replace_range(range, s);
        self.revision += 1;
        self.ring.record(Entry {
            at,
            removed,
            inserted: s.to_string(),
            caret: before,
            anchor,
        });

        // **The index, spliced or dropped, before the caret is placed** — the caret's column is
        // read off its own visual row, so the row it is read off has to be the new one.
        self.index = match restart {
            Some(row) => {
                self.recomputes += 1;
                let rev = self.revision;
                let buf = std::mem::take(&mut self.buf);
                let spliced = self.index.as_ref().map(|i| i.spliced(&buf, row, rev));
                self.buf = buf;
                spliced
            }
            None => None,
        };

        // **The caret lands after what was inserted**, walked in cluster steps from the start of
        // its own visual row — so its column is a screen column and every step from a boundary is
        // one too.
        self.caret = self.walked_to(w, at + s.len());
        self.anchor = self.caret.byte;
    }

    /// The pair at `target`, walked from the start of the visual row `target` falls in.
    ///
    /// `target` is always the end of an edit or of a cluster step, so it is a boundary by
    /// construction — this walks to find the **column**, which is the half a byte offset does not
    /// carry.
    fn walked_to(&mut self, w: u16, target: usize) -> Caret {
        let from = {
            let index = self.index(w);
            index.row_start(index.row_of(target))
        };
        let mut c = Caret { byte: from, col: 0 };
        while c.byte < target {
            let next = step_right(&self.buf, c);
            if next.byte == c.byte {
                break;
            }
            c = next;
        }
        c
    }

    /// **Type a cluster.** `keys::text` decides what reaches here; a chord types nothing.
    pub fn insert(&mut self, w: u16, s: &str) {
        self.edit(w, s);
    }

    /// **`Backspace`**: the cluster before the caret, or the selection where there is one.
    ///
    /// **At the start of the buffer it does nothing at all**, and that is a guard rather than an
    /// arm: without it the edit removes nothing, bumps the revision — invalidating the wrap memo —
    /// and records an entry that undoes nothing, so holding `Backspace` on an empty document fills
    /// the ring with edits and pushes the real history out of it.
    pub fn backspace(&mut self, w: u16) {
        if self.selection().is_empty() {
            let from = self.row_before(w);
            let prev = step_left(&self.buf, self.caret, from);
            if prev.byte == self.caret.byte {
                return;
            }
            self.anchor = prev.byte;
        }
        self.edit(w, "");
    }

    /// **`Delete`**: the cluster after the caret, or the selection where there is one.
    ///
    /// At the end of the buffer it does nothing, for [`Text::backspace`]'s reason.
    pub fn delete(&mut self, w: u16) {
        if self.selection().is_empty() {
            let next = step_right(&self.buf, self.caret);
            if next.byte == self.caret.byte {
                return;
            }
            self.anchor = next.byte;
        }
        self.edit(w, "");
    }

    /// **Undo one entry, and answer whether one was applied.**
    ///
    /// `false` where the ring is empty. It is **not** `false` where the ring truncated — the entry
    /// it applied was real — which is exactly why [`Ring::truncated`] exists beside the answer.
    pub fn undo(&mut self) -> bool {
        let Some(entry) = self.ring.entries.pop_back() else {
            return false;
        };
        self.ring.bytes -= entry.bytes();
        let end = entry.at + entry.inserted.len();
        self.buf.replace_range(entry.at..end, &entry.removed);
        self.revision += 1;
        // **The pair, restored rather than recomputed.** §11's 0.0007 µs against 6 109.
        self.caret = entry.caret;
        self.anchor = entry.anchor;
        self.index = None;
        true
    }

    /// **Put the window at a visual row**, which is what a scroll gesture and a caller's own
    /// scrollbar do to a field that owns its offset.
    pub fn scroll_to(&mut self, row: usize) {
        self.offset = row;
    }

    /// **The wheel, in visual rows.** A notch up or down, clamped to the index.
    ///
    /// A field declares `Interest::SCROLL` and owns its offset (§17), so it must **consume** the
    /// notch: a widget that declares the pointer and does nothing with it is worse than one that
    /// declares nothing, because it is the topmost region over its rectangle and an enclosing
    /// `scroll_area` never sees the notch either. That is components ticket 20's defect class, and
    /// this is the line that keeps `field` out of it.
    pub fn wheel(&mut self, w: u16, rows: u16, notches: i32) {
        if notches == 0 {
            return;
        }
        let last = self.last_offset(w, rows);
        let to = i64::from(notches).saturating_add(self.offset as i64);
        self.offset = to.clamp(0, last as i64) as usize;
    }

    /// The largest offset that still shows content. A window scrolled past the end shows blank
    /// filler rows whose start is row 0's, which is a caret painted on nothing.
    fn last_offset(&mut self, w: u16, rows: u16) -> usize {
        self.index(w)
            .rows()
            .saturating_sub(usize::from(rows.max(1)))
    }

    /// **Bring the caret's row into view**, and answer the row it is on.
    ///
    /// Conditional, which is `CONTEXT.md`'s rule and the wheel defect's one component over: a
    /// widget that scrolls to its caret unconditionally drags the viewport back the moment the user
    /// scrolls away from it.
    pub fn reveal(&mut self, w: u16, rows: u16) -> usize {
        let at = self.caret.byte;
        let row = self.index(w).row_of(at);
        let h = usize::from(rows.max(1));
        if row < self.offset {
            self.offset = row;
        } else if row >= self.offset + h {
            self.offset = row + 1 - h;
        }
        // **Clamped, and it is not tidiness.** A window past the last row shows filler whose
        // `row_start` is row 0's — `Index::row_start` answers 0 out of range — so the caret would be
        // painted on a blank row and the selection compared against row 0's range.
        let last = self.last_offset(w, rows);
        self.offset = self.offset.min(last);
        row
    }
}

// ── the refused spellings, kept runnable ─────────────────────────────────────────────────────────

/// **The four builds §11 refuses, each one somebody would ship.**
///
/// Every arm is simpler than the correct one and three of the four are cheaper, which is the
/// property that makes a gate over them worth having. None of them is a strawman: the caret at a
/// byte offset is the API every text buffer in the survey has, the run list is §5's own selection
/// one family over, the naive splice restart is the one a reader of §10 writes, and the memo keyed
/// on the revision is the key a reader of `CONTEXT.md`'s **Memo** paragraph writes.
pub mod defective {
    use super::{Caret, Index, Text, WrapKind, step, step_right};
    use vitui_runtime::layout::text::width;

    /// **The deleted API, built anyway so a gate can watch it fail.** §11's *put the caret at byte
    /// N*.
    ///
    /// The column is the engine's tables over the prefix, which is the *expensive* correct answer —
    /// so this arm is wrong on gate 1 and right on gate 2, and the two are separable. The caret it
    /// returns is a boundary only by luck.
    pub fn at_byte(text: &str, byte: usize) -> Caret {
        Caret::of(byte, width(&text[..byte]))
    }

    /// **The column counted in code points.** Gate 2's defect: one column a code point.
    ///
    /// Right on ASCII and wrong on every cluster in [`crate::clusters::corpus`], which is why a
    /// gate that runs on ASCII has said nothing.
    pub fn column_by_chars(text: &str, byte: usize) -> Caret {
        Caret::of(
            byte,
            u16::try_from(text[..byte].chars().count()).unwrap_or(u16::MAX),
        )
    }

    /// **`Left` with no index**: the only boundary known is byte 0.
    ///
    /// Correct, and **3 161.68 µs at the end of a pasted megabyte against 0.327**. §11's whole
    /// complexity argument, as the arm that does the work.
    pub fn blind_left(text: &str, caret: Caret) -> Caret {
        blind_left_counted(text, caret, &mut 0)
    }

    /// **[`blind_left`], with the clusters it walked counted.** See
    /// [`super::step_left_counted`], whose twin this is.
    ///
    /// O6 reads this arm and the shipped one over the same buffer at the same three volumes, and
    /// the two numbers are *the same claim §11 makes about the two microsecond figures*: the walk
    /// from byte 0 is the buffer and the walk from the row start is a row.
    pub fn blind_left_counted(text: &str, caret: Caret, steps: &mut u64) -> Caret {
        super::step_left_counted(text, caret, Caret::HOME, steps)
    }

    /// **The splice restarted at the edit's own row.** Gate 3's defect.
    ///
    /// It agreed with a rebuild 499 times in 500 and the screen looked right. A greedy break
    /// depends on text *after* it, so an insertion at an offset greater than a break can move that
    /// break — and the break that closed the row the edit landed in is the one this keeps.
    pub fn splice_at_row_of(index: &Index, edited: &str, at: usize, revision: u64) -> Index {
        index.spliced(edited, index.row_of(at), revision)
    }

    /// **The memo keyed on the revision alone.** Gate 4's defect.
    ///
    /// A resize is a cache **hit**, so it recomputes *once* where the correct key recomputes twice —
    /// and what comes back was built at the old width. `recomputes` points the wrong way.
    pub fn stale_at(st: &Text, w: u16) -> Index {
        match st.indexed() {
            Some(i) if i.revision() == st.revision() => i.clone(),
            _ => Index::build_with(st.text(), w, st.revision(), st.kind()),
        }
    }

    /// **The selection as a run list over cluster indices**, which is §5's spelling one family over.
    ///
    /// A text selection is contiguous by construction, so this is always exactly one run — and
    /// **that is not the cost argument**. The cost argument is the *unit*: a run addresses cluster
    /// indices and a buffer addresses bytes, so every gesture pays a prefix walk to convert. This
    /// type is that walk, counted.
    #[derive(Clone, Default, Debug)]
    pub struct RunList {
        runs: Vec<(usize, usize)>,
        walked: u64,
    }

    impl RunList {
        /// A run list holding one run of cluster indices.
        pub fn of(lo: usize, hi: usize) -> RunList {
            RunList {
                runs: vec![(lo, hi)],
                walked: 0,
            }
        }

        /// **The byte range the run names**, which costs a prefix walk every time it is asked.
        pub fn bytes(&mut self, text: &str) -> std::ops::Range<usize> {
            let (lo, hi) = self.runs.first().copied().unwrap_or((0, 0));
            let mut at = 0usize;
            let mut i = 0usize;
            let mut start = 0usize;
            while i < hi {
                let Some(cluster) = step(&text[at..]) else {
                    break;
                };
                at += cluster.len();
                i += 1;
                self.walked += 1;
                if i == lo {
                    start = at;
                }
            }
            if lo == 0 {
                start = 0;
            }
            start..at
        }

        /// **How many clusters the conversions have walked.** Zero is what the anchored range costs.
        pub fn walked(&self) -> u64 {
            self.walked
        }
    }

    /// **An `input` spelled as a very wide `Words` index**, which is the reduction that looks like
    /// the flag and is not.
    ///
    /// One row at `u16::MAX` columns is one row over the whole buffer, so the caret's row start is
    /// byte 0 and `Left` is back to O(prefix). The flag is a break rule and not a width.
    pub fn ruler_as_a_wide_words_index(text: &str, revision: u64) -> Index {
        Index::build_with(text, u16::MAX, revision, WrapKind::Words)
    }

    /// **A caret advanced one `char` at a time**, which is *faster per step* and comes apart on a
    /// cluster. Kept here beside the other three; `crate::document::char_walk` is the sweep.
    pub fn char_step(text: &str, caret: Caret) -> Caret {
        match text[caret.byte..].chars().next() {
            Some(c) => Caret::of(caret.byte + c.len_utf8(), caret.col().saturating_add(1)),
            None => caret,
        }
    }

    /// **Key the wrap memo on the revision alone.** Gate 4's defect, installed on a real state.
    ///
    /// One field, so the shipped build and this one are the same function — `crate::document`'s
    /// scene 13 draws through [`crate::input::field`] either way, and the reviewer's diff between
    /// the two arms is this call.
    pub fn keyed_on_revision(st: &mut Text) {
        st.shape.key = super::Key::Revision;
    }

    /// **Restart a splice at the edit's own row.** Gate 3's defect, installed on a real state.
    pub fn restarted_at_row_of(st: &mut Text) {
        st.shape.restart = super::Restart::AtRowOf;
    }

    /// A cluster step, re-exported so a sweep can compare the two without leaving this module.
    pub fn cluster_step(text: &str, caret: Caret) -> Caret {
        step_right(text, caret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clusters;
    use std::time::Instant;

    /// A document of `lines` lines of ASCII prose, deterministic and long enough to wrap.
    fn prose(lines: usize) -> String {
        let mut out = String::new();
        for i in 0..lines {
            out.push_str("the quick brown fox jumps over the lazy dog ");
            out.push_str(&i.to_string());
            out.push('\n');
        }
        out
    }

    /// **Criterion 1: one component, one flag.**
    ///
    /// `input` and `textarea` are not two functions with two states, and the check is a `size_of`
    /// rather than a sentence: **the two states are the same type**, so there is nothing a
    /// `textarea` carries that an `input` does not. What differs is one byte of enum, and it
    /// differs in the *rows*, which is the only place a break rule can show.
    #[test]
    fn an_input_and_a_textarea_are_one_type_and_one_flag() {
        let one = Text::input();
        let many = Text::textarea();
        assert_eq!(std::mem::size_of_val(&one), std::mem::size_of_val(&many));
        assert_eq!(one.kind(), WrapKind::Ruler);
        assert_eq!(many.kind(), WrapKind::Words);

        // The same buffer under the two rules: the ruler breaks at the column and the words rule
        // breaks at the space, and neither is the other at any width.
        let text = "the quick brown fox";
        let ruler = Index::build_with(text, 10, 0, WrapKind::Ruler);
        let words = Index::build_with(text, 10, 0, WrapKind::Words);
        assert_eq!(ruler.row_text(text, 0), "the quick ");
        assert_eq!(words.row_text(text, 0), "the quick");
        assert_eq!(ruler.rows(), 2);
        assert_eq!(words.rows(), 2);
    }

    /// **The reduction that looks like the flag and is not: an `input` as a very wide `Words`
    /// index.**
    ///
    /// One row over the whole buffer, so the caret's row start is byte 0 and `Left` is back to
    /// O(prefix) — which is the one thing the flag exists to prevent. The ruler's job is stated as
    /// *a boundary is never more than one window behind the caret*, and this arm has one boundary
    /// and it is byte 0.
    #[test]
    fn an_input_spelled_as_a_wide_words_index_has_one_boundary_and_it_is_byte_zero() {
        let text = "x".repeat(40_000);
        let wide = defective::ruler_as_a_wide_words_index(&text, 0);
        let ruler = Index::build_with(&text, 80, 0, WrapKind::Ruler);
        assert_eq!(wide.rows(), 1, "one row over the whole buffer");
        assert_eq!(ruler.rows(), 500);
        // The row start before the end of the buffer: byte 0 against one window back.
        let end = text.len();
        assert_eq!(wide.row_start(wide.row_of(end)), 0);
        assert_eq!(ruler.row_start(ruler.row_of(end - 1)), 39_920);
    }

    /// **Criterion 2, and it is the four cluster gates over the corpus rather than over ASCII.**
    ///
    /// Gate 1 — the caret is always on a cluster boundary — and gate 2 — its column equals the
    /// engine's tables over the same prefix — swept over every gesture that places a caret. The
    /// corpus is [`crate::clusters::corpus`]: a combining acute, two marks on one base, a ZWJ
    /// family, a skin-tone modifier, `U+FE0F`, a regional-indicator pair and CJK.
    #[test]
    fn every_gesture_lands_on_a_boundary_at_the_engines_own_column() {
        let corpus = clusters::corpus();
        let mut st = Text::of(corpus.text().to_string(), WrapKind::Words);
        let w = 40u16;
        // A caret step does not build an index — it is a read of one, and building it here is what
        // the component's draw does before the loop.
        let _ = st.index(w);
        let check = |st: &Text, what: &str| {
            let c = st.caret();
            assert!(
                corpus.is_boundary(c.byte()),
                "{what}: the caret is at {} and that is not a cluster boundary",
                c.byte()
            );
            let index = st.indexed().expect("a gesture built one");
            let row = index.row_of(c.byte());
            let start = index.row_start(row);
            assert_eq!(
                c.col(),
                width(&st.text()[start..c.byte()]),
                "{what}: the caret's column is not the engine's tables over its own row"
            );
        };
        for step in 0..corpus.len() {
            st.right(w, false);
            check(&st, &format!("right {step}"));
        }
        for step in 0..corpus.len() {
            st.left(w, false);
            check(&st, &format!("left {step}"));
        }
        for row in 0..4usize {
            for col in 0..w {
                st.click(w, row, col, false);
                check(&st, &format!("click {row}:{col}"));
            }
        }
        st.end(w, false);
        check(&st, "end");
        st.home(w, false);
        check(&st, "home");
    }

    /// **The two caret defects, watched failing the two gates they are about.**
    ///
    /// A gate whose defective arm is not built is a gate that has never been run. `at_byte` lands
    /// inside a cluster and `column_by_chars` counts code points — right on ASCII, and the reason
    /// a caret gate run on ASCII has said nothing.
    #[test]
    fn the_two_caret_defects_fail_the_two_caret_gates_and_ascii_hides_both() {
        let corpus = clusters::corpus();
        let text = corpus.text();
        let mut off = 0usize;
        let mut wrong_column = 0usize;
        for byte in 0..text.len() {
            if !text.is_char_boundary(byte) {
                continue;
            }
            let c = defective::at_byte(text, byte);
            if !corpus.is_boundary(c.byte()) {
                off += 1;
            }
            if defective::column_by_chars(text, byte).col() != width(&text[..byte]) {
                wrong_column += 1;
            }
        }
        assert!(off > 0, "the off-boundary arm never left a boundary");
        assert!(wrong_column > 0, "the code-point column is never wrong");

        // The same sweep over ASCII: the off-boundary arm still fires — every byte of ASCII is a
        // boundary, so it does not — and the column arm reports **nothing at all**.
        let ascii = clusters::ascii_corpus();
        let text = ascii.text();
        let mut ascii_off = 0usize;
        let mut ascii_column = 0usize;
        for byte in 0..text.len() {
            if !ascii.is_boundary(byte) {
                ascii_off += 1;
            }
            if defective::column_by_chars(text, byte).col() != width(&text[..byte]) {
                ascii_column += 1;
            }
        }
        assert_eq!((ascii_off, ascii_column), (0, 0), "ASCII says nothing");
    }

    /// **Criterion 4: the selection is one anchored range in bytes, and the run list pays a prefix
    /// walk per gesture.**
    ///
    /// §11's cost argument is the **unit** and not contiguity: a run addresses cluster indices and
    /// a buffer addresses bytes. Fifty select-half-and-replace edits, counted rather than timed —
    /// the anchored range walks **nothing** and the run list walks a prefix every time.
    #[test]
    fn the_anchored_range_walks_nothing_and_the_run_list_walks_a_prefix_every_gesture() {
        let text = prose(400);
        let clusters = boundaries(&text).len() - 1;

        let mut st = Text::of(text.clone(), WrapKind::Words);
        let before = st.recomputes();
        for _ in 0..50 {
            st.select_all(300);
            let _ = st.selected().len();
        }
        // The range is two `usize`s of the buffer's own unit, so asking what it covers is a slice.
        assert_eq!(st.selection().start, 0);
        assert!(
            st.recomputes() - before <= 1,
            "one index, not one a gesture"
        );

        let mut runs = defective::RunList::of(0, clusters / 2);
        for _ in 0..50 {
            let _ = runs.bytes(&text);
        }
        assert_eq!(
            runs.walked(),
            (clusters as u64 / 2) * 50,
            "the run list walks half the clusters fifty times"
        );
    }

    /// **Criterion 5: the ring is bounded on entries and on bytes, and a truncated undo says so.**
    ///
    /// Sixty-four edits through a sixteen-entry ring. Sixteen undos each return `true`, the
    /// document is **not** back to the original, and the only thing that separates that from
    /// success is [`Ring::truncated`].
    #[test]
    fn sixty_four_edits_through_a_sixteen_entry_ring() {
        let original = String::from("start");
        let mut st = Text::of(original.clone(), WrapKind::Ruler)
            .with_ring(Ring::bounded(16, usize::MAX).uncoalesced());
        st.end(40, false);
        for i in 0..64u32 {
            st.insert(40, &format!(" {i}"));
        }
        assert_eq!(st.ring().len(), 16);
        assert!(
            st.ring().truncated(),
            "the ring dropped entries and says so"
        );

        let mut applied = 0;
        while st.undo() {
            applied += 1;
        }
        assert_eq!(applied, 16, "sixteen steps, each answering yes");
        assert_ne!(
            st.text(),
            original,
            "the document is not back, which is what the ring reports and the answer does not"
        );

        // The byte bound is the second one, **because one large paste is one entry**: a ring with
        // room for sixteen entries and 64 bytes drops on the paste rather than on the sixteenth.
        let mut st =
            Text::of(String::new(), WrapKind::Ruler).with_ring(Ring::bounded(16, 64).uncoalesced());
        st.insert(40, &"x".repeat(4_096));
        st.insert(40, "y");
        assert!(st.ring().truncated());
        assert_eq!(
            st.ring().len(),
            1,
            "the paste was evicted by the byte bound"
        );
    }

    /// **Coalescing is an entry count and not a time win.**
    ///
    /// §11 states **2.4×** over 1 012 keystrokes and gives the reason in the same sentence: undoing
    /// a sentence is 414 presses instead of 1 012. What is asserted is the direction and the
    /// mechanism — a run of typing is one entry and a word break opens the next — and the ratio is
    /// printed by `examples/field_numbers.rs` rather than pinned here.
    #[test]
    fn coalescing_is_an_entry_count_and_a_word_break_opens_the_next_entry() {
        let sentence = "the quick brown fox jumps over the lazy dog";
        let mut coalesced = Text::of(String::new(), WrapKind::Ruler)
            .with_ring(Ring::bounded(usize::MAX, usize::MAX));
        let mut flat = Text::of(String::new(), WrapKind::Ruler)
            .with_ring(Ring::bounded(usize::MAX, usize::MAX).uncoalesced());
        let mut keystrokes = 0u32;
        for c in sentence.chars() {
            let s = c.to_string();
            coalesced.insert(200, &s);
            flat.insert(200, &s);
            keystrokes += 1;
        }
        assert_eq!(flat.ring().len(), keystrokes as usize);
        assert!(
            coalesced.ring().len() < flat.ring().len(),
            "coalescing did not reduce the entry count"
        );
        // A run of typing is one entry; the buffer is identical either way.
        assert_eq!(coalesced.text(), flat.text());
        assert!(coalesced.ring().bytes() < flat.ring().bytes());

        // **A line break opens an entry**, because the reason to coalesce is that undoing a
        // sentence is one press and a sentence ends at a line.
        let before = coalesced.ring().len();
        coalesced.insert(200, "\n");
        coalesced.insert(200, "x");
        assert_eq!(coalesced.ring().len(), before + 2);
    }

    /// **Criterion 3: undo restores the pair, and recovering it from the byte is the negative
    /// case.**
    ///
    /// A caret restored from an offset has to segment the prefix to find its column, which is
    /// §11's **6 109 µs against 0.0007** on a 1 MB line. The gate is the *work*: restoring the pair
    /// crosses nothing and recovering it crosses the prefix.
    #[test]
    fn undo_restores_the_pair_and_the_byte_spelling_has_to_segment_the_prefix() {
        let mut st = Text::of("x".repeat(200_000), WrapKind::Ruler);
        let last = st.index(80).rows() - 1;
        st.click(80, last, u16::MAX, false);
        let before = st.caret();
        assert!(before.byte() > 0 && before.col() > 0);
        st.insert(80, "z");
        assert!(st.undo());
        assert_eq!(st.caret(), before, "the pair came back whole");

        // The byte spelling: the column is not carried, so it is recomputed — and the only way to
        // recompute it is to walk what is in front of it.
        let text = st.text().to_string();
        let walked = boundaries(&text[..before.byte()]).len() - 1;
        assert!(
            walked > 100_000,
            "recovering the column from the byte crosses {walked} clusters, which is not a prefix"
        );
    }

    /// **Criterion 7: the memo is keyed on `(revision, width)`, and the revision does not move when
    /// the caret does.**
    ///
    /// §11's *200 recomputes at 306.51 µs/key against 1 at 1.587*, as a count. Two hundred caret
    /// steps over one document at one width cost **one** build; a revision that moved with the
    /// caret would cost two hundred.
    #[test]
    fn two_hundred_caret_steps_are_one_index_and_a_resize_is_a_second() {
        let mut st = Text::of(prose(600), WrapKind::Words);
        let _ = st.index(300);
        assert_eq!(st.recomputes(), 1);
        let revision = st.revision();
        for _ in 0..200 {
            st.right(300, false);
            let _ = st.index(300);
        }
        assert_eq!(st.recomputes(), 1, "a caret step is not an edit");
        assert_eq!(st.revision(), revision, "and it did not move the revision");

        // **The width is the second half of the key.** The same revision at another width is a
        // miss, which is §11's fourth gate: the index's recorded width equals the width drawn.
        let _ = st.index(120);
        assert_eq!(st.recomputes(), 2);
        assert_eq!(st.indexed().expect("built").built_at(), 120);

        // And the defective key is a **hit** on the resize, so what comes back was built at 300.
        let _ = st.index(300);
        let stale = defective::stale_at(&st, 120);
        assert_eq!(stale.built_at(), 300, "keyed on the revision alone");
    }

    /// **Criterion 8: the splice restarts one row before the edit's, and the naive point does not
    /// agree with a rebuild.**
    ///
    /// The five hundred deterministic edits `crate::document::splice_sweep` plays, driven through
    /// the shipped [`Text::edit`] rather than through a splice written beside the gate. The
    /// component's own splice agrees with a rebuild five hundred times in five hundred.
    #[test]
    fn the_shipped_splice_agrees_with_a_rebuild_five_hundred_times_in_five_hundred() {
        let sweep = crate::document::splice_sweep();
        assert_eq!(sweep.one_row_earlier, 0, "the shipped restart point");
        assert!(
            sweep.at_row_of > 0,
            "the naive point must be watched disagreeing, or the population cannot see it"
        );

        // The same claim over the component: every edit of the sweep, applied through `Text::edit`,
        // leaves an index a rebuild agrees with.
        let base = crate::document::splice_document();
        let w = crate::document::SPLICE_W;
        let mut disagreed = 0usize;
        for at in (0..base.len()).step_by(97) {
            let mut st = Text::of(base.clone(), WrapKind::Words);
            let _ = st.index(w);
            st.click(w, st.indexed().expect("built").row_of(at), 0, false);
            st.insert(w, "z");
            let rebuilt = Index::build_with(st.text(), w, st.revision(), WrapKind::Words);
            if !st.indexed().expect("spliced").same_rows(&rebuilt) {
                disagreed += 1;
            }
        }
        assert_eq!(disagreed, 0, "the shipped splice disagreed with a rebuild");
    }

    /// **Criterion 9: a keystroke at 1 MB splices rather than rebuilds.**
    ///
    /// §11 prices it at **34.34 µs against 3 216 to rebuild**. A timing is a report (R15), so what
    /// is gated is the relation the timing is about: the splice keeps the rows before the edit and
    /// the rebuild does not keep any — and the two answer the same index.
    #[test]
    fn a_keystroke_at_a_megabyte_splices_and_the_splice_is_the_rebuilds_answer() {
        let mut st = Text::of(crate::document::pasted(), WrapKind::Words);
        let w = crate::document::WIDE;
        let _ = st.index(w);
        let rows = st.indexed().expect("built").rows();
        assert!(rows > 1_000, "{rows} rows is not a megabyte's worth");

        st.click(w, rows / 2, 0, false);
        let started = Instant::now();
        st.insert(w, "z");
        let spliced = started.elapsed();

        let started = Instant::now();
        let rebuilt = Index::build_with(st.text(), w, st.revision(), WrapKind::Words);
        let rebuild = started.elapsed();

        assert!(
            st.indexed().expect("spliced").same_rows(&rebuilt),
            "the splice is not the rebuild's answer"
        );
        // Reported, not gated: the ratio is the machine's. What is asserted is that the splice did
        // not have to look at the head of the document at all.
        let _ = (spliced, rebuild);
        assert_eq!(
            st.recomputes(),
            2,
            "one build and one splice, not two builds"
        );
    }

    /// **The ring is not read during a draw**, which is the half of §11's *the frame does not pay
    /// for the ring* that lives on this side of the seam.
    ///
    /// The whole figure — *at cap 160.67 µs / 0 allocs, emptied 160.71 / 0 allocs, same document,
    /// same screen* — is a frame and is gated in [`crate::input`], over the component. What is
    /// asserted here is what makes it possible: two states holding the same text produce the same
    /// index and the same rows whatever their rings hold.
    ///
    /// The cap arm is **uncoalesced**, and that is stated rather than tuned: 256 entries of typed
    /// digits *is* one entry under the word-break rule, so a coalescing ring cannot be filled by
    /// typing at all — which is the ratio the rule exists for, arriving as a fixture problem.
    #[test]
    fn a_ring_at_cap_and_a_ring_emptied_index_the_same_document() {
        let mut full = Text::of(prose(200), WrapKind::Words)
            .with_ring(Ring::bounded(RING_ENTRIES, RING_BYTES).uncoalesced());
        full.end(300, false);
        for i in 0..RING_ENTRIES + 64 {
            full.insert(300, &(i % 10).to_string());
        }
        assert_eq!(full.ring().len(), RING_ENTRIES, "the ring is at its cap");
        assert!(full.ring().truncated());
        assert!(full.ring().bytes() <= RING_BYTES);

        let mut empty = Text::of(full.text().to_string(), WrapKind::Words);
        assert!(empty.ring().is_empty());
        assert_eq!(full.index(300).rows(), empty.index(300).rows());
        assert_eq!(full.index(300).starts(), empty.index(300).starts());
    }

    /// **`Up` and `Down` step a visual row and not a hard line.**
    ///
    /// The reason the index carries visual rows at all: in a textarea `Down` from the first half of
    /// a wrapped line lands in its second half. A caret that stepped by hard lines would jump a
    /// screenful on every wrapped paragraph.
    #[test]
    fn down_from_a_wrapped_line_lands_in_its_own_second_half() {
        let text = "the quick brown fox jumps over the lazy dog and keeps going\nnext\n";
        let mut st = Text::of(text.to_string(), WrapKind::Words);
        let w = 20u16;
        st.click(w, 0, 3, false);
        let first = st.caret();
        st.step_row(w, false, false);
        let second = st.caret();
        assert!(second.byte() > first.byte());
        assert!(
            second.byte() < text.find('\n').expect("a line break"),
            "`Down` left the hard line, so it stepped a line and not a row"
        );
    }

    /// **`Backspace` at the start and `Delete` at the end do nothing at all.**
    ///
    /// Not an arm and not a nicety: without the guard the edit removes nothing, **bumps the
    /// revision** — which invalidates the wrap memo — and records an entry that undoes nothing, so
    /// holding either key on a boundary fills the ring with edits and pushes the real history out
    /// of it. Both bounds are watched not moving.
    #[test]
    fn backspace_at_the_start_and_delete_at_the_end_change_nothing_at_all() {
        let mut st = Text::of("hello".into(), WrapKind::Ruler);
        let w = 20u16;
        st.home(w, false);
        let (revision, entries) = (st.revision(), st.ring().len());
        for _ in 0..8 {
            st.backspace(w);
        }
        assert_eq!(st.text(), "hello");
        assert_eq!((st.revision(), st.ring().len()), (revision, entries));

        st.end(w, false);
        for _ in 0..8 {
            st.delete(w);
        }
        assert_eq!(st.text(), "hello");
        assert_eq!((st.revision(), st.ring().len()), (revision, entries));

        // And they do work one cluster in, or the guard is a widget that has stopped editing.
        st.home(w, false);
        st.delete(w);
        assert_eq!(st.text(), "ello");
        st.end(w, false);
        st.backspace(w);
        assert_eq!(st.text(), "ell");
    }

    /// **The rows do not tile the buffer, and every byte of it is still somewhere.**
    ///
    /// `layout::text::wrap` trims each piece, so the whitespace a greedy break consumed and any
    /// trailing whitespace belong to no row's *content*. They are still places a caret goes — a
    /// single trailing space is enough to reach one — and the gate is that `End` reaches them and
    /// that `row_of` puts every byte of the buffer on a row that is drawn.
    #[test]
    fn end_reaches_a_trailing_space_and_every_byte_of_the_buffer_is_on_a_row() {
        for (content, w, kind) in [
            ("hi ", 20u16, WrapKind::Words),
            ("hi   ", 20, WrapKind::Words),
            ("hi there", 5, WrapKind::Words),
            ("hi   \nx", 20, WrapKind::Words),
            ("abcdefghij", 5, WrapKind::Ruler),
            (
                "the quick brown fox jumps over the lazy dog",
                11,
                WrapKind::Words,
            ),
        ] {
            let mut st = Text::of(content.into(), kind);
            let rows = st.index(w).rows();

            // **Every byte is on a row, and the row it is on is one that exists.** A byte the draw
            // could not place is a caret the terminal never receives.
            for byte in boundaries(content) {
                let r = st.index(w).row_of(byte);
                assert!(r < rows, "`{content}`: byte {byte} is on row {r} of {rows}");
            }

            // **`End` on the last row reaches the end of the buffer**, trailing space and all.
            let last = rows - 1;
            st.click(w, last, u16::MAX, false);
            st.end(w, false);
            assert_eq!(
                st.caret().byte(),
                content.len(),
                "`{content}` {kind:?} at {w}: `End` on the last row did not reach the end"
            );

            // **And `Ctrl+A` selects all of it**, which is the same claim through the gesture a
            // caller actually makes.
            st.select_all(w);
            assert_eq!(st.selected(), content, "`{content}` {kind:?} at {w}");

            // **`End` on an earlier row stays on that row.** The byte after a soft break is the
            // next row's own column 0, so a walk that reached it would leave the caret carrying one
            // row's column while the draw put it on another's.
            for r in 0..last {
                st.click(w, r, 0, false);
                st.end(w, false);
                let at = st.caret().byte();
                assert_eq!(
                    st.index(w).row_of(at),
                    r,
                    "`{content}` {kind:?} at {w}: `End` on row {r} left it"
                );
            }
        }
    }

    /// **The wheel is consumed, and it is clamped at both ends.**
    ///
    /// A field declares `Interest::SCROLL` and owns its offset, so a notch it did not consume is
    /// worse than one it never declared: it is the topmost region over its rectangle, so an
    /// enclosing `scroll_area` never sees the notch either. Components ticket 20's defect class.
    #[test]
    fn the_wheel_moves_the_window_and_stops_at_both_ends() {
        let mut st = Text::of(prose(600), WrapKind::Words);
        let (w, h) = (80u16, 20u16);
        let last = st.index(w).rows() - usize::from(h);
        st.wheel(w, h, 5);
        assert_eq!(st.offset(), 5);
        st.wheel(w, h, -50);
        assert_eq!(st.offset(), 0, "a notch up at the top stays at the top");
        st.wheel(w, h, i32::MAX);
        assert_eq!(st.offset(), last, "and the bottom is the last full window");
        st.wheel(w, h, 0);
        assert_eq!(st.offset(), last);
    }

    /// **A window past the last row is pulled back**, or the caret is painted on a filler row.
    ///
    /// `Index::row_start` answers 0 out of range, so a blank row below the content reports row 0's
    /// start — which is a caret on nothing and a selection compared against the wrong range.
    #[test]
    fn a_window_past_the_last_row_is_pulled_back_by_the_reveal() {
        let mut st = Text::of(prose(20), WrapKind::Words);
        let (w, h) = (80u16, 8u16);
        let rows = st.index(w).rows();
        st.scroll_to(rows + 500);
        st.reveal(w, h);
        assert!(
            st.offset() + usize::from(h) <= rows,
            "the window is at {} over {rows} rows",
            st.offset()
        );
    }

    /// **A splice restarted past the last row does not double the document.**
    ///
    /// `Index::row_start` answers 0 out of range, so a restart *at* `rows()` would keep every row
    /// as the head and rebuild the tail from byte 0. `Text::edit` never passes that value and
    /// `Index::spliced` is `pub`.
    #[test]
    fn a_splice_restarted_past_the_last_row_is_clamped() {
        let base = "aaa\nbbb\nccc".to_string();
        let index = Index::build(&base, 20, 0);
        assert_eq!(index.rows(), 3);
        for restart in [3usize, 4, usize::MAX] {
            let spliced = index.spliced(&base, restart, 1);
            assert_eq!(
                spliced.rows(),
                3,
                "a restart at {restart} produced {} rows",
                spliced.rows()
            );
            assert!(spliced.same_rows(&Index::build(&base, 20, 1)));
        }
    }

    /// **A `Ruler` index over a cluster wider than the band makes progress.**
    ///
    /// One cluster a row is the only answer that does, and it is what a one-column input showing a
    /// CJK glyph is. Written down because the loop that builds the ruler would otherwise spin.
    #[test]
    fn a_ruler_narrower_than_its_widest_cluster_makes_progress() {
        let text = "\u{4e00}\u{4e8c}\u{4e09}";
        let index = Index::build_with(text, 1, 0, WrapKind::Ruler);
        assert_eq!(index.rows(), 3);
        assert_eq!(index.row_text(text, 1), "\u{4e8c}");
    }
}
