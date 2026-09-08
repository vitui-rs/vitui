//! The one thread that reads the terminal.
//!
//! # It adopts detection's reader rather than opening its own
//!
//! `crate::detect::Tty` spawns a thread that blocks on standard input before the query batch goes
//! out, because a key pressed at the shell prompt is already in flight when a program starts. That
//! thread is **kept**, and this is what adopts its channel: a second reader of the same file
//! descriptor steals bytes from the first, silently, and only under load.
//!
//! So there are two threads on this side and not one — a `read` that blocks, and this one, which
//! parses and never blocks on anything but the channel. The split is detection's, not a choice made
//! here: a deadline on a `read` needs a channel in this crate's dependency policy, and the deadline
//! is what the capability batch is built on.
//!
//! # It is never joined
//!
//! The input thread sits in a blocking `read` with nothing to wake it short of a signal, so
//! it dies with the process. The render thread **is** joined — it is either parked on a condvar or
//! inside a bounded write, both finite — and this one is not, and the asymmetry is deliberate rather
//! than an omission.

use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::Instant;

use crate::clock::WakeSource;
use crate::handoff::TerminalSize;
use crate::input::parse::Parser;
use crate::input::{Event, Queue};

/// Everything the thread needs, so that the spawn site reads as one thing.
pub(crate) struct Wiring {
    /// Detection's channel, adopted.
    pub(crate) reads: Receiver<Vec<u8>>,
    /// What detection read past the sentinel and never handed to anyone: type-ahead, in order.
    pub(crate) type_ahead: Vec<u8>,
    pub(crate) queue: Arc<Queue>,
    pub(crate) wakes: Arc<WakeSource>,
    pub(crate) size: Arc<TerminalSize>,
    pub(crate) paste_limit: usize,
    /// How to ask the terminal its size. A function pointer rather than a call to
    /// `crate::detect::Tty::size`, so that a test can drive this loop without a terminal.
    pub(crate) measure: fn() -> Option<(u16, u16)>,
}

/// Start it. The handle is dropped: nothing joins this thread, ever.
pub(crate) fn spawn(wiring: Wiring) {
    let spawned = std::thread::Builder::new()
        .name("vitui-input".to_string())
        .spawn(move || run(wiring));
    // A process that cannot spawn a thread is about to fail at something louder than this. The
    // engine still runs — it simply never delivers an event — which is strictly better than
    // refusing to attach over an input pipeline the application may not use.
    drop(spawned);
}

/// Parse until the channel closes.
pub(crate) fn run(wiring: Wiring) {
    let Wiring {
        reads,
        type_ahead,
        queue,
        wakes,
        size,
        paste_limit,
        measure,
    } = wiring;
    let mut parser = Parser::new(paste_limit);
    let mut published = 0u64;

    // The type-ahead first, and before anything else can be read: those bytes are older than every
    // byte the channel still holds, and a hand-back out of order reorders somebody's keystrokes.
    if !type_ahead.is_empty() {
        let woke = drain(&mut parser, &type_ahead, &queue, &mut published);
        if woke {
            wakes.input();
        }
    }

    while let Ok(bytes) = reads.recv() {
        // **The size is re-sampled here**, on a thread that just woke anyway, and the store is the
        // one packed word spec §7 makes authoritative. See `resize` below for what this does not
        // catch and why the alternative was not available.
        let before = size.get();
        if let Some(now) = measure()
            && now != before
            && now.0 > 0
            && now.1 > 0
        {
            size.set(now);
            queue.push(Event::Resize(now.0, now.1));
        }
        let woke = drain(&mut parser, &bytes, &queue, &mut published) || size.get() != before;
        if woke {
            wakes.input();
        }
    }

    // **The terminal is gone, and this is the only place in the process that can know.**
    //
    // The channel closes when detection's blocking `read` on standard input returned `Ok(0)` or an
    // error, and `Tty::open` refuses to hand a reader over unless *both* ends are a terminal — so
    // an end-of-file here is not a redirect and not an empty file. It is the pty's far side
    // closing: the ssh connection dropped, the terminal window was closed, the multiplexer detached.
    //
    // **An error counts as the same thing, and that is deliberate rather than sloppy.** A closed pty
    // master presents as end-of-file on macOS and as `EIO` on Linux, so a quit that fired only on
    // `Ok(0)` would answer this question correctly on one of the two platforms this workspace
    // builds for. The way to get a spurious one is to put this process's reader in a **background**
    // process group — where a `read` on the controlling terminal raises `SIGTTIN`, and an orphaned
    // or ignoring group gets `EIO` back — and that is reachable only by handing the terminal to a
    // child that takes the foreground while this process keeps running, which is the case
    // `Screen::suspend` documents that it does not support, for the reason stated there: nothing in
    // safe Rust cancels the blocking `read` this thread is fed by.
    //
    // What the process would otherwise do is worse than a crash and looks like nothing at all.
    // Every write fails and is discarded — the frame path has no `Result` in it — so
    // `present` goes on answering `submitted: true` for ever, and an application parked in
    // `Screen::wait` waits on a keyboard that cannot send another byte. **A hang is worse than a
    // failure**, and this is the flag that makes it a failure, exactly as `Mailbox::gone` is for the
    // render thread.
    //
    // `Wake::Quit` rather than a fifth spelling invented here, and it is the same argument
    // `WakeSource::renderer_gone` makes one file over: an application that handles quit already does
    // the right thing, and one that does not was going to hang either way.
    wakes.quit();
}

