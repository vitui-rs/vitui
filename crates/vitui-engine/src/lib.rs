//! The vitui rendering engine: drawing calls in, terminal bytes out, as fast as the terminal will
//! take them.
//!
//! It owns cells, surfaces, layers, compositing, damage tracking and the frame writer. It stands
//! on its own — nothing above it is required, and it has nothing to call upward, so an application
//! that brings its own layout can depend on this crate alone.
//!
//! It does **not** lay anything out, does not know what a widget is, and never iterates your data.
//! You bring rectangles; it offers clipping and offset viewports, and culling stays yours. What
//! that buys is the invariant the whole design is arranged around:
//!
//! > **A frame costs what the visible cells cost, never what the data behind them costs.**
//!
//! A one-million-row list costs what a one-thousand-row list costs, because only the rows on
//! screen are ever written.
//!
//! # Examples
//!
//! A whole frame, headless. [`Clock::Manual`] runs the render pass inline on the calling thread
//! before `present` returns — same mailbox, same packet, same bytes as the threaded mode — so a
//! test or an example is a straight-line program:
//!
//! ```
//! use vitui_engine::{Clock, Color, Config, Engine, Output, Rect, Style};
//!
//! let engine = Engine::new(Config {
//!     clock: Clock::Manual,
//!     output: Output::Sink(Box::new(Vec::new())),
//!     size: (20, 3),
//!     ..Config::default()
//! });
//! let (mut screen, _wake) = engine.attach()?;
//!
//! let card = screen.layers().add_content(0, Rect::new(0, 0, 20, 3), true);
//! let mut view = screen.layers().view(card).expect("the layer was just added");
//! view.fill(Rect::new(0, 0, 20, 3), " ", Style::new().bg(Color::rgb(20, 20, 28)));
//! view.text(1, 1, "ready", Style::new().fg(Color::rgb(180, 220, 180)).bold());
//! drop(view);
//!
//! assert!(screen.present().submitted);
//!
//! // Damage is what makes the next frame cheap. The three verbs mark it as they write and
//! // `present` clears it, so a frame with nothing drawn into it sends nothing at all.
//! assert!(!screen.present().submitted);
//! # Ok::<(), vitui_engine::AttachError>(())
//! ```
//!
//! Drawing is three verbs on a [`View`], and a view is a rectangle you cannot escape: [`View::child`]
//! narrows it and [`View::scrolled`] moves the content under it, both by clipping rather than by
//! trusting the caller's arithmetic.
//!
//! ```
//! use vitui_engine::{Color, Rect, Style, Surface};
//!
//! // A surface can be drawn into with no terminal anywhere — which is also how a worker thread
//! // prepares one and donates it to the layer stack.
//! let mut surface = Surface::new(10, 4);
//! let mut view = surface.root();
//!
//! let written = view.text(0, 0, "hello", Style::new().fg(Color::indexed(4)));
//! assert_eq!(written.cells, 5);
//!
//! // A child clips: this row is two cells wide, so the text is cut and nothing lands outside.
//! let mut narrow = view.child(Rect::new(0, 1, 2, 1));
//! assert_eq!(narrow.text(0, 0, "hello", Style::new()).cells, 2);
//!
//! // And a clip is not a lie about coordinates: a write that starts left of the rectangle is
//! // discarded up to its edge, never shifted into it.
//! assert_eq!(narrow.text(-3, 0, "hello", Style::new()).cells, 2);
//! ```
//!
//! # The frame, as a sequence
//!
//! ```text
//! Engine::new(Config) -> attach() -> (Screen, WakeHandle)
//!   |                                 raw mode, the capability queries, the prologue -- and only
//!   |                                 then the render thread, where no concurrency existed yet
//!   |
//!   +- layers()                add layers; view(id) to draw into one
//!   |   +- the verbs           text - fill - restyle, marking damage as they write
//!   |
//!   +- set_mouse(level)        what the frame's components asked the pointer for, as a `max`
//!   +- set_cursor(caret)       where the caret goes, applied after the frame's last write
//!   |
//!   +- present() -> Presented  the app thread:
//!                                lease a packet, or fold this frame into the next one
//!                                composite the damaged rectangles bottom-up
//!                                pack runs and their cells into the packet
//!                                submit, clear damage, return
//!                              the render thread:
//!                                take the packet, serialise against the mirror,
//!                                one write into the sink, give the packet back
//! ```
//!
//! On `Clock::Manual` both halves run on the calling thread, in that order, before `present`
//! returns — same mailbox, same packet, same bytes. That is the deterministic mode and it is
//! public API rather than test scaffolding.
//!
//! **The engine does not own the loop.** It hands out the verbs and `present`, and the runtime
//! drives. Damage is marked by the verbs and cleared by `present`, and neither is reachable from
//! outside — which is also why nothing above the engine can force a full repaint.
//!
//! # What this crate will not do
//!
//! Each of these is a decision with a measurement behind it, and each is what keeps the invariant
//! above true:
//!
//! - **Lay anything out.** Callers bring rectangles. No layout concept enters through the surface
//!   API; that is what `vitui-runtime` is for.
//! - **Iterate your data.** Clipping and offset viewports are offered; culling is yours.
//! - **Let you read the screen back.** No cell, grapheme handle or style bit is readable from
//!   outside, which is why the oracle that checks the picture lives inside this crate.
//! - **Own the loop.** It hands out the verbs and `present`, and something above it drives. Damage
//!   is marked by the verbs and cleared by `present`, and neither is reachable from outside — so
//!   nothing above the engine can force a full repaint.
//! - **Run your work.** There is no executor and no thread pool: [`WakeHandle`] posts from
//!   anywhere and [`Slot`] carries one result back. A `thread::spawn` charged to the drawing thread
//!   costs 12.94 µs at p50, which is one and a half frames at 120 Hz.
//! - **Ask you for a trait.** The public surface has none, and the crate is `#![forbid(unsafe_code)]`.
//!
//! # Performance
//!
//! The budgets are gated in CI rather than aspired to: a full-screen 300x80 composite under 1 ms, a
//! typical damage-tracked frame under 100 µs, 60 fps steady state under 5% of a core, zero
//! allocations while a frame is composed, and zero wakeups when an application is genuinely idle —
//! thirty idle seconds cost `0.00 user, 0.00 sys` and no voluntary context switches, where a 120 Hz
//! ticker would have woken 3,600 times.
//!
//! Damage is marked at write time and never derived by diffing a buffer: a full-buffer diff of a
//! 300x80 screen costs about 170 µs, which is over the budget for a whole frame before anything is
//! drawn.
//!
//! # Status
//!
//! Implementation-complete, and there is no stability promise before 0.x. Minimum supported Rust
//! version 1.88.
//!
//! Green on Linux, macOS and Windows since 2026-09-10. Run against Ghostty, kitty, WezTerm,
//! Alacritty, iTerm2, Terminal.app and tmux through the conformance suite, all on macOS — **Windows
//! Terminal is the one supported terminal nobody has driven through it**, so what this crate
//! believes about it is inference, which is the one thing to know before depending on this crate
//! for a Windows application.

