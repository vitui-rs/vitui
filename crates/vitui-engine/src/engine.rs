//! The lifecycle: `Engine`, `Screen`, and the one exit that is `present`.
//!
//! # Scope
//!
//! [`Config::clock`] decides which of two paths `present` takes, and **only who runs the
//! serializer differs**:
//!
//! | | composite and pack | serialise and write |
//! |---|---|---|
//! | [`Clock::Manual`] | the calling thread | the calling thread, inline, before `present` returns |
//! | [`Clock::System`] | the calling thread | the render thread, out of the mailbox |
//!
//! Both go through the same mailbox, the same pool of two packets and the same [`Renderer`]. That is
//! not a tidiness: it is what makes the deterministic mode exercise the handoff rather than bypass
//! it, and it is why nothing this ticket added can change a byte the deterministic mode asserts.
//!
//! The deterministic mode is **public API rather than test scaffolding** (spec §14): an application
//! author testing their own UI needs the same determinism the engine's tests need. A test is a
//! straight-line program — draw, present, assert on the sink — with no condvar, no join, no timeout
//! and no flake.
//!
//! The parking point is here since ticket
//! 19: [`Screen::wait`] is the app thread's only blocking call, the frame clock gates it rather than
//! `present` (ADR 0004), and *the renderer is free* reaches it through [`crate::clock::WakeSource`]
//! rather than through the mailbox's own condvar — one thread cannot park on two of them.
//!
//! # The three threads, and what each one owns
//!
//! | thread | blocks in | owns |
//! |---|---|---|
//! | **app** | one condvar | the layer stack, the canonical composite, the handle tables, the pool |
//! | **render** | the mailbox condvar | the terminal's **write** direction, exclusively, and the mirror |
//! | **input** | `read` on the tty | the terminal's **read** direction; writes the authoritative size |
//!
//! **No timer thread.** An animation is a `wait_timeout` on a condvar the app thread already owns.
//!
//! Terminal setup — raw mode, the alt screen and §10's capability queries, which are the only place
//! the engine both writes *and* reads — happens entirely inside [`Engine::attach`] and **before the
//! render thread exists**, where no concurrency does.
//!
//! The reader is the one this crate had first: [`Tty::open`] spawns it, because detection has to
//! read raw bytes with a deadline and **one thread ever reads that file descriptor**. `attach` hands
//! its channel to the input thread, which parses, writes [`TerminalSize`] and queues an
//! [`Event::Resize`] — the app thread samples the size at frame start, re-checks it at submit, and
//! discards a composite that was built for a screen that no longer exists.
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
use std::thread::JoinHandle;
use std::time::Instant;

use crate::actuate::{Actuators, Cursor};
use crate::caps::{Capabilities, Env, Ground, Overrides, assemble};
use crate::clock::{FrameClock, Wake, WakeSource};
use crate::damage::Run;
use crate::detect::{CEILING, Tty, detect};
use crate::exts::LinkId;
use crate::handoff::{Lease, Mailbox, TerminalSize};
use crate::input::{Event, InputConfig, InputDiagnostics, MouseMode};
use crate::layer::LayerStack;
use crate::packet::Packet;
use crate::quirks::Quirks;
use crate::serial::Serializer;
use crate::shutdown::Site;
use crate::surface::Surface;

/// Where the frame clock takes its time from.
///
/// A concrete enum, never a trait, because the seam has no traits in it and a trait object here
/// would be the first. `std::time::Instant` has no constructor from a number — `from_nanos`,
/// `From<Duration>` and `default` all fail to compile — so the manual clock is one captured
/// `Instant` plus an offset, which is why no public signature changes and why nobody needs a
/// `vitui::Timestamp` newtype to make testing possible.
///
/// **This is what decides whether a render thread exists.** [`Engine::attach`] spawns one on
/// [`Clock::System`] and none on [`Clock::Manual`], where `present` runs the whole round inline
/// before it returns. The two paths share the mailbox, the pool and the serializer, so the bytes are
/// the same bytes; what differs is which thread produced them and when.
///
/// A test therefore pins `Clock::Manual`, and so does anything measuring the inline round — a timing
/// taken on the threaded path is the app thread's share of a frame, which is a different number and
/// a better one.
///
/// It is also what decides whether the frame clock **paces** anything. `Manual` is unpaced, which is
/// the promise this variant was already carrying — *time only moves when the caller moves it* — and it
/// makes `Manual` reproducible in its timing as well as in its interleaving. A registered deadline
/// still holds on that path: an `Instant` the caller chose is the caller's own clock, not this one's.
/// [`Screen::set_max_frame_rate`] is **ignored** on this clock. Not rejected — there is nothing in
/// §12's signature to reject with, and a `debug_assert` would fire on a caller doing something
/// perfectly reasonable: setting the ceiling once at startup and choosing the clock elsewhere. So it
/// returns having done nothing, and this sentence is where that is written down.
///
/// **[`Screen::wait`] still blocks on this clock**, and it has to: what `Manual` removes is the
/// *pacing*, not the parking. A deterministic test does not call it — there is no thread to be woken
/// by, so an unpaced indefinite wait is an indefinite wait — and a deterministic *application* posts
/// through its [`WakeHandle`] from wherever its work happens, exactly as it would on `System`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Clock {
    /// Real time, and a render thread that owns the write direction.
    #[default]
    System,
    /// Time only moves when the caller moves it, and there is no thread to interleave with.
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
    /// The process's standard output, on the **alternate screen**: `begin_session` enters it before
    /// a frame exists and the restoration leaves it, so a full-screen application writing here never
    /// touches the user's scrollback.
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
    /// The ceiling on frames per second, in hertz.
    ///
    /// **Set by the application, never discovered.** A tty cannot report a refresh rate, so nothing
    /// in this crate asks a display anything — §12's refusal 10, and
    /// `crate::gates::nothing_anywhere_queries_a_display` is what keeps it true. The application
    /// learns the number from the platform and says so here;
    /// [`Screen::set_max_frame_rate`] covers a monitor changing under a running program.
    ///
    /// It is a **minimum gap and not a tick** (ADR 0004): the first damage after a quiet period
    /// paints immediately and everything arriving inside the gap coalesces into one frame at the end
    /// of it. The gate sits on [`Screen::wait`], so a frame nobody will see costs neither a composite
    /// nor a layout.
    ///
    /// `f32::INFINITY` is unlimited, and so is any value at or below zero — *never paint* is a frozen
    /// application and not a configuration worth being able to reach by arithmetic.
    ///
    /// Ignored on [`Clock::Manual`], where nothing is paced — and so is
    /// [`Screen::set_max_frame_rate`] for the life of such a screen.
    pub max_frame_rate: f32,
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
    /// What the terminal is switched on for, as a **floor** rather than a setting.
    ///
    /// See [`InputConfig`]. Every field is load-bearing: the escape sequences that ask a terminal
    /// for mouse tracking, focus reporting and bracketed paste go out at `attach`, before the render
    /// thread exists and before anything has drawn — which is what makes the mouse level a *floor*
    /// rather than a setting, since the union of what the frame's components want is known only after
    /// a draw. See [`crate::actuate::negotiation`].
    pub input: InputConfig,
}

impl Config {
    /// The size a terminal that has never been asked is assumed to be.
    const DEFAULT_SIZE: (u16, u16) = (80, 24);

    /// The ceiling an application that has not said anything gets: spec §7's `MIN_GAP` of 16.6 ms.
    ///
    /// A default rather than a discovery, and the two are not close: the engine cannot ask, so the
    /// alternative to a default is refusing to start.
    const DEFAULT_MAX_FRAME_RATE: f32 = 60.0;
}

