//! The primary instrument, and the sinks it writes into.
//!
//! A frame's bytes are the only thing `present` produces, so every gate on this lineage is a
//! statement about what reached one of these.
//!
//! [`Harness`] is the **round trip** itself (spec §8, §14): composite a frame, serialise it, replay
//! the bytes through the terminal model, assert the replayed screen equals the frame. It stores
//! nothing, so there is no file to review, nothing to bless and no maintenance — and a golden byte
//! string would have pinned the encoding, which is exactly the part tickets 13, 14 and 15 are going
//! to change. It lives here rather than in [`crate::roundtrip`] because
//! [`crate::gates`] drives spec §14's twelve scenes through the same instrument, and a second copy
//! of it was a second copy that could quietly assert less: the first draft of the scene gates
//! checked the replayed screen and forgot the mirror.

use std::io::{Error, ErrorKind, Result, Write};
use std::sync::{Arc, Mutex};

use crate::caps::Overrides;
use crate::engine::{Clock, Config, Engine, Output, Presented, Screen};
use crate::surface::Surface;
use crate::term_model::TermModel;

/// The invariant of spec §3, over a whole surface: a `CONTINUATION` never appears without a wide
/// head immediately to its left, and a wide head is always followed by a `CONTINUATION`.
///
/// **It is asserted of two different things and that is the point.** Ticket 06 established it over
/// one [`Surface`] — what a sequence of drawing verbs leaves behind — and ticket 11 establishes it
/// over the **composited frame**, which is where it has to hold, because the frame is what the
/// serializer reads. The round trip cannot see this class of defect: the serializer emits nothing
/// for a continuation and the terminal model consumes nothing for one, so a frame with a bare
/// continuation in it round-trips green while a real terminal would show something else. One
/// definition, so that neither caller can quietly assert less than the other.
///
/// The one place it is deliberately *not* asserted is a pair bisected by a
/// [`View::child`](crate::View::child) clip: a child may not widen its clip (spec §4), so the half
/// outside stays. That is architecture ticket 20's to decide and not this instrument's to hide.
pub(crate) fn assert_pairing_holds(s: &Surface) {
    let (w, h) = s.size();
    for y in 0..h {
        let row = s.row(y);
        for x in 0..w as usize {
            let g = row[x].grapheme;
            if g.is_continuation() {
                assert!(x > 0, "a continuation in column 0 at row {y}");
                assert!(
                    row[x - 1].grapheme.is_wide_head(),
                    "a continuation at ({x}, {y}) with no wide head to its left"
                );
            }
            if g.is_wide_head() {
                assert!(
                    x + 1 < w as usize,
                    "a wide head in the last column at row {y}"
                );
                assert!(
                    row[x + 1].grapheme.is_continuation(),
                    "a wide head at ({x}, {y}) with no continuation after it"
                );
            }
        }
    }
}

/// The twelve-bisecting-layers-over-CJK fixture: the rows, and the rectangles that cut them.
///
/// **Two callers and one definition, for the reason the second caller exists.**
/// [`crate::gates::the_pairing_invariant_survives_twelve_bisecting_layers_over_cjk`] drives it to
/// hold spec §3's pairing invariant over the composited frame, and [`crate::golden`] blesses the
/// picture it makes — *the one frame in this crate whose correctness no other instrument makes
/// visible to a human.* The round trip cannot see a frame whose halves do not pair, because the
/// serializer emits nothing for a continuation and the terminal model consumes nothing for one; the
/// reference compositor checks the picture against an oracle and produces nothing anybody reads.
///
/// The rects were duplicated into the golden at first, and that was the defect: change one of them
/// and the golden goes on passing against its own blessed file while quietly no longer being the
/// picture the gate exercises.
pub(crate) mod bisecting_cjk {
    /// How many rectangles bisect the screen of CJK.
    pub(crate) const BISECTORS: i32 = 12;

    /// One row of mixed CJK, offset so that pairs do not all start on the same parity.
    ///
    /// The offset is the point. Twelve rectangles at fixed columns over a screen where every pair
    /// began on an even column would bisect either all of them or none of them, and a repair that
    /// was right for one parity and wrong for the other would pass.
    pub(crate) fn row(y: u16, width: u16) -> String {
        let mut s = ".".repeat((y % 3) as usize);
        while crate::text::width_of(&s) < width {
            s.push_str("漢字ab漢c");
        }
        s
    }

