//! The two setters `present` applies, and the negotiation that decides what can ever arrive.
//!
//! Everything the engine says to the terminal that is not a cell is here: the mouse tracking level,
//! the caret, and the one-time startup batch that switches the input protocols on. All of it is
//! **bytes with no reply** — the questions were asked and answered in [`crate::detect`], before this
//! module's first byte goes out — which is why none of it needs a deadline, a sentinel, or a read.
//!
//! # Two setters, one shape
//!
//! [`Screen::set_mouse`](crate::Screen::set_mouse) and
//! [`Screen::set_cursor`](crate::Screen::set_cursor) both **record** rather than write. The write
//! direction belongs to the render thread, so what these produce is an [`Actuation`] that
//! rides the next packet, and the delta is computed on the app thread rather than on the render
//! thread — which is what makes *free when unchanged* free at the right end: an unchanged setter
//! does not merely emit nothing, it does not cause a frame.
//!
//! [`Actuators`] holds both halves of that: what the caller asked for, and what the last submitted
//! packet carried. Nothing is committed until a packet actually goes out, so a frame discarded for a
//! resize leaves the mouse and the caret owed rather than lost.
//!
//! # Why the caret is on every packet and the mouse is not
//!
//! A frame that wrote a cell left the terminal's cursor after that cell, so **a visible caret has to
//! be re-placed by every frame that emitted anything**, whether or not the application moved it.
//! That is not a redundant write: it is where the caret is, and the terminal has just been told
//! otherwise. It is also nearly free — the serializer's own [`shortest`
//! move](crate::serial::Serializer) prices it against where the last write left the cursor, and in
//! the case that matters, a character typed into a field with the caret after it, the price is zero
//! bytes. The 29-byte caret frame is that case.
//!
//! The mouse mode has no such property. It is a terminal mode, it stays set, and it is emitted only
//! when it changes.
//!
//! # The caret is the terminal's, and that is a measurement rather than a preference
//!
//! A software caret is one `restyle` of one cell, toggled, at **two wakeups a second for
//! as long as anything has focus** — 7 200 an hour on a screen where nothing is happening. Ticket
//! 19's measured idle (`30.01 s real, 0.00 user, 0.00 sys, 0 voluntary context switches`) does not
//! survive a text field, and a form is not an exotic component. The terminal's own caret blinks for
//! free, in the terminal's process, at the user's configured rate, and is the only thing a screen
//! reader or an IME can follow. **There is no software caret anywhere in this crate**, and
//! `crate::gates::nothing_anywhere_blinks_a_caret_in_software` is the grep that says so.

use crate::caps::{Capabilities, KITTY_ALL};
use crate::input::{InputConfig, MouseMode};

/// Where the caret is and what it looks like.
///
/// **Screen coordinates**, because the caret is one thing on the whole terminal and a layer is not
/// the whole terminal. The runtime translates from layer coordinates, which it can, because it
/// brought the layer's rectangle.
///
/// ```
/// use vitui_engine::{Cursor, CursorShape};
///
/// let caret = Cursor {
///     x: 12,
///     y: 3,
///     shape: CursorShape::Bar,
/// };
/// assert_eq!(caret.shape, CursorShape::Bar);
/// // The shape nobody chose is the one the person at the terminal chose.
/// assert_eq!(Cursor::default().shape, CursorShape::Terminal);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct Cursor {
    /// The column, in screen cells.
    pub x: u16,
    /// The row, in screen cells.
    pub y: u16,
    /// What the caret looks like.
    pub shape: CursorShape,
}

/// What the caret looks like.
///
/// # `DECSCUSR` conflates shape with blink, and [`Terminal`](CursorShape::Terminal) is the answer
///
/// There is one escape for this — `CSI Ps SP q` — and its parameters pair each shape with a blink
/// state: 1/3/5 blinking, 2/4/6 steady. So *choose a shape* and *choose whether it blinks* cannot be
/// two decisions on the wire, however much they are two decisions in a design.
///
/// It falls one way: **blinking is the terminal's**, at the rate its user
/// configured, and the engine's only job is to say where the caret is. So the three named shapes are
/// the blinking spellings — a caret is a thing that blinks — and [`Terminal`](CursorShape::Terminal)
/// is `Ps = 0`, which is *whatever this user set up*, and is the default for exactly that reason.
///
/// An application that wants a steady bar is asking for something the person at the terminal already
/// answered. It is not offered, and this paragraph is where that is written down rather than
/// discovered from a missing variant.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum CursorShape {
    /// Whatever the person at the terminal configured. `DECSCUSR 0`.
    #[default]
    Terminal,
    /// A blinking block. `DECSCUSR 1`.
    Block,
    /// A blinking underline. `DECSCUSR 3`.
    Underline,
    /// A blinking bar. `DECSCUSR 5`.
    Bar,
}

