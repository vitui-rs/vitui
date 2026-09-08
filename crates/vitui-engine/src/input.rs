//! What the terminal reported, and nothing it did not.
//!
//! Six event variants, one owned type, no borrow across the seam.
//!
//! # The two conveniences that are deliberately absent
//!
//! Everyone reaches for a synthesised key release and a synthesised `mouse-enter`, and neither is
//! here. Three facts decide the keyboard and none of them is ours to fix: without kitty flag 2 a key
//! **release never arrives at all**; without it **auto-repeat is indistinguishable** from a fast
//! series of presses; and even at kitty baseline **Enter and Tab stay ambiguous** with Ctrl+M and
//! Ctrl+I, because the spec carves them out on purpose so that `reset` remains typeable after a
//! program crashes with the mode set.
//!
//! The tempting design papers over the first two — synthesise a [`KeyKind::Release`] when the next
//! press arrives, fold [`KeyKind::Repeat`] into [`KeyKind::Press`] — and it is refused for placement
//! rather than principle: **a synthesised release has no honest timestamp.** The key was released at
//! *some* moment between two presses, and this crate knows which second and not which millisecond. A
//! component drawing a held key would show it held until the next keystroke, which on an idle form is
//! forever. An application learns from [`Capabilities`](crate::Capabilities) which kinds it will
//! actually see — eight separate booleans, never a tier — and branches once.
//!
//! # Intent is never dropped; position is
//!
//! **Consecutive mouse moves coalesce.** An intermediate pointer position carries no intent — it says
//! only where the pointer was on the way — and an unbounded queue fed by a waving hand is the one
//! path by which a terminal can eat an application's memory. A press, a release, a wheel turn, a
//! keystroke and a resize all express something the user meant and are **never** dropped.
//!
//! **The queue grows and nothing caps it.** A terminal without bracketed paste delivers a pasted
//! megabyte as key events, so the growth is real; it is bounded by the paste and drains itself.
//! Dropping the oldest events would repeal the rule above for a case that resolves in seconds.
//!
//! # Three permanent consequences, recorded rather than mitigated
//!
//! - **`mouse-enter` and `mouse-leave` do not exist on the wire** and cannot be engine events: the
//!   terminal sends positions, and turning a position into *entered this field* needs the widget
//!   index, which is the runtime's.
//! - **The pointer can cross a widget entirely between two frames and generate no enter or leave at
//!   all**, because moves coalesce and the frame clock holds them to the gap. Invisible by
//!   construction, since a highlight lasting zero frames cannot be seen.
//! - **No protocol has a "the mouse left the terminal" event**, so a hover highlight can stick.
//!   Synthesising one from focus loss was refused for the same reason as the synthetic release.
//!
//! The first and third are claims about other people's software and are unverified. They are
//! written here as the reason for a shape, not as facts about any terminal.
//!
//! # This module reaches for nothing in the crate, and that is load-bearing
//!
//! The types, the parser and the queue are `std` and nothing else — the thread that wires them to a
//! terminal is `crate::reader`, one level up. That is not tidiness: `tests/alloc.rs` installs the
//! counting allocator and is its own process, and it `#[path]`-includes this file so that **the
//! keystroke path is measured end to end rather than at whichever boundary a public constructor
//! happens to reach**. A `use crate::` anywhere below would drag the whole engine into that binary,
//! and the gate would have to be weakened to fit.

use std::borrow::Cow;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Instant;

pub(crate) mod parse {
    //! Bytes to events, incrementally, inventing nothing.
    //!
    //! # Why this is ours rather than crossterm's
    //!
    //! crossterm is kept for raw mode, the tty test and the terminal's size, and its own
    //! parser is unreachable for the one reason that decides it: it comes bundled with a reader.
    //! `crossterm::event::read` opens `/dev/tty` itself and registers `SIGWINCH` on the same poll, and
    //! there is no door into the parser without the fd. Detection already holds the one reader this
    //! process has (`crate::detect::Tty`), and a second reader of the same terminal steals bytes from
    //! the first — silently, and only under load.
    //!
    //! The other half of the same thing is that *the input thread is testable without a tty by
    //! feeding byte sequences to the parser*. A parser that owns no file descriptor is what makes
    //! five adversarial byte splits a cheap assertion rather than a pty fixture.
    //!
    //! # Incremental, because a pty splits where the kernel felt like splitting
    //!
    //! Same reasoning as `crate::detect`'s parser and the same failure mode: a parser that assumed whole
    //! sequences would work on every terminal that is fast and fail on every terminal that is slow. The
    //! state lives across [`Parser::feed`] calls, and every buffer it keeps is reused rather than
    //! reallocated, because a keystroke may not allocate.
    //!
    //! # The one place a read boundary means something
    //!
    //! A lone `ESC` is the Escape key, and it is also the first byte of every other escape sequence.
    //! Nothing on the wire tells the two apart, so an implementation picks one of three costs:
    //!
    //! 1. **A timer** — hold the `ESC` for 25 ms and emit it if nothing follows. Refused: it is a timer
    //!    in a crate whose idle guarantee is `0.00 user 0.00 sys` over thirty seconds, bought
    //!    to disambiguate a keystroke.
    //! 2. **Hold it until the next read** — correct, and it makes Escape undeliverable on an idle
    //!    application until the user presses something else. Escape is how a dialog closes; forever is
    //!    not an acceptable latency for it.
    //! 3. **Flush it at the end of the read it arrived in** — wrong only when a terminal splits its own
    //!    write between the `ESC` and the byte after it, which requires the pty buffer to end at exactly
    //!    that offset.
    //!
    //! The third is taken, and it is what [`Parser::end_of_read`] is. It flushes **only** a bare `ESC`:
    //! a half-finished `CSI` is never guessed at, because that case has no ambiguity to resolve and the
    //! adversarial-split gate is about exactly it.

    use std::time::Instant;

    use super::{
        Button, Buttons, Event, Key, KeyCode, KeyKind, KeyText, Keypad, Media, Modifier, Mods,
        Mouse, MouseKind, Paste, Wheel,
    };

    const ESC: u8 = 0x1b;

    /// Where in a sequence the byte stream is.
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
    enum State {
        #[default]
        Ground,
        /// `ESC` seen, and nothing after it yet. The one state [`Parser::end_of_read`] resolves.
        Esc,
        /// `ESC [`, collecting parameters and intermediates.
        Csi,
        /// `ESC O`, the single-shift-3 introducer: F1..F4 and the application cursor keys.
        Ss3,
        /// A string sequence — OSC, DCS, APC, PM, SOS. Nothing in the batch survives to here after
        /// detection, so all of it is unrecognised; it is still consumed to its terminator, because
        /// treating its body as keystrokes is how a colour reply becomes forty phantom key presses.
        String,
        /// An `ESC` inside a string sequence: `ESC \` ends it, anything else aborts it.
        StringEsc,
        /// Inside `CSI 200 ~` … `CSI 201 ~`.
        Paste,
        /// A multi-byte UTF-8 scalar, part way through.
        Utf8,
    }

    /// The end of a bracketed paste, byte for byte.
    const PASTE_END: &[u8] = b"\x1b[201~";
    /// The start, matched as a `CSI` sequence rather than by bytes.
    const PASTE_START_PARAM: u32 = 200;
    const PASTE_END_PARAM: u32 = 201;

    /// One terminal's input stream, turned into events.
    ///
    /// Owns no file descriptor and no clock: `at` is passed in by the caller, which is what lets the
    /// gates assert on timestamps without owning real time.
    pub(crate) struct Parser {
        state: State,
        /// `CSI` parameters and intermediates, or the `String` body. Reused across sequences.
        buf: Vec<u8>,
        /// The bytes of a paste, up to the ceiling.
        paste: Vec<u8>,
        /// Whether the paste being collected has already hit the ceiling.
        paste_truncated: bool,
        /// How many bytes of [`PASTE_END`] have matched so far.
        paste_end_at: usize,
        /// The ceiling on one paste, in bytes.
        paste_limit: usize,
        /// A partial UTF-8 scalar: the bytes so far and how many the leader promised.
        utf8: [u8; 4],
        utf8_at: usize,
        utf8_len: usize,
        /// Which mouse buttons are down, carried across events because the wire reports transitions
        /// — and reconciled from every motion report, which carries the same bits.
        buttons: Buttons,
        /// Whether the `CSI` being collected ran past [`MAX_SEQUENCE`] and can no longer be trusted
        /// to mean what its remaining parameters say.
        csi_overflowed: bool,
        /// How many sequences were dropped because nothing here recognised them.
        unrecognised: u64,
        /// The last of them, kept whole. Two fields buy the thread a "Shift+F5 does nothing" report
        /// would otherwise have none of.
        last_unrecognised: Vec<u8>,
    }

    impl Parser {
        /// A parser with the paste ceiling `paste_limit` bytes.
        pub(crate) fn new(paste_limit: usize) -> Parser {
            Parser {
                state: State::default(),
                buf: Vec::with_capacity(32),
                paste: Vec::new(),
                paste_truncated: false,
                paste_end_at: 0,
                paste_limit,
                utf8: [0; 4],
                utf8_at: 0,
                utf8_len: 0,
                buttons: Buttons::NONE,
                csi_overflowed: false,
                unrecognised: 0,
                last_unrecognised: Vec::new(),
            }
        }

