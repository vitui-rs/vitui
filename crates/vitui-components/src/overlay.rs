//! **F9 overlays**, ~35 entries, expressed by `overlay`, layers, placement, scopes and `Trap`.
//!
//! The reduction is R1 and R2 (spec §18): modality is a `bool`, and the eighteen named popup,
//! dialog, drawer, sheet and toast entries are two axes of one component.
//!
//! `select` declares this family and is homed under F6, because §12's finding is that the popup is
//! the *owner's* — `SelectState` is written only by the owner and `PopupState` only by the body,
//! and `&'f mut` is what makes *request the overlay last* a borrow error rather than a comment.

//! # One diagnostic path is a trap, and the sentence that closes it is owed on a runtime item
//!
//! Spec §1 states it as an obligation with an owner:
//!
//! > On an overlay body that captures its owner's state by `&mut`, rustc's own `help:` line — *add
//! > explicit lifetime `'f` to the type of `st`* — **compiles**. What then fails is the caller, with
//! > `E0503: cannot use st.open because it was mutably borrowed`, naming the caller's own read one
//! > level away from the mistake and never mentioning the overlay. `Ctx::overlay`'s documentation
//! > owes: *the body answers through the inbox; a `&'f mut` capture compiles and costs you the state
//! > for the rest of the frame.*
//!
//! **Components ticket 10 was to write it if runtime ticket 13 had landed. It has** —
//! `Ctx::overlay` exists at `crates/vitui-runtime/src/ctx.rs` — **and the note is not there**, so
//! the sentence is still owed, on that item and in that file. Ticket 10 may not write it: it does
//! not touch `crates/vitui-runtime/`, and a note about `Ctx::overlay` written anywhere else is a
//! note nobody hits, because the diagnostic that sends a reader looking arrives at the **caller**
//! and names `E0503` on a field read.
//!
//! It is recorded here rather than dropped: this module is F9, `Ctx::overlay` is what every
//! component in it will be built on, and components ticket 26 — `select` and the overlay family —
//! is the next ticket that has to read this file. **The note belongs on
//! `vitui_runtime::ctx::Ctx::overlay`, and it needs a runtime-side change to put it there.**

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &["overlay"];
