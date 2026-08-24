//! Key maps: chords stored inline, matching on **intent**, and a capability the runtime reads and
//! never acts on.
//!
//! Spec §9; ADR 0021 (a declared thing is not rewritten), ADR 0010 (a detected axis is not a ladder).
//!
//! ```
//! use vitui_engine::{Key, KeyCode, KeyKind, KeyText, Mods};
//! use vitui_runtime::keys::{Chord, KeyMap};
//! # use std::time::Instant;
//!
//! const SAVE: u32 = 1;
//! let map = KeyMap::new().bind(&[Chord::key('s').ctrl()], SAVE, "Save");
//!
//! // Caps lock is keyboard *state*, so it does not disarm a binding.
//! let pressed = Key {
//!     code: KeyCode::Char('s'),
//!     mods: Mods::CTRL.with(Mods::CAPS),
//!     kind: KeyKind::Press,
//!     text: KeyText::EMPTY,
//!     at: Instant::now(),
//! };
//! assert_eq!(map.match_first(&pressed), Some(SAVE));
//! ```
//!
//! # Chords are stored inline, and that is `E0716`'s doing
//!
//! Spec §12 sketched `keys: &'static [Chord]`. **It cannot be written at a call site**: rvalue
//! promotion does not cover a `const fn` call, so `&[Chord::key('s').ctrl()]` is
//! *temporary value dropped while borrowed* and every binding list would need its own named `const`.
//! That is a requirement-10 failure in the one demand that exists **because** bindings are declared
//! once and read twice — see [`Binding`], which carries the paired compile outcome.
//!
//! Inline costs **64 bytes against 40** and buys a map that can be built in one expression, including
//! from a config file.
//!
//! # Two objects, two lifetimes
//!
//! Writing the copy showed that **matching needs the chords and the action and never the help.** So
//! there are two types: [`Binding`], which an application authors and a help bar reads, and [`Match`],
//! which is what a frame keeps — **44 bytes against 64**, copied into a [`Matches`] buffer that is
//! cleared and never freed.
//!
//! The reason it must be a copy rather than a borrow is ownership: a frame outlives every draw, and a
//! modal's key map is a local, so a frame holding `&KeyMap` cannot exist.
//!
//! # "First match wins" is not an equality
//!
//! Read as one, a 33-binding map fires **33 of 33 normally and 0 of 33 the moment caps lock is on** —
//! every binding disarmed at once, with nothing on screen to say why. Six of the engine's eight
//! modifier bits are *intent* and two are *state*, and matching consults the intent half. See
//! [`INTENT`].
//!
//! # The base layout is a capability, and this module reads it and does nothing
//!
//! Below kitty flag 4 the engine's `Key::code` is *inferred from* `Key::text`, so a binding on a
//! keycap that prints something else simply never fires. On a Cyrillic layout a 33-binding map reaches
//! **33 / 28 / 14 of 33** across the three cases, with **0 wrong** at every one of them: the failure is
//! **silence, not misfire**, and a US-layout test suite finds none of it.
//!
//! **Matching never consults the terminal's keyboard at all**, and [`base_layout_reported`] is the
//! whole of what is readable — one boolean, not a tier. Two reasons, and the second is the sharper:
//! a binding is not rewritten because the terminal is poor, any more than a declared interest is
//! (ADR 0021); and **which legacy terminal we are in is unobservable**, both cases being silence on
//! the wire, so a ladder's middle rungs cannot be told apart. ADR 0010 refuses `KeyboardTier` by name
//! and this is why.

// The thirty-three-binding corpus and the synthetic terminal that presses it. `#[path]`-included by
// the report as well as compiled here, which is `crate::screen`'s arrangement and for the same
// reason: an instrument is not part of the library.
#[cfg(test)]
#[allow(dead_code)]
#[path = "keys/corpus.rs"]
pub mod corpus;

use std::time::{Duration, Instant};

use vitui_engine::{Key, KeyCode, KeyKind, Mods};

pub use vitui_engine::{Key as Pressed, KeyCode as Code, KeyKind as Edge, KeyText as Text};

/// What a binding fires.
///
/// **A plain alias, which is the only definition any document gives it** — and it composes with
/// nothing, which is worth knowing before it bites: two subsystems both using small integers collide
/// silently, and `ActionId` will not stop them. A `#[repr(transparent)]` newtype would cost nothing
/// and make that collision a type error. **Filed rather than taken**: the alias is what the spec
/// wrote, and changing a public type is a decision this backlog does not get to make on its own.
pub type ActionId = u32;

/// The six modifier bits that are **intent**, as one named constant.
///
/// `SHIFT`, `ALT`, `CTRL`, `SUPER`, `HYPER`, `META`. The two that are missing are `CAPS` and `NUM`,
/// which are keyboard *state*: a binding on `Ctrl+C` must still fire with caps lock on.
///
/// The engine's own projection is [`Mods::chord`] — *"the same modifiers with the two locks removed,
/// which is what a key binding compares"*, in its own words — and **spec §9 calls that method
/// `Mods::intent()`.** There is no `intent` in the engine and `Mods` is a foreign type this crate
/// cannot add an inherent method to, so this module calls `chord()` and names the mask here rather
/// than wrapping it in an extension trait. Two names for one mask is worse than one name in the wrong
/// crate.
pub const INTENT: Mods = Mods::SHIFT
    .with(Mods::ALT)
    .with(Mods::CTRL)
    .with(Mods::SUPER)
    .with(Mods::HYPER)
    .with(Mods::META);

/// Whether the terminal reports the key the physical layout would have produced.
///
/// A projection of `Capabilities::alternate_keys`, which is literally kitty flag 4 — **one boolean,
/// and deliberately not a tier.** ADR 0010 refuses `KeyboardTier` by name, and this module's own
/// measurement is why: below flag 4 there are two cases, a terminal that falls back to Latin and one
/// that sends nothing, and **they are indistinguishable from inside the process** because both are
/// silence on the wire. A ladder whose middle rungs cannot be told apart is not a ladder.
///
/// A free function rather than a field, because `Caps` is the engine's `Capabilities` re-exported and
/// a foreign struct cannot grow one.
///
/// **Nothing in this module calls it.** It is here so an application can put a line in its own help —
/// *your terminal does not report the physical layout, so letter shortcuts may not work* — which is
/// the only useful thing anyone can do with the answer.
pub const fn base_layout_reported(caps: &vitui_engine::Capabilities) -> bool {
    caps.alternate_keys
}