        /// How many sequences nothing here recognised, and the last of them.
        pub(crate) fn diagnostics(&self) -> (u64, &[u8]) {
            (self.unrecognised, &self.last_unrecognised)
        }

        /// Feed one read's worth of bytes. Every event it completes goes to `sink`, in wire order.
        ///
        /// `at` is stamped onto every event this call produces: it is the moment the **read** returned,
        /// which is the last moment anything in this process knows about. Splitting it finer would be
        /// inventing a timestamp per byte.
        pub(crate) fn feed(&mut self, bytes: &[u8], at: Instant, sink: &mut dyn FnMut(Event)) {
            for &b in bytes {
                self.step(b, at, sink);
            }
        }

        /// The read is over. Resolve a bare `ESC` into the Escape key.
        ///
        /// See the module documentation: this is the one place a read boundary carries meaning, and it
        /// resolves **only** [`State::Esc`]. A `CSI` that is half arrived stays half arrived.
        pub(crate) fn end_of_read(&mut self, at: Instant, sink: &mut dyn FnMut(Event)) {
            if self.state == State::Esc {
                self.state = State::Ground;
                sink(key(KeyCode::Escape, Mods::NONE, KeyText::EMPTY, at));
            }
        }

        /// How many bytes this parser is holding, and how many it has reserved to hold them.
        ///
        /// Both halves, because the buffers are **reused** rather than freed: one that was cleared
        /// and not shrunk has a length of zero and every byte it ever saw still committed. The
        /// second fuzz target has *no unbounded growth* as one of its three oracles, and that is a claim
        /// about the second number as much as the first.
        ///
        /// The `utf8` array is not counted: it is four inline bytes and it is why the parser is
        /// allowed to reassemble a scalar across a read without allocating.
        #[cfg(any(test, feature = "fuzz"))]
        pub(crate) fn retained(&self) -> (usize, usize) {
            let held = self.buf.len() + self.paste.len() + self.last_unrecognised.len();
            let reserved =
                self.buf.capacity() + self.paste.capacity() + self.last_unrecognised.capacity();
            (held + self.utf8_at, reserved)
        }

        /// The most this parser may ever be holding, whatever arrives and however much of it.
        ///
        /// [`MAX_SEQUENCE`] twice — once for the sequence being collected and once for the last
        /// unrecognised one kept beside it, which carries its own `ESC [` and final byte — plus the
        /// paste ceiling and the four bytes of a partial scalar. **Nothing in it is a function of
        /// how many bytes have been fed**, which is the whole content of the claim: a `String` state
        /// that kept appending, or a paste that stopped checking its ceiling, exceeds this on a long
        /// input and is invisible on every short one.
        #[cfg(any(test, feature = "fuzz"))]
        pub(crate) fn retention_bound(&self) -> usize {
            2 * (MAX_SEQUENCE + 4) + self.paste_limit + 4
        }

        fn step(&mut self, b: u8, at: Instant, sink: &mut dyn FnMut(Event)) {
            match self.state {
                State::Ground => self.ground(b, at, sink),
                State::Esc => {
                    self.buf.clear();
                    match b {
                        b'[' => self.state = State::Csi,
                        b'O' => self.state = State::Ss3,
                        b']' | b'P' | b'^' | b'_' | b'X' => {
                            self.buf.push(ESC);
                            self.buf.push(b);
                            self.state = State::String;
                        }
                        // `ESC ESC` is Escape with Alt held on every terminal that sends a meta prefix,
                        // and it is also two Escapes on one that does not. The prefix reading wins
                        // because the double press is the rarer thing a user does deliberately.
                        ESC => {
                            self.state = State::Ground;
                            sink(key(KeyCode::Escape, Mods::ALT, KeyText::EMPTY, at));
                        }
                        // The meta prefix: `ESC` then an ordinary key means Alt is held. Every legacy
                        // terminal spells Alt this way and none of them spells it any other way.
                        _ => {
                            self.state = State::Ground;
                            self.ground_with(b, Mods::ALT, at, sink);
                        }
                    }
                }
                State::Csi => {
                    // **`ESC` aborts a control sequence; it is not a parameter byte.** The `String`
                    // state has said so since this file was written and this one did not, which was
                    // the same defect in the shape that costs a keystroke rather than a screen: a
                    // truncated `ESC [ 1 ; 5` followed by a real `Ctrl+Up` parsed `1;5[` as one
                    // unrecognised sequence and then typed the `A` into the application as a capital
                    // letter. ECMA-48 executes the other C0 controls and stays in the sequence, and
                    // so does this.
                    if b == ESC {
                        self.state = State::Ground;
                        self.abandon_csi();
                        return self.step(b, at, sink);
                    }
                    if (0x40..=0x7e).contains(&b) {
                        self.state = State::Ground;
                        if std::mem::take(&mut self.csi_overflowed) {
                            // The parameters were cut, so what is left may well *parse* — a
                            // truncated `1;5` under a final `A` is a perfectly good `Ctrl+Up` that
                            // the terminal never sent. The flag is what refuses it.
                            self.buf.push(b);
                            return self.unrecognised();
                        }
                        let payload = std::mem::take(&mut self.buf);
                        self.csi(&payload, b, at, sink);
                        self.buf = payload;
                        self.buf.clear();
                    } else if self.buf.len() < MAX_SEQUENCE {
                        self.buf.push(b);
                    } else {
                        // A `CSI` that never ends is a broken stream, not a key. Stop growing the
                        // buffer and **keep consuming to the final byte**: dropping to Ground here
                        // was the first draft, and it emitted the tail of a runaway sequence as key
                        // presses — the forty phantom keystrokes the `String` state exists to
                        // prevent, from the other door.
                        self.csi_overflowed = true;
                    }
                }
                State::Ss3 => {
                    self.state = State::Ground;
                    match b {
                        b'P' => sink(key(KeyCode::F(1), Mods::NONE, KeyText::EMPTY, at)),
                        b'Q' => sink(key(KeyCode::F(2), Mods::NONE, KeyText::EMPTY, at)),
                        b'R' => sink(key(KeyCode::F(3), Mods::NONE, KeyText::EMPTY, at)),
                        b'S' => sink(key(KeyCode::F(4), Mods::NONE, KeyText::EMPTY, at)),
                        b'A' => sink(key(KeyCode::Up, Mods::NONE, KeyText::EMPTY, at)),
                        b'B' => sink(key(KeyCode::Down, Mods::NONE, KeyText::EMPTY, at)),
                        b'C' => sink(key(KeyCode::Right, Mods::NONE, KeyText::EMPTY, at)),
                        b'D' => sink(key(KeyCode::Left, Mods::NONE, KeyText::EMPTY, at)),
                        b'H' => sink(key(KeyCode::Home, Mods::NONE, KeyText::EMPTY, at)),
                        b'F' => sink(key(KeyCode::End, Mods::NONE, KeyText::EMPTY, at)),
                        _ => {
                            self.buf.clear();
                            self.buf.extend_from_slice(&[ESC, b'O', b]);
                            self.unrecognised();
                        }
                    }
                }
                State::String => match b {
                    0x07 => self.finish_string(),
                    ESC => self.state = State::StringEsc,
                    _ => {
                        if self.buf.len() < MAX_SEQUENCE {
                            self.buf.push(b);
                        }
                    }
                },
                State::StringEsc => {
                    if b == b'\\' {
                        self.finish_string();
                    } else {
                        // Same rule as detection's parser, for the same reason: `ESC` aborts a string,
                        // it is not content. Absorbing it lets one keystroke swallow every byte after it.
                        self.state = State::Ground;
                        self.unrecognised();
                        self.step(b, at, sink);
                    }
                }
                State::Paste => self.paste_byte(b, at, sink),
                State::Utf8 => {
                    if b & 0b1100_0000 != 0b1000_0000 {
                        // Not a continuation byte. The scalar was cut off by whatever this is; the
                        // partial bytes are dropped and the new byte is parsed from Ground, because
                        // guessing a replacement character here would put a `U+FFFD` in somebody's
                        // password field.
                        // The **partial scalar** is what goes in the diagnostic, not whatever was
                        // last left in `buf`: a count with an unrelated byte string attached is a
                        // worse thread to pull than no byte string at all.
                        let partial = self.utf8_at;
                        self.utf8_at = 0;
                        self.state = State::Ground;
                        self.last_unrecognised.clear();
                        self.last_unrecognised
                            .extend_from_slice(&self.utf8[..partial]);
                        self.unrecognised += 1;
                        self.step(b, at, sink);
                        return;
                    }
                    self.utf8[self.utf8_at] = b;
                    self.utf8_at += 1;
                    if self.utf8_at == self.utf8_len {
                        self.state = State::Ground;
                        let bytes = &self.utf8[..self.utf8_len];
                        self.utf8_at = 0;
                        match std::str::from_utf8(bytes) {
                            Ok(s) => {
                                let c = s.chars().next().unwrap_or('\u{fffd}');
                                sink(key(KeyCode::Char(c), Mods::NONE, KeyText::from_str(s), at));
                            }
                            // A well-formed length over an ill-formed scalar — an encoded surrogate,
                            // an overlong form. **Its own bytes go in the diagnostic**, for the same
                            // reason as the truncated case above: `buf` is empty here, and a count
                            // with no bytes attached is a thread with nothing on the end of it.
                            Err(_) => {
                                let len = self.utf8_len;
                                self.last_unrecognised.clear();
                                self.last_unrecognised.extend_from_slice(&self.utf8[..len]);
                                self.unrecognised += 1;
                            }
                        }
                    }
                }
            }
        }

