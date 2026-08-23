//! Asking the terminal on the other end, in one batch, behind one sentinel.
//!
//! # Never terminfo
//!
//! terminfo describes what `$TERM` **claims**, not what the terminal implements, and it cannot
//! express mode 2026 or the kitty-keyboard flag stack at all. libvaxis is the production proof that
//! querying the live pty works; tcell is the counterexample and shows what the other road costs —
//! suffix matching, name patterns, a "best guess" 256-colour default and a hardcoded
//! known-terminal profile table. Spec §10, and it is a decision rather than a preference.
//!
//! # The timeout problem is solved by the sentinel, not by tuning a number
//!
//! Every query goes out in **one** write with a trailing **DA1**, and the loop reacts to DA1's
//! *arrival* rather than to a clock. Virtually everything answers DA1, including tmux, so it is a
//! sentinel rather than an identity probe: when it comes back, every answer that was going to come
//! back already has, because a terminal answers in order.
//!
//! The numeric ceiling exists for exactly one case — **there is no terminal at all** — and the
//! difference is visible in the loop below: the ceiling is an *idle* deadline that resets on every
//! byte, so a terminal that dribbles its answers over ten times the ceiling is never cut off, and a
//! pipe that says nothing costs one ceiling and returns [`AttachError::NoAnswer`]. Tuning the
//! numeric timeout alone is fragile, and a real bug — `terminal-light` against iTerm2 — is why that
//! is stated rather than assumed.
//!
//! **The seventeen queries the degradation model added cost no extra round trip**, because they sit
//! in the same batch ahead of the sentinel. That is the whole argument for asking sixteen palette
//! entries at all.

use std::io::{ErrorKind, Read, Write};
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use crate::caps::{
    Detected, KITTY_ALL, MODE_BRACKETED_PASTE, MODE_DECSLRM, MODE_FOCUS, MODE_GRAPHEME_CLUSTERS,
    MODE_MOUSE, MODE_MOUSE_MOTION, MODE_SYNC_OUTPUT, Rgb,
};
use crate::engine::AttachError;

/// How long to wait for the *first* byte, and for each byte after a silence.
///
/// Spec §10 names 100–300 ms and does not pick one, because the number decides nothing that
/// matters: it is the cost of discovering that a pipe is a pipe.
pub(crate) const CEILING: Duration = Duration::from_millis(250);

const ESC: u8 = 0x1b;
const BEL: u8 = 0x07;

/// Every query, in one write, with the sentinel last.
///
/// Order is not decorative. The two `CSI ? u` reads sit either side of the push so that the second
/// says **what stuck** rather than what was asked for — the difference between four honest booleans
/// and a `KeyboardTier` that lies about tmux — and the pop puts the terminal back before the
/// application ever sees it. Mode 2027 is *requested* before it is asked about, because the question
/// is whether the request took.
pub(crate) fn batch() -> Vec<u8> {
    let mut out = String::with_capacity(512);

    // Identity: XTVERSION where it exists, DA2 as the fallback.
    out.push_str("\x1b[>0q");
    out.push_str("\x1b[>c");

    // Truecolor, from the terminal itself rather than from `$COLORTERM`. XTGETTCAP is used for this
    // one capability and no other — a wider use of it would be terminfo through a different door.
    out.push_str("\x1bP+q524742\x1b\\");

    // The terminal's real default foreground and background (§5).
    out.push_str("\x1b]10;?\x1b\\");
    out.push_str("\x1b]11;?\x1b\\");

    // Palette 0..16, needed for quantisation at `Ansi16`, where there is no escape into a fixed
    // cube. Sixteen queries and no extra round trip.
    for i in 0..16 {
        out.push_str(&format!("\x1b]4;{i};?\x1b\\"));
    }

    // Mode 2027, requested once, then asked about. Our tables stay authoritative either way.
    out.push_str("\x1b[?2027h");
    for mode in [
        MODE_GRAPHEME_CLUSTERS,
        MODE_SYNC_OUTPUT,
        MODE_MOUSE,
        MODE_MOUSE_MOTION,
        MODE_FOCUS,
        MODE_BRACKETED_PASTE,
        MODE_DECSLRM,
    ] {
        out.push_str(&format!("\x1b[?{mode}$p"));
    }

    // Push every kitty flag and read back what survived, then put the stack back.
    out.push_str("\x1b[?u");
    out.push_str(&format!("\x1b[>{KITTY_ALL}u"));
    out.push_str("\x1b[?u");
    out.push_str("\x1b[<u");

    // The sentinel. Last, always.
    out.push_str("\x1b[c");
    out.into_bytes()
}

/// One end of a pty, as much of it as detection needs.
///
/// Internal, and it is not a seam the way [`Output`](crate::Output) is: nothing outside this crate
/// can supply one, because a caller who wants to decide the answers has [`Overrides`](crate::Overrides)
/// and does not need a fake terminal to say them through.
pub(crate) trait Probe {
    /// Send the whole batch. One call, because a batch split across writes is no longer a batch.
    fn write_batch(&mut self, bytes: &[u8]) -> std::io::Result<()>;

    /// Read whatever has arrived, waiting at most `timeout`. `Ok(0)` means the deadline fired.
    fn read(&mut self, buf: &mut [u8], timeout: Duration) -> std::io::Result<usize>;

    /// Give back what was read past the sentinel.
    ///
    /// The last read of the exchange takes whatever the kernel had, and DA1 is rarely the last byte
    /// in it — a key pressed while the application was starting is sitting right behind the
    /// sentinel. Those bytes belong to the application, and dropping them would make the retained
    /// pty a half-promise: one reader, but a hole in what it read.
    fn unread(&mut self, bytes: &[u8]);
}