    /// Rectangle `i` of [`BISECTORS`], over a screen `width` columns wide.
    ///
    /// Ten inside the frame, one hanging off the left edge and one off the right. The two edge
    /// rectangles are what exercise the clamp: their content is clipped mid-pair by the frame rather
    /// than by anything the layer stack decided.
    pub(crate) fn rect(i: i32, width: u16) -> crate::geom::Rect {
        match i {
            0 => crate::geom::Rect::new(-3, 40, 24, 9),
            1 => crate::geom::Rect::new(width as i32 - 9, 52, 24, 9),
            _ => crate::geom::Rect::new(5 + i * 22, i * 6, 31, 9),
        }
    }
}

/// The two terminals the compositor is driven against, and why there are exactly two.
///
/// Spec §5's colour resolution depends on an answer from the other end (ADR 0025), so a test about
/// an operator has to say which terminal it is on. One definition of each, because the *silent* one
/// is the default state of every headless test in this crate and a second copy of it could quietly
/// answer something.
pub(crate) mod terminal {
    use crate::caps::{Capabilities, ColorDepth, Rgb};

    /// **Silent on OSC 10 and OSC 11, at a depth that has colour.**
    ///
    /// Both halves are deliberate. Silent about the two default colours is spec §5's silent path,
    /// which is load-bearing rather than a limitation: a cell with a default colour is left unmixed
    /// rather than mixed against a guess, because the guess is a dark theme and on a light-theme
    /// terminal it draws a shadow backwards. And **truecolor**, because at [`ColorDepth::None`] §5
    /// skips operator layers outright and a test would then be about the depth instead — which is
    /// exactly the trap a headless `Harness` falls into by default, since a caller-supplied sink is
    /// asked nothing and §10 will not invent a colour for one.
    ///
    /// This tier *is* reachable through [`Overrides`](crate::Overrides) — it is what
    /// `Harness::with_overrides` with `colors: TrueColor` produces, which is
    /// [`pinned_truecolor`](super::pinned_truecolor) — and it is built here the same way [`mixing`]
    /// is so that the two read as a pair.
    pub(crate) fn silent() -> Capabilities {
        Capabilities::answering(ColorDepth::TrueColor, None, None)
    }

    /// A terminal that answered both default colours, in white.
    ///
    /// **Reachable through [`Overrides`](crate::Overrides) since architecture ticket 22** —
    /// `default_fg` and `default_bg` are fields now — and this helper survives anyway, because its
    /// callers sit **below** `Screen` and have no `Overrides` to speak through: `crate::layer` calls
    /// `composite_run` with a `&Capabilities` directly, and [`silent`]'s callers in `crate::view` and
    /// `crate::reference` do the same. That is arch 22's *the two doors*, and it is why
    /// `Capabilities::answering` was never the alternative to a field it looked like: **the instrument
    /// that pins a capability has to be at the same depth as the gate.**
    ///
    /// White because it is the far end from a darkening: a `Mix` toward black over a default-coloured
    /// cell then moves as far as it can, so the picture says which cells were touched rather than
    /// leaving that to a two-value comparison.
    pub(crate) fn mixing() -> Capabilities {
        let white = Rgb::new(0xff, 0xff, 0xff);
        Capabilities::answering(ColorDepth::TrueColor, Some(white), Some(white))
    }
}

/// What a [`Recorder`] saw.
#[derive(Default, Debug)]
pub(crate) struct Recording {
    /// Every byte, in order, reassembled across partial writes.
    pub(crate) bytes: Vec<u8>,
    /// How many `write` calls took at least one byte.
    pub(crate) writes: usize,
    /// How many `write` calls returned `WouldBlock`.
    pub(crate) retries: usize,
    calls: usize,
}

/// A sink that counts what it is given, and can be made as awkward as a real pipe.
#[derive(Clone, Default)]
pub(crate) struct Recorder {
    shared: Arc<Mutex<Recording>>,
    /// The most bytes one `write` will take.
    chunk: Option<usize>,
    /// Every Nth call returns `WouldBlock` instead of taking anything.
    would_block_every: Option<usize>,
}

impl Recorder {
    pub(crate) fn new() -> Recorder {
        Recorder::default()
    }

    /// A sink that takes `chunk` bytes at a time and returns `WouldBlock` every `block`th call.
    ///
    /// That is spec §8's honest way to test the partial-write loop: whether a real pipe fragments a
    /// write is the kernel's business, so the fragmentation is made deterministic instead.
    pub(crate) fn awkward(chunk: usize, block: usize) -> Recorder {
        Recorder {
            shared: Arc::new(Mutex::new(Recording::default())),
            chunk: Some(chunk),
            would_block_every: Some(block),
        }
    }

