//! Bytes on the wire: the mirror, the emit loop, the differential SGR, and the scroll region.
//!
//! # Scope
//!
//! Spec §8 in full and **nothing about it deferred**: the `shortest` cursor encoding, a differential
//! SGR with its three traps, the two extended channels, synchronised output, the mirror updated as
//! bytes go out, the equality filter with the gap merge that needs no threshold, and the scroll region
//! verified before a byte is emitted.
//!
//! # The scroll region, and the mirror it needs
//!
//! `DECSTBM` + `SU`/`SD` takes a steady frame of a scrolling list from **643 bytes to 20** — a band of
//! rows moved instead of rewritten. It is a pre-pass and it is **verified rather than guessed**,
//! because the filter behind it can only emit cells the packet carries and `SU` moves every column of
//! every row in the band. See [`Serializer::scroll_prepass`], which is also where §8's rejected
//! speculative version and its 27x candidate-verification regression are written down.
//!
//! It reaches four of §14's twelve — the two list arms, the virtualised tree and the table — and
//! **two of those four are not on §8's list of what it is for.** The other eight are screens with no
//! shift in them and pay the probe and nothing else.
//!
//! # The equality filter, always on, and a gap priced in bytes
//!
//! Damage says which cells a frame *wrote*; the mirror says which of them actually **changed**. The
//! comparison sits inside the run scan — the scan was already reading every cell — and costs 0.9 ns
//! a damaged cell to buy between 1% and 30x.
//!
//! **It runs always, and the intuition about when to switch it off is exactly inverted.** A
//! full-screen change damages 24 000 cells and the filter buys 1%; a cleared-row list scroll damages
//! the same 24 000 cells and buys 14x. An area heuristic would switch the filter off precisely where
//! it is worth most, so there is no heuristic and no switch.
//!
//! Skipping a cell leaves a **gap**, and the cursor has to step over it — which is a move, and a
//! move costs bytes too. **A fixed gap threshold is in the wrong unit and there is no right value
//! for one:** a cell is one byte of ASCII, three of braille and four of an emoji, so six cells is
//! six bytes on a chart of `*` and eighteen on a chart of braille. §8 swept it and the two scenes
//! want opposite thresholds — the chart gets monotonically worse from 0 to 24 while the dialogs get
//! better and then flat.
//!
//! So the gap is priced in the unit the wire is measured in: the clusters' UTF-8 lengths plus
//! [`SGR_FLOOR`] for each style change inside, against the digit-counted cost of the cheapest
//! encoding of the move it would avoid. No constant is tuned, because none is a threshold — see
//! [`Serializer::price_gap`], and [`Filter`] for the configurations that lost, which are how the
//! conclusion stays reproducible rather than quoted.
//!
//! **The same rule bridges two runs.** Damage produces genuinely separate runs on one row and the
//! columns between them are not in the packet — but they are in the mirror, so they can be repainted
//! out of it and priced the same way. That is the one place this file emits a cell the packet does
//! not carry, and it is fenced by two conditions rather than one: the row must be **known**, and
//! every handle the mirror's cell carries must be in *this* packet.
//!
//! # The serializer has no knobs
//!
//! The prototype was `Options`-shaped only so the variants could be measured. The engine ships
//! exactly one configuration and none of it is a choice an application makes — **no `Options` type
//! reaches the public surface**, and everything wire-specific lives below the packet. What the
//! [`Capabilities`] argument selects is not a preference: it is what the terminal on the other end
//! can parse.
//!
//! # An extended style's two channels, and the capability that gates one of them
//!
//! An extended word spends bits 51..0 on a handle, so [`channels_of`] resolves all four channels
//! through `packet.ext` instead of reading the word — reading that handle as a 26-bit foreground and
//! a 26-bit background is exactly the silent wrong answer `Style::with_fg_bg` was deleted for, one
//! layer down.
//!
//! The underline colour reaches the wire unconditionally, as SGR 58 or 59. **The hyperlink does
//! not**: OSC 8 is emitted only where [`Capabilities::hyperlinks`] says the terminal implements it,
//! and where it does not, a hyperlinked cell is painted in the right colours and loses its link.
//! That is degradation, which this engine has a model for; a handle emitted as a colour is not.
//!
//! Because `hyperlinks` is **inferred** rather than detected — OSC 8 has no query — it is the one
//! capability a caller can correct, through `Overrides::hyperlinks` (architecture ticket 22). That
//! is also what lets a headless round trip close on a hyperlinked cell at all.
//!
//! # The frame's framing, and the session's
//!
//! Every frame that has anything to say opens with SGR 0. That is tcell's "make no style
//! assumptions", four bytes, and it is what lets the emit loop start from a style it knows rather
//! than from one it inherited. Where the terminal has mode 2026 the frame is wrapped in it, and
//! `?2026h` + `0m` + `?2026l` is §8's **20 bytes of fixed framing** — which matters only because a
//! caret blink is a 29-byte frame.
//!
//! **A hyperlink is closed before the frame ends, and SGR 0 is not what closes it.** An open OSC 8
//! survives a reset, so a frame that left one open would hyperlink whatever the next frame wrote
//! next to it. The reset at the top is what makes a frame self-contained in *style*; the close at
//! the bottom is what makes it self-contained in *link*.
//!
//! Auto-wrap is off for the lifetime of the alt screen rather than per frame, so it is not here —
//! see [`crate::engine`]. It deletes two of cellbuf's bug-driven workarounds outright, and neither
//! is ported: with no wrap there is no pending-wrap state and no bottom-right corner that scrolls.

use std::ops::RangeInclusive;

use crate::caps::Capabilities;
use crate::cell::{Cell, GraphemeId};
use crate::damage::Run;
use crate::exts::LinkId;
use crate::packet::Packet;
use crate::quant::Quantiser;
use crate::quirks::Underlines;
use crate::style::{Color, Style, TAG_DEFAULT, TAG_INDEXED, TAG_RGB};

/// What the terminal is currently showing.
///
/// Distinct from a frame, which is what the application *wants* shown. The mirror is what lets the
/// serializer skip a cell the frame rewrote without changing, and what lets a scroll be proved
/// before it is emitted (ADR 0006). It holds no application state and no handle, which is why it
/// does not reopen "the render thread holds no screen-sized state".
///
/// Read by the equality filter, which is what it was built for, and written as bytes go out. It is
/// also asserted against the terminal model on every `present` of every test that uses
/// [`Harness`](crate::testing::Harness), which is the gate that says the two agree at all.
///
/// # Unknown, and why it is per cell rather than per row
///
/// A part of the mirror that cannot be trusted is **unknown**, and that is how a full repaint
/// expresses itself without a separate mode (ADR 0006, §8). Three things make it unknown, and only
/// the third is new:
///
/// - **at startup**, because the mirror records what the terminal shows and nobody recorded that;
/// - **after a resize**, for the same reason — a terminal reflows on `SIGWINCH`, it does not clear;
/// - **after a sweep renumbered a handle table**, which is [`Packet::repaint`].
///
/// A cell leaves the unknown state when this serializer emits it, and nothing else does it.
///
/// **ADR 0006 and §8 both say *row*, and impl 08 built the row: a flag per row, set when one frame
/// had written every column of it. Impl 14 measured what that costs and replaced it, which is the
/// decision this ticket was told to make** — impl 13 left the question here in as many words, *ticket
/// 14 owns the filter and is where a tighter answer, per-cell rather than per-row knowledge, would be
/// paid for or refused.
///
/// It is paid for, and the measurement is not close. A row becomes known only when **one** frame
/// writes all of it, and at 300 columns almost nothing ever does: a dialog is sixty columns wide, a
/// chart plots four hundred points across eighty rows, a list draws its rows and not the gutter beside
/// them. Driven over spec §14's twelve scenes, per-row knowledge left **every row of eleven of them
/// unknown for ever**, and the filter measured byte for byte identical to no filter at all. That is
/// not the *cost in bytes on the frames after a sweep* the row flag was priced as; it is the filter
/// not existing.
///
/// # What it costs, which is nothing, and why that is not too good to be true
///
/// There is no bitset and no branch. The unknown state is a **value**: [`Cell::UNKNOWN`], whose
/// grapheme is the `EMPTY` sentinel, and **no composited frame can hold one** — the frame is opaque so
/// its ground is a blank, and a non-opaque layer's `EMPTY` cells are skipped rather than copied
/// (spec §5). So `frame_cell == mirror_cell` is *already* false wherever the mirror does not know, for
/// the same reason `1 != NaN`, and the filter's comparison needs nothing added to it.
///
/// That is what makes it sound rather than convenient. The danger the unknown state exists for is not
/// the false *inequality* — which re-emits, and is merely bytes — but the false *equality*: a stale
/// mirror cell holding an old handle, a later frame whose new handle happens to equal it, compared
/// equal, skipped, and the terminal left showing the wrong text. A sentinel no frame cell can equal
/// makes that unreachable **by construction**, where a flag makes it unreachable only while everybody
/// remembers to consult the flag.
///
/// Two consequences, both wanted:
///
/// - [`forget`](Mirror::forget) is now a 384 KiB fill rather than an eighty-byte one. It runs on a
///   sweep that renumbered, which spec §3 already prices at *one full frame on the render thread*, and
///   this is a fraction of that.
/// - The mirror no longer reads as a screen of blanks before anything is drawn. That was never true of
///   the terminal and the flag existed to say so; now the cells say it themselves.
///
/// # It holds what the terminal was **sent**, and one cell is a proxy for that rather than a copy
///
/// Impl 17 narrows colour to what the terminal can express, and it does so **before** the comparison
/// below, precisely so that this stays true: a mirror holding colours the terminal never got would
/// make the filter under-filter by exactly the amount the depth collapses, and an animated gradient
/// on a 16-colour terminal would re-emit every frame for no visible change. See [`crate::quant`].
///
/// **For an extended cell it is a proxy, and the residual is bytes rather than correctness.** Bits
/// 51..0 of an extended word are a handle, so there is nothing in the word to narrow; what this
/// records is the handle, and the handle is what the next frame compares. Two *distinct* table
/// entries whose colours narrow to the same wire — two underline colours a shade apart on a
/// 16-colour terminal — therefore compare unequal and are re-emitted, exactly the shape the inline
/// case no longer has.
///
/// It is not made exact, and the reason is that the exact version already has a price attached. It
/// needs an identity derived from the *narrowed content* rather than from the table, which is spec
/// §6's **content-keyed packet** one layer along: built, measured at 1.8x on an adversarial page, and
/// refused on the app thread's behalf. Nothing here reopens that. What bounds the residual instead is
/// what bounds the table itself — an extended cell is under 1% of a screen (spec §3), and the channel
/// that would have to be *animated* to make this cost anything is an underline colour.
///
/// [`is_known`](Mirror::is_known) survives as a **row** query over the cells, and impl 15 corrected
/// which readers ask it. The amendment named the scroll region as the shipping one, and the scroll
/// region turned out not to *ask* it: both of its obligations need per-cell knowledge, for exactly
/// the reason the filter does — a row is the wrong granularity for a screen no single frame writes
/// whole — and asking it of an exposed row is *stricter than the obligation*, which forfeits the
/// scroll rather than proving it. What does ask it is the invariant on the far side: after a verified
/// scroll the mirror knows **every row of the band**, because obligation 1 held over every column of
/// every row moved and the terminal erased every column of every row exposed. That is a row question,
/// it is not trivially true, and it is what keeps the next frame's filter sound. The other reader is
/// `Screen::known_rows`, which is the gate on `Packet::repaint`.
#[derive(Clone, Debug)]
pub(crate) struct Mirror {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
}

impl Mirror {
    pub(crate) fn new(width: u16, height: u16) -> Mirror {
        Mirror {
            width,
            height,
            // Unknown, not blank, and the cells say so themselves rather than a flag saying it for
            // them. A terminal at startup is showing whatever it was showing.
            cells: vec![Cell::UNKNOWN; width as usize * height as usize],
        }
    }

    /// What the terminal is showing at `(x, y)`, or [`Cell::UNKNOWN`] where this mirror does not know.
    ///
    /// **The read the equality filter is built on.** No flag has to be consulted first, which is the
    /// whole of the previous section: an unknown cell is a value no frame cell equals.
    pub(crate) fn cell(&self, x: u16, y: u16) -> Cell {
        self.cells[y as usize * self.width as usize + x as usize]
    }

    /// What the terminal is showing at `(x, y)`, or `None` where this mirror does not know.
    ///
    /// The same read, in the shape the **gap merge** needs: repainting a column out of the mirror is
    /// only legitimate where the mirror is a fact, so the one caller that emits a cell the packet
    /// never carried asks in a form it cannot forget to check.
    pub(crate) fn known_cell(&self, x: u16, y: u16) -> Option<Cell> {
        let cell = self.cell(x, y);
        (cell != Cell::UNKNOWN).then_some(cell)
    }

    /// Whether **every cell** of row `y` records what the terminal is showing.
    ///
    /// A row query over per-cell state, and a walk rather than a flag because the two readers are
    /// rare: `Screen::known_rows` is the gate on [`Packet::repaint`], and the scroll region asks it
    /// of the rows a scroll would **expose** — obligation 2 of
    /// [`Serializer::scroll_prepass`], where nothing can be said about a row the mirror does not
    /// know. The filter itself asks nothing per row — see the type's own documentation.
    ///
    /// **Obligation 1 does not ask it, and that is deliberate rather than an omission.** The rows a
    /// scroll *moves* are the whole band, so a row query over them would be a second pass over a
    /// screen for a fact the comparison already carries: what the frame wants is read through
    /// [`known_cell`](Mirror::known_cell), and a cell the frame wants can never equal a
    /// [`Cell::UNKNOWN`] source. See [`Serializer::row_lands_on`].
    pub(crate) fn is_known(&self, y: u16) -> bool {
        let row = y as usize * self.width as usize;
        self.cells[row..row + self.width as usize]
            .iter()
            .all(|c| *c != Cell::UNKNOWN)
    }

    /// The columns `lo..hi` of row `y`, as a slice.
    ///
    /// **The shape the scroll region's two obligations are asked in**, and the reason they are asked
    /// in it is cost: both walk a band against a band, so a per-column accessor puts an index
    /// calculation and a bounds check on every cell of a screen. A slice compares in one call and
    /// vectorises; the same walk through [`cell`](Mirror::cell) measured 2.4x the frame it was
    /// guarding.
    fn span(&self, y: u16, lo: u16, hi: u16) -> &[Cell] {
        let row = y as usize * self.width as usize;
        &self.cells[row + lo as usize..row + hi as usize]
    }

    /// Forget the whole screen: what [`Packet::repaint`] asks for.
    fn forget(&mut self) {
        self.cells.fill(Cell::UNKNOWN);
    }

    /// Move the rows of the band `top..=bot` the way `SU`/`SD` is about to move them on the terminal,
    /// and record the rows it exposes as **blank**.
    ///
    /// Blank rather than unknown, and that is exact rather than optimistic (spec §8, ADR 0006): a
    /// terminal erases what it exposes with the *current* background, and the scroll is emitted after
    /// the frame's own `SGR 0`, so background-colour-erase and erase-to-default agree. **The
    /// correctness of this line is an ordering, not a capability** — and the terminal model erases
    /// with the style it is actually holding, so a serializer that ever emitted the scroll under a
    /// live SGR would fail the round trip rather than quietly disagree by a background colour.
    ///
    /// An exposed row is recorded **whole**, which is the one place the mirror gains knowledge of a
    /// cell no byte named. That is sound for the same reason: the terminal wrote every column of it,
    /// because `SU` has no horizontal margins.
    /// A scroll is at least one row and always fewer than its band, which is
    /// [`Serializer::probe`]'s own range and is what makes the arithmetic below total. There is no
    /// clamp: [`TermModel::scroll`](crate::term_model) has one because its count arrives off the wire,
    /// and copying that here would be a branch the caller cannot reach, documented as one it can.
    fn scroll(&mut self, s: Scroll) {
        let (top, bot) = s.band;
        debug_assert!(
            s.by >= 1 && s.by <= bot - top,
            "a scroll of its own whole band"
        );
        let w = self.width as usize;
        let row = |y: u16| y as usize * w;
        // **Downwards runs backwards**, because a row is copied over the one below it and a forward
        // walk would copy the row it has just written. Upwards is the mirror image and runs forwards.
        // One `Scroll::moved` for both, reversed rather than a second range, so the range the filter
        // is later told to skip cannot be a different one.
        if s.up {
            for y in s.moved() {
                let src = row(s.source(y));
                self.cells.copy_within(src..src + w, row(y));
            }
        } else {
            for y in s.moved().rev() {
                let src = row(s.source(y));
                self.cells.copy_within(src..src + w, row(y));
            }
        }
        for y in s.exposed() {
            self.cells[row(y)..row(y) + w].fill(Cell::BLANK);
        }
    }

    fn set(&mut self, x: u16, y: u16, c: Cell) {
        self.cells[y as usize * self.width as usize + x as usize] = c;
    }
}

/// A candidate scroll: the band, the direction and the distance.
///
/// One value rather than four arguments that travelled together through the probe, the verification,
/// the emission and the mirror — and, while they did, **the range each of them is about was computed
/// twice**: once where obligation 2 was checked and once where the mirror recorded it. Two
/// computations of one range is precisely the shape a scroll optimisation gets wrong, so there is one
/// of each here and every caller asks.
#[derive(Clone, Copy, Debug)]
struct Scroll {
    /// The first and last rows of the band, inclusive.
    band: (u16, u16),
    /// `SU`: content moves toward the top of the band and the bottom of it is exposed. `SD` is the
    /// mirror image.
    up: bool,
    /// Rows. At least one, and always fewer than the band's height — see [`Serializer::probe`].
    by: u16,
}

impl Scroll {
    /// The rows the scroll **moves**: what obligation 1 is about, and what the equality filter may
    /// then skip, because obligation 1 having held over every column of them *is* the filter's answer.
    fn moved(self) -> RangeInclusive<u16> {
        let (top, bot) = self.band;
        if self.up {
            top..=bot - self.by
        } else {
            top + self.by..=bot
        }
    }

    /// The rows the scroll **exposes** for the terminal to erase: what obligation 2 is about, and what
    /// the mirror records blank.
    fn exposed(self) -> RangeInclusive<u16> {
        let (top, bot) = self.band;
        if self.up {
            bot + 1 - self.by..=bot
        } else {
            top..=top + self.by - 1
        }
    }

    /// The row `y` is about to hold what this row holds now.
    fn source(self, y: u16) -> u16 {
        if self.up { y + self.by } else { y - self.by }
    }
}