/// Which half of a key a chord is about.
///
/// **The author's call, never the runtime's.** A shortcut is normally about *where the key is*, so
/// that `Ctrl+S` is the same physical key on every layout; a binding on a character the user
/// deliberately typed is about *what it produced*.
///
/// Not in the ticket's checklist — an addition on the architecture answer's authority, and the thing
/// that makes a bare-letter binding two different bindings rather than one ambiguous one.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum On {
    /// Where the key is. Compares [`Key::code`].
    #[default]
    BaseLayout,
    /// What the key produced. Compares [`Key::text`].
    Typed,
}

/// How strictly modifiers are compared. Exists so that *first match wins is not an equality* is a
/// runnable gate rather than an assertion.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum MatchMode {
    /// Compare the intent half. **This is what ships.**
    #[default]
    Masked,
    /// Compare all eight bits, locks included. Kept only so that
    /// `tests::an_equality_match_loses_every_binding_when_caps_lock_is_on` can run.
    Equality,
}

/// One chord: a key, some modifiers, and which half of the key it is about.
///
/// `const`-constructible through the builders, which is what lets a whole map be one expression.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Chord {
    /// Which key.
    pub code: KeyCode,
    /// Which modifiers. Only the [`INTENT`] half is ever compared.
    pub mods: Mods,
    /// Position or product.
    pub on: On,
}

impl Chord {
    /// A chord on a named key, no modifiers.
    pub const fn new(code: KeyCode) -> Chord {
        Chord {
            code,
            mods: Mods::NONE,
            on: On::BaseLayout,
        }
    }

    /// A chord on where a character's key is.
    pub const fn key(c: char) -> Chord {
        Chord::new(KeyCode::Char(c))
    }

    /// A chord on a character the user actually typed.
    pub const fn typed(c: char) -> Chord {
        Chord {
            code: KeyCode::Char(c),
            mods: Mods::NONE,
            on: On::Typed,
        }
    }

    /// Hold control.
    pub const fn ctrl(mut self) -> Chord {
        self.mods = self.mods.with(Mods::CTRL);
        self
    }

    /// Hold alt.
    pub const fn alt(mut self) -> Chord {
        self.mods = self.mods.with(Mods::ALT);
        self
    }

    /// Hold shift.
    pub const fn shift(mut self) -> Chord {
        self.mods = self.mods.with(Mods::SHIFT);
        self
    }

    /// Whether a key press is this chord.
    ///
    /// # Three refusals, in one function
    ///
    /// **A release never matches.** At kitty flag 2 a terminal reports both edges, and a map that
    /// matched both would fire every binding twice.
    ///
    /// **A repeat does match**, and that is a decision rather than an oversight. Below the kitty
    /// protocol a held key arrives as a stream of presses, so *matching repeats is what makes a held
    /// key behave the same at both tiers* — and `j`/`k` navigation is the common case. The cost is
    /// stated where an author will meet it: a held `Ctrl+S` fires repeatedly, and debouncing an
    /// action that should not repeat is the application's, because only the application knows which
    /// those are.
    ///
    /// **Locks are masked out**, through the engine's [`Mods::chord`] — see [`INTENT`].
    pub fn matches(self, k: &Key, mode: MatchMode) -> bool {
        if k.kind == KeyKind::Release {
            return false;
        }
        let mods_ok = match mode {
            MatchMode::Masked => k.mods.chord() == self.mods.chord(),
            MatchMode::Equality => k.mods == self.mods,
        };
        if !mods_ok {
            return false;
        }
        match self.on {
            On::BaseLayout => k.code == self.code,
            On::Typed => match self.code {
                // `KeyText` has no public constructor, so the comparison is against a `&str` this
                // module encodes itself. It is also why an overflowed cluster can never match: the
                // engine reports it as empty, and empty is not a character.
                KeyCode::Char(c) => {
                    let mut buf = [0u8; 4];
                    !k.text.is_empty() && k.text.as_str() == c.encode_utf8(&mut buf)
                }
                other => k.code == other,
            },
        }
    }
}

/// How many alternatives one binding may name, and how long a sequence may be.
///
/// Three covers the survey — one or two in almost every case, three at the widest. **Beyond three it
/// truncates rather than failing** (ADR 0022's clamp-and-discard): a binding with four alternatives
/// is a design smell, not an error, and a runtime that refused one would be turning a smell into an
/// outage on a config reload.
pub const MAX_ALTS: usize = 3;

/// Up to [`MAX_ALTS`] chords, inline.
///
/// Read as **alternatives** by [`Match`] and as a **sequence** by [`SeqBinding`] — the same storage,
/// two readings, which is why `g g` needs no second container.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Chords {
    n: u8,
    a: [Chord; MAX_ALTS],
}

impl Chords {
    /// Take up to [`MAX_ALTS`] chords, truncating past that.
    pub const fn of(src: &[Chord]) -> Chords {
        let filler = Chord::new(KeyCode::Escape);
        let mut a = [filler; MAX_ALTS];
        let n = if src.len() > MAX_ALTS {
            MAX_ALTS
        } else {
            src.len()
        };
        let mut i = 0;
        while i < n {
            a[i] = src[i];
            i += 1;
        }
        Chords { n: n as u8, a }
    }

    /// One chord.
    pub const fn one(c: Chord) -> Chords {
        Chords::of(&[c])
    }

    /// The chords, in declaration order.
    pub fn as_slice(&self) -> &[Chord] {
        &self.a[..usize::from(self.n)]
    }
}

/// **What a frame keeps, and it is deliberately less than what a help bar reads.**
///
/// Forty-four bytes against [`Binding`]'s sixty-four. The difference is the help string, which never
/// routes anything.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Match {
    /// The alternatives, any one of which fires.
    pub keys: Chords,
    /// What fires.
    pub action: ActionId,
}

impl Match {
    /// Whether any alternative is this key.
    pub fn matches(&self, k: &Key, mode: MatchMode) -> bool {
        self.keys.as_slice().iter().any(|c| c.matches(k, mode))
    }
}

/// A binding as an application authors it: chords, an action, and help text.
///
/// # The compile outcome that put the chords inline
///
/// Spec §12's shape was `keys: &'static [Chord]`, and it cannot be written at a call site:
///
/// ```compile_fail,E0716
/// use vitui_runtime::keys::Chord;
/// struct StaticBinding { keys: &'static [Chord], action: u32 }
/// let _ = StaticBinding { keys: &[Chord::key('s').ctrl()], action: 1 };
/// ```
///
/// Rvalue promotion does not cover a `const fn` call, so the slice is a temporary. Every binding list
/// would need a named `const` beside it — in the one demand that exists *because* bindings are
/// declared once and read twice.
///
/// The twin, which is the shape that ships and is one expression:
///
/// ```
/// use vitui_runtime::keys::{Binding, Chord, KeyMap};
///
/// const SAVE: u32 = 1;
/// const QUIT: u32 = 2;
/// let map = KeyMap::new()
///     .bind(&[Chord::key('s').ctrl()], SAVE, "Save")
///     .bind(&[Chord::key('q').ctrl(), Chord::key('c').ctrl()], QUIT, "Quit");
/// assert_eq!(map.len(), 2);
/// assert_eq!(size_of::<Binding>(), 64);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Binding {
    /// The half that routes.
    pub m: Match,
    /// The half that does not. Read by a help bar and copied into no frame.
    pub help: &'static str,
}