/// One read's worth: parse it, publish the diagnostics if they moved, and say whether anything
/// reached the queue.
fn drain(parser: &mut Parser, bytes: &[u8], queue: &Queue, published: &mut u64) -> bool {
    let mut any = false;
    let at = Instant::now();
    {
        let mut sink = |event: Event| {
            any = true;
            queue.push(event);
        };
        parser.feed(bytes, at, &mut sink);
        // The read is over, so a bare `ESC` is the Escape key. See `super::parse`'s module
        // documentation for the two alternatives and what each costs.
        parser.end_of_read(at, &mut sink);
    }
    let (unrecognised, last) = parser.diagnostics();
    if unrecognised != *published {
        *published = unrecognised;
        queue.set_diagnostics(unrecognised, last);
    }
    any
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{Event, KeyCode};
    use std::sync::mpsc::{Sender, channel};

    /// The size the fake terminal answers with. **A constant per test rather than a shared
    /// static**, which the first draft used and which was a race: `cargo test` runs a binary's
    /// tests on several threads, and one test setting the size out from under another is a flake
    /// with a fixture's clothes on. A function pointer costs nothing and cannot be raced.
    const STARTED_AT: (u16, u16) = (80, 24);

    fn unchanged() -> Option<(u16, u16)> {
        Some(STARTED_AT)
    }

    fn resized() -> Option<(u16, u16)> {
        Some((120, 40))
    }

    fn gone() -> Option<(u16, u16)> {
        Some((0, 0))
    }

    /// The loop, on this thread, over a channel that is closed before it starts.
    ///
    /// **Not a spawn**: `run` returns when the channel closes, so feeding it everything up front and
    /// then dropping the sender makes this a straight-line program with no join, no timeout and no
    /// flake — the same shape the deterministic clock gives the frame path.
    fn drive(
        reads: &[&[u8]],
        type_ahead: &[u8],
        measure: fn() -> Option<(u16, u16)>,
    ) -> (Arc<Queue>, Arc<WakeSource>, Arc<TerminalSize>) {
        let (tx, rx): (Sender<Vec<u8>>, _) = channel();
        for read in reads {
            tx.send(read.to_vec()).expect("the receiver is alive");
        }
        drop(tx);
        let queue = Arc::new(Queue::new());
        let wakes = Arc::new(WakeSource::new());
        let size = Arc::new(TerminalSize::new(STARTED_AT));
        run(Wiring {
            reads: rx,
            type_ahead: type_ahead.to_vec(),
            queue: Arc::clone(&queue),
            wakes: Arc::clone(&wakes),
            size: Arc::clone(&size),
            paste_limit: 1 << 20,
            measure,
        });
        (queue, wakes, size)
    }

    fn codes(queue: &Queue) -> Vec<Event> {
        std::iter::from_fn(|| queue.pop()).collect()
    }

    #[test]
    fn the_loop_parses_a_read_and_posts_one_wake() {
        let (queue, wakes, _) = drive(&[b"ab"], b"", unchanged);
        assert_eq!(codes(&queue).len(), 2);
        assert_eq!(
            wakes.pending(),
            crate::clock::INPUT | crate::clock::QUIT,
            "one post, not one per event — and the quit `drive`'s closed channel always ends on"
        );
    }

    /// The type-ahead is **older** than everything in the channel and has to reach the queue first.
    /// A hand-back out of order reorders somebody's keystrokes, which is the defect detection's own
    /// `unread` was written to avoid on the other side of the sentinel.
    #[test]
    fn the_type_ahead_is_parsed_before_the_first_read() {
        let (queue, _, _) = drive(&[b"b"], b"a", unchanged);
        let seen: Vec<_> = codes(&queue)
            .into_iter()
            .map(|event| match event {
                Event::Key(key) => key.code,
                other => panic!("unexpected {other:?}"),
            })
            .collect();
        assert_eq!(seen, vec![KeyCode::Char('a'), KeyCode::Char('b')]);
    }

    /// The size is written **and** an event is queued, in that order: the app thread samples the
    /// atomic at frame start and would otherwise compose one frame at the size it just stopped
    /// being.
    #[test]
    fn a_resize_writes_the_authoritative_size_and_queues_an_event() {
        let (queue, wakes, size) = drive(&[b"a"], b"", resized);
        assert_eq!(size.get(), (120, 40));
        match codes(&queue).first() {
            Some(Event::Resize(120, 40)) => {}
            other => panic!("expected the resize first, got {other:?}"),
        }
        assert_eq!(wakes.pending(), crate::clock::INPUT | crate::clock::QUIT);
    }

    /// A read that changes nothing about the size produces no resize at all, which is what keeps
    /// `Screen::resize` — a full repaint — off every keystroke.
    #[test]
    fn a_read_at_an_unchanged_size_queues_no_resize() {
        let (queue, _, _) = drive(&[b"a", b"b", b"c"], b"", unchanged);
        assert!(
            codes(&queue)
                .iter()
                .all(|event| !matches!(event, Event::Resize(..))),
            "three reads at one size are three keys and nothing else"
        );
    }

    /// A terminal that answers `0x0` is one the kernel has nothing to say about — mid-teardown, or
    /// a pty that has gone. Every buffer in the engine is sized from this pair and a zero-column
    /// screen is not a smaller screen, it is a different set of edge cases in every loop.
    #[test]
    fn a_zero_size_is_refused_rather_than_believed() {
        let (queue, _, size) = drive(&[b"a"], b"", gone);
        assert_eq!(size.get(), STARTED_AT);
        assert!(
            codes(&queue)
                .iter()
                .all(|event| !matches!(event, Event::Resize(..)))
        );
    }

    /// A read that parses to nothing — an unrecognised sequence — must not wake the app thread: a
    /// wake with an empty queue is a frame nobody asked for, and the idle guarantee is a
    /// count.
    #[test]
    fn a_read_that_yields_no_event_posts_no_wake() {
        let (queue, wakes, _) = drive(&[b"\x1b[99999q"], b"", unchanged);
        assert_eq!(codes(&queue).len(), 0);
        assert_eq!(
            wakes.pending(),
            crate::clock::QUIT,
            "nothing happened, so nothing was posted — and the channel then closed, which is not \
             nothing"
        );
    }

    /// **The terminal went away, and the application is told.**
    ///
    /// `Tty::open` hands a reader over only when standard input *and* standard output are both a
    /// terminal, so the channel closing is the pty's far side closing — the connection dropped, the
    /// window was closed, the multiplexer detached. Every write after that is discarded by
    /// `write_frame` and `present` goes on answering `submitted: true`, so without this the
    /// application parks in `Screen::wait` on a keyboard that cannot send another byte and nothing
    /// anywhere says so.
    #[test]
    fn the_channel_closing_is_a_quit_because_the_terminal_is_the_thing_that_closed_it() {
        let (_, wakes, _) = drive(&[], b"", unchanged);
        assert_eq!(
            wakes.pending() & crate::clock::QUIT,
            crate::clock::QUIT,
            "a reader whose channel is gone left the app thread parked with nothing coming"
        );
    }

    /// **And the negative twin, which is the half that makes the one above mean anything.** A loop
    /// that raised `QUIT` on every read would pass the test above and end every session at the
    /// first keystroke.
    #[test]
    fn a_read_that_arrives_while_the_terminal_is_still_there_is_not_a_quit() {
        let (tx, rx): (Sender<Vec<u8>>, _) = channel();
        let queue = Arc::new(Queue::new());
        let wakes = Arc::new(WakeSource::new());
        let size = Arc::new(TerminalSize::new(STARTED_AT));
        let (heard, told) = channel();
        let loop_queue = Arc::clone(&queue);
        let loop_wakes = Arc::clone(&wakes);
        let loop_size = Arc::clone(&size);
        // Spawned rather than driven, because the property is about a channel that is **open**, and
        // `run` does not return while one is.
        let thread = std::thread::spawn(move || {
            run(Wiring {
                reads: rx,
                type_ahead: Vec::new(),
                queue: loop_queue,
                wakes: loop_wakes,
                size: loop_size,
                paste_limit: 1 << 20,
                measure: unchanged,
            });
            let _ = heard.send(());
        });
        tx.send(b"a".to_vec()).expect("the receiver is alive");
        // The keystroke has to have been parsed before the flags are read, or this asserts about a
        // loop that has not run yet — which passes for the wrong reason. The queue is what says so.
        while queue.diagnostics().unrecognised() == 0 && codes(&queue).is_empty() {
            std::thread::yield_now();
        }
        assert_eq!(
            wakes.pending() & crate::clock::QUIT,
            0,
            "a terminal that is still connected does not end the session"
        );
        drop(tx);
        told.recv()
            .expect("the loop returns when the channel closes");
        thread.join().expect("the loop does not panic");
        assert_eq!(wakes.pending() & crate::clock::QUIT, crate::clock::QUIT);
    }

    #[test]
    fn the_diagnostics_reach_the_queue_where_the_app_thread_can_read_them() {
        let (queue, _, _) = drive(&[b"\x1b[99999q"], b"", unchanged);
        let diagnostics = queue.diagnostics();
        assert_eq!(diagnostics.unrecognised(), 1);
        assert_eq!(diagnostics.last_unrecognised(), Some(&b"\x1b[99999q"[..]));
    }
}