/// Fire the batch and read until the sentinel comes back.
///
/// # Errors
///
/// [`AttachError::NoAnswer`] when **nothing at all** arrived inside `ceiling`. A terminal that
/// answered some of the batch and then went quiet is not an error: what arrived is kept, and every
/// question it did not answer stays at its conservative default, which is the same place it would
/// have been had the question never been asked.
pub(crate) fn detect(probe: &mut dyn Probe, ceiling: Duration) -> Result<Detected, AttachError> {
    let mut out = Detected::default();
    if probe.write_batch(&batch()).is_err() {
        return Err(AttachError::NoAnswer);
    }

    let mut parser = Parser::default();
    let mut buf = [0u8; 4096];
    let mut heard = false;
    // The tail of the read the sentinel arrived in. Empty on every other exit, because a timeout and
    // a broken pty both leave nothing after the last byte read.
    let mut tail: Vec<u8> = Vec::new();
    loop {
        match probe.read(&mut buf, ceiling) {
            Ok(0) | Err(_) if !heard => return Err(AttachError::NoAnswer),
            // The idle deadline fired after the terminal had started answering, or the pty broke
            // mid-answer. Keep what arrived; the sentinel is a fast path, not the only exit.
            Ok(0) | Err(_) => break,
            Ok(n) => {
                heard = true;
                if let Some(used) = parser.feed(&buf[..n], &mut out) {
                    tail.extend_from_slice(&buf[used..n]);
                    break;
                }
            }
        }
    }

    // **One hand-back, on every exit.** The first draft did it twice on the sentinel path — once
    // inside the loop and once after it — which pushed the type-ahead in front of itself and lost
    // the tail behind the duplicate. Everything the parser set aside, then the tail, in that order:
    // the tail is later in the stream, and a hand-back out of order reorders somebody's keystrokes.
    let mut back = parser.type_ahead().to_vec();
    back.append(&mut tail);
    probe.unread(&back);
    Ok(out)
}

/// Where in an escape sequence the byte stream is.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum State {
    #[default]
    Ground,
    Esc,
    Csi,
    Osc,
    Dcs,
    /// An `ESC` inside a string sequence: `ESC \` ends it, anything else is part of it.
    StringEsc,
}

/// The answers, as they arrive, one byte at a time.
///
/// It has to be incremental because a pty delivers escape sequences split wherever the kernel felt
/// like splitting them, and a parser that assumed whole sequences would work on every terminal that
/// is fast and fail on every terminal that is slow — which is the failure mode that survives
/// testing and reaches a user.
#[derive(Default)]
struct Parser {
    state: State,
    buf: Vec<u8>,
    string_is_dcs: bool,
    /// Bytes that were never an answer to anything: **the user's own input**, arriving interleaved
    /// with the replies because a pty has one input stream and the person at the keyboard does not
    /// wait to be asked.
    ///
    /// The reader thread is spawned before the batch goes out, so a key pressed at the shell prompt
    /// is already in the channel when detection starts. Parsing it as an answer and dropping it is
    /// the same hole `unread` closes on the other side of the sentinel, and it is the more likely
    /// one: type-ahead at startup is ordinary, and bytes after DA1 need the user to be quick.
    spill: Vec<u8>,
}

impl Parser {
    /// Feed bytes. Answers with how many were consumed once the DA1 sentinel has been seen, and
    /// `None` while it has not — everything past that index is the application's.
    fn feed(&mut self, bytes: &[u8], out: &mut Detected) -> Option<usize> {
        for (at, &b) in bytes.iter().enumerate() {
            if self.step(b, out) {
                return Some(at + 1);
            }
        }
        None
    }

    /// Everything read that was the user's rather than the terminal's, in arrival order.
    fn type_ahead(&self) -> &[u8] {
        &self.spill
    }

    fn step(&mut self, b: u8, out: &mut Detected) -> bool {
        match self.state {
            State::Ground => {
                if b == ESC {
                    self.state = State::Esc;
                } else {
                    // Nothing in the batch is answered with a bare byte, so this is the user.
                    self.spill.push(b);
                }
            }
            State::Esc => {
                self.buf.clear();
                match b {
                    b'[' => self.state = State::Csi,
                    b']' => {
                        self.string_is_dcs = false;
                        self.state = State::Osc;
                    }
                    b'P' => {
                        self.string_is_dcs = true;
                        self.state = State::Dcs;
                    }
                    ESC => {}
                    _ => {
                        // `ESC O P` is F1 on every terminal there is. Not an answer to anything in
                        // the batch, so both bytes go back to whoever typed them.
                        self.spill.push(ESC);
                        self.spill.push(b);
                        self.state = State::Ground;
                    }
                }
            }
            State::Csi => {
                // Anything in 0x40..=0x7E ends a control sequence; everything before it is
                // parameters and intermediates, and both are kept because DECRQM's answer is told
                // apart from a key press by its `$`.
                if (0x40..=0x7e).contains(&b) {
                    self.state = State::Ground;
                    let payload = std::mem::take(&mut self.buf);
                    match csi(&payload, b, out) {
                        Reply::Sentinel => return true,
                        Reply::Ours => {}
                        // An arrow key is `ESC [ A`, and it is indistinguishable from a reply by
                        // shape alone — only by not being one of the four the batch asked for.
                        Reply::Theirs => {
                            self.spill.push(ESC);
                            self.spill.push(b'[');
                            self.spill.extend_from_slice(&payload);
                            self.spill.push(b);
                        }
                    }
                    return false;
                }
                self.buf.push(b);
            }
            State::Osc | State::Dcs => match b {
                BEL => {
                    self.state = State::Ground;
                    let payload = std::mem::take(&mut self.buf);
                    string(&payload, self.string_is_dcs, out);
                }
                ESC => self.state = State::StringEsc,
                _ => self.buf.push(b),
            },
            State::StringEsc => {
                if b == b'\\' {
                    self.state = State::Ground;
                    let payload = std::mem::take(&mut self.buf);
                    string(&payload, self.string_is_dcs, out);
                } else {
                    // **`ESC` aborts a string; it is not content.** Absorbing it was the bug: one
                    // arrow key pressed while an OSC 11 reply was in flight would push `ESC [` into
                    // the payload, and every byte after it — the DA1 sentinel included — would be
                    // eaten as OSC content until the next terminator. Detection would then wait out
                    // the whole idle ceiling and lose the rest of the batch.
                    //
                    // The truncated payload is **discarded** rather than dispatched: half an OSC 11
                    // colour that happens to parse is a wrong default background, which §5 says is
                    // wrong in *direction*.
                    self.buf.clear();
                    self.state = State::Ground;
                    return self.step(b, out);
                }
            }
        }
        false
    }
}