impl Binding {
    /// Author a binding.
    pub const fn new(keys: &[Chord], action: ActionId, help: &'static str) -> Binding {
        Binding {
            m: Match {
                keys: Chords::of(keys),
                action,
            },
            help,
        }
    }

    /// Its chords.
    pub fn keys(&self) -> &[Chord] {
        self.m.keys.as_slice()
    }

    /// What it fires.
    pub const fn action(&self) -> ActionId {
        self.m.action
    }

    /// Whether any of its chords is this key.
    pub fn matches(&self, k: &Key, mode: MatchMode) -> bool {
        self.m.matches(k, mode)
    }
}

/// A multi-chord binding: `g g`, `Ctrl+X Ctrl+S`.
#[derive(Clone, Copy, Debug)]
pub struct SeqBinding {
    /// The chords, **in order**. Same storage as an alternative list, read differently.
    pub seq: Chords,
    /// What fires when the last one lands.
    pub action: ActionId,
    /// Help text.
    pub help: &'static str,
}

/// An unresolved prefix.
///
/// **Map-owned rather than id-keyed**, which is the whole reason it needs no sweep: a caller holds
/// this beside its [`KeyMap`], [`KeyMap::step`] takes it by `&mut` and the map by `&`, and nothing is
/// keyed by an identity. It is the only key-map state that crosses a frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Pending {
    /// Which sequence, if any, is part-matched.
    seq: Option<usize>,
    /// How many of its chords have landed.
    at: usize,
    /// When the prefix started standing, for the timeout.
    since: Option<Instant>,
}

impl Pending {
    /// Nothing is pending.
    pub const fn none() -> Pending {
        Pending {
            seq: None,
            at: 0,
            since: None,
        }
    }

    /// Whether a prefix is standing.
    pub const fn is_standing(&self) -> bool {
        self.seq.is_some()
    }

    /// **The one wakeup a standing prefix owes.**
    ///
    /// A screen with a half-typed `g` would otherwise block for ever waiting for input that decides
    /// nothing. This is one deadline per unresolved prefix — not one per frame and not one per key —
    /// discharged when the prefix resolves or expires. An application arms its own timer from it; the
    /// runtime owns no timer.
    pub fn deadline(&self, timeout: Duration) -> Option<Instant> {
        self.since.map(|since| since + timeout)
    }
}

/// What one key did to a sequence.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SeqStep {
    /// It is not part of any sequence.
    Miss,
    /// It advanced a prefix, which is now standing.
    Held,
    /// It completed a sequence.
    Fired(ActionId),
}

/// An application's key map. Ordered, and **first match wins**.
#[derive(Clone, Debug, Default)]
pub struct KeyMap {
    /// The flat bindings, in declaration order.
    pub bindings: Vec<Binding>,
    /// The sequences.
    pub seqs: Vec<SeqBinding>,
    /// How modifiers are compared. [`MatchMode::Masked`] unless a test says otherwise.
    pub mode: MatchMode,
}

impl KeyMap {
    /// How long an unresolved prefix stands, by default.
    ///
    /// **One second, matching vim's `timeoutlen`** — the value the largest existing corpus of muscle
    /// memory was trained against. It is the difference between `g g` being usable and `g` feeling
    /// laggy.
    ///
    /// **A parameter, never a stored setting.** It is passed to the calls that advance and expire a
    /// prefix, so an application can use a different one per map or per mode without the runtime
    /// holding a mutable knob.
    pub const TIMEOUT: Duration = Duration::from_millis(1000);

    /// An empty map, comparing intent.
    pub fn new() -> KeyMap {
        KeyMap {
            bindings: Vec::new(),
            seqs: Vec::new(),
            mode: MatchMode::Masked,
        }
    }

    /// Compare modifiers differently. For gates; an application has no reason to.
    #[must_use]
    pub fn with_mode(mut self, mode: MatchMode) -> KeyMap {
        self.mode = mode;
        self
    }

    /// Add a binding. Declaration order is match order.
    #[must_use]
    pub fn bind(mut self, keys: &[Chord], action: ActionId, help: &'static str) -> KeyMap {
        self.bindings.push(Binding::new(keys, action, help));
        self
    }

    /// Add a sequence.
    #[must_use]
    pub fn bind_seq(mut self, seq: &[Chord], action: ActionId, help: &'static str) -> KeyMap {
        self.seqs.push(SeqBinding {
            seq: Chords::of(seq),
            action,
            help,
        });
        self
    }

    /// The first binding this key fires, in declaration order.
    pub fn match_first(&self, k: &Key) -> Option<ActionId> {
        self.bindings
            .iter()
            .find(|b| b.matches(k, self.mode))
            .map(Binding::action)
    }

    /// Advance the sequence machine.
    ///
    /// **Call this before [`KeyMap::match_first`]**, or a flat binding on `g` fires and `g g` is
    /// unreachable. The ordering is load-bearing and is why these are two calls rather than one.
    ///
    /// An expired prefix is dropped before the key is considered, so the key starts a fresh
    /// sequence rather than completing a stale one. And **an abandoned prefix re-offers the key that
    /// broke it**: without that, `g x g g` loses the second `g g`.
    pub fn step(&self, p: &mut Pending, k: &Key, timeout: Duration) -> SeqStep {
        if k.kind == KeyKind::Release {
            return SeqStep::Miss;
        }
        // Expire first, so a key arriving after the timeout starts something rather than finishing
        // something the user has forgotten about.
        if let Some(deadline) = p.deadline(timeout)
            && k.at >= deadline
        {
            *p = Pending::none();
        }
        if let Some(ix) = p.seq {
            let seq = self.seqs[ix].seq.as_slice();
            if p.at < seq.len() && seq[p.at].matches(k, self.mode) {
                p.at += 1;
                if p.at == seq.len() {
                    let action = self.seqs[ix].action;
                    *p = Pending::none();
                    return SeqStep::Fired(action);
                }
                p.since = Some(k.at);
                return SeqStep::Held;
            }
            // The prefix is broken. Fall through and let this key start a sequence of its own.
            *p = Pending::none();
        }
        for (ix, s) in self.seqs.iter().enumerate() {
            let seq = s.seq.as_slice();
            if seq.first().is_some_and(|c| c.matches(k, self.mode)) {
                if seq.len() == 1 {
                    return SeqStep::Fired(s.action);
                }
                *p = Pending {
                    seq: Some(ix),
                    at: 1,
                    since: Some(k.at),
                };
                return SeqStep::Held;
            }
        }
        SeqStep::Miss
    }

