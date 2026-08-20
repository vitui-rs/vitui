//! The lifecycle: `Engine`, `Screen`, and the one exit that is `present`.
//!
//! # Scope
//!
//! `present` composites, packs, serialises and writes into the caller's sink inline on the calling
//! thread. That is the deterministic mode of spec §14, and it is **public API rather than test
//! scaffolding**: an application author testing their own UI needs the same determinism the
//! engine's tests need. A test is a straight-line program — draw, present, assert on the sink — with
//! no condvar, no join, no timeout and no flake.
//!
//! The three-thread path arrives at ticket 18 and does not change a byte of what this asserts.
//! `wait`, `next_event`, the frame clock and the caret arrive with tickets 19, 20 and 21.
//!
//! # What `attach` now does before it hands anything back
//!
//! It asks. [`Engine::attach`] fires spec §10's whole query batch at the terminal behind one DA1
//! sentinel, resolves the seven levels of precedence over the answers, and the result is immutable
//! for the life of the [`Screen`] — [`Screen::capabilities`]. **Which of the three grounds it is
//! on is decided by [`Output`]**: a caller-supplied sink is a fully declared tier and asks nothing,
//! a standard output that is not a tty is a terminal nothing is known about and also asks nothing,
//! and only a real terminal is queried.

use std::io::{ErrorKind, Write};
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::caps::{Capabilities, Env, Ground, Overrides, assemble};
use crate::damage::Run;
use crate::detect::{CEILING, Tty, detect};
use crate::exts::LinkId;
use crate::layer::LayerStack;
use crate::packet::Packet;
use crate::quirks::Quirks;
use crate::serial::Serializer;
use crate::surface::Surface;

/// Where the frame clock takes its time from.
///
/// A concrete enum, never a trait, because the seam has no traits in it and a trait object here
/// would be the first. `std::time::Instant` has no constructor from a number — `from_nanos`,
/// `From<Duration>` and `default` all fail to compile — so the manual clock is one captured
/// `Instant` plus an offset, which is why no public signature changes and why nobody needs a
/// `vitui::Timestamp` newtype to make testing possible.
///
/// **Nothing reads this yet, and that is the honest state rather than an oversight.** `present`
/// composites, packs, serialises and writes inline on the calling thread whichever value is set,
/// because there is no second thread for `System` to differ by and no `wait` for a deadline to
/// gate. It is declared now because the ticket that decided it insisted the deterministic mode is
/// public API and not a test fixture, and because a knob that appears later is a breaking change to
/// every `Config` literal. Ticket 19 is its first reader.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Clock {
    /// Real time.
    #[default]
    System,
    /// Time only moves when the caller moves it.
    Manual,
}

/// Where the frame's bytes go.
///
/// A caller-supplied sink is what makes headless a fully *declared* tier rather than the lowest one
/// (spec §10): with detection switched off and every axis pinned, any tier is testable, truecolor
/// included.
///
/// This is the one trait object in the engine and it is `std::io::Write`, not one of ours. Spec
/// §12's refusal is that the engine defines no trait to call *upward* — it has nothing to ask of
/// its caller. A sink is downward I/O, and inventing a `vitui::Sink` for it would make every caller
/// write an adapter for a trait std already has.
#[derive(Default)]
pub enum Output {
    /// The process's standard output. Ticket 22 is what puts a terminal into a state where that is
    /// the right thing to do.
    ///
    /// **This is what decides whether anything is detected.** A real terminal is queried; a
    /// standard output that turns out to be a pipe or a file is not, because there is nobody to
    /// answer and the sentinel would burn the whole ceiling for nothing.
    #[default]
    Terminal,
    /// Anywhere the caller likes.
    ///
    /// **Headless, and a fully *declared* tier rather than the lowest one**: detection is switched
    /// off and every axis is pinned through [`Config::overrides`], so *any* tier is testable,
    /// truecolor included. A floor tier could never have provided that, and spec §14's nine option
    /// sets are what need it.
    Sink(Box<dyn Write + Send>),
}