        fn ground(&mut self, b: u8, at: Instant, sink: &mut dyn FnMut(Event)) {
            if b == ESC {
                self.state = State::Esc;
                return;
            }
            self.ground_with(b, Mods::NONE, at, sink);
        }

        /// One byte of ordinary input, with `extra` already-known modifiers folded in.
        ///
        /// `extra` is `ALT` when a meta prefix put it there, and nothing otherwise. It is never `SHIFT`:
        /// a capital `A` on the wire says the terminal produced a capital `A` and says nothing at all
        /// about which key was pressed to produce it — on a Dvorak layout, on a dead-key sequence, or
        /// with caps lock on, the answer differs. Kitty's disambiguate flag is what makes `code` and
        /// `text` two different things; without it they are one thing, reported once in each field.
        fn ground_with(&mut self, b: u8, extra: Mods, at: Instant, sink: &mut dyn FnMut(Event)) {
            match b {
                // `NUL` is `Ctrl+@` by the same arithmetic as the four below and `Ctrl+Space` by
                // what people press. Space wins: the chord an application binds is the one a user
                // can find, and `@` needs a shift on most layouts.
                0x00 => sink(key(
                    KeyCode::Char(' '),
                    extra.with(Mods::CTRL),
                    KeyText::EMPTY,
                    at,
                )),
                0x08 => sink(key(
                    KeyCode::Backspace,
                    extra.with(Mods::CTRL),
                    KeyText::EMPTY,
                    at,
                )),
                b'\t' => sink(key(KeyCode::Tab, extra, KeyText::EMPTY, at)),
                b'\r' | b'\n' => sink(key(KeyCode::Enter, extra, KeyText::EMPTY, at)),
                0x7f => sink(key(KeyCode::Backspace, extra, KeyText::EMPTY, at)),
                // The C0 controls that are `Ctrl` plus a letter. Enter, Tab and Backspace are carved out
                // above because the terminal spells them the same way and the kitty spec keeps them
                // ambiguous on purpose, so that `reset` stays typeable after a crash.
                0x01..=0x1a => {
                    let c = (b - 1 + b'a') as char;
                    sink(key(
                        KeyCode::Char(c),
                        extra.with(Mods::CTRL),
                        KeyText::EMPTY,
                        at,
                    ));
                }
                // `Ctrl+X` is `X & 0x1f` for every `X` in `@A-Z[\]^_`, so the four above the
                // letters are the four punctuation keys and not the digits. **Digits is what the
                // first draft said**, on the folklore that `Ctrl+4` also sends `0x1c` — true on a US
                // layout and nowhere else, and `code` is supposed to be the base key.
                0x1c..=0x1f => {
                    let c = (b + 0x40) as char;
                    sink(key(
                        KeyCode::Char(c),
                        extra.with(Mods::CTRL),
                        KeyText::EMPTY,
                        at,
                    ));
                }
                0x20..=0x7e => {
                    let c = b as char;
                    let mut text = KeyText::EMPTY;
                    text.push_char(c);
                    sink(key(KeyCode::Char(c), extra, text, at));
                }
                _ => {
                    let len = utf8_len(b);
                    if len == 0 {
                        self.unrecognised_byte(b);
                        return;
                    }
                    self.utf8[0] = b;
                    self.utf8_at = 1;
                    self.utf8_len = len;
                    self.state = State::Utf8;
                }
            }
        }

        fn finish_string(&mut self) {
            self.state = State::Ground;
            self.unrecognised();
        }

        /// A finished `CSI`, dispatched by its final byte.
        fn csi(
            &mut self,
            payload: &[u8],
            final_byte: u8,
            at: Instant,
            sink: &mut dyn FnMut(Event),
        ) {
            // SGR mouse (mode 1006): `CSI < b ; x ; y M|m`, and it is the only encoding this engine ever
            // asks for, because without it coordinates stop at column 223.
            if payload.first() == Some(&b'<') && (final_byte == b'M' || final_byte == b'm') {
                if !self.sgr_mouse(&payload[1..], final_byte == b'M', at, sink) {
                    self.unrecognised_csi(payload, final_byte);
                }
                return;
            }
            if payload.first() == Some(&b'?') {
                // A mode report, arriving after detection is over. Nothing asked for it.
                return self.unrecognised_csi(payload, final_byte);
            }

            let params = Params::parse(payload);
            match final_byte {
                b'I' => sink(Event::FocusGained),
                b'O' => sink(Event::FocusLost),
                b'Z' => sink(key(KeyCode::BackTab, Mods::SHIFT, KeyText::EMPTY, at)),
                b'A' | b'B' | b'C' | b'D' | b'H' | b'F' | b'E' => {
                    let code = match final_byte {
                        b'A' => KeyCode::Up,
                        b'B' => KeyCode::Down,
                        b'C' => KeyCode::Right,
                        b'D' => KeyCode::Left,
                        b'H' => KeyCode::Home,
                        b'F' => KeyCode::End,
                        _ => KeyCode::Keypad(Keypad::Begin),
                    };
                    let (mods, kind) = params.modifiers(1);
                    sink(Event::Key(Key {
                        code,
                        mods,
                        kind,
                        text: KeyText::EMPTY,
                        at,
                    }));
                }
                b'~' => {
                    let n = params.at(0).unwrap_or(0);
                    if n == PASTE_START_PARAM {
                        self.paste.clear();
                        self.paste_truncated = false;
                        self.paste_end_at = 0;
                        self.state = State::Paste;
                        return;
                    }
                    if n == PASTE_END_PARAM {
                        // A close with no open. Nothing to hand up and nothing to guess at.
                        return self.unrecognised_csi(payload, final_byte);
                    }
                    match tilde_key(n) {
                        Some(code) => {
                            let (mods, kind) = params.modifiers(1);
                            sink(Event::Key(Key {
                                code,
                                mods,
                                kind,
                                text: KeyText::EMPTY,
                                at,
                            }));
                        }
                        None => self.unrecognised_csi(payload, final_byte),
                    }
                }
                // The kitty keyboard protocol: `CSI key:alt1:alt2 ; mods:event ; text… u`.
                b'u' => {
                    // Field one is `key:shifted:base-layout`, and **`code` is the base layout when
                    // the terminal sent one** (kitty flag 4). That is the whole reason `code` and
                    // `text` are two fields: on a Cyrillic layout the `ф` key reports `1092` as the
                    // key and `97` as its base layout, and an application binding `Ctrl+A` needs the
                    // second. The shifted key is dropped — it is what `text` already says.
                    let Some(primary) = params.at(0) else {
                        return self.unrecognised_csi(payload, final_byte);
                    };
                    let Some(code) = params
                        .sub(0, 2)
                        .and_then(functional)
                        .or_else(|| functional(primary))
                    else {
                        return self.unrecognised_csi(payload, final_byte);
                    };
                    let (mods, kind) = params.modifiers(1);
                    let mut text = KeyText::EMPTY;
                    // Field three is the *associated text*: the codepoints the key would have
                    // produced, which is a sequence and not a scalar (kitty flag 16). Inline, so a
                    // keystroke allocates nothing.
                    for cp in params.subs(2) {
                        match char::from_u32(cp) {
                            Some(c) => text.push_char(c),
                            None => text = KeyText::EMPTY,
                        }
                    }
                    // Without flag 16 the terminal sends no text field at all, and a printable key
                    // is its own text. Filling it in here is reporting rather than inventing: it is
                    // the same byte the legacy path would have delivered. **From the primary field
                    // and not from `code`** — `code` may now be the base layout, and a Cyrillic `ф`
                    // whose base layout is `a` must not be reported as having printed an `a`.
                    if text.is_empty()
                        && mods.is_typing()
                        && let Some(c) = char::from_u32(primary).filter(|c| !c.is_control())
                    {
                        text.push_char(c);
                    }
                    sink(Event::Key(Key {
                        code,
                        mods,
                        kind,
                        text,
                        at,
                    }));
                }
                _ => self.unrecognised_csi(payload, final_byte),
            }
        }