impl CursorShape {
    /// `DECSCUSR`'s `Ps`.
    fn decscusr(self) -> u8 {
        match self {
            CursorShape::Terminal => 0,
            CursorShape::Block => 1,
            CursorShape::Underline => 3,
            CursorShape::Bar => 5,
        }
    }
}

/// What one frame has to say to the terminal besides its cells.
///
/// Computed on the app thread from [`Actuators`], carried by the packet, and emitted by the
/// serializer. Every field is already a **delta** except [`caret`](Actuation::caret), and that
/// exception has a reason: see this module's documentation.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct Actuation {
    /// `(from, to)` when the tracking level changed, and `None` when it did not.
    ///
    /// The old level is carried because switching level means resetting the old mode as well as
    /// setting the new one: 1000, 1002 and 1003 are three modes rather than three values of one.
    pub(crate) mouse: Option<(MouseMode, MouseMode)>,
    /// Where the caret is, or `None` when there is not one. **Carried on every packet**, unlike
    /// everything else here.
    pub(crate) caret: Option<Cursor>,
    /// The shape to put on the wire, when the terminal does not already have it.
    pub(crate) shape: Option<CursorShape>,
    /// The visibility to put on the wire, when the terminal does not already have it.
    pub(crate) show: Option<bool>,
    /// Whether the caret is somewhere it was not.
    ///
    /// Separate from [`show`](Actuation::show) and [`shape`](Actuation::shape) so that a caret that
    /// only *moved* costs a move and nothing else — which is the whole of why a caret frame is 29
    /// bytes rather than 40.
    pub(crate) moved: bool,
}

impl Actuation {
    /// Whether this frame has nothing to say. **A quiet actuation must not cause a frame**: it is
    /// the other half of *idempotent, and free when the value has not changed*.
    pub(crate) fn is_quiet(&self) -> bool {
        self.mouse.is_none() && self.shape.is_none() && self.show.is_none() && !self.moved
    }
}

/// What the caller asked for, and what the terminal was last told.
///
/// One struct rather than two sets of fields on [`Screen`](crate::Screen), because the delta between
/// the two halves is the whole logic and a delta computed at the call site is a delta two call sites
/// can disagree about.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Actuators {
    /// [`Config::input`](crate::Config::input)'s mouse level, clamped. **A floor, not a setting**:
    /// the union of what the frame's components want is known only after a draw, so an
    /// application that knows it wants the mouse says so once and has no blind frame.
    floor: MouseMode,
    /// The highest level this terminal answered for. Everything above it is clamped away silently —
    /// never a `Result`, because an application that did not ask
    /// [`capabilities`](crate::Screen::capabilities) will not handle an error usefully and one that
    /// did already knows.
    ceiling: MouseMode,
    wanted: MouseMode,
    handed: MouseMode,
    caret: Option<Cursor>,
    handed_caret: Option<Cursor>,
    /// The shape the terminal was last told, which is **not** `handed_caret.shape`: hiding a caret
    /// does not put its shape back, so the two go out of step and the shape is the one that has to
    /// be remembered on its own.
    handed_shape: CursorShape,
}

impl Actuators {
    /// The state the startup negotiation leaves the terminal in.
    ///
    /// `handed` starts at the floor rather than at `Off`, and that is what makes
    /// [`Config::input`](crate::Config::input) a floor: the negotiation has already switched the
    /// declared level on, so the first frame has nothing to say about it and no frame is blind.
    pub(crate) fn new(config: &InputConfig, caps: &Capabilities) -> Actuators {
        let ceiling = ceiling(caps);
        let floor = config.mouse.min(ceiling);
        Actuators {
            floor,
            ceiling,
            wanted: floor,
            handed: floor,
            caret: None,
            // The negotiation hides the caret, because nothing has asked for one.
            handed_caret: None,
            handed_shape: CursorShape::Terminal,
        }
    }

    /// The level the runtime's `max` over this frame's components came to, clamped and floored.
    pub(crate) fn set_mouse(&mut self, mode: MouseMode) {
        self.wanted = mode.max(self.floor).min(self.ceiling);
    }

    /// Where the caret is, clamped to the screen.
    ///
    /// A caret off the edge is clamped rather than refused, for the same reason a level above the
    /// ceiling is: there is nothing in the signature to refuse with, and a caller that put it
    /// there has a layout bug the terminal cannot help with.
    pub(crate) fn set_cursor(&mut self, cursor: Option<Cursor>, size: (u16, u16)) {
        self.caret = cursor.map(|c| Cursor {
            x: c.x.min(size.0.saturating_sub(1)),
            y: c.y.min(size.1.saturating_sub(1)),
            shape: c.shape,
        });
    }