/// Whose a finished sequence turned out to be.
enum Reply {
    /// DA1: the sentinel, and the end of the exchange.
    Sentinel,
    /// An answer to something the batch asked.
    Ours,
    /// Not an answer to anything. The user typed it, and it is theirs to keep.
    Theirs,
}

/// A finished control sequence, and who it belongs to.
fn csi(payload: &[u8], final_byte: u8, out: &mut Detected) -> Reply {
    let text = String::from_utf8_lossy(payload).into_owned();
    match final_byte {
        b'c' if text.starts_with('>') => {
            let mut it = text[1..].split(';').map(|p| p.parse::<u32>().unwrap_or(0));
            out.da2 = Some((
                it.next().unwrap_or(0),
                it.next().unwrap_or(0),
                it.next().unwrap_or(0),
            ));
        }
        // DA1. Not an identity probe — a sentinel, and the only thing that proves there is a
        // terminal at all.
        b'c' => {
            out.answered = true;
            return Reply::Sentinel;
        }
        // The kitty keyboard flags. The last answer wins, and the batch asks twice on purpose.
        b'u' if text.starts_with('?') => {
            out.kitty_flags = Some(text[1..].trim().parse::<u32>().unwrap_or(0));
        }
        // DECRQM: `CSI ? Ps ; Pm $ y`. A `Pm` of zero is *not recognised*, which is the whole
        // answer for a mode nobody implements.
        b'y' if text.starts_with('?') => {
            let body = text[1..].trim_end_matches('$');
            let mut it = body
                .split(';')
                .map(|p| p.trim().parse::<u32>().unwrap_or(0));
            if let (Some(mode), Some(state)) = (it.next(), it.next()) {
                out.modes.push((mode, state));
            } else {
                return Reply::Theirs;
            }
        }
        _ => return Reply::Theirs,
    }
    Reply::Ours
}

/// A finished OSC or DCS string.
fn string(payload: &[u8], is_dcs: bool, out: &mut Detected) {
    let text = String::from_utf8_lossy(payload).into_owned();
    if is_dcs {
        // XTVERSION answers `DCS > | name ST`.
        if let Some(name) = text.strip_prefix(">|") {
            out.version = Some(name.trim().to_string());
            return;
        }
        // XTGETTCAP answers `DCS 1 + r <hex>=<hex> ST` on success and `DCS 0 + r <hex> ST` on
        // failure. The two are kept apart from silence, which is a third thing: a terminal that
        // does not implement XTGETTCAP has not denied anything.
        if let Some(rest) = text.strip_prefix("1+r") {
            if rest
                .split('=')
                .next()
                .is_some_and(|k| k.eq_ignore_ascii_case("524742"))
            {
                out.rgb = Some(true);
            }
        } else if text.starts_with("0+r") {
            out.rgb = Some(false);
        }
        return;
    }

    let mut fields = text.split(';');
    match fields.next() {
        Some("10") => out.default_fg = fields.next().and_then(colour),
        Some("11") => out.default_bg = fields.next().and_then(colour),
        Some("4") => {
            let index = fields.next().and_then(|i| i.trim().parse::<usize>().ok());
            let value = fields.next().and_then(colour);
            if let (Some(i), Some(v)) = (index, value)
                && i < out.palette.len()
            {
                out.palette[i] = Some(v);
            }
        }
        _ => {}
    }
}

/// `rgb:rrrr/gggg/bbbb`, `rgb:rr/gg/bb`, or `#rrggbb`.
///
/// The X11 form carries up to four hex digits per channel and the high byte is what a cell can
/// hold, so it is taken rather than scaled — `ffff` is `ff` and `0000` is `00`, which is exact at
/// both ends and off by less than a level anywhere else.
fn colour(spec: &str) -> Option<Rgb> {
    let spec = spec.trim().trim_end_matches(['\x07', '\x1b', '\\']);
    if let Some(hex) = spec.strip_prefix('#') {
        if hex.len() == 6 {
            return Some(Rgb::new(
                u8::from_str_radix(&hex[0..2], 16).ok()?,
                u8::from_str_radix(&hex[2..4], 16).ok()?,
                u8::from_str_radix(&hex[4..6], 16).ok()?,
            ));
        }
        return None;
    }
    let body = spec
        .strip_prefix("rgb:")
        .or_else(|| spec.strip_prefix("rgba:"))?;
    let mut parts = body.split('/');
    let r = channel_value(parts.next()?)?;
    let g = channel_value(parts.next()?)?;
    let b = channel_value(parts.next()?)?;
    Some(Rgb::new(r, g, b))
}

fn channel_value(hex: &str) -> Option<u8> {
    let hex = hex.trim();
    if hex.is_empty() || hex.len() > 4 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let value = u32::from_str_radix(hex, 16).ok()?;
    // Widen to the top of the field, then take the high byte: `f` is `ff`, not `0f`.
    Some(match hex.len() {
        1 => (value * 0x11) as u8,
        2 => value as u8,
        3 => ((value << 4 | value >> 8) >> 8) as u8,
        _ => (value >> 8) as u8,
    })
}