        /// `CSI < button ; x ; y M|m`. Answers whether it parsed.
        fn sgr_mouse(
            &mut self,
            body: &[u8],
            down: bool,
            at: Instant,
            sink: &mut dyn FnMut(Event),
        ) -> bool {
            let params = Params::parse(body);
            let (Some(raw), Some(x), Some(y)) = (params.at(0), params.at(1), params.at(2)) else {
                return false;
            };
            // The wire is 1-based and every rectangle in this crate is 0-based. A zero would underflow,
            // and a terminal that sends one is telling us about a cell that does not exist.
            let (Some(x), Some(y)) = (x.checked_sub(1), y.checked_sub(1)) else {
                return false;
            };
            let mods = Mods::from_sgr(raw);
            let motion = raw & 0b10_0000 != 0;
            let wheel = raw & 0b100_0000 != 0;
            let extended = raw & 0b1000_0000 != 0;
            let low = raw & 0b11;

            let kind = if wheel {
                match low {
                    0 => MouseKind::Wheel(Wheel::Up),
                    1 => MouseKind::Wheel(Wheel::Down),
                    2 => MouseKind::Wheel(Wheel::Left),
                    _ => MouseKind::Wheel(Wheel::Right),
                }
            } else if motion {
                // **The motion report carries the held button and `3` means none**, and taking it is
                // reporting rather than remembering. A `buttons` set that only ever grows turns
                // every later hover into a stuck drag: press inside the window, drag out, release
                // out — the terminal sends no release, and without this the left bit is set for the
                // rest of the session. Only the named button is added, because a report names one
                // and the user may be holding two.
                if !extended && low == 0b11 {
                    self.buttons = Buttons::NONE;
                } else {
                    self.buttons.press(button_of(extended, low));
                }
                MouseKind::Move
            } else {
                let button = button_of(extended, low);
                if down {
                    self.buttons.press(button);
                    MouseKind::Down(button)
                } else {
                    self.buttons.release(button);
                    MouseKind::Up(button)
                }
            };
            sink(Event::Mouse(Mouse {
                x: clamp_u16(x),
                y: clamp_u16(y),
                kind,
                buttons: self.buttons,
                mods,
                at,
            }));
            true
        }

        /// One byte inside a bracketed paste, watching for the closing marker.
        fn paste_byte(&mut self, b: u8, at: Instant, sink: &mut dyn FnMut(Event)) {
            if b == PASTE_END[self.paste_end_at] {
                self.paste_end_at += 1;
                if self.paste_end_at == PASTE_END.len() {
                    self.paste_end_at = 0;
                    self.state = State::Ground;
                    let bytes = std::mem::take(&mut self.paste);
                    sink(Event::Paste(Paste::new(bytes, self.paste_truncated, at)));
                    self.paste_truncated = false;
                }
                return;
            }
            // The match broke. Everything that had matched is ordinary pasted content after all, and it
            // has to go into the buffer in order before this byte does. `PASTE_END` starts with the only
            // `ESC` in it, so a restart can only begin at this byte and never inside what was matched.
            if self.paste_end_at > 0 {
                let matched = self.paste_end_at;
                self.paste_end_at = 0;
                for &b in &PASTE_END[..matched] {
                    self.push_pasted(b);
                }
                return self.paste_byte(b, at, sink);
            }
            self.push_pasted(b);
        }

        fn push_pasted(&mut self, b: u8) {
            if self.paste.len() < self.paste_limit {
                self.paste.push(b);
            } else {
                // **The flag is on the event, not in a diagnostic**: silent truncation is data loss
                // the user will attribute to the application.
                self.paste_truncated = true;
            }
        }

        /// A control sequence that never reached a final byte, counted with what there was of it.
        fn abandon_csi(&mut self) {
            self.csi_overflowed = false;
            self.last_unrecognised.clear();
            self.last_unrecognised.extend_from_slice(&[ESC, b'[']);
            self.last_unrecognised.extend_from_slice(&self.buf);
            self.buf.clear();
            self.unrecognised += 1;
        }

        fn unrecognised_csi(&mut self, payload: &[u8], final_byte: u8) {
            self.last_unrecognised.clear();
            self.last_unrecognised.push(ESC);
            self.last_unrecognised.push(b'[');
            self.last_unrecognised.extend_from_slice(payload);
            self.last_unrecognised.push(final_byte);
            self.unrecognised += 1;
        }

        fn unrecognised_byte(&mut self, b: u8) {
            self.last_unrecognised.clear();
            self.last_unrecognised.push(b);
            self.unrecognised += 1;
        }

        /// The sequence in `buf` was not recognised.
        fn unrecognised(&mut self) {
            self.last_unrecognised.clear();
            self.last_unrecognised.extend_from_slice(&self.buf);
            self.buf.clear();
            self.unrecognised += 1;
        }
    }

    /// The longest sequence anything here will hold before giving up on the stream.
    ///
    /// A `CSI` is a few dozen bytes at the outside — kitty's longest functional key with associated text
    /// and modifiers is under forty. This is the guard against a terminal that has gone wrong, not a
    /// limit anything real reaches.
    const MAX_SEQUENCE: usize = 256;

    fn key(code: KeyCode, mods: Mods, text: KeyText, at: Instant) -> Event {
        Event::Key(Key {
            code,
            mods,
            kind: KeyKind::Press,
            text,
            at,
        })
    }

    /// How many bytes the UTF-8 scalar starting with `b` occupies. `0` when it starts nothing.
    fn utf8_len(b: u8) -> usize {
        match b {
            0b1100_0000..=0b1101_1111 => 2,
            0b1110_0000..=0b1110_1111 => 3,
            0b1111_0000..=0b1111_0111 => 4,
            _ => 0,
        }
    }

    /// The button the SGR encoding's bits name, for a press, a release and a drag alike.
    fn button_of(extended: bool, low: u32) -> Button {
        match (extended, low) {
            (false, 0) => Button::Left,
            (false, 1) => Button::Middle,
            (false, 2) => Button::Right,
            (true, 0) => Button::Back,
            (true, 1) => Button::Forward,
            (_, n) => Button::Other(n as u8),
        }
    }

    fn clamp_u16(n: u32) -> u16 {
        n.min(u32::from(u16::MAX)) as u16
    }

    /// `CSI n ~`, the numbering every terminal shares and nobody documents in one place.
    fn tilde_key(n: u32) -> Option<KeyCode> {
        Some(match n {
            1 | 7 => KeyCode::Home,
            2 => KeyCode::Insert,
            3 => KeyCode::Delete,
            4 | 8 => KeyCode::End,
            5 => KeyCode::PageUp,
            6 => KeyCode::PageDown,
            11..=15 => KeyCode::F((n - 10) as u8),
            17..=21 => KeyCode::F((n - 11) as u8),
            23..=26 => KeyCode::F((n - 12) as u8),
            28 | 29 => KeyCode::F((n - 13) as u8),
            31..=34 => KeyCode::F((n - 14) as u8),
            _ => return None,
        })
    }

    /// A kitty `CSI number u` key code, which is a Unicode scalar for a text key and a private-use
    /// codepoint for everything else.
    fn functional(n: u32) -> Option<KeyCode> {
        Some(match n {
            // The four the kitty spec carves out on purpose, so that `reset` stays typeable.
            13 => KeyCode::Enter,
            9 => KeyCode::Tab,
            127 => KeyCode::Backspace,
            27 => KeyCode::Escape,
            57358 => KeyCode::CapsLock,
            57359 => KeyCode::ScrollLock,
            57360 => KeyCode::NumLock,
            57361 => KeyCode::PrintScreen,
            57362 => KeyCode::Pause,
            57363 => KeyCode::Menu,
            57376..=57398 => KeyCode::F((n - 57376 + 13) as u8),
            57399..=57408 => KeyCode::Keypad(Keypad::Digit((n - 57399) as u8)),
            57409 => KeyCode::Keypad(Keypad::Decimal),
            57410 => KeyCode::Keypad(Keypad::Divide),
            57411 => KeyCode::Keypad(Keypad::Multiply),
            57412 => KeyCode::Keypad(Keypad::Subtract),
            57413 => KeyCode::Keypad(Keypad::Add),
            57414 => KeyCode::Keypad(Keypad::Enter),
            57415 => KeyCode::Keypad(Keypad::Equal),
            57416 => KeyCode::Keypad(Keypad::Separator),
            57417 => KeyCode::Keypad(Keypad::Left),
            57418 => KeyCode::Keypad(Keypad::Right),
            57419 => KeyCode::Keypad(Keypad::Up),
            57420 => KeyCode::Keypad(Keypad::Down),
            57421 => KeyCode::Keypad(Keypad::PageUp),
            57422 => KeyCode::Keypad(Keypad::PageDown),
            57423 => KeyCode::Keypad(Keypad::Home),
            57424 => KeyCode::Keypad(Keypad::End),
            57425 => KeyCode::Keypad(Keypad::Insert),
            57426 => KeyCode::Keypad(Keypad::Delete),
            57427 => KeyCode::Keypad(Keypad::Begin),
            57428 => KeyCode::Media(Media::Play),
            57429 => KeyCode::Media(Media::Pause),
            57430 => KeyCode::Media(Media::PlayPause),
            57431 => KeyCode::Media(Media::Reverse),
            57432 => KeyCode::Media(Media::Stop),
            57433 => KeyCode::Media(Media::FastForward),
            57434 => KeyCode::Media(Media::Rewind),
            57435 => KeyCode::Media(Media::TrackNext),
            57436 => KeyCode::Media(Media::TrackPrevious),
            57437 => KeyCode::Media(Media::Record),
            57438 => KeyCode::Media(Media::LowerVolume),
            57439 => KeyCode::Media(Media::RaiseVolume),
            57440 => KeyCode::Media(Media::MuteVolume),
            57441 => KeyCode::Modifier(Modifier::LeftShift),
            57442 => KeyCode::Modifier(Modifier::LeftControl),
            57443 => KeyCode::Modifier(Modifier::LeftAlt),
            57444 => KeyCode::Modifier(Modifier::LeftSuper),
            57445 => KeyCode::Modifier(Modifier::LeftHyper),
            57446 => KeyCode::Modifier(Modifier::LeftMeta),
            57447 => KeyCode::Modifier(Modifier::RightShift),
            57448 => KeyCode::Modifier(Modifier::RightControl),
            57449 => KeyCode::Modifier(Modifier::RightAlt),
            57450 => KeyCode::Modifier(Modifier::RightSuper),
            57451 => KeyCode::Modifier(Modifier::RightHyper),
            57452 => KeyCode::Modifier(Modifier::RightMeta),
            57453 => KeyCode::Modifier(Modifier::IsoLevel3Shift),
            57454 => KeyCode::Modifier(Modifier::IsoLevel5Shift),
            // Anything else in the private-use block is a kitty key this version does not know.
            57344..=63743 => return None,
            _ => KeyCode::Char(char::from_u32(n)?),
        })
    }