    /// The screen changed size under the caret.
    ///
    /// Only the clamp needs redoing, and it is worth being exact about why nothing else does. The
    /// terminal's *modes* survive a reflow — `DECTCEM` and `DECSCUSR` are not reset by `SIGWINCH` —
    /// so the visibility and the shape the terminal was last told are still true. Its cursor
    /// *position* is anybody's guess, and that needs nothing here either: a resize damages every cell,
    /// so the frame that follows opens and re-places the caret against a fresh mirror that
    /// knows nothing, which is an absolute move.
    pub(crate) fn resized(&mut self, size: (u16, u16)) {
        let caret = self.caret;
        self.set_cursor(caret, size);
    }

    /// The startup negotiation has been written a second time, so the terminal is back in the
    /// state [`new`](Actuators::new) describes.
    ///
    /// **Only the *handed* half moves, and that is the whole of it.** What the caller asked for —
    /// the raised mouse level, the caret it put somewhere — is unchanged and still wanted; what the
    /// terminal has been told is now the negotiation's own baseline again, because
    /// [`negotiation`] sets the mouse to the floor and hides the caret. So the next frame's
    /// [`pending`](Actuators::pending) re-emits exactly the difference between the two, which is
    /// what a frame after a resume has to carry.
    ///
    /// Without it, an application that had raised the mouse to `Motion` and placed a caret comes
    /// back from a suspend with `handed` still claiming both — and the terminal, which left the alt
    /// screen and re-entered it, has neither. The screen would repaint perfectly and the mouse
    /// would be dead.
    pub(crate) fn renegotiated(&mut self) {
        self.handed = self.floor;
        self.handed_caret = None;
        self.handed_shape = CursorShape::Terminal;
    }

    /// The current mouse level, for the epilogue that has to put it back.
    pub(crate) fn mouse(&self) -> MouseMode {
        self.handed
    }

    /// What the next packet has to carry.
    pub(crate) fn pending(&self) -> Actuation {
        Actuation {
            mouse: (self.wanted != self.handed).then_some((self.handed, self.wanted)),
            caret: self.caret,
            shape: self
                .caret
                .filter(|c| c.shape != self.handed_shape)
                .map(|c| c.shape),
            show: (self.caret.is_some() != self.handed_caret.is_some())
                .then_some(self.caret.is_some()),
            moved: match (self.caret, self.handed_caret) {
                (Some(now), Some(was)) => (now.x, now.y) != (was.x, was.y),
                // Appearing is a move: the terminal's cursor is wherever the last frame's last write
                // left it, which is not where this caret is.
                (Some(_), None) => true,
                _ => false,
            },
        }
    }

    /// A packet carrying [`pending`](Actuators::pending) has gone out.
    ///
    /// Called **after** the submit and nowhere else, which is what leaves a frame discarded for a
    /// resize with the mouse and the caret still owed.
    pub(crate) fn handed(&mut self) {
        self.handed = self.wanted;
        if let Some(caret) = self.caret {
            self.handed_shape = caret.shape;
        }
        self.handed_caret = self.caret;
    }
}

/// The highest tracking level this terminal answered for.
///
/// **Mode 1002 has no query of its own**, and the probe set is not widened for one: `Capabilities`
/// carries eight input booleans and eight is a decision. So a terminal that reported 1000
/// and not 1003 is still asked for 1002, and what decides that direction is which way is safe — a
/// terminal that ignores the request reports buttons only, which is wrong in *degree*, where
/// clamping to [`MouseMode::Buttons`] would deny drag to every terminal that has it and be wrong in
/// *direction*.
fn ceiling(caps: &Capabilities) -> MouseMode {
    if !caps.mouse {
        MouseMode::Off
    } else if caps.mouse_motion {
        MouseMode::Motion
    } else {
        MouseMode::Drag
    }
}

/// The DEC private mode a tracking level is, or `None` for [`MouseMode::Off`].
fn tracking(mode: MouseMode) -> Option<u32> {
    match mode {
        MouseMode::Off => None,
        MouseMode::Buttons => Some(1000),
        MouseMode::Drag => Some(1002),
        MouseMode::Motion => Some(1003),
    }
}

/// SGR mouse encoding, and it is **on whenever the mouse is on**.
///
/// Not a preference: without it the coordinates are a byte each, biased by 32, so a press stops
/// being reportable past column 223 — and the performance budget is written against a 300-column
/// screen. The parser reads no other encoding, which is the same decision from the other end.
const MODE_SGR_MOUSE: u32 = 1006;

/// `CSI ? m h` or `CSI ? m l`.
fn private_mode(out: &mut Vec<u8>, mode: u32, set: bool) {
    out.extend_from_slice(b"\x1b[?");
    push_num(out, mode);
    out.push(if set { b'h' } else { b'l' });
}