    /// Drop a standing prefix. What an application's timeout calls.
    pub fn abandon(&self, p: &mut Pending) {
        *p = Pending::none();
    }

    /// How many flat bindings.
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Whether there are none.
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }
}

/// The buffer a frame copies matches into, **cleared and never freed**.
///
/// This is the frame's half of the two-lifetimes split, and it lives here rather than on the frame
/// because the frame does not exist yet — ticket 08 owns `Ctx` and will hold one of these. A frame
/// that declares the same maps every frame allocates on the first one and never again.
#[derive(Clone, Debug, Default)]
pub struct Matches {
    of: Vec<Match>,
    ranges: Vec<Declared>,
}

/// One `key_map` call: **a range tagged with the scope that was open when it was made.**
///
/// Spec §9's scope resolution is this record and a walk outward. The flat pass — one list in
/// declaration order — is wrong in the expensive direction: `Ctrl+S` under a
/// [`Trap`](crate::focus::ScopeKind::Trap) fires the application's *Save* instead of the modal's,
/// which is a document written behind a dialog the user has not confirmed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Declared {
    /// The scope open at the declaration, as an index into the frame's scope list. `None` is the
    /// frame level, which is where an application declares its own map.
    scope: Option<u32>,
    /// Where the copied bindings start.
    at: u32,
    /// How many there are.
    len: u32,
    /// How this map compares modifiers. **Per range and not per buffer**, because a modal may
    /// legitimately match differently from the application under it.
    mode: MatchMode,
}

impl Matches {
    /// A buffer with room for a typical map.
    pub fn new() -> Matches {
        Matches {
            of: Vec::with_capacity(64),
            ranges: Vec::with_capacity(8),
        }
    }

    /// Forget the contents and keep the allocation. Called once a frame.
    pub fn clear(&mut self) {
        self.of.clear();
        self.ranges.clear();
    }

    /// Copy a map's routing half in. **The help is not copied**, which is the saving: 44 bytes a
    /// binding instead of 64, and no `&'static str` travelling with the frame.
    pub fn declare(&mut self, map: &KeyMap) {
        self.declare_in(map, None);
    }

    /// Copy a map in, **tagged with the scope that is open**.
    ///
    /// The frame's own [`Ctx::key_map`](crate::Ctx::key_map) is the only caller that passes anything
    /// but `None`; see [`Matches::match_in`] for the walk that reads the tag.
    pub fn declare_in(&mut self, map: &KeyMap, scope: Option<u32>) {
        let at = u32::try_from(self.of.len()).unwrap_or(u32::MAX);
        self.of.extend(map.bindings.iter().map(|b| b.m));
        self.ranges.push(Declared {
            scope,
            at,
            len: u32::try_from(map.bindings.len()).unwrap_or(u32::MAX),
            mode: map.mode,
        });
    }

    /// The first match among the maps declared **for exactly this scope**, in declaration order.
    ///
    /// One rung of the innermost-first walk: the caller climbs the scope's parents and finishes at
    /// `None`, which is the frame level. **First match wins** inside a rung, exactly as it does
    /// inside one map.
    pub fn match_in(&self, k: &Key, scope: Option<u32>) -> Option<ActionId> {
        self.ranges
            .iter()
            .filter(|r| r.scope == scope)
            .find_map(|r| {
                let from = r.at as usize;
                let to = from + r.len as usize;
                self.of[from..to]
                    .iter()
                    .find(|m| m.matches(k, r.mode))
                    .map(|m| m.action)
            })
    }

    /// The first match, in declaration order across everything declared this frame.
    pub fn match_first(&self, k: &Key, mode: MatchMode) -> Option<ActionId> {
        self.of
            .iter()
            .find(|m| m.matches(k, mode))
            .map(|m| m.action)
    }

    /// How many matches are declared.
    pub fn len(&self) -> usize {
        self.of.len()
    }

    /// Whether none are.
    pub fn is_empty(&self) -> bool {
        self.of.is_empty()
    }

    /// How much room the buffer holds without allocating.
    pub fn capacity(&self) -> usize {
        self.of.capacity()
    }
}

/// Write a chord the way a help bar shows it, into a caller-owned buffer.
///
/// `&mut String` rather than a returned `String`, on the rule that **a call a human makes may
/// allocate and a call the frame makes may not** — a help bar is drawn once per keystroke at worst.
///
/// # What it cannot do, which is the honest part
///
/// **It names the base layout, and there is no query in any terminal protocol that answers what key
/// K would print.** So on a Cyrillic layout a help bar truthfully shows `Ctrl+N` for a key whose
/// keycap reads `Т`, and **no protocol tier fixes it before the user has pressed that key at least
/// once.** That is a refusal rather than a limitation: the alternative is inventing a layout table,
/// and the terminal already knows the answer this process cannot ask for.
///
/// ```
/// use vitui_runtime::keys::{Chord, write_chord};
/// use vitui_engine::KeyCode;
///
/// let mut out = String::new();
/// write_chord(&mut out, Chord::key('s').ctrl());
/// assert_eq!(out, "Ctrl+S");
///
/// out.clear();
/// write_chord(&mut out, Chord::new(KeyCode::F(5)));
/// assert_eq!(out, "F5");
///
/// out.clear();
/// write_chord(&mut out, Chord::key('p').ctrl().shift());
/// assert_eq!(out, "Ctrl+Shift+P");
/// ```
pub fn write_chord(out: &mut String, c: Chord) {
    use std::fmt::Write as _;

    // The locks are masked out before anything is printed: a help bar that showed `Caps+Ctrl+S`
    // would be reporting keyboard state as part of a shortcut.
    let mods = c.mods.chord();
    for (bit, name) in [
        (Mods::CTRL, "Ctrl+"),
        (Mods::ALT, "Alt+"),
        (Mods::SHIFT, "Shift+"),
        (Mods::SUPER, "Super+"),
        (Mods::HYPER, "Hyper+"),
        (Mods::META, "Meta+"),
    ] {
        if mods.contains(bit) {
            out.push_str(name);
        }
    }
    match c.code {
        KeyCode::Char(' ') => out.push_str("Space"),
        KeyCode::Char(ch) => {
            for up in ch.to_uppercase() {
                out.push(up);
            }
        }
        KeyCode::F(n) => {
            let _ = write!(out, "F{n}");
        }
        // `KeyCode` is `#[non_exhaustive]`, so this arm is required rather than tidy.
        other => {
            let _ = write!(out, "{other:?}");
        }
    }
}