/// Turns a packet into the bytes that go to the terminal.
pub(crate) struct Serializer {
    out: Vec<u8>,
    mirror: Mirror,
    /// Where the terminal's cursor is, or `None` before the first move of a frame. With auto-wrap
    /// off the cursor stops in the last column rather than advancing off it, and that is what is
    /// recorded — see [`Serializer::advance`].
    cursor: Option<(u16, u16)>,
    style: Style,
    /// The hyperlink the terminal currently has open, or [`LinkId::NONE`].
    ///
    /// Tracked apart from [`style`](Serializer::style) because OSC 8 is not an SGR and **SGR 0 does
    /// not close it**: a frame that left one open would hyperlink whatever the next frame wrote.
    link: LinkId,
    /// The cluster emitted last, when the next cell's bytes would follow it with nothing between.
    /// See [`Serializer::joins_left`].
    prev: Option<GraphemeId>,
    /// Whether a non-ASCII cluster has gone out on the row the cursor is on.
    ///
    /// Spec §10's rule, as one boolean carried through a scan that is already running: **after a run
    /// containing any non-ASCII cluster, the next move on that row is `CHA` rather than `CUF`.** A
    /// width disagreement is *permanent* once a mirror exists — the mirror records what was
    /// intended, so an overpainted neighbour is never re-emitted — and `CUF` is relative and
    /// compounds the error along the row. An absolute move bounds the corruption to the one cell the
    /// terminal measured differently.
    ///
    /// Cleared whenever the cursor's row changes, because the fact is about a row.
    non_ascii_on_row: bool,
    /// Whether this frame's opening bytes have gone out.
    ///
    /// §8's *every frame that has anything to say opens with SGR 0*, read strictly: the framing is
    /// written by [`open_frame`](Serializer::open_frame) on the first cell that actually reaches the
    /// wire, so **a frame the filter emptied says nothing at all** rather than spending twenty bytes
    /// announcing it. It is not only a byte count: the reset is what lets the emit loop start from a
    /// style it knows rather than one it inherited, so a serializer that wrote it eagerly and then
    /// found nothing to emit would have to choose between sending it for nothing and claiming a reset
    /// the terminal never saw. Deferring it makes both wrong answers unreachable.
    frame_open: bool,
    /// What this terminal can express, rebuilt from [`Capabilities`] at the top of every
    /// [`serialize`](Serializer::serialize).
    ///
    /// Rebuilt rather than invalidated because it is forty-odd bytes of fixed-size value and
    /// capabilities are immutable for the life of the `Screen` — so there is no staleness to
    /// track, and a `Serializer` a test hands two different terminals still narrows for the one it
    /// was given.
    quant: Quantiser,
    /// The **one-entry memo** on the last style word narrowed, which is the whole of what keeps
    /// quantisation off the per-cell path.
    ///
    /// Cells in a run are contiguous and share a `u64`, so a memo one entry deep turns a per-cell
    /// narrowing into a per-distinct-style one — the same trick spec §3 used three times for the
    /// side-table round trip, and the reason this ticket's owed measurement is *per style word*
    /// rather than per cell. Cleared at the top of every frame, because that is where
    /// [`quant`](Serializer::quant) is rebuilt and a memo outliving its quantiser would answer for
    /// the wrong terminal.
    memo: Option<(Style, Style)>,
    /// How many style words this serializer has actually narrowed, as opposed to answered from the
    /// memo.
    ///
    /// **A count, because a memo that thrashes is invisible to every correctness test.** It is the
    /// property `crate::gates::a_frame_narrows_once_a_distinct_style_word_and_never_twice_a_cell`
    /// is about, and it exists because that gate's first version did not: the memo answered one
    /// spelling of a word and stored the other, so a screen of one colour paid two full narrowings
    /// a cell and every test stayed green.
    #[cfg(test)]
    narrowings: usize,
    /// Which configuration of the filter this serializer is running. **One value ships**; the rest
    /// exist so §8's *there is no threshold* is reproducible. See [`Filter`].
    #[cfg(test)]
    filter: Filter,
    /// Whether the scroll region pre-pass runs at all. **True in a shipping build**, and switchable
    /// from a test for the same reason [`Filter`] has three variants that lost: §8's byte table has a
    /// *filtered* column and a *+ scroll region* column, and a table with one arm cannot reproduce a
    /// claim about two.
    #[cfg(test)]
    scroll_region: bool,
    /// Whether to keep verifying candidates after one has been refused: **§8's 27x regression**, and
    /// the reason it is a field rather than a paragraph. See
    /// [`verifies_every_match`](Serializer::verifies_every_match).
    #[cfg(test)]
    verify_every_match: bool,
    /// How many frames took the scroll path, and how many candidates were verified to get there.
    ///
    /// **The second number is the gate.** §8's 27x regression was verifying every candidate that
    /// matched the probe, and the shape of the fix is *one*, so the property is a count and not a
    /// stopwatch — see `crate::gates::a_repeating_rows_screen_verifies_one_candidate_a_frame`.
    #[cfg(test)]
    scrolls: usize,
    #[cfg(test)]
    verifies: usize,
    /// What §10's rule has cost, in bytes, since this serializer was built.
    ///
    /// **A report, and spec §15's second owed measurement.** The rule refuses `CUF` on a row that has
    /// emitted a non-ASCII cluster, and what that costs is exactly the difference between the
    /// encoding actually chosen and the `CUF` that was refused — summed here rather than obtained by
    /// running a second serializer, because a second serializer would be a second configuration and
    /// this crate ships one.
    #[cfg(test)]
    cha_rule_bytes: usize,
}

impl Serializer {
    pub(crate) fn new(width: u16, height: u16) -> Serializer {
        Serializer {
            // A realistic full-screen frame is 24 430 bytes (spec §8). Reserving it up front is
            // what keeps the steady state free of a growth reallocation.
            out: Vec::with_capacity(32 * 1024),
            mirror: Mirror::new(width, height),
            cursor: None,
            style: Style::DEFAULT,
            link: LinkId::NONE,
            prev: None,
            non_ascii_on_row: false,
            frame_open: false,
            quant: Quantiser::transparent(),
            memo: None,
            #[cfg(test)]
            narrowings: 0,
            #[cfg(test)]
            filter: Filter::default(),
            #[cfg(test)]
            scroll_region: true,
            #[cfg(test)]
            verify_every_match: false,
            #[cfg(test)]
            scrolls: 0,
            #[cfg(test)]
            verifies: 0,
            #[cfg(test)]
            cha_rule_bytes: 0,
        }
    }

    /// Serialise under one of the configurations that lost, which is how §8's sweep is reproduced
    /// rather than quoted. See [`Filter`].
    #[cfg(test)]
    pub(crate) fn set_filter(&mut self, filter: Filter) {
        self.filter = filter;
    }

    /// Serialise with the scroll pre-pass off: §8's *filtered* column, which is the arm its
    /// *+ scroll region* column is a ratio against.
    #[cfg(test)]
    pub(crate) fn set_scroll_region(&mut self, on: bool) {
        self.scroll_region = on;
    }

    /// Serialise the way §8 rejected: verify every candidate the probe matches, not the first.
    #[cfg(test)]
    pub(crate) fn set_verify_every_match(&mut self, on: bool) {
        self.verify_every_match = on;
    }

    /// How many frames this serializer has put a scroll on the wire for.
    #[cfg(test)]
    pub(crate) fn scrolls(&self) -> usize {
        self.scrolls
    }

    /// How many candidates this serializer has verified. **One a frame at most, by construction.**
    #[cfg(test)]
    pub(crate) fn verifies(&self) -> usize {
        self.verifies
    }

    /// Whether the mirror is compared at all. **Always, in a shipping build** — §8's *run it
    /// always*, and the reason is that damage area does not predict whether it pays.
    #[cfg(not(test))]
    fn compares(&self) -> bool {
        true
    }

    /// See the shipping arm above; [`Filter::Off`] is §8's `span` column and reachable from tests.
    #[cfg(test)]
    fn compares(&self) -> bool {
        self.filter != Filter::Off
    }

    /// Whether the scroll pre-pass runs. **Always, in a shipping build** — it is not a capability
    /// question: `DECSTBM` and `SU` are VT100 and VT420, everything in tier 1 has them, and what
    /// makes the optimisation safe is the verification rather than an answer from the other end.
    #[cfg(not(test))]
    fn scrolls_at_all(&self) -> bool {
        true
    }

    /// See the shipping arm above. Off is §8's *filtered* column.
    #[cfg(test)]
    fn scrolls_at_all(&self) -> bool {
        self.scroll_region
    }

    /// Whether a candidate the probe matched but the obligations refused is followed by the next one.
    ///
    /// **Never, in a shipping build.** This is §8's 27x regression, and it is here rather than
    /// described because a cliff quoted is a cliff nobody can re-measure: `Filter` carries the three
    /// gap rules that lost for the same reason, and the argument is the same one — a variant no
    /// shipping build can construct is an instrument, and a variant a caller can select is a knob.
    #[cfg(not(test))]
    fn verifies_every_match(&self) -> bool {
        false
    }

    /// See the shipping arm above, which is what ships. `true` is the version §8 rejected.
    #[cfg(test)]
    fn verifies_every_match(&self) -> bool {
        self.verify_every_match
    }

    /// How many bytes a gap may cost before [`price_gap`](Serializer::price_gap) stops walking it:
    /// the move it would avoid, because a gap that already costs more than the move has lost.
    #[cfg(not(test))]
    fn gap_budget(&self, move_cost: usize) -> usize {
        move_cost
    }

    /// See the shipping arm above. A cell-counted threshold does not care what a gap costs, so its
    /// walk has to reach the end of the gap — where the only thing left to refuse is a handle this
    /// packet cannot resolve.
    #[cfg(test)]
    fn gap_budget(&self, move_cost: usize) -> usize {
        match self.filter {
            Filter::Cells(_) => usize::MAX,
            _ => move_cost,
        }
    }

    /// Whether to paint through a gap priced at `price` bytes, against the `move_cost` bytes of the
    /// move it would avoid. `None` is a gap that may not be painted through at any price.
    ///
    /// **A tie goes to the move**, for `shortest`'s own reason one level down: where the price is
    /// equal, take the answer that gives the terminal fewer cells to render.
    #[cfg(not(test))]
    fn merges(&self, price: Option<usize>, move_cost: usize, _cols: u16) -> bool {
        price.is_some_and(|p| p < move_cost)
    }

    /// See the shipping arm above, which is [`Filter::Bytes`].
    #[cfg(test)]
    fn merges(&self, price: Option<usize>, move_cost: usize, cols: u16) -> bool {
        match self.filter {
            Filter::Bytes => price.is_some_and(|p| p < move_cost),
            // `Cells(0)` is `Strict`, which is what makes the sweep's first point the same
            // configuration as its own control.
            Filter::Cells(n) => price.is_some() && cols <= n,
            Filter::Strict | Filter::Off => false,
        }
    }

    /// What §10's `CHA`-after-non-ASCII rule has cost in bytes. See
    /// [`cha_rule_bytes`](Serializer::cha_rule_bytes).
    #[cfg(test)]
    pub(crate) fn cha_rule_bytes(&self) -> usize {
        self.cha_rule_bytes
    }

    #[cfg(test)]
    pub(crate) fn mirror(&self) -> &Mirror {
        &self.mirror
    }

    /// How many style words this serializer has narrowed rather than answered from the memo. See
    /// [`narrowings`](Serializer::narrowings).
    #[cfg(test)]
    pub(crate) fn narrowings(&self) -> usize {
        self.narrowings
    }

    /// Serialise a packet. The returned slice is valid until the next call.
    ///
    /// `caps` is what the terminal can parse, not what anybody prefers: the two SGR forms, ConPTY's
    /// underline-colour spelling, whether OSC 8 is worth emitting and whether the frame is wrapped
    /// in mode 2026 are all read from it and from nowhere else.
    pub(crate) fn serialize(&mut self, packet: &Packet, caps: &Capabilities) -> &[u8] {
        self.out.clear();
        // Before the early return, not after it: an empty packet is a frame that opened nothing, and
        // a flag left standing from the frame before would make the next one inherit a style it never
        // reset. See [`open_frame`](Serializer::open_frame).
        self.frame_open = false;
        // What this terminal can express, and the memo that keeps narrowing off the per-cell path.
        // Both here rather than in `new`, because `new` has no terminal — and together, so a memo
        // can never outlive the quantiser that filled it.
        self.quant = Quantiser::for_terminal(caps);
        self.memo = None;
        // **Before the frame, and outside its synchronised block.** A tracking level is a terminal
        // mode rather than a picture, so it has nothing to be atomic with — and putting it first is
        // what makes it arrive even on the frame that damaged nothing.
        let actuation = packet.actuation();
        if let Some((from, to)) = actuation.mouse {
            crate::actuate::write_mouse(&mut self.out, from, to);
        }

        // A packet carries its own size so that a stale one can be refused on its own (spec §2).
        // Nothing can produce a stale one until ticket 22 brings the resize that makes sizes move,
        // so this is the debug-only assertion that shape of invariant gets rather than a runtime
        // check on the frame path.
        debug_assert_eq!(
            packet.size(),
            (self.mirror.width, self.mirror.height),
            "a packet packed at one size is being written into a mirror of another"
        );

        // Before anything is emitted, **and before the cell-less early return below**: a sweep
        // renumbered a table, so every handle this mirror recorded names an entry that has moved.
        // Nothing about the *screen* changed — the cells still say the same thing — which is exactly
        // why there is no damage to go with it and a flag is what carries it (spec §3).
        //
        // The placement is load-bearing since impl 21, and it was wrong for one commit. `repaint` is
        // taken out of the `Screen`'s latch at pack time, so a packet that carries it has spent it;
        // a frame with no cells is now reachable — a caret that moved with nothing else changing — and
        // it can be the frame the flag lands on. Reading the flag after the early return would have
        // dropped it silently, and the mirror would go on trusting handles a sweep had moved.
        if packet.repaint() {
            self.mirror.forget();
        }

        if packet.is_empty() {
            // No cells, and something for the terminal anyway: a caret that moved, or a tracking
            // level that changed. Nothing has opened a frame, so the caret's move is priced against
            // a cursor position this serializer does not claim to know.
            self.emit_caret(&actuation);
            return &self.out;
        }

        // The scroll region, before a cell is considered: it moves the mirror, so the filter below
        // is comparing against where the terminal will be rather than against where it was. Nothing
        // is emitted unless both obligations hold — see [`scroll_prepass`](Serializer::scroll_prepass).
        //
        // What it hands back is the rows it **settled**: obligation 1 proved every column of them
        // equal to the mirror it has just shifted, so the filter over them is a walk that provably
        // emits nothing. Skipping it is not an optimisation bolted on afterwards — it is what makes
        // the pre-pass close to free, because obligation 1 and the filter are the same comparison and
        // this is the frame paying for it once.
        let settled = self.scroll_prepass(packet, caps);

        // **Rows, not runs.** A gap between two runs on one row can be bridged out of the mirror
        // and a gap between two rows cannot, so the row is the unit the filter plans in.
        for row in PacketRows::new(packet) {
            if settled.as_ref().is_some_and(|s| s.contains(&row.y())) {
                continue;
            }
            self.emit_row(&row, packet, caps);
        }

        if self.frame_open {
            // An open hyperlink outlives the frame that opened it, and SGR 0 is not what closes one.
            if !self.link.is_none() {
                emit_osc8(&mut self.out, None);
                self.link = LinkId::NONE;
            }
            // **After the frame's last write and inside its block**, which is the only moment at
            // which the caret's position is correct and a moment only the engine has (ADR 0005). The
            // frame has just moved the terminal's cursor to wherever its last cell was, so a visible
            // caret is put back here whether or not the application moved it — and in the case that
            // matters it costs nothing, because that is exactly where the caret already is.
            self.emit_caret(&actuation);
            if caps.sync_output() {
                self.out.extend_from_slice(SYNC_END);
            }
        } else {
            // Every cell was filtered out, so the frame never opened. The caret still has to be
            // answered, and there is no block to put it in.
            self.emit_caret(&actuation);
        }
        &self.out
    }

    /// The caret: shape, then position, then visibility.
    ///
    /// The order is the one that cannot be seen going wrong. A shape set after the caret is shown
    /// changes it under the eye; a caret shown before it is placed appears for one refresh where the
    /// last write ended. **Nothing here blinks anything** — the terminal's own caret does that, in
    /// the terminal's process, at the user's rate (ADR 0005).
    ///
    /// A move is emitted when the caret went somewhere, when it has just become visible, or when
    /// this frame wrote a cell — and only then, which is what keeps a steady caret free.
    fn emit_caret(&mut self, actuation: &crate::actuate::Actuation) {
        // A frame that never opened never reset the cursor, and what is left in the field is the
        // *previous* frame's last position. It is very probably still true — this thread owns the
        // write direction and nothing else has written since — but `open_frame` establishes *a
        // frame's first move is absolute* and a second rule for one case is a rule that will be
        // wrong once. Five bytes on a caret that moved with nothing else changing.
        //
        // §10's row flag needs nothing here: `move_to` clears it whenever the row it is moving from
        // is unknown, which an unknown cursor makes true.
        if !self.frame_open {
            self.cursor = None;
        }
        if let Some(shape) = actuation.shape {
            crate::actuate::write_shape(&mut self.out, shape);
        }
        if let Some(caret) = actuation.caret
            && (actuation.moved || actuation.show == Some(true) || self.frame_open)
        {
            self.move_to(caret.x, caret.y, false);
        }
        if let Some(show) = actuation.show {
            crate::actuate::write_visibility(&mut self.out, show);
        }
    }

    /// The scroll region: `DECSTBM` + `SU`/`SD`, **verified before a byte is emitted**.
    ///
    /// A cleared-row list scroll is what this is for — `less`, `tail -f`, a file manager, every log
    /// pane — and it is the difference between moving eighty rows and rewriting them.
    ///
    /// # The reasoning that was wrong, because it is the attractive one
    ///
    /// The first version §8 wrote was **speculative**: guess cheaply, because the filter behind the
    /// guess compares against the mirror and repairs whatever the guess got wrong. **That is false,
    /// and the round trip caught it within a minute.** The filter can only emit cells the packet
    /// carries, and the packet carries damaged cells only. `SU` moves every column of every row in
    /// the region — including the columns this frame never touched, which are exactly the ones no
    /// later pass can put back. A screen with two text verbs and a one-column gap between them loses
    /// that column.
    ///
    /// So the guess is verified, against two obligations:
    ///
    /// 1. Every row the scroll **moves** must already hold, in the mirror, what this frame wants on
    ///    the row it lands on — undamaged columns included. [`row_lands_on`](Serializer::row_lands_on).
    /// 2. Every row the scroll **exposes** must be blank in every column this frame does not
    ///    repaint. [`exposed_row_is_clear`](Serializer::exposed_row_is_clear).
    ///
    /// **Obligation 2 is checked first**, because it is a handful of rows against the whole band and
    /// it is the one that actually fails.
    ///
    /// # Exactly one candidate is verified
    ///
    /// A screen whose rows repeat — an alternating pattern, a ruled table — matches a probe many
    /// times over, and §8 measured verifying each of them turning a 38 µs frame into **1.03 ms**: a
    /// 27x regression that only appeared because numbers were kept per scene. Taking the first match
    /// forfeits a scroll that could in principle have been found, and that is not worth 27x.
    ///
    /// The rule is structural here rather than remembered: [`probe`](Serializer::probe) returns an
    /// `Option`, so there is one candidate to verify or none. The probe *is* obligation 1 asked of
    /// one row — the band's own edge — which is what makes the candidate cheap to reject and means
    /// the verification never re-derives what the probe established.
    ///
    /// # `SU` has no horizontal margins
    ///
    /// So a pane that is not the full width can only use this when the columns it does not own hold
    /// the same thing after the shift as before it — which blank-in-both-frames is the ordinary case
    /// of. That falls out of obligation 1 rather than being tested for: the obligation is over every
    /// column of the row, not over the pane's.
    ///
    /// **`DECSLRM` (mode 69) would lift the restriction and is deliberately neither queried nor
    /// used.** It is not in tier-1's confirmed set, and the verification above turns an unsupported
    /// margin into *silent corruption* rather than into a wasted escape — the terminal would apply
    /// `SU` to the whole width while this pre-pass had proved something about a band of it. Spec §15
    /// is where that question lives, and nothing in this file depends on the answer.
    fn scroll_prepass(
        &mut self,
        packet: &Packet,
        caps: &Capabilities,
    ) -> Option<RangeInclusive<u16>> {
        if !self.scrolls_at_all() {
            return None;
        }
        let rows = PacketRows::new(packet);
        let (top, bot) = self.changed_band(rows)?;
        // **One candidate, expressed as a loop that runs once.** `from` is where the probe resumes,
        // and in a shipping build nothing resumes it: `verifies_every_match` is a constant `false`, so
        // this compiles to a probe, a verification and a return. The loop exists because §8's
        // rejected version is reachable from a test, which is how its 27x is *reproduced* rather than
        // quoted — see [`Filter`] for the same argument about the gap merge.
        let mut from = 0;
        loop {
            let (k, scroll) = self.probe(rows, top, bot, from)?;
            #[cfg(test)]
            {
                self.verifies += 1;
            }
            if self.both_obligations_hold(rows, scroll) {
                self.emit_scroll(scroll, caps);
                return Some(scroll.moved());
            }
            if !self.verifies_every_match() {
                return None;
            }
            from = k + 1;
        }
    }

    /// The band a scroll could be in: the first and last rows on which this frame wants something
    /// other than what the mirror is showing.
    ///
    /// `None` when fewer than two rows changed, because a band of one row has nothing to move.
    ///
    /// **Only a damaged row can differ**, which is what keeps this proportional to damage: an
    /// undamaged cell is one an earlier frame left in the mirror, so what the frame wants there *is*
    /// what the mirror says. The walk stops at the first differing cell of each row, so a scrolling
    /// list costs a handful of comparisons a row; the worst case is a frame that damages the screen
    /// and changes one row of it, which costs the pass the filter is about to make anyway.
    ///
    /// **The band is one band and it is not searched for.** A frame that scrolls a log pane *and*
    /// changes an unrelated cell below it has a band spanning both, obligation 1 fails on the row
    /// between them, and the scroll is forfeited where a band of the pane alone would have worked.
    /// Narrowing it by search is verifying more than one candidate under another name, which is the
    /// 27x this file is built around, so it is not done.
    fn changed_band(&self, rows: PacketRows<'_>) -> Option<(u16, u16)> {
        let mut top = None;
        let mut bot = 0;
        for row in rows {
            if self.row_changed(&row) {
                top.get_or_insert(row.y());
                bot = row.y();
            }
        }
        let top = top?;
        (bot > top).then_some((top, bot))
    }

    /// Whether any cell this row damaged is not already on the terminal.
    fn row_changed(&self, row: &Row<'_>) -> bool {
        let y = row.y();
        let mut base = 0usize;
        for r in row.runs {
            let cells = &row.cells[base..base + r.len()];
            if !self.damaged_span_is_on_the_terminal(cells, y, r.lo) {
                return true;
            }
            base += r.len();
        }
        false
    }