fn push_num(out: &mut Vec<u8>, mut n: u32) {
    let mut digits = [0u8; 10];
    let mut at = digits.len();
    loop {
        at -= 1;
        digits[at] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    out.extend_from_slice(&digits[at..]);
}

/// Switch the terminal from one tracking level to another.
///
/// The old mode is reset before the new one is set, because 1000, 1002 and 1003 are three modes and
/// not three values of one: leaving 1000 standing under 1003 works on xterm and is a bet on every
/// other implementation agreeing about which one wins.
pub(crate) fn write_mouse(out: &mut Vec<u8>, from: MouseMode, to: MouseMode) {
    if let Some(old) = tracking(from) {
        private_mode(out, old, false);
    }
    match tracking(to) {
        Some(new) => {
            // Before the tracking mode rather than after it, so that no event can be encoded in the
            // form the parser does not read.
            if from == MouseMode::Off {
                private_mode(out, MODE_SGR_MOUSE, true);
            }
            private_mode(out, new, true);
        }
        None => private_mode(out, MODE_SGR_MOUSE, false),
    }
}

/// `DECSCUSR` — `CSI Ps SP q`.
pub(crate) fn write_shape(out: &mut Vec<u8>, shape: CursorShape) {
    out.extend_from_slice(b"\x1b[");
    push_num(out, shape.decscusr() as u32);
    out.extend_from_slice(b" q");
}

/// `DECTCEM` — `CSI ? 25 h` / `l`.
pub(crate) fn write_visibility(out: &mut Vec<u8>, show: bool) {
    private_mode(out, MODE_DECTCEM, show);
}

/// The caret's own mode, and the one mode here that is **never** detected.
///
/// Every terminal that draws a caret can hide it, there is no query for it in the probe set, and
/// an engine that asked would be asking a question whose answer it would ignore. So it is written
/// unconditionally — including into a caller-supplied sink, which is what makes the caret's gates
/// reachable from a headless test where the mouse's are not.
const MODE_DECTCEM: u32 = 25;

/// Whose page the negotiation is about to be written onto.
///
/// **The whole decision, as two words.** The alternate screen is entered exactly
/// once per session and the only question is *by whom*: by [`crate::detect::batch`], ahead of the
/// first capability question, whenever there is a terminal to ask; and by [`negotiation`] itself
/// when there is not — a caller-supplied sink, a non-tty, `TERM=dumb`, and
/// [`Screen::resume`](crate::Screen::resume), which gave the page back and is taking it again.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Page {
    /// Still the user's own screen: the negotiation's first bytes are `?1049h`.
    Users,
    /// Already ours, because the capability batch entered it. The negotiation adds no second
    /// `?1049h`, and the count is a gate — see `crate::gates::nothing_reaches_the_users_own_page`.
    Ours,
}

/// What goes out once, at `attach`, after raw mode and the capability batch and before the render
/// thread exists.
///
/// # What is asked for and what is not
///
/// **The kitty enhancement flags are pushed as one set, never cherry-picked**, per kitty's own
/// instruction that implementing a subset makes no sense — and they cost nothing in idle, so there
/// is no reason to ask for less than the protocol has. The push is skipped entirely on a terminal
/// that answered no flags at all, because there is nobody to push to.
///
/// **Mouse tracking, focus reporting and bracketed paste are requested only if the application
/// declared them.** Each converts an idle application into a woken one — focus reporting on every
/// alt-tab, motion tracking on every pointer move — and standing requirement 11 is that where the
/// engine can idle it idles completely. **Focus reporting is opt-in and off by default**, which is
/// the one of the three the spec names on its own.
///
/// **The caret is hidden**, because nothing has asked for one and a cursor left wherever the last
/// write landed is a caret the application did not put there.
///
/// # What is not here
///
/// **DEC mode 2048** — in-band resize reports — would close the one open consequence, that a
/// terminal resized while the application is completely idle is not noticed until the next byte
/// arrives. It is not requested, and the reason is that requesting it is the cheap half: the report
/// arrives as a `CSI 48 ; … t` nothing parses, so it would land in
/// [`InputDiagnostics`](crate::InputDiagnostics) as an unrecognised sequence per resize, and the
/// mode has no DECRQM answer in the capability batch to tell whether it took. That is a detection
/// axis, a parser arm and a `Capabilities` field, and none of the three is free.
pub(crate) fn negotiation(config: &InputConfig, caps: &Capabilities, page: Page) -> Vec<u8> {
    let mut out = Vec::with_capacity(64);
    // **First, because everything after it is a mode set on the page this enters.** The alt screen
    // is what makes a full-screen application a full-screen application: the user's scrollback is
    // untouched, and leaving it puts their shell back exactly as they left it rather than eight
    // hundred lines further down.
    //
    // **And it is written here only when nobody has entered it yet**. On a
    // real terminal the page is already ours — [`crate::detect::batch`] enters it ahead of the first
    // question, because a terminal that *prints* a sequence it does not implement would otherwise
    // leave the probe's text on the user's shell screen, where nothing this engine does afterwards
    // can reach it. `?1049h` twice is not free: a terminal without xterm's *already on the alternate
    // buffer* guard would save the cursor a second time, and give the user's shell back at the wrong
    // one.
    if page == Page::Users {
        out.extend_from_slice(ENTER_ALT_SCREEN);
    } else {
        // **And when the batch opened it, the page is erased**, because `?1049h` clears the
        // alternate screen on the way in and the batch's questions go out *after* that clear. On a
        // terminal that prints what it cannot parse, what is sitting at the home position is the
        // artefact rather than blank — and nothing else here would remove it. The mirror starts
        // `Cell::UNKNOWN` everywhere, so a cell **no layer covers** is never damaged and never
        // written, and an application whose layers do not tile the screen would keep the artefact in
        // its gaps for the whole session.
        //
        // **This is not the erase that was refused**, and the difference is which page.
        // What was refused is `CSI 2 J` on the *primary* screen, as a substitute for switching pages
        // — it destroys the user's scrollback view and the engine cannot know how many cells to
        // repair because it cannot know which sequences a terminal will print. This is our own page,
        // switched to eight bytes earlier, where there is no scrollback and the whole point is that
        // the extent does not have to be known.
        //
        // The one terminal it costs anything on is one that implements neither mode 1049 **nor** the
        // sequences in the batch: it is on its own page, it printed the artefacts there, and this
        // clears it. Stated rather than hidden — it is about to be painted over by the first frame
        // regardless, inline rendering is out of scope, and Terminal.app 2.15, which is
        // the terminal this whole ticket is about, demonstrably implements 1049.
        out.extend_from_slice(ERASE_PAGE);
    }
    // Auto-wrap off, once, for the lifetime of the session. Every `shortest` move in the
    // serializer is priced on the assumption that nothing wrapped.
    out.extend_from_slice(DISABLE_AUTO_WRAP);
    write_visibility(&mut out, false);
    if caps.kitty_flags() != 0 {
        out.extend_from_slice(b"\x1b[>");
        push_num(&mut out, KITTY_ALL);
        out.push(b'u');
    }
    let floor = config.mouse.min(ceiling(caps));
    // Guarded, because `write_mouse(Off, Off)` still resets the SGR encoding — one mode nobody set,
    // on a terminal that may not have a mouse at all.
    if floor != MouseMode::Off {
        write_mouse(&mut out, MouseMode::Off, floor);
    }
    if config.focus && caps.focus_events {
        private_mode(&mut out, MODE_FOCUS, true);
    }
    if config.paste && caps.bracketed_paste {
        private_mode(&mut out, MODE_BRACKETED_PASTE, true);
    }
    out
}