impl Default for Config {
    fn default() -> Config {
        Config {
            clock: Clock::default(),
            max_frame_rate: Config::DEFAULT_MAX_FRAME_RATE,
            output: Output::default(),
            size: Config::DEFAULT_SIZE,
            overrides: Overrides::default(),
            input: InputConfig::default(),
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
        self.attach_as(None, None)
    }

    /// Attach with the capabilities handed in rather than detected.
    ///
    /// **The door arch 22 named for the axes it refused a field**, one ticket further along than it
    /// expected: impl 21's actuator and negotiation read the eight *input* facts, and every arm of
    /// both is unreachable from a caller-supplied sink, where nothing is detected and all eight are
    /// false. `Overrides` may not carry them — *a declaration cannot make an event arrive*, ADR 0007
    /// — so the arms come from inside the crate, exactly as `sync_output` and the ConPTY underline
    /// form already do. See [`Capabilities::with_input`](crate::Capabilities).
    #[cfg(test)]
    pub(crate) fn attach_declaring(
        self,
        caps: Capabilities,
    ) -> Result<(Screen, WakeHandle), AttachError> {
        self.attach_as(Some(caps), None)
    }

    /// Attach with somewhere for the **panic** path's restoration to go.
    ///
    /// The ordinary epilogue goes through the renderer's sink, which a headless gate already holds a
    /// handle to. The panic path cannot: it may be the render thread's own unwind, so it writes
    /// through [`crate::shutdown::Site`]'s sink — `std::io::stdout()` for a real terminal, and
    /// nowhere at all for a caller-supplied one, because a file is not a terminal and has no modes
    /// to give back. This door swaps that one field, so that register entry #15 can be **captured**
    /// rather than inspected.
    #[cfg(test)]
    pub(crate) fn attach_restoring_into(
        self,
        declared: Option<Capabilities>,
        restore: Box<dyn Write + Send>,
    ) -> Result<(Screen, WakeHandle), AttachError> {
        self.attach_as(declared, Some(restore))
    }

    fn attach_as(
        self,
        declared: Option<Capabilities>,
        restore: Option<Box<dyn Write + Send>>,
    ) -> Result<(Screen, WakeHandle), AttachError> {
        let env = Env::from_process();
        let headless = matches!(self.config.output, Output::Sink(_));
        // Declared capabilities and a real terminal are a combination with no honest meaning: the
        // batch would go out, the terminal would answer, and the answer would be thrown away — which
        // is a test that eats the developer's keystrokes to learn nothing. The door is for a sink.
        debug_assert!(
            declared.is_none() || headless,
            "capabilities may only be declared over a caller-supplied sink"
        );
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
        let caps = declared
            .unwrap_or_else(|| assemble(self.config.overrides, &env, ground, &detected, quirks));

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
        // **The one thing that can wake the app thread**, minted here so that both halves of the
        // split — `Screen`'s `wait` and `WakeHandle`'s `post` — are the same source. ADR 0003's
        // split handles are internal after ticket 12: what stays public is `Screen` (`!Send`) and
        // `WakeHandle` (`Send + Sync + Clone`, two verbs).
        let wakes = Arc::new(WakeSource::new());
        let terminal_size = Arc::new(TerminalSize::new((w, h)));
        let input = Arc::new(crate::input::Queue::new());
        let paste_limit = self.config.input.paste_limit;
        // **Where the terminal's restoration lives, and it outlives the `Screen` on purpose.** A
        // panic hook runs on whichever thread panicked, so what gives the terminal back cannot be a
        // method on a `!Send` type the app thread owns — it is one atomic and the two things the
        // epilogue is computed from. See `crate::shutdown`.
        let mailbox = Arc::new(Mailbox::new());
        let shutdown = Site::new(
            self.config.input,
            caps.clone(),
            restore.or_else(|| match headless {
                // A caller-supplied sink is not a terminal: nothing pushed a kitty flag at a file,
                // and the epilogue the ordinary path writes into it is a courtesy rather than a
                // rescue. The panic path has no second handle to it and invents none — and,
                // crucially, does not claim to have restored anything either.
                true => None,
                // The same descriptor every frame goes to. `std::io::stdout()` is a handle to one
                // process-global, internally locked stream, so this is not a second writer.
                false => Some(Box::new(std::io::stdout()) as Box<dyn Write + Send>),
            }),
            // Whether the negotiation below is about to switch a tracking mode on, which is the only
            // thing about the mouse the restoration needs. See `crate::shutdown::Site::mouse`.
            Actuators::new(&self.config.input, &caps).mouse() != MouseMode::Off,
            Arc::clone(&mailbox),
        );
        crate::shutdown::arm(&shutdown);
        let renderer = Renderer {
            serializer: Serializer::new(w, h),
            size: (w, h),
            sink: match self.config.output {
                Output::Terminal => Box::new(std::io::stdout()),
                Output::Sink(sink) => sink,
            },
            caps: caps.clone(),
        };
        let mut screen = Screen {
            size: (w, h),
            layers: LayerStack::new(),
            frame: Surface::new(w, h),
            runs: Vec::with_capacity(h as usize * 4),
            mailbox,
            // Nothing is spawned here. The prologue below writes through this sink, and the render
            // thread cannot exist before the terminal is set up — that is the whole of why setup
            // happens where no concurrency does.
            renderer: Some(renderer),
            render: None,
            clock: self.config.clock,
            // **The gap belongs to the app thread and the wake source belongs to everyone**, which
            // is why they are two fields rather than one: `next_allowed` is read on every `wait` and
            // written on every submit, both on this thread, so it never needs a lock.
            frame_clock: FrameClock::new(
                self.config.max_frame_rate,
                self.config.clock == Clock::System,
            ),
            wakes: Arc::clone(&wakes),
            terminal_size: Arc::clone(&terminal_size),
            input: Arc::clone(&input),
            generation: 0,
            coalesced: 0,
            actuators: Actuators::new(&self.config.input, &caps),
            input_config: self.config.input,
            shutdown,
            caps,
            repaint: false,
            tty,
            #[cfg(test)]
            sweeps: 0,
            #[cfg(test)]
            resize_before_submit: None,
            _not_send: PhantomData,
        };
        // The one thing the handle space has to be told about the terminal, and it is told once:
        // whether a hyperlink is part of an extended style's identity. Spec §10's narrow exception —
        // a channel the terminal cannot express at all is dropped from the intern *key*, where one it
        // expresses imprecisely is degraded at serialise time. See
        // [`Tables::key`](crate::tables::Tables::key).
        let hyperlinks = screen.caps.hyperlinks;
        screen.layers.tables_mut().set_links_in_key(hyperlinks);
        screen.begin_session();
        // **Last, and after every byte of setup has gone out.** Raw mode, the query batch and the
        // prologue are the one place the engine both writes and reads, and they are finished before
        // a second thread exists.
        screen.spawn_render_thread();
        // And then the input thread, which adopts detection's reader rather than opening one of its
        // own — a second reader of the same file descriptor steals bytes from the first. Nothing is
        // spawned when there is no terminal: a headless screen has no keyboard, and a thread parked
        // on a channel nobody sends to is a thread that shows up in every process listing for ever.
        if let Some((reads, type_ahead)) = screen.tty.as_mut().and_then(Tty::take_reader) {
            crate::reader::spawn(crate::reader::Wiring {
                reads,
                type_ahead,
                queue: input,
                wakes: Arc::clone(&wakes),
                size: terminal_size,
                paste_limit,
                measure: Tty::size,
            });
        }
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
    /// How many `present` calls were folded into this one because the render thread was busy.
    ///
    /// A frame the renderer has not asked for is **not composited**: damage stays in the damage
    /// structure, which is already the coalescing mechanism, and the next frame that does composite
    /// paints all of it. So this counts calls that returned without doing any work, and the frame
    /// that submits is the one that reports and clears the count. Always zero on [`Clock::Manual`],
    /// where the renderer is the calling thread and is never busy.
    pub coalesced: u32,
    /// Whether the frame was thrown away because the terminal resized **under it**.
    ///
    /// Writing a 300x80 frame into a terminal that is now 120x40 wraps and scrolls, which is worse
    /// than a missing frame. The size is sampled at frame start and re-checked at submit, so this is
    /// true **once** per resize observed inside a frame, and not on every frame after one: the frame
    /// that follows samples the size the discarded one was refused for. The composite is lost; the
    /// lease is not — a leased surface never changes size under a drawing caller.
    ///
    /// **A resize observed between two frames is not this**, and the review is what made the
    /// distinction explicit rather than implied. The sample is taken at frame start, so a terminal
    /// that resized while the application was idle is already the sampled size and the frame submits
    /// against a mirror built for the old geometry. That frame is not discarded and does not need to
    /// be: [`Screen::next_event`] rebuilds the surfaces and schedules a full repaint on the way past
    /// the [`Event::Resize`], so the geometry has already caught up by the time anything draws. This
    /// flag is only for the window a caller cannot reach — between the sample and the submit of one
    /// frame, where there is no `next_event` to run.
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
    wakes: Arc<WakeSource>,
}

impl WakeHandle {
    /// Ask the app thread to wake and run a frame.
    ///
    /// This is the whole of how work done somewhere else enters the app thread: a worker finishes,
    /// leaves its result where the application can take it, and posts. The wake arrives as
    /// [`Wake::Posted`] and **never as a callback on the worker's own thread**, which is what keeps
    /// the reactive layer above this crate single-threaded by construction.
    ///
    /// Paced like everything else except a quit — a background result that lands 1 ms after a paint
    /// is held to the end of the gap.
    pub fn post(&self) {
        self.wakes.post();
    }

    /// Ask the app thread to stop. **Never paced.**
    ///
    /// At a 1 Hz ceiling, checking the clock before this flag hangs shutdown for a second, so
    /// [`Screen::wait`] checks it first. Writing the test is what found it.
    pub fn quit(&self) {
        self.wakes.quit();
    }

    /// The input thread parsed something.
    ///
    /// `pub(crate)` and it stays that way: §12 gives this handle two verbs, and the third is the
    /// input thread's — spawned by `attach`, never held by an application. The production caller is
    /// `crate::reader::run`, which reaches the same source without going through this handle.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn input(&self) {
        self.wakes.input();
    }

    /// The wake source itself, for the gate that reads the park counters while the app thread is
    /// parked in them. `Arc<WakeSource>` is `Send + Sync`, so it travels where `Screen` cannot.
    #[cfg(test)]
    pub(crate) fn source(&self) -> Arc<WakeSource> {
        Arc::clone(&self.wakes)
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
    /// The handoff: the slot, the pool of two, and the two condvars. Shared with the render thread
    /// when there is one, and used by both paths so that the deterministic mode exercises it.
    mailbox: Arc<Mailbox>,
    /// The renderer, while this thread is the one that runs it. `None` once a render thread owns it,
    /// and the reason it comes back is [`Screen::drop`]: the sink is in there, and the epilogue has
    /// to be written through it.
    renderer: Option<Renderer>,
    /// The render thread, which answers with the renderer when it is joined.
    render: Option<JoinHandle<Renderer>>,
    /// Which path `present` takes. Immutable for the life of this screen.
    clock: Clock,
    /// The minimum gap between two frames, and when the last one went out. **The app thread's
    /// alone**, which is why it is not behind the wake source's lock.
    frame_clock: FrameClock,
    /// The one thing that can wake this thread: an input event, a post, a deadline, or the renderer
    /// going free with a frame owed. Shared with every [`WakeHandle`] and with the render thread.
    wakes: Arc<WakeSource>,
    /// The authoritative size of the terminal: sampled at frame start, re-checked at submit.
    ///
    /// Behind an `Arc` because the **input thread is the only writer** (spec §7) and this thread is
    /// the only reader.
    terminal_size: Arc<TerminalSize>,
    /// What the input thread has parsed and this thread has not taken yet.
    input: Arc<crate::input::Queue>,
    /// How many packs have happened. Stamped into every packet, and never reused.
    generation: u64,
    /// How many `present` calls have been folded into the next frame because the renderer was busy.
    /// Reported by the one that submits, and reset by it.
    coalesced: u32,
    /// What the terminal can do, sampled once during `attach` and never again.
    caps: Capabilities,
    /// What [`Screen::set_mouse`] and [`Screen::set_cursor`] were told, and what the last submitted
    /// packet carried. See [`crate::actuate`].
    actuators: Actuators,
    /// What the terminal was switched on for at startup, kept for the epilogue that switches it off.
    ///
    /// A copy of [`Config::input`] rather than a reference to it: `Config` is consumed by `attach`,
    /// and the two questions the epilogue asks of it — *was focus reporting declared* and *was paste*
    /// — have no other home.
    input_config: InputConfig,
    /// What the terminal is owed back, and the one atomic that says whether it has had it.
    ///
    /// **Shared with the process's panic hook**, which is the whole reason it is not three fields on
    /// this struct: a hook runs on whichever thread panicked, and this type is `!Send`. `Drop`
    /// restores through it and the hook restores through it, and the atomic inside is what makes
    /// *both of those happening* still one restoration. See [`crate::shutdown`].
    shutdown: Arc<Site>,
    /// Whether a sweep has renumbered a handle table since the last packet went out.
    ///
    /// Latched here rather than passed straight through, because the sweep and the frame are not the
    /// same event: the app thread may sweep twice, or not at all, between two `present` calls, and
    /// what the render thread has to be told is only that the handles moved at some point since it
    /// last recorded any. Cleared when a packet carrying it is actually submitted — an idle
    /// `present` submits nothing and must not consume it.
    repaint: bool,
    /// The pty detection read its answers from, held rather than dropped so that **one thread ever
    /// reads this file descriptor**. The input thread has adopted its channel; what is left here is
    /// the `Drop` that gives back raw mode and mode 2027. `None` whenever there is no terminal.
    #[allow(dead_code)]
    tty: Option<Tty>,
    /// How many times [`Screen::layers`] has swept. Read by the gate that says `present` never does.
    #[cfg(test)]
    sweeps: u32,
    /// A size for the *next* frame to find moved under it, applied between the pack and the submit.
    ///
    /// **The interleaving this exists for cannot be produced from outside the frame.** `present`
    /// samples the size as its first statement and re-checks it at submit, and there is no instant
    /// between those two at which a caller of this crate is running: the drawing verbs are before,
    /// the sink is after. A second thread can be made to land in the window nearly always and never
    /// certainly, and *nearly always* in a gate is a flake with a budget's clothes on.
    ///
    /// So the input thread's one write is simulated where the input thread would have made it. Two
    /// lines in the frame path, in the same shape as [`Screen::sweeps`] beside it.
    #[cfg(test)]
    resize_before_submit: Option<(u16, u16)>,
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
    /// 100 µs and 1 ms budgets are taken around — and the serialise half of that is the **render
    /// thread's** since ticket 18.
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
    /// # The alt screen, and why auto-wrap is switched off *inside* it
    ///
    /// §8 says *for the lifetime of the alt screen*, and this is where that lifetime begins:
    /// `?1049h` is the first thing on the wire and every mode after it is set on the page this
    /// session owns. The restoration is the same list backwards, ending with `?1049l` — see
    /// [`crate::actuate::restoration`] and [`crate::shutdown`], which is what makes it happen under
    /// a panic as well as under this type's `Drop`.
    ///
    /// Auto-wrap is a mode rather than a page, so switching it off is not undone by leaving: it has
    /// to be given back explicitly, and **before** the page is, or the last bytes of the session
    /// reprogram the terminal behind the user's returned prompt. Every `shortest` move in the
    /// serializer is priced on the assumption that nothing wrapped (impl 13), and a serializer that
    /// assumed it without asking for it would be right on most terminals and silently wrong on
    /// one.
    fn begin_session(&mut self) {
        let bytes = crate::actuate::negotiation(&self.input_config, &self.caps);
        let sink = &mut self.inline_renderer_mut().sink;
        write_frame(&mut **sink, &bytes);
    }

    /// Give back what [`begin_session`](Screen::begin_session) took.
    ///
    /// Reached from [`Screen::drop`] and from nowhere else, for the reason `Tty`'s own `Drop`
    /// states: a restoration that happens only when somebody remembers to ask for it is a
    /// restoration that does not happen when `attach` fails, when a `?` propagates, or when the
    /// process is unwinding.
    ///
    /// **The renderer's sink when there still is one, and [`crate::shutdown`]'s otherwise.** A
    /// render thread that panicked took the sink with it, and a terminal is not left in raw mode with
    /// kitty flags pushed because the thread that owned the descriptor is the one that died — so the
    /// site's own sink is what catches that, exactly as it catches a panic on any other thread.
    ///
    /// Either way it goes through [`Site::restore`], so a `Drop` that follows a panic hook writes
    /// nothing: one atomic, one restoration, whichever arrived first.
    fn end_session(&mut self) {
        let site = Arc::clone(&self.shutdown);
        match self.renderer.as_mut() {
            Some(renderer) => site.restore(Some(&mut *renderer.sink)),
            None => site.restore(None),
        };
    }

    /// Hand the renderer to a thread of its own, if the clock says there is one.
    ///
    /// Called once, at the end of `attach`, after every byte of terminal setup has gone out.
    fn spawn_render_thread(&mut self) {
        if self.clock == Clock::Manual {
            return;
        }
        let renderer = self
            .renderer
            .take()
            .expect("attach has not handed the renderer anywhere yet");
        let mailbox = Arc::clone(&self.mailbox);
        let wakes = Arc::clone(&self.wakes);
        // **The renderer is handed over after the spawn succeeded, not moved into the closure.**
        // `Builder::spawn` does not give a failed closure back, and a renderer lost that way takes
        // the sink with it — so the thread waits for it on a channel that carries exactly one
        // message. A thread that cannot be spawned is then not a reason to fail `attach`, and not
        // silently ignored either: the renderer stays on this thread, every frame takes the inline
        // path, and that is a slower engine rather than a broken one. `AttachError` names the
        // terminal not answering; a resource limit at spawn time is the process's, not the
        // terminal's.
        let (tx, rx) = std::sync::mpsc::channel();
        match std::thread::Builder::new()
            .name("vitui-render".to_string())
            .spawn(move || {
                let mut renderer: Renderer = rx.recv().expect("attach sends before it returns");
                render_loop(&mailbox, &wakes, &mut renderer);
                renderer
            }) {
            Ok(handle) => {
                let _ = tx.send(renderer);
                self.render = Some(handle);
            }
            Err(_) => self.renderer = Some(renderer),
        }
    }

    /// The renderer, for the two writes that are not a frame and for the `cfg(test)` doors.
    ///
    /// It is `Some` on the deterministic path and `None` once a render thread owns it, so this
    /// panics rather than lying: **the mirror, the byte counters and the filter switches are the
    /// render thread's**, and a test that reaches for them from the threaded path is asking a
    /// question about a thread it is not on. Pin [`Clock::Manual`].
    fn inline_renderer_mut(&mut self) -> &mut Renderer {
        self.renderer.as_mut().expect(
            "the render thread owns the serializer, the mirror and the sink — pin Clock::Manual",
        )
    }

    /// Bring the renderer back from the render thread, so that the epilogue has a sink to go to.
    ///
    /// **The render thread is joined; the input thread never is** (spec §7). This one is either
    /// parked on a condvar or inside a bounded `write`, both finite. The input thread sits in a
    /// blocking `read` with nothing to wake it short of a signal, so it dies with the process — and
    /// that asymmetry is here rather than in a document because this is the function where the second
    /// join would go.
    ///
    /// It happens **before** the epilogue, and that ordering is the whole reason this is a separate
    /// method: the sink is inside the renderer, and writing the epilogue while a frame is still in
    /// flight interleaves two writes on one descriptor.
    fn reclaim_renderer(&mut self) {
        let Some(handle) = self.render.take() else {
            return;
        };
        // And **here is where the input thread's join would go**, and it is deliberately not here.
        // It sits in a blocking `read` on the tty with nothing to wake it short of a signal, so
        // joining it would hang every exit; it dies with the process instead. Nothing it owns needs
        // giving back — the terminal's read direction is a mode, and modes are `Tty`'s `Drop` and
        // `crate::shutdown`'s bytes.
        self.mailbox.quit();
        // A panicked render thread has already dropped the sink. There is nothing to recover and
        // nothing to write through, and `end_session` is what handles that.
        if let Ok(renderer) = handle.join() {
            self.renderer = Some(renderer);
        }
    }

    /// Whether the render thread has left, by panic or otherwise.
    #[cfg(test)]
    pub(crate) fn renderer_is_gone(&self) -> bool {
        self.mailbox.renderer_is_gone()
    }

    /// The same door, immutably.
    #[cfg(test)]
    fn inline_renderer(&self) -> &Renderer {
        self.renderer.as_ref().expect(
            "the render thread owns the serializer, the mirror and the sink — pin Clock::Manual",
        )
    }

    /// Block until there is a reason to run a frame, and until the frame clock allows one.
    ///
    /// **The one place the app thread blocks.** Everything that can wake it is multiplexed here — an
    /// input event, a [`WakeHandle::post`] from another thread, a deadline registered through
    /// [`Screen::request_wake_at`], and the render thread taking a packet the last `present` could
    /// not hand over.
    ///
    /// # The clock gates here, not `present`
    ///
    /// See `docs/adr/0004-the-frame-clock-gates-the-wait.md`. Refusing the frame at `present` is half
    /// a solution: by
    /// then the runtime has already run its layout, its reactivity and every drawing verb for a frame
    /// nobody will see. Measured against an event storm at 1000 Hz with a 167 µs frame standing in
    /// for one iteration of runtime work, gating at `present` runs 800.4 iterations a second to show
    /// 114.7 frames — **14% useful** — against 114.5 for 114.5 and nothing wasted. And at a sparse
    /// event rate it *delivers fewer frames*, 83.8 fps against 110.8, because it can only paint when
    /// an event happens to arrive. The cheaper scheme is the smoother one, so there is no trade.
    ///
    /// The consequence, stated rather than buried: **a keystroke arriving 1 ms after a paint is held
    /// for the rest of the gap** — 7.3 ms at a 120 Hz ceiling — before the runtime is told about it.
    /// Input latency is bounded by one frame interval by design, and the delay is only observable
    /// through a repaint, which was going to cost the same wait anyway.
    ///
    /// # A minimum gap, not a tick, and an idle that is zero
    ///
    /// The first damage after a quiet period returns immediately; everything arriving inside the gap
    /// coalesces into one return at the end of it. With nothing pending and no deadline registered
    /// **the wait is indefinite** — there is no timer anywhere in this crate — so an idle application
    /// costs no wakeups and no CPU at all. A fixed-rate ticker would cost 3 600 wakeups over thirty
    /// idle seconds at 120 Hz, and add up to half a frame of latency to the first keystroke.
    ///
    /// **[`Wake::Quit`] is checked before the clock.** At a 1 Hz ceiling, checking the clock first
    /// hangs shutdown for a second; shutdown latency must not be a function of the refresh rate.
    ///
    /// # The ceiling is achieved from below
    ///
    /// `wait_timeout` overshoots — a hold measures 10.18 ms p50 against an 8.333 ms gap on macOS —
    /// so a configured 120 Hz yields about 110 fps under load. Undershooting a ceiling is safe by
    /// construction, and the number is written down so nobody spends a day rediscovering it.
    ///
    /// ```
    /// use vitui_engine::{Config, Engine, Output, Wake};
    ///
    /// let (mut screen, wake) = Engine::new(Config {
    ///     output: Output::Sink(Box::new(Vec::new())),
    ///     ..Default::default()
    /// })
    /// .attach()
    /// .unwrap();
    ///
    /// // Nothing is pending, so this would park for ever. A quit is never paced.
    /// wake.quit();
    /// assert_eq!(screen.wait(), Wake::Quit);
    /// ```
    pub fn wait(&mut self) -> Wake {
        self.wakes.wait(self.frame_clock.next_allowed())
    }

    /// Take the next thing the terminal reported, or `None` when there is nothing waiting.
    ///
    /// The queue drains in arrival order and this is the only door to it. [`Wake::Input`] says a
    /// read happened; this says what was in it, and the two are not one call because a single wake
    /// can carry a hundred events and a `wait` that returned one of them would need ninety-nine more
    /// wakes to deliver the rest.
    ///
    /// # A resize is applied on the way past
    ///
    /// [`Event::Resize`] is the one variant that means something to the engine as well as to the
    /// application, and this is the only app-thread call that sees it. So the surfaces are rebuilt
    /// and a full repaint is scheduled **before** the event is returned: an application that draws
    /// in response to it is already drawing at the new size, and one that ignores it entirely still
    /// gets a correct screen. Doing it at `present` instead would put a frame composed at the old
    /// size on the wire first.
    ///
    /// The layers keep their rectangles — the runtime brings a rectangle and a draw for every layer
    /// every frame (spec §12), so a layer whose shape must change is
    /// [`LayerStack::set_rect`](crate::LayerStack::set_rect)'s business and not this one's.
    ///
    /// ```
    /// use vitui_engine::{Config, Engine, Output};
    ///
    /// let (mut screen, _wake) = Engine::new(Config {
    ///     // Headless: no terminal, so no input thread, so nothing ever arrives.
    ///     output: Output::Sink(Box::new(Vec::new())),
    ///     ..Default::default()
    /// })
    /// .attach()
    /// .unwrap();
    /// assert_eq!(screen.next_event(), None);
    /// ```
    pub fn next_event(&mut self) -> Option<Event> {
        let event = self.input.pop()?;
        if let Event::Resize(w, h) = event {
            if (w, h) != self.size {
                self.resize(w, h);
            }
        }
        Some(event)
    }

    /// What the input parser could not make sense of.
    ///
    /// **Unrecognised escape sequences are dropped, counted, and the last one kept.** There is
    /// nothing to hand upward — a sequence nothing recognises is not a key — but silent discard is
    /// the defect class that costs a day: "Shift+F5 does nothing", with no thread to pull. Two
    /// fields buy the thread.
    pub fn input_diagnostics(&self) -> InputDiagnostics {
        self.input.diagnostics()
    }

    /// Wake at `when`, unless something wakes the app thread sooner.
    ///
    /// **The engine has no animation concept and this is the whole of what it offers one.** Timelines,
    /// easing and interpolation are the runtime's. There is no cancel: *deregistration is simply not
    /// renewing it*, because a deadline that has arrived and been reported is already gone and a
    /// caller that no longer wants one only has to stop asking.
    ///
    /// # It is one slot, not a set
    ///
    /// The earliest instant wins and **a later one is dropped rather than queued**. Two components
    /// registering `+8 ms` and `+500 ms` in one frame leave only the 8 ms, and once it has fired
    /// nothing is registered at all.
    ///
    /// That is a sink and not a loss, because **the caller re-registers every frame**: the runtime
    /// keeps its own set and flushes the earliest of it once per settle, so the 500 ms deadline is
    /// re-supplied on the frame the 8 ms one bought. An engine that held the set would be a
    /// scheduler, and §12's refusal 9 is that there isn't one. A caller that registers once and never
    /// again gets exactly one wake, which is what it asked for.
    ///
    /// It is `&self` because it only writes to the wake source, which is shared: an animating
    /// component can register a deadline while holding a [`View`](crate::View) borrowed out of the
    /// layer stack.
    ///
    /// A deadline is paced like any other reason: one arriving inside the gap is held to the end of
    /// it. So the wake is *not before* `when`, and never far after it.
    pub fn request_wake_at(&self, when: Instant) {
        self.wakes.request_wake_at(when);
    }

    /// A monitor changed under the running program.
    ///
    /// The engine never asks the hardware — a tty cannot answer — so the ceiling is the
    /// application's to discover and to say. See [`Config::max_frame_rate`] for what the number
    /// means; the same rules apply, `f32::INFINITY` and anything at or below zero being unlimited.
    ///
    /// **Ignored on [`Clock::Manual`]**, which is not paced at all: a deterministic mode that
    /// contained a real sleep would be reproducible in its interleaving and not in its timing. Ignored
    /// and not rejected — §12's signature has nothing to reject with, and setting a ceiling at startup
    /// while choosing the clock elsewhere is a reasonable thing for an application to do.
    pub fn set_max_frame_rate(&mut self, hz: f32) {
        self.frame_clock.set_rate(hz);
    }

    /// How much mouse reporting the terminal is switched on for.
    ///
    /// **The engine's entire mouse actuator**, and it carries an obligation the spec states rather
    /// than implies: *it is idempotent, and free when the value has not changed.* That is not
    /// politeness, it is load-bearing — the runtime calls this after every frame, and a naive
    /// implementation writes an escape sequence per frame for ever. Free here means free in the
    /// strong sense: an unchanged level does not merely emit nothing, it does not cause a frame.
    ///
    /// # A `max`, not a union, and the caller is the one that takes it
    ///
    /// [`MouseMode`] is totally ordered — `Off < Buttons < Drag < Motion`, modes 1000, 1002 and
    /// 1003 — because each level strictly contains the one below. So combining what several
    /// components want is a `max` over what they declared, and that `max` is the runtime's to take:
    /// there is no mount, a component is a function, and its declaration rides the draw. The engine
    /// receives one level per frame and has no idea how many components it came from.
    ///
    /// [`Config::input`] is the **floor** under it. The union is known only after a draw, so the
    /// frame that first paints a hover-wanting modal did not yet have tracking on; an application
    /// that knows it wants the mouse says so once and has no blind frame, and one that does not pays
    /// nothing.
    ///
    /// # A terminal without a mouse takes this silently
    ///
    /// Clamp-and-discard, never a `Result`. An application that did not ask
    /// [`capabilities`](Screen::capabilities) will not handle an error usefully, and one that did
    /// already knows. SGR encoding is switched on with the mouse and never separately — without it
    /// coordinates stop at column 223, and the performance budget is written against 300 columns.
    ///
    /// ```
    /// use vitui_engine::{Config, Engine, MouseMode, Output};
    ///
    /// let (mut screen, _wake) = Engine::new(Config {
    ///     output: Output::Sink(Box::new(Vec::new())),
    ///     ..Default::default()
    /// })
    /// .attach()
    /// .unwrap();
    /// // A sink is not a terminal, so it has no mouse, and this is silent rather than an error.
    /// screen.set_mouse(MouseMode::Motion);
    /// ```
    pub fn set_mouse(&mut self, mode: MouseMode) {
        self.actuators.set_mouse(mode);
    }

    /// Where the caret is, in **screen** coordinates, or `None` for no caret.
    ///
    /// Applied by [`present`](Screen::present) after the frame's last write, **which is the only
    /// moment at which it is correct and a moment only the engine has** (ADR 0005): the frame has
    /// just moved the terminal's cursor to wherever its last cell was. The runtime translates from
    /// layer coordinates, which it can, because it brought the layer's rectangle.
    ///
    /// Blinking is the terminal's, at the rate its user configured. **There is no software caret
    /// anywhere in this crate**, and that is a measurement rather than a preference: one `restyle` of
    /// one cell, toggled, costs two wakeups a second for as long as anything has focus — 7 200 an
    /// hour on a screen where nothing is happening — so ticket 19's measured idle would not survive a
    /// text field, and a form is not an exotic component. The terminal's own caret is also the only
    /// one a screen reader or an IME can follow.
    ///
    /// A caret off the edge of the screen is clamped, not refused.
    ///
    /// ```
    /// use vitui_engine::{Config, Cursor, CursorShape, Engine, Output};
    ///
    /// let (mut screen, _wake) = Engine::new(Config {
    ///     output: Output::Sink(Box::new(Vec::new())),
    ///     ..Default::default()
    /// })
    /// .attach()
    /// .unwrap();
    /// screen.set_cursor(Some(Cursor { x: 12, y: 3, shape: CursorShape::Bar }));
    /// screen.set_cursor(None);
    /// ```
    pub fn set_cursor(&mut self, cursor: Option<Cursor>) {
        self.actuators.set_cursor(cursor, self.size);
    }

    /// Composite the damaged rectangles, pack them, serialise them, and write once.
    ///
    /// The only exit. Damage is marked by the drawing verbs and cleared here, and neither is
    /// reachable from outside — a forgotten clear produces a frame that repaints for ever, so it is
    /// an invariant rather than a chore, and it is also why nothing above the engine can force a
    /// full repaint.
    pub fn present(&mut self) -> Presented {
        // **Sampled here and re-checked at submit** (spec §2's sixth invariant). One load, and what
        // it buys is that a frame composited at 300x80 is never written into a terminal that became
        // 120x40 while it was being composited — which wraps and scrolls, and is worse than a
        // missing frame.
        let sampled = self.terminal_size.get();
        // **The surfaces are the wrong shape for the terminal, and this frame would wrap and
        // scroll.** The re-check below catches a resize that lands *inside* the frame; this catches
        // one the input thread recorded before the frame began and the application has not drained
        // yet — [`Screen::next_event`] is what applies it, and nothing obliges an application to
        // call it before every `present`. The review found this: before impl 20 nothing wrote the
        // authoritative size in a release build, so the two comparisons were the same comparison and
        // only one of them was written.
        //
        // Refused **before** the 107 µs composite rather than after it, and a frame is owed so the
        // retry is guaranteed rather than hoped for. Damage has not been taken out of the layer
        // stack at this point, so the frame that does run sees exactly what this one would have.
        if sampled != self.size {
            self.wakes.owe_frame();
            self.repaint = true;
            return self.not_submitted(true);
        }
        self.layers.take_damage_into(&mut self.frame);

        // What the two setters have to say, and it is asked here so that the idle path below can be
        // about *nothing to do* rather than about *no cells to write*.
        let actuation = self.actuators.pending();

        // The idle path, and it is one scan of a couple of summary words rather than of the bitset:
        // an idle frame costs nanoseconds and clears nothing. **Nothing is owed here**, and that is
        // the whole difference from the `Lease::Busy` arm below: there is no damage waiting for a
        // frame, so `wait` has nothing to come back for and parks indefinitely.
        //
        // A quiet actuation is what makes *free when unchanged* free in the strong sense: an
        // unchanged `set_mouse` after every frame does not merely emit nothing, it does not get
        // here. A caret that moved with nothing else changing is the other arm — no damage, a frame
        // to send, and it is a handful of bytes.
        if self.frame.damage().is_empty() && actuation.is_quiet() {
            return self.not_submitted(false);
        }

        // **The pacing gate, before the 107 µs composite rather than after it.** While the render
        // thread has not taken the last packet, this frame is not composited at all: damage stays in
        // §6's structure, which is already the coalescing mechanism and costs 6.6 ns to interrogate,
        // and the next frame that does composite paints all of it at once. That is the whole of the
        // backpressure design — there is no queue to grow and no frame to drop, because a frame that
        // would have been dropped was never composed.
        //
        // On the deterministic path this cannot answer anything but `Ready`: the renderer is this
        // thread and it finished before `present` returned.
        let mut packet = match self.mailbox.lease() {
            Lease::Ready(packet) => packet,
            Lease::Busy => {
                self.coalesced += 1;
                // **The damage is owed a frame, and `wait` is what pays it.** Without this the app
                // thread parks with a composite nobody asked it to throw away: on a slow link, a
                // user who stops typing while the renderer is inside a 200 ms write never sees the
                // last keystroke, because nothing else is going to happen. The wake source releases
                // when the renderer takes the packet it is holding. See `crate::clock`.
                self.wakes.owe_frame();
                return self.not_submitted(false);
            }
            // Unreachable at a pool of two, and register entry #8 is the count that says so over
            // 10 000 cycles. Answering with a frame nobody asked for would be worse than answering
            // with nothing — but a caller retrying an unsubmitted frame would then turn on the spot,
            // and a hang is worse than a failure. So a debug build says which invariant broke and a
            // release build answers with a missing frame.
            Lease::Starved => {
                debug_assert!(
                    false,
                    "the pool starved with the renderer free, which the ready gate makes \
                     unreachable: one packet is being filled and one is in the renderer's hands, \
                     and there is no third"
                );
                self.wakes.owe_frame();
                return self.not_submitted(false);
            }
        };

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
        self.generation += 1;
        packet.pack(
            &self.runs,
            &self.frame,
            self.layers.tables(),
            std::mem::take(&mut self.repaint),
            actuation,
            self.generation,
        );

        // The input thread's write, at the one instant no caller of this crate can reach. See
        // `Screen::resize_before_submit`.
        #[cfg(test)]
        if let Some(size) = self.resize_before_submit.take() {
            self.terminal_size.set(size);
        }

        // **The re-check.** A lease is never invalidated — the surfaces are untouched and the caller
        // that drew into them is long gone — but the frame it produced is discarded, because its
        // cells describe a screen that no longer exists. Once per resize, not once per frame after
        // one: the comparison is against the value this frame *started* at, so the next frame samples
        // the new size and agrees with itself.
        if self.terminal_size.get() != sampled {
            self.mailbox.give_back(packet);
            // Owed, exactly as a coalesced frame is: the damage below is going to need a frame and
            // nothing else is guaranteed to ask for one. The renderer is free — the packet went back
            // to the pool rather than into the slot — so the next `wait` releases at the gap.
            self.wakes.owe_frame();
            // Two things at once, and they are the same thing: the `repaint` this frame took out of
            // the latch goes back in, and a resize needs one anyway — the mirror is describing a
            // terminal that has just reflowed under it.
            self.repaint = true;
            self.frame.damage_mut().mark_all();
            return self.not_submitted(true);
        }

        // **Before the submit, and the order is load-bearing.** `submit` notifies the render thread,
        // which may take the packet and call `mark_renderer_free` before this thread runs again — and
        // then a `frame_submitted` afterwards would write `free = false` over a renderer that is
        // free. Setting it first closes the window by construction: the renderer cannot learn there
        // is a packet until `submit`, so nothing can raise the flag before this line lowers it.
        //
        // What that buys is an invariant `wait` rests on: **whenever the mailbox says ready, `free`
        // is true**, because the only thing that makes the mailbox ready is a take and every take
        // raises the flag. Without it, an owed frame could be recorded against a stale `false` and
        // the app thread would park on a renderer that had already let go.
        self.wakes.frame_submitted();
        // **After the last early return and before the submit.** What the setters asked for is owed
        // until a packet carrying it actually goes out: a frame discarded for a resize leaves the
        // mouse level and the caret exactly as they were, to be carried by the frame that replaces
        // it, in the same shape as `repaint` above.
        self.actuators.handed();
        // One relaxed store, so that the thread that restores the terminal — which is not this one
        // when the process is unwinding — knows the session has asked for a mouse. It is a flag and
        // not a level, and it is never cleared: the packet carrying this actuation has not been
        // written yet, so what the terminal is *in* is one of two modes until it has been. Here
        // rather than inside `handed`, because `Actuators` is `Copy` and knows nothing about a site.
        self.shutdown
            .mouse_may_be_on(self.actuators.mouse() != MouseMode::Off);
        self.mailbox.submit(packet);
        // The gap starts at the submit, not at the write: what is being paced is how often frames are
        // handed on, and §12's refusal 7 is that the engine cannot say when one was shown.
        self.frame_clock.submitted();
        // The deterministic path: this thread is the render thread, so it does the render thread's
        // work here, through the same mailbox and the same renderer. `take_now` cannot answer `None`
        // — the submit above put a packet in the slot and nothing else can take it.
        if let Some(renderer) = self.renderer.as_mut() {
            if let Some(packet) = self.mailbox.take_now() {
                self.wakes.mark_renderer_free();
                renderer.render(&packet);
                self.mailbox.finish(packet);
            }
        }

        self.frame.damage_mut().clear();
        self.layers.clear_damage();

        Presented {
            submitted: true,
            coalesced: std::mem::take(&mut self.coalesced),
            discarded_for_resize: false,
        }
    }

    /// What every early return out of `present` answers with.
    ///
    /// `coalesced` is reported rather than reset: the frames folded so far are folded into the frame
    /// that eventually submits, and that is the one that clears the count.
    fn not_submitted(&self, discarded_for_resize: bool) -> Presented {
        Presented {
            submitted: false,
            coalesced: self.coalesced,
            discarded_for_resize,
        }
    }

    /// Block until the render thread has taken the last packet.
    ///
    /// **Not the parking point, and ticket 19 is where that was decided.** [`Screen::wait`] is the
    /// app thread's blocking call and it multiplexes the renderer going free along with everything
    /// else — through [`crate::clock::WakeSource`], because a thread cannot park on two condvars. What
    /// this is for is the gates: *do not go on until the renderer has taken that frame* is an
    /// ordering over the handoff, and asking it through `wait` would be asking it through the clock.
    ///
    /// Returns at once on the deterministic path, where the renderer is never busy.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn wait_for_renderer(&self) {
        self.mailbox.wait_until_free();
    }

    /// Do what the panic hook would do, on this thread and without a panic.
    ///
    /// The hook reaches `crate::shutdown`'s process-global site, and a test that panicked to get at
    /// it would be a test about which of its siblings attached last. This is the same call the hook
    /// makes, against this screen's own site.
    #[cfg(test)]
    pub(crate) fn restore_as_a_panic_would(&self) -> bool {
        self.shutdown.restore(None)
    }

    /// A handle to the mailbox that outlives this `Screen`.
    ///
    /// For the one gate that has to watch what `Screen::drop` does *from another thread* — the drop
    /// is what sets `quit`, and a test that wants to act on that ordering cannot borrow a value that
    /// is being dropped.
    #[cfg(test)]
    pub(crate) fn mailbox_handle(&self) -> Arc<Mailbox> {
        Arc::clone(&self.mailbox)
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
    /// **Nothing outside this crate calls it, and nothing will.** §12's public surface has no
    /// `resize` on `Screen`: the authoritative size is one packed atomic written by the input
    /// thread, and a resize reaches the application as an [`Event::Resize`] — which
    /// [`Screen::next_event`] applies here on its way past.
    pub(crate) fn resize(&mut self, w: u16, h: u16) {
        self.size = (w, h);
        // The caret was clamped to the screen it was set on. See [`Actuators::resized`] for why the
        // clamp is the only part of it that needs redoing.
        self.actuators.resized(self.size);
        self.frame = Surface::new(w, h);
        self.frame.damage_mut().mark_all();
        self.runs = Vec::with_capacity(h as usize * 4);
        // **The mirror is invalidated by the flag and not by the size delta**, and the difference is
        // a defect the review found. The render thread rebuilds its serializer when a packet's size
        // differs from the one it holds — which misses a size that leaves and comes back: 300x80 to
        // 120x40 to 300x80, with both events handled before the next frame, hands the renderer a
        // packet of the size it already has, and the mirror still describes a screen that has
        // reflowed twice. The frame damages every cell and the equality filter then suppresses
        // exactly the ones that match the pre-reflow mirror, so the garbage stays on the terminal.
        //
        // The flag has no such hole, because it is about the *event* rather than about a value.
        self.repaint = true;
        // Eagerly on the deterministic path as well, because a caller may read the mirror between a
        // resize and the next frame and a mirror of the old size would answer about rows that no
        // longer exist. On the threaded path the packet's size is what carries it.
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.resize(w, h);
        }
        // **The authoritative size is not written here, and the review is what corrected that.** It
        // belongs to the input thread, and the app thread agreeing with it is not harmless: a second
        // resize can land while the application is still draining the first event, and then this
        // would put the *older* size back. The next frame would sample it, agree with itself at
        // submit, and write a 120x40 frame into an 80x24 terminal — the wrap-and-scroll §2's sixth
        // invariant exists to prevent, with the evidence erased by the thread that was supposed to
        // read it. The surfaces' size is `Screen::size`; the terminal's is nobody here's to say.
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

    /// The packet the last frame packed, for register entry #10's equality.
    ///
    /// A closure, because the packet lives in the pool behind the mailbox's lock. `None` when no
    /// frame has packed one yet. Deterministic path only: on the threaded path the renderer may still
    /// be holding the frame being asked about, and then this is `None` as well.
    #[cfg(test)]
    pub(crate) fn with_packet<R>(&self, f: impl FnOnce(&Packet) -> R) -> Option<R> {
        self.mailbox.with_last_packed(f)
    }

    /// How many times the render thread's wait has returned. Register entry #17's other half:
    /// **zero over an idle window**, where a fixed-rate ticker would put one per tick.
    #[cfg(test)]
    pub(crate) fn render_wakeups(&self) -> u64 {
        self.mailbox.wakeups()
    }

    /// App submit to the render thread holding the packet, one entry per frame it took.
    ///
    /// Register entry #26, and the distribution is a **report**: it is OS scheduler latency on the
    /// way to the wire rather than app-thread CPU work, so it does not spend §13's 100 µs budget and
    /// it is not the sort of number a shared runner can gate.
    #[cfg(test)]
    pub(crate) fn wake_latencies(&self) -> Vec<std::time::Duration> {
        self.mailbox.latencies()
    }

    /// Forget what the birth frames recorded.
    #[cfg(test)]
    pub(crate) fn reset_wake_latencies(&self) {
        self.mailbox.reset_latencies();
    }

    /// Whether `present` left a frame owed to the damage it refused to composite.
    #[cfg(test)]
    pub(crate) fn owes_a_frame(&self) -> bool {
        self.wakes.owes_a_frame()
    }

    /// The earliest instant at which the frame clock would allow another frame.
    ///
    /// `None` when nothing is paced — an unlimited rate, the deterministic clock, or a screen that
    /// has not submitted a frame yet, which is the leading edge.
    #[cfg(test)]
    pub(crate) fn next_frame_allowed(&self) -> Option<Instant> {
        self.frame_clock.next_allowed()
    }

    /// How many packets a submit dropped on the floor, how many leases found the pool empty, and how
    /// many frames the renderer has finished. Register entries #9 and #8, and the count that says the
    /// render thread is doing the work.
    #[cfg(test)]
    pub(crate) fn handoff_counts(&self) -> (u32, u32, u64) {
        (
            self.mailbox.superseded(),
            self.mailbox.starved(),
            self.mailbox.painted(),
        )
    }

    /// Move the authoritative size **inside** the next frame — between its pack and its submit.
    ///
    /// The write the input thread makes, at the one instant it matters: it observes a new size, puts
    /// it in the authoritative atomic and queues an `Event::Resize`. What is gated here is the
    /// sample, the re-check and the discard.
    ///
    /// See [`Screen::resize_before_submit`] for why the window is not reachable from outside a
    /// frame.
    #[cfg(test)]
    pub(crate) fn resize_during_next_frame(&mut self, w: u16, h: u16) {
        self.resize_before_submit = Some((w, h));
    }

    /// Write the authoritative size as the input thread would, without going through the queue.
    ///
    /// `cfg(test)` and it stays that way — the app thread may never write this (see
    /// [`Screen::resize`]). What it is for is the harness, which drives a resize synchronously and
    /// would otherwise leave the surfaces and the terminal disagreeing about a screen that has to
    /// exist for the round trip to mean anything.
    #[cfg(test)]
    pub(crate) fn observe_terminal_size(&self, w: u16, h: u16) {
        self.terminal_size.set((w, h));
    }

    /// Put an event on the queue as the input thread would.
    ///
    /// `cfg(test)` and it stays that way. A headless screen has no terminal, so it has no input
    /// thread and nothing ever fills the queue — and a public door here would let an application
    /// fabricate a keystroke, which is the whole of what ADR 0007 refuses.
    #[cfg(test)]
    pub(crate) fn inject(&self, event: Event) {
        self.input.push(event);
    }

    /// What the app thread believes the terminal's size is, which is not what the surfaces are.
    #[cfg(test)]
    pub(crate) fn terminal_size(&self) -> (u16, u16) {
        self.terminal_size.get()
    }

    /// What §10's `CHA`-after-non-ASCII rule has cost this screen in bytes, for the report that pays
    /// spec §15's second owed measurement. See `crate::serial::Serializer::cha_rule_bytes`.
    #[cfg(test)]
    pub(crate) fn cha_rule_bytes(&self) -> usize {
        self.inline_renderer().serializer.cha_rule_bytes()
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
            .filter(|&y| self.inline_renderer().serializer.mirror().is_known(y))
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
        self.inline_renderer_mut().serializer.set_filter(filter);
    }

    /// Serialise this screen's frames with the scroll pre-pass off: §8's *filtered* column, which is
    /// the arm its *+ scroll region* column is a ratio against. See
    /// [`Serializer::set_scroll_region`](crate::serial::Serializer::set_scroll_region).
    #[cfg(test)]
    pub(crate) fn set_scroll_region(&mut self, on: bool) {
        self.inline_renderer_mut().serializer.set_scroll_region(on);
    }

    /// Serialise the way §8 rejected: verify every candidate the probe matches rather than the first.
    /// The instrument its 27x is reproduced with. See
    /// [`Serializer::set_verify_every_match`](crate::serial::Serializer::set_verify_every_match).
    #[cfg(test)]
    pub(crate) fn set_verify_every_match(&mut self, on: bool) {
        self.inline_renderer_mut()
            .serializer
            .set_verify_every_match(on);
    }

    /// How many of this screen's frames put a scroll on the wire, and how many candidates were
    /// verified to get there. The second is the count §8's 27x regression is gated by.
    #[cfg(test)]
    pub(crate) fn scrolls(&self) -> (usize, usize) {
        let serializer = &self.inline_renderer().serializer;
        (serializer.scrolls(), serializer.verifies())
    }

    #[cfg(test)]
    pub(crate) fn mirror(&self) -> &crate::serial::Mirror {
        self.inline_renderer().serializer.mirror()
    }

    /// How many style words the serializer has narrowed rather than answered from its memo.
    #[cfg(test)]
    pub(crate) fn narrowings(&self) -> usize {
        self.inline_renderer().serializer.narrowings()
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

impl Drop for Screen {
    /// **The guard spec §7 asks for**, and it is the type itself rather than a separate one: a
    /// normal return and a `?` out of `main` both drop the `Screen`, so both take this path and
    /// neither needs the application to remember anything.
    ///
    /// The order is the order it has to be. The render thread is joined **first**, because it owns
    /// the write direction and the sink, and an epilogue written while a frame is in flight
    /// interleaves with it. Then the restoration. Then the site is disarmed, so that a panic later
    /// in the same process does not restore a terminal this screen no longer has.
    fn drop(&mut self) {
        self.reclaim_renderer();
        self.end_session();
        crate::shutdown::disarm(&self.shutdown);
    }
}

/// The render thread's whole life: take a packet, write it, give it back.
///
/// It holds **no application state and no handle at all** (ADR 0011). Everything a cell names was
/// resolved into the packet's own side tables at pack time, so nothing here points into a table the
/// app thread is free to sweep — which is what keeps resize, shutdown and panic small.
pub(crate) struct Renderer {
    serializer: Serializer,
    /// What the serializer's mirror is sized for. Compared against every packet, because a resize
    /// reaches this thread as a packet of a different size and nothing else.
    size: (u16, u16),
    sink: Box<dyn Write + Send>,
    /// A clone of what the terminal answered at `attach`, which is immutable for the life of the
    /// screen — so this is a copy of a constant rather than shared state.
    caps: Capabilities,
}

impl Renderer {
    /// Serialise one packet against the mirror and write it, in one write.
    fn render(&mut self, packet: &Packet) {
        if packet.size() != self.size {
            self.resize(packet.size().0, packet.size().1);
        }
        let bytes = self.serializer.serialize(packet, &self.caps);
        write_frame(&mut *self.sink, bytes);
    }

    /// Start again at a new size: a fresh mirror, which knows nothing, which is the honest state for
    /// a terminal that has just reflowed (ADR 0006).
    fn resize(&mut self, w: u16, h: u16) {
        self.serializer = Serializer::new(w, h);
        self.size = (w, h);
    }
}

/// Block, write, repeat, until the app thread says stop.
///
/// **The guard is the whole of the panic story here**, and the review is what added it: a renderer
/// that dies holding a packet leaves `ready` false for ever, and the app thread then parks on a
/// condvar nobody will signal — `quit` is set from `Screen::drop`, which cannot run while the app
/// thread is parked. A `Drop` runs on the way out of an unwind as well as on the way out of a
/// return, so one guard covers both exits and there is no `catch_unwind` anywhere.
fn render_loop(mailbox: &Mailbox, wakes: &WakeSource, renderer: &mut Renderer) {
    /// Tell the app thread the renderer has left, however it left.
    ///
    /// **Both sides, and the wake source is the one that matters since ticket 19**: the mailbox's
    /// flag releases a `wait_until_free`, and the app thread does not park there any more — it parks
    /// in [`Screen::wait`], which is released from here.
    struct Guard<'a>(&'a Mailbox, &'a WakeSource);

    impl Drop for Guard<'_> {
        fn drop(&mut self) {
            self.0.renderer_gone();
            self.1.renderer_gone();
        }
    }

    let _guard = Guard(mailbox, wakes);
    while let Some(packet) = mailbox.take() {
        // **The take is what frees the renderer, not the write** (spec §7, and it is what fixes the
        // pool at two). So the app thread is told here, before a 200 ms write rather than after it,
        // and the composite of the next frame overlaps this one's bytes.
        wakes.mark_renderer_free();
        renderer.render(&packet);
        mailbox.finish(packet);
    }
}

/// Write the whole frame, retrying what the kernel would not take.
///
/// **The frame is never split on purpose.** A synchronised-output block spanning two `write` calls
/// is still one block to the terminal; a frame split into two blocks tears. This loop is only about
/// the kernel's buffer being smaller than the frame (spec §8).
///
/// A write error is still dropped on the floor here, and shutdown turned out not to be where that
/// gets an answer: the render thread has nowhere upward to report one — `Presented` has no field
/// for it and the engine offers no completion anywhere (§12's refusal 7) — and a terminal whose
/// descriptor has stopped taking bytes is a terminal the epilogue cannot reach either. **No ticket
/// on this backlog owns it**, and that is a statement rather than an omission: it is a public
/// surface decision, and ticket 24 is where the public surface is settled.
pub(crate) fn write_frame(sink: &mut (dyn Write + Send), bytes: &[u8]) {
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
    use crate::actuate::{
        CursorShape, DISABLE_AUTO_WRAP, ENABLE_AUTO_WRAP, ENTER_ALT_SCREEN, LEAVE_ALT_SCREEN,
    };

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

    /// A terminal that can do everything, so that a gate about what was *asked for* is not a gate
    /// about what could be.
    fn everything() -> Capabilities {
        Capabilities::with_input(true, true, true, true, crate::caps::KITTY_ALL)
    }

    /// **The startup negotiation, asserted by what is written before any frame exists.**
    ///
    /// The kitty flags go as one set; mouse tracking, focus reporting and bracketed paste go only
    /// when the application declared them; the caret is hidden because nothing has asked for one.
    /// Read through the terminal model rather than as a byte string, because what matters is the
    /// state the terminal is left in.
    #[test]
    fn the_startup_negotiation_asks_for_what_was_declared_and_nothing_else() {
        // Nothing declared.
        let quiet = crate::testing::Harness::declaring(8, 1, everything(), InputConfig::default());
        assert_eq!(
            quiet.terminal().modes(),
            Vec::<u32>::new(),
            "nothing asked for"
        );
        assert_eq!(
            quiet.terminal().kitty(),
            [crate::caps::KITTY_ALL],
            "the flags cost nothing in idle and go as one set"
        );
        assert_eq!(
            quiet.terminal().caret(),
            None,
            "no caret until one is asked for"
        );

        // Everything declared. **Focus reporting is opt-in**, which is what the pair of harnesses
        // says and one could not.
        let loud = crate::testing::Harness::declaring(
            8,
            1,
            everything(),
            InputConfig {
                mouse: MouseMode::Drag,
                focus: true,
                paste: true,
                ..InputConfig::default()
            },
        );
        assert_eq!(loud.terminal().modes(), vec![1002, 1004, 1006, 2004]);
    }

    /// **A subset that stuck is still pushed as the whole set**, per kitty's own instruction that
    /// implementing part of the stack makes no sense.
    #[test]
    fn the_kitty_flags_are_never_cherry_picked_to_what_survived() {
        let partial = Capabilities::with_input(false, false, false, false, 0b0_0011);
        let h = crate::testing::Harness::declaring(8, 1, partial, InputConfig::default());
        assert_eq!(h.terminal().kitty(), [crate::caps::KITTY_ALL]);
    }

    /// **`Config::input` is a floor**: a declared level is active on the very first frame, before
    /// anything has drawn.
    ///
    /// The frame that first paints a hover-wanting modal did not yet have tracking on, which is the
    /// whole reason a floor exists — so the assertion is taken *before* a single drawing verb has run.
    #[test]
    fn a_declared_mouse_level_is_active_before_anything_has_drawn() {
        let h = crate::testing::Harness::declaring(
            8,
            1,
            everything(),
            InputConfig {
                mouse: MouseMode::Motion,
                ..InputConfig::default()
            },
        );
        assert_eq!(h.terminal().modes(), vec![1003, 1006], "no blind frame");
        assert!(
            String::from_utf8_lossy(&h.prologue()).contains("?1006h"),
            "SGR encoding arrives with the mouse, or a press past column 223 is unreportable"
        );
    }

    /// **A terminal without a mouse takes `set_mouse` silently**, and returns nothing.
    ///
    /// Clamp-and-discard, never a `Result`: an application that did not ask `capabilities` will not
    /// handle an error usefully, and one that did already knows.
    #[test]
    fn set_mouse_on_a_terminal_without_a_mouse_is_silent() {
        let mut h = crate::testing::Harness::declaring(
            8,
            1,
            Capabilities::with_input(false, false, false, false, 0),
            InputConfig {
                mouse: MouseMode::Motion,
                ..InputConfig::default()
            },
        );
        assert_eq!(
            h.terminal().modes(),
            Vec::<u32>::new(),
            "nothing was asked for"
        );
        // The signature is `()`, so this line is the assertion that there is nothing to handle.
        let () = h.screen.set_mouse(MouseMode::Motion);
        assert!(!h.screen.present().submitted, "and it caused no frame");
        assert_eq!(h.bytes_written(), 0);
    }

    /// **The caret is applied after the frame's last write**, asserted by byte order in the sink.
    ///
    /// It is the only moment at which it is correct: the frame has just moved the terminal's cursor
    /// to wherever its last cell was (ADR 0005).
    #[test]
    fn the_caret_is_applied_after_the_frames_last_write() {
        let mut h = crate::testing::Harness::new(20, 2);
        let id = h
            .screen
            .layers()
            .add_content(0, crate::geom::Rect::new(0, 0, 20, 2), true);
        h.screen
            .layers()
            .view(id)
            .unwrap()
            .text(0, 1, "hello", crate::style::Style::new());
        h.screen.set_cursor(Some(Cursor {
            x: 0,
            y: 0,
            shape: CursorShape::Bar,
        }));
        h.present();

        let wire = h.wire();
        let seen = String::from_utf8_lossy(&wire).replace('\x1b', "^[");
        // Shape, then position, then visibility, and all three after the last cell. The order is the
        // one that cannot be seen going wrong: a shape set after the caret is shown changes it under
        // the eye, and a caret shown before it is placed appears where the last write ended.
        assert!(
            seen.ends_with("^[[5 q^[[1;1H^[[?25h"),
            "the caret is the last thing the frame says: {seen}"
        );
        assert!(
            seen.find("hello").expect("the cells went out") < seen.rfind("^[[5 q").unwrap(),
            "and the cells are before it: {seen}"
        );
        assert_eq!(h.terminal().caret(), Some((0, 0)));
        assert_eq!(
            h.terminal().caret_shape(),
            5,
            "a blinking bar is DECSCUSR 5"
        );
    }

    /// A caret that moved with nothing else changing is still a frame, and hiding it is another.
    ///
    /// The other half of *free when unchanged*: unchanged is free, and changed is not silently
    /// dropped because no cell moved.
    #[test]
    fn a_caret_that_moved_with_nothing_else_changing_is_a_frame_of_its_own() {
        let mut h = crate::testing::Harness::new(20, 2);
        let id = h
            .screen
            .layers()
            .add_content(0, crate::geom::Rect::new(0, 0, 20, 2), true);
        h.screen
            .layers()
            .view(id)
            .unwrap()
            .text(0, 0, "x", crate::style::Style::new());
        h.screen.set_cursor(Some(Cursor::default()));
        h.present();

        h.screen.set_cursor(Some(Cursor {
            x: 7,
            y: 1,
            shape: CursorShape::Terminal,
        }));
        assert!(
            h.present().submitted,
            "no damage, and still a frame to send"
        );
        assert_eq!(h.terminal().caret(), Some((7, 1)));

        h.screen.set_cursor(None);
        assert!(h.present().submitted);
        assert_eq!(h.terminal().caret(), None);

        assert!(!h.present().submitted, "and then nothing at all");
    }

    /// A caret set on a screen that then shrank is clamped to the screen it is on, not left off it.
    ///
    /// Nothing else about it moves: the terminal's `DECTCEM` and `DECSCUSR` survive a reflow, and its
    /// cursor position does not need to, because a resize damages every cell and the frame that
    /// follows re-places the caret against a mirror that knows nothing.
    #[test]
    fn a_caret_is_reclamped_when_the_screen_shrinks_under_it() {
        let mut h = crate::testing::Harness::new(40, 8);
        let id = h
            .screen
            .layers()
            .add_content(0, crate::geom::Rect::new(0, 0, 40, 8), true);
        h.screen.set_cursor(Some(Cursor {
            x: 39,
            y: 7,
            shape: CursorShape::Block,
        }));
        h.present();
        assert_eq!(h.terminal().caret(), Some((39, 7)));

        h.resize(10, 2);
        h.screen
            .layers()
            .set_rect(id, crate::geom::Rect::new(0, 0, 10, 2));
        h.present();
        assert_eq!(
            h.terminal().caret(),
            Some((9, 1)),
            "clamped, not off the screen"
        );
        assert_eq!(h.terminal().caret_shape(), 1, "and still a block");
    }

    /// A frame discarded for a resize leaves the caret and the mouse **owed**, in the same shape as
    /// `repaint`: what the setters asked for is not spent until a packet carrying it goes out.
    #[test]
    fn a_frame_discarded_for_a_resize_leaves_the_setters_owed() {
        let mut h = crate::testing::Harness::declaring(20, 2, everything(), InputConfig::default());
        let id = h
            .screen
            .layers()
            .add_content(0, crate::geom::Rect::new(0, 0, 20, 2), true);
        h.screen
            .layers()
            .view(id)
            .unwrap()
            .text(0, 0, "x", crate::style::Style::new());
        h.screen.set_mouse(MouseMode::Buttons);
        h.screen.resize_during_next_frame(20, 2);
        // The size does not actually move, so nothing is discarded — this is the control, and it is
        // what says the assertion below is about the discard rather than about the setter.
        assert!(h.screen.present().submitted);
        assert_eq!(h.terminal().modes(), Vec::<u32>::new(), "not replayed yet");

        let mut h = crate::testing::Harness::declaring(20, 2, everything(), InputConfig::default());
        let id = h
            .screen
            .layers()
            .add_content(0, crate::geom::Rect::new(0, 0, 20, 2), true);
        h.screen
            .layers()
            .view(id)
            .unwrap()
            .text(0, 0, "x", crate::style::Style::new());
        h.screen.set_mouse(MouseMode::Buttons);
        h.screen.resize_during_next_frame(10, 1);
        let discarded = h.screen.present();
        assert!(discarded.discarded_for_resize && !discarded.submitted);
        assert_eq!(h.bytes_written(), 0, "nothing reached the wire");

        // The screen catches up, and the level the discarded frame was carrying is still owed.
        h.resize(10, 1);
        h.screen
            .layers()
            .set_rect(id, crate::geom::Rect::new(0, 0, 10, 1));
        h.screen
            .layers()
            .view(id)
            .unwrap()
            .text(0, 0, "x", crate::style::Style::new());
        assert!(h.present().submitted);
        assert_eq!(h.terminal().modes(), vec![1000, 1006]);
    }

    /// A resize is applied on the way past, and the application still receives it.
    ///
    /// Both halves matter and the second is the one that is easy to lose: an engine that swallowed
    /// the event to do its own bookkeeping would leave every component sized from a frame ago, and
    /// one that only handed it up would put a 300x80 composite on a 120x40 terminal first.
    #[test]
    fn next_event_resizes_the_screen_before_it_hands_the_event_up() {
        let (mut screen, _wake) = Engine::new(headless())
            .attach()
            .expect("a sink cannot fail");
        assert_eq!(screen.size(), (80, 24));
        screen.inject(Event::Resize(120, 40));

        assert_eq!(screen.next_event(), Some(Event::Resize(120, 40)));
        assert_eq!(
            screen.size(),
            (120, 40),
            "the surfaces moved before the caller was told"
        );
        assert!(
            screen.repaint_pending(),
            "a reflowed terminal shows nothing the mirror knows"
        );
        assert_eq!(screen.next_event(), None);
    }

    /// A resize to the size the screen already is costs nothing. The event still arrives — the
    /// terminal said something happened and it is not this crate's place to decide it did not — but
    /// no repaint is scheduled for it.
    #[test]
    fn a_resize_to_the_size_it_already_is_schedules_no_repaint() {
        let (mut screen, _wake) = Engine::new(headless())
            .attach()
            .expect("a sink cannot fail");
        screen.inject(Event::Resize(80, 24));
        assert_eq!(screen.next_event(), Some(Event::Resize(80, 24)));
        assert!(!screen.repaint_pending());
    }

    /// The queue drains in order and the events come back exactly as they went in.
    #[test]
    fn next_event_drains_the_queue_in_arrival_order() {
        let (mut screen, _wake) = Engine::new(headless())
            .attach()
            .expect("a sink cannot fail");
        screen.inject(Event::FocusGained);
        screen.inject(Event::FocusLost);
        assert_eq!(screen.next_event(), Some(Event::FocusGained));
        assert_eq!(screen.next_event(), Some(Event::FocusLost));
        assert_eq!(screen.next_event(), None);
    }

    /// A screen that has seen nothing reports nothing, and `last_unrecognised` is `None` rather than
    /// an empty slice — the two are different answers and only one of them is true here.
    #[test]
    fn a_fresh_screen_has_nothing_to_report_about_input() {
        let (screen, _wake) = Engine::new(headless())
            .attach()
            .expect("a sink cannot fail");
        let diagnostics = screen.input_diagnostics();
        assert_eq!(diagnostics.unrecognised(), 0);
        assert_eq!(diagnostics.last_unrecognised(), None);
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

    /// **A post and a quit are separate reasons, and the quit is reported first.**
    ///
    /// It used to assert on the raised bits, which was a test of the flag layout. Since ticket 19
    /// there is a `wait` to ask instead, and asking it is strictly stronger: it covers the priority
    /// as well as the recording, and the priority is the part that had a defect in it.
    #[test]
    fn a_wake_handle_records_a_post_and_a_quit_separately() {
        let (mut screen, wake) = Engine::new(headless()).attach().unwrap();
        wake.post();
        wake.quit();
        assert_eq!(
            screen.wait(),
            Wake::Quit,
            "shutdown queued behind a frame somebody asked for"
        );
        assert_eq!(screen.wait(), Wake::Posted);
    }

    #[test]
    fn a_wake_handle_clone_shares_the_same_source() {
        let (mut screen, wake) = Engine::new(headless()).attach().unwrap();
        wake.clone().post();
        assert_eq!(screen.wait(), Wake::Posted);
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

    /// **Gate, both directions: the alt screen is entered and left, and auto-wrap is switched off
    /// once inside it and given back before it is.**
    ///
    /// §8 makes auto-wrap a decision rather than an implementation detail, and both halves have a
    /// way of going missing separately — `Tty`'s own `Drop` exists because mode 2027 was being set
    /// and never reset. So the sequence is asserted as a *pair*, and asserted through the terminal
    /// model rather than as two byte strings, because what matters is the state the terminal is left
    /// in.
    ///
    /// # The order of the last two is the part that is not decoration
    ///
    /// Auto-wrap is restored **and then** the alt screen is left, and never the other way round.
    /// Mode 1049 switches the page, not the modes: everything this session set is still set
    /// afterwards, so leaving first means the last bytes of the session are reprogramming the
    /// terminal *behind the user's returned prompt*. DECAWM is the one where that is immediately
    /// visible — a shell whose line editor cannot wrap overwrites its own prompt — and it is the last
    /// thing this sequence says before it gives the page back.
    #[test]
    fn the_alt_screen_is_entered_and_left_and_auto_wrap_goes_off_inside_it() {
        let sink = crate::testing::Recorder::new();
        let recording = sink.handle();
        {
            let (mut screen, _wake) = Engine::new(Config {
                size: (8, 1),
                output: Output::Sink(Box::new(sink)),
                clock: Clock::Manual,
                max_frame_rate: f32::INFINITY,
                overrides: Overrides::default(),
                input: InputConfig::default(),
            })
            .attach()
            .expect("attaching to a sink cannot fail");

            let prologue = recording.lock().unwrap().bytes.clone();
            // The alt screen first and auto-wrap immediately inside it, both before any frame. What
            // follows is ticket 21's negotiation, which `crate::actuate` asserts in full and
            // [`the_startup_negotiation_asks_for_what_was_declared_and_nothing_else`] asserts
            // through a `Screen`.
            assert!(prologue.starts_with(ENTER_ALT_SCREEN), "before any frame");
            assert!(
                prologue[ENTER_ALT_SCREEN.len()..].starts_with(DISABLE_AUTO_WRAP),
                "auto-wrap is switched off on the page this session owns"
            );

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
        assert!(!term.alt_screen(), "the user's own screen is back");
        assert!(
            bytes.ends_with(LEAVE_ALT_SCREEN),
            "leaving the alt screen is the last thing said"
        );
        let restored = bytes.len() - LEAVE_ALT_SCREEN.len();
        assert!(
            bytes[..restored].ends_with(ENABLE_AUTO_WRAP),
            "auto-wrap was given back after the user's screen was, which is a mode change \
             behind their prompt"
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