/// Write a binding's first chord and its help, into a caller-owned buffer.
///
/// The first alternative, because that is the one a user was shown.
pub fn write_help(out: &mut String, b: &Binding) {
    if let Some(&first) = b.keys().first() {
        write_chord(out, first);
        out.push(' ');
    }
    out.push_str(b.help);
}

#[cfg(test)]
mod tests {
    use super::corpus::{self, Layout, LegacyCtrl, Tier};
    use super::*;
    use vitui_engine::KeyText;

    /// A press, for the gates that do not go through the rig.
    fn press(c: Chord) -> Key {
        Key {
            code: c.code,
            mods: c.mods,
            kind: KeyKind::Press,
            text: KeyText::EMPTY,
            at: Instant::now(),
        }
    }

    /// The same press with extra modifiers held.
    fn press_with(c: Chord, extra: Mods) -> Key {
        Key {
            mods: c.mods.with(extra),
            ..press(c)
        }
    }

    /// How many of a map's bindings fire when their own first chord is pressed with `extra` held.
    fn fire_count(map: &KeyMap, extra: Mods) -> usize {
        map.bindings
            .iter()
            .filter(|b| {
                let Some(&first) = b.keys().first() else {
                    return false;
                };
                map.match_first(&press_with(first, extra)) == Some(b.action())
            })
            .count()
    }

    // ---------------------------------------------------------------------------------------------
    // The sizes, which are the two-objects-two-lifetimes split as numbers.
    // ---------------------------------------------------------------------------------------------

    /// **The frame copies the match and never the help**: 44 bytes against 64.
    ///
    /// Asserted as literals rather than as an inequality, because the point is not *smaller* — it is
    /// that the help string is 16 of those 64 bytes and it routes nothing.
    #[test]
    fn the_frame_copies_the_match_and_never_the_help() {
        assert_eq!(size_of::<Match>(), 44);
        assert_eq!(size_of::<Binding>(), 64);
        assert_eq!(size_of::<Chord>(), 12);
        assert_eq!(size_of::<Chords>(), 40);
        // And the help is exactly the difference.
        assert_eq!(
            size_of::<Binding>() - size_of::<Match>(),
            size_of::<&str>() + 4
        );
    }

    /// The price of escaping `E0716`: inline chords are 64 bytes where a static slice would be 40.
    ///
    /// Stated as a number so the trade is in the repository. The compile outcome that makes it
    /// necessary is the paired doctest on [`Binding`].
    #[test]
    fn inline_chords_cost_twenty_four_bytes_over_a_static_slice() {
        struct StaticBinding {
            _keys: &'static [Chord],
            _action: ActionId,
            _help: &'static str,
        }
        assert_eq!(size_of::<StaticBinding>(), 40);
        assert_eq!(size_of::<Binding>(), 64);
    }

    // ---------------------------------------------------------------------------------------------
    // "First match wins" is not an equality.
    // ---------------------------------------------------------------------------------------------

    /// **Thirty-three of thirty-three fire, and thirty-three still fire with caps lock on.**
    ///
    /// Also with num lock, and with both — because the failure is a mask and a mask that got one bit
    /// right would pass a one-lock test.
    #[test]
    fn every_binding_still_fires_with_the_locks_on() {
        let map = corpus::map(On::BaseLayout);
        assert_eq!(map.len(), 33);
        assert_eq!(fire_count(&map, Mods::NONE), 33);
        assert_eq!(
            fire_count(&map, Mods::CAPS),
            33,
            "caps lock disarmed something"
        );
        assert_eq!(
            fire_count(&map, Mods::NUM),
            33,
            "num lock disarmed something"
        );
        assert_eq!(fire_count(&map, Mods::CAPS.with(Mods::NUM)), 33);
    }

    /// **And read as an equality, caps lock disarms the whole map at once.**
    ///
    /// Zero of thirty-three, with nothing on screen to say why. This is the defect the mask exists
    /// against, kept runnable so the mask is evidence rather than an assertion.
    #[test]
    fn an_equality_match_loses_every_binding_when_caps_lock_is_on() {
        let map = corpus::map(On::BaseLayout).with_mode(MatchMode::Equality);
        assert_eq!(
            fire_count(&map, Mods::NONE),
            33,
            "equality is fine until a lock is on"
        );
        assert_eq!(
            fire_count(&map, Mods::CAPS),
            0,
            "every binding at once, which is the shape of the failure"
        );
    }

    /// The two state bits never enter a chord in the first place.
    #[test]
    fn the_locks_are_not_expressible_in_a_chord() {
        // `Chord`'s builders are ctrl, alt and shift. There is no `caps()` and no `num()`, so the
        // only way a lock could reach a chord is a caller writing `Mods::CAPS` into the field — and
        // the mask means it would still be ignored.
        let c = Chord::key('s').ctrl();
        assert_eq!(c.mods, Mods::CTRL);
        let with_lock = Chord {
            mods: Mods::CTRL.with(Mods::CAPS),
            ..c
        };
        assert!(
            with_lock.matches(&press(c), MatchMode::Masked),
            "a lock written into a chord is masked out of it"
        );
        assert!(!INTENT.contains(Mods::CAPS));
        assert!(!INTENT.contains(Mods::NUM));
        for bit in [
            Mods::SHIFT,
            Mods::ALT,
            Mods::CTRL,
            Mods::SUPER,
            Mods::HYPER,
            Mods::META,
        ] {
            assert!(INTENT.contains(bit));
        }
    }

    /// The intent mask is the engine's own projection, bit for bit.
    ///
    /// Spec §9 calls it `Mods::intent()`; the engine ships it as `Mods::chord()`. This is the
    /// assertion that the two names are one operation, so the rename is a documentation fact rather
    /// than a behavioural guess.
    #[test]
    fn the_intent_mask_is_the_engines_own_projection() {
        for extra in [
            Mods::NONE,
            Mods::CAPS,
            Mods::NUM,
            Mods::CAPS.with(Mods::NUM),
        ] {
            for base in [Mods::NONE, Mods::CTRL, Mods::ALT.with(Mods::SHIFT), INTENT] {
                let held = base.with(extra);
                assert_eq!(held.chord(), base, "chord() is the intent half of {held:?}");
            }
        }
    }