impl std::fmt::Debug for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Output::Terminal => f.write_str("Output::Terminal"),
            Output::Sink(_) => f.write_str("Output::Sink(..)"),
        }
    }
}

/// Everything the engine is told before it starts.
///
/// Every knob visible in one place, and no builder: that is worth more to "the engine is
/// comprehensible on its own" than a chain of setters. Use struct update syntax.
///
/// ```
/// use vitui_engine::{Clock, Config};
///
/// let config = Config {
///     clock: Clock::Manual,
///     ..Default::default()
/// };
/// assert_eq!(config.size, (80, 24));
/// ```
#[derive(Debug)]
pub struct Config {
    /// Where the frame clock takes its time from.
    pub clock: Clock,
    /// Where the frame's bytes go.
    pub output: Output,
    /// The size to use when there is no terminal to ask. A real one is asked, and answers.
    pub size: (u16, u16),
    /// What was asked of the terminal, before anything was detected.
    ///
    /// This is where an application's own `--ascii` and `--no-color` land: **the engine parses no
    /// argv**, and every field is an `Option` so that the environment can fill a `None` and can
    /// never overrule a `Some`.
    pub overrides: Overrides,
}

impl Config {
    /// The size a terminal that has never been asked is assumed to be.
    const DEFAULT_SIZE: (u16, u16) = (80, 24);
}

impl Default for Config {
    fn default() -> Config {
        Config {
            clock: Clock::default(),
            output: Output::default(),
            size: Config::DEFAULT_SIZE,
            overrides: Overrides::default(),
        }
    }
}

/// Why attaching failed.
///
/// Not `std::io::Error`: "the terminal never answered the capability query" is not an I/O failure,
/// and saying so in a signature is a lie a reader has to unlearn.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum AttachError {
    /// The terminal did not answer the capability query inside the negotiation window.
    ///
    /// **Nothing at all came back**, which is the only case the numeric ceiling exists for. A
    /// terminal that answered part of the batch and then went quiet is not this: what arrived is
    /// kept and the rest falls through to conservative defaults.
    NoAnswer,
}

impl std::fmt::Display for AttachError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttachError::NoAnswer => {
                f.write_str("the terminal did not answer the capability query")
            }
        }
    }
}

impl std::error::Error for AttachError {}

/// The engine, before any thread exists.
///
/// ```
/// fn assert_send<T: Send>() {}
/// assert_send::<vitui_engine::Engine>();
/// ```
#[derive(Debug)]
pub struct Engine {
    config: Config,
}

impl Engine {
    /// An engine that has not touched anything yet.
    pub fn new(config: Config) -> Engine {
        Engine { config }
    }