    /// Whether a span of the packet's **own** cells is what the mirror is already showing at
    /// `(from, y)`.
    ///
    /// **The one place the pre-pass reads a packet cell against a narrowed mirror, and it is a
    /// function because it was two loops that disagreed.** The mirror holds what the terminal was
    /// *sent*, which since impl 17 is narrowed; a packet holds what the application asked for. So the
    /// two are comparable only through [`Quantiser`], and a comparison that forgot it was not merely
    /// approximate — it made every colour below truecolor look like a change, which forfeited **every
    /// scroll on every terminal that narrows anything**. Measured: three scrolls over three frames of
    /// a scrolling list at truecolor, and zero at `Indexed256` and `Ansi16`.
    ///
    /// The slice compare survives where nothing narrows, which is the case
    /// [`Mirror::span`]'s 2.4x was measured on, and the walk is what the other depths pay. It is
    /// bounded by a band and stops at the first disagreement, and it cannot use the emit loop's memo
    /// because this side of the pre-pass is `&self` — which is the right trade for a pass that exists
    /// to reject cheaply.
    fn damaged_span_is_on_the_terminal(&self, cells: &[Cell], y: u16, from: u16) -> bool {
        let mirrored = self.mirror.span(y, from, from + cells.len() as u16);
        if !self.quant.narrows() {
            return cells == mirrored;
        }
        cells
            .iter()
            .zip(mirrored)
            .all(|(want, shown)| self.quant.cell(*want) == *shown)
    }

    /// The next candidate at or after `from`, and its own index so a caller can ask for the one after
    /// it.
    ///
    /// Candidates are numbered rather than nested, so that *resume from here* is one integer: index
    /// `k` is a distance of `k / 2 + 1` rows, upwards on the even indices and downwards on the odd.
    /// A distance is at least one row and at most one less than the band's height, which is what makes
    /// [`Scroll::moved`] and [`Scroll::exposed`] total.
    ///
    /// Both directions at every distance, smallest first, because the common scroll is one row and
    /// which way it went is not knowable more cheaply than asking. The edge differs per direction and
    /// that is the whole of the asymmetry: `SU` leaves the band's **top** row holding content that was
    /// below it, `SD` leaves the **bottom** row holding content that was above it.
    ///
    /// A candidate is obligation 1 asked of that one row, so a match is a fact rather than a guess and
    /// rejecting one is cheap. Its cost is bounded by the band against the screen's width — the same
    /// order as the verification it guards — because each candidate stops at the first column that
    /// disagrees.
    fn probe(&self, rows: PacketRows<'_>, top: u16, bot: u16, from: u16) -> Option<(u16, Scroll)> {
        // Hoisted, because [`PacketRows::row`] is a walk from the front of the packet: asked inside
        // the loop this was the band squared, and on a full-screen frame that is the whole of what a
        // probe finding nothing costs.
        let (top_row, bot_row) = (rows.row(top), rows.row(bot));
        for k in from..2 * (bot - top) {
            let scroll = Scroll {
                band: (top, bot),
                up: k % 2 == 0,
                by: k / 2 + 1,
            };
            let edge = if scroll.up { top } else { bot };
            let row = if scroll.up { &top_row } else { &bot_row };
            if self.row_lands_on(row.as_ref(), edge, scroll.source(edge)) {
                return Some((k, scroll));
            }
        }
        None
    }

    /// Both obligations, in the order §8 puts them in.
    ///
    /// Obligation 1 walks the moved rows against a **forward cursor** over the packet rather than
    /// looking each row up, for the reason [`probe`](Serializer::probe) hoists its two: a lookup is a
    /// walk from the front, and one per row of a band is the band squared.
    fn both_obligations_hold(&self, rows: PacketRows<'_>, scroll: Scroll) -> bool {
        // Obligation 2 first: a handful of rows, and the one that actually fails.
        for y in scroll.exposed() {
            if !self.exposed_row_is_clear(rows, y) {
                return false;
            }
        }
        // Obligation 1, over every row the scroll moves — the probe's own row included, because
        // skipping one row of a band is not worth a cursor that can be wrong about which.
        let mut cursor = rows.peekable();
        for y in scroll.moved() {
            while cursor.peek().is_some_and(|r| r.y() < y) {
                cursor.next();
            }
            let row = cursor.next_if(|r| r.y() == y);
            if !self.row_lands_on(row.as_ref(), y, scroll.source(y)) {
                return false;
            }
        }
        true
    }

    /// **Obligation 1.** Whether what this frame wants on row `y` is what the mirror is already
    /// showing on row `src` — in every column, undamaged ones included.
    ///
    /// What the frame wants at a column is the packet's cell where the frame damaged it and the
    /// mirror's where it did not, which is the same read the gap merge makes and for the same reason:
    /// a cell nobody has damaged is one an earlier frame put there. So the row is walked as the
    /// alternating spans the runs cut it into rather than column by column — see
    /// [`Mirror::span`] for what that is worth.
    fn row_lands_on(&self, row: Option<&Row<'_>>, y: u16, src: u16) -> bool {
        let mut x = 0u16;
        if let Some(row) = row {
            let mut base = 0usize;
            for r in row.runs {
                if !self.undamaged_span_lands_on(y, src, x, r.lo) {
                    return false;
                }
                // A damaged span is the packet's own cells against the mirror's, and the packet's
                // cells are a frame's: nothing here can be `Cell::UNKNOWN`, so an unknown source
                // cell fails this comparison rather than passing it. Narrowed, because the mirror
                // holds what was sent — see
                // [`damaged_span_is_on_the_terminal`](Serializer::damaged_span_is_on_the_terminal),
                // which is where forgetting it cost every scroll below truecolor.
                let cells = &row.cells[base..base + r.len()];
                if !self.damaged_span_is_on_the_terminal(cells, src, r.lo) {
                    return false;
                }
                base += r.len();
                x = r.hi + 1;
            }
        }
        self.undamaged_span_lands_on(y, src, x, self.mirror.width)
    }

    /// The columns `lo..hi` that this frame did not damage: what it wants there is what the mirror
    /// holds, so they land where the scroll puts them only if the two rows already agree.
    ///
    /// **A column the mirror does not know refuses the scroll, and it is refused explicitly.** Two
    /// unknown cells compare *equal*, so leaving this to the comparison alone is exactly the false
    /// equality ADR 0006 exists to make unreachable — and here it would move eighty rows of a screen
    /// to the wrong place rather than leave one cell stale.
    fn undamaged_span_lands_on(&self, y: u16, src: u16, lo: u16, hi: u16) -> bool {
        let want = self.mirror.span(y, lo, hi);
        !want.contains(&Cell::UNKNOWN) && want == self.mirror.span(src, lo, hi)
    }

    /// **Obligation 2.** Whether row `y` is one the scroll may leave the terminal to erase.
    ///
    /// The terminal blanks what it exposes, so every column this frame does not repaint has to be a
    /// column the frame wants blank — and what the frame wants in an undamaged column is what the
    /// mirror holds. A column the mirror does not know is therefore refused, by the same comparison
    /// that refuses a column holding something: `Cell::UNKNOWN` is not `Cell::BLANK`.
    ///
    /// **The row question is deliberately not asked here.** [`Mirror::is_known`] over the whole row
    /// would be cheaper to write and is *stricter than the obligation*: the columns this frame
    /// repaints need not be known at all, and on the scene this optimisation exists for the frame
    /// repaints every one of them. That is ADR 0006's amendment arriving a second time — the row is
    /// the wrong granularity for a screen no single frame writes whole — and forfeiting the scroll it
    /// is the whole point of is a worse answer than a per-cell walk.
    fn exposed_row_is_clear(&self, rows: PacketRows<'_>, y: u16) -> bool {
        let mut x = 0u16;
        if let Some(row) = rows.row(y) {
            for r in row.runs {
                if !self.span_is_blank(y, x, r.lo) {
                    return false;
                }
                x = r.hi + 1;
            }
        }
        self.span_is_blank(y, x, self.mirror.width)
    }

    fn span_is_blank(&self, y: u16, lo: u16, hi: u16) -> bool {
        self.mirror
            .span(y, lo, hi)
            .iter()
            .all(|c| *c == Cell::BLANK)
    }

    /// Put the scroll on the wire and move the mirror the same way.
    ///
    /// **The band's escapes are skipped where the band is the screen**, because the scrolling region a
    /// terminal starts in *is* the whole screen: `SU` alone then says exactly what `DECSTBM` + `SU` +
    /// `DECSTBM` reset says, eleven bytes cheaper, and a scroll that is worth taking at all is usually
    /// the whole screen. Where the band is narrower the region is set and put straight back, so
    /// nothing outside this function ever runs against a region that is not the screen — which is what
    /// lets `shortest` go on pricing an `LF` as a move rather than as a scroll.
    fn emit_scroll(&mut self, scroll: Scroll, caps: &Capabilities) {
        let (top, bot) = scroll.band;
        self.open_frame(caps);
        let whole_screen = top == 0 && bot == self.mirror.height - 1;
        if !whole_screen {
            self.out.extend_from_slice(b"\x1b[");
            push_num(&mut self.out, u32::from(top) + 1);
            self.out.push(b';');
            push_num(&mut self.out, u32::from(bot) + 1);
            self.out.push(b'r');
        }
        self.out.extend_from_slice(b"\x1b[");
        if scroll.by > 1 {
            push_num(&mut self.out, u32::from(scroll.by));
        }
        self.out.push(if scroll.up { b'S' } else { b'T' });
        if !whole_screen {
            self.out.extend_from_slice(b"\x1b[r");
        }
        self.mirror.scroll(scroll);
        // **The band is fully known afterwards, and that is not free — it is obligation 1 cashed
        // in.** Every column of every row the scroll moved had a *known* want equal to its source, so
        // every destination row inherits knowledge rather than a hole; every row it exposed is a
        // blank the terminal wrote. A destination row that inherited a `Cell::UNKNOWN` would be a
        // false equality for the next frame's filter to skip, which is the one failure ADR 0006 is
        // written to make unreachable — so the row question is asked here, where it is a row.
        debug_assert!(
            (top..=bot).all(|y| self.mirror.is_known(y)),
            "a scroll left the mirror not knowing a row of the band it had just shifted"
        );
        // `DECSTBM` homes the cursor and `SU` does not move it, so the honest record is that nothing
        // is known about it. It costs nothing: `open_frame` has just said the same thing, and the
        // first cell of the frame was going to be an absolute move either way.
        self.cursor = None;
        #[cfg(test)]
        {
            self.scrolls += 1;
        }
    }

    /// Write the bytes every frame opens with, once, on the first cell that actually goes out.
    ///
    /// Mode 2026 where the terminal has it, outside the reset: `?2026h` + `0m` + `?2026l` is §8's
    /// twenty bytes of fixed framing. **The frame is never split on purpose** — a
    /// synchronised-output block spanning two `write` calls is still one block to the terminal, and
    /// a frame split into two blocks tears — so `write_frame`'s partial-write loop is about the
    /// kernel's buffer being smaller than the frame and about nothing else.
    ///
    /// SGR 0 does not close an OSC 8, so the link is cleared here because the frame's own close at
    /// the bottom put it back rather than because the reset does.
    ///
    /// See [`frame_open`](Serializer::frame_open) for why this is deferred rather than written at
    /// the top of `serialize`.
    fn open_frame(&mut self, caps: &Capabilities) {
        if self.frame_open {
            return;
        }
        self.frame_open = true;
        if caps.sync_output() {
            self.out.extend_from_slice(SYNC_BEGIN);
        }
        self.out.extend_from_slice(b"\x1b[0m");
        self.style = Style::DEFAULT;
        self.cursor = None;
        self.prev = None;
        self.non_ascii_on_row = false;
        self.link = LinkId::NONE;
    }

    /// One row: the equality filter, the gap merge, and the emit loop.
    ///
    /// **The filter is one `continue` and nothing else.** What surrounds it is the gap it opens: a
    /// cell the comparison skipped leaves the cursor behind, and stepping over it costs bytes too, so
    /// skipping a cell and stepping over it are one decision and are taken in one place rather than in
    /// two that could disagree.
    fn emit_row(&mut self, row: &Row<'_>, packet: &Packet, caps: &Capabilities) {
        let y = row.y();
        // Whether the mirror is consulted at all, which is a shipping constant and a `cfg(test)`
        // question — [`Filter::Off`] is §8's `span` column. **What the mirror does not know needs no
        // branch here**: it holds `Cell::UNKNOWN`, which no composited frame cell can equal, so the
        // comparison below fails on it exactly as it should. See [`Mirror`].
        let compare = self.compares();
        let mut base = 0usize;
        // Where this row last put a cell, which is where a gap would begin.
        let mut emitted: Option<u16> = None;
        for r in row.runs {
            let cells = &row.cells[base..base + r.len()];
            base += r.len();
            for (i, &cell) in cells.iter().enumerate() {
                // The column comes from the index, not from an advance counter. That is what makes
                // skipping a `CONTINUATION` free of bookkeeping, and spec §8 records the first
                // version getting it wrong in the one case where a run *begins* on a continuation.
                let x = r.lo + i as u16;
                // **Narrowed before the comparison, which is the whole placement.** The mirror holds
                // what the terminal was *sent*, so two colours this depth cannot tell apart have to
                // arrive here already indistinguishable — otherwise the filter under-filters by
                // exactly the amount the depth collapses, and an animated gradient on a 16-colour
                // terminal re-emits every frame for no visible change. See [`crate::quant`].
                let cell = self.narrow(cell);
                if cell.grapheme.is_continuation() {
                    // Before the comparison, and unconditionally. The head already painted both
                    // columns so there is nothing to emit and nothing to decide — but the mirror
                    // records the pair, because the mirror is what the terminal *shows* and the
                    // terminal shows both halves. `emitted` is deliberately not moved: the cursor is
                    // past this column already, and a gap measured from the head is the same gap.
                    self.mirror.set(x, y, cell);
                    continue;
                }
                // The half of the sentinel argument that is a claim about the *frame* rather than
                // about the mirror, so it is asserted where a future compositor would break it.
                debug_assert!(
                    !cell.grapheme.is_empty(),
                    "a composited frame holds no `EMPTY`, which is what lets `Cell::UNKNOWN` be one"
                );
                if compare && cell == self.mirror.cell(x, y) {
                    continue;
                }
                if let Some(from) = emitted
                    && x > from + 1
                {
                    self.consider_gap(row, from, x, packet, caps);
                }
                self.emit_cell(x, y, cell, packet, caps);
                emitted = Some(x);
            }
        }
    }

    /// One style word, narrowed to what this terminal can express, through the one-entry memo.
    ///
    /// The memo is the whole implementation and two rejected alternatives are recorded rather than
    /// re-derived: spec §3 measured a per-cell branch that took the cheap path on an inline word
    /// (it loses, because the branch breaks the mask loop's vectorisation) and a surface-level
    /// *contains nothing to narrow* gate (indistinguishable from the memo alone).
    fn narrow(&mut self, cell: Cell) -> Cell {
        if let Some((from, to)) = self.memo {
            // **Both spellings hit, and the second `==` is not redundant.** The run scan narrows a
            // cell to compare it and `emit_cell` narrows again to be correct whoever called it, so
            // every emitted cell asks twice — once for the word the application wrote and once for
            // the word this returned. A memo keyed on the first spelling alone answers the second
            // with a miss *and then stores it*, so the next cell of the run misses too: the memo
            // thrashes and every emitted cell is narrowed twice. Measured on
            // `every-cell-a-distinct-style` at sixteen colours, where narrowing is 26 ns a word and
            // no two cells share one.
            //
            // Answering `to` for `to` is sound rather than convenient: `to` is a fixed point,
            // because [`Quantiser::color`] is idempotent at every depth.
            if from == cell.style || to == cell.style {
                return Cell::new(cell.grapheme, to);
            }
        }
        let narrowed = self.quant.cell(cell);
        #[cfg(test)]
        {
            self.narrowings += 1;
        }
        self.memo = Some((cell.style, narrowed.style));
        narrowed
    }

    /// Put one cell on the wire, and record it in the mirror.
    ///
    /// Called for a damaged cell the filter kept and for a gap cell the merge decided to paint
    /// through, which is why it is a function: the two arrive by different routes and must not be
    /// two loops that drift.
    ///
    /// **Narrowing is repeated here rather than assumed**, because the gap merge sources columns the
    /// run scan never saw — one out of the packet's row and one out of the mirror, which is already
    /// narrowed. [`Quantiser::color`] is idempotent at every depth precisely so that this costs a
    /// memo hit rather than a flag saying which of the two is in hand.
    fn emit_cell(&mut self, x: u16, y: u16, cell: Cell, packet: &Packet, caps: &Capabilities) {
        let cell = self.narrow(cell);
        if cell.grapheme.is_continuation() {
            // Reachable only from the gap merge, which walks columns rather than a run's cells. The
            // head painted both, and `emit_grapheme` would put a space here: `text_of` cannot render
            // the sentinel and falls back to one.
            self.mirror.set(x, y, cell);
            return;
        }
        self.open_frame(caps);
        // Two cells emitted back to back arrive with nothing between them, and UAX #29 does not know
        // where one cell ended. Forcing a move breaks the adjacency, and a `CUP` is something every
        // terminal has always treated as ending a run of text.
        //
        // An SGR landing between them would separate them too, and skipping the move when the style
        // changes was written and then taken back out: *whether a terminal's own clustering survives
        // an SGR is a claim about other people's software*, and the mode 2027 specification does not
        // make it. The saving was six bytes on a case that needs two adjacent cells holding joinable
        // clusters.
        //
        // **The forced move is the cheapest one that is not nothing, not a `CUP`.** The first draft
        // of `shortest` expressed the force by clearing the cursor, which is what ticket 03 did when
        // `CUP` was the only encoding — and that quietly spent eight bytes where `CR` spends one. The
        // flag says *move*, and the encoder still says *how*.
        let force = self.cursor == Some((x, y)) && self.joins_left(cell.grapheme, packet);
        self.move_to(x, y, force);
        if cell.style != self.style {
            emit_sgr_delta(
                &mut self.out,
                self.style,
                cell.style,
                packet,
                caps,
                self.quant,
            );
            self.style = cell.style;
            self.retarget_link(cell.style, packet, caps);
        }
        emit_grapheme(&mut self.out, cell.grapheme, packet);
        self.mirror.set(x, y, cell);
        self.advance(x, y, cell.grapheme.columns());
        self.prev = Some(cell.grapheme);
        self.non_ascii_on_row |= !is_ascii_scalar(cell.grapheme);
    }

    /// Price the columns strictly between `from` and `to` against the move that skipping them needs,
    /// and paint through them where they are cheaper.
    ///
    /// Both halves of §8's gap merge, because they are one mechanism seen twice: the columns inside a
    /// run that the filter skipped, and the columns between two runs that the packet never carried.
    /// The second is the only place this file emits a cell the packet does not carry — see
    /// [`price_gap`](Serializer::price_gap) for the two conditions that fence it.
    ///
    /// §8 credits it with taking the three dialogs from 1 491 bytes to 1 203. **That scene cannot show
    /// it here**, because §14's `three-dialogs-apart` rewrites every cell of all three dialogs with a
    /// new counter and a new colour every frame and so has no unchanged column to bridge; §8's dialogs
    /// had a live status bar. What it is worth on §14's list is 160 bytes a scene on the three rows
    /// that do produce gaps, and the mechanism is pinned directly by
    /// `tests::two_runs_on_one_row_are_bridged_out_of_the_mirror`.
    fn consider_gap(
        &mut self,
        row: &Row<'_>,
        from: u16,
        to: u16,
        packet: &Packet,
        caps: &Capabilities,
    ) {
        let y = row.y();
        // What the move would cost is what the gap is being priced against, so it is asked of the
        // same function that will spell it. `force` is not a candidate here: a forced move happens
        // because two clusters would join, and this cursor is not where the next cell goes.
        let (_, move_cost) = self.price_move(to, y);
        let price = self.price_gap(row, from + 1, to, packet, self.gap_budget(move_cost));
        if !self.merges(price, move_cost, to - from - 1) {
            return;
        }
        for x in from + 1..to {
            // `price_gap` said yes, so every column here is either the packet's or one the mirror
            // knows. Asked in the same shape it was priced in, because the two answering differently
            // is the only way this could emit a cell nobody has.
            let cell = row
                .at(x)
                .or_else(|| self.mirror.known_cell(x, y))
                .expect("`price_gap` refuses a gap it cannot source every column of");
            self.emit_cell(x, y, cell, packet, caps);
        }
    }