    /// A handle to the same recording, so a test can read what the engine wrote.
    pub(crate) fn handle(&self) -> Arc<Mutex<Recording>> {
        Arc::clone(&self.shared)
    }
}

impl Write for Recorder {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut r = self.shared.lock().expect("the recorder is never poisoned");
        r.calls += 1;
        if let Some(n) = self.would_block_every {
            if r.calls % n == 0 {
                r.retries += 1;
                return Err(Error::new(
                    ErrorKind::WouldBlock,
                    "the sink is being awkward",
                ));
            }
        }
        let take = self.chunk.map_or(buf.len(), |c| c.min(buf.len()));
        r.bytes.extend_from_slice(&buf[..take]);
        r.writes += 1;
        Ok(take)
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

/// A truecolor terminal, silent on both default colours: what a compositing test needs pinned.
///
/// §5 skips an operator layer **outright** at [`ColorDepth::None`](crate::ColorDepth), which is what
/// a headless screen is unless something says otherwise — so a test about an operator that does not
/// pin this measures the depth instead of the operator.
pub(crate) fn pinned_truecolor() -> Overrides {
    Overrides {
        colors: Some(crate::caps::ColorDepth::TrueColor),
        ..Overrides::default()
    }
}

/// The same, and **OSC 8 as well**: what a test whose extended cells are hyperlinks needs pinned.
///
/// A hyperlink is the one channel `Mix` never names, so it is what this crate's table fixtures use to
/// make a cell extended without the mechanism under test also transforming the variable. The cost of
/// that choice is that the fixture then depends on a *capability*: OSC 8 reaches the wire only where
/// the terminal has it, and a headless screen is asked nothing.
///
/// **So a gate that pins the depth and not this is a gate about the depth.** Architecture ticket 22
/// found six instances of that shape on this axis alone, none of them a defect yet and all of them
/// one ticket away from being one — because a gate can be written today and go vacuous later, and
/// nothing in the gate changes. What changes is a capability arriving with a reader.
///
/// The audit that finds them is a grep and not a list: `rg 'screen\.link\('` over `src/`, `tests/`
/// and `examples/`. Three of the six were missed by a reading that enumerated instead, and two of
/// those three are not in `src/` at all.
pub(crate) fn pinned_extended() -> Overrides {
    Overrides {
        hyperlinks: Some(true),
        ..pinned_truecolor()
    }
}

/// A screen, a sink, and the terminal model the sink's bytes are replayed through.
///
/// One harness, two callers: [`crate::roundtrip`] drives the shapes ticket 03 could express and
/// [`crate::gates`] drives spec §14's normative twelve through it. Every `present` here closes the
/// round trip, so a caller cannot accidentally get a weaker one by writing its own loop.
pub(crate) struct Harness {
    pub(crate) screen: Screen,
    recording: Arc<Mutex<Recording>>,
    term: TermModel,
    replayed: usize,
    /// Prefixes every failure message. A gate driving twelve scenes has to say which one failed;
    /// a single-scene test has nothing useful to add and leaves it empty.
    label: String,
    /// What the session prologue cost, so that the counters below are about **frames**.
    ///
    /// Auto-wrap is switched off once on entering the alt screen (§8), which is one write of five
    /// bytes before any frame exists. Subtracting it here rather than in every caller is what keeps
    /// *one `write` per frame* a statement a gate can make — and the prologue itself is not left
    /// unasserted for it: `crate::engine`'s
    /// `auto_wrap_is_disabled_once_on_entry_and_restored_on_leaving` is what covers it, in both
    /// directions.
    prologue: (usize, usize),
    /// Rows a sweep's `repaint` invalidated and no frame has re-established since.
    ///
    /// **All false until a `repaint` actually crosses**, which is what separates this from the
    /// mirror's own unknown row — see
    /// [`assert_screen_matches_frame`](Harness::assert_screen_matches_frame).
    stale: Vec<bool>,
}

impl Harness {
    pub(crate) fn new(w: u16, h: u16) -> Harness {
        Harness::with_sink(w, h, Recorder::new())
    }

    pub(crate) fn with_sink(w: u16, h: u16, sink: Recorder) -> Harness {
        Harness::with_sink_and_overrides(w, h, sink, Overrides::default())
    }

