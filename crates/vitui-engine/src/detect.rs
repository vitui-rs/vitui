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
    loop {
        match probe.read(&mut buf, ceiling) {
            Ok(0) | Err(_) if !heard => return Err(AttachError::NoAnswer),
            // The idle deadline fired after the terminal had started answering, or the pty broke
            // mid-answer. Keep what arrived; the sentinel is a fast path, not the only exit.
            Ok(0) | Err(_) => break,
            Ok(n) => {
                heard = true;
                if let Some(used) = parser.feed(&buf[..n], &mut out) {
                    probe.unread(&buf[used..n]);
                    break;
                }
            }
        }
    }
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

    fn step(&mut self, b: u8, out: &mut Detected) -> bool {
        match self.state {
            State::Ground => {
                if b == ESC {
                    self.state = State::Esc;
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
                    _ => self.state = State::Ground,
                }
            }
            State::Csi => {
                // Anything in 0x40..=0x7E ends a control sequence; everything before it is
                // parameters and intermediates, and both are kept because DECRQM's answer is told
                // apart from a key press by its `$`.
                if (0x40..=0x7e).contains(&b) {
                    self.state = State::Ground;
                    let payload = std::mem::take(&mut self.buf);
                    return csi(&payload, b, out);
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
                    self.buf.push(ESC);
                    self.buf.push(b);
                    self.state = if self.string_is_dcs {
                        State::Dcs
                    } else {
                        State::Osc
                    };
                }
            }
        }
        false
    }
}

/// A finished control sequence. Returns true for DA1, which is the sentinel and ends the loop.
fn csi(payload: &[u8], final_byte: u8, out: &mut Detected) -> bool {
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
            return true;
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
            }
        }
        _ => {}
    }
    false
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
            if let (Some(i), Some(v)) = (index, value) {
                if i < out.palette.len() {
                    out.palette[i] = Some(v);
                }
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
/// is never coming. Restoring it on drop is the floor rather than the design: **ticket 22 owns
/// shutdown** — the alt screen, the panic hook, and restoration that is idempotent under both.
pub(crate) struct Tty {
    rx: Receiver<Vec<u8>>,
    pending: Vec<u8>,
    at: usize,
}

impl Tty {
    /// Take the terminal, or answer `None` when standard output is not one.
    ///
    /// A non-tty is **not** a failure and not headless: it is a terminal nothing is known about, so
    /// no queries are fired at all — there is nobody to answer, and the sentinel would burn the
    /// whole ceiling for nothing.
    pub(crate) fn open() -> Option<Tty> {
        use crossterm::tty::IsTty;

        if !std::io::stdout().is_tty() {
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
            rx,
            pending: Vec::new(),
            at: 0,
        })
    }

    /// What the terminal says its size is, which is the one thing there is no escape sequence for
    /// in this batch because the kernel already knows.
    pub(crate) fn size() -> Option<(u16, u16)> {
        crossterm::terminal::size().ok()
    }
}

impl Drop for Tty {
    fn drop(&mut self) {
        let _ = crossterm::terminal::disable_raw_mode();
    }
}

impl Probe for Tty {
    fn write_batch(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        let mut out = std::io::stdout();
        out.write_all(bytes)?;
        out.flush()
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
            match self.rx.recv_timeout(timeout) {
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