    /// Take the terminal, ask it what it can do, and hand back the app thread's world.
    ///
    /// # What is asked, and of whom
    ///
    /// Only a real terminal is queried, and [`Output`] is what says whether there is one. The batch
    /// goes out in a single write behind a trailing DA1 and the loop reacts to the sentinel's
    /// arrival rather than to a clock; the numeric ceiling exists for the one case where there is
    /// no terminal at all.
    ///
    /// A terminal that answers is also asked its size, which is the one question with no escape
    /// sequence in the batch because the kernel already knows. [`Config::size`] is what applies
    /// when there is nobody to ask.
    ///
    /// # Errors
    ///
    /// [`AttachError::NoAnswer`] when standard output is a terminal and **nothing at all** came
    /// back inside the ceiling.
    pub fn attach(self) -> Result<(Screen, WakeHandle), AttachError> {
        let env = Env::from_process();
        let headless = matches!(self.config.output, Output::Sink(_));
        let mut tty = if headless { None } else { Tty::open() };
        let ground = match (headless, &tty) {
            (true, _) => Ground::Headless,
            (false, None) => Ground::NotATty,
            (false, Some(_)) => Ground::Tty,
        };
        // **`TERM=dumb` is asked nothing at all**, and that is not an optimisation. It sits at
        // level 4 of spec §10's precedence, *above* detection, so every answer it could give is
        // already overruled — and a terminal that says it speaks no escape sequences would answer
        // no DA1 either, which would turn a legitimate `TERM=dumb` into `AttachError::NoAnswer`.
        // The pty is still opened, because size and the single reader are not escape sequences.
        let detected = match tty.as_mut() {
            Some(tty) if !env.term_is_dumb() => detect(tty, CEILING)?,
            _ => crate::caps::Detected::default(),
        };
        let quirks = Quirks::lookup(detected.version.as_deref(), &env);
        let caps = assemble(self.config.overrides, &env, ground, &detected, quirks);

        // A terminal that is there is asked its size, which is the one question with no escape
        // sequence in the batch because the kernel already knows. A zero falls back rather than
        // through: every buffer here is sized from this pair, and a zero-column screen is not a
        // smaller screen, it is a different set of edge cases in every loop below.
        let (w, h) = match ground {
            Ground::Tty => Tty::size()
                .filter(|(w, h)| *w > 0 && *h > 0)
                .unwrap_or(self.config.size),
            _ => self.config.size,
        };
        let wakes = Arc::new(AtomicU32::new(0));
        let screen = Screen {
            size: (w, h),
            layers: LayerStack::new(),
            frame: Surface::new(w, h),
            runs: Vec::with_capacity(h as usize * 4),
            packet: Packet::new(),
            serializer: Serializer::new(w, h),
            sink: match self.config.output {
                Output::Terminal => Box::new(std::io::stdout()),
                Output::Sink(sink) => sink,
            },
            caps,
            tty,
            _not_send: PhantomData,
        };
        Ok((screen, WakeHandle { wakes }))
    }
}

/// What `present` did.
///
/// `submitted`, never `painted`. The distinction is spec §12's refusal 7: the engine offers no
/// completion anywhere, so it can say a frame was handed on and it cannot say a frame was shown.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Presented {
    /// Whether a frame was handed on. False when nothing was damaged.
    pub submitted: bool,
    /// How many frames were folded into this one because the render thread was busy. Always zero
    /// while there is one thread; ticket 18 is what makes it move.
    pub coalesced: u32,
    /// Whether the frame was thrown away because the terminal resized under it. Writing a 300x80
    /// frame into a terminal that is now 120x40 wraps and scrolls, which is worse than a missing
    /// frame. Ticket 22 is what brings the size atomic this reads.
    pub discarded_for_resize: bool,
}

/// A handle that can wake the app thread from anywhere.
///
/// ```
/// fn assert_send_sync<T: Send + Sync>() {}
/// assert_send_sync::<vitui_engine::WakeHandle>();
/// ```
#[derive(Clone, Debug)]
pub struct WakeHandle {
    wakes: Arc<AtomicU32>,
}

impl WakeHandle {
    const POSTED: u32 = 1;
    const QUIT: u32 = 2;

    /// Ask the app thread to wake and run a frame.
    pub fn post(&self) {
        self.wakes.fetch_or(WakeHandle::POSTED, Ordering::Release);
    }

    /// Ask the app thread to stop. Never paced.
    pub fn quit(&self) {
        self.wakes.fetch_or(WakeHandle::QUIT, Ordering::Release);
    }

    #[cfg(test)]
    fn pending(&self) -> u32 {
        self.wakes.load(Ordering::Acquire)
    }
}