/// **A unit test may not reach for the developer's terminal, and this makes that structural rather
/// than remembered.**
///
/// The defect it closes was live: `Config::default()` is `Output::Terminal`, three tests used it, and
/// `cargo test` in a real terminal enabled raw mode, wrote the query batch onto the screen, ate the
/// developer's keystrokes and failed. **CI has no tty, so it was green there and broken only for
/// humans** — a test that reaches for real I/O passes in the environment that has none.
///
/// A guard beats a second test environment because it cannot rot and costs nothing. What it cannot
/// cover is a **doctest**: those compile against the crate as a dependency, without `cfg(test)`, so
/// it is invisible to them. `.gitlab-ci.yml` runs the suite a second time under a pty for that half.
///
/// # Why a function pair and not a `#[cfg(test)] panic!` in the body
///
/// The first form put the `panic!` inside [`Tty::open`], which made the whole rest of that body
/// **unreachable** under `cfg(test)` — so `IsTty` and `Read` became unused imports in the `lib test`
/// target, and CI runs under `RUSTFLAGS: -D warnings` where an unused import is a build failure. A
/// call to a `()`-returning function keeps the body reachable and both imports used, in both
/// configurations, while still failing the test at runtime.
#[cfg(test)]
fn refuse_in_tests() {
    panic!(
        "a test called Tty::open, which would query the developer's real terminal — use \
         Output::Sink (see engine::tests::headless) instead of Output::Terminal"
    );
}

#[cfg(not(test))]
fn refuse_in_tests() {}

/// The process's own terminal.
///
/// # One reader, ever
///
/// Detection has to read raw bytes with a deadline, and there is exactly one way to do that in this
/// crate's dependency policy: a thread that blocks on `stdin` and a channel with `recv_timeout`.
/// The thread is therefore **kept** rather than detached — it is held on the `Screen` — because a
/// second reader of the same file descriptor steals bytes from the first, and ticket 20's input
/// thread is the one that will adopt this channel rather than opening its own.
///
/// Raw mode is entered here because a line-buffered terminal answers nothing until a newline that
/// is never coming, and giving it back is this type's `Drop` — see below for where that sits beside
/// [`crate::shutdown`], which owns everything the terminal is told in escape sequences.
pub(crate) struct Tty {
    /// `None` once the input thread has adopted it, which is the last thing `attach` does. Detection
    /// is over by then, so nothing here reads again — and the `Tty` itself is kept, because raw mode
    /// and mode 2027 are its to give back.
    rx: Option<Receiver<Vec<u8>>>,
    pending: Vec<u8>,
    at: usize,
    /// Whether the batch went out, and therefore whether mode 2027 has to be given back.
    requested_2027: bool,
}

impl Tty {
    /// Take the terminal, or answer `None` when standard output is not one.
    ///
    /// A non-tty is **not** a failure and not headless: it is a terminal nothing is known about, so
    /// no queries are fired at all — there is nobody to answer, and the sentinel would burn the
    /// whole ceiling for nothing.
    pub(crate) fn open() -> Option<Tty> {
        use crossterm::tty::IsTty;

        refuse_in_tests();
        {
            // **Both ends, and the first draft asked only about stdout.** Detection writes to stdout and
            // reads the answers from stdin, so a tty on one side and a redirect on the other is not a
            // terminal for this purpose — and it was failing in the worst available way. With
            // `app < /dev/null` or a supervisor holding stdin, the reader thread saw EOF at once, the
            // channel disconnected, and `attach` answered `NoAnswer` on a perfectly good terminal. With
            // stdin redirected from a *non-empty* file it was worse than an error: the file's bytes were
            // fed to the parser as though the terminal had said them.
            //
            // Asking about both is the conservative half of the trade, and it is the right half: a
            // terminal that is only half connected falls through to declared defaults, which is exactly
            // where a terminal nothing is known about belongs.
            if !std::io::stdout().is_tty() || !std::io::stdin().is_tty() {
                return None;
            }
            if crossterm::terminal::enable_raw_mode().is_err() {
                return None;
            }
            let (tx, rx) = std::sync::mpsc::channel();
            let spawned = std::thread::Builder::new()
                .name("vitui-pty".to_string())
                .spawn(move || {
                    let mut stdin = std::io::stdin();
                    let mut buf = [0u8; 4096];
                    loop {
                        match stdin.read(&mut buf) {
                            Ok(0) => break,
                            Ok(n) => {
                                if tx.send(buf[..n].to_vec()).is_err() {
                                    break;
                                }
                            }
                            Err(e) if e.kind() == ErrorKind::Interrupted => {}
                            Err(_) => break,
                        }
                    }
                });
            if spawned.is_err() {
                let _ = crossterm::terminal::disable_raw_mode();
                return None;
            }
            Some(Tty {
                rx: Some(rx),
                pending: Vec::new(),
                at: 0,
                requested_2027: false,
            })
        }
    }

    /// What the terminal says its size is, which is the one thing there is no escape sequence for
    /// in this batch because the kernel already knows.
    pub(crate) fn size() -> Option<(u16, u16)> {
        crossterm::terminal::size().ok()
    }

    /// Hand the channel and everything still unread to the input thread.
    ///
    /// **Once, and after detection.** The `Tty` keeps its `Drop` — raw mode and mode 2027 are what
    /// it took and what it owes back — and gives up the only thing a second owner could use: the
    /// reader. Everything `unread` put back comes with it, because those bytes are the user's
    /// type-ahead and are older than every byte still in the channel.
    pub(crate) fn take_reader(&mut self) -> Option<(Receiver<Vec<u8>>, Vec<u8>)> {
        let rx = self.rx.take()?;
        let pending = self.pending.split_off(self.at.min(self.pending.len()));
        self.pending.clear();
        self.at = 0;
        Some((rx, pending))
    }
}

