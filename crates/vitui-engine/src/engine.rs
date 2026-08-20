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
//! `wait`, `next_event`, the frame clock, the caret and capability detection arrive with tickets
//! 16, 19, 20 and 21.

use std::io::{ErrorKind, Write};
use std::marker::PhantomData;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::damage::Run;
use crate::layer::LayerStack;
use crate::packet::Packet;
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
    /// the right thing to do; ticket 16 is what asks the terminal what it can do.
    #[default]
    Terminal,
    /// Anywhere the caller likes.
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
    /// The size to use when there is no terminal to ask. Ticket 16 is what asks a real one.
    pub size: (u16, u16),
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
        }
    }
}

/// Why attaching failed.
///
/// Not `std::io::Error`: "the terminal never answered the capability query" is not an I/O failure,
/// and saying so in a signature is a lie a reader has to unlearn. Ticket 16 is what returns one.
#[derive(Debug)]
#[non_exhaustive]
pub enum AttachError {
    /// The terminal did not answer the capability query inside the negotiation window.
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

    /// Take the terminal and hand back the app thread's world.
    ///
    /// # Errors
    ///
    /// Never, yet. Ticket 16 is what can fail here.
    pub fn attach(self) -> Result<(Screen, WakeHandle), AttachError> {
        let (w, h) = self.config.size;
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
    _not_send: PhantomData<*const ()>,
}

impl Screen {
    /// The screen's size in cells.
    pub fn size(&self) -> (u16, u16) {
        self.size
    }

    /// The layer stack: add a layer, and draw into one. Removing, reordering and the point query
    /// arrive with ticket 10.
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
        self.layers.union_damage_into(&mut self.frame);

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

        for run in &self.runs {
            self.layers.composite_run(&mut self.frame, *run);
        }

        self.packet
            .pack(&self.runs, &self.frame, self.layers.interner());
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

    /// The handle space this screen's layers speak, for the terminal model to intern into.
    #[cfg(test)]
    pub(crate) fn interner_mut(&mut self) -> &mut crate::intern::Interner {
        self.layers.interner_mut()
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

    #[test]
    fn a_default_config_is_the_declared_size() {
        let (screen, _wake) = Engine::new(Config::default()).attach().unwrap();
        assert_eq!(screen.size(), Config::DEFAULT_SIZE);
    }
}