    /// A harness on a **declared** tier: a caller-supplied sink detects nothing, so whatever these
    /// overrides pin is what [`Screen::capabilities`](crate::Screen::capabilities) reports — up to
    /// and including truecolor, which is what makes headless a declared tier rather than the lowest
    /// one.
    pub(crate) fn with_overrides(w: u16, h: u16, overrides: Overrides) -> Harness {
        Harness::with_sink_and_overrides(w, h, Recorder::new(), overrides)
    }

    pub(crate) fn with_sink_and_overrides(
        w: u16,
        h: u16,
        sink: Recorder,
        overrides: Overrides,
    ) -> Harness {
        let recording = sink.handle();
        let (screen, _wake) = Engine::new(Config {
            size: (w, h),
            output: Output::Sink(Box::new(sink)),
            // The deterministic mode is public API, not a test fixture: `present` composites,
            // packs, serialises and writes inline on this thread, so a test is a straight-line
            // program with no condvar, no join, no timeout and no flake.
            clock: Clock::Manual,
            overrides,
        })
        .attach()
        .expect("attaching to a sink cannot fail");
        let prologue = {
            let r = recording.lock().expect("the recorder is never poisoned");
            (r.bytes.len(), r.writes)
        };
        Harness {
            screen,
            recording,
            term: TermModel::new(w, h),
            // **Zero, not the prologue's length.** The model has to see those bytes: it tracks
            // DECAWM from ticket 13 on, and a model that never saw the reset would be modelling a
            // terminal this engine does not talk to.
            replayed: 0,
            label: String::new(),
            prologue,
            stale: vec![false; h as usize],
        }
    }

    /// Resize the screen, and the terminal with it.
    ///
    /// A real terminal that changes size clears itself, and the model is replaced for the same
    /// reason the mirror is: a resized screen is showing something nobody recorded. Everything the
    /// round trip asserts still has to hold on the first frame after, which is the point of driving
    /// a resize through the harness rather than through `Screen` alone.
    pub(crate) fn resize(&mut self, w: u16, h: u16) {
        self.screen.resize(w, h);
        self.term = TermModel::new(w, h);
        // A fresh model beside a fresh mirror: they agree by construction, so nothing is stale.
        self.stale = vec![false; h as usize];
        // The bytes already written described the old screen; nothing after this replays them.
        self.replayed = self.recording.lock().unwrap().bytes.len();
    }

    /// Name what this harness is driving, so a failure says which of twelve scenes it was.
    pub(crate) fn labelled(mut self, label: &str) -> Harness {
        self.label = format!("{label}: ");
        self
    }

    /// Present, replay whatever is new in the sink, and assert the three things that must agree.
    pub(crate) fn present(&mut self) -> Presented {
        let presented = self.screen.present();
        let fresh = {
            let r = self.recording.lock().unwrap();
            r.bytes[self.replayed..].to_vec()
        };
        self.replayed += fresh.len();
        // The model interns into the engine's own table, so a handle it mints for a cluster the
        // engine already knows *is* the engine's handle — which is what lets the assertion below
        // compare whole cells rather than rendered text. A cluster the engine never wrote gets a
        // handle nobody has, and the comparison fails, which is the point.
        self.term.feed(&fresh, self.screen.tables_mut());
        self.note_what_a_sweep_invalidated();

        assert_eq!(
            self.term.unrecognised(),
            0,
            "{}the serializer emitted a sequence the terminal model does not parse",
            self.label
        );
        self.assert_screen_matches_frame();
        self.assert_mirror_matches_frame();
        presented
    }

    /// Carry [`stale`](Harness::stale) forward across the frame that has just gone out.
    ///
    /// A `repaint` packet invalidates every row; a row is re-established when the serializer records
    /// it as known, which is the same condition — one frame having written every column of it — and
    /// is read off the mirror rather than recomputed here so the two cannot drift apart.
    fn note_what_a_sweep_invalidated(&mut self) {
        // **Two triggers, and the order between them is the whole of this method.**
        //
        // A packet that carried the flag is the frame the mirror was told on: from there, a row is
        // re-established when the serializer records it as known, and that is read off the mirror
        // rather than recomputed here so the two cannot drift apart.
        //
        // A flag still **latched** on the `Screen` is a different state: a sweep has run, the handles
        // have already moved, and no frame has taken the news yet. In that window the mirror's own
        // `known` flags are answers to an older question — they were set before the handles moved and
        // nothing has reset them — so they must not clear anything. Hence the fill *after* the loop.
        //
        // The window is reachable and is not a corner: `sweep_now`, then any number of idle
        // `present` calls, then the frame that carries it. Found the moment the extended gates came
        // back onto this harness at impl 13, which is the first time a sweep and a round trip were in
        // the same test.
        if self.screen.packet().repaint() {
            self.stale.fill(true);
        }
        for y in 0..self.screen.size().1 {
            if self.screen.mirror().is_known(y) {
                self.stale[y as usize] = false;
            }
        }
        if self.screen.repaint_pending() {
            self.stale.fill(true);
        }
    }