impl Drop for Tty {
    /// Give back the two things the batch changed.
    ///
    /// **Mode 2027 was being set and never reset**, which is a real asymmetry rather than a tidiness
    /// one: the batch pops the kitty flag stack in the same write it pushes it, and left 2027 on. A
    /// process that set it and exited — including one whose `attach` failed with `NoAnswer` and fell
    /// back to plain output — left the user's shell in grapheme-cluster mode it did not have before,
    /// with nothing on screen to say so.
    ///
    /// # Why these two are here and the rest of the epilogue is not
    ///
    /// [`crate::shutdown`] gives back everything that is an escape sequence *this engine chose to
    /// send*: the alt screen, auto-wrap, the caret, the kitty flags and the three input modes, from
    /// any thread, under a panic. These two are neither of those. Raw mode is a `termios` call and
    /// not a byte on the wire, and mode 2027 was set by **detection** — before `attach` had decided
    /// there would be a session at all, and still owed back by an `attach` that failed with
    /// `NoAnswer` and returned no `Screen` for a site to be armed on.
    ///
    /// So the boundary is where the thing was taken, and it is worth being exact about what that
    /// costs. A panic that unwinds through the thread holding the `Screen` runs this too, because
    /// the `Screen` owns the `Tty`. A panic on the **render** thread does not: the hook gives the
    /// terminal its modes back immediately, and raw mode comes back a moment later, when the app
    /// thread reads its `Wake::Quit` and drops the `Screen`. A shell that is briefly in raw mode is
    /// a shell that echoes nothing for one keystroke; a shell with the kitty flags still pushed is
    /// broken until somebody runs `reset`, which is why the two are not given equal urgency.
    fn drop(&mut self) {
        if self.requested_2027 {
            let mut out = std::io::stdout();
            let _ = out.write_all(b"\x1b[?2027l");
            let _ = out.flush();
        }
        let _ = crossterm::terminal::disable_raw_mode();
    }
}

impl Probe for Tty {
    fn write_batch(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        let mut out = std::io::stdout();
        out.write_all(bytes)?;
        out.flush()?;
        // The batch sets mode 2027, so from here on this `Tty` owes it back on drop.
        self.requested_2027 = true;
        Ok(())
    }

    fn unread(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        let mut kept = bytes.to_vec();
        kept.extend_from_slice(&self.pending[self.at..]);
        self.pending = kept;
        self.at = 0;
    }