/// Give back everything [`negotiation`] took, in the order it took it.
///
/// These are the **bytes**; [`crate::shutdown`] is what decides who writes them and how often, and
/// the split is the reason the bytes stayed here: whatever changed a mode is what knows how to
/// change it back, and *restoration includes input state* is not satisfied by a list somewhere
/// else. What the site adds is the atomic — a panic hook runs on whichever thread panicked, so the
/// same sequence has to be reachable from a thread that owns none of this.
///
/// # `mouse_on` is a boolean, and every tracking mode is reset
///
/// It was a [`MouseMode`] first — *the level the terminal was last told* — and that turned out not
/// to be knowable. The level the app thread holds is the one the **next packet** will ask for; the
/// bytes that change the mode are written later, by the render thread, so there is a window in which
/// the terminal is in either the old mode or the new one and nothing on the app thread can say
/// which. Resetting only the level this side believes in leaves the other one standing, and a shell
/// that outlives the crash goes on receiving mouse reports — which is exactly the property register
/// entry #15 is about.
///
/// So all three tracking modes go off whenever the session asked for a mouse at all, and the extra
/// eighteen bytes are spent once, at shutdown. **A session that never asked says nothing**, which is
/// the rule that matters here: a mode this process did not set is a mode it may not clear, because
/// the person at the terminal may have set it for their own reasons.
pub(crate) fn restoration(config: &InputConfig, caps: &Capabilities, mouse_on: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(64);
    if config.paste && caps.bracketed_paste {
        private_mode(&mut out, MODE_BRACKETED_PASTE, false);
    }
    if config.focus && caps.focus_events {
        private_mode(&mut out, MODE_FOCUS, false);
    }
    if mouse_on {
        // Highest first, which is only for a reader: they are three independent modes and the
        // terminal does not care about the order they go off in.
        for level in [MouseMode::Motion, MouseMode::Drag, MouseMode::Buttons] {
            private_mode(&mut out, tracking(level).expect("not Off"), false);
        }
        private_mode(&mut out, MODE_SGR_MOUSE, false);
    }
    if caps.kitty_flags() != 0 {
        // Pop, which is the protocol's own undo: the push at startup was one entry on the
        // terminal's stack and this is that entry leaving.
        out.extend_from_slice(b"\x1b[<u");
    }
    write_visibility(&mut out, true);
    out.extend_from_slice(ENABLE_AUTO_WRAP);
    // **Last, and after auto-wrap.** Both halves of that are load-bearing. Last, because everything
    // above it is a mode this session set and the terminal keeps modes across the switch — leaving
    // first would put the user's shell back and then go on resetting things behind their prompt.
    // And after auto-wrap specifically, because DECAWM is the one mode a shell notices immediately:
    // a shell whose line editor cannot wrap is a shell that overwrites its own prompt.
    out.extend_from_slice(LEAVE_ALT_SCREEN);
    out
}