    /// What repainting the columns `from..to` would cost on the wire, or `None` when it may not be
    /// done at all or would cost at least `budget`.
    ///
    /// **The budget is what keeps this proportional to damage rather than to the screen.** A move is
    /// at most eight bytes and every cluster that is not a continuation costs at least one, so the
    /// walk stops after a handful of columns however far apart two runs are — which is what lets the
    /// three dialogs be priced without anybody scanning the two hundred blank columns between them.
    ///
    /// # The two conditions on a column the packet does not carry
    ///
    /// Such a column is repainted out of the mirror, and the mirror is only a fact about the terminal
    /// where it says so — hence [`Mirror::known_cell`], which is the shape that cannot be forgotten.
    ///
    /// **That check is deliberately redundant today and is kept anyway.** `Cell::UNKNOWN`'s grapheme
    /// is the `EMPTY` sentinel, and `text_of` cannot render one, so the `?` two lines below refuses an
    /// unknown column on its own. What `known_cell` buys is that the refusal does not *depend* on how
    /// the unknown state happens to be spelled: a future `Cell::UNKNOWN` whose grapheme were a real
    /// scalar would walk straight past `text_of` and be painted onto the screen. A mutation that
    /// deletes it therefore breaks no test, which is exactly why the reason is written here.
    ///
    /// The second condition is the handles. A mirror cell records what was emitted, and what was
    /// emitted may name a cluster, an extended style or a URI that **this** packet's side tables do
    /// not carry: a frame only resolves the handles its own damaged cells name (`Packet::pack`). Sent
    /// anyway, that cell would go out as a space in the terminal's own colours with no hyperlink —
    /// `emit_grapheme` and `channels_of` both answer that way rather than inventing — and the mirror
    /// would then record a cell the screen does not show. **That is the one way this optimisation
    /// could put something on screen no frame asked for**, so an unresolvable handle refuses the
    /// whole gap rather than being painted approximately.
    fn price_gap(
        &self,
        row: &Row<'_>,
        from: u16,
        to: u16,
        packet: &Packet,
        budget: usize,
    ) -> Option<usize> {
        let y = row.y();
        let mut price = 0usize;
        let mut style = self.style;
        let mut scratch = [0u8; 4];
        for x in from..to {
            let cell = match row.at(x) {
                Some(cell) => cell,
                None => self.mirror.known_cell(x, y)?,
            };
            // Free, because nothing is emitted for one. It is also why the bound above is two
            // columns per byte rather than one.
            if cell.grapheme.is_continuation() {
                continue;
            }
            price += text_of(cell.grapheme, packet, &mut scratch)?.len();
            // The same refusal as the cluster's, on the other three channels a cell can name. An
            // extended word spends bits 51..0 on a handle, so a cell whose handle this packet did not
            // resolve would be painted in `Color::DEFAULT` with no hyperlink — `channels_of` answers
            // that way rather than inventing — and the mirror would then record an extended cell the
            // screen shows plain. Unlike the cluster's, this one is **not** redundant with anything.
            if let Some(handle) = cell.style.ext_handle() {
                let ext = packet.ext(handle)?;
                if !ext.link.is_none() && packet.link(ext.link).is_none() {
                    return None;
                }
            }
            // Narrowed before it is compared, for the reason the filter is: two words this depth
            // cannot tell apart cost one SGR and not two, and a gap priced against the unnarrowed
            // words would be priced against a wire nobody is going to write. Without the memo,
            // because a gap is a handful of columns and this takes `&self`.
            let narrowed = self.quant.style(cell.style);
            if narrowed != style {
                price += SGR_FLOOR;
                style = narrowed;
            }
            if price >= budget {
                return None;
            }
        }
        Some(price)
    }

    /// Open, change or close the terminal's hyperlink to match the style just emitted.
    ///
    /// Called only where the style word changed, which is the same place the SGR delta goes out:
    /// cells in a run share a word, so this is a per-distinct-style cost and not a per-cell one.
    ///
    /// Where the terminal has no OSC 8 nothing is emitted and [`link`](Serializer::link) stays
    /// [`LinkId::NONE`], so the close at the end of the frame has nothing to do either.
    fn retarget_link(&mut self, style: Style, packet: &Packet, caps: &Capabilities) {
        if !caps.hyperlinks {
            return;
        }
        let want = channels_of(style, packet, self.quant).link;
        if want == self.link {
            return;
        }
        // A URI the packet does not carry is the same disagreement `channels_of` answers
        // `Color::DEFAULT` for: `pack` and `serialize` think this is a different frame. Closing is
        // the answer that invents nothing.
        let uri = packet.link(want).filter(|_| !want.is_none());
        emit_osc8(&mut self.out, uri);
        self.link = if uri.is_some() { want } else { LinkId::NONE };
    }

    /// Where the terminal's cursor ends up after printing one cell at `(x, y)`.
    ///
    /// With auto-wrap off it stops in the last column instead of advancing off it, so that is what
    /// is recorded rather than a position one past the end. Ticket 03 could not tell the two apart —
    /// a mutation swapping them left every test green, because runs are disjoint and ascending, so no
    /// later move on the same row ever targets a column already written — and **`shortest` is what
    /// makes it matter**: `CUF`'s distance is measured from this position, so a cursor model that is
    /// off by one puts every relative move on the row one column out.
    fn advance(&mut self, x: u16, y: u16, columns: u16) {
        self.cursor = Some(((x + columns).min(self.mirror.width.saturating_sub(1)), y));
    }

    /// Whether the cluster about to be emitted would join the one before it into a single cluster.
    ///
    /// This is spec §10's rule — *a width disagreement is permanent once a mirror exists* — arriving
    /// one ticket early and for the neighbouring reason. §10 states it as `CHA` rather than `CUF`
    /// after a non-ASCII run, which is about the cursor *compounding* an error; this is about the
    /// bytes themselves re-segmenting. Both are the same underlying fact: **what the engine put in
    /// two cells is not what the terminal reads unless something separates them.**
    ///
    /// Found by the round trip, not by reading. A cluster ending in ZWJ followed by a pictograph
    /// (GB11) and a lone regional indicator followed by another (GB12/13) both merge, and the frame
    /// then holds two cells where the terminal shows one.
    ///
    /// The question is asked of the same tables the verbs segmented with, rather than of a list of
    /// rules copied out of UAX #29 — a second list is a second thing to keep in step. Two ASCII
    /// scalars can never join, which is the fast path and covers nearly every cell ever emitted.
    fn joins_left(&self, next: GraphemeId, packet: &Packet) -> bool {
        let Some(prev) = self.prev else {
            return false;
        };
        if is_ascii_scalar(prev) && is_ascii_scalar(next) {
            return false;
        }
        let (mut prev_buf, mut next_buf) = ([0u8; 4], [0u8; 4]);
        let Some(prev_text) = text_of(prev, packet, &mut prev_buf) else {
            return false;
        };
        let Some(next_text) = text_of(next, packet, &mut next_buf) else {
            return false;
        };
        let Some(first) = next_text.chars().next() else {
            return false;
        };
        let mut cursor = crate::ucd::Cursor::new();
        for ch in prev_text.chars() {
            cursor.is_break(ch);
        }
        !cursor.is_break(first)
    }

    /// Put the cursor at `(x, y)` in the fewest bytes §8's encoding set allows.
    ///
    /// `CUP`, `CHA`, `CUF`, `CR`, `CR`+`LF`s — **priced by digit count, with no lookup table.**
    /// Every candidate's cost is a small sum and the cheapest wins; there is no per-move search over
    /// five encodings including content overwrite, which is cellbuf's version and is refused along
    /// with its `ICH`/`DCH` line editing that the equality filter subsumes.
    ///
    /// It is worth **0.2% to 15%**, and the largest win is the chart — the scene with the most runs.
    /// Even there the win is not the *encoding*: absolute and natural tie, because a chart's runs are
    /// never contiguous, and the 848 bytes come almost entirely from choosing `CUF` and `CHA` over
    /// `CUP`.
    ///
    /// # `force`
    ///
    /// True when the cursor is already where the next cell goes and the bytes must nevertheless be
    /// separated — [`joins_left`](Serializer::joins_left). The cheapest non-empty move is then
    /// emitted, which is `CR` in column zero and `CHA` anywhere else. Both are absolute, which is
    /// also what [`non_ascii_on_row`](Serializer::non_ascii_on_row) would have demanded: a forced
    /// move only ever happens because a non-ASCII cluster is involved.
    ///
    /// # Two things a candidate must not be
    ///
    /// **`CUF` is refused on a row that has emitted a non-ASCII cluster** (spec §10) — that is the
    /// one rule here that is correctness rather than byte count.
    ///
    /// **An `LF` is only ever a downward move**, so it can never scroll: runs arrive in ascending row
    /// order, so `y > cy` implies `cy < height - 1`, and the feeds land on `y` at the latest. A
    /// terminal scrolls on `LF` from the last row whatever auto-wrap says, and this is why that never
    /// happens rather than a mode that prevents it.
    fn move_to(&mut self, x: u16, y: u16, force: bool) {
        if !force && self.cursor == Some((x, y)) {
            return;
        }
        let was = self.cursor;
        let (best, cost) = self.price_move(x, y);

        // The report §15 is owed: what the rule cost on this move, which is the gap between what
        // was chosen and the `CUF` that was refused. Zero on every move where `CUF` would not have
        // won anyway, which is most of them.
        #[cfg(test)]
        if let Some((cx, cy)) = was
            && y == cy
            && x > cx
            && self.non_ascii_on_row
        {
            self.cha_rule_bytes += cost.saturating_sub(3 + omissible((x - cx) as u32));
        }
        #[cfg(not(test))]
        let _ = cost;

        match best {
            Move::Cup => self.cup(x, y),
            Move::Cha => {
                self.out.extend_from_slice(b"\x1b[");
                if x > 0 {
                    push_num(&mut self.out, x as u32 + 1);
                }
                self.out.push(b'G');
            }
            Move::Cuf(by) => {
                self.out.extend_from_slice(b"\x1b[");
                if by > 1 {
                    push_num(&mut self.out, by as u32);
                }
                self.out.push(b'C');
            }
            Move::Cr => self.out.push(b'\r'),
            Move::Feed => {
                let (cx, cy) = was.expect("`Feed` is only a candidate against a known cursor");
                if cx != 0 {
                    self.out.push(b'\r');
                }
                for _ in cy..y {
                    self.out.push(b'\n');
                }
            }
        }
        if was.is_none_or(|(_, cy)| y != cy) {
            self.non_ascii_on_row = false;
        }
        self.cursor = Some((x, y));
    }

    /// The cheapest spelling of a move to `(x, y)`, and what it costs in bytes.
    ///
    /// Every candidate priced, then the cheapest taken. Written as a list rather than as nested
    /// branches because the price is a sum over candidates and the winner is only known at the end —
    /// and because a branch that emits as it prices is how the first version of this ended up
    /// spending eight bytes on a forced move.
    ///
    /// **Separated from the emission because the gap merge has to ask the price of a move it may not
    /// make.** Which also means this answers for a cursor that is already at `(x, y)`: the caller
    /// owns the early return that makes a no-op free, and `consider_gap` wants the real number.
    fn price_move(&self, x: u16, y: u16) -> (Move, usize) {
        let cup = (Move::Cup, 4 + digits(y as u32 + 1) + digits(x as u32 + 1));
        let Some((cx, cy)) = self.cursor else {
            // Nothing is known about where the cursor is — the first move of a frame — so the only
            // truthful encoding is the absolute one.
            return cup;
        };
        let same_row = y == cy;
        let candidates = [
            // One byte, and nothing beats it. `CR` is absolute in the column, so §10's rule has no
            // objection to it.
            (same_row && x == 0).then_some((Move::Cr, 1)),
            same_row.then(|| (Move::Cha, 3 + omissible(x as u32 + 1))),
            (same_row && x > cx && !self.non_ascii_on_row)
                .then(|| (Move::Cuf(x - cx), 3 + omissible((x - cx) as u32))),
            // `CR` then one `LF` per row, or the feeds alone when the cursor is already in column
            // zero. One-byte controls against a parameterised CSI, which is where §8's "fewest
            // bytes" and "fewest sequences" stop pulling in opposite directions.
            (!same_row && x == 0 && y > cy)
                .then(|| (Move::Feed, usize::from(cx != 0) + (y - cy) as usize)),
        ];
        let mut best = cup;
        for candidate in candidates.into_iter().flatten() {
            if candidate.1 < best.1 {
                best = candidate;
            }
        }
        best
    }

    /// `CSI y;x H`, and nothing else: the cursor and §10's row flag are the caller's to update, so
    /// that there is one place each is written.
    fn cup(&mut self, x: u16, y: u16) {
        self.out.extend_from_slice(b"\x1b[");
        push_num(&mut self.out, y as u32 + 1);
        self.out.push(b';');
        push_num(&mut self.out, x as u32 + 1);
        self.out.push(b'H');
    }
}

/// Which of §8's five cursor encodings a move is spelled with.
///
/// An enum rather than five branches writing bytes as they are priced, because the price is a sum
/// over candidates and the winner is only known at the end. `CUB` is deliberately absent: runs are
/// disjoint and ascending, so nothing ever moves left except to column zero, which `CR` already
/// spells in one byte.
enum Move {
    /// `CSI y;x H`.
    Cup,
    /// `CSI x G` — absolute column, same row.
    Cha,
    /// `CSI n C` — relative, and refused after a non-ASCII cluster on this row.
    Cuf(u16),
    /// `\r`.
    Cr,
    /// `\r` where needed, then one `\n` per row.
    Feed,
}

/// A packet's damaged rows, in the shape both the filter and the scroll pre-pass need.
///
/// Runs arrive in ascending row order and are disjoint (§14's gate #2), so a row's runs are a
/// contiguous slice of the packet and finding one is a walk rather than a sort. `Copy`, because the
/// pre-pass asks the same packet three questions and each of them wants its own cursor.
#[derive(Clone, Copy)]
struct PacketRows<'a> {
    runs: &'a [Run],
    /// Every remaining run's cells, concatenated in run order: the packet's own layout.
    cells: &'a [Cell],
}

impl<'a> PacketRows<'a> {
    fn new(packet: &'a Packet) -> PacketRows<'a> {
        PacketRows {
            runs: packet.runs(),
            cells: packet.cells(),
        }
    }

    /// The one row `y`, or `None` where this frame damaged nothing on it.
    ///
    /// A walk from the front rather than a search, for the reason [`Row::at`] is a walk: the index
    /// that would make this a lookup costs the screen's height on a frame that damaged one cell,
    /// which is the shape the engine's own invariant forbids. Asked only by the scroll pre-pass, and
    /// only about the rows of a band it is already walking.
    fn row(self, y: u16) -> Option<Row<'a>> {
        self.take_while(|r| r.y() <= y).find(|r| r.y() == y)
    }
}

impl<'a> Iterator for PacketRows<'a> {
    type Item = Row<'a>;

    fn next(&mut self) -> Option<Row<'a>> {
        let y = self.runs.first()?.y;
        let mut last = 0usize;
        let mut end = self.runs[0].len();
        while last + 1 < self.runs.len() && self.runs[last + 1].y == y {
            last += 1;
            end += self.runs[last].len();
        }
        let row = Row {
            runs: &self.runs[..=last],
            cells: &self.cells[..end],
        };
        self.runs = &self.runs[last + 1..];
        self.cells = &self.cells[end..];
        Some(row)
    }
}

/// One row of a packet: the runs that damaged it, and the cells they carry.
///
/// The filter plans in rows because a gap between two runs on one row is bridgeable and a gap
/// between two rows is not. Runs are disjoint and arrive in row order, so this is a borrowed slice
/// of the packet rather than anything gathered — which is what keeps the filter allocating nothing.
struct Row<'a> {
    runs: &'a [Run],
    /// Every run's cells, concatenated in run order: the packet's own layout, narrowed.
    cells: &'a [Cell],
}

impl Row<'_> {
    fn y(&self) -> u16 {
        self.runs[0].y
    }

    /// The packet's cell at column `x`, or `None` when no run of this row covers it.
    ///
    /// A walk over the row's runs rather than a lookup table: it is asked only about the columns of a
    /// gap, a gap is bounded by the bytes of the move it is priced against, and a row has a handful
    /// of runs. Building an index per row would cost the screen's width on a frame that damaged one
    /// cell, which is exactly the shape the engine's own invariant forbids.
    fn at(&self, x: u16) -> Option<Cell> {
        let mut base = 0usize;
        for r in self.runs {
            if x >= r.lo && x <= r.hi {
                return Some(self.cells[base + (x - r.lo) as usize]);
            }
            base += r.len();
        }
        None
    }
}

/// How the mirror is consulted. **One value ships**; the other three are the instrument.
///
/// §8's conclusion about the gap merge is that *there is no threshold*, and the sweep behind it is
/// the evidence rather than the claim: the chart gets monotonically worse as a cell-count threshold
/// climbs 0 → 24 while the dialogs get better and then flat, so the two scenes want opposite
/// thresholds. **A conclusion of that shape cannot be reproduced by the configuration that won** —
/// it needs the ones that lost. So they are here, reachable from tests and from nowhere else:
/// `Serializer` carries no field for them outside `cfg(test)`, and
/// `crate::gates::the_equality_filter_needs_no_threshold` is what runs the sweep.
///
/// This is the same shape spec §8 refuses for the serializer itself — *the prototype was `Options`-
/// shaped only so the variants could be measured* — and the difference is where it lives: a variant
/// that no shipping build can construct is an instrument, and a variant a caller can select is a
/// knob.
#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum Filter {
    /// What ships: compare every cell against the mirror, and price a gap in bytes.
    #[default]
    Bytes,
    /// Compare, and never merge a gap: §8's `strict` column.
    Strict,
    /// Compare, and merge any gap of at most `n` columns: §8's `gap 6 cells` column, and every point
    /// of its sweep.
    Cells(u16),
    /// Do not compare at all: §8's `span` column, which is what impl 13 shipped.
    Off,
}

/// The fewest bytes an SGR that says anything can take: `CSI`, one digit, `m`.
///
/// Charged once per style change inside a gap, and **deliberately a floor rather than the cost**: the
/// real one is a function of which channels moved, and it is computed nowhere but inside
/// `emit_sgr_delta`, which writes as it computes. Pricing a gap therefore uses a lower bound, and
/// where the bound is wrong it is wrong in the direction of painting through — by at most four bytes
/// per style change inside the gap.
///
/// §8 calibrated it there and its own table is the evidence: the byte-priced rule lands at 4 623
/// bytes on the chart against `strict`'s 4 581 — forty-two bytes **worse** — and at 1 203 on the
/// dialogs against `strict`'s 1 491. A rule that never overshot could not have produced the first
/// number, and a rule that always overshot could not have produced the second.
const SGR_FLOOR: usize = 4;

/// DEC mode 2026 on: the terminal holds the frame back until the block closes.
const SYNC_BEGIN: &[u8] = b"\x1b[?2026h";
/// DEC mode 2026 off, which is what makes the frame appear at once.
const SYNC_END: &[u8] = b"\x1b[?2026l";

/// Write one cell's cluster.
///
/// A scalar handle *is* its own bytes; anything above the scalars is a lookup in the packet's own
/// arena, which is where the app thread put a copy at pack time. **The engine table is never
/// reached from here** — that is the whole of ADR 0011's second sentence, and it is what lets the
/// app thread sweep its interner while a frame is being written.
fn emit_grapheme(out: &mut Vec<u8>, g: GraphemeId, packet: &Packet) {
    let mut buf = [0u8; 4];
    let cluster = text_of(g, packet, &mut buf).unwrap_or(" ");
    out.extend_from_slice(cluster.as_bytes());
}

/// The bytes a handle names: the packet's copy for a cluster, the scalar itself for a scalar.
fn text_of<'a>(g: GraphemeId, packet: &'a Packet, scratch: &'a mut [u8; 4]) -> Option<&'a str> {
    if let Some(cluster) = packet.cluster(g) {
        return Some(cluster);
    }
    Some(g.as_scalar()?.encode_utf8(scratch))
}

/// The cell that costs nothing to check: an ASCII scalar, which nothing can join to.
fn is_ascii_scalar(g: GraphemeId) -> bool {
    g.payload() < 0x80
}