    /// The replayed screen must agree with the frame everywhere a sweep has not moved the handles.
    ///
    /// **A sweep is the one thing the round trip cannot see across.** A renumbered handle names the
    /// same text in the same colours, so the terminal goes on showing the right thing — but the model
    /// records a *cell*, and the cell it recorded carries the handle that was current when the bytes
    /// went out. Comparing it against a frame whose handles have since moved compares two spellings
    /// of one screen.
    ///
    /// # It skips [`stale`](Harness::stale) rows and **not** unknown ones, and the difference is the
    /// whole of this method's coverage
    ///
    /// `Mirror`'s own flag starts *unknown* at construction, because ADR 0006 is about a real
    /// terminal and a real terminal at startup is showing something nobody recorded. **That is not
    /// true of this harness**, where the model is constructed blank beside a mirror constructed
    /// blank and the two agree by construction — the note on
    /// [`assert_mirror_matches_frame`](Harness::assert_mirror_matches_frame) has said so since
    /// ticket 03.
    ///
    /// Keying the skip off `is_known` therefore looked right and was a silent hole: a row only
    /// becomes known when one frame writes **every column** of it, so any harness whose layers never
    /// span the screen has every row unknown for ever. `crate::roundtrip`'s
    /// `a_layer_hanging_off_an_edge_survives_the_round_trip` is twenty columns wide with a layer that
    /// clips to seven of them and **contains no assertion of its own** — it would have asserted
    /// literally nothing. So the harness tracks the narrower fact itself: a row is stale from the
    /// frame a `repaint` reaches the mirror until some frame re-establishes it, and before the first
    /// sweep nothing is stale and every cell is compared exactly as it was.
    fn assert_screen_matches_frame(&self) {
        let (w, h) = self.screen.size();
        let frame = self.screen.frame();
        for y in 0..h {
            if self.stale[y as usize] {
                continue;
            }
            for x in 0..w {
                assert_eq!(
                    self.term.cell(x, y),
                    frame.row(y)[x as usize],
                    "{}the replayed screen and the composited frame disagree at ({x}, {y})",
                    self.label
                );
            }
        }
    }

    /// The mirror must agree with the frame everywhere.
    ///
    /// What this buys is the cells the frame *wrote*: a mirror that missed an update, or recorded
    /// the wrong style, fails here. What it does not buy is the cells nobody touched — a fresh
    /// mirror and a fresh frame are both blank, so those match by construction.
    ///
    /// [`stale`](Harness::stale) rows are skipped, for the reason
    /// [`assert_screen_matches_frame`](Harness::assert_screen_matches_frame) states at length: after
    /// a sweep the mirror holds handles that have moved, which is the whole of what `Packet::repaint`
    /// is about. Before the first sweep on a harness nothing is stale and this walk covers everything
    /// unconditionally, exactly as it did before ticket 08.
    fn assert_mirror_matches_frame(&self) {
        let (w, h) = self.screen.size();
        let frame = self.screen.frame();
        for y in 0..h {
            if self.stale[y as usize] {
                continue;
            }
            for x in 0..w {
                assert_eq!(
                    self.screen.mirror().cell(x, y),
                    frame.row(y)[x as usize],
                    "{}the mirror and the composited frame disagree at ({x}, {y})",
                    self.label
                );
            }
        }
    }

    /// Bytes the **frames** have written, which is the session's total less the prologue.
    pub(crate) fn bytes_written(&self) -> usize {
        self.recording.lock().unwrap().bytes.len() - self.prologue.0
    }

    /// `write` calls the **frames** have made. See [`prologue`](Harness::prologue).
    pub(crate) fn writes(&self) -> usize {
        self.recording.lock().unwrap().writes - self.prologue.1
    }

    pub(crate) fn retries(&self) -> usize {
        self.recording.lock().unwrap().retries
    }
}