    /// `CSI` parameters: semicolon-separated, each with optional colon-separated sub-parameters.
    ///
    /// A borrowing view over the payload rather than a `Vec<Vec<u32>>`: the parse happens on the input
    /// thread, once per keystroke, and a keystroke may not allocate.
    struct Params<'a> {
        body: &'a [u8],
    }

    impl<'a> Params<'a> {
        fn parse(body: &'a [u8]) -> Params<'a> {
            Params { body }
        }

        /// Field `n`'s first sub-parameter, or `None` when the field is absent or empty.
        fn at(&self, n: usize) -> Option<u32> {
            self.sub(n, 0)
        }

        /// Field `n`, sub-parameter `i`, **by position**.
        ///
        /// Positional, and the first draft was not — it filtered the empty sub-parameters out and
        /// then indexed what was left, which is two defects rather than one. `97::5` is *primary 97,
        /// no shifted key, base-layout 5* and became *97 then 5 at index one*; `1:` with an empty
        /// modifier field and an event type became *a modifier of 3 and a press*, so a release would
        /// have arrived as an `Alt` chord. An omitted sub-parameter takes its default and is `None`,
        /// never a zero, because zero is a legitimate value in both fields.
        fn sub(&self, n: usize, i: usize) -> Option<u32> {
            self.field(n).split(|&b| b == b':').nth(i).and_then(number)
        }

        fn field(&self, n: usize) -> &'a [u8] {
            self.body.split(|&b| b == b';').nth(n).unwrap_or(b"")
        }

        /// Field `n`'s sub-parameters, in order, skipping the empty ones.
        ///
        /// For the **associated-text** field only, where the sub-parameters are a sequence of
        /// codepoints and their positions carry nothing.
        fn subs(&self, n: usize) -> impl Iterator<Item = u32> + 'a {
            self.field(n).split(|&b| b == b':').filter_map(number)
        }

        /// The modifier field: `mods` is `1 + bitmask`, and its second sub-parameter is the event
        /// type.
        ///
        /// Absent means "no modifiers, a press", which is what every legacy terminal means by
        /// sending nothing. **A terminal that never sends the second sub-parameter never yields
        /// `Repeat` or `Release`**, and this line is where that is decided rather than a rule
        /// written down somewhere.
        fn modifiers(&self, n: usize) -> (Mods, KeyKind) {
            let mods = self.sub(n, 0).map_or(Mods::NONE, Mods::from_csi);
            let kind = match self.sub(n, 1) {
                Some(2) => KeyKind::Repeat,
                Some(3) => KeyKind::Release,
                _ => KeyKind::Press,
            };
            (mods, kind)
        }
    }

    fn number(field: &[u8]) -> Option<u32> {
        if field.is_empty() {
            return None;
        }
        let mut n: u32 = 0;
        for &b in field {
            let d = b.checked_sub(b'0').filter(|d| *d < 10)?;
            n = n.checked_mul(10)?.checked_add(u32::from(d))?;
        }
        Some(n)
    }
}

/// Something the terminal reported.
///
/// **Owns everything.** [`Screen::next_event`](crate::Screen::next_event) is
/// `&mut self -> Option<Event>`, and a returned value cannot borrow the
/// queue it came from. The only variant that allocates is [`Event::Paste`], which is also the rare
/// one.
/// # Exhaustive, where the wire's own vocabulary is not
///
/// This enum, [`MouseKind`], [`Wheel`], [`KeyKind`] and [`MouseMode`] are **not**
/// `#[non_exhaustive]`: their variant sets are decided rather than discovered — six things a terminal
/// reports, four things a pointer does, four directions a wheel turns, three things a key does and
/// four levels of tracking — and an event loop that could not match them exhaustively would carry a
/// `_ =>` arm that silently swallows whatever is added. [`KeyCode`], [`Keypad`], [`Media`],
/// [`Modifier`] and [`Button`] *are* non-exhaustive, because those are the wire's vocabulary and it
/// grows: kitty's functional block has spare codepoints in it right now.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Event {
    /// A key went down, repeated, or came up.
    Key(Key),
    /// The pointer moved, a button changed, or the wheel turned.
    Mouse(Mouse),
    /// A bracketed paste arrived whole.
    Paste(Paste),
    /// The terminal is now this many columns by this many rows.
    Resize(u16, u16),
    /// The terminal window took focus. Only ever delivered when the application asked for focus
    /// reporting: it is opt-in because it converts an idle application into one woken by every
    /// alt-tab.
    FocusGained,
    /// The terminal window lost focus.
    FocusLost,
}

/// A keystroke, as the terminal described it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Key {
    /// **The base layout — where the key is, not what it printed.**
    ///
    /// A shortcut is about where the key is; [`text`](Key::text) is about what it produced.
    /// Reporting only one collapses `Ctrl+Shift+5` on a non-US layout into something the application
    /// cannot bind. Kitty flag 4 supplies both; where it is absent, `text` is what the terminal sent
    /// and this is inferred from it, so the two agree and neither is invented.
    pub code: KeyCode,
    /// Every modifier the terminal reported, including the two locks.
    pub mods: Mods,
    /// Press, repeat, or release — and a terminal that reports only presses yields only
    /// [`KeyKind::Press`].
    pub kind: KeyKind,
    /// **What the key produced: a grapheme cluster, inline, possibly empty.**
    pub text: KeyText,
    /// When the input thread read it. See [`Mouse::at`].
    pub at: Instant,
}

/// Which key, in the layout the keyboard is actually in.
///
/// Every spelling the wire has, including the ones only the kitty protocol delivers. **Dropping the
/// media and modifier keys is free now and impossible later**, which is the same argument that keeps
/// hyper and meta in [`Mods`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum KeyCode {
    /// A key that produces a character. The scalar is the **base** one: with kitty's disambiguate
    /// flag, `Shift+a` is `Char('a')` with [`Mods::SHIFT`] and a [`Key::text`] of `"A"`.
    Char(char),
    /// Backspace, which is `0x7f` on nearly every terminal and `0x08` on the rest.
    Backspace,
    /// Enter. **Ambiguous with `Ctrl+M` below the kitty protocol, permanently and on purpose.**
    Enter,
    /// Tab. **Ambiguous with `Ctrl+I` below the kitty protocol, permanently and on purpose.**
    Tab,
    /// `Shift+Tab`, which legacy terminals spell `CSI Z` rather than as Tab with a modifier.
    BackTab,
    /// Escape.
    Escape,
    /// Insert.
    Insert,
    /// Delete.
    Delete,
    /// Left arrow.
    Left,
    /// Right arrow.
    Right,
    /// Up arrow.
    Up,
    /// Down arrow.
    Down,
    /// Home.
    Home,
    /// End.
    End,
    /// Page up.
    PageUp,
    /// Page down.
    PageDown,
    /// A function key, numbered from one. Legacy terminals reach F20; kitty reaches F35.
    F(u8),
    /// Caps lock **as a key press**, which is not the same thing as [`Mods::CAPS`].
    CapsLock,
    /// Num lock as a key press.
    NumLock,
    /// Scroll lock as a key press.
    ScrollLock,
    /// Print screen.
    PrintScreen,
    /// Pause.
    Pause,
    /// The menu key.
    Menu,
    /// A keypad key, which the kitty protocol reports distinctly from the main block.
    Keypad(Keypad),
    /// A media key.
    Media(Media),
    /// A modifier pressed on its own, which only kitty's *report all keys* flag delivers.
    Modifier(Modifier),
}

