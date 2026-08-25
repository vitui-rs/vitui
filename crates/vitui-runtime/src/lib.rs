//! Everything between the engine and a component: layout, identity, focus, hit-testing, routing,
//! key maps, theming, overlays and the data contract.
//!
//! There is **no scene tree** and no retained structure of any kind — the clip stack is the call
//! stack and the id path is the closure tree. What the runtime keeps for one draw and rebuilds on
//! the next is five flat structures; `CONTEXT.md` calls them the frame state. Reactivity is not
//! here either: it is an application's, and it is above this crate rather than inside it.
//!
//! Replaceable in principle — a different runtime should be able to sit on the same engine. That
//! claim is tested by the engine↔runtime seam ticket, not assumed.
//!
//! # Status
//!
//! Being built one ticket at a time from `.scratch/vitui-runtime-architecture/spec.md`, whose map is
//! closed; the backlog is `.scratch/vitui-runtime-impl/`. What exists so far:
//!
//! - [`data`] — `Revision`, `Versioned`, `Edit`, `Memo`. Spec §14, ADR 0019. **`std` only**: it
//!   reaches for neither the engine nor the frame, which is why it is first.
//! - [`layout`] — the constraint solver and the rect algebra. Spec §11. Pure functions over integer
//!   rectangles: no solver state, no allocation, and no floats anywhere.
//! - [`theme`] — `Paint`, `Role`, `Roles`, `Theme`, `Repaint`, `Glyph`, `Distinction`, `Density`.
//!   Spec §3 and §10; ADR 0018, 0021, 0010. A component names a role and can never construct a
//!   paint.
//! - [`theme::registry`] — `Scheme`, `Themes`, and the fourteen shipped palettes. Spec §15. The
//!   import is a `const fn`, so a theme is `.rodata`; the registry is **application state**, because
//!   a picker reads the set inside the frame while the loop writes it between frames.
//! - [`keys`] — `Chord`, `Binding`, `KeyMap`, sequences and help. Spec §9. Chords stored inline
//!   because `&'static [Chord]` cannot be written at a call site, and matching on the intent half of
//!   eight modifier bits.
//! - [`ctx`] — `Ctx<'f, 'v>`, `Frame`, `Env`, `Response`, `Interest`, `Driver`. Spec §1, §3, §6;
//!   ADR 0012. Five flat structures rebuilt from the draw, four id-keyed facts, and a `begin` that
//!   cannot be skipped.
//! - [`id`] — `Id`, `IdTable`, the id stack. Spec §5; ADR 0013. The call site is the source, FNV-1a
//!   with no finalizer, and three of the four id-keyed facts swept when a widget stops drawing.
//! - [`focus`] — `ScopeKind`, `Stop`, the ring and the vanish rule. Spec §8. A sixth interest bit,
//!   three scope answers as frame-local ranges, and the previous frame's ring as one more swapped
//!   buffer.
//! - [`route`] — `Edge`, `edge_of`, `batch_len`, and the one key queue behind `Ctx::next_key`. Spec
//!   §7; ADR 0016. **A frame consumes at most one routing edge**, there are no per-id inboxes, and
//!   bubbling is `Ctx::scope`'s after-the-body moment rather than a walk of the id path.
//! - [`scroll`] — `Scrollable`, `IntoView`, `Area`, `Wheel`. Spec §13; ADR 0015. **Two mechanisms
//!   that must never be conflated** — a scroll area costs the content and a virtualised collection
//!   costs the window — four direction bits rather than two axis bools, and the one sixteen-byte
//!   fact that crosses a frame.
//! - [`sizing`] — the sizing-function contract, `Ctx::measured` and the detector. Spec §12; ADR
//!   0014. **No trait and no type a component implements**: a sizing function is a shape, and the
//!   dry run survives only as the test that keeps one honest against the component beside it.
//! - [`anim`] — `Easing`, `Tween`, `Steps`, `Spring`, `WakeLedger`. Spec §16. **No animation
//!   object**: every helper is a closed form over `(now, start, duration)`, a `Tween` is 48 bytes of
//!   the component's own state, and the runtime holds nothing but the wake accounting — which is
//!   unconditional, because a detector armed only in a debug build never sees the application.
//! - [`overlay`] — `Z`, `Placement`, `place`, `Scrim`, `OverlayOpts` and the body queue. Spec §10;
//!   ADR 0017 and ADR 0034. **Request during the draw, satisfy after it, answer next frame**, with
//!   the owner id handed over rather than derived. A body is one `Box` in a queue the frame call
//!   owns; the bump region that held it with its type erased was this crate's only `unsafe`, and
//!   ticket 21 traded *the overlay frame allocates nothing* for the attribute below.
//! - [`work`] — `Slot` (the engine's, re-exported), `Drain`, `Task`, `Worker`, `Landing`, `Cancel`.
//!   Spec §17. **A worker is a noun, not a spawned future**: the runtime has no executor to lean on,
//!   so the handoff is a resident thread with a one-slot inbox and eight bytes of generation that
//!   say which question an answer answers.