// Refusal 12, as a lint rather than as a claim. `forbid` and not `deny`, so that nothing inside the
// crate can turn it back on with an `allow` — and it subsumes the `unsafe_op_in_unsafe_fn` this line
// used to carry, which only shaped `unsafe` that was allowed to exist. The three places `unsafe`
// would have bought something are recorded where they were refused: `crate::view` on the
// column-band split, `crate::slot` on an `AtomicPtr`, and `crate::handoff` on the alternatives to
// one mailbox.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

// `crate::scenes` and `crate::register` are `#[path]`-included by `examples/budget.rs` as well as
// compiled here, so that the twelve scenes and the thirty-one register entries have exactly one
// definition. They are written against the public API — `use vitui_engine::…` — and this alias is
// what makes that resolve inside the library too. Spec the rule for the comparative suite is that
// a scene is defined by what the user sees rather than by what a framework does, and a scene that
// could reach past the public surface would be defined by what this framework does.
extern crate self as vitui_engine;

#[doc(hidden)]
pub mod refusals;

mod ucd;

mod actuate;
mod caps;
mod cell;
mod clock;
mod damage;
mod detect;
mod engine;
mod exts;
mod geom;
mod handoff;
mod input;
mod intern;
mod layer;
mod mix;
mod packet;
mod perf;
mod quant;
mod quirks;
mod reader;
mod serial;
mod shutdown;
mod tables;

