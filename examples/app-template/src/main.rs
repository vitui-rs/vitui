//! A vitui application that does everything spec §11 asks of an app thread, and nothing else.
//!
//! It is a **template rather than a demo**: there is no interesting UI here, because what it exists
//! to show is the shape of the loop and the shape of getting work off it. Copy this directory,
//! `clippy.toml` included, and start replacing the `draw` function.
//!
//! ```text
//!   wait()            park until something happens — the app thread's ONLY blocking call
//!     |
//!   next_event()      drain what the terminal reported
//!     |
//!   slot.take()       pick up whatever a worker left; never wait for one
//!     |
//!   draw              layers, views, verbs
//!     |
//!   present()         hand the frame on and go round again
//! ```
//!
//! # The two rules this file is here to demonstrate
//!
//! **Work that is not drawing happens somewhere else and posts.** A worker computes, leaves the
//! result in a [`Slot`], and calls [`WakeHandle::post`]. The app thread learns about it as a
//! [`Wake::Posted`] and takes it with a non-blocking [`Slot::take`] — never as a callback on the
//! worker's own thread, which is what keeps everything above the engine single-threaded by
//! construction.
//!
//! **A region that is going to be slow says so.** [`Screen::permit_slow`] is a declaration, not a
//! switch: it excuses the in-loop detector for what it covers and its reason is printed by whichever
//! diagnostic fires. A debug build panics on the *first* unexcused overrun, which is why the
//! cold-start frame below holds one.
//!
//! # Run it
//!
//! `cargo run` and press a key; `q` or Ctrl-C to leave. `cargo clippy` is the half that matters:
//! `clippy.toml` beside this file is spec §11's lint rung, and `README.md` says what it does and
//! does not protect.

// **The line the fragment needs to do anything.** `clippy.toml` supplies the list; this switches the
// lint on. Without it the file is inert, which is worth knowing before wondering why nothing fires.
#![warn(clippy::disallowed_methods)]

use std::sync::Arc;

use vitui_engine::{
    Clock, Color, Config, Engine, Event, KeyCode, Rect, Screen, Slot, Style, Wake, WakeHandle,
};

/// What the worker in this template computes. Anything at all; the point is where it happens.
struct Summary {
    lines: usize,
}

fn main() {
    // Nothing has been touched yet, and `Engine` is `Send`, so this may be built anywhere and
    // `attach`ed on whichever thread is going to be the app thread.
    let engine = Engine::new(Config {
        clock: Clock::System,
        max_frame_rate: 60.0,
        // Where a release build's one warning about a dropped frame goes. `None` is silence, and
        // silence is the default because a full-screen application's stderr is the terminal it is
        // drawing on. A real application points this at its log.
        overrun_report: Some(Box::new(std::io::sink())),
        ..Default::default()
    });
    let (mut screen, wake) = match engine.attach() {
        Ok(attached) => attached,
        Err(why) => {
            eprintln!("vitui could not attach: {why}");
            return;
        }
    };

    // The one place a background result lands. `Slot` is `Send + Sync`, so an `Arc` of one goes into
    // as many workers as there are kinds of result.
    let summary: Arc<Slot<Summary>> = Arc::new(Slot::new());

    // **The cold-start frame, declared.** Building the first screen is legitimately slower than a
    // frame budget — nothing is warm, no surface is allocated — and a debug build panics on the first
    // unexcused overrun. The permit is the declaration that this one is expected, and its reason is
    // what a stall report would print.
    {
        let permit = screen.permit_slow("the cold-start frame");
        let (w, h) = screen.size();
        screen.layers().add_content(0, Rect::new(0, 0, w, h), true);
        start_the_work(&summary, &wake);
        draw(&mut screen, None);
        screen.present();
        drop(permit);
    }

    let mut latest: Option<Summary> = None;
    loop {
        // The app thread's only blocking call, and the frame clock gates it: the first damage after a
        // quiet period returns at once, everything inside the gap coalesces into one return at the
        // end of it, and an idle application costs no wakeups at all.
        match screen.wait() {
            Wake::Quit => break,
            Wake::Input | Wake::Posted | Wake::Deadline => {}
        }

        // One wake can carry a hundred events, so this drains rather than returning one.
        while let Some(event) = screen.next_event() {
            match event {
                Event::Key(key) if key.code == KeyCode::Char('q') => return,
                Event::Key(key) if key.code == KeyCode::Char('c') && key.mods.ctrl() => return,
                Event::Resize(..) => {}
                _ => {}
            }
        }

        // Non-blocking, and the check *is* the take: a predicate followed by a take is two reads with
        // a worker running between them.
        if let Some(fresh) = summary.take() {
            latest = Some(fresh);
        }

        draw(&mut screen, latest.as_ref());
        screen.present();
    }
}

/// Everything that is not drawing, on a thread that is not this one.
///
/// `thread::spawn` costs the calling thread about 6-10 µs at p50 and tens of microseconds at p99,
/// which is nothing once per keystroke and a tenth of a frame budget once per frame. A long-lived
/// worker reading from a channel is about 0.5 µs a message. Neither number is a reason to build a
/// pool: a pool is an executor under another name, and the engine refuses one.
///
/// Note what is **not** here: no handle is kept and nothing is joined. The worker's only way back is
/// the slot and the post.
fn start_the_work(summary: &Arc<Slot<Summary>>, wake: &WakeHandle) {
    let summary = Arc::clone(summary);
    let wake = wake.clone();
    std::thread::spawn(move || {
        // Read a file, call a service, walk a tree — whatever this application is for. Here, an
        // arithmetic stand-in, so that the template has no I/O in it to be flagged.
        let lines = (1..=10_000u64).filter(|n| n % 7 == 0).count();
        summary.put(Summary { lines });
        // And *then* the wake, so the app thread never sees a post with nothing behind it.
        wake.post();
    });
}

/// The frame. Coordinates and `&str` go in; the engine holds nothing of yours.
fn draw(screen: &mut Screen, summary: Option<&Summary>) {
    let (w, h) = screen.size();
    let layer = match screen.layers().is_empty() {
        true => screen.layers().add_content(0, Rect::new(0, 0, w, h), true),
        false => screen
            .layers()
            .topmost_at(0, 0)
            .expect("the one layer covers the screen"),
    };
    let text = match summary {
        Some(summary) => format!("{} lines, computed on a worker", summary.lines),
        None => String::from("working..."),
    };
    let style = Style::new().fg(Color::rgb(0xd0, 0xd0, 0xd0));
    let Some(mut view) = screen.layers().view(layer) else {
        return;
    };
    view.fill(Rect::new(0, 0, w, h), " ", style);
    view.text(1, 1, &text, style);
    view.text(1, 3, "q to quit", style);
}