/// Mode 1004.
const MODE_FOCUS: u32 = 1004;
/// Mode 2004.
const MODE_BRACKETED_PASTE: u32 = 2004;

/// DECAWM off. Once, on entering the alt screen.
pub(crate) const DISABLE_AUTO_WRAP: &[u8] = b"\x1b[?7l";
/// DECAWM on, which is what the terminal had before this process took it.
pub(crate) const ENABLE_AUTO_WRAP: &[u8] = b"\x1b[?7h";

/// Mode 1049 — the alternate screen buffer, with the cursor saved and the page cleared.
///
/// 1049 rather than 47 or 1047: it is the composite that saves the cursor position on the way in and
/// restores it on the way out, which is the difference between a shell prompt that comes back where
/// the user left it and one that comes back somewhere down the page. Every terminal this engine
/// targets has it and there is no query for it in the probe set, so it goes out unconditionally —
/// the same reasoning [`MODE_DECTCEM`] is written unconditionally for.
pub(crate) const ENTER_ALT_SCREEN: &[u8] = b"\x1b[?1049h";
/// `ED 2` — erase the whole page, cursor left where it is.
///
/// Written once, on the arm where [`crate::detect::batch`] opened the page and then printed
/// questions onto it. Four bytes, on our own screen, and the serializer needs
/// nothing from it: the mirror is `Cell::UNKNOWN` at birth, so this makes the terminal *more* like
/// what the mirror will conservatively assume rather than less.
pub(crate) const ERASE_PAGE: &[u8] = b"\x1b[2J";
/// Mode 1049 off: the user's own screen, their scrollback and their cursor, back.
pub(crate) const LEAVE_ALT_SCREEN: &[u8] = b"\x1b[?1049l";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caps::Capabilities;

    fn text(bytes: &[u8]) -> String {
        String::from_utf8_lossy(bytes).replace('\x1b', "^[")
    }

    /// **The ordering is a design decision rather than an accident of numbering.** Each level
    /// strictly contains the one below, so combining what several components want is a `max` and not
    /// a set union — and the test says so with three components rather than with two, because two
    /// values cannot tell a `max` from a pick.
    #[test]
    fn combining_three_components_declarations_is_a_max_and_not_a_union() {
        let declared = [MouseMode::Buttons, MouseMode::Motion, MouseMode::Off];
        let combined = declared.iter().copied().max().expect("three of them");
        assert_eq!(combined, MouseMode::Motion);

        // A union would be *these three modes*, which is three escape sequences and a terminal in a
        // state no single mode describes. There is one mode, and it is the largest.
        assert!(MouseMode::Off < MouseMode::Buttons);
        assert!(MouseMode::Buttons < MouseMode::Drag);
        assert!(MouseMode::Drag < MouseMode::Motion);
    }

    /// SGR encoding is on **whenever the mouse is on**, and off with it.
    #[test]
    fn sgr_encoding_arrives_with_the_mouse_and_leaves_with_it() {
        for level in [MouseMode::Buttons, MouseMode::Drag, MouseMode::Motion] {
            let mut on = Vec::new();
            write_mouse(&mut on, MouseMode::Off, level);
            assert!(
                text(&on).contains("?1006h"),
                "{level:?} went on without SGR: {}",
                text(&on)
            );

            let mut off = Vec::new();
            write_mouse(&mut off, level, MouseMode::Off);
            assert!(
                text(&off).contains("?1006l"),
                "{level:?} went off and left SGR standing: {}",
                text(&off)
            );
        }
    }

    /// A level change resets the old mode as well as setting the new one, and does **not** ask for
    /// the encoding twice.
    #[test]
    fn a_level_change_resets_the_old_mode_and_asks_for_sgr_once() {
        let mut out = Vec::new();
        write_mouse(&mut out, MouseMode::Buttons, MouseMode::Motion);
        let seen = text(&out);
        assert_eq!(seen, "^[[?1000l^[[?1003h", "{seen}");
    }

    /// `Off` to `Off` is nothing at all, which is the property the whole zero-byte gate rests on.
    #[test]
    fn no_change_writes_nothing() {
        let mut out = Vec::new();
        write_mouse(&mut out, MouseMode::Off, MouseMode::Off);
        // `Off -> Off` is the one pair `write_mouse` is never called with, because the delta is
        // computed first — and it still has to be harmless, since it is one `if` away.
        assert_eq!(text(&out), "^[[?1006l");
    }

    /// The clamp, from the top: what the terminal did not answer for cannot be asked for.
    #[test]
    fn a_terminal_without_a_mouse_clamps_every_level_to_off() {
        let caps = Capabilities::with_input(false, false, false, false, 0);
        let mut a = Actuators::new(
            &InputConfig {
                mouse: MouseMode::Motion,
                ..InputConfig::default()
            },
            &caps,
        );
        a.set_mouse(MouseMode::Motion);
        assert!(
            a.pending().mouse.is_none(),
            "nothing to say to this terminal"
        );
        assert_eq!(a.mouse(), MouseMode::Off);
    }

    /// Buttons but no motion: `Motion` is clamped to `Drag`, which is mode 1002 and has no query of
    /// its own. See [`ceiling`].
    #[test]
    fn a_terminal_with_buttons_and_no_motion_clamps_to_drag() {
        let caps = Capabilities::with_input(true, false, false, false, 0);
        let mut a = Actuators::new(&InputConfig::default(), &caps);
        a.set_mouse(MouseMode::Motion);
        assert_eq!(a.pending().mouse, Some((MouseMode::Off, MouseMode::Drag)));
    }

    /// The floor holds against a caller asking for less.
    #[test]
    fn the_declared_level_is_a_floor_and_a_lower_request_does_not_lower_it() {
        let caps = Capabilities::with_input(true, true, false, false, 0);
        let mut a = Actuators::new(
            &InputConfig {
                mouse: MouseMode::Drag,
                ..InputConfig::default()
            },
            &caps,
        );
        assert!(
            a.pending().mouse.is_none(),
            "the negotiation already switched the floor on"
        );
        a.set_mouse(MouseMode::Off);
        assert!(a.pending().mouse.is_none(), "a floor is not a setting");
        a.set_mouse(MouseMode::Motion);
        assert_eq!(
            a.pending().mouse,
            Some((MouseMode::Drag, MouseMode::Motion))
        );
    }

    /// A caret that only moved costs a move: no shape, no visibility. **This is the property a
    /// 29-byte caret frame is 29 bytes because of.**
    #[test]
    fn a_caret_that_only_moved_says_only_that() {
        let caps = Capabilities::with_input(false, false, false, false, 0);
        let mut a = Actuators::new(&InputConfig::default(), &caps);
        a.set_cursor(
            Some(Cursor {
                x: 4,
                y: 2,
                shape: CursorShape::Bar,
            }),
            (80, 24),
        );
        let first = a.pending();
        assert_eq!(first.show, Some(true));
        assert_eq!(first.shape, Some(CursorShape::Bar));
        assert!(first.moved);
        a.handed();

        a.set_cursor(
            Some(Cursor {
                x: 5,
                y: 2,
                shape: CursorShape::Bar,
            }),
            (80, 24),
        );
        let moved = a.pending();
        assert_eq!(moved.show, None, "it was already visible");
        assert_eq!(moved.shape, None, "it was already a bar");
        assert!(moved.moved);
        a.handed();

        let same = a.pending();
        assert!(
            same.is_quiet(),
            "nothing changed, so there is nothing to say"
        );
        assert_eq!(
            same.caret,
            Some(Cursor {
                x: 5,
                y: 2,
                shape: CursorShape::Bar
            }),
            "and the caret is still carried, because a frame that writes moves the cursor"
        );
    }

    /// Hiding a caret does not put its shape back, so the shape has to be remembered on its own —
    /// otherwise showing it again re-emits `DECSCUSR` for a shape the terminal already has.
    #[test]
    fn showing_a_caret_again_does_not_re_emit_the_shape_it_already_has() {
        let caps = Capabilities::with_input(false, false, false, false, 0);
        let mut a = Actuators::new(&InputConfig::default(), &caps);
        let bar = Cursor {
            x: 1,
            y: 1,
            shape: CursorShape::Bar,
        };
        a.set_cursor(Some(bar), (80, 24));
        a.handed();
        a.set_cursor(None, (80, 24));
        assert_eq!(a.pending().show, Some(false));
        a.handed();
        a.set_cursor(Some(bar), (80, 24));
        let back = a.pending();
        assert_eq!(back.show, Some(true));
        assert_eq!(back.shape, None, "the terminal is still set to a bar");
    }

    /// A caret off the edge is clamped, not refused: the signature has nothing to refuse with.
    #[test]
    fn a_caret_off_the_screen_is_clamped() {
        let caps = Capabilities::with_input(false, false, false, false, 0);
        let mut a = Actuators::new(&InputConfig::default(), &caps);
        a.set_cursor(
            Some(Cursor {
                x: 900,
                y: 900,
                shape: CursorShape::Terminal,
            }),
            (80, 24),
        );
        assert_eq!(a.pending().caret.map(|c| (c.x, c.y)), Some((79, 23)));
    }

    /// Nothing declared is nothing asked for, and the caret is hidden either way.
    #[test]
    fn an_application_that_declared_nothing_asks_for_nothing() {
        let caps = Capabilities::with_input(true, true, true, true, KITTY_ALL);
        let out = negotiation(&InputConfig::default(), &caps, Page::Users);
        let seen = text(&out);
        assert!(!seen.contains("1000"), "{seen}");
        assert!(!seen.contains("1004"), "focus reporting is opt-in: {seen}");
        assert!(!seen.contains("2004"), "{seen}");
        assert!(
            seen.contains("?25l"),
            "the caret is hidden until asked for: {seen}"
        );
    }

    /// The flags go as one set. **Never cherry-picked**, even when only some of them stuck.
    #[test]
    fn the_kitty_flags_go_as_one_set() {
        let partial = Capabilities::with_input(false, false, false, false, 0b0_0011);
        let seen = text(&negotiation(&InputConfig::default(), &partial, Page::Users));
        assert!(
            seen.contains(&format!("^[[>{KITTY_ALL}u")),
            "the whole stack, not the two that stuck: {seen}"
        );
    }

    /// A terminal that answered no flags at all is not pushed to.
    #[test]
    fn a_terminal_with_no_kitty_protocol_is_not_pushed_to() {
        let caps = Capabilities::with_input(false, false, false, false, 0);
        let seen = text(&negotiation(&InputConfig::default(), &caps, Page::Users));
        assert!(!seen.contains('u'), "{seen}");
    }

    /// Everything the negotiation took, given back in the order it was taken.
    #[test]
    fn the_epilogue_undoes_the_prologue() {
        let caps = Capabilities::with_input(true, true, true, true, KITTY_ALL);
        let config = InputConfig {
            mouse: MouseMode::Buttons,
            focus: true,
            paste: true,
            ..InputConfig::default()
        };
        let prologue = text(&negotiation(&config, &caps, Page::Users));
        assert_eq!(
            prologue,
            format!("^[[?1049h^[[?7l^[[?25l^[[>{KITTY_ALL}u^[[?1006h^[[?1000h^[[?1004h^[[?2004h"),
            "{prologue}"
        );
        // The level the application raised it to, not the one it started at.
        let epilogue = text(&restoration(&config, &caps, true));
        assert_eq!(
            epilogue,
            "^[[?2004l^[[?1004l^[[?1003l^[[?1002l^[[?1000l^[[?1006l^[[<u^[[?25h^[[?7h^[[?1049l",
            "{epilogue}"
        );
    }

    /// **The page is entered once per session and the negotiation is not always who does it.**
    ///
    /// On a real terminal `crate::detect::batch` has already switched the page
    /// before the first question went out, so a `?1049h` here would be a second one. What that arm
    /// writes instead is `ED 2`, because the questions went out *after* `?1049h` cleared the page and
    /// a terminal that prints one has left it on our own screen. The epilogue is unchanged either
    /// way — what a session leaves is a page it is on, however it got there.
    #[test]
    fn a_page_that_is_already_ours_is_not_entered_twice() {
        let caps = Capabilities::with_input(true, true, true, true, KITTY_ALL);
        let config = InputConfig {
            mouse: MouseMode::Buttons,
            focus: true,
            paste: true,
            ..InputConfig::default()
        };
        let ours = text(&negotiation(&config, &caps, Page::Ours));
        assert!(
            !ours.contains("1049"),
            "the batch entered the page and the negotiation entered it again: {ours}"
        );
        assert!(
            ours.starts_with("^[[2J"),
            "the batch printed its questions onto the page after `?1049h` cleared it, so the page \
             is erased once and nothing else will do it: {ours}"
        );
        let users = text(&negotiation(&config, &caps, Page::Users));
        assert_eq!(
            users,
            format!("^[[?1049h{}", &ours["^[[2J".len()..]),
            "the two arms differ by exactly one opening move — `?1049h` on a page nobody has \
             touched, `ED 2` on one the batch has — and by nothing else"
        );
        // And the restoration does not ask, because there is nothing to ask: a session on the
        // alternate screen leaves it, and which of the two wrote the `h` is not a fact it needs.
        assert!(text(&restoration(&config, &caps, true)).ends_with("^[[?1049l"));
    }

    /// A declared mode the terminal cannot do is not asked for, and is not restored either.
    #[test]
    fn a_declaration_the_terminal_cannot_honour_reaches_the_wire_nowhere() {
        let caps = Capabilities::with_input(false, false, false, false, 0);
        let config = InputConfig {
            mouse: MouseMode::Motion,
            focus: true,
            paste: true,
            ..InputConfig::default()
        };
        assert_eq!(
            text(&negotiation(&config, &caps, Page::Users)),
            "^[[?1049h^[[?7l^[[?25l"
        );
        assert_eq!(
            text(&restoration(&config, &caps, false)),
            "^[[?25h^[[?7h^[[?1049l"
        );
    }
}