// The scene list and the register. Both are instruments rather than parts of
// the engine, and neither is on the public surface: a caller cannot read back what is already on
// screen, so an oracle over cells lives inside the crate.
#[cfg(test)]
mod audit;
#[cfg(test)]
mod gates;
#[cfg(test)]
mod golden;
// **The ledger, and it is one table because the audit found the alternative.** Impl 26 went
// looking for a provenance comment beside every gate number and found three in the whole tree,
// against a hundred numbers — and the 1 ms / 100 us split written four times. See `ledger.rs`.
#[cfg(test)]
mod ledger;
#[cfg(test)]
mod register;
#[cfg(test)]
mod scenes;

// **The reference compositor, which stopped being `cfg(test)`** — as this file
// predicted it would. It is the oracle for the first fuzz target as well as for gate #1, and a
// fuzz target lives in another crate: `fuzz/` is its own workspace, because `cargo-fuzz` needs
// nightly and `libfuzzer-sys`. So the compositor is compiled whenever the tests are **or** the
// `fuzz` feature is on, and `crate::audit`'s `SOAK_ONLY_MODULES` is the list that keeps a module in
// that state from being silently exempt from the surface scans.
#[cfg(any(test, feature = "fuzz"))]
mod reference;

// **The fuzz door, and the only public module here but the prelude.** It is behind a non-default
// feature and hidden from rustdoc, so nothing an ordinary caller compiles can reach it and nothing
// on the documented surface names it. `crate::audit`'s
// `the_fuzz_door_is_behind_a_feature_and_hidden` asserts all three of those, because a second public
// module is a real widening and *the loophole is closed rather than enjoyed*.
//
// The harness is here rather than in `fuzz/fuzz_targets/` for one reason: the committed corpus
// replayed as an ordinary test is the gate, and the fuzzer is a soak over the same function. Two
// copies of the decoding would make the gate a check on a second oracle.
#[cfg(any(test, feature = "fuzz"))]
#[doc(hidden)]
pub mod fuzz;

// The round trip is the primary instrument, and the model it replays through is engine-internal:
// nothing in the public surface names it. The fuzz targets are what will need it
// outside `cfg(test)`, and that is when it moves.
// `crate::input`'s unit tests, declared here rather than inside `input.rs` — which is where they
// would go, and where they were. `tests/alloc.rs` `#[path]`-includes `input.rs` so that the
// keystroke path can be measured under the counting allocator, and a `mod tests` inside it would put
// all of them in that binary, running beside an allocation window. See
// `alloc::a_keystroke_allocates_nothing`, which says the same thing from the other end.
#[cfg(test)]
#[path = "input/tests.rs"]
mod input_tests;

#[cfg(test)]
mod term_model;
#[cfg(test)]
mod testing;

mod restyle;
mod slot;
mod style;
mod surface;
mod sweep;
mod text;
mod view;

#[cfg(test)]
mod roundtrip;

/// The nine names an application writes at the top of a file.
///
/// The list is stated in two places, which is how a prelude stays a prelude:
/// nine names is a glance, and the moment it is twenty it is the crate's re-export list with an
/// extra path segment in front of it. `crate::audit`'s
/// `the_prelude_re_exports_exactly_nine_names` is what keeps it at nine.
///
/// Everything else is reached by its own path. The input types are not here on purpose — a runtime
/// matching on [`Event`] names it once, at the top of one function, and a component
/// author names none of these at all.
pub mod prelude {
    pub use crate::{Color, Config, Engine, LayerId, Rect, Screen, Style, View, Wake};
}

pub use actuate::{Cursor, CursorShape};
pub use caps::{Capabilities, ColorDepth, GlyphSet, Overrides, Rgb, WidthSource};
pub use clock::Wake;
pub use engine::{AttachError, Clock, Config, Engine, Output, Presented, Screen, WakeHandle};
pub use geom::Rect;
pub use input::{
    Button, Buttons, Event, InputConfig, InputDiagnostics, Key, KeyCode, KeyKind, KeyText, Keypad,
    Media, Modifier, Mods, Mouse, MouseKind, MouseMode, Paste, Wheel,
};
pub use layer::{LayerId, LayerStack};
pub use mix::Mix;
pub use perf::Permit;
pub use restyle::{Link, Restyle};
pub use slot::Slot;
pub use style::{Color, Style};
pub use surface::Surface;
pub use text::{graphemes, width_of};
pub use view::{Stop, View, Written};