    fn read(&mut self, buf: &mut [u8], timeout: Duration) -> std::io::Result<usize> {
        if self.at == self.pending.len() {
            let Some(rx) = self.rx.as_ref() else {
                return Err(std::io::Error::from(ErrorKind::BrokenPipe));
            };
            match rx.recv_timeout(timeout) {
                Ok(bytes) => {
                    self.pending = bytes;
                    self.at = 0;
                }
                Err(RecvTimeoutError::Timeout) => return Ok(0),
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(std::io::Error::from(ErrorKind::BrokenPipe));
                }
            }
        }
        let n = (self.pending.len() - self.at).min(buf.len());
        buf[..n].copy_from_slice(&self.pending[self.at..self.at + n]);
        self.at += n;
        Ok(n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caps::{KITTY_ALTERNATE_KEYS, KITTY_ASSOCIATED_TEXT, KITTY_EVENT_TYPES};
    use std::collections::VecDeque;

    /// A terminal on a script, and a clock that only moves when somebody waits.
    ///
    /// The point of a scripted probe rather than a real pty is not that a pty is awkward to open in
    /// CI. It is that **"reacts to the sentinel's arrival rather than to a clock" is a statement
    /// about time**, and the only way to assert a statement about time without a flake is to own
    /// the clock. Here time advances by exactly what somebody waited for, so a terminal can be made
    /// to answer over ten ceilings without the test taking ten ceilings.
    pub(crate) struct Scripted {
        pub(crate) writes: Vec<Vec<u8>>,
        chunks: VecDeque<(Duration, Vec<u8>)>,
        /// How many times a read hit its deadline with nothing to give.
        pub(crate) timeouts: usize,
        /// Simulated time spent waiting, in total.
        pub(crate) waited: Duration,
    }

    impl Scripted {
        /// A terminal that answers `bytes` all at once, immediately.
        fn at_once(bytes: &str) -> Scripted {
            Scripted::dribbled(&[(Duration::ZERO, bytes)])
        }

        /// A terminal that answers in pieces, each after a gap.
        fn dribbled(script: &[(Duration, &str)]) -> Scripted {
            Scripted {
                writes: Vec::new(),
                chunks: script
                    .iter()
                    .map(|(d, s)| (*d, s.as_bytes().to_vec()))
                    .collect(),
                timeouts: 0,
                waited: Duration::ZERO,
            }
        }

        /// A terminal that is not there.
        fn silent() -> Scripted {
            Scripted::dribbled(&[])
        }
    }

    impl Probe for Scripted {
        fn write_batch(&mut self, bytes: &[u8]) -> std::io::Result<()> {
            self.writes.push(bytes.to_vec());
            Ok(())
        }

        fn unread(&mut self, bytes: &[u8]) {
            if !bytes.is_empty() {
                self.chunks.push_front((Duration::ZERO, bytes.to_vec()));
            }
        }

        fn read(&mut self, buf: &mut [u8], timeout: Duration) -> std::io::Result<usize> {
            match self.chunks.front_mut() {
                None => {
                    self.timeouts += 1;
                    self.waited += timeout;
                    Ok(0)
                }
                Some((after, _)) if *after > timeout => {
                    *after -= timeout;
                    self.timeouts += 1;
                    self.waited += timeout;
                    Ok(0)
                }
                Some(_) => {
                    let (after, bytes) = self.chunks.pop_front().expect("just matched");
                    self.waited += after;
                    let n = bytes.len().min(buf.len());
                    buf[..n].copy_from_slice(&bytes[..n]);
                    if n < bytes.len() {
                        self.chunks
                            .push_front((Duration::ZERO, bytes[n..].to_vec()));
                    }
                    Ok(n)
                }
            }
        }
    }

    /// What a modern terminal that implements everything answers, in the batch's own order.
    fn full_answer() -> String {
        let mut s = String::new();
        s.push_str("\x1bP>|kitty(0.32.2)\x1b\\");
        s.push_str("\x1b[>1;4000;19c");
        s.push_str("\x1bP1+r524742=54727565\x1b\\");
        s.push_str("\x1b]10;rgb:c5c5/c8c8/c6c6\x1b\\");
        s.push_str("\x1b]11;rgb:1d1d/1f1f/2121\x1b\\");
        for i in 0..16 {
            s.push_str(&format!("\x1b]4;{i};rgb:{i:02x}{i:02x}/0000/ffff\x1b\\"));
        }
        s.push_str("\x1b[?2027;1$y");
        s.push_str("\x1b[?2026;2$y");
        s.push_str("\x1b[?1000;2$y");
        s.push_str("\x1b[?1003;2$y");
        s.push_str("\x1b[?1004;2$y");
        s.push_str("\x1b[?2004;2$y");
        s.push_str("\x1b[?69;0$y");
        s.push_str("\x1b[?0u");
        s.push_str("\x1b[?31u");
        s.push_str("\x1b[?64;1;9c");
        s
    }

    /// **Gate.** One write, every query in it, and the sentinel last.
    ///
    /// The seventeen queries the degradation model added cost no extra round trip, and this is the
    /// assertion that says so: they are in the same batch, ahead of the DA1 that ends the wait.
    #[test]
    fn every_query_goes_out_in_one_write_behind_a_trailing_da1() {
        let mut pty = Scripted::at_once(&full_answer());
        detect(&mut pty, CEILING).expect("the terminal answered");

        assert_eq!(
            pty.writes.len(),
            1,
            "a batch split across writes is not a batch"
        );
        let sent = String::from_utf8(pty.writes[0].clone()).expect("the batch is text");
        assert!(sent.ends_with("\x1b[c"), "DA1 is not last: {sent:?}");
        assert_eq!(
            sent.matches("\x1b[c").count(),
            1,
            "one sentinel, not several"
        );

        for query in [
            "\x1b[>0q",            // XTVERSION
            "\x1b[>c",             // DA2
            "\x1bP+q524742\x1b\\", // XTGETTCAP RGB
            "\x1b]10;?\x1b\\",     // OSC 10
            "\x1b]11;?\x1b\\",     // OSC 11
            "\x1b[?2026$p",        // synchronised output
            "\x1b[?2027$p",        // grapheme clusters
            "\x1b[?1000$p",        // mouse
            "\x1b[?1003$p",        // mouse motion
            "\x1b[?1004$p",        // focus
            "\x1b[?2004$p",        // bracketed paste
            "\x1b[?69$p",          // DECSLRM
            "\x1b[?u",             // the kitty flags
            "\x1b[<u",             // and the stack put back
        ] {
            assert!(sent.contains(query), "the batch is missing {query:?}");
        }
        for i in 0..16 {
            assert!(
                sent.contains(&format!("\x1b]4;{i};?\x1b\\")),
                "the batch is missing palette entry {i}"
            );
        }
    }

    /// **Gate.** Mode 2027 is given back.
    ///
    /// The batch pops the kitty flag stack in the same write it pushes it, and the first draft set
    /// mode 2027 and never reset it. A process that set it and exited — including one whose `attach`
    /// failed with `NoAnswer` and fell back to plain output — left the user's shell in
    /// grapheme-cluster mode it did not have before, with nothing on screen to say so.
    #[test]
    fn every_mode_the_batch_changes_is_given_back() {
        let sent = String::from_utf8(batch()).expect("the batch is text");
        assert!(sent.contains("\x1b[?2027h"), "2027 is set");
        assert!(sent.contains("\x1b[>31u"), "the kitty stack is pushed");
        assert!(
            sent.contains("\x1b[<u"),
            "and popped in the same write, which is what 2027's reset is modelled on"
        );
        // 2027's reset cannot be in the batch — it has to outlive detection — so it is `Drop`'s,
        // and `Tty::requested_2027` is what makes it conditional on the batch having gone out.
        assert!(
            !sent.contains("\x1b[?2027l"),
            "the reset belongs to Drop, not the batch"
        );
    }

    /// **Gate.** Mode 2027 is *requested* once, and only once.
    #[test]
    fn mode_2027_is_requested_exactly_once() {
        let sent = String::from_utf8(batch()).expect("the batch is text");
        assert_eq!(sent.matches("\x1b[?2027h").count(), 1);
        assert!(
            !sent.contains("\x1b[?2026h"),
            "2026 is per frame, not a mode to set"
        );
    }

    /// The kitty flags are read back **after** a push, which is the whole reason there are four
    /// booleans here and not a `KeyboardTier`: what a terminal accepts is not what it was offered.
    #[test]
    fn the_kitty_flags_are_the_ones_that_survived_the_push() {
        let sent = String::from_utf8(batch()).expect("the batch is text");
        let before = sent.find("\x1b[?u").expect("a read before");
        let push = sent.find("\x1b[>31u").expect("the push");
        let after = sent[before + 1..].find("\x1b[?u").expect("a read after") + before + 1;
        let pop = sent.find("\x1b[<u").expect("the pop");
        assert!(
            before < push && push < after && after < pop,
            "the batch is out of order"
        );

        // A terminal that takes the encoding and none of the flag stack: it answers, and it answers
        // zero. tmux is the real one, and a tier would have called it `Full`.
        let mut lying = String::new();
        lying.push_str("\x1b[?0u");
        lying.push_str("\x1b[?0u");
        lying.push_str("\x1b[?64;1;9c");
        let mut pty = Scripted::at_once(&lying);
        let out = detect(&mut pty, CEILING).expect("it answered DA1");
        assert_eq!(out.kitty_flags, Some(0));

        let mut pty = Scripted::at_once(&full_answer());
        let out = detect(&mut pty, CEILING).expect("it answered DA1");
        let flags = out.kitty_flags.expect("kitty answered");
        assert_eq!(flags, 31, "the second answer wins, not the first");
        assert!(flags & KITTY_EVENT_TYPES != 0);
        assert!(flags & KITTY_ALTERNATE_KEYS != 0);
        assert!(flags & KITTY_ASSOCIATED_TEXT != 0);
    }

    /// **Gate.** The numeric ceiling fires only where there is no terminal at all, and it fires once.
    #[test]
    fn the_ceiling_fires_only_when_nothing_answers_at_all() {
        let mut pty = Scripted::silent();
        let err = detect(&mut pty, CEILING).expect_err("a pipe answers nothing");
        assert!(matches!(err, AttachError::NoAnswer));
        assert_eq!(
            pty.timeouts, 1,
            "one ceiling is the whole cost of finding out"
        );
        assert_eq!(pty.waited, CEILING);
    }

    /// **Gate.** A terminal that answers slowly is never cut off, however long it takes in total.
    ///
    /// Ten gaps of nine tenths of a ceiling is **nine ceilings of wall clock**, and a numeric
    /// deadline on the whole exchange would have truncated this terminal's answers into a
    /// conservative tier while every test that used a fast terminal stayed green. The deadline is
    /// idle rather than total, and that is what this asserts.
    #[test]
    fn a_terminal_that_answers_slowly_is_never_cut_off() {
        let answer = full_answer();
        let gap = CEILING.mul_f32(0.9);
        let piece = answer.len().div_ceil(10);
        let mut script: Vec<(Duration, String)> = Vec::new();
        let mut at = 0;
        while at < answer.len() {
            // Split on a character boundary; the parser is fed whatever the kernel felt like.
            let mut end = (at + piece).min(answer.len());
            while !answer.is_char_boundary(end) {
                end += 1;
            }
            script.push((gap, answer[at..end].to_string()));
            at = end;
        }
        let borrowed: Vec<(Duration, &str)> =
            script.iter().map(|(d, s)| (*d, s.as_str())).collect();
        let mut pty = Scripted::dribbled(&borrowed);

        let out = detect(&mut pty, CEILING).expect("it answered, slowly");
        assert!(out.answered, "the sentinel arrived");
        assert_eq!(
            pty.timeouts, 0,
            "no deadline fired: every gap was under the ceiling"
        );
        assert!(
            pty.waited > CEILING * 8,
            "the exchange took {:?}, which is not slow enough to be the test",
            pty.waited
        );
        assert_eq!(out.version.as_deref(), Some("kitty(0.32.2)"));
        assert_eq!(out.palette[15], Some(Rgb::new(0x0f, 0x00, 0xff)));
    }

    /// A terminal that starts answering and then stops is not an error: what arrived is kept, and
    /// the rest falls through to the conservative defaults it would have had anyway.
    #[test]
    fn a_terminal_that_goes_quiet_mid_answer_keeps_what_arrived() {
        let mut pty = Scripted::at_once("\x1b]11;rgb:1111/2222/3333\x1b\\");
        let out = detect(&mut pty, CEILING).expect("something arrived");
        assert!(!out.answered, "the sentinel never came");
        assert_eq!(out.default_bg, Some(Rgb::new(0x11, 0x22, 0x33)));
        assert_eq!(pty.timeouts, 1);
    }

    /// The parser is incremental because a pty splits sequences wherever it likes — and a parser
    /// that assumed whole sequences would work on every fast terminal and fail on every slow one,
    /// which is the failure mode that survives testing and reaches a user.
    #[test]
    fn a_sequence_split_at_every_byte_still_parses() {
        let answer = full_answer();
        let mut parser = Parser::default();
        let mut out = Detected::default();
        let mut stopped = false;
        for b in answer.as_bytes() {
            if parser.feed(&[*b], &mut out).is_some() {
                stopped = true;
                break;
            }
        }
        assert!(stopped, "the sentinel was seen");
        assert_eq!(out.version.as_deref(), Some("kitty(0.32.2)"));
        assert_eq!(out.da2, Some((1, 4000, 19)));
        assert_eq!(out.rgb, Some(true));
        assert_eq!(out.default_fg, Some(Rgb::new(0xc5, 0xc8, 0xc6)));
        assert!(out.mode(MODE_GRAPHEME_CLUSTERS));
        assert!(out.mode(MODE_SYNC_OUTPUT));
        assert!(!out.mode(MODE_DECSLRM), "a Pm of zero is not recognised");
        assert_eq!(out.kitty_flags, Some(31));
    }

    /// The sentinel ends the wait, and **what came after it is handed back**.
    ///
    /// DA1 is rarely the last byte of the last read: a key pressed while the application was
    /// starting is sitting right behind it. Detection neither reads those bytes nor eats them, which
    /// is what makes "one reader of this descriptor, ever" a whole promise rather than half of one.
    #[test]
    fn nothing_after_the_sentinel_is_consumed_and_none_of_it_is_lost() {
        let mut answer = full_answer();
        answer.push_str("\x1b]11;rgb:ffff/ffff/ffff\x1b\\q");
        let mut pty = Scripted::at_once(&answer);
        let out = detect(&mut pty, CEILING).expect("it answered");
        assert_eq!(
            out.default_bg,
            Some(Rgb::new(0x1d, 0x1f, 0x21)),
            "a later OSC 11 was read past the sentinel"
        );

        let mut left = [0u8; 64];
        let n = pty
            .read(&mut left, CEILING)
            .expect("the tail was handed back");
        assert_eq!(
            &left[..n],
            b"\x1b]11;rgb:ffff/ffff/ffff\x1b\\q",
            "the application's own bytes were eaten by detection"
        );
    }

    /// XTGETTCAP has three answers and they are not two. Silence means *this terminal does not
    /// implement XTGETTCAP*, which is not the claim *this terminal has no truecolor*.
    #[test]
    fn the_three_answers_of_xtgettcap_are_kept_apart() {
        let ask = |dcs: &str| {
            let mut pty = Scripted::at_once(&format!("{dcs}\x1b[?64;1;9c"));
            detect(&mut pty, CEILING).expect("it answered").rgb
        };
        assert_eq!(ask("\x1bP1+r524742=54727565\x1b\\"), Some(true));
        assert_eq!(ask("\x1bP0+r524742\x1b\\"), Some(false));
        assert_eq!(ask(""), None);
    }

    /// **The user's own keystrokes survive detection.**
    ///
    /// A pty has one input stream and the person at the keyboard does not wait to be asked, so
    /// type-ahead arrives interleaved with the replies — and the reader thread starts *before* the
    /// batch goes out, so a key pressed at the shell prompt is already in the channel. The first
    /// draft parsed those bytes as answers and dropped them, which is the same hole `unread` closes
    /// on the far side of the sentinel and the likelier one: type-ahead at startup is ordinary.
    #[test]
    fn type_ahead_before_the_sentinel_is_handed_back_not_eaten() {
        let mut answer = String::from("q");
        answer.push_str("\x1b]11;rgb:1d1d/1f1f/2121\x1b\\");
        answer.push_str("\x1b[A"); // an arrow key, shaped exactly like a reply
        answer.push_str("\x1b[?64;1;9c");
        answer.push('Z'); // and one after the sentinel
        let mut pty = Scripted::at_once(&answer);

        let out = detect(&mut pty, CEILING).expect("it answered");
        assert_eq!(
            out.default_bg,
            Some(Rgb::new(0x1d, 0x1f, 0x21)),
            "the reply between two keystrokes was still read"
        );

        let mut left = [0u8; 64];
        let n = pty
            .read(&mut left, CEILING)
            .expect("the input was handed back");
        assert_eq!(
            &left[..n],
            b"q\x1b[AZ",
            "a keystroke was eaten: detection kept bytes that were never an answer"
        );
    }

    /// Type-ahead is handed back even when the sentinel never arrives, because the bytes are the
    /// user's whether or not the terminal finished talking.
    #[test]
    fn type_ahead_survives_an_exchange_that_times_out() {
        let mut pty = Scripted::at_once("hello");
        let out = detect(&mut pty, CEILING).expect("something arrived");
        assert!(!out.answered);

        let mut left = [0u8; 16];
        let n = pty.read(&mut left, CEILING).expect("handed back");
        assert_eq!(&left[..n], b"hello");
    }

    /// **`ESC` aborts a string; it is not content.**
    ///
    /// Absorbing it was a bug with a wide blast radius: one arrow key pressed while an OSC 11 reply
    /// was in flight pushed `ESC [` into the payload, and every byte after it — **the sentinel
    /// included** — was eaten as OSC content until the next terminator. Detection then waited out
    /// the whole idle ceiling and lost the rest of the batch.
    #[test]
    fn an_escape_inside_a_string_aborts_it_rather_than_swallowing_the_batch() {
        // An OSC 11 cut in half by an arrow key, then the rest of the batch behind it.
        let mut answer = String::from("\x1b]11;rgb:1d1d/1f");
        answer.push_str("\x1b[A");
        answer.push_str("\x1bP>|kitty(0.32.2)\x1b\\");
        answer.push_str("\x1b[?64;1;9c");
        let mut pty = Scripted::at_once(&answer);

        let out = detect(&mut pty, CEILING).expect("the sentinel was reached");
        assert!(
            out.answered,
            "the sentinel was swallowed by the aborted string"
        );
        assert_eq!(
            out.version.as_deref(),
            Some("kitty(0.32.2)"),
            "the reply after the aborted string was lost"
        );
        assert_eq!(
            out.default_bg, None,
            "half an OSC 11 colour must be discarded, not parsed — a wrong background is wrong in \
             direction, not degree"
        );
        assert_eq!(
            pty.timeouts, 0,
            "nothing waited on a string that never ended"
        );
    }

    #[test]
    fn a_colour_is_read_in_every_form_a_terminal_spells_it() {
        assert_eq!(
            colour("rgb:c5c5/c8c8/c6c6"),
            Some(Rgb::new(0xc5, 0xc8, 0xc6))
        );
        assert_eq!(colour("rgb:12/34/56"), Some(Rgb::new(0x12, 0x34, 0x56)));
        assert_eq!(colour("rgb:f/0/8"), Some(Rgb::new(0xff, 0x00, 0x88)));
        assert_eq!(colour("#123456"), Some(Rgb::new(0x12, 0x34, 0x56)));
        assert_eq!(
            colour("rgb:ffff/ffff/ffff"),
            Some(Rgb::new(0xff, 0xff, 0xff))
        );
        assert_eq!(colour("rgb:0000/0000/0000"), Some(Rgb::new(0, 0, 0)));
        assert_eq!(colour("nonsense"), None);
        assert_eq!(colour("rgb:zz/00/00"), None);
    }
}