/// The app thread's whole world.
///
/// # Threading
///
/// `Screen` is deliberately not [`Send`]: it is the app thread's, and the app thread is the one
/// that must never block.
///
/// ```compile_fail,E0277
/// let (screen, _wake) = vitui_engine::Engine::new(Default::default()).attach().unwrap();
/// std::thread::spawn(move || screen.size());
/// ```
///
/// ```
/// let (screen, _wake): (vitui_engine::Screen, vitui_engine::WakeHandle) =
///     vitui_engine::Engine::new(Default::default()).attach().unwrap();
/// assert_eq!(screen.size(), (80, 24));
/// ```
pub struct Screen {
    size: (u16, u16),
    layers: LayerStack,
    /// The composited grid. It stays on this thread; only the packed damage crosses.
    frame: Surface,
    /// The frame's damaged runs, reused every frame so the steady state allocates nothing.
    runs: Vec<Run>,
    packet: Packet,
    serializer: Serializer,
    sink: Box<dyn Write + Send>,
    /// What the terminal can do, sampled once during `attach` and never again.
    caps: Capabilities,
    /// The pty detection read its answers from, held rather than dropped so that **one thread ever
    /// reads this file descriptor**. Ticket 20's input thread adopts this channel; a second reader
    /// would steal bytes from the first. `None` whenever there is no terminal.
    #[allow(dead_code)]
    tty: Option<Tty>,
    _not_send: PhantomData<*const ()>,
}

impl Screen {
    /// The screen's size in cells.
    pub fn size(&self) -> (u16, u16) {
        self.size
    }

    /// What the terminal on the other end can do.
    ///
    /// **Sampled once during `attach` and immutable for the life of this `Screen`**, which is why
    /// this takes `&self` and why no component ever has to handle a capability changing between
    /// frames. The price is explicit: a terminal that changed underneath the process — a
    /// reconnected ssh session, a SIGTSTP/SIGCONT cycle — cannot be re-detected without a fresh
    /// `attach`. That is spec §15's terminal lifecycle, not a degradation question.
    pub fn capabilities(&self) -> &Capabilities {
        &self.caps
    }

    /// The layer stack: add a layer, draw into one, move it, remove it, and ask which one is on
    /// top at a point.
    pub fn layers(&mut self) -> &mut LayerStack {
        &mut self.layers
    }

    /// Composite the damaged rectangles, pack them, serialise them, and write once.
    ///
    /// The only exit. Damage is marked by the drawing verbs and cleared here, and neither is
    /// reachable from outside — a forgotten clear produces a frame that repaints for ever, so it is
    /// an invariant rather than a chore, and it is also why nothing above the engine can force a
    /// full repaint.
    pub fn present(&mut self) -> Presented {
        self.layers.take_damage_into(&mut self.frame);

        // The idle path, and it is one scan of a couple of summary words rather than of the bitset:
        // an idle frame costs nanoseconds and clears nothing. This is the question ticket 19's
        // condvar path will interrogate, asked here first.
        if self.frame.damage().is_empty() {
            return Presented {
                submitted: false,
                coalesced: 0,
                discarded_for_resize: false,
            };
        }

        self.runs.clear();
        let runs = &mut self.runs;
        self.frame.damage().for_each_run(|r| runs.push(r));

        // The composite answers with the run it actually repainted, which is the damaged one
        // widened by whatever a wide-glyph repair had to reach — a repair damages cells outside the
        // layer's own rectangle (spec §5), and a repair that is not packed is a blanked half the
        // terminal goes on showing.
        for run in &mut self.runs {
            *run = self.layers.composite_run(&mut self.frame, *run);
        }
        merge_touching(&mut self.runs);

        self.packet
            .pack(&self.runs, &self.frame, self.layers.tables());
        let bytes = self.serializer.serialize(&self.packet);
        write_frame(&mut *self.sink, bytes);

        self.frame.damage_mut().clear();
        self.layers.clear_damage();

        Presented {
            submitted: true,
            coalesced: 0,
            discarded_for_resize: false,
        }
    }