    // ---------------------------------------------------------------------------------------------
    // Matching, and the edges it refuses.
    // ---------------------------------------------------------------------------------------------

    /// First match wins, in declaration order.
    #[test]
    fn first_match_wins_in_declaration_order() {
        let map = KeyMap::new()
            .bind(&[Chord::key('s').ctrl()], 1, "first")
            .bind(&[Chord::key('s').ctrl()], 2, "second");
        assert_eq!(map.match_first(&press(Chord::key('s').ctrl())), Some(1));
    }

    /// Any alternative fires.
    #[test]
    fn any_alternative_fires() {
        let map = KeyMap::new().bind(&[Chord::key('q').ctrl(), Chord::key('c').ctrl()], 9, "Quit");
        assert_eq!(map.match_first(&press(Chord::key('q').ctrl())), Some(9));
        assert_eq!(map.match_first(&press(Chord::key('c').ctrl())), Some(9));
        assert_eq!(map.match_first(&press(Chord::key('x').ctrl())), None);
    }

    /// **A release never matches**, or every binding would fire twice on a terminal that reports
    /// both edges.
    #[test]
    fn a_release_never_matches() {
        let map = corpus::map(On::BaseLayout);
        let mut key = press(Chord::key('s').ctrl());
        assert_eq!(map.match_first(&key), Some(corpus::SAVE));
        key.kind = KeyKind::Release;
        assert_eq!(map.match_first(&key), None);
    }

    /// **A repeat does match**, which is what makes a held key behave the same at both tiers.
    #[test]
    fn a_repeat_matches_because_a_legacy_terminal_sends_presses() {
        let map = corpus::map(On::BaseLayout);
        let mut key = press(Chord::key('j'));
        key.kind = KeyKind::Repeat;
        assert_eq!(
            map.match_first(&key),
            Some(50),
            "a held j scrolls, and below kitty it arrives as presses anyway"
        );
    }

    /// A binding with too many alternatives is clamped, not refused.
    #[test]
    fn a_binding_with_too_many_alternatives_is_clamped_not_refused() {
        let map = KeyMap::new().bind(
            &[
                Chord::key('a'),
                Chord::key('b'),
                Chord::key('c'),
                Chord::key('d'),
            ],
            1,
            "four",
        );
        assert_eq!(map.bindings[0].keys().len(), MAX_ALTS);
        assert_eq!(map.match_first(&press(Chord::key('c'))), Some(1));
        assert_eq!(
            map.match_first(&press(Chord::key('d'))),
            None,
            "the fourth was dropped, and dropping it is the documented answer"
        );
    }

    /// A `Typed` chord never matches a key that produced no text.
    ///
    /// **The only half of `On::Typed` this crate can test.** `KeyText` has no public constructor —
    /// deliberately, so that nothing can forge a key whose `code` and `text` disagree — so there is
    /// no way to build a key that a `Typed` chord *would* match. The gate is worth more than the
    /// test, and the shortfall is written down rather than implied.
    #[test]
    fn a_typed_chord_never_matches_a_key_with_no_text() {
        let map = KeyMap::new().bind(&[Chord::typed('j')], 1, "typed j");
        // Same code, same modifiers, empty text.
        assert_eq!(map.match_first(&press(Chord::key('j'))), None);
    }

    // ---------------------------------------------------------------------------------------------
    // The base layout, as a scene.
    // ---------------------------------------------------------------------------------------------

    /// **The base-layout claim is a capability, and the count says so: 33 / 28 / 14 of 33, with 0
    /// wrong at every one.**
    ///
    /// The failure is **silence, not misfire** — which is exactly why a US-layout test suite finds
    /// none of it, and why this has to be a scene rather than a check.
    #[test]
    fn the_base_layout_claim_is_a_capability_and_the_count_says_so() {
        let map = corpus::map(On::BaseLayout);

        let full = corpus::reach(&map, &Layout::RU, Tier::BaseLayout, LegacyCtrl::None);
        let fallback = corpus::reach(&map, &Layout::RU, Tier::Legacy, LegacyCtrl::LatinFallback);
        let silent = corpus::reach(&map, &Layout::RU, Tier::Legacy, LegacyCtrl::None);

        assert_eq!(
            (full.reachable, full.lost),
            (33, 0),
            "flag 4 reaches everything"
        );
        assert_eq!(
            (fallback.reachable, fallback.lost),
            (28, 5),
            "the bare letters go"
        );
        assert_eq!(
            (silent.reachable, silent.lost),
            (14, 19),
            "and the accelerators too"
        );
        assert_eq!(
            full.wrong + fallback.wrong + silent.wrong,
            0,
            "nothing ever fires the wrong action — the loss is silence"
        );
    }

    /// And on a US layout every tier reaches everything, which is why nine tickets noticed nothing.
    #[test]
    fn a_us_layout_reaches_everything_at_every_tier() {
        let map = corpus::map(On::BaseLayout);
        for (tier, legacy) in [
            (Tier::BaseLayout, LegacyCtrl::None),
            (Tier::Legacy, LegacyCtrl::LatinFallback),
            (Tier::Legacy, LegacyCtrl::None),
        ] {
            let r = corpus::reach(&map, &Layout::US, tier, legacy);
            assert_eq!(
                r.reachable, r.total,
                "US loses something at {tier:?}/{legacy:?}"
            );
            assert_eq!(r.wrong, 0);
        }
    }

    /// **The whole loss is one family.** Function keys, named keys and `Alt`+named are escape
    /// sequences and are whole at every tier; bare letters go from six of six to one of six.
    #[test]
    fn the_loss_is_one_family_and_the_survivor_is_ascii() {
        let letters = {
            let mut m = KeyMap::new();
            for (c, a, h) in corpus::LETTERS {
                m = m.bind(&[Chord::key(c)], a, h);
            }
            m
        };
        let escapes = {
            let mut m = KeyMap::new();
            for (n, a, h) in corpus::FKEYS {
                m = m.bind(&[Chord::new(KeyCode::F(n))], a, h);
            }
            for (code, a, h) in corpus::NAMED {
                m = m.bind(&[Chord::new(code)], a, h);
            }
            for (code, a, h) in corpus::ALT {
                m = m.bind(&[Chord::new(code).alt()], a, h);
            }
            m
        };

        let full = corpus::reach(&letters, &Layout::RU, Tier::BaseLayout, LegacyCtrl::None);
        let legacy = corpus::reach(&letters, &Layout::RU, Tier::Legacy, LegacyCtrl::None);
        assert_eq!(full.reachable, 6, "six of six at flag 4");
        assert_eq!(
            legacy.reachable, 1,
            "and one of six below it — `/`, whose Cyrillic position still prints ASCII"
        );

        for (tier, legacy) in [
            (Tier::BaseLayout, LegacyCtrl::None),
            (Tier::Legacy, LegacyCtrl::LatinFallback),
            (Tier::Legacy, LegacyCtrl::None),
        ] {
            let r = corpus::reach(&escapes, &Layout::RU, tier, legacy);
            assert_eq!(
                r.reachable, r.total,
                "an escape sequence is whole at {tier:?}/{legacy:?}"
            );
        }
    }

