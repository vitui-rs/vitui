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
        let mut screen = Screen {
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
            repaint: false,
            tty,
            #[cfg(test)]
            sweeps: 0,
            _not_send: PhantomData,
        };
        screen.begin_session();
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
/// let config = vitui_engine::Config {
///     // Headless, because a doctest must not reach for the developer's terminal.
///     output: vitui_engine::Output::Sink(Box::new(Vec::new())),
///     ..Default::default()
/// };
/// let (screen, _wake) = vitui_engine::Engine::new(config).attach().unwrap();
/// std::thread::spawn(move || screen.size());
/// ```
///
/// ```
/// let (screen, _wake): (vitui_engine::Screen, vitui_engine::WakeHandle) =
///     vitui_engine::Engine::new(vitui_engine::Config {
///         output: vitui_engine::Output::Sink(Box::new(Vec::new())),
///         ..Default::default()
///     })
///     .attach()
///     .unwrap();
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
    /// Whether a sweep has renumbered a handle table since the last packet went out.
    ///
    /// Latched here rather than passed straight through, because the sweep and the frame are not the
    /// same event: the app thread may sweep twice, or not at all, between two `present` calls, and
    /// what the render thread has to be told is only that the handles moved at some point since it
    /// last recorded any. Cleared when a packet carrying it is actually submitted — an idle
    /// `present` submits nothing and must not consume it.
    repaint: bool,
    /// The pty detection read its answers from, held rather than dropped so that **one thread ever
    /// reads this file descriptor**. Ticket 20's input thread adopts this channel; a second reader
    /// would steal bytes from the first. `None` whenever there is no terminal.
    #[allow(dead_code)]
    tty: Option<Tty>,
    /// How many times [`Screen::layers`] has swept. Read by the gate that says `present` never does.
    #[cfg(test)]
    sweeps: u32,
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
    ///
    /// # This is also where the handle tables are swept, and it is the only such place
    ///
    /// Spec §3 puts eviction *where allocation is already permitted — a scene topology change, or a
    /// high-water mark on the table — and **never inside a frame***. This door is that place, and
    /// the reason it is the right one is that it is the only door those two things arrive through:
    /// every topology change is a method on [`LayerStack`], every drawing verb is reached through
    /// [`LayerStack::view`], and **there is no way to change an operator's `Mix` except to remove
    /// the layer and add it again** — so an animating operator, which is the one shape that grows a
    /// table without bound, passes through here on every frame it animates.
    ///
    /// What is checked is the high-water mark, not the topology change: sweeping on every
    /// `add_content` would spend 82.5 µs of a screen, or 897 µs of twenty layers, to reclaim
    /// whatever one layer's arrival happened to orphan. The mark is
    /// [`LayerStack::sweep_due`](LayerStack) — two loads and two compares — and this door is where
    /// it is legal to answer yes.
    ///
    /// # What "never inside a frame" does and does not mean here
    ///
    /// **`present` does not call this**, and that is the property rather than an accident: a future
    /// change that reached the stack from inside `present` would have to route around this method,
    /// and `crate::gates::the_sweep_never_runs_inside_present` says so as a count.
    ///
    /// It is worth being exact about what that buys, because this door is not only the topology
    /// door — it is the **drawing** door, `layers().view(id)`, and spec §12 has the runtime bring a
    /// draw for every layer every frame. So the sweep does land inside the application's frame loop.
    /// What it never lands inside is `present`, which is the composite-pack-serialise path §13's
    /// 100 µs and 1 ms budgets are taken around and the path that becomes the *render thread's* at
    /// ticket 18.
    ///
    /// That is the whole of what §3 asks for, and §3 says so itself one line further on: *the app
    /// thread may sweep while the render thread is inside a 200 ms `write`.* A sweep on the app
    /// thread, concurrent with the render thread's frame, is the shipped design rather than a
    /// concession — and while there is one thread there is no moment that is outside a frame in any
    /// stronger sense than this one.
    pub fn layers(&mut self) -> &mut LayerStack {
        if self.layers.sweep_due() {
            // The frame goes in with the layers: its cells were copied out of them and name the
            // same tables. See `crate::sweep`.
            let swept = self.layers.sweep_with(&mut self.frame);
            self.repaint |= swept.renumbered;
            #[cfg(test)]
            {
                self.sweeps += 1;
            }
        }
        &mut self.layers
    }

    /// Write the bytes that hold for the whole session rather than for one frame.
    ///
    /// **Auto-wrap off, once, and it is a decision the spec states rather than an implementation
    /// detail** (§8). It is worth ten bytes of every frame, which matters only because a caret blink
    /// is a 29-byte frame — and it deletes both of cellbuf's bug-driven workarounds outright, because
    /// with no wrap there is no pending-wrap state and no bottom-right corner that scrolls. **Neither
    /// workaround is ported.** The cost of refusing auto-wrap is that a run ending at the right
    /// margin can no longer flow into the next row's, and §8 measured that at *zero*: 73 290 bytes
    /// either way, because damage already splits at row boundaries.
    ///
    /// # Why this is the alt screen's prologue without an alt screen in it
    ///
    /// §8 says *for the lifetime of the alt screen*, and **ticket 22 owns the alt screen** — entering
    /// it, the panic hook, and a restoration that is idempotent under both. So this pair of methods
    /// is the two points that ticket adds `?1049h` and `?1049l` to, and auto-wrap is here now because
    /// impl 13 is what makes the serializer depend on it: every `shortest` move is priced on the
    /// assumption that nothing wrapped, and a serializer that assumed it without asking for it would
    /// be right on most terminals and silently wrong on one.
    fn begin_session(&mut self) {
        write_frame(&mut *self.sink, DISABLE_AUTO_WRAP);
    }

    /// Give back what [`begin_session`](Screen::begin_session) took.
    ///
    /// A `Drop` rather than a method, for the reason `Tty`'s own `Drop` states: a restoration that
    /// happens only when somebody remembers to ask for it is a restoration that does not happen when
    /// `attach` fails, when a `?` propagates, or when the process is unwinding. This is the floor and
    /// not the design — **ticket 22 owns shutdown** — and it lives here because this is what changed
    /// the mode.
    fn end_session(&mut self) {
        write_frame(&mut *self.sink, ENABLE_AUTO_WRAP);
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
            *run = self.layers.composite_run(&mut self.frame, *run, &self.caps);
        }
        merge_touching(&mut self.runs);

        // Spent here and nowhere else, which is the whole of why the flag is latched on the screen
        // rather than passed down from the sweep: everything above this line can return early, and a
        // `repaint` spent on a frame that never went out is a mirror that is never told.
        self.packet.pack(
            &self.runs,
            &self.frame,
            self.layers.tables(),
            std::mem::take(&mut self.repaint),
        );
        let bytes = self.serializer.serialize(&self.packet, &self.caps);
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
    /// # The mirror starts unknown, which is what the debt here used to be
    ///
    /// A terminal that has just changed size is showing something nobody recorded — it reflows on
    /// `SIGWINCH`, it does not clear — so the honest value for the mirror is ADR 0006's **unknown**
    /// state. Until ticket 08 there was no such thing and a fresh `Mirror` said *blank* instead, which
    /// was a claim about the terminal that is not true; it was harmless only because every cell of the
    /// new screen is damaged and therefore written unconditionally, and impl 14's equality filter is
    /// what ended that.
    ///
    /// A fresh `Mirror` knows nothing, and this function makes a fresh one. So there is no separate
    /// resize mode and nothing here to remember. What that is worth is pinned by
    /// [`a_resize_does_not_let_the_filter_trust_a_fresh_mirror`](tests::a_resize_does_not_let_the_filter_trust_a_fresh_mirror),
    /// whose terminal model keeps its cells the way a real terminal keeps them.
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
    /// let config = vitui_engine::Config {
    ///     // Headless, because a doctest must not reach for the developer's terminal.
    ///     output: vitui_engine::Output::Sink(Box::new(Vec::new())),
    ///     ..Default::default()
    /// };
    /// let (mut screen, _wake) = vitui_engine::Engine::new(config).attach().unwrap();
    /// let a = screen.link("https://example.com/");
    /// assert_eq!(a, screen.link("https://example.com/"));
    /// assert_ne!(a, vitui_engine::LinkId::NONE);
    /// ```
    pub fn link(&mut self, uri: &str) -> LinkId {
        self.layers.tables_mut().link(uri)
    }

    /// The handle space this screen's layers speak, for the terminal model to mint back into.
    ///
    /// **All three tables, not the interner alone.** Ticket 06 needed one, because a cluster was the
    /// only handle a cell carried that the model had to agree about; impl 13 puts an underline colour
    /// and a URI on the wire, so the model has to reach the extended-style and link tables too — a
    /// cell is compared whole, handle included, and two tables cannot produce equal handles for one
    /// entry.
    #[cfg(test)]
    pub(crate) fn tables_mut(&mut self) -> &mut crate::tables::Tables {
        self.layers.tables_mut()
    }

    /// The handle space this screen's layers speak, for a golden to resolve a frame's handles
    /// through. The frame surface carries tables of its own and they are empty: it is composited
    /// into, never drawn into.
    #[cfg(test)]
    pub(crate) fn tables(&self) -> &crate::tables::Tables {
        self.layers.tables()
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

    /// What §10's `CHA`-after-non-ASCII rule has cost this screen in bytes, for the report that pays
    /// spec §15's second owed measurement. See `crate::serial::Serializer::cha_rule_bytes`.
    #[cfg(test)]
    pub(crate) fn cha_rule_bytes(&self) -> usize {
        self.serializer.cha_rule_bytes()
    }

    /// How many mark-and-compact sweeps this screen has run.
    #[cfg(test)]
    pub(crate) fn sweeps(&self) -> u32 {
        self.sweeps
    }

    /// Whether a sweep has renumbered a table and no packet has carried the news yet.
    ///
    /// Read by [`crate::testing::Harness`], and it exists because the harness's own staleness window
    /// started one event too late: it began when a `repaint` packet **crossed**, and the handles move
    /// when the sweep **runs**. Between the two there can be any number of idle `present` calls, and
    /// on each of them the terminal model holds the handles that were current when the bytes went out
    /// while the frame holds the ones the sweep moved them to. Comparing those two compares two
    /// spellings of one screen, which is the whole thing `Packet::repaint` is about.
    #[cfg(test)]
    pub(crate) fn repaint_pending(&self) -> bool {
        self.repaint
    }

    /// Sweep now, whether or not the high-water mark says to.
    ///
    /// The gates' door, and it is the same one [`Screen::layers`] uses: a gate about what a sweep
    /// does to a *screen* must not have to grow a table past a threshold first, because then it
    /// would be a test of the threshold. The threshold has tests of its own.
    #[cfg(test)]
    pub(crate) fn sweep_now(&mut self) -> crate::sweep::Swept {
        let swept = self.layers.sweep_with(&mut self.frame);
        self.repaint |= swept.renumbered;
        self.sweeps += 1;
        swept
    }

    /// How many entries each swept table holds, clusters first.
    #[cfg(test)]
    pub(crate) fn table_lengths(&self) -> (usize, usize) {
        let tables = self.layers.tables();
        (tables.interner.len(), tables.exts.len())
    }

    /// How many rows of the mirror record what the terminal is showing.
    ///
    /// The count `Packet::repaint` is about: a full repaint expresses itself as *every row unknown*
    /// rather than as a mode, so the gate on it is this number reaching zero.
    #[cfg(test)]
    pub(crate) fn known_rows(&self) -> usize {
        (0..self.size.1)
            .filter(|&y| self.serializer.mirror().is_known(y))
            .count()
    }

    /// How many extended-style entries this screen has ever created, sweeps included.
    ///
    /// Not the same question as [`Screen::table_lengths`], and spec §3's growth table is made of this
    /// one: a table that gains 96 entries a frame and is swept back to size every third frame has a
    /// length delta of about nothing while creating 96 a frame.
    #[cfg(test)]
    pub(crate) fn extended_styles_minted(&self) -> u64 {
        self.layers.tables().exts.minted()
    }

    /// Whether a sweep would run at the next [`Screen::layers`].
    ///
    /// Read by the gate that says `present` never sweeps, so that the gate is about `present`
    /// refusing rather than about the mark not having been reached.
    #[cfg(test)]
    pub(crate) fn sweep_due(&self) -> bool {
        self.layers.sweep_due()
    }

    /// Every cell of the frame and of every layer surface, with every handle resolved.
    ///
    /// Register entry #11's oracle. It takes `&self` deliberately: reaching the layers through
    /// [`Screen::layers`] would run a sweep on the way past, and a snapshot that swept before
    /// snapshotting is not a *before*.
    #[cfg(test)]
    pub(crate) fn channels(&self) -> Vec<crate::sweep::Channels> {
        let tables = self.layers.tables();
        let mut out = Vec::new();
        crate::sweep::channels_of(&self.frame, tables, &mut out);
        for layer in self.layers.as_stored() {
            if let Some(surface) = layer.surface() {
                crate::sweep::channels_of(surface, tables, &mut out);
            }
        }
        out
    }

    /// Serialise this screen's frames under one of the filter configurations that lost.
    ///
    /// The instrument spec §8's *there is no threshold* is reproduced with, and it is on `Screen`
    /// because the scenes are driven through `present`: a sweep over thresholds has to be a sweep
    /// over the same twelve scenes the byte budget is measured on, or it is a sweep over a fixture
    /// somebody chose. See [`crate::serial::Filter`].
    #[cfg(test)]
    pub(crate) fn set_filter(&mut self, filter: crate::serial::Filter) {
        self.serializer.set_filter(filter);
    }

    #[cfg(test)]
    pub(crate) fn mirror(&self) -> &crate::serial::Mirror {
        self.serializer.mirror()
    }

    /// The whole stack, composited the slow obvious way: gate #1's oracle, over this screen.
    ///
    /// It lives here rather than being called on [`Screen::layers`] because the oracle needs the
    /// layer stack **and** the capabilities — an operator resolves colour against what the terminal
    /// answered (ADR 0025) — and those are two fields of this struct. A caller that reached for both
    /// itself would be holding one borrow of `self` mutably and another immutably; the split belongs
    /// where the fields are.
    #[cfg(test)]
    pub(crate) fn reference(&mut self) -> Surface {
        let (w, h) = self.size;
        crate::reference::composite(&mut self.layers, &self.caps, w, h)
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
/// DECAWM off. Once, on entering the alt screen (spec §8).
const DISABLE_AUTO_WRAP: &[u8] = b"\x1b[?7l";
/// DECAWM on, which is what the terminal had before this process took it.
const ENABLE_AUTO_WRAP: &[u8] = b"\x1b[?7h";

impl Drop for Screen {
    fn drop(&mut self) {
        self.end_session();
    }
}

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

    /// A `Config` that is the default in every way **except that it reaches for no terminal**.
    ///
    /// This exists because of a real defect, and the defect is worth stating so nobody puts
    /// `Config::default()` back. `Output::Terminal` is the default, and `attach` on it now performs
    /// live detection against the process's own tty — so three tests here were doing exactly that:
    /// under `cargo test` in a real terminal they enabled raw mode, wrote the 277-byte batch onto
    /// the developer's screen, ate their keystrokes, and failed with `NoAnswer`. **CI has no tty, so
    /// the pipeline stayed green and only humans saw it.**
    ///
    /// The general rule this is an instance of: *a test that reaches for real I/O passes in the
    /// environment that has none.*
    fn headless() -> Config {
        Config {
            output: Output::Sink(Box::new(Vec::new())),
            ..Config::default()
        }
    }

    /// The guard that makes the defect above structural rather than remembered.
    ///
    /// `Tty::open` panics under `cfg(test)`, so a unit test that reaches for the real terminal fails
    /// with a message naming the fix instead of quietly working on the machine with no tty. This is
    /// the positive twin of that: it proves the guard is armed, because a guard nothing exercises is
    /// indistinguishable from one that was removed.
    ///
    /// What it cannot cover is a **doctest** — those compile against the crate as a dependency,
    /// without `cfg(test)`, so the guard is invisible to them. `.gitlab-ci.yml` runs the whole suite
    /// a second time under a pty for that half.
    #[test]
    #[should_panic(expected = "would query the developer's real terminal")]
    fn a_test_that_reaches_for_the_real_terminal_fails_loudly() {
        let _ = Engine::new(Config::default()).attach();
    }

    #[test]
    fn a_wake_handle_records_a_post_and_a_quit_separately() {
        let (_screen, wake) = Engine::new(headless()).attach().unwrap();
        assert_eq!(wake.pending(), 0);
        wake.post();
        assert_eq!(wake.pending(), WakeHandle::POSTED);
        wake.quit();
        assert_eq!(wake.pending(), WakeHandle::POSTED | WakeHandle::QUIT);
    }

    #[test]
    fn a_wake_handle_clone_shares_the_same_flags() {
        let (_screen, wake) = Engine::new(headless()).attach().unwrap();
        let other = wake.clone();
        other.post();
        assert_eq!(wake.pending(), WakeHandle::POSTED);
    }

    #[test]
    fn a_screen_takes_its_size_from_the_config() {
        let config = Config {
            size: (120, 40),
            ..headless()
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

    /// **The resize the equality filter has to reach, and the pretence that used to hide it.**
    ///
    /// A terminal reflows on `SIGWINCH`; it does not clear. So after a resize the screen is showing
    /// content nobody recorded, a fresh `Mirror` knows nothing, and **every cell of the new frame has
    /// to go out even where the frame's own value is a blank.** Before impl 14 that happened for a
    /// reason that was about to stop being true — every cell of a resized screen was written
    /// unconditionally because nothing compared anything — and the mirror said *blank* where it should
    /// have said *unknown*.
    ///
    /// The fixture is the smallest one that can tell the two apart. A layer covers the whole 8x3
    /// screen and paints it; then the layer shrinks to four columns and the terminal grows a row. The
    /// new frame wants columns 4..8 blank, a fresh mirror believes they are blank, and the terminal is
    /// still showing `#` in them. A filter that trusted the mirror would skip all twelve of those
    /// cells and leave them on screen.
    ///
    /// **`Harness::resize` is the other half of this test.** It used to replace the terminal model
    /// with a fresh blank one, which agreed with a fresh mirror by construction — so this fixture
    /// would have passed whatever the serializer did. The model keeps its cells now, and the
    /// assertions inside `Harness::present` are what fail.
    #[test]
    fn a_resize_does_not_let_the_filter_trust_a_fresh_mirror() {
        let mut h = crate::testing::Harness::new(8, 3);
        let id = h
            .screen
            .layers()
            .add_content(0, crate::geom::Rect::new(0, 0, 8, 3), true);
        let paint = |screen: &mut Screen, w: u16| {
            screen
                .layers()
                .view(id)
                .expect("the layer is still there")
                .fill(
                    crate::geom::Rect::new(0, 0, w, 3),
                    "#",
                    crate::style::Style::new(),
                );
        };
        paint(&mut h.screen, 8);
        h.present();
        assert_eq!(
            h.terminal_glyph(5, 0),
            Some('#'),
            "the fixture needs the terminal to be showing something at (5, 0)"
        );

        // The layer keeps its cells across a resize and **not** across a `set_rect`, which
        // reallocates at the new size (ticket 10) — so this is a shrink and a repaint, which is what
        // a component whose window narrowed actually does.
        h.screen
            .layers()
            .set_rect(id, crate::geom::Rect::new(0, 0, 4, 3));
        paint(&mut h.screen, 4);
        h.resize(8, 4);

        // `Harness::present` is the assertion: it replays the frame's bytes into a terminal model that
        // still holds the old `#`s and requires the result to equal the composited frame.
        assert!(h.present().submitted, "a resize repaints the whole screen");
        assert_eq!(
            h.terminal_glyph(5, 0),
            Some(' '),
            "the column the layer gave up was never cleared on the terminal"
        );
        assert_eq!(
            h.terminal_glyph(3, 0),
            Some('#'),
            "and the layer still paints"
        );
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
            let interner = &mut h.screen.tables_mut().interner;
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

    /// **Gate, both directions: auto-wrap is switched off once and given back.**
    ///
    /// §8 makes this a decision rather than an implementation detail, and both halves have a way of
    /// going missing separately — `Tty`'s own `Drop` exists because mode 2027 was being set and never
    /// reset. So the sequence is asserted as a *pair*, and asserted through the terminal model rather
    /// than as two byte strings, because what matters is the state the terminal is left in.
    #[test]
    fn auto_wrap_is_disabled_once_on_entry_and_restored_on_leaving() {
        let sink = crate::testing::Recorder::new();
        let recording = sink.handle();
        {
            let (mut screen, _wake) = Engine::new(Config {
                size: (8, 1),
                output: Output::Sink(Box::new(sink)),
                clock: Clock::Manual,
                overrides: Overrides::default(),
            })
            .attach()
            .expect("attaching to a sink cannot fail");

            let prologue = recording.lock().unwrap().bytes.clone();
            assert_eq!(prologue, DISABLE_AUTO_WRAP, "before any frame");

            // And **once**, not once per frame: ten bytes of every frame is what §8 prices this at.
            let id = screen
                .layers()
                .add_content(0, crate::geom::Rect::new(0, 0, 8, 1), true);
            screen
                .layers()
                .view(id)
                .unwrap()
                .text(0, 0, "a", crate::style::Style::new());
            screen.present();
            let after = recording.lock().unwrap().bytes.clone();
            assert_eq!(
                after
                    .windows(DISABLE_AUTO_WRAP.len())
                    .filter(|w| *w == DISABLE_AUTO_WRAP)
                    .count(),
                1,
                "one DECAWM reset for the session, not one per frame"
            );
        }

        // The screen is gone, so the epilogue has been written. Replayed as a whole session, the
        // model ends with auto-wrap back on — which is the state the user's shell had before this
        // process took it.
        let bytes = recording.lock().unwrap().bytes.clone();
        let mut term = crate::term_model::TermModel::new(8, 1);
        let mut tables = crate::tables::Tables::new();
        term.feed(&bytes, &mut tables);
        assert_eq!(term.unrecognised(), 0, "every byte of the session parses");
        assert!(term.autowrap(), "restored on leaving");
        assert!(
            bytes.ends_with(ENABLE_AUTO_WRAP),
            "and it is the last thing said"
        );
    }

    /// Neither of cellbuf's two bug-driven workarounds is ported, and the reason is that with
    /// auto-wrap off neither has a subject: there is no pending-wrap state and no bottom-right
    /// corner that scrolls.
    ///
    /// The claim is executed rather than asserted: a full row written to the last column, and the
    /// row below it left alone.
    #[test]
    fn writing_the_last_column_of_the_last_row_wraps_and_scrolls_nothing() {
        let mut h = crate::testing::Harness::new(6, 2);
        let id = h
            .screen
            .layers()
            .add_content(0, crate::geom::Rect::new(0, 0, 6, 2), true);
        h.screen
            .layers()
            .view(id)
            .unwrap()
            .text(0, 1, "abcdef", crate::style::Style::new());
        // `Harness::present` is the round trip, so this asserts the replayed screen and the mirror
        // both equal the frame — including the row above, which a scroll would have moved.
        assert!(h.present().submitted);
    }

    /// The size comes from the terminal when there is one to ask, and from `Config` when there is
    /// not. A sink is the second case, and it is the one every test in this crate is on.
    #[test]
    fn a_default_config_is_the_declared_size() {
        let (screen, _wake) = Engine::new(headless()).attach().unwrap();
        assert_eq!(screen.size(), Config::DEFAULT_SIZE);
    }
}