    /// Start again at a new size.
    ///
    /// **Resize damages everything and is simply correct, not clever** (spec §6): mark the whole
    /// screen, clear every structure, reallocate the surfaces. A resized frame is composited from
    /// the layers rather than patched out of the one before it.
    ///
    /// # The mirror starts blank, and that is a debt rather than a claim
    ///
    /// A terminal that has just changed size is showing something nobody recorded — it reflows on
    /// `SIGWINCH`, it does not clear — and the honest value for the mirror is ADR 0006's **unknown
    /// row**, which does not exist yet. A fresh `Mirror` says *blank* instead, which is a claim
    /// about the terminal that is not true.
    ///
    /// It is harmless here only because every cell of the new screen is damaged and therefore
    /// written unconditionally. **Ticket 14 is what ends that** — the equality filter skips a cell
    /// whose composited value equals the mirror's, so the first blank cell of a resized screen would
    /// be skipped and the reflowed content under it would stay — and ticket 14 carries the unknown
    /// row in its own criteria, pointed at this function.
    ///
    /// There is nothing to invalidate beyond that, and the reason is §5's: the flattened prefix
    /// cache that would have had to be invalidated was refused, on a budget the damage rectangles
    /// deliver instead.
    ///
    /// The layers keep their rectangles and their cells. The runtime brings a rectangle and a draw
    /// for every layer every frame (spec §12), so a layer whose shape must change is
    /// [`set_rect`](crate::LayerStack::set_rect)'s business and not this one's.
    ///
    /// **Nothing outside this crate calls it yet.** §12's public surface has no `resize` on
    /// `Screen`: the authoritative size is one packed atomic written by the input thread, and a
    /// resize reaches the application as an `Event`. Ticket 22 is what brings both, and this is what
    /// it will call.
    #[allow(dead_code)]
    pub(crate) fn resize(&mut self, w: u16, h: u16) {
        self.size = (w, h);
        self.frame = Surface::new(w, h);
        self.frame.damage_mut().mark_all();
        self.runs = Vec::with_capacity(h as usize * 4);
        self.serializer = Serializer::new(w, h);
        self.layers.forget_damage();
    }

    /// Mint a hyperlink id for `uri`, or return the one this screen already minted for it.
    ///
    /// The only way to get a [`LinkId`], which is what makes the type opaque in the sense ADR 0023
    /// asks for: a caller can say *this hyperlink again* without being able to say *entry 7*. Ids
    /// are deduplicated, so a page of a hundred distinct links costs a hundred entries however many
    /// cells carry them.
    ///
    /// The id is then handed to [`View::restyle`](crate::View::restyle) through
    /// [`Restyle::link`](crate::Restyle::link).
    ///
    /// # It belongs to this screen's handle space
    ///
    /// Spec §3's invariant is that *every surface in a layer stack speaks that stack's handle
    /// space*, and this id is part of that space. Using it on a **standalone** [`Surface`] — one
    /// reached through [`Surface::root`](crate::Surface::root) rather than through
    /// [`LayerStack::view`](crate::LayerStack::view) — puts a handle into a table that never minted
    /// it, and the URI does not travel with the cell. `add_content_with` is what reconciles a
    /// surface drawn off to one side, by renumbering it once at donation (ticket 10); until then,
    /// a hyperlink belongs in a layer of the screen that minted it.
    ///
    /// ```
    /// let (mut screen, _wake) = vitui_engine::Engine::new(Default::default()).attach().unwrap();
    /// let a = screen.link("https://example.com/");
    /// assert_eq!(a, screen.link("https://example.com/"));
    /// assert_ne!(a, vitui_engine::LinkId::NONE);
    /// ```
    pub fn link(&mut self, uri: &str) -> LinkId {
        self.layers.tables_mut().link(uri)
    }

    /// The handle space this screen's layers speak, for the terminal model to intern into.
    #[cfg(test)]
    pub(crate) fn interner_mut(&mut self) -> &mut crate::intern::Interner {
        &mut self.layers.tables_mut().interner
    }

    #[cfg(test)]
    pub(crate) fn frame(&self) -> &Surface {
        &self.frame
    }

    /// The damaged runs of the frame `present` last submitted.
    ///
    /// They survive `present` — `runs` is cleared at the *start* of the next one — which is what
    /// lets a gate composite, present, and only then ask what the damage structure had reported.
    #[cfg(test)]
    pub(crate) fn runs(&self) -> &[Run] {
        &self.runs
    }

    #[cfg(test)]
    pub(crate) fn packet(&self) -> &Packet {
        &self.packet
    }