    /// **Matching never consults the tier**, the boolean, or anything about the keyboard.
    ///
    /// There is no parameter by which it could — `match_first` takes a `&Key` and nothing else — so
    /// this is structural rather than behavioural, and the test says so instead of pretending to vary
    /// something.
    ///
    /// # And `base_layout_reported`'s `true` arm cannot be tested from anywhere in this workspace
    ///
    /// `Capabilities` has a `pub(crate)` field and no public constructor, so the runtime cannot build
    /// one — the only way to hold one is to be handed it by an attached `Screen`. A headless attach is
    /// the only kind a test can do, and it forces every input axis to `false` (ADR 0010's amendment:
    /// *a declaration cannot make an event arrive*, so there is no `Overrides` field to turn it on
    /// with either).
    ///
    /// So the `false` arm is measured below against a real `Capabilities`, and **the `true` arm needs
    /// a kitty-capable terminal and a human.** That is the right trade — a forgeable `Capabilities`
    /// would let a component declare a capability into existence — and it is written down rather than
    /// left as a gap in the count.
    #[test]
    fn matching_never_consults_the_keyboard_capability() {
        let map = corpus::map(On::BaseLayout);
        let key = press(Chord::key('s').ctrl());
        let answer = map.match_first(&key);
        assert_eq!(answer, Some(corpus::SAVE));

        // A real `Capabilities`, from the only door there is.
        let (screen, _wake) = vitui_engine::Engine::new(vitui_engine::Config {
            clock: vitui_engine::Clock::Manual,
            output: vitui_engine::Output::Sink(Box::new(Vec::new())),
            size: (8, 1),
            ..Default::default()
        })
        .attach()
        .expect("attaching to a sink cannot fail");
        assert!(
            !base_layout_reported(screen.capabilities()),
            "a headless attach reports no alternate keys, which is the only arm reachable here"
        );

        // And the answer did not move, because the capability never reached the matcher.
        assert_eq!(map.match_first(&key), answer);
    }

    // ---------------------------------------------------------------------------------------------
    // Sequences.
    // ---------------------------------------------------------------------------------------------

    /// `g g` resolves inside one batch, with `Pending` the only thing that crosses a frame.
    #[test]
    fn a_sequence_resolves_inside_one_batch() {
        let map = corpus::map_with_seqs(On::BaseLayout);
        let mut p = Pending::none();
        let g = press(Chord::key('g'));
        assert_eq!(map.step(&mut p, &g, KeyMap::TIMEOUT), SeqStep::Held);
        assert!(p.is_standing());
        assert_eq!(
            map.step(&mut p, &g, KeyMap::TIMEOUT),
            SeqStep::Fired(corpus::TOP)
        );
        assert!(!p.is_standing(), "and nothing is left standing");
    }

    /// **The sequence machine runs before the flat map**, or `g g` is unreachable.
    #[test]
    fn a_sequence_must_be_stepped_before_the_flat_map() {
        // A map with both a flat `g` and a `g g`.
        let map = KeyMap::new()
            .bind(&[Chord::key('g')], 1, "flat g")
            .bind_seq(&[Chord::key('g'), Chord::key('g')], 2, "g g");
        let g = press(Chord::key('g'));
        // Stepped first: the prefix stands and the flat binding is not consulted.
        let mut p = Pending::none();
        assert_eq!(map.step(&mut p, &g, KeyMap::TIMEOUT), SeqStep::Held);
        // The other order: the flat binding fires and the sequence can never be typed.
        assert_eq!(map.match_first(&g), Some(1));
    }

    /// **An abandoned prefix re-offers the key that broke it**, or `g x g g` loses the second pair.
    #[test]
    fn an_abandoned_prefix_re_offers_the_key_that_broke_it() {
        let map = corpus::map_with_seqs(On::BaseLayout);
        let mut p = Pending::none();
        let g = press(Chord::key('g'));
        let x = press(Chord::key('z'));
        assert_eq!(map.step(&mut p, &g, KeyMap::TIMEOUT), SeqStep::Held);
        assert_eq!(map.step(&mut p, &x, KeyMap::TIMEOUT), SeqStep::Miss);
        assert_eq!(map.step(&mut p, &g, KeyMap::TIMEOUT), SeqStep::Held);
        assert_eq!(
            map.step(&mut p, &g, KeyMap::TIMEOUT),
            SeqStep::Fired(corpus::TOP)
        );
    }

    /// A one-chord sequence fires immediately, which is what makes `bind_seq` safe to use for a
    /// single key.
    #[test]
    fn a_one_chord_sequence_fires_at_once() {
        let map = corpus::map_with_seqs(On::BaseLayout);
        let mut p = Pending::none();
        assert_eq!(
            map.step(&mut p, &press(Chord::key('G')), KeyMap::TIMEOUT),
            SeqStep::Fired(corpus::BOTTOM)
        );
        assert!(!p.is_standing());
    }

    /// **A prefix expires, and the key that arrives late starts something rather than finishing
    /// something forgotten.**
    #[test]
    fn a_prefix_expires_and_the_late_key_starts_afresh() {
        let map = corpus::map_with_seqs(On::BaseLayout);
        let mut p = Pending::none();
        let mut g = press(Chord::key('g'));
        assert_eq!(map.step(&mut p, &g, KeyMap::TIMEOUT), SeqStep::Held);
        // The same key, a second and a half later.
        g.at += Duration::from_millis(1500);
        assert_eq!(
            map.step(&mut p, &g, KeyMap::TIMEOUT),
            SeqStep::Held,
            "the stale prefix was dropped and this g starts a new one"
        );
        assert!(p.is_standing());
    }