/// A keypad key, told apart from the main block only under the kitty protocol.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum Keypad {
    /// `0` through `9`.
    Digit(u8),
    /// The decimal separator key.
    Decimal,
    /// `/`.
    Divide,
    /// `*`.
    Multiply,
    /// `-`.
    Subtract,
    /// `+`.
    Add,
    /// The keypad's own Enter.
    Enter,
    /// `=`.
    Equal,
    /// The thousands separator key.
    Separator,
    /// Keypad left.
    Left,
    /// Keypad right.
    Right,
    /// Keypad up.
    Up,
    /// Keypad down.
    Down,
    /// Keypad page up.
    PageUp,
    /// Keypad page down.
    PageDown,
    /// Keypad home.
    Home,
    /// Keypad end.
    End,
    /// Keypad insert.
    Insert,
    /// Keypad delete.
    Delete,
    /// The centre key, `5` with num lock off.
    Begin,
}

/// A media key.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum Media {
    /// Play.
    Play,
    /// Pause.
    Pause,
    /// The single key that toggles between the two.
    PlayPause,
    /// Reverse.
    Reverse,
    /// Stop.
    Stop,
    /// Fast forward.
    FastForward,
    /// Rewind.
    Rewind,
    /// Next track.
    TrackNext,
    /// Previous track.
    TrackPrevious,
    /// Record.
    Record,
    /// Volume down.
    LowerVolume,
    /// Volume up.
    RaiseVolume,
    /// Mute.
    MuteVolume,
}

/// A modifier key pressed on its own, with the side it was pressed on.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
#[allow(
    missing_docs,
    reason = "each variant is its own name and a doc line would only repeat it"
)]
pub enum Modifier {
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    LeftHyper,
    LeftMeta,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
    RightHyper,
    RightMeta,
    IsoLevel3Shift,
    IsoLevel5Shift,
}

/// Press, repeat, or release.
///
/// **A terminal that reports only presses produces only [`KeyKind::Press`]**, and nothing here
/// invents the other two. [`Capabilities::key_release`](crate::Capabilities::key_release) and
/// [`Capabilities::key_repeat`](crate::Capabilities::key_repeat) are what an application branches
/// on.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum KeyKind {
    /// The key went down.
    #[default]
    Press,
    /// The keyboard's auto-repeat produced another one. **Indistinguishable from a fast series of
    /// presses without kitty flag 2**, which is why it is a separate variant rather than a folded-in
    /// one.
    Repeat,
    /// The key came up. **Never arrives at all without kitty flag 2.**
    Release,
}

/// Eight modifier bits, including the two locks.
///
/// **All eight, and dropping hyper and meta would be free now and impossible later.** Lock state is
/// keyboard *state* and is never part of a chord: a binding on `Ctrl+C` must still fire with caps
/// lock on, so [`Mods::chord`] is what a key map compares and the raw value is what a component
/// reads when it wants to draw the lock indicators.
///
/// ```
/// use vitui_engine::Mods;
/// let held = Mods::CTRL.with(Mods::CAPS);
/// assert!(held.ctrl() && held.caps());
/// assert_eq!(held.chord(), Mods::CTRL, "a lock is state, not part of the chord");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Mods(u8);

impl Mods {
    /// Nothing held.
    pub const NONE: Mods = Mods(0);
    /// Shift.
    pub const SHIFT: Mods = Mods(1 << 0);
    /// Alt, which a legacy terminal spells as an `ESC` prefix.
    pub const ALT: Mods = Mods(1 << 1);
    /// Control.
    pub const CTRL: Mods = Mods(1 << 2);
    /// Super, which is the command key on macOS and the windows key elsewhere.
    pub const SUPER: Mods = Mods(1 << 3);
    /// Hyper.
    pub const HYPER: Mods = Mods(1 << 4);
    /// Meta.
    pub const META: Mods = Mods(1 << 5);
    /// Caps lock, which is **state and not a chord**.
    pub const CAPS: Mods = Mods(1 << 6);
    /// Num lock, which is **state and not a chord**.
    pub const NUM: Mods = Mods(1 << 7);

    /// The two locks, which [`Mods::chord`] removes.
    const LOCKS: u8 = Mods::CAPS.0 | Mods::NUM.0;

    /// Both sets held at once.
    #[must_use]
    pub const fn with(self, other: Mods) -> Mods {
        Mods(self.0 | other.0)
    }

    /// Whether every bit of `other` is held.
    #[must_use]
    pub const fn contains(self, other: Mods) -> bool {
        self.0 & other.0 == other.0
    }

    /// Nothing at all is held.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The same modifiers with the two locks removed, which is what a key binding compares.
    #[must_use]
    pub const fn chord(self) -> Mods {
        Mods(self.0 & !Mods::LOCKS)
    }

    /// Shift is held.
    #[must_use]
    pub const fn shift(self) -> bool {
        self.contains(Mods::SHIFT)
    }
    /// Alt is held.
    #[must_use]
    pub const fn alt(self) -> bool {
        self.contains(Mods::ALT)
    }
    /// Control is held.
    #[must_use]
    pub const fn ctrl(self) -> bool {
        self.contains(Mods::CTRL)
    }
    /// Super is held.
    #[must_use]
    pub const fn super_key(self) -> bool {
        self.contains(Mods::SUPER)
    }
    /// Hyper is held.
    #[must_use]
    pub const fn hyper(self) -> bool {
        self.contains(Mods::HYPER)
    }
    /// Meta is held.
    #[must_use]
    pub const fn meta(self) -> bool {
        self.contains(Mods::META)
    }
    /// Caps lock is on.
    #[must_use]
    pub const fn caps(self) -> bool {
        self.contains(Mods::CAPS)
    }
    /// Num lock is on.
    #[must_use]
    pub const fn num(self) -> bool {
        self.contains(Mods::NUM)
    }

    /// Whether this combination is one a person is typing text with rather than pressing a chord.
    ///
    /// Shift and the locks pass; everything that turns a key into a shortcut does not.
    pub(crate) const fn is_typing(self) -> bool {
        self.0 & !(Mods::SHIFT.0 | Mods::LOCKS) == 0
    }

    /// The `CSI` modifier field, which every terminal encodes as `1 + bitmask` in kitty's order.
    pub(crate) fn from_csi(field: u32) -> Mods {
        // Clamped rather than cast: `as u8` on a field of 257 answers `Mods::NONE`, which is a
        // terminal saying *eight modifiers* and this crate hearing *none*.
        Mods(field.saturating_sub(1).min(u32::from(u8::MAX)) as u8)
    }

    /// The SGR mouse button word, whose bits 2, 3 and 4 are shift, meta and control.
    ///
    /// Three bits and not eight: the 1006 encoding has no room for the rest, and a mouse event that
    /// claimed to know about hyper would be inventing.
    pub(crate) fn from_sgr(raw: u32) -> Mods {
        let mut mods = Mods::NONE;
        if raw & 0b0000_0100 != 0 {
            mods = mods.with(Mods::SHIFT);
        }
        if raw & 0b0000_1000 != 0 {
            mods = mods.with(Mods::ALT);
        }
        if raw & 0b0001_0000 != 0 {
            mods = mods.with(Mods::CTRL);
        }
        mods
    }
}

impl std::fmt::Debug for Mods {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return f.write_str("Mods::NONE");
        }
        let names = [
            (Mods::SHIFT, "shift"),
            (Mods::ALT, "alt"),
            (Mods::CTRL, "ctrl"),
            (Mods::SUPER, "super"),
            (Mods::HYPER, "hyper"),
            (Mods::META, "meta"),
            (Mods::CAPS, "caps"),
            (Mods::NUM, "num"),
        ];
        let mut first = true;
        f.write_str("Mods(")?;
        for (bit, name) in names {
            if self.contains(bit) {
                if !first {
                    f.write_str("+")?;
                }
                f.write_str(name)?;
                first = false;
            }
        }
        f.write_str(")")
    }
}

/// What a keystroke produced, as an extended grapheme cluster, stored inline.
///
/// **A grapheme cluster and never a `char`**, because kitty flag 16 reports the *codepoints* a key
/// would produce, which is a sequence: a dead-key accent, an Indic conjunct and an IME commit all
/// arrive as more than one scalar for one keystroke — the same reason a cell holds a cluster
/// rather than a `char`.
///
/// **Inline and never a `String`**, because a keystroke must not allocate — asserted through the
/// counting allocator rather than reviewed.
///
/// ```
/// use vitui_engine::KeyText;
/// assert!(KeyText::EMPTY.is_empty());
/// assert_eq!(KeyText::EMPTY.as_str(), "");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct KeyText {
    buf: [u8; KeyText::CAPACITY],
    len: u8,
}

impl KeyText {
    /// How many bytes one keystroke's text may occupy.
    ///
    /// Twenty-four, which holds every cluster a keyboard can produce with room over: a
    /// four-codepoint Devanagari conjunct is twelve bytes, a regional-indicator flag is eight, and a
    /// three-person ZWJ family — which no keystroke produces — is eighteen.
    pub const CAPACITY: usize = 24;

    /// No text at all, which is what a function key, a chord and a release all produce.
    pub const EMPTY: KeyText = KeyText {
        buf: [0; KeyText::CAPACITY],
        len: 0,
    };