    #[cfg(test)]
    pub(crate) fn mirror(&self) -> &crate::serial::Mirror {
        self.serializer.mirror()
    }
}

/// Fold runs that a repair widened into each other back into one.
///
/// Two runs on a row arrive with at least one undamaged column between them, and each can widen by
/// one column — so the gap can close, and a gap that closed is one run rather than two. Leaving them
/// apart would cost a cursor move between adjacent cells and break §14's gate #2, which reads
/// exactly that: *two runs that touch are one run*.
///
/// They cannot **overlap**, and the reason is worth keeping: a run widens to its right only by
/// blanking an orphaned `CONTINUATION` there, and its neighbour widens to its left only by blanking
/// an orphaned wide head — and one column cannot be both.
///
/// In place, because the steady state allocates nothing (`tests/alloc.rs`).
fn merge_touching(runs: &mut Vec<Run>) {
    if runs.is_empty() {
        return;
    }
    let mut kept = 0;
    for at in 1..runs.len() {
        let next = runs[at];
        if next.y == runs[kept].y && next.lo <= runs[kept].hi.saturating_add(1) {
            runs[kept].hi = runs[kept].hi.max(next.hi);
        } else {
            kept += 1;
            runs[kept] = next;
        }
    }
    runs.truncate(kept + 1);
}