// **No `unsafe` in any shipped crate above the engine** (ticket 21, ADR 0034). `forbid` and not
// `deny`, so nothing inside the crate can turn it back on with an `allow` — and it subsumes the
// `unsafe_op_in_unsafe_fn` this line used to carry, which only shaped `unsafe` that was allowed to
// exist. It is the whole gate: a compile outcome, with no test to write and no number to tune. The one
// place `unsafe` bought something is recorded where it was given up — `crate::overlay`, on the frame
// arena — and `vitui-alloc-probe` is the stated exemption, because `GlobalAlloc` cannot be
// implemented in safe Rust and it is `publish = false`.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

// `crate::screen` is `#[path]`-included by `tests/alloc.rs` and by both reports as well as compiled
// here, so the one realistic screen has exactly one definition. It is written against the public API
// — `use vitui_runtime::…` — and this alias is what makes that resolve inside the library too. The
// engine does the same for its scene list and gives the same second reason, which is the better one:
// **a fixture that could reach past the public surface would be measuring something a caller cannot
// do.**
extern crate self as vitui_runtime;

pub mod anim;
pub mod ctx;
pub mod data;
pub mod focus;
pub mod id;
pub mod keys;
pub mod layout;
pub mod overlay;
pub mod route;
pub mod scroll;
pub mod sizing;
pub mod theme;
pub mod work;

// **One realistic screen, shared by the reports and the allocation gates.** `#[path]`-included by its
// callers rather than exported, which is the engine's arrangement for its scene list and for the same
// reason: an instrument is not part of the library. Compiled under `cfg(test)` here so the crate's own
// suite checks its two counts; the examples include the file directly.
// `allow(dead_code)` rather than `expect`, and the same reason the engine's `budget.rs` gives for its
// own included modules: each *caller* uses a different part of this file — the crate's suite checks the
// counts, one example wraps `ROWS`, another only splits — and a lint that fires on the half you are not
// looking at teaches people to delete the other half.
#[cfg(test)]
#[allow(dead_code)]
#[path = "screen.rs"]
mod screen;

// **The headroom ledger, as an assertion.** Ticket 20. Every gated or reported number in this crate
// has exactly one home and this is it — most importantly the frame budget itself, which spec §19 says
// may not move without a new map decision and which was written out eleven times across `examples/`
// before this file existed. `#[cfg(test)]` for `screen`'s reason and the engine's; the examples
// `#[path]`-include it, which is how a report reads the budget it divides by.
#[cfg(test)]
#[allow(dead_code)]
#[path = "ledger.rs"]
mod ledger;

// **Spec §20's register, as a value.** Ticket 19. Every gate this crate ships, each naming the
// instruments that run it — and every instrument is checked against the source, which is the whole
// difference between a register and a document. `#[cfg(test)]` for `crate::line`'s reason and the
// engine's (`crates/vitui-engine/src/audit.rs`): an instrument is not part of the library.
#[cfg(test)]
mod register;