    /// The length that means *this did not fit and nothing is being reported*.
    ///
    /// A sentinel rather than a second field, and it has to be sticky: a cluster that overflowed
    /// must not quietly accept the scalars after the one that broke it, which would report the
    /// **tail** of a dead-key sequence as though it were the whole of it.
    const OVERFLOWED: u8 = u8::MAX;

    /// The cluster, as a string. Empty when the key produced nothing and when what it produced did
    /// not fit.
    #[must_use]
    pub fn as_str(&self) -> &str {
        if self.len == KeyText::OVERFLOWED {
            return "";
        }
        // Every write goes through `push_char`, which encodes whole scalars and empties the buffer
        // rather than cutting one in half, so the prefix is always valid UTF-8.
        std::str::from_utf8(&self.buf[..usize::from(self.len)]).unwrap_or("")
    }

    /// Whether the key produced nothing this can report.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0 || self.len == KeyText::OVERFLOWED
    }

    /// **What one character produced, for a caller that has to build the key the terminal sends.**
    ///
    /// A scalar is at most four bytes and [`CAPACITY`](KeyText::CAPACITY) is twenty-four, so this is
    /// total by arithmetic: it has no failure mode and returns no `Option`. The cluster
    /// constructor it is deliberately not is the point — see below.
    ///
    /// # Why this is public when the rest of the type is not
    ///
    /// The stated reason for the private fields was that *nothing outside should be able to forge a
    /// key whose `code` and `text` disagree*, and that reason was never true of this engine: a
    /// terminal at kitty flag 4 reports the **base layout** as `code` while flag 16 reports what the
    /// key **produced** as `text`, so `CSI 61;2;43u` is `code == Char('=')` with `text == "+"` and
    /// the two disagreeing is the design rather than a forgery. What the private fields actually
    /// protect is the overflow sentinel — a `len` a caller could set to a length the buffer does not
    /// hold — and a `char` cannot reach it.
    ///
    /// It is public because the layer above could not otherwise **test** the half of a binding that
    /// reads `text`: `vitui_runtime::keys::On::Typed` had no positive test in its own crate for
    /// want of any way to build the key that would match one.
    ///
    /// **No `&str` constructor is offered**, and not for want of a use: a multi-scalar cluster is a
    /// dead key, an Indic conjunct or an IME commit, none of which any binding compares against, so
    /// the only thing it could build is text nothing can match — and it would carry the overflow
    /// case this one does not have.
    ///
    /// ```
    /// use vitui_engine::KeyText;
    /// assert_eq!(KeyText::of('+').as_str(), "+");
    /// assert!(!KeyText::of('+').is_empty());
    /// ```
    #[must_use]
    pub fn of(c: char) -> KeyText {
        let mut text = KeyText::EMPTY;
        text.push_char(c);
        text
    }

    /// Build one from a string, keeping nothing at all when it does not fit.
    pub(crate) fn from_str(s: &str) -> KeyText {
        let mut text = KeyText::EMPTY;
        for c in s.chars() {
            text.push_char(c);
        }
        text
    }

    /// Append one scalar, or **empty the whole thing** when it will not fit.
    ///
    /// Clamp-and-discard, as everywhere else in this crate, with the discard taken whole: half a
    /// grapheme cluster is not a shorter cluster, it is a different one, and a text field that
    /// received the first half of a dead-key sequence is worse than one that received nothing. The
    /// keystroke itself is never lost — [`Key::code`] is unaffected.
    pub(crate) fn push_char(&mut self, c: char) {
        if self.len == KeyText::OVERFLOWED {
            return;
        }
        let at = usize::from(self.len);
        let needed = c.len_utf8();
        if at + needed > KeyText::CAPACITY {
            *self = KeyText::EMPTY;
            self.len = KeyText::OVERFLOWED;
            return;
        }
        c.encode_utf8(&mut self.buf[at..]);
        self.len += needed as u8;
    }
}

impl std::fmt::Debug for KeyText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_str(), f)
    }
}

/// A pointer event, in screen cells, with nothing resolved.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Mouse {
    /// The column, zero-based. The wire is one-based and this is not; every rectangle in this crate
    /// counts from zero and a boundary that changes convention half way is a defect generator.
    pub x: u16,
    /// The row, zero-based.
    pub y: u16,
    /// What happened.
    pub kind: MouseKind,
    /// What is held down, **which is what makes a [`MouseKind::Move`] a drag**.
    pub buttons: Buttons,
    /// Shift, alt and control. The 1006 encoding carries three bits and no more.
    pub mods: Mods,
    /// When the input thread read it.
    ///
    /// **Stamped at read time**, because the app thread cannot recover it: between the read and the
    /// handler sit the frame clock's hold — 7.3 ms at 120 Hz — and the application's own slowness.
    /// **Double-click detection is not the engine's**: it is policy with a tunable threshold and it
    /// belongs where hit-testing belongs.
    pub at: Instant,
}

/// What the pointer did. Exhaustive — see [`Event`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum MouseKind {
    /// A button went down.
    Down(Button),
    /// A button came up.
    Up(Button),
    /// The pointer moved. **Consecutive moves coalesce** — see the module documentation.
    Move,
    /// The wheel turned one notch.
    Wheel(Wheel),
}

/// A wheel notch, which is a direction.
///
/// This was sketched as `Delta`, and the name did not survive contact: the 1006 encoding reports
/// one notch per event and carries no magnitude at all, so a type called `Delta` would promise a
/// number no terminal sends. An application that wants acceleration counts the notches, with the
/// timestamps it already has.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Wheel {
    /// Scrolled up.
    Up,
    /// Scrolled down.
    Down,
    /// Scrolled left, which a tilt wheel or a trackpad produces.
    Left,
    /// Scrolled right.
    Right,
}

/// One pointer button.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[non_exhaustive]
pub enum Button {
    /// The primary button.
    Left,
    /// The wheel, pressed.
    Middle,
    /// The secondary button.
    Right,
    /// The thumb button that usually means *back*.
    Back,
    /// The thumb button that usually means *forward*.
    Forward,
    /// A button the SGR encoding numbered and this crate has no name for.
    Other(u8),
}

/// Which buttons are held.
///
/// ```
/// use vitui_engine::Buttons;
/// assert!(Buttons::NONE.is_empty());
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Buttons(u8);

impl Buttons {
    /// Nothing held.
    pub const NONE: Buttons = Buttons(0);

    /// The primary button is down.
    #[must_use]
    pub const fn left(self) -> bool {
        self.0 & 1 != 0
    }
    /// The wheel button is down.
    #[must_use]
    pub const fn middle(self) -> bool {
        self.0 & 2 != 0
    }
    /// The secondary button is down.
    #[must_use]
    pub const fn right(self) -> bool {
        self.0 & 4 != 0
    }
    /// The back button is down.
    #[must_use]
    pub const fn back(self) -> bool {
        self.0 & 8 != 0
    }
    /// The forward button is down.
    #[must_use]
    pub const fn forward(self) -> bool {
        self.0 & 16 != 0
    }
    /// Nothing at all is held, which is what makes a [`MouseKind::Move`] a hover rather than a drag.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    fn bit(button: Button) -> u8 {
        match button {
            Button::Left => 1,
            Button::Middle => 2,
            Button::Right => 4,
            Button::Back => 8,
            Button::Forward => 16,
            // A button with no name gets no bit. Reporting it as one of the five would be worse
            // than reporting nothing, and `MouseKind::Down` still names it.
            Button::Other(_) => 0,
        }
    }

    pub(crate) fn press(&mut self, button: Button) {
        self.0 |= Buttons::bit(button);
    }

    pub(crate) fn release(&mut self, button: Button) {
        self.0 &= !Buttons::bit(button);
    }
}

impl std::fmt::Debug for Buttons {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return f.write_str("Buttons::NONE");
        }
        let names = [
            (Buttons::left as fn(Buttons) -> bool, "left"),
            (Buttons::middle, "middle"),
            (Buttons::right, "right"),
            (Buttons::back, "back"),
            (Buttons::forward, "forward"),
        ];
        f.write_str("Buttons(")?;
        let mut first = true;
        for (held, name) in names {
            if held(*self) {
                if !first {
                    f.write_str("+")?;
                }
                f.write_str(name)?;
                first = false;
            }
        }
        f.write_str(")")
    }
}

/// A bracketed paste, whole.
///
/// **Owns bytes, not a `String`.** Pasted bytes are not guaranteed to be UTF-8 under any answer: a
/// file in latin-1, a binary, a multi-byte sequence cut by a read boundary. A `String` would force
/// this crate to resolve that silently, and silently means either a panic or corruption.
///
/// The asymmetry with the drawing verbs is deliberate and belongs in both doc comments: **at the
/// drawing verbs the engine may demand well-formed input from its caller; at the input boundary it
/// may demand nothing** — a paste is composed by the world, not by this API.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Paste {
    bytes: Vec<u8>,
    truncated: bool,
    at: Instant,
}

impl Paste {
    pub(crate) fn new(bytes: Vec<u8>, truncated: bool, at: Instant) -> Paste {
        Paste {
            bytes,
            truncated,
            at,
        }
    }

    /// When the input thread read the paste's closing marker.
    #[must_use]
    pub fn at(&self) -> Instant {
        self.at
    }