    /// **A standing prefix owes exactly one wakeup**, and nothing else does.
    #[test]
    fn a_standing_prefix_owes_exactly_one_deadline() {
        let map = corpus::map_with_seqs(On::BaseLayout);
        let mut p = Pending::none();
        assert_eq!(
            p.deadline(KeyMap::TIMEOUT),
            None,
            "nothing pending, nothing owed"
        );
        let g = press(Chord::key('g'));
        map.step(&mut p, &g, KeyMap::TIMEOUT);
        let first = p.deadline(KeyMap::TIMEOUT).expect("a prefix is standing");
        assert_eq!(first, g.at + KeyMap::TIMEOUT);
        // Still one, not two.
        assert_eq!(p.deadline(KeyMap::TIMEOUT), Some(first));
        map.abandon(&mut p);
        assert_eq!(p.deadline(KeyMap::TIMEOUT), None, "and it is discharged");
    }

    /// The default is a second, which is vim's `timeoutlen`, and it is a parameter.
    #[test]
    fn the_timeout_is_a_second_and_it_is_a_parameter() {
        assert_eq!(KeyMap::TIMEOUT, Duration::from_millis(1000));
        // Passed per call, so two maps can differ without the runtime holding a knob.
        let map = corpus::map_with_seqs(On::BaseLayout);
        let mut p = Pending::none();
        let mut g = press(Chord::key('g'));
        map.step(&mut p, &g, Duration::from_millis(10));
        g.at += Duration::from_millis(50);
        assert_eq!(
            map.step(&mut p, &g, Duration::from_millis(10)),
            SeqStep::Held,
            "a ten-millisecond timeout expires where a second would not"
        );
    }

    // ---------------------------------------------------------------------------------------------
    // The frame's buffer, and help.
    // ---------------------------------------------------------------------------------------------

    /// The buffer is cleared and never freed.
    #[test]
    fn the_match_buffer_is_cleared_and_never_freed() {
        let map = corpus::map(On::BaseLayout);
        let mut buf = Matches::new();
        buf.declare(&map);
        assert_eq!(buf.len(), 33);
        let capacity = buf.capacity();
        buf.clear();
        assert!(buf.is_empty());
        assert_eq!(buf.capacity(), capacity, "the allocation stayed");
        buf.declare(&map);
        assert_eq!(buf.capacity(), capacity, "and it was reused");
        assert_eq!(
            buf.match_first(&press(Chord::key('s').ctrl()), MatchMode::Masked),
            Some(corpus::SAVE)
        );
    }

    /// Help is a chord and a string, into a buffer the caller owns.
    #[test]
    fn help_is_a_chord_and_a_string() {
        let map = corpus::map(On::BaseLayout);
        let mut out = String::new();
        write_help(&mut out, &map.bindings[0]);
        assert_eq!(out, "Ctrl+S Save");
        out.clear();
        write_chord(&mut out, Chord::new(KeyCode::Char(' ')));
        assert_eq!(out, "Space");
        out.clear();
        write_chord(&mut out, Chord::new(KeyCode::Escape));
        assert_eq!(out, "Escape");
    }

    /// **A lock is never printed in help**, because it is not part of the shortcut.
    #[test]
    fn a_lock_is_never_printed_in_help() {
        let mut out = String::new();
        write_chord(
            &mut out,
            Chord {
                mods: Mods::CTRL.with(Mods::CAPS).with(Mods::NUM),
                ..Chord::key('s')
            },
        );
        assert_eq!(out, "Ctrl+S");
    }

    /// The whole corpus writes into one reused buffer.
    #[test]
    fn the_whole_corpus_writes_into_one_buffer() {
        let map = corpus::map(On::BaseLayout);
        let mut out = String::new();
        for b in &map.bindings {
            out.clear();
            write_help(&mut out, b);
            assert!(!out.is_empty());
            assert!(out.contains(b.help));
        }
    }
}

#[cfg(test)]
mod the_accelerator_a_field_ate {
    //! **Found by pressing `Ctrl+S` into a form, and six tickets old.**
    //!
    //! A text field matched on `Key::code` alone, so a focused field typed the letter of every
    //! accelerator in the map: `Ctrl+S` put an `s` in the box and saved nothing. It survived six
    //! tickets because until the modifier byte existed there was nothing to check.
    //!
    //! The fix is one comparison, and it belongs in the component library rather than here — a
    //! runtime that filtered keys before a field saw them would be deciding what a field is for. So
    //! this module is the **obligation written where a component author reads it**, with both halves
    //! of the defect runnable.

    use super::corpus;
    use super::*;
    use vitui_engine::KeyText;

    fn press(c: Chord) -> Key {
        Key {
            code: c.code,
            mods: c.mods,
            kind: KeyKind::Press,
            text: KeyText::EMPTY,
            at: Instant::now(),
        }
    }

    /// A field that matches on `code` alone. **This is the defect.**
    fn field_matching_on_code_alone(k: &Key) -> Option<char> {
        match k.code {
            KeyCode::Char(c) => Some(c),
            _ => None,
        }
    }

    /// A field that checks the intent half first. **This is the fix, and it is one comparison.**
    ///
    /// `mods.chord()` and not `text.is_empty()`, although both work today: a chord is defined by
    /// *intent*, and a field that keyed off the absence of text would be relying on a coincidence of
    /// the wire — `KeyText` is also empty when a cluster overflowed.
    fn field_checking_intent(k: &Key) -> Option<char> {
        if !k.mods.chord().is_empty() {
            return None;
        }
        match k.code {
            KeyCode::Char(c) => Some(c),
            _ => None,
        }
    }

    /// **Both halves**, because a field that swallowed the key silently would pass the first alone.
    #[test]
    fn a_field_must_decline_a_chord_instead_of_typing_it() {
        let map = corpus::map(On::BaseLayout);
        let accelerator = press(Chord::key('s').ctrl());

        assert_eq!(
            field_matching_on_code_alone(&accelerator),
            Some('s'),
            "the defect reproduces: a field typed the letter of the accelerator"
        );
        assert_eq!(
            field_checking_intent(&accelerator),
            None,
            "and one comparison stops it"
        );
        assert_eq!(
            map.match_first(&accelerator),
            Some(corpus::SAVE),
            "and the accelerator still reaches the map, which is the half that makes this a \
             regression test rather than a mute"
        );
    }

    /// And an ordinary letter still reaches the field, which is the direction that matters second.
    #[test]
    fn an_ordinary_letter_still_reaches_the_field() {
        let typed = press(Chord::key('s'));
        assert_eq!(field_checking_intent(&typed), Some('s'));
        // Shift is intent too, so a capital is declined by this simple field — which is why a real
        // one reads `text` rather than `code`. Named here so nobody reads the fix as complete.
        let shifted = press(Chord::key('s').shift());
        assert_eq!(
            field_checking_intent(&shifted),
            None,
            "a real field reads `text`; this one is the minimal shape of the comparison"
        );
    }
}
