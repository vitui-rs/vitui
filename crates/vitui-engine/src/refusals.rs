//! **The refusals with no type to hang a doctest on.**
//!
//! Internal, and hidden from rustdoc: this module is a corpus of negative cases, not a thing
//! a caller uses. It is a module rather than a test file because a `compile_fail` fence only
//! runs where rustdoc collects doctests, and a hidden `pub` item is still collected.
//!
//! Most of what this crate refuses is gated on the type that would have carried the refused
//! item. The cases below have no such type, because the whole of the claim is that a **name**
//! is absent.
//!
//! Nine of §12's twelve refusals are gated on the type that would have carried the refused item —
//! [`Screen`](crate::Screen) for reactivity and the display query, [`View`](crate::View) for layout, widgets, a clock and a
//! scheduler, [`LayerStack`](crate::LayerStack) for alpha and the flattened cache, [`Slot`](crate::Slot) for blocking and
//! completion, and `lib.rs`'s own `#![forbid(unsafe_code)]` for the twelfth. Three have no such
//! type, because the whole of the claim is that a **name** is absent, so they are here.
//!
//! Each is a pair, and the pair is the unit: **deleting the hostile line is caught by the first half
//! and renaming the item it protects is caught only by the second.** A rename turns a lone
//! `compile_fail` from `E0599` into `E0433`, which the mechanism cannot distinguish and reports as
//! `ok` — so every twin below names the item it protects **by path**. The error code on a fence is
//! documentation for the reader and not an assertion: `compile_fail,E0308` on a snippet whose real
//! error is `E0277` passes on stable.
//!
//! **Refusal 5 — no trait, zero of them.** The engine has nothing to call upward, so the dependency
//! arrow is enforced by there being no arrow. `dyn Painter` stays a negative result, and
//! `crate::audit`'s `there_are_no_public_traits` is the count beside this case.
//!
//! ```compile_fail,E0405
//! struct Mine;
//! impl vitui_engine::Painter for Mine {}
//! ```
//!
//! **Refusal 8 — no executor and no thread pool.** What is offered instead is [`WakeHandle`](crate::WakeHandle) and
//! [`Slot`](crate::Slot), and the numbers that stop anybody reaching for a pool are beside `Slot` itself: a
//! `thread::spawn` charged to the calling thread is 12.94 µs at p50 on the CI runner, which is one
//! and a half frames at 120 Hz.
//!
//! ```compile_fail,E0433
//! let _ = vitui_engine::ThreadPool::new(4);
//! ```
//!
//! **Refusal 11 — no cells, no grapheme handles, no style bits**. A caller cannot read
//! back what is on screen, which is why the oracle over cells lives inside this crate — and since
//! architecture ticket 21 it holds with **no exception beside it**. There *was* one: `LinkId`, the
//! one handle that was public, opaque and minted by `Screen::link`. The URI travels at the drawing
//! verb now ([`Link`](crate::Link)), so the handle is internal and the exception is unnecessary rather than
//! rewritten.
//!
//! ```compile_fail,E0433
//! let _ = vitui_engine::Cell::default();
//! ```
//!
//! ```compile_fail,E0433
//! let _: vitui_engine::LinkId = Default::default();
//! ```
//!
//! And the mint that went with it, which is the half a name check cannot see — a method is not a
//! re-export:
//!
//! ```compile_fail,E0599
//! let config = vitui_engine::Config {
//!     output: vitui_engine::Output::Sink(Box::new(Vec::new())),
//!     ..Default::default()
//! };
//! let (mut screen, _wake) = vitui_engine::Engine::new(config).attach().unwrap();
//! let _ = screen.link("https://example.com/");
//! ```
//!
//! And the two names §12 lists or prices that are not here either — `Resolver`, which appears once
//! in the whole architecture and never again, and the serializer `Options` type, priced rather than
//! overlooked in §8:
//!
//! ```compile_fail,E0433
//! let _: vitui_engine::Resolver = Default::default();
//! ```
//!
//! ```compile_fail,E0433
//! let _ = vitui_engine::Options::default();
//! ```
//!
//! The twin for all seven, naming by path what stands in each one's place — the drawing verb a
//! `Painter` would have been called from, the slot a pool would have fed, the write that replaces
//! reading a cell, the descriptor field and the URI that replace the handle and its mint, the
//! capabilities an operator's colour is resolved against, and the config that carries every
//! serializer axis there is:
//!
//! ```
//! use vitui_engine::{
//!     Capabilities, Config, Engine, Link, Output, Rect, Restyle, Screen, Slot, Style, Surface, View,
//! };
//!
//! let mut surface = Surface::new(4, 1);
//! let mut view = surface.root();
//! assert_eq!(View::set(&mut view, 0, 0, "x", Style::new()).cells, 1);
//! let hyperlink: Restyle<'_> = Restyle {
//!     link: Some(Link::Uri("https://example.com/")),
//!     ..Default::default()
//! };
//! View::restyle(&mut view, Rect::new(0, 0, 1, 1), &hyperlink);
//! assert_eq!(hyperlink.link, Some(Link::Uri("https://example.com/")));
//! let slot: Slot<u32> = Slot::new();
//! assert_eq!(Slot::take(&slot), None);
//! let (screen, _wake) = Engine::new(Config {
//!     output: Output::Sink(Box::new(Vec::new())),
//!     ..Default::default()
//! })
//! .attach()
//! .unwrap();
//! let caps: &Capabilities = Screen::capabilities(&screen);
//! assert!(!Capabilities::report(caps).is_empty());
//! let _ = Rect::new(0, 0, 1, 1);
//! ```
//!