    /// Exactly what arrived, byte for byte.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The paste as text, **lossily**, and it says so in the type: a [`Cow::Owned`] is what came back
    /// with replacement characters in it.
    #[must_use]
    pub fn text(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.bytes)
    }

    /// Whether the paste hit [`InputConfig::paste_limit`] and lost its tail.
    ///
    /// **On the event rather than in a diagnostic**, because silent truncation is data loss the user
    /// will attribute to the application.
    #[must_use]
    pub fn truncated(&self) -> bool {
        self.truncated
    }
}

impl Event {
    /// When the input thread read it, for the three variants that have somewhere to keep it.
    ///
    /// Every event **is** stamped at read time — the input thread takes one [`Instant`] per read and
    /// hands it to the parser — and three of the six have a field to keep it in. [`Event::Resize`],
    /// [`Event::FocusGained`] and [`Event::FocusLost`] carry a size and nothing, and widening them
    /// to hold a timestamp nobody has asked for would be paying for a field on every event.
    ///
    /// The stamp exists because **the app thread cannot recover it**: between the read and the
    /// handler sit the frame clock's hold — 7.3 ms at 120 Hz — and the application's own slowness.
    #[must_use]
    pub fn at(&self) -> Option<Instant> {
        match self {
            Event::Key(key) => Some(key.at),
            Event::Mouse(mouse) => Some(mouse.at),
            Event::Paste(paste) => Some(paste.at),
            Event::Resize(..) | Event::FocusGained | Event::FocusLost => None,
        }
    }
}

/// How much mouse reporting the terminal is asked for.
///
/// **Totally ordered, deliberately**: `Off < Buttons < Drag < Motion` — modes 1000, 1002 and 1003 —
/// and each strictly contains the one below, so **combining what several components want is a `max`
/// and not a set union**. The actuator that applies it is
/// [`Screen::set_mouse`](crate::Screen::set_mouse), and the `max` is the runtime's to take: there is
/// no mount, a component is a function, and its declaration rides the draw.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub enum MouseMode {
    /// No mouse reporting at all.
    #[default]
    Off,
    /// Presses and releases.
    Buttons,
    /// Presses, releases, and motion while a button is held.
    Drag,
    /// All of the above and every motion, held or not.
    Motion,
}

/// What the application wants the terminal switched on for.
///
/// **A floor, not a setting.** The union of what the components want is known only after a draw, so
/// the frame that first paints a hover-wanting modal did not yet have tracking on. An application
/// that knows it wants the mouse says so once and has no blind frame; one that does not, pays
/// nothing.
///
/// Focus reporting and paste are off by default because each converts an idle application into a
/// woken one — focus reporting on every alt-tab, motion tracking on every pointer move — and the
/// standing requirement is that an idle application costs nothing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct InputConfig {
    /// The floor on mouse reporting.
    pub mouse: MouseMode,
    /// Whether to ask for focus events (mode 1004). **Opt-in and off by default.**
    pub focus: bool,
    /// Whether to ask for bracketed paste (mode 2004).
    pub paste: bool,
    /// The ceiling on one paste, in bytes. Exceeding it sets [`Paste::truncated`].
    pub paste_limit: usize,
}

impl InputConfig {
    /// One mebibyte, which is far more than a person pastes into a form and far less than a file
    /// somebody drops on the window by accident.
    const DEFAULT_PASTE_LIMIT: usize = 1 << 20;
}

impl Default for InputConfig {
    fn default() -> InputConfig {
        InputConfig {
            mouse: MouseMode::Off,
            focus: false,
            paste: false,
            paste_limit: InputConfig::DEFAULT_PASTE_LIMIT,
        }
    }
}

/// What the parser could not make sense of.
///
/// **Silent discard is the defect class that costs a day** — "Shift+F5 does nothing", with no thread
/// to pull. Two fields buy the thread.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct InputDiagnostics {
    unrecognised: u64,
    last: Vec<u8>,
}

impl InputDiagnostics {
    /// How many sequences were dropped because nothing recognised them.
    #[must_use]
    pub fn unrecognised(&self) -> u64 {
        self.unrecognised
    }

    /// The last of them, exactly as it arrived. `None` when there has not been one.
    #[must_use]
    pub fn last_unrecognised(&self) -> Option<&[u8]> {
        (self.unrecognised > 0).then_some(&self.last[..])
    }
}

/// The queue between the input thread and the app thread.
///
/// One mutex, held for a push and for a pop and for nothing else. It is not the wake source's mutex
/// on purpose: the app thread parks in that one, and a queue behind it would make every keystroke
/// contend with every park.
#[derive(Debug)]
pub(crate) struct Queue {
    inner: Mutex<Inner>,
}

#[derive(Debug)]
struct Inner {
    events: VecDeque<Event>,
    diagnostics: InputDiagnostics,
}

impl Queue {
    /// How many events the queue is built to hold before it grows.
    ///
    /// A keystroke may not allocate, and a `VecDeque` allocates when it grows — so the capacity is
    /// taken once, at `attach`, where allocation is permitted. It is a floor and not a cap: **the
    /// queue grows and nothing bounds it**, because a terminal without bracketed paste delivers a
    /// pasted megabyte as key events, and dropping the oldest would throw away intent for a case
    /// that resolves in seconds.
    const CAPACITY: usize = 256;

    pub(crate) fn new() -> Queue {
        Queue {
            inner: Mutex::new(Inner {
                events: VecDeque::with_capacity(Queue::CAPACITY),
                diagnostics: InputDiagnostics::default(),
            }),
        }
    }

    /// Add one event, coalescing it into the last if both are pointer motion.
    ///
    /// **This is the whole of *intent is never dropped, position is*.** A press, a release, a wheel
    /// turn, a keystroke and a resize
    /// all express something the user meant and fall through to the push; an intermediate pointer
    /// position says only where the pointer was on the way, and the newer one supersedes it whole —
    /// position, buttons, modifiers and timestamp.
    pub(crate) fn push(&self, event: Event) {
        let mut inner = self.lock();
        let moving = matches!(
            event,
            Event::Mouse(Mouse {
                kind: MouseKind::Move,
                ..
            })
        );
        if moving
            && let Some(
                back @ Event::Mouse(Mouse {
                    kind: MouseKind::Move,
                    ..
                }),
            ) = inner.events.back_mut()
        {
            *back = event;
            return;
        }
        inner.events.push_back(event);
    }

    /// Take the oldest event, or `None`.
    pub(crate) fn pop(&self) -> Option<Event> {
        self.lock().events.pop_front()
    }

    /// Throw away what the user did, and keep what the terminal became.
    ///
    /// **The one caller is [`Screen::resume`](crate::Screen::resume)**, and it is there because of a
    /// consequence of this engine's shape that no verb can undo: the reader is a thread parked in a
    /// blocking `read` on standard input, and nothing in safe Rust cancels one. So while a
    /// `Screen` is suspended the reader is *still there*, and whatever reaches the terminal reaches
    /// it. A process that suspended itself is stopped and so is its reader; one that handed the
    /// terminal to a child and kept running is competing with that child for every byte.
    ///
    /// What this decides is the half that is decidable: **those bytes are not this application's
    /// input.** Delivered on the far side of a resume they are the editor's session replayed as
    /// keystrokes, hundreds of them, acted on by a screen that has just repainted.
    ///
    /// **A resize is kept, and that is not a nicety.** `Screen::next_event` is what applies one to
    /// the surfaces; the authoritative size is already stored by the input thread, and `present`
    /// refuses to composite while the two disagree. Dropping the event would leave that disagreement
    /// standing with nothing left to resolve it — a frame owed for ever, which is a hang. Focus is
    /// kept for the weaker version of the same reason: it is a level the terminal is in rather than
    /// something somebody typed.
    pub(crate) fn drop_what_the_user_typed(&self) {
        let mut inner = self.lock();
        inner.events.retain(|event| {
            matches!(
                event,
                Event::Resize(..) | Event::FocusGained | Event::FocusLost
            )
        });
    }

    /// What the parser could not make sense of, as of the last read.
    pub(crate) fn diagnostics(&self) -> InputDiagnostics {
        self.lock().diagnostics.clone()
    }

    /// Publish the parser's counters. Called once per read, and only when they moved.
    pub(crate) fn set_diagnostics(&self, unrecognised: u64, last: &[u8]) {
        let mut inner = self.lock();
        inner.diagnostics.unrecognised = unrecognised;
        inner.diagnostics.last.clear();
        inner.diagnostics.last.extend_from_slice(last);
    }

    /// How many events are waiting. The gate over the queue is a count, so this exists.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.lock().events.len()
    }

    /// **Poisoning is recovered from rather than propagated**, and the reason is what is behind the
    /// lock: a queue of plain values and two counters, none of which has an invariant a half-finished
    /// write could break. Nothing that runs while this lock is held can panic, so a poisoned lock
    /// means some *other* thread panicked — and the app thread losing its keyboard on the way to a
    /// panic handler helps nobody. There is no `unwrap` here for the same reason there is none
    /// anywhere else in this crate.
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