/// Write the SGR that turns `old` into `new`.
///
/// A differential SGR is worth it and costs nothing to decide, because the decision is one style
/// compare and the decomposition is off the hot path by construction: a realistic full-screen frame
/// emits **one** SGR sequence for 24 000 cells.
///
/// # Both sides lose the flags the terminal does not render, and it is one line for a reason
///
/// §10 says an unsupported attribute is *dropped silently at serialise time*, and production ticket 10
/// is what made that true. The drop is **both sides at once** and nothing below it changes: mask only
/// `new` and a frame that turns overline off emits `SGR 55` for a bit the terminal never had, which is
/// a byte spent to undo nothing and a diff computed against a style it was never in.
///
/// It is [`Quantiser::attrs`] rather than a mask spelled here, so the wire and
/// [`crate::quant::OnTheWire`] read the same field through the same function — and it is **repeated**
/// rather than assumed, exactly as [`Serializer::emit_cell`] repeats the colour narrowing: every word
/// that arrives here has already been through [`Quantiser::style`], masking is idempotent, and what
/// the repetition buys is that this function is correct whoever calls it.
fn emit_sgr_delta(
    out: &mut Vec<u8>,
    old: Style,
    new: Style,
    packet: &Packet,
    caps: &Capabilities,
    quant: Quantiser,
) {
    let (old, new) = (quant.attrs(old), quant.attrs(new));
    let mark = out.len();
    out.extend_from_slice(b"\x1b[");
    let mut params = 0u32;

    let removed = old.attrs() & !new.attrs();
    let added = new.attrs() & !old.attrs();

    // SGR 22 resets bold *and* dim together. There is no un-bold that leaves dim standing, so
    // removing either means emitting 22 and reapplying the other. A naive per-attribute on/off loop
    // silently drops dim whenever bold turns off (spec §8).
    if removed & (crate::style::BOLD | crate::style::DIM) != 0 {
        param(out, &mut params, 22);
        if new.attrs() & crate::style::BOLD != 0 {
            param(out, &mut params, 1);
        }
        if new.attrs() & crate::style::DIM != 0 {
            param(out, &mut params, 2);
        }
    } else {
        if added & crate::style::BOLD != 0 {
            param(out, &mut params, 1);
        }
        if added & crate::style::DIM != 0 {
            param(out, &mut params, 2);
        }
    }

    for (bit, on, off) in [
        (crate::style::ITALIC, 3, 23),
        (crate::style::BLINK, 5, 25),
        (crate::style::REVERSE, 7, 27),
        (crate::style::CONCEAL, 8, 28),
        (crate::style::STRIKETHROUGH, 9, 29),
        (crate::style::OVERLINE, 53, 55),
    ] {
        if added & bit != 0 {
            param(out, &mut params, on);
        } else if removed & bit != 0 {
            param(out, &mut params, off);
        }
    }

    if old.underline_style() != new.underline_style() {
        match new.underline_style() {
            0 => param(out, &mut params, 24),
            // SGR 21 is never emitted: ECMA-48 assigns it "doubly underlined" and a meaningful
            // population of terminals implements it as "bold off". Double underline is `4:2`.
            n => {
                param(out, &mut params, 4);
                out.push(b':');
                push_num(out, n as u32);
            }
        }
    }

    let was = channels_of(old, packet, quant);
    let now = channels_of(new, packet, quant);
    let legacy = caps.legacy_sgr();
    if was.fg != now.fg {
        emit_color(out, &mut params, now.fg, FOREGROUND, legacy);
    }
    if was.bg != now.bg {
        emit_color(out, &mut params, now.bg, BACKGROUND, legacy);
    }
    // The underline colour reaches the wire whatever the terminal answered, because there is nothing
    // to answer: SGR 58 has no query and a terminal that does not implement it ignores the sequence.
    // What *is* selected is the spelling — ConPTY parses only the semicolon form, which is
    // `legacy_sgr` one parameter along and invisible to whoever is looking at 38 and 48.
    if was.ul != now.ul {
        let conpty = caps.underlines() == Underlines::ConPty;
        emit_color(out, &mut params, now.ul, UNDERLINE, conpty);
    }

    if params == 0 {
        // Nothing to say. An empty `CSI m` is `CSI 0 m`, a reset, which is emphatically not
        // nothing — so the opener is taken back instead.
        out.truncate(mark);
        return;
    }
    out.push(b'm');
}

/// The four channels a style word paints with, whichever side of the extended bit it is on.
///
/// An extended word has no inline colours — bits 51..0 are the handle — so all four come from the
/// packet's own copy of the table, which is where the app thread put them at pack time. The engine
/// table is never reached from here, and that is ADR 0011's second sentence again.
///
/// A handle the packet does not carry means `pack` and `serialize` disagree about which frame this
/// is, and there is nothing truthful to paint: the terminal's own colours and no hyperlink are the
/// one answer that invents nothing.
///
/// # Narrowing happens here, and for the extended arm this is the *only* place it can
///
/// An inline word arrives already narrowed — the run scan did it, before the mirror comparison — and
/// [`Quantiser::color`] is idempotent, so asking again costs a few instructions per style change and
/// buys the property that this function is correct whoever calls it. **An extended word cannot be
/// narrowed before this point**: bits 51..0 are a handle, and rewriting them would be rewriting the
/// identity the mirror compares on. See [`Mirror`] for what that leaves on the table and why it is
/// bytes rather than correctness.
fn channels_of(style: Style, packet: &Packet, quant: Quantiser) -> crate::exts::ExtStyle {
    let inline = |fg, bg| crate::exts::ExtStyle {
        fg,
        bg,
        ul: Color::DEFAULT,
        link: LinkId::NONE,
    };
    let Some(handle) = style.ext_handle() else {
        return inline(
            quant.color(style.foreground()),
            quant.color(style.background()),
        );
    };
    match packet.ext(handle) {
        Some(e) => quant.channels(e),
        None => inline(Color::DEFAULT, Color::DEFAULT),
    }
}

/// Open, retarget or close the terminal's hyperlink.
///
/// `None` closes: `OSC 8 ; ; ST` with an empty URI is how the sequence says *no link here*, so
/// opening and closing are one shape with one spelling. The `id=` parameter is deliberately empty —
/// it exists so a terminal can join two runs of one logical link across a wrap, and with auto-wrap
/// off for the lifetime of the alt screen there are no wrapped runs to join.
///
/// `ST` is spelled `ESC \` rather than `BEL`: two bytes against one, and the one is xterm's
/// deviation rather than the standard.
fn emit_osc8(out: &mut Vec<u8>, uri: Option<&str>) {
    out.extend_from_slice(b"\x1b]8;;");
    if let Some(uri) = uri {
        out.extend_from_slice(uri.as_bytes());
    }
    out.extend_from_slice(b"\x1b\\");
}

/// Which of the three colour channels a `Color` is being emitted into.
///
/// The three differ in more than a base number, which is why this is a type: only the first two have
/// short forms for the low sixteen palette entries, and only the third spells *no colour* as a number
/// that is not `base + 9`.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Channel {
    /// The parameterised selector: `38`, `48`, `58`. **Not the short-form base plus eight** — that
    /// arithmetic is true of the first two and false of the third, and writing `base + 8` here is how
    /// the first version of this emitted `SGR 30:2::r:g:b` and painted every truecolor foreground as
    /// palette entry zero.
    extended: u32,
    /// What *the terminal's own colour* is spelled as: `39`, `49`, `59`.
    default: u32,
    /// Where `SGR 30`-`37` and `90`-`97` begin for this channel, where they exist at all.
    short: Option<u32>,
}

const FOREGROUND: Channel = Channel {
    extended: 38,
    default: 39,
    short: Some(30),
};
const BACKGROUND: Channel = Channel {
    extended: 48,
    default: 49,
    short: Some(40),
};
/// The underline colour, which has **no** short forms: there is no `SGR 58`-equivalent of `31`, so
/// palette entry 1 is spelled the long way like entry 200.
const UNDERLINE: Channel = Channel {
    extended: 58,
    default: 59,
    short: None,
};

/// Colour emission, spec §8: default is 39/49; indices under 16 use 30-37 / 90-97 and their
/// background forms rather than the long one; truecolor is three channels.
///
/// # The two spellings, and which is the default
///
/// The parameterised forms exist twice over. **Modern** is ITU-T T.416's colon form —
/// `38:2::r:g:b`, with the empty colour-space id the standard puts there — and **legacy** is xterm's
/// pre-ITU-T semicolon form, `38;2;r;g;b`. The modern one is what goes out unless something says
/// otherwise, and `Overrides::legacy_sgr`, `VITUI_FORCE_LEGACY_SGR` and four quirk entries are the
/// somethings: ConPTY, Termux, VSCode's integrated terminal and JetBrains' all mis-parse the colon
/// form. [`crate::quirks`] is where the four are, and where the count is joined to them.
///
/// **§8's own table spells the semicolon form and §10 names the legacy one *pre-ITU-T*, and the two
/// sentences cannot both be about the default.** The reading taken here is §10's, because it is the
/// one with a mechanism: four quirk entries force `legacy` on terminals that parse only semicolons,
/// so `legacy` cannot be what a terminal with no quirk entry receives. §8's spellings are the
/// configuration its byte tables were measured on, which is why the colon form's one extra byte per
/// parameterised colour shows up as a wire-budget number that had to be re-measured. Recorded in
/// impl 13's Progress rather than resolved by this comment.
fn emit_color(out: &mut Vec<u8>, params: &mut u32, c: Color, ch: Channel, legacy: bool) {
    /// `;` for the pre-ITU-T form, `:` for the modern one.
    fn sep(out: &mut Vec<u8>, legacy: bool) {
        out.push(if legacy { b';' } else { b':' });
    }
    match c.tag() {
        TAG_DEFAULT => param(out, params, ch.default),
        TAG_INDEXED => {
            let i = c.payload();
            match ch.short {
                Some(base) if i < 8 => param(out, params, base + i),
                Some(base) if i < 16 => param(out, params, base + 60 + (i - 8)),
                _ => {
                    param(out, params, ch.extended);
                    sep(out, legacy);
                    push_num(out, 5);
                    sep(out, legacy);
                    push_num(out, i);
                }
            }
        }
        TAG_RGB => {
            let v = c.payload();
            param(out, params, ch.extended);
            sep(out, legacy);
            push_num(out, 2);
            // T.416's colour-space id, which is empty and has to be there: `38:2::r:g:b`. The
            // semicolon form has no such parameter, and a terminal that wants one form and is given
            // the other reads the channels off by one and paints the wrong colour.
            if !legacy {
                out.push(b':');
            }
            for shift in [16, 8, 0] {
                sep(out, legacy);
                push_num(out, (v >> shift) & 0xFF);
            }
        }
        // `tag()` returns a `u32`, so the match has to be total. Tag 3 is reserved, `Color` has no
        // constructor that produces it, and this arm exists to satisfy the compiler rather than to
        // handle a case — the safe total answer being the terminal's own colour.
        _ => param(out, params, ch.default),
    }
}

fn param(out: &mut Vec<u8>, params: &mut u32, n: u32) {
    if *params > 0 {
        out.push(b';');
    }
    *params += 1;
    push_num(out, n);
}

/// How many decimal digits `n` takes, which is the whole of §8's pricing.
///
/// A loop rather than a table: the numbers here are a row, a column or a distance on an 80x24 to
/// 300x80 screen, so this runs one to three times and a table would be a table to keep in step.
fn digits(n: u32) -> usize {
    let mut d = 1;
    let mut n = n;
    while n >= 10 {
        n /= 10;
        d += 1;
    }
    d
}

/// The digits a parameter costs when a value of one may be left out entirely.
///
/// `CSI C` and `CSI G` both default their parameter to one, so `CUF` by one column and `CHA` to
/// column one are three bytes rather than four. It is one byte and it is the caret's kind of frame
/// that notices.
fn omissible(n: u32) -> usize {
    if n == 1 { 0 } else { digits(n) }
}

/// Decimal, into the buffer, with no formatting machinery and no allocation.
fn push_num(out: &mut Vec<u8>, n: u32) {
    if n == 0 {
        out.push(b'0');
        return;
    }
    let mut buf = [0u8; 10];
    let mut i = buf.len();
    let mut n = n;
    while n > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    out.extend_from_slice(&buf[i..]);
}

