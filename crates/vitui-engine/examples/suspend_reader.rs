//! Register entry #31: **a suspension does not vacate standard input.**
//!
//! ```text
//! scripts/suspend-reader-gate.sh
//! ```
//!
//! # Why a binary and not a test
//!
//! The property is *the `vitui-pty` thread is still reading the terminal while this `Screen` is
//! suspended*, and no gate in this crate can state it. `Tty::open` panics under `cfg(test)`, so no
//! test in this workspace ever spawns that thread at all — every keystroke a headless gate sees was
//! put into the queue by `Screen::inject`, which is this side of the reader and cannot say whether
//! the reader exists. `crate::gates::a_resume_delivers_no_keystroke_from_the_suspension_and_every_resize`
//! is the decidable half — *what happens to the bytes it took* — and it is decidable precisely
//! because it never asks where they came from.
//!
//! So this is the other half, on a real pty, with `script(1)` for the terminal and
//! `scripts/suspend-reader-gate.sh` for the hand that types.
//!
//! # The property is a refusal, so this gate is a tripwire
//!
//! Nothing here is trying to make the reader stop. ADR 0052 and engine spec §7 both say it never
//! will, `Screen::suspend`'s first paragraph says it to a caller, and `Screen::resume` throws away
//! what arrived because of it. **Four documents claim it and, before this file, nothing watched
//! it.** If someone makes the read cancellable — the four ways it could be done are priced in ADR
//! 0052 and all four were refused — this gate goes red, and that is the point: the doc paragraph,
//! the discard in `resume`, and production ticket 13's answer all have to be revisited in the same
//! edit, and a red gate is what puts them in front of whoever makes the change.
//!
//! # The protocol, which has no sleep in it
//!
//! Three markers on standard output, and the harness types only when it has seen the one before it.
//! Timing is what makes a gate flaky, so there is none: the harness polls the typescript for a
//! marker and this side polls the queue for a key, each with a deadline that fails loudly.
//!
//! 0. before anything, the harness writes a DA1 reply — because `script(1)` allocates a pty and a
//!    pty is not an emulator, so without one, detection reads nothing inside its 250 ms ceiling and
//!    there is no session here to suspend. Written unprompted rather than in answer to the batch,
//!    because *answering* it is a race against that ceiling and writing it first is not a race at
//!    all. Every other question stays at its conservative default, which is the honest answer for a
//!    terminal that is not one.
//! 1. `[MARK] armed` — attached, negotiated, nothing typed yet. The harness types `a`.
//! 2. the control arm: `a` arrives, which is this pty delivering a keystroke **before** any
//!    suspension. Without it the arm below is met by a terminal that was never able to type at all.
//! 3. `[MARK] suspended` — the epilogue is out, raw mode is gone, the terminal is the user's. The
//!    harness types `b\n`, and the newline is not decoration: raw mode is *off* during a suspension,
//!    so the line discipline is canonical again and holds a bare `b` until one arrives.
//! 4. the subject: `b` arrives **while suspended**, which is the reader that did not stop.
//!
//! The verdict goes to the file named in `argv[1]` rather than to the terminal, so that the harness
//! reads a decision instead of parsing a typescript full of escape sequences — and it is written
//! after the `Screen` is dropped, so its existence also means the terminal was given back.

use std::io::Write;
use std::time::{Duration, Instant};

use vitui_engine::{Config, Engine, Event, KeyCode, Screen};

/// How long either arm may wait for a keystroke before it is a failure rather than a slow machine.
///
/// Generous on purpose: the harness has no sleep in it, so the only thing this bounds is a CI
/// container under load. A gate that hangs is worse than one that fails — the repository's position
/// on `retry` says so in as many words — so both arms end.
const PATIENCE: Duration = Duration::from_secs(15);

/// A marker the harness waits for, with the `\r` a raw-mode terminal needs and a cooked one ignores.
fn mark(what: &str) {
    print!("[MARK] {what}\r\n");
    let _ = std::io::stdout().flush();
}

/// Pop events until `wanted` is one of them, or the patience runs out.
///
/// **`next_event` and not `wait`.** The app thread is not parked here, it is polling, and the two
/// millisecond sleep is what keeps that from being a spin — this file measures nothing, so the cost
/// of the poll is not a subject. `wait` would be wrong for the second arm anyway: there is no frame
/// to owe and no renderer to be free during a suspension.
fn typed(screen: &mut Screen, wanted: char) -> bool {
    let deadline = Instant::now() + PATIENCE;
    loop {
        while let Some(event) = screen.next_event() {
            if let Event::Key(key) = event
                && key.code == KeyCode::Char(wanted)
            {
                return true;
            }
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn main() {
    let Some(verdict) = std::env::args().nth(1) else {
        eprintln!(
            "usage: suspend_reader <verdict-file>; run it through scripts/suspend-reader-gate.sh"
        );
        std::process::exit(2);
    };

    let (mut screen, _wake) = match Engine::new(Config::default()).attach() {
        Ok(pair) => pair,
        Err(why) => {
            // The verdict file is still written, because its absence is how the harness reports an
            // instrument that never ran and this is a `Screen` that never existed — a different
            // failure, and one worth telling apart from a pty that was never allocated.
            let _ = std::fs::write(&verdict, format!("attach=failed\nwhy={why}\n"));
            std::process::exit(1);
        }
    };

    // **A permit for the whole run, and it is honest rather than defensive.** §11's in-loop overrun
    // detector watches the app thread, and this thread is about to spend up to fifteen seconds
    // waiting for a person — or for a shell script pretending to be one. That is exactly what
    // `permit_slow` is for, and without it a debug build reports an overrun that is a property of
    // the harness.
    let _permit = screen.permit_slow("waiting for the harness to type");

    mark("armed");
    let control = typed(&mut screen, 'a');

    // Nothing is drawn, before or after. A frame would put the compositor between the terminal and
    // the answer, and every byte it wrote would be in the typescript the harness greps.
    screen.suspend();
    mark("suspended");
    // Skipped when the control arm failed, because the subject is then unreadable — a `b` that never
    // arrives says nothing about the reader when an `a` did not arrive either — and because fifteen
    // seconds is a long time to spend confirming that a broken instrument is still broken.
    let during = if control {
        Some(typed(&mut screen, 'b'))
    } else {
        None
    };
    screen.resume();
    mark("resumed");
    drop(screen);

    let during = match during {
        Some(true) => "ok",
        Some(false) => "timeout",
        None => "skipped",
    };
    let report = format!(
        "attach=ok\ncontrol={}\nduring={during}\n",
        if control { "ok" } else { "timeout" }
    );
    if let Err(why) = std::fs::write(&verdict, report) {
        eprintln!("suspend_reader: could not write the verdict to {verdict}: {why}");
        std::process::exit(1);
    }
}