/// Write the whole frame, retrying what the kernel would not take.
///
/// **The frame is never split on purpose.** A synchronised-output block spanning two `write` calls
/// is still one block to the terminal; a frame split into two blocks tears. This loop is only about
/// the kernel's buffer being smaller than the frame (spec §8).
///
/// A write error is dropped on the floor here. Ticket 22 owns shutdown, and it is what will have
/// somewhere to put one.
fn write_frame(sink: &mut (dyn Write + Send), bytes: &[u8]) {
    let mut at = 0;
    while at < bytes.len() {
        match sink.write(&bytes[at..]) {
            Ok(0) => break,
            Ok(n) => at += n,
            Err(e) if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::Interrupted => {}
            Err(_) => break,
        }
    }
    let _ = sink.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wake_handle_records_a_post_and_a_quit_separately() {
        let (_screen, wake) = Engine::new(Config::default()).attach().unwrap();
        assert_eq!(wake.pending(), 0);
        wake.post();
        assert_eq!(wake.pending(), WakeHandle::POSTED);
        wake.quit();
        assert_eq!(wake.pending(), WakeHandle::POSTED | WakeHandle::QUIT);
    }

    #[test]
    fn a_wake_handle_clone_shares_the_same_flags() {
        let (_screen, wake) = Engine::new(Config::default()).attach().unwrap();
        let other = wake.clone();
        other.post();
        assert_eq!(wake.pending(), WakeHandle::POSTED);
    }

    #[test]
    fn a_screen_takes_its_size_from_the_config() {
        let config = Config {
            size: (120, 40),
            ..Default::default()
        };
        let (screen, _wake) = Engine::new(config).attach().unwrap();
        assert_eq!(screen.size(), (120, 40));
    }

    /// A resized frame is composited, not patched.
    ///
    /// The assertion that says so is the one about the cell at (11, 5): it is inside the *new*
    /// screen and outside the old one, so a frame that carried anything across would either hold a
    /// stale cell there or hold nothing. `Harness::present` closes the round trip on the frame
    /// after, so the mirror and the replayed screen have to agree with it too — which is the half
    /// that would fail if the serializer kept writing against a mirror of the old size.
    #[test]
    fn a_resized_frame_is_composited_rather_than_patched() {
        let mut h = crate::testing::Harness::new(8, 3);
        let id = h
            .screen
            .layers()
            .add_content(0, crate::geom::Rect::new(0, 0, 20, 8), true);
        h.screen.layers().view(id).unwrap().fill(
            crate::geom::Rect::new(0, 0, 20, 8),
            "#",
            crate::style::Style::new(),
        );
        h.present();
        assert_eq!(h.screen.size(), (8, 3));

        h.resize(16, 6);
        assert_eq!(h.screen.size(), (16, 6));
        let presented = h.present();
        assert!(presented.submitted, "a resize repaints the whole screen");
        assert_eq!(
            h.screen.runs().len(),
            6,
            "every row of the new screen is one run"
        );
        let frame = h.screen.frame();
        assert_eq!(frame.size(), (16, 6));
        for y in 0..6u16 {
            for x in 0..16u16 {
                assert_eq!(
                    frame.row(y)[x as usize].grapheme.as_scalar(),
                    Some('#'),
                    "({x}, {y}) was not composited from the layer"
                );
            }
        }
    }

    /// The other half of "clears every structure": a resize does not leave last frame's exposures
    /// or last frame's per-layer damage behind, and the frame after a repainted one is idle again.
    #[test]
    fn the_frame_after_a_resize_is_idle() {
        let mut h = crate::testing::Harness::new(8, 3);
        let id = h
            .screen
            .layers()
            .add_content(0, crate::geom::Rect::new(0, 0, 8, 3), true);
        h.screen.layers().view(id).unwrap().fill(
            crate::geom::Rect::new(0, 0, 8, 3),
            ".",
            crate::style::Style::new(),
        );
        h.present();
        // A topology change whose exposure would otherwise survive the resize.
        h.screen.layers().remove(id);
        h.resize(20, 5);
        assert!(h.present().submitted);
        assert!(
            !h.present().submitted,
            "the frame after a resize has nothing left to say"
        );
    }

    /// **Gate.** The engine substitutes no glyph, at any `GlyphSet`, anywhere.
    ///
    /// > **Choosing a glyph before drawing is legitimate; replacing one after it is drawn is
    /// > forbidden.**
    ///
    /// The engine's behaviour on `--ascii` is **zero, and that is the point**: it emits no glyphs of
    /// its own — spec §4 leaves it three verbs and no box-drawing primitive, and boxes are `fill`s —
    /// so there is nothing for it to substitute even if it wanted to. The prohibition costs no
    /// discipline; it is a property of the surface that already exists, and this is what keeps it
    /// one.
    ///
    /// Braille is the cluster that makes it matter: 256 states per cell, exactly what a font lacks,
    /// and a substitution table applied here would turn a chart into noise while every budget stayed
    /// green. `Harness::present` closes the round trip, so the replayed terminal has to agree too.
    #[test]
    fn the_engine_substitutes_no_glyph_at_any_glyph_set() {
        let drawn = "\u{28ff}\u{2500}\u{2502}\u{250c}\u{25cf}e\u{301}";
        for overrides in [
            crate::caps::Overrides::default(),
            crate::caps::Overrides::plain(),
            crate::caps::Overrides {
                glyphs: Some(crate::caps::GlyphSet::Unicode),
                ..Default::default()
            },
        ] {
            let mut h = crate::testing::Harness::with_overrides(12, 1, overrides);
            let id = h
                .screen
                .layers()
                .add_content(0, crate::geom::Rect::new(0, 0, 12, 1), true);
            h.screen
                .layers()
                .view(id)
                .unwrap()
                .text(0, 0, drawn, crate::style::Style::new());
            h.present();

            let cells = h.screen.frame().row(0).to_vec();
            let glyphs = h.screen.capabilities().glyphs;
            let interner = h.screen.interner_mut();
            let mut seen = String::new();
            for c in &cells {
                if c.grapheme.is_continuation() {
                    continue;
                }
                seen.push_str(&interner.resolve(c.grapheme).unwrap_or_default());
            }
            assert_eq!(seen.trim_end(), drawn, "a glyph was replaced at {glyphs:?}");
        }
    }

    /// The size comes from the terminal when there is one to ask, and from `Config` when there is
    /// not. A sink is the second case, and it is the one every test in this crate is on.
    #[test]
    fn a_default_config_is_the_declared_size() {
        let (screen, _wake) = Engine::new(Config::default()).attach().unwrap();
        assert_eq!(screen.size(), Config::DEFAULT_SIZE);
    }
}