// **Spec §20's twenty scenes, as a normative list.** Ticket 19, and `#[cfg(test)]` on the same
// terms. A scene is removed only by a ticket naming the property it can no longer distinguish.
#[cfg(test)]
mod scenes;

// **The crate line, as a value.** Ticket 17's module map, the visibility count and the four manifest
// decisions that had been comments — `#[cfg(test)]` for `screen`'s reason and for the engine's
// (`crates/vitui-engine/src/audit.rs`): an instrument is not part of the library. The *build* behind
// the count is `crates/vitui-components/tests/crate_line.rs`, which is a crate that cannot name the
// engine.
#[cfg(test)]
mod line;

// Re-exported at the root as well as in the module, because the four are named constantly and
// `data::` in front of every one of them is noise at a call site. The module stays public: a reader
// looking for *why there is no trait* should land on its documentation, not on four scattered types.
pub use ctx::{Ctx, Driver, Interest, Response};
pub use data::{Edit, Memo, Revision, Versioned};
pub use focus::{ScopeKind, Stop};
pub use id::{Id, IdTable};
pub use keys::{ActionId, Binding, Chord, Chords, KeyMap, Match, MatchMode, On};
// The four an application names on the line that opens a menu. `place`, `Side` and `Align` stay in
// the module: a caller that needs them is already reading about placement.
pub use overlay::{OverlayOpts, Placement, Scrim, Z};
pub use scroll::{IntoView, Scrollable};
pub use theme::{Density, Distinction, Glyph, GlyphSet, Paint, Repaint, Role, Roles, Theme};
// Ticket 05's two: the shipped palette as data, and the set that holds which one is current. At the
// root for the same reason as the four above — an application names both on the line that starts it.
pub use theme::{Scheme, Themes};

// **The engine's vocabulary, re-exported so a crate that may not name `vitui_engine` can name it.**
//
// Runtime architecture issue 22. The rule is checkable and is gated in `crate::line` in both
// directions: *every engine type this crate's public surface names is reachable through this crate*,
// together with every type needed to **construct** one that the surface accepts. `Driver::post_mouse`
// takes a `Mouse`, so `Mouse`'s field types are here too — a name a consumer can write but not build
// is a barrier wearing a re-export's clothes, which is what `crates/vitui-components/src/listing.rs`
// found when it went looking for a wheel click and got as far as `Buttons`.
//
// **At the root and not in the modules**, which is where the five older ones sit (`theme::GlyphSet`,
// `theme::Link`, `work::Slot`, `work::Wake`, the four `keys` aliases). Those stay. The difference is
// that each of them is owned by the module it sits in, and none of these is: `Rect` is named by seven
// modules and no runtime module owns rectangles.
//
// **`Wheel` arrives aliased**, for the reason `keys` aliases four and `CONTEXT.md` states once: two
// types with one name across a seam is the failure being avoided. `vitui_runtime::scroll::Wheel` is a
// *configuration* — lines and columns per click — and the engine's is a notch direction. The runtime's
// own `scroll` tests already wrote `Wheel as Notch` before this line existed.
//
// `Restyle` and `Style` are here and are **not** named by the public surface: `Restyle` appears only
// where `theme`'s `pub const BOLD: u16` is defined, and `Style` only as `Paint`'s `pub(crate)` field.
// They are re-exported rather than struck from `crate::line::ENGINE_NAMES`, because deleting a row to
// make a gate come out even is the move this repository forbids by name — `route` is the precedent.
pub use vitui_engine::{
    AttachError, Button, Buttons, Capabilities, ColorDepth, Config, Cursor, CursorShape, Event,
    Mods, Mouse, MouseKind, MouseMode, Permit, Presented, Rect, Restyle, Rgb, Style,
    Wheel as Notch, Written,
};