/// The final byte of every CSI in `out`, which is what a property about *which sequences a frame
/// spent* is counted from.
///
/// At module level rather than inside `tests` because two files ask: this one counts moves and style
/// changes with it, and `crate::gates` asserts an **absence** with it — `DECSLRM` is neither queried
/// nor used, and a second copy of this could quietly answer differently.
#[cfg(test)]
pub(crate) fn csi_finals(out: &[u8]) -> impl Iterator<Item = u8> {
    let mut in_csi = false;
    out.iter().enumerate().filter_map(move |(i, b)| {
        if *b == 0x1b {
            in_csi = false;
            return None;
        }
        if i > 0 && out[i - 1] == 0x1b && *b == b'[' {
            in_csi = true;
            return None;
        }
        if in_csi && (0x40..=0x7E).contains(b) {
            in_csi = false;
            return Some(*b);
        }
        None
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caps::ColorDepth;
    use crate::cell::GraphemeId;
    use crate::geom::Rect;
    use crate::style::Style;
    use crate::surface::Surface;
    use crate::term_model::TermModel;

    /// A modern terminal with everything this file can reach: truecolor, OSC 8, the colon SGR form,
    /// the standard underline spelling and **no** synchronised output.
    ///
    /// 2026 is off because it would put eight bytes on either end of every byte string below, and
    /// the frames that are *about* the framing pin it on for themselves.
    fn modern() -> Capabilities {
        Capabilities::on_the_wire(ColorDepth::TrueColor, false, Underlines::Standard, false)
    }

    fn bytes_with(frame: &Surface, caps: &Capabilities) -> Vec<u8> {
        let mut runs = Vec::new();
        frame.damage().for_each_run(|r| runs.push(r));
        let mut packet = Packet::new();
        packet.pack_cells(&runs, frame, frame.tables(), false, 1);
        let (w, h) = frame.size();
        let mut s = Serializer::new(w, h);
        s.serialize(&packet, caps).to_vec()
    }

    fn bytes_for(frame: &Surface) -> Vec<u8> {
        bytes_with(frame, &modern())
    }

    fn text(s: &[u8]) -> String {
        String::from_utf8_lossy(s).replace('\x1b', "ESC")
    }

    /// The SGR one style change spells, on a terminal `caps` describes.
    fn sgr(old: Style, new: Style, caps: &Capabilities) -> String {
        let mut out = Vec::new();
        emit_sgr_delta(
            &mut out,
            old,
            new,
            &Packet::new(),
            caps,
            Quantiser::for_terminal(caps),
        );
        text(&out)
    }

    /// Replay the bytes and read the screen back. The round trip is the instrument for anything
    /// about *where a glyph lands*; a byte string would pin the encoding, which is the part
    /// ticket 15 is still allowed to change.
    fn replay(frame: &Surface) -> TermModel {
        let (w, h) = frame.size();
        let mut term = TermModel::new(w, h);
        // A set of its own is enough here: every scene in this module is scalars, and a scalar
        // handle is identity in any table. `Harness` is where a shared one is load-bearing.
        let mut tables = crate::tables::Tables::new();
        term.feed(&bytes_for(frame), &mut tables);
        assert_eq!(term.unrecognised(), 0);
        term
    }

    fn glyphs(term: &TermModel, y: u16, w: u16) -> String {
        (0..w)
            .map(|x| term.cell(x, y).grapheme.as_scalar().unwrap_or('?'))
            .collect()
    }

    /// How many cursor-positioning sequences the frame spent. Counting them is a property; naming
    /// the bytes they are spelled with is not.
    fn moves(out: &[u8]) -> usize {
        csi_finals(out)
            .filter(|b| matches!(b, b'H' | b'G' | b'C' | b'd' | b'f'))
            .count()
    }

    /// How many SGR sequences the frame spent, not counting the reset it opens with.
    fn style_changes(out: &[u8]) -> usize {
        csi_finals(out).filter(|b| *b == b'm').count() - 1
    }

    #[test]
    fn an_empty_packet_emits_nothing() {
        assert!(bytes_for(&Surface::new(8, 2)).is_empty());
    }

    #[test]
    fn a_frame_opens_with_an_sgr_reset() {
        let mut f = Surface::new(8, 2);
        f.root().text(0, 0, "a", Style::new());
        assert!(bytes_for(&f).starts_with(b"\x1b[0m"));
    }

    #[test]
    fn a_run_costs_one_cursor_move() {
        let mut f = Surface::new(8, 2);
        f.root().text(2, 1, "abc", Style::new());
        assert_eq!(moves(&bytes_for(&f)), 1);
        assert_eq!(glyphs(&replay(&f), 1, 8), "  abc   ");
    }

    #[test]
    fn two_runs_on_one_row_cost_two_moves() {
        let mut f = Surface::new(20, 1);
        f.root().text(0, 0, "ab", Style::new());
        f.root().text(10, 0, "cd", Style::new());
        assert_eq!(moves(&bytes_for(&f)), 2);
        assert_eq!(glyphs(&replay(&f), 0, 20), "ab        cd        ");
    }

    #[test]
    fn a_style_change_inside_a_run_emits_one_sgr() {
        let mut f = Surface::new(8, 1);
        f.root().text(0, 0, "ab", Style::new());
        f.root().text(2, 0, "cd", Style::new().bold());
        assert_eq!(style_changes(&bytes_for(&f)), 1);
        assert_eq!(replay(&f).cell(2, 0).style, Style::new().bold());
        assert_eq!(replay(&f).cell(1, 0).style, Style::DEFAULT);
    }

    #[test]
    fn one_style_covers_a_whole_run() {
        let mut f = Surface::new(300, 1);
        f.root()
            .fill(Rect::new(0, 0, 300, 1), "x", Style::new().bold());
        assert_eq!(
            style_changes(&bytes_for(&f)),
            1,
            "one SGR covers a run, however many cells it has"
        );
    }

    #[test]
    fn removing_bold_reapplies_dim() {
        // SGR 22 clears both, so the naive per-attribute loop drops dim here and nothing looks
        // wrong until a dim run silently turns normal.
        assert_eq!(
            sgr(Style::new().bold().dim(), Style::new().dim(), &modern()),
            "ESC[22;2m"
        );
    }

    #[test]
    fn removing_dim_reapplies_bold() {
        assert_eq!(
            sgr(Style::new().bold().dim(), Style::new().bold(), &modern()),
            "ESC[22;1m"
        );
    }

    #[test]
    fn adding_bold_alone_is_one_parameter() {
        assert_eq!(sgr(Style::new(), Style::new().bold(), &modern()), "ESC[1m");
    }

    #[test]
    fn sgr_21_is_never_emitted_for_a_double_underline() {
        assert_eq!(
            sgr(Style::new(), Style::new().underline_double(), &modern()),
            "ESC[4:2m"
        );
    }

    #[test]
    fn dropping_an_underline_emits_24() {
        assert_eq!(
            sgr(Style::new().underline_curly(), Style::new(), &modern()),
            "ESC[24m"
        );
    }

    #[test]
    fn the_low_sixteen_palette_entries_are_spelled_short() {
        let caps = modern();
        for (style, expected) in [
            (Style::new().fg(Color::indexed(3)), "ESC[33m"),
            (Style::new().fg(Color::indexed(9)), "ESC[91m"),
            (Style::new().bg(Color::indexed(1)), "ESC[41m"),
            (Style::new().bg(Color::indexed(15)), "ESC[107m"),
        ] {
            assert_eq!(sgr(Style::new(), style, &caps), expected);
        }
    }

    #[test]
    fn a_palette_entry_above_fifteen_needs_the_long_form() {
        assert_eq!(
            sgr(
                Style::new(),
                Style::new().fg(Color::indexed(200)),
                &modern()
            ),
            "ESC[38:5:200m"
        );
    }

    #[test]
    fn truecolor_is_three_channels() {
        assert_eq!(
            sgr(
                Style::new(),
                Style::new().bg(Color::rgb(1, 2, 255)),
                &modern()
            ),
            "ESC[48:2::1:2:255m",
            "T.416's empty colour-space id is between the 2 and the channels"
        );
        // **Both channels, and the foreground is the half that was wrong.** The parameterised
        // selector is 38/48/58 and the short-form base is 30/40/none, and `base + 8` is true of the
        // first two and false of the third — so a `Channel` carrying only the base emitted
        // `SGR 30:2::r:g:b` and every truecolor foreground arrived as palette entry zero. The round
        // trip caught it; neither of the two colour tests that existed did, because one used a
        // background and the other used the low sixteen.
        assert_eq!(
            sgr(
                Style::new(),
                Style::new().fg(Color::rgb(3, 4, 5)),
                &modern()
            ),
            "ESC[38:2::3:4:5m"
        );
    }

    #[test]
    fn returning_to_the_default_colour_emits_39_and_49() {
        assert_eq!(
            sgr(
                Style::new().fg(Color::rgb(1, 2, 3)).bg(Color::indexed(4)),
                Style::new(),
                &modern()
            ),
            "ESC[39;49m"
        );
    }

    #[test]
    fn an_extended_cell_is_painted_in_the_colours_its_table_entry_names() {
        // The defect this exists for: an extended word spends bits 51..0 on a handle, so reading
        // them as a 26-bit foreground and a 26-bit background emits whatever the handle happens to
        // look like. That is `Style::with_fg_bg`'s silent wrong answer one layer down, and it is a
        // *release* failure — the `debug_assert` on `Style::foreground` only catches the debug half.
        let mut frame = Surface::new(4, 1);
        frame.root().text(0, 0, "ab", Style::new());
        frame.root().restyle(
            Rect::new(0, 0, 2, 1),
            &crate::restyle::Restyle {
                fg: Some(Color::indexed(3)),
                bg: Some(Color::rgb(1, 2, 255)),
                link: Some(crate::restyle::Link::Uri("https://example.com/")),
                ..Default::default()
            },
        );
        let out = text(&bytes_for(&frame));
        assert_eq!(
            out,
            "ESC[0mESC[1;1HESC[33;48:2::1:2:255mESC]8;;https://example.com/ESC\\abESC]8;;ESC\\",
            "the entry's colours reached the wire, not its handle"
        );
    }

    /// A hyperlinked surface, and the one URI on it.
    fn hyperlinked(uri: &str) -> Surface {
        let mut frame = Surface::new(4, 1);
        frame.root().text(0, 0, "ab", Style::new());
        frame.root().restyle(
            Rect::new(0, 0, 2, 1),
            &crate::restyle::Restyle {
                link: Some(crate::restyle::Link::Uri(uri)),
                ..Default::default()
            },
        );
        frame
    }

    /// **A frame closes the hyperlink it opened, and SGR 0 is not what closes one.**
    ///
    /// An open OSC 8 survives a reset on a real terminal, so a frame that left one open would
    /// hyperlink whatever the next frame wrote beside it — including a cell whose style word says
    /// nothing about links, which is the failure that has no cell to blame.
    #[test]
    fn a_frame_closes_the_hyperlink_it_opened() {
        let out = text(&bytes_for(&hyperlinked("https://example.com/")));
        assert!(out.ends_with("ESC]8;;ESC\\"), "{out}");
        assert_eq!(
            out.matches("ESC]8;;").count(),
            2,
            "one open, one close: {out}"
        );
    }

    /// The degradation half, and it is a capability rather than a deferral now.
    ///
    /// Where the terminal has no OSC 8 a hyperlinked cell is painted in the right colours and loses
    /// its link. A handle emitted as a colour would not be degradation, which is what the test above
    /// this one exists for.
    #[test]
    fn a_terminal_without_osc8_gets_the_colours_and_not_the_link() {
        let frame = hyperlinked("https://example.com/");
        let mute = Capabilities::answering(ColorDepth::TrueColor, None, None);
        assert!(
            !mute.hyperlinks,
            "kitty answered nothing, so nothing is inferred"
        );
        let out = text(&bytes_with(&frame, &mute));
        assert!(!out.contains("]8;"), "{out}");
        assert!(out.contains("ESC[1;1H"), "the cells still went out: {out}");
    }

    /// **The round trip closes on a hyperlinked cell**, which is what ticket 13 is for: the model
    /// mints the URI back through the engine's own link table, so the replayed cell carries the
    /// frame's own extended-style handle rather than an equal-looking one.
    #[test]
    fn the_round_trip_closes_on_a_hyperlinked_cell() {
        let mut frame = hyperlinked("https://example.com/");
        assert!(
            frame.row(0)[0].style.ext_handle().is_some(),
            "the cell being compared has to be genuinely extended, or this closes on nothing"
        );
        let expected: Vec<Cell> = frame.row(0).to_vec();
        let bytes = bytes_for(&frame);
        let mut term = TermModel::new(4, 1);
        term.feed(&bytes, frame.tables_mut());
        assert_eq!(term.unrecognised(), 0);
        for x in 0..4 {
            assert_eq!(term.cell(x, 0), expected[x as usize], "column {x}");
        }
        assert_eq!(term.cell(0, 0).grapheme, GraphemeId::scalar('a'));
    }

    /// An underline colour round-trips the same way, and needs no capability to do it.
    #[test]
    fn the_round_trip_closes_on_an_underline_colour() {
        let mut frame = Surface::new(4, 1);
        frame.root().text(0, 0, "ab", Style::new());
        frame.root().restyle(
            Rect::new(0, 0, 2, 1),
            &crate::restyle::Restyle {
                ul: Some(Color::rgb(9, 8, 7)),
                ..Default::default()
            },
        );
        let bytes = bytes_for(&frame);
        let out = text(&bytes);
        assert!(out.contains("58:2::9:8:7"), "{out}");
        let expected = frame.row(0)[0];
        let mut term = TermModel::new(4, 1);
        term.feed(&bytes, frame.tables_mut());
        assert_eq!(term.unrecognised(), 0);
        assert_eq!(term.cell(0, 0), expected);
    }

    /// Dropping an underline colour is SGR 59, which is *the text's colour* and not a colour.
    #[test]
    fn dropping_an_underline_colour_is_fifty_nine() {
        // Built through the table, because that is the only way a style word carries one.
        let mut frame = Surface::new(4, 1);
        frame.root().text(0, 0, "ab", Style::new());
        frame.root().restyle(
            Rect::new(0, 0, 1, 1),
            &crate::restyle::Restyle {
                ul: Some(Color::rgb(9, 8, 7)),
                ..Default::default()
            },
        );
        let with = frame.row(0)[0].style;
        let mut packet = Packet::new();
        let mut runs = Vec::new();
        frame.damage().for_each_run(|r| runs.push(r));
        packet.pack_cells(&runs, &frame, frame.tables(), false, 1);
        let mut out = Vec::new();
        emit_sgr_delta(
            &mut out,
            with,
            Style::new(),
            &packet,
            &modern(),
            Quantiser::transparent(),
        );
        assert_eq!(text(&out), "ESC[59m");
    }

    /// ConPTY parses only the semicolon form of SGR 58, which is `legacy_sgr` one parameter along.
    #[test]
    fn conpty_gets_its_own_underline_colour_spelling() {
        let mut frame = Surface::new(4, 1);
        frame.root().text(0, 0, "ab", Style::new());
        frame.root().restyle(
            Rect::new(0, 0, 2, 1),
            &crate::restyle::Restyle {
                ul: Some(Color::rgb(9, 8, 7)),
                ..Default::default()
            },
        );
        let conpty =
            Capabilities::on_the_wire(ColorDepth::TrueColor, false, Underlines::ConPty, true);
        let out = text(&bytes_with(&frame, &conpty));
        assert!(out.contains("58;2;9;8;7"), "{out}");
        assert!(!out.contains("58:"), "{out}");
    }

    /// The pre-ITU-T spelling of 38 and 48, which four quirk entries force.
    #[test]
    fn legacy_sgr_spells_a_colour_the_way_xterm_did() {
        let legacy =
            Capabilities::on_the_wire(ColorDepth::TrueColor, false, Underlines::Standard, true);
        assert_eq!(
            sgr(
                Style::new(),
                Style::new().bg(Color::rgb(1, 2, 255)),
                &legacy
            ),
            "ESC[48;2;1;2;255m"
        );
        assert_eq!(
            sgr(Style::new(), Style::new().fg(Color::indexed(200)), &legacy),
            "ESC[38;5;200m"
        );
    }

    /// A model that reads only the form we emit would agree with a wrong one, so it reads both.
    #[test]
    fn the_model_reads_a_colour_in_either_spelling() {
        for bytes in [
            &b"\x1b[48:2::1:2:255mx"[..],
            &b"\x1b[48;2;1;2;255mx"[..],
            &b"\x1b[48:2:1:2:255mx"[..],
        ] {
            let mut t = TermModel::new(4, 1);
            let mut tables = crate::tables::Tables::new();
            t.feed(bytes, &mut tables);
            assert_eq!(t.unrecognised(), 0, "{:?}", text(bytes));
            assert_eq!(
                t.cell(0, 0).style,
                Style::new().bg(Color::rgb(1, 2, 255)),
                "{:?}",
                text(bytes)
            );
        }
    }

    #[test]
    fn an_sgr_that_would_say_nothing_emits_nothing() {
        assert!(
            sgr(Style::new().bold(), Style::new().bold(), &modern()).is_empty(),
            "an empty CSI m is a reset, not a no-op"
        );
    }

    #[test]
    fn the_mirror_records_what_was_emitted() {
        let mut f = Surface::new(8, 2);
        f.root().text(3, 1, "z", Style::new().bold());
        let mut runs = Vec::new();
        f.damage().for_each_run(|r| runs.push(r));
        let mut packet = Packet::new();
        packet.pack_cells(&runs, &f, f.tables(), false, 1);
        let mut s = Serializer::new(8, 2);
        s.serialize(&packet, &modern());
        assert_eq!(s.mirror().cell(3, 1), f.row(1)[3]);
        assert_eq!(
            s.mirror().cell(0, 0),
            Cell::UNKNOWN,
            "nothing else was emitted, so nothing else is known — and `UNKNOWN` rather than a blank \
             is what stops the filter believing a cell nobody wrote"
        );
    }

    #[test]
    fn a_continuation_cell_emits_no_glyph() {
        // Planted by hand, because no verb can write one until ticket 06. What is pinned is only
        // that the second half of a pair is not printed; the cursor advance over a wide head is
        // ticket 06's and is deliberately not modelled here.
        let mut f = Surface::new(6, 1);
        f.root().text(0, 0, "ab", Style::new());
        f.row_mut(0)[1] = Cell::new(GraphemeId::CONTINUATION, Style::DEFAULT);
        assert_eq!(
            glyphs(&replay(&f), 0, 6),
            "a     ",
            "the `b` was replaced and never printed"
        );
    }

    #[test]
    fn a_run_beginning_on_a_continuation_still_places_the_rest_correctly() {
        // The one case spec §8 records the first version getting wrong: the column comes from the
        // index, so skipping the first cell of a run must not shift what follows.
        let mut f = Surface::new(6, 1);
        f.root().text(2, 0, "ab", Style::new());
        f.row_mut(0)[2] = Cell::new(GraphemeId::CONTINUATION, Style::new().bold());
        let term = replay(&f);
        assert_eq!(
            glyphs(&term, 0, 6),
            "   b  ",
            "skipping the first cell of a run must not shift what follows"
        );
        assert_eq!(term.cell(2, 0), Cell::BLANK);
    }

    // -----------------------------------------------------------------------------------------
    // The equality filter, and the gap merge that needs no threshold.
    // -----------------------------------------------------------------------------------------

    /// A serializer driven over more than one frame of one surface.
    ///
    /// Every test about the filter needs at least two frames and **the same serializer across
    /// them**: on the first, every row of the mirror is unknown and the filter is inert by design.
    /// [`bytes_for`] builds a fresh serializer per call and is therefore always measuring a birth
    /// frame, which is why it cannot be used here.
    struct Frames {
        serializer: Serializer,
        packet: Packet,
        caps: Capabilities,
        /// One packet across many frames, so the stamp has to move the way `Screen`'s does.
        generation: u64,
    }

    impl Frames {
        fn new(w: u16, h: u16) -> Frames {
            Frames {
                serializer: Serializer::new(w, h),
                packet: Packet::new(),
                caps: modern(),
                generation: 0,
            }
        }

        /// The next stamp, never reused. `Screen` counts packs; so does this.
        fn next_generation(&mut self) -> u64 {
            self.generation += 1;
            self.generation
        }

        fn with_filter(w: u16, h: u16, filter: Filter) -> Frames {
            let mut f = Frames::new(w, h);
            f.serializer.set_filter(filter);
            f
        }

        /// Pack and serialise whatever `frame` has damaged, then clear its damage the way `present`
        /// does. Everything the engine puts between those two steps is above this file.
        fn present(&mut self, frame: &mut Surface) -> Vec<u8> {
            let mut runs = Vec::new();
            frame.damage().for_each_run(|r| runs.push(r));
            let generation = self.next_generation();
            self.packet
                .pack_cells(&runs, frame, frame.tables(), false, generation);
            let out = self.serializer.serialize(&self.packet, &self.caps).to_vec();
            frame.damage_mut().clear();
            out
        }
    }

    /// **The one configuration that ships.**
    ///
    /// Everything else on [`Filter`] is the instrument §8's sweep is reproduced with, and a default
    /// that drifted would quietly make every byte count in this crate a measurement of something
    /// nobody chose. Spec §8's *the serializer has no knobs* is what this asserts.
    #[test]
    fn the_filter_that_ships_is_priced_in_bytes() {
        assert_eq!(Serializer::new(8, 2).filter, Filter::Bytes);
    }

    /// The filter, at its simplest: a frame that rewrites a row without changing it says nothing.
    ///
    /// **And it says nothing *at all*** — not twenty bytes of framing announcing that it has nothing
    /// to say. §8's *every frame that has anything to say opens with SGR 0*, read strictly.
    #[test]
    fn a_frame_that_changes_nothing_emits_no_bytes_at_all() {
        let mut frame = Surface::new(8, 2);
        let mut f = Frames::new(8, 2);
        frame.root().text(0, 0, "abcdefgh", Style::new());
        assert!(
            !f.present(&mut frame).is_empty(),
            "the birth frame writes the row: its mirror row is unknown"
        );
        frame.root().text(0, 0, "abcdefgh", Style::new());
        let second = f.present(&mut frame);
        assert!(
            second.is_empty(),
            "eight cells were damaged and none of them changed: {}",
            text(&second)
        );
    }

    /// **The defect the unknown state exists for, at its smallest.**
    ///
    /// A fresh mirror used to *say* blank, so a frame whose cell is a blank compared equal to it and
    /// was skipped — and the terminal, which nobody had cleared, went on showing whatever was there.
    /// `Cell::UNKNOWN` is what makes the comparison fail instead, and this is the whole of the resize
    /// hazard in four columns. `crate::engine::tests::a_resize_does_not_let_the_filter_trust_a_fresh_mirror`
    /// is the same fact through the round trip.
    #[test]
    fn a_blank_the_mirror_never_wrote_is_still_written() {
        let mut frame = Surface::new(4, 1);
        let mut f = Frames::new(4, 1);
        frame.root().fill(Rect::new(0, 0, 4, 1), " ", Style::new());
        let out = f.present(&mut frame);
        assert!(
            out.ends_with(b"    "),
            "four blanks nobody has written are four blanks to write: {}",
            text(&out)
        );
    }

    /// A cell is known only while the mirror has not been told to forget, which is
    /// [`Packet::repaint`] — the flag a renumbering sweep sets.
    ///
    /// The row question survives as a query over the cells, and this is what asks it: every column of
    /// the row emitted is a known row, and a `repaint` puts it back to none of them. §14's gate on the
    /// count is `crate::gates::a_renumbering_sweep_marks_every_mirror_row_unknown`.
    #[test]
    fn a_repaint_forgets_every_cell_and_the_next_frame_emits_them_all() {
        let mut frame = Surface::new(8, 1);
        let mut f = Frames::new(8, 1);
        frame.root().text(0, 0, "abcdefgh", Style::new());
        f.present(&mut frame);
        assert!(
            f.serializer.mirror().is_known(0),
            "one frame wrote all of it"
        );
        frame.root().text(0, 0, "abcdefgh", Style::new());
        assert!(
            f.present(&mut frame).is_empty(),
            "nothing changed, so nothing goes out"
        );

        frame.root().text(0, 0, "abcdefgh", Style::new());
        let mut runs = Vec::new();
        frame.damage().for_each_run(|r| runs.push(r));
        let generation = f.next_generation();
        f.packet
            .pack_cells(&runs, &frame, frame.tables(), true, generation);
        let after = f.serializer.serialize(&f.packet, &f.caps).to_vec();
        assert!(
            after.ends_with(b"abcdefgh"),
            "a repaint is every cell again: {}",
            text(&after)
        );
    }

    /// A row of eight, made known, then two of its cells changed with `gap` ASCII columns between.
    ///
    /// The fixture the gap merge is decided on: the two changed cells are damaged along with
    /// everything between them, so the columns in between are cells the filter skipped rather than
    /// cells the packet never carried.
    fn changed_ends(gap: usize) -> (Surface, Frames) {
        let width = gap as u16 + 2;
        let mut frame = Surface::new(width, 1);
        let mut f = Frames::new(width, 1);
        let settled: String = std::iter::repeat_n('.', width as usize).collect();
        frame.root().text(0, 0, &settled, Style::new());
        f.present(&mut frame);
        let changed = format!("X{}Y", ".".repeat(gap));
        frame.root().text(0, 0, &changed, Style::new());
        (frame, f)
    }

    /// The gap merge, in the direction it pays: two ASCII columns are cheaper than the move over
    /// them.
    ///
    /// Two bytes of `.` against a four-byte `CUF`, so the serializer repaints cells it knows are
    /// already right — which is §8's *the gap merge removes escapes at the cost of cells the terminal
    /// was going to parse as a run anyway.*
    #[test]
    fn a_cheap_gap_is_painted_through_rather_than_moved_over() {
        let (mut frame, mut f) = changed_ends(2);
        let merged = f.present(&mut frame);
        assert_eq!(
            moves(&merged),
            1,
            "one move for the frame: {}",
            text(&merged)
        );
        assert!(
            merged.ends_with(b"X..Y"),
            "the two unchanged columns went out between the two changed ones: {}",
            text(&merged)
        );

        let (mut frame, mut f) = changed_ends(2);
        f.serializer.set_filter(Filter::Strict);
        let strict = f.present(&mut frame);
        assert_eq!(
            moves(&strict),
            2,
            "strict pays for the move: {}",
            text(&strict)
        );
        assert!(
            merged.len() < strict.len(),
            "merged {} bytes, strict {}",
            merged.len(),
            strict.len()
        );
    }

    /// The same geometry, and the gap merge in the direction it refuses — **because a cell is not a
    /// byte, which is the whole of why there is no threshold.**
    ///
    /// Two columns of braille are six bytes against the same four-byte move, so the merge that paid
    /// above loses here. A six-cell threshold cannot tell the two fixtures apart: it is in the wrong
    /// unit, and the second half of this test executes that rather than quoting it — `Cells(6)` spends
    /// six bytes to save four on the very fixture the byte price refuses.
    #[test]
    fn a_gap_of_braille_is_moved_over_and_a_cell_threshold_cannot_tell() {
        let mut frame = Surface::new(4, 1);
        let mut f = Frames::new(4, 1);
        frame.root().text(0, 0, ".⠿⠿.", Style::new());
        f.present(&mut frame);
        frame.root().text(0, 0, "X⠿⠿Y", Style::new());
        let priced = f.present(&mut frame);
        assert_eq!(
            moves(&priced),
            2,
            "six bytes of braille lose to a four-byte move: {}",
            text(&priced)
        );

        let mut frame = Surface::new(4, 1);
        let mut f = Frames::with_filter(4, 1, Filter::Cells(6));
        frame.root().text(0, 0, ".⠿⠿.", Style::new());
        f.present(&mut frame);
        frame.root().text(0, 0, "X⠿⠿Y", Style::new());
        let counted = f.present(&mut frame);
        assert_eq!(moves(&counted), 1, "a cell count sees two cells and merges");
        assert!(
            counted.len() > priced.len(),
            "the threshold spent bytes to save an escape: {} against {}",
            counted.len(),
            priced.len()
        );
    }

    /// A 300-column row whose changed cells stand 1, 2, 3, … columns apart, everything else `filler`.
    ///
    /// **Triangular gaps are the whole design.** Every gap width from one column up appears exactly
    /// once, so a threshold that merges gaps of at most `n` columns merges exactly the first `n` of
    /// them and a sweep over `n` walks the space one gap at a time. One `text` call a frame, so the
    /// row is one run and every gap is a gap the filter opened rather than one damage left.
    fn triangular_gaps(filler: char, filter: Filter) -> usize {
        const WIDTH: u16 = 300;
        let mut changed = vec![0usize];
        let mut gap = 1usize;
        while changed.last().expect("seeded") + gap + 1 < WIDTH as usize {
            changed.push(changed.last().expect("seeded") + gap + 1);
            gap += 1;
        }
        let settled: String = std::iter::repeat_n(filler, WIDTH as usize).collect();
        let mut frame = Surface::new(WIDTH, 1);
        let mut f = Frames::with_filter(WIDTH, 1, filter);
        frame.root().text(0, 0, &settled, Style::new());
        f.present(&mut frame);
        let mut next: Vec<char> = settled.chars().collect();
        for &at in &changed {
            next[at] = 'X';
        }
        let next: String = next.into_iter().collect();
        frame.root().text(0, 0, &next, Style::new());
        f.present(&mut frame).len()
    }

    /// **§8's sweep, and its conclusion: there is no threshold.**
    ///
    /// > A sweep shows it: the chart goes 4 581 → 4 735 → 5 135 → 5 276 → 5 507 → 5 909 → 7 611 as
    /// > the threshold climbs 0 → 24, monotonically worse, while the dialogs go 1 491 → 1 203 and then
    /// > flat. **The two scenes want opposite thresholds, so there is no threshold.**
    ///
    /// The same shape, over one geometry in two glyph widths, because **§14's twelve cannot produce
    /// it and that is worth knowing rather than working around.** `crate::gates`'s sweep over the
    /// twelve is flat past four columns on every one of them: the scenes that filter at all are label
    /// rows where a counter changes, so their gaps are a handful of ASCII columns and every threshold
    /// above four behaves identically. §8's chart is braille and ours plots `*`; §8's dialogs have a
    /// live status bar and ours rewrite every cell of all three every frame. Neither is a defect in
    /// the scene — both are §14's list as it was settled — but a conclusion about the *unit* a
    /// threshold is in cannot be drawn from scenes whose gaps are all one byte wide.
    ///
    /// So the fixture is a row whose gaps are one, two, three, … columns wide, filled once with a
    /// one-byte cluster and once with a three-byte one. The two sweeps disagree about where the
    /// optimum is, and the byte price finds both without being told either.
    #[test]
    fn a_fixed_gap_threshold_is_in_the_wrong_unit_and_the_two_fillers_want_opposite_ones() {
        const THRESHOLDS: [u16; 7] = [0, 4, 8, 12, 16, 20, 24];
        let mut best = Vec::new();
        for (what, filler) in [("one-byte cluster", '.'), ("three-byte cluster", '⠿')] {
            let swept: Vec<usize> = THRESHOLDS
                .iter()
                .map(|&n| triangular_gaps(filler, Filter::Cells(n)))
                .collect();
            let priced = triangular_gaps(filler, Filter::Bytes);
            let at = swept
                .iter()
                .enumerate()
                .min_by_key(|(_, b)| **b)
                .map(|(i, _)| i)
                .expect("seven points");
            println!(
                "\n  a row of triangular gaps in a {what}, thresholds {THRESHOLDS:?}:\n  \
                 {swept:?}\n  \
                 cheapest threshold {} columns; priced in bytes {priced}",
                THRESHOLDS[at]
            );
            assert!(
                priced <= swept[at],
                "pricing each gap on its own can never lose to the best single threshold: \
                 {priced} against {}",
                swept[at]
            );
            best.push(THRESHOLDS[at]);
        }
        assert_ne!(
            best[0], best[1],
            "the two fillers want the same threshold, so this fixture no longer says why there \
             is none: {best:?}"
        );
        assert_eq!(
            best[1], 0,
            "a three-byte cluster is never worth painting through: every gap costs three times \
             the columns and a move costs three or four bytes whatever the distance"
        );
    }

    /// **The second win the same mirror pays for**: two runs on one row, bridged through columns the
    /// packet never carried.
    ///
    /// §6 produces genuinely separate runs on a row and the gap between them is not in the packet —
    /// but it *is* in the mirror, so it can be repainted out of it and priced by the same rule. This
    /// is where §8's three dialogs go from 1 491 bytes to 1 203.
    ///
    /// The assertion that says the bytes came from the mirror is the `abc` in the middle: `b` and `c`
    /// are columns this frame did not damage, so nothing in the packet holds them.
    #[test]
    fn two_runs_on_one_row_are_bridged_out_of_the_mirror() {
        let mut frame = Surface::new(8, 1);
        let mut f = Frames::new(8, 1);
        frame.root().text(0, 0, "abcdefgh", Style::new());
        f.present(&mut frame);
        frame.root().text(0, 0, "X", Style::new());
        frame.root().text(3, 0, "Y", Style::new());
        let bridged = f.present(&mut frame);
        assert_eq!(moves(&bridged), 1, "one move, not two: {}", text(&bridged));
        assert!(
            bridged.ends_with(b"XbcY"),
            "the untouched columns were repainted out of the mirror: {}",
            text(&bridged)
        );
    }

    /// The bridge's first condition: **the mirror has to know the column.**
    ///
    /// The same two runs, over a screen where the columns between them have never been written. There
    /// is nothing to skip, because nothing was compared equal — and nothing to bridge either, because
    /// what the mirror says about a cell it never wrote is what it says about a screen it never
    /// recorded.
    #[test]
    fn a_gap_the_mirror_has_never_written_is_never_bridged() {
        let mut frame = Surface::new(8, 1);
        let mut f = Frames::new(8, 1);
        frame.root().text(0, 0, "a", Style::new());
        frame.root().text(3, 0, "d", Style::new());
        f.present(&mut frame);
        frame.root().text(0, 0, "X", Style::new());
        frame.root().text(3, 0, "Y", Style::new());
        let out = f.present(&mut frame);
        assert_eq!(moves(&out), 2, "two runs, two moves: {}", text(&out));
        assert!(
            !out.contains(&b'b') && !out.contains(&b'c'),
            "an unknown row's mirror is not a fact about the terminal: {}",
            text(&out)
        );
    }

    /// The bridge's second condition, and **the one way this optimisation could put something on
    /// screen that no frame asked for.**
    ///
    /// A mirror cell records what was emitted, and what was emitted may name a cluster that *this*
    /// packet's side tables do not carry: a frame resolves only the handles its own damaged cells
    /// name. Sent anyway it would go out as a space — `emit_grapheme` falls back to one rather than
    /// inventing — and the mirror would then record a cell the screen does not show.
    ///
    /// Three fixtures, and the middle one is what makes the third's refusal mean something. All three
    /// have the same one-column gap and the same five-byte move, refused `CUF` included, because the
    /// changed cell before the gap is non-ASCII (§10). The gap is one byte, then three, then three
    /// again — so the third is refused for its **handle** and not for its price.
    #[test]
    fn a_bridge_is_refused_where_the_packet_cannot_resolve_the_mirrors_handle() {
        /// A twelve-column row is written whole to make it known, then columns 0 and 2 are changed.
        /// Column 1 is the gap: one column, held only by the mirror, whose content is `gap`.
        fn bridged(gap: &str) -> (usize, String) {
            let mut frame = Surface::new(12, 1);
            let mut f = Frames::new(12, 1);
            let settled = format!(".{gap}{}", ".".repeat(10));
            frame.root().text(0, 0, &settled, Style::new());
            f.present(&mut frame);
            // Non-ASCII, so §10's rule refuses `CUF` for the rest of the row and the move the gap is
            // priced against is a four-byte `CHA` rather than a three-byte `CUF`. A **scalar** rather
            // than a cluster, so that this packet carries no cluster at all and the third fixture's
            // refusal cannot be confused with the packet happening to hold its handle.
            frame.root().text(0, 0, "⠿", Style::new());
            frame.root().text(2, 0, "Z", Style::new());
            let out = f.present(&mut frame);
            (moves(&out), text(&out))
        }

        let (ascii, bytes) = bridged(".");
        assert_eq!(ascii, 1, "one ASCII byte beats the four-byte move: {bytes}");

        let (scalar, bytes) = bridged("⠿");
        assert_eq!(
            scalar, 1,
            "three bytes still beat it, and a scalar handle *is* its own bytes: {bytes}"
        );

        let (cluster, bytes) = bridged("a\u{300}");
        assert_eq!(
            cluster, 2,
            "the same three bytes, in a handle this packet does not carry: {bytes}"
        );
    }

    /// The same refusal, on the **other three channels a cell can name.**
    ///
    /// An extended word spends its low fifty-two bits on a handle into a side table, so a mirror cell
    /// carrying one is unpaintable for exactly the same reason a cluster is — and for a worse
    /// consequence, because `channels_of` answers a handle it cannot resolve with the terminal's own
    /// colours and no hyperlink. That is a cell that goes out looking plain while the mirror records it
    /// extended, which is a disagreement no later frame repairs.
    ///
    /// The pair is the point: the same geometry, the same one-byte gap, and the only difference is
    /// whether the column between the two runs was ever restyled.
    #[test]
    fn a_bridge_is_refused_where_the_packet_cannot_resolve_the_mirrors_extended_style() {
        /// The gap is one column, at 98 of a 300-column row. **The column number is the fixture.** An
        /// extended gap cell costs its one byte plus `SGR_FLOOR`, so the move it is priced against has
        /// to cost more than five bytes for the handle to be the only thing left to refuse it — and a
        /// `CHA` past column ninety-nine is the first one that does, at six.
        fn bridged(extend: bool) -> (usize, String) {
            let mut frame = Surface::new(300, 1);
            let mut f = Frames::new(300, 1);
            frame.root().text(0, 0, &".".repeat(300), Style::new());
            if extend {
                frame.root().restyle(
                    Rect::new(98, 0, 1, 1),
                    &crate::restyle::Restyle {
                        ul: Some(Color::rgb(1, 2, 3)),
                        ..Default::default()
                    },
                );
            }
            f.present(&mut frame);
            // Non-ASCII first, so §10's rule refuses `CUF` and the move is the absolute form.
            frame.root().text(97, 0, "⠿", Style::new());
            frame.root().text(99, 0, "Z", Style::new());
            let out = f.present(&mut frame);
            (moves(&out), text(&out))
        }

        let (inline, bytes) = bridged(false);
        assert_eq!(
            inline, 1,
            "one inline byte beats the six-byte move: {bytes}"
        );

        let (extended, bytes) = bridged(true);
        assert_eq!(
            extended, 2,
            "the same byte, in a style word whose handle this packet does not carry: {bytes}"
        );
    }

    /// A gap inside a run is never refused for its handle, and this is why: its cells are damaged, so
    /// the packet resolved them at pack time whatever they name.
    ///
    /// Two runs would refuse this row; one run merges it. That difference is the whole of the fence
    /// above, and stating it as a test is what keeps somebody from "simplifying" the two conditions
    /// into one.
    #[test]
    fn a_gap_inside_a_run_carries_its_own_handles() {
        let mut frame = Surface::new(3, 1);
        let mut f = Frames::new(3, 1);
        frame.root().text(0, 0, ".a\u{300}.", Style::new());
        f.present(&mut frame);
        // One verb, so one run: the cluster in the middle is damaged and therefore in the packet. The
        // first column is non-ASCII so that §10's rule refuses `CUF` and the move the gap is priced
        // against is a four-byte `CHA`, which is the same geometry the refusal test uses.
        frame.root().text(0, 0, "⠿a\u{300}Y", Style::new());
        let out = f.present(&mut frame);
        assert_eq!(
            moves(&out),
            1,
            "three bytes beat a four-byte move: {}",
            text(&out)
        );
    }

    /// The filter is exact, and this is the property that makes it so: a skipped cell is skipped on a
    /// **whole-cell** comparison, style word and handle included.
    ///
    /// A frame that changes only the style of a cell whose glyph is unchanged must still emit it.
    /// Comparing glyphs alone would pass every test in this file that looks at a picture, and put the
    /// wrong colours on the screen.
    #[test]
    fn a_style_change_alone_is_a_change() {
        let mut frame = Surface::new(4, 1);
        let mut f = Frames::new(4, 1);
        frame.root().text(0, 0, "abcd", Style::new());
        f.present(&mut frame);
        frame.root().text(0, 0, "abcd", Style::new().bold());
        let out = f.present(&mut frame);
        assert_eq!(style_changes(&out), 1, "{}", text(&out));
        assert!(out.ends_with(b"abcd"), "{}", text(&out));
    }

    // -----------------------------------------------------------------------------------------
    // `shortest`, and the one rule in it that is correctness rather than byte count.
    // -----------------------------------------------------------------------------------------

    /// Every encoding in §8's set, each on the frame that makes it the cheapest.
    ///
    /// **The costs are asserted, not the spellings' presence**, because the property is *fewest
    /// bytes* and a test that only checked which letter appeared would pass a `CUF` that cost more
    /// than the `CUP` it replaced. Each row is one move, so the cursor byte count is the frame's
    /// length minus the reset and the glyph.
    #[test]
    fn each_encoding_is_used_where_it_is_the_cheapest() {
        // Same row, one column along: `CUF` with the parameter left out, three bytes.
        let mut f = Surface::new(20, 3);
        f.root().text(0, 0, "a", Style::new());
        f.root().text(2, 0, "b", Style::new());
        let out = text(&bytes_for(&f));
        assert_eq!(out, "ESC[0mESC[1;1HaESC[Cb", "{out}");

        // A genuine tie, and **a tie goes to the absolute encoding.** From column one to column
        // twenty is a distance of 19 and a target of 21, both two digits, so `CUF` and `CHA` cost
        // five bytes each. Preferring `CHA` is not arbitrary: §10's rule exists because a relative
        // move compounds an error, so where the price is equal the encoding that cannot compound is
        // the one taken.
        let mut f = Surface::new(40, 1);
        f.root().text(0, 0, "a", Style::new());
        f.root().text(20, 0, "b", Style::new());
        let out = text(&bytes_for(&f));
        assert!(out.contains("ESC[21G"), "{out}");

        // And where `CUF` is strictly cheaper it wins: two columns along from column eight is four
        // bytes relative against five absolute.
        let mut f = Surface::new(20, 1);
        f.root().text(6, 0, "ab", Style::new());
        f.root().text(10, 0, "cd", Style::new());
        let out = text(&bytes_for(&f));
        assert!(out.contains("ESC[2C"), "{out}");

        // Down the left margin: `CR` and one `LF` per row, two one-byte controls against a
        // six-byte `CUP`.
        let mut f = Surface::new(20, 4);
        f.root().text(1, 0, "a", Style::new());
        f.root().text(0, 1, "b", Style::new());
        let out = text(&bytes_for(&f));
        assert!(out.contains("a\r\nb"), "{out}");

        // Three rows down: the same `CR` and one more byte a row, where a `CUP` would cost six
        // however far it went.
        let mut f = Surface::new(20, 4);
        f.root().text(0, 0, "a", Style::new());
        f.root().text(0, 3, "b", Style::new());
        let out = text(&bytes_for(&f));
        assert!(out.contains("a\r\n\n\nb"), "{out}");
    }

    /// The frame's cursor cost, priced, against the encoding ticket 03 shipped.
    ///
    /// §8 puts `shortest` at 0.2% to 15% and says the win is not the encoding but the *choice*. This
    /// is that as a byte count on the shape the win comes from: many short runs on one row, which is
    /// the sub-cell chart's shape.
    #[test]
    fn shortest_beats_cup_only_on_a_row_of_short_runs() {
        let mut f = Surface::new(300, 1);
        for x in (0..300).step_by(3) {
            f.root().text(x, 0, "x", Style::new());
        }
        let bytes = bytes_for(&f).len();
        // What ticket 03's `CUP`-only loop would have spent: four fixed bytes plus the digits of
        // both coordinates, per run, plus one glyph, plus the four-byte reset.
        let cup_only = 4
            + (0..300)
                .step_by(3)
                .map(|x| 4 + 1 + digits(x + 1) + 1)
                .sum::<usize>();
        assert!(
            bytes < cup_only,
            "shortest spent {bytes} where CUP-only spends {cup_only}"
        );
        // And the picture is still right, which is the half a byte count cannot say.
        assert_eq!(replay(&f).cell(297, 0).grapheme, GraphemeId::scalar('x'));
    }

    /// **Spec §10's rule.** After a run containing a non-ASCII cluster, the next move on that row is
    /// absolute — and the frame it is about is a frame where `CUF` would otherwise have won.
    #[test]
    fn a_non_ascii_run_forces_the_next_move_on_its_row_to_be_absolute() {
        let ascii = {
            let mut f = Surface::new(20, 1);
            f.root().text(6, 0, "ab", Style::new());
            f.root().text(10, 0, "cd", Style::new());
            bytes_for(&f)
        };
        assert!(
            csi_finals(&ascii).any(|b| b == b'C'),
            "CUF is strictly cheaper here, so the rule has something to refuse: {}",
            text(&ascii)
        );

        let mut f = Surface::new(20, 1);
        f.root().text(6, 0, "a\u{e9}", Style::new());
        f.root().text(10, 0, "cd", Style::new());
        let out = bytes_for(&f);
        assert!(
            csi_finals(&out).all(|b| b != b'C'),
            "no CUF after a non-ASCII cluster: {}",
            text(&out)
        );
        assert!(
            text(&out).contains("ESC[11G"),
            "the move is CHA instead: {}",
            text(&out)
        );
    }

    /// And the flag is about a *row*: the next row starts trusting `CUF` again.
    #[test]
    fn the_rule_is_cleared_by_moving_to_another_row() {
        let mut f = Surface::new(20, 2);
        f.root().text(0, 0, "\u{e9}", Style::new());
        f.root().text(6, 1, "ab", Style::new());
        f.root().text(10, 1, "cd", Style::new());
        let out = text(&bytes_for(&f));
        assert!(
            out.contains("ESC[2C"),
            "row 1 has emitted nothing wide: {out}"
        );
    }

    /// **The corruption a width disagreement causes stays bounded to one cell instead of
    /// compounding**, which is the whole reason the rule is correctness rather than byte count.
    ///
    /// The counterfactual is written as bytes rather than as a second serializer configuration,
    /// because this crate ships one: the two streams below say the same thing, one absolutely and one
    /// relatively, and only the relative one moves the third run.
    #[test]
    fn a_width_disagreement_does_not_compound_along_the_row() {
        // **Two** wide clusters in separate runs, then an ASCII one. Two is the smallest number
        // that can tell *bounded* from *compounding* apart: with one, a relative move is one column
        // out and stays one column out, and the fixture would report the rule working whether it
        // was there or not.
        let mut f = Surface::new(20, 1);
        f.root().text(0, 0, "漢", Style::new());
        f.root().text(4, 0, "漢", Style::new());
        f.root().text(8, 0, "cd", Style::new());
        let bytes = bytes_for(&f);
        assert!(
            csi_finals(&bytes).all(|b| b != b'C'),
            "the serializer chose absolute moves: {}",
            text(&bytes)
        );

        // Replayed on a terminal that gives 漢 one column: what is wrong is the column each pair's
        // continuation should have covered, and nothing else. **Every run still starts where it
        // belongs**, which is the whole of what the rule buys.
        let mut narrow = TermModel::disagreeing_about_width(20, 1);
        narrow.feed(&bytes, f.tables_mut());
        assert_eq!(
            narrow.cell(0, 0).grapheme.columns(),
            2,
            "our tables' opinion"
        );
        assert_eq!(narrow.cell(4, 0).grapheme.columns(), 2);
        assert_eq!(narrow.cell(8, 0).grapheme, GraphemeId::scalar('c'));
        assert_eq!(narrow.cell(9, 0).grapheme, GraphemeId::scalar('d'));

        // The same picture with every move spelled relatively, which is what the rule refuses: two
        // columns from the cursor after the first cluster, two more after the second. The first run
        // lands one column early and the last lands **two**, and on a longer row it keeps going —
        // that is *compounds*, and a mirror is what makes it permanent, because the mirror records
        // what was intended and an overpainted neighbour is never re-emitted.
        let mut compounding = TermModel::disagreeing_about_width(20, 1);
        let mut tables = crate::tables::Tables::new();
        compounding.feed(
            "\x1b[0m\x1b[1;1H漢\x1b[2C漢\x1b[2Ccd".as_bytes(),
            &mut tables,
        );
        assert_eq!(
            compounding.cell(3, 0).grapheme.columns(),
            2,
            "one column early"
        );
        assert_eq!(
            compounding.cell(6, 0).grapheme,
            GraphemeId::scalar('c'),
            "and this one is two columns early, not one"
        );
    }

    /// **Spec §15's second owed measurement, paid.** What the `CHA`-after-non-ASCII rule costs in
    /// bytes, against the 848 bytes `shortest` won on the chart.
    ///
    /// A report rather than a gate — the numbers belong to the fixtures, not to the mechanism — and
    /// three rows rather than one, because a single number here would have been either meaningless or
    /// alarming depending on which fixture produced it.
    ///
    /// **The answer is that it costs nothing on anything §14 measures, and up to a third of the frame
    /// on a shape §14 does not have.** The rule only ever charges for a move that is *along a row the
    /// serializer has already put a non-ASCII cluster on*, and that needs two things at once: several
    /// runs on one row, and non-ASCII inside them. §14's twelve have the first — the chart is 113
    /// moves — and none of them has the second, because the chart plots with `*`. A full screen of
    /// CJK has the second and not the first: its runs are whole rows, so every move is a row change
    /// and `CUF` was never a candidate.
    ///
    /// So the 848 bytes `shortest` won on the chart stand undiminished, and the ceiling is a shape
    /// that is real but is not on the list: a chart drawn in braille or a table of CJK columns. **The
    /// last row is what a component library has to know**, and it is the reason this is reported per
    /// fixture rather than summed.
    #[test]
    fn what_the_cha_rule_costs_in_bytes() {
        /// One fixture's bytes and what the rule charged for them.
        fn priced(f: &Surface) -> (usize, usize) {
            let mut runs = Vec::new();
            f.damage().for_each_run(|r| runs.push(r));
            let mut packet = Packet::new();
            packet.pack_cells(&runs, f, f.tables(), false, 1);
            let (w, h) = f.size();
            let mut s = Serializer::new(w, h);
            let bytes = s.serialize(&packet, &modern()).len();
            (bytes, s.cha_rule_bytes())
        }

        // §14's twelve, through the screens they are actually measured on.
        let mut worst = ("", 0usize, 0usize);
        for mut scene in crate::scenes::scenes() {
            let mut harness = crate::testing::Harness::with_overrides(
                crate::scenes::W,
                crate::scenes::H,
                scene.overrides(),
            );
            scene.build(&mut harness.screen);
            harness.present();
            for t in 1..=3 {
                scene.step(&mut harness.screen, t);
                harness.present();
            }
            let cost = harness.screen.cha_rule_bytes();
            if cost >= worst.1 {
                worst = (scene.name(), cost, harness.bytes_written());
            }
        }

        // A full screen of mixed CJK, whole rows: the second ingredient without the first.
        let mut cjk = Surface::new(300, 80);
        for y in 0..80 {
            let row = crate::testing::bisecting_cjk::row(y, 300);
            cjk.root().text(0, y as i32, &row, Style::new());
        }
        let (cjk_bytes, cjk_cost) = priced(&cjk);

        // Both ingredients: the chart's shape with a wide cluster in place of the `*`. Not on §14's
        // list, and entirely constructible by a component — a braille chart is this.
        let mut dense = Surface::new(300, 80);
        for y in 0..80 {
            for x in (0..300).step_by(3) {
                dense.root().text(x, y, "漢", Style::new());
            }
        }
        let (dense_bytes, dense_cost) = priced(&dense);

        println!(
            "\nwhat the CHA-after-non-ASCII rule costs, spec §15's second owed measurement:\n  \
             §14's twelve, worst row ({}) {} bytes of {}\n  \
             a full screen of mixed CJK, whole rows          {cjk_cost} bytes of {cjk_bytes}\n  \
             a chart's runs drawn in a wide cluster          {dense_cost} bytes of {dense_bytes} \
             ({:.1}%)\n  \
             The rule charges only for a move along a row that has already emitted a non-ASCII \
             cluster, so it\n  \
             needs several runs on one row *and* non-ASCII inside them. §14's twelve have the first \
             and not\n  \
             the second — the chart plots with `*` — and a screen of CJK has the second and not the \
             first,\n  \
             because whole-row runs make every move a row change and `CUF` was never a candidate. \
             **So the\n  \
             848 bytes `shortest` won on the chart are not paid back**, and the third row is the \
             ceiling: a\n  \
             shape that is real, that a component library will write, and that is not on the list. \
             Report, not\n  \
             a gate.",
            worst.0,
            worst.1,
            worst.2,
            100.0 * dense_cost as f64 / dense_bytes as f64,
        );
        assert_eq!(
            worst.1, 0,
            "a scene on §14's list started paying for the rule"
        );
        assert!(
            dense_cost > 0,
            "the ceiling fixture has to exercise the rule to price it"
        );
    }

    // -----------------------------------------------------------------------------------------
    // The frame's framing, and the session's.
    // -----------------------------------------------------------------------------------------

    /// §8's twenty bytes of fixed framing, on the frame that is the reason they are counted.
    #[test]
    fn the_frame_is_wrapped_in_mode_2026_where_the_terminal_has_it() {
        // §8's caret frame: an eight-byte `CUP` and one ASCII cell is the nine bytes of *the
        // caret's own change*, and the framing is the other twenty.
        let mut f = Surface::new(80, 24);
        f.root().text(40, 12, "x", Style::new());
        let syncing =
            Capabilities::on_the_wire(ColorDepth::TrueColor, true, Underlines::Standard, false);
        let out = bytes_with(&f, &syncing);
        assert!(out.starts_with(b"\x1b[?2026h\x1b[0m"), "{}", text(&out));
        assert!(out.ends_with(b"\x1b[?2026l"), "{}", text(&out));
        // `?2026h` + `0m` + `?2026l` is twenty bytes, and the caret's own change is the rest.
        assert_eq!(SYNC_BEGIN.len() + 4 + SYNC_END.len(), 20);
        assert_eq!(
            out.len(),
            29,
            "§8's 29-byte caret frame, which no test could construct before impl 13: {}",
            text(&out)
        );

        // And the model agrees it was one block, which is §8's *the frame is never split on
        // purpose* read from the other end.
        let mut term = TermModel::new(80, 24);
        let mut tables = crate::tables::Tables::new();
        term.feed(&out, &mut tables);
        assert_eq!(term.unrecognised(), 0);
        assert_eq!(term.sync_blocks(), 1);
    }

    /// A terminal without mode 2026 gets no framing at all, rather than framing it will ignore.
    #[test]
    fn a_terminal_without_mode_2026_is_sent_none() {
        let mut f = Surface::new(8, 1);
        f.root().text(0, 0, "a", Style::new());
        assert!(!text(&bytes_for(&f)).contains("2026"));
    }

    // -----------------------------------------------------------------------------------------
    // Register #12, over the configurations the terminal model can express.
    // -----------------------------------------------------------------------------------------

    /// **Gate, equality (register #12): the round trip stays green under every wire configuration.**
    ///
    /// There are four axes below the packet that change the bytes without changing the picture — the
    /// two SGR spellings, the two underline-colour spellings, mode 2026 on or off, and OSC 8 on or
    /// off — and the property is that none of them changes the *screen*. Sixteen combinations, one
    /// frame, and the frame is chosen to touch every branch there is: a wide cluster, a combining
    /// one, a palette colour under sixteen, one above it, a truecolor pair, an underline colour and a
    /// hyperlink.
    ///
    /// Two of the axes are declarable and two are not, and that is why this gate is here rather than
    /// in `crate::gates`: `sync_output` and `underlines` have no `Overrides` field — nobody at the
    /// terminal can name mode 2026 or ConPTY's underline-colour form — so their arms are reached from
    /// inside the crate through a synthetic `Detected` put through `assemble`
    /// (`Capabilities::on_the_wire`). A gate above `Screen` could not construct half of this table.
    ///
    /// The OSC 8 axis is the one asymmetry: where the terminal has no hyperlinks the replayed cell is
    /// *deliberately* not equal, because the link is dropped. So that arm asserts the degradation
    /// instead — same glyphs, same colours, no link — which is a statement about the same bytes and
    /// not a weaker one.
    #[test]
    fn the_round_trip_closes_under_every_wire_configuration() {
        /// The frame every arm replays: one of everything the encoding set can spell.
        fn frame() -> Surface {
            let mut f = Surface::new(24, 2);
            f.root()
                .text(0, 0, "漢a\u{301}b", Style::new().bold().dim());
            f.root().text(
                0,
                1,
                "cd",
                Style::new()
                    .fg(Color::indexed(3))
                    .bg(Color::indexed(200))
                    .underline_curly(),
            );
            f.root()
                .text(4, 1, "ef", Style::new().fg(Color::rgb(1, 2, 3)));
            f.root().restyle(
                Rect::new(0, 1, 2, 1),
                &crate::restyle::Restyle {
                    ul: Some(Color::rgb(9, 8, 7)),
                    link: Some(crate::restyle::Link::Uri("https://example.com/vitui#1")),
                    ..Default::default()
                },
            );
            f
        }

        /// The four channels a cell paints with, resolved through the surface's own tables.
        ///
        /// The **same** tables the terminal model minted into, which is what makes the two
        /// comparable: a channel the engine wrote resolves to the entry the engine made, and one it
        /// did not gets an entry nobody else has.
        fn resolve(cell: Cell, f: &Surface) -> crate::exts::ExtStyle {
            match cell.style.ext_handle() {
                Some(h) => f
                    .tables()
                    .exts
                    .get(h)
                    .expect("the handle came out of this table"),
                None => crate::exts::ExtStyle {
                    fg: cell.style.foreground(),
                    bg: cell.style.background(),
                    ul: Color::DEFAULT,
                    link: LinkId::NONE,
                },
            }
        }

        for legacy in [false, true] {
            for underlines in [Underlines::Standard, Underlines::ConPty] {
                for sync in [false, true] {
                    for hyperlinks in [false, true] {
                        let caps = Capabilities::on_the_wire(
                            ColorDepth::TrueColor,
                            sync,
                            underlines,
                            legacy,
                        );
                        // The one axis that is a field, applied on top of the synthetic terminal.
                        let caps = Capabilities { hyperlinks, ..caps };
                        let mut f = frame();
                        let expected: Vec<Cell> = (0..2).flat_map(|y| f.row(y).to_vec()).collect();
                        let bytes = bytes_with(&f, &caps);
                        let mut term = TermModel::new(24, 2);
                        term.feed(&bytes, f.tables_mut());
                        let what = format!(
                            "legacy {legacy}, {underlines:?}, sync {sync}, hyperlinks {hyperlinks}"
                        );
                        assert_eq!(term.unrecognised(), 0, "{what}: {}", text(&bytes));
                        assert_eq!(term.sync_blocks(), usize::from(sync), "{what}");

                        for y in 0..2u16 {
                            for x in 0..24u16 {
                                let want = expected[y as usize * 24 + x as usize];
                                let got = term.cell(x, y);
                                assert_eq!(got.grapheme, want.grapheme, "{what} at ({x}, {y})");
                                if hyperlinks {
                                    assert_eq!(got.style, want.style, "{what} at ({x}, {y})");
                                    continue;
                                }
                                // The degradation, asserted rather than skipped: **the link is the
                                // only thing that may differ.** The underline colour still arrives —
                                // SGR 58 needs no capability — so the cell is extended either way,
                                // and comparing attributes alone would let a dropped colour pass as
                                // a dropped link.
                                assert_eq!(
                                    got.style.attr_word(),
                                    want.style.attr_word(),
                                    "{what} at ({x}, {y})"
                                );
                                let (had, has) = (resolve(want, &f), resolve(got, &f));
                                assert_eq!(
                                    (had.fg, had.bg, had.ul),
                                    (has.fg, has.bg, has.ul),
                                    "{what} at ({x}, {y}): a colour was dropped, not a link"
                                );
                                assert!(has.link.is_none(), "{what} at ({x}, {y})");
                            }
                        }
                        if !hyperlinks {
                            assert!(!text(&bytes).contains("]8;"), "{what}: {}", text(&bytes));
                        }
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------------------------------
    // The quirk table on the wire: a flag the terminal does not render.
    // -----------------------------------------------------------------------------------------

    /// **Gate, equality (production ticket 10): the flag a terminal drops is never sent, in either
    /// direction.**
    ///
    /// §10 has always said an unsupported attribute is *dropped silently at serialise time*, and for
    /// four tickets nothing did it: `Quirks::apply` filled `attrs_dropped`, `Capabilities::report`
    /// printed it, and this file never asked. The gate is written against the **version string** and
    /// not against a mask, so it fails the day the tmux entry is deleted rather than passing over an
    /// empty table — see `Capabilities::identified_as`.
    ///
    /// **The off direction is half the property and the easier half to lose.** Masking only the new
    /// side leaves `SGR 55` going out to turn off a bit the terminal never had: bytes spent to undo
    /// nothing, and the next frame's diff computed against a style it was never in.
    #[test]
    fn the_flag_tmux_does_not_render_reaches_neither_side_of_the_wire() {
        let tmux = Capabilities::identified_as("tmux 3.7c");
        assert_eq!(
            tmux.attrs_dropped(),
            crate::style::OVERLINE,
            "the entry is what this gate is about; without it there is nothing to assert"
        );
        let all = Style::new()
            .bold()
            .dim()
            .italic()
            .reverse()
            .blink()
            .strikethrough()
            .conceal()
            .overline()
            .underline_double();

        // Every attribute the encoding set can spell, less the one tmux throws away — and the other
        // ten asserted by being *there*, because a mask that dropped more than it was observed to
        // drop is the failure `quirks.rs` exists to avoid.
        assert_eq!(
            sgr(Style::new(), all, &tmux),
            "ESC[1;2;3;5;7;8;9;4:2m",
            "53 is the only parameter that may be missing"
        );
        assert_eq!(
            sgr(all, Style::new(), &tmux),
            "ESC[22;23;25;27;28;29;24m",
            "and 55 does not turn off a bit the terminal never had"
        );

        // The same two words on a terminal with no entry, which is what says the gate above is about
        // the quirk table and not about the encoding.
        let plain =
            Capabilities::on_the_wire(ColorDepth::TrueColor, false, Underlines::Standard, false);
        assert_eq!(plain.attrs_dropped(), 0);
        assert_eq!(sgr(Style::new(), all, &plain), "ESC[1;2;3;5;7;8;9;53;4:2m");
        assert_eq!(
            sgr(all, Style::new(), &plain),
            "ESC[22;23;25;27;28;29;55;24m"
        );
    }

    /// **Gate, equality: a mask of more than one bit, which is the case the entry above never had.**
    ///
    /// tmux drops one attribute and kitty drops two, and *two* is not the same test: a mask applied
    /// with a `!=` where a `&` belonged, or a loop that stops at the first cleared bit, passes the
    /// one-bit gate and fails here. `quirks.rs`'s fifth entry is what made the case reachable, and
    /// the gate is against the **version string** so it fails the day that entry is deleted rather
    /// than passing over an empty table.
    ///
    /// The nine attributes kitty does render are asserted by being present, for the same reason the
    /// tmux gate asserts ten: a mask that drops more than it was observed to drop is the failure the
    /// quirk table exists to avoid, and it is invisible in a test that only checks the absences.
    #[test]
    fn the_two_flags_kitty_cannot_store_reach_neither_side_of_the_wire() {
        let kitty = Capabilities::identified_as("kitty(0.48.2)");
        assert_eq!(
            kitty.attrs_dropped(),
            crate::style::CONCEAL | crate::style::OVERLINE,
            "the entry is what this gate is about; without it there is nothing to assert"
        );
        let all = Style::new()
            .bold()
            .dim()
            .italic()
            .reverse()
            .blink()
            .strikethrough()
            .conceal()
            .overline()
            .underline_double();

        // 8 and 53 are the two that may be missing, and `4:2` is still there — the underline
        // enumeration is not a bit this mask could clear even if kitty's capture format could spell
        // the style it does lose. See `conform/FINDINGS.md`.
        assert_eq!(
            sgr(Style::new(), all, &kitty),
            "ESC[1;2;3;5;7;9;4:2m",
            "8 and 53 are the only parameters that may be missing"
        );
        assert_eq!(
            sgr(all, Style::new(), &kitty),
            "ESC[22;23;25;27;29;24m",
            "and neither 28 nor 55 turns off a bit the terminal never had"
        );
    }

    /// **Gate, equality (production ticket 10): the degradation, on a whole frame.**
    ///
    /// The SGR gate above pins the parameters; this one pins what the *terminal* ends up holding, and
    /// the two are not the same statement — an `emit_sgr_delta` that masked correctly while the mirror
    /// recorded the unmasked word would pass the first and leave the next frame's diff computed
    /// against a style the terminal was never in.
    ///
    /// The shape is `the_round_trip_closes_under_every_wire_configuration`'s OSC 8 arm: where the
    /// terminal cannot express something the replayed cell is *deliberately* not equal, so the arm
    /// asserts the **degradation** — same glyph, same colours, same ten other flags, no overline.
    #[test]
    fn a_frame_on_tmux_arrives_with_every_attribute_but_the_dropped_one() {
        let tmux = Capabilities::identified_as("tmux 3.7c");
        let all = Style::new()
            .bold()
            .italic()
            .conceal()
            .overline()
            .fg(Color::rgb(1, 2, 3));
        let mut frame = Surface::new(4, 1);
        frame.root().text(0, 0, "ab", all);
        let want = frame.row(0)[0];

        let bytes = bytes_with(&frame, &tmux);
        let mut term = TermModel::new(4, 1);
        term.feed(&bytes, frame.tables_mut());
        assert_eq!(term.unrecognised(), 0, "{}", text(&bytes));

        let got = term.cell(0, 0);
        assert_eq!(got.grapheme, want.grapheme);
        assert_eq!(
            got.style.attrs(),
            want.style.attrs() & !crate::style::OVERLINE,
            "overline and nothing else: {}",
            text(&bytes)
        );
        assert_eq!(
            got.style.foreground(),
            want.style.foreground(),
            "a colour was dropped, not an attribute"
        );
        assert!(
            want.style.attrs() & crate::style::OVERLINE != 0,
            "the frame has to be asking for the bit, or this closes on nothing"
        );
    }

    /// **Report: the worst case, and the tearing that is a known consequence rather than a bug.**
    ///
    /// §8's row is 805 657 bytes, 24 000 SGR sequences and 925 µs for a frame where every cell has a
    /// distinct style — 33.6 bytes a cell against 1.0 for a realistic one. The byte count and the SGR
    /// count are exact and machine-independent, which is why they are the numbers this prints; the
    /// microseconds are a build and a machine, so run it with `--release` before believing them.
    ///
    /// ```text
    /// cargo test --release -p vitui-engine the_worst_case_frame -- --nocapture
    /// ```
    ///
    /// The consequence is the point. **On a 4 MB/s link this frame is nearly twice Alacritty's 150 ms
    /// force-flush limit**, so a worst-case frame over a slow link *will* be force-flushed and *will*
    /// tear. Nothing mitigates it and nothing needs to: a screen where every cell has a unique colour
    /// is a test fixture. It is recorded so that the tearing is a known consequence and not a bug
    /// report — and the byte limit is emphatically **not** what binds: 805 KB is 0.38× of Alacritty's
    /// 2 MiB.
    #[test]
    fn the_worst_case_frame() {
        const W: u16 = 300;
        const H: u16 = 80;
        /// Bytes a second on the link §8 prices this against.
        const LINK_BYTES_PER_SEC: f64 = 4.0 * 1024.0 * 1024.0;
        /// Alacritty's, which is the shortest of the three §8 names.
        const FORCE_FLUSH_MS: f64 = 150.0;

        let mut f = Surface::new(W, H);
        for y in 0..H {
            for x in 0..W {
                // Every cell a distinct 24-bit **pair**, which is what makes every cell its own
                // style run and therefore its own SGR — and what makes the row 33.6 bytes a cell
                // rather than 17.5. A fixture that varied one channel is the same shape at half the
                // size, and half of the worst case is not the worst case.
                let n = y as u32 * W as u32 + x as u32;
                let rgb = |v: u32| Color::rgb((v >> 16) as u8, (v >> 8) as u8, v as u8);
                f.root().text(
                    x as i32,
                    y as i32,
                    "m",
                    Style::new().fg(rgb(n ^ 0xFF_FFFF)).bg(rgb(n)),
                );
            }
        }
        let mut runs = Vec::new();
        f.damage().for_each_run(|r| runs.push(r));
        let mut packet = Packet::new();
        packet.pack_cells(&runs, &f, f.tables(), false, 1);
        let mut s = Serializer::new(W, H);
        let caps = modern();
        let at = std::time::Instant::now();
        let bytes = s.serialize(&packet, &caps).len();
        let took = at.elapsed().as_secs_f64() * 1e6;

        let sgr = {
            let mut again = Serializer::new(W, H);
            let out = again.serialize(&packet, &caps).to_vec();
            csi_finals(&out).filter(|b| *b == b'm').count() - 1
        };
        let cells = u32::from(W) * u32::from(H);
        let transmit_ms = bytes as f64 / LINK_BYTES_PER_SEC * 1000.0;
        println!(
            "\nthe worst case, {W}x{H}, every cell a distinct style:\n  \
             {bytes} bytes, {sgr} SGR, {took:.0} us to serialise, {:.1} bytes a cell\n  \
             spec §8 measured 805 657 bytes, 24 000 SGR, 925 us and 33.6 bytes a cell\n  \
             on a 4 MB/s link that is {transmit_ms:.0} ms, against Alacritty's {FORCE_FLUSH_MS:.0} ms \
             force-flush limit — {:.1}x\n  \
             so this frame **will** be force-flushed and **will** tear. The byte limit is not what \
             binds: 2 MiB is {:.2}x this.\n  \
             A realistic full-screen frame is 1.0 bytes a cell. Report, not a gate.",
            bytes as f64 / cells as f64,
            transmit_ms / FORCE_FLUSH_MS,
            2.0 * 1024.0 * 1024.0 / bytes as f64,
        );
        assert_eq!(
            sgr, cells as usize,
            "one SGR a cell is what makes this the worst case"
        );
        // **The sentence above is asserted, not just printed.** It is a byte count against two
        // constants, which is the register's own favourite shape, and it exists so that a change
        // making this frame smaller cannot leave a report claiming a tear that no longer happens.
        assert!(
            transmit_ms > FORCE_FLUSH_MS,
            "{transmit_ms:.0} ms is inside the force-flush limit, so this frame no longer tears \
             and the paragraph above is no longer true"
        );
    }

    #[test]
    fn push_num_writes_decimal() {
        let mut out = Vec::new();
        for n in [0u32, 1, 9, 10, 255, 4294967295] {
            out.clear();
            push_num(&mut out, n);
            assert_eq!(text(&out), n.to_string());
        }
    }
}
