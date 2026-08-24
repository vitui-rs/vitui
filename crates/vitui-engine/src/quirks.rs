//! The layer for terminals that answer the query correctly and then misbehave.
//!
//! Level 5 of spec §10's precedence, and the whole reason it sits **after** detection: a terminal
//! that reports a capability it then mis-renders is not a detection failure, and asking again more
//! carefully cannot fix it. Detection says what the other end claims; this says what it does.
//!
//! # It is a mechanism, and populating it is field work
//!
//! §15 put populating this table in the fog on purpose. **A quirk table is field work**: each entry
//! is one terminal, one version range and one observed misbehaviour, and none of that can be
//! established from a document. Three entries shipped on libvaxis's production experience; the
//! fourth and fifth are the ones this repository gathered itself, and it took an instrument to get
//! either.
//!
//! The five, and how each is recognised, which is the part that matters:
//!
//! | terminal | recognised by | quirk |
//! |---|---|---|
//! | ConPTY | the target is Windows and nothing answered XTVERSION | legacy SGR, and its own underline-colour form |
//! | Termux | `$TERMUX_VERSION` is set | legacy SGR |
//! | VSCode's integrated terminal | `$TERM_PROGRAM` is `vscode` | legacy SGR |
//! | tmux | **XTVERSION answers `tmux …`** | overline is accepted, stored, and never forwarded |
//! | kitty | **XTVERSION answers `kitty(…)`** | conceal and overline have no attribute to be stored in |
//!
//! **The first three are not recognised by a query, and that is not an oversight**: they are
//! recognised the way libvaxis recognises them, because the misbehaviour is not something the
//! terminal will admit to. This is the one place `$TERM_PROGRAM`-shaped evidence is legitimate, and
//! it is legitimate precisely because it is not being used to *detect a capability* — spec §10's
//! refusal of terminfo is a refusal to infer capabilities from a name, and an entry here overrides
//! a capability that was measured.
//!
//! # The fourth entry, and what it cost to know
//!
//! tmux **is** recognised by a query, which makes it the first entry here that is. It answers
//! XTVERSION with `DCS >|tmux 3.7c ST`, so the version arrives with the identity and neither is
//! inferred from a name.
//!
//! Its quirk was found by `conform/`, and the shape of the finding is why the entry can be trusted.
//! Three captures of the same scene, committed as fixtures:
//!
//! | path | overline |
//! |---|---|
//! | the engine into Ghostty 1.3.1, Ghostty's own dump | **present** |
//! | the engine into tmux 3.7c, tmux's own grid via `capture-pane -e` | **present** |
//! | the engine into tmux 3.7c into Ghostty, Ghostty's dump | **gone** |
//!
//! So tmux parses SGR 53, stores it in the cell, hands it back when asked — and does not put it on
//! the wire. Ten of the eleven attribute bits survive the same trip, so this is one attribute and
//! not a broken path. A control probe with a raw `printf` and no engine anywhere in it reproduces
//! the whole of it, and the bytes tmux writes to its client contain no `53` at all.
//!
//! **The mechanism, and it is the reason the entry has no version boundary.** tmux 3.0's CHANGES:
//! *"Add support for the overline attribute (SGR 53). The Smol capability is needed in
//! terminal-overrides."* So tmux emits overline only where it has been told the terminal has it —
//! and `Smol` is defined in none of `xterm-ghostty`, `xterm-256color`, `tmux-256color` or
//! `screen-256color` on the machine this was measured on, while `Smulx` is defined in
//! `xterm-ghostty`, which is why the underline styles survive the trip and this does not. Before 3.0
//! there is no overline output at all. Every tmux therefore drops it **by default**, and the
//! exception is a user with `set -as terminal-features ",…:overline"` rather than a version — which
//! is a configuration this engine cannot see and §10 would not read terminfo to find.
//!
//! Ghostty renders SGR 53 (the first row of that table), so the loss is tmux's and the terminfo that
//! omits `Smol` describes a terminal that has it. That is spec §10's refusal of terminfo, arriving
//! as field evidence from a direction nothing planned for.
//!
//! # The fifth entry, and its evidence is not a screen dump
//!
//! kitty 0.48.2 came third to `conform/`'s scene 01 and disagreed on two rows: conceal and overline
//! come back bare. **A dump alone could not have earned either**, and saying why is the whole of what
//! makes this entry different from the fourth. *Not stored* and *not serialised* look identical in a
//! capture, and `conform/FINDINGS.md` had left conceal open on exactly that — kitty could resolve the
//! foreground to the background at paint time, storing nothing in a flag and rendering correctly.
//!
//! The shipped `kitty.fast_data_types.so` closes it. kitty's `Cursor` repr enumerates every
//! formatting attribute the cursor carries:
//!
//! ```text
//! Cursor(x, y, shape, blink, fg, bg, bold, italic, reverse, strikethrough, dim,
//!        decoration, decoration_fg, text_blink)
//! ```
//!
//! and the attribute constants beside it — `BOLD ITALIC REVERSE MARK STRIKETHROUGH DECORATION BLINK`
//! — say it a second time from a second table. **SGR is a mutation of the cursor**, so an attribute
//! the cursor cannot carry is one no cell can hold and no paint can consult. Paint-time resolution
//! needs a flag, and there is none. A control probe agrees from the other side: `SGR 8 ; 31` comes
//! back as `SGR 31`, so the foreground is stored unmodified and nothing was resolved on the way in
//! either.
//!
//! **The third row of that scene is not in this entry, and that is the entry's most important
//! property.** kitty writes a *dotted* underline into a capture as `CSI 4 : m`, which ECMA-48 reads
//! as single — its serialiser has a string for `4:2` and `4:3` and none for the other two. Compared
//! anyway, that row would have earned a third bit here, and the misbehaviour it described would not
//! be happening. It is a limit of the capture format, the arm declares it as one before each run,
//! and `attrs_dropped` could not hold it in any case: it is the eight flags and not the three-bit
//! underline enumeration, for the reason [`Quirks::attrs_dropped`] gives.
//!
//! **No version boundary, and this one is a bet rather than a mechanism.** tmux's has none because
//! its cause — a terminfo capability nobody defines — has none. kitty's cause is a struct, and a
//! future kitty could grow the field. The bet is cheap and one-directional: a wrong entry costs an
//! attribute that is not offered, where a missing one costs an attribute sent every frame and
//! ignored every frame. The day a kitty renders either, the boundary arrives with the run that
//! observed it — which is what [`Capabilities::identified_as`](crate::caps::Capabilities) gates.

use crate::caps::{Capabilities, Env};

/// Which escape spells an underline colour.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum Underlines {
    /// `SGR 58:2::r:g:b`, the colon-separated ITU-T form the rest of the world takes.
    #[default]
    Standard,
    /// `SGR 58;2;r;g;b`. ConPTY parses only the semicolon-separated form, which is the same
    /// disagreement that makes `legacy_sgr` necessary for 38 and 48, one parameter along.
    ConPty,
}

impl Underlines {
    pub(crate) fn word(self) -> &'static str {
        match self {
            Underlines::Standard => "standard",
            Underlines::ConPty => "conpty",
        }
    }
}

/// How long a synchronised-output block may stay open before the terminal force-flushes it.
///
/// **Every implementation force-flushes and the limits differ by an order of magnitude** — so this is
/// not one number with a safe default. §8 is where it binds, and it is private because nothing above
/// the engine opens a block.
///
/// | terminal | limit | where the number is from |
/// |---|---|---|
/// | Alacritty | 150 ms, 2 MiB | its own documentation |
/// | kitty | 2000 ms | its own documentation |
/// | tmux | 1 s | its own documentation |
/// | **Ghostty 1.3.1** | **1000 ms, no byte limit** | `sync_reset_ms = 1000` in `src/termio/Thread.zig`, *"the number of milliseconds before we reset the synchronized output flag if the running program hasn't already"* |
///
/// Ghostty was added by production ticket 05, which asked for it to be **measured**. It could not be:
/// a force-flush is a *rendering* event, and nothing a process inside the terminal can ask reports
/// whether the terminal painted. Only a screen capture can, and this repository's capture is an
/// AppleScript round trip that four runs on 2026-08-23 put between **136 ms and 623 ms** — the same
/// order as Alacritty's entire 150 ms limit, and confirming ticket 04's prediction rather than
/// discovering it. Every run prints its own figure into `conform/REPORT-ghostty.md`, so the reason is
/// a measurement with a spread and not an assertion. The number here therefore has the same
/// provenance as the three above it: the implementation, read. That is what `quirks.rs` can honestly
/// hold, and it is worth saying rather than leaving a fourth row blank.
///
/// **No entry sets this field, and nothing reads it.** The engine's own block is opened and closed
/// inside one `write` (§8's twenty bytes of fixed framing), so a frame cannot approach the smallest
/// of these limits; they bind only on a block spanning two writes, which nothing does. The numbers
/// are the headroom, written down where a future block that *does* span writes will look for them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct SyncFlush {
    /// Milliseconds a block may stay open.
    pub(crate) after_ms: u32,
    /// Bytes a block may carry, or `None` where the terminal states no limit.
    pub(crate) after_bytes: Option<usize>,
}

/// What one entry of the table says.
///
/// Every field is an override of something already established, never a fact of its own — which is
/// what keeps this a *layer* rather than a second detection path.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct Quirks {
    /// The name [`Capabilities::report`](crate::Capabilities::report) prints when detection found no
    /// identity of its own.
    pub(crate) name: Option<&'static str>,
    /// Force the pre-ITU-T **semicolon** form of SGR 38/48, for a terminal that mis-parses the
    /// modern colon one. A `bool` and not an `Option<bool>` deliberately: [`Quirks::apply`] can
    /// only ever set it to `true`, and that one-way shape is what settled which spelling is the
    /// default (arch 23). See [`crate::caps::Capabilities::legacy_sgr`].
    pub(crate) legacy_sgr: bool,
    /// Which escape spells an underline colour.
    pub(crate) underlines: Underlines,
    /// Attribute bits the terminal does not render, in the style word's own positions. This is where
    /// the eleven attribute facts live, because there is no query for any of them.
    ///
    /// **The eight flags of [`crate::style::ATTRS`] and not the eleven bits**, and `apply` refuses the
    /// other three rather than mangling them: clearing a bit of the three-bit underline *enumeration*
    /// turns `double` into `none`, so an underline style a terminal does not render is not a fact this
    /// field can hold. A terminal that has one needs a new axis, not a wider mask.
    ///
    /// Read on the wire by [`crate::quant::Quantiser::attrs`], which is what makes §10's *dropped
    /// silently at serialise time* true — production ticket 10, filed by the ticket that populated
    /// this field and found nothing reading it.
    pub(crate) attrs_dropped: u64,
    /// Overrule what was inferred about OSC 8, in either direction.
    pub(crate) hyperlinks: Option<bool>,
    /// The terminal's force-flush limits for mode 2026.
    pub(crate) sync_flush: Option<SyncFlush>,
}

impl Quirks {
    /// Which entry applies, if any.
    ///
    /// `version` is what XTVERSION reported, and its **absence** is load-bearing for the ConPTY
    /// entry: ConPTY does not implement XTVERSION, so a Windows process whose terminal did answer it
    /// is talking to something else through ConPTY rather than to ConPTY itself.
    pub(crate) fn lookup(version: Option<&str>, env: &Env) -> Quirks {
        // **First, and the order is the argument.** One entry applies, not a union of them, so
        // whichever is checked first wins where two could match — and inside tmux the thing at the
        // other end of the pty *is* tmux. `$TERM_PROGRAM=vscode` and `$TERMUX_VERSION` are inherited
        // by a tmux pane's environment and would otherwise force `legacy_sgr` for a multiplexer that
        // was observed to parse the colon form correctly. A query beats an inherited variable, and it
        // beats it about the right terminal.
        if version.is_some_and(|v| v.starts_with("tmux ")) {
            return Quirks {
                // Overline, and nothing else: the other ten attribute bits were observed to survive
                // the same trip. `conform/fixtures/ghostty-1.3.1-via-tmux-3.7c-scene01-attrs.vt` is
                // the capture, and `conform/FINDINGS.md` 2026-08-23 is the run.
                //
                // No version boundary, because the misbehaviour has none — see the module docs. A
                // tmux that starts advertising `Smol` for itself, or a future release that emits SGR
                // 53 unconditionally, is what adds one, with the run that observed it.
                attrs_dropped: crate::style::OVERLINE,
                ..Quirks::default()
            };
        }
        // Second, and for the same reason tmux is first: this is a **query**, and a query beats an
        // inherited environment variable about a terminal that is not the one at the other end of
        // the pty. kitty answers XTVERSION `DCS >| kitty(0.48.2) ST`, so the version arrives with
        // the identity.
        if version.is_some_and(|v| v.starts_with("kitty(")) {
            return Quirks {
                // Two bits, and neither is a dump's inference — see the module docs. kitty's cursor
                // has no attribute for either, so no cell can hold one and no paint can consult one.
                //
                // The scene's third disagreement, a dotted underline, is deliberately **not** here:
                // it is the capture format that cannot spell it, and an underline style is not a bit
                // this field could clear even if it were.
                attrs_dropped: crate::style::CONCEAL | crate::style::OVERLINE,
                ..Quirks::default()
            };
        }
        if env.termux.is_some() {
            return Quirks {
                name: Some("termux"),
                legacy_sgr: true,
                ..Quirks::default()
            };
        }
        if env.term_program.as_deref() == Some("vscode") {
            return Quirks {
                name: Some("vscode"),
                legacy_sgr: true,
                ..Quirks::default()
            };
        }
        if cfg!(windows) && version.is_none() {
            return Quirks {
                name: Some("conpty"),
                legacy_sgr: true,
                underlines: Underlines::ConPty,
                ..Quirks::default()
            };
        }
        // Field-populated. Everything below this line is what a real terminal, a real version range
        // and a real observed misbehaviour will add — one entry at a time, each with the report that
        // produced it.
        //
        // **Five is where the evidence stops, not where the need does**, and the last two are what
        // the sentence is for: it took building `conform/` to get either, and the eleven attribute
        // facts are now observed on **three** of spec §10's tier-1 terminals out of seven. The four
        // that remain are inference from libvaxis's three entries, and none of the four is named by
        // any of them.
        Quirks::default()
    }

    /// Lay the entry over what detection established.
    pub(crate) fn apply(self, caps: &mut Capabilities) {
        if self.legacy_sgr {
            caps.private.legacy_sgr = true;
        }
        if self.underlines != Underlines::Standard {
            caps.private.underlines = self.underlines;
        }
        debug_assert_eq!(
            self.attrs_dropped & !crate::style::ATTRS,
            0,
            "attrs_dropped is the eight flags; an underline style is not a bit to clear"
        );
        caps.private.attrs_dropped |= self.attrs_dropped;
        if let Some(hyperlinks) = self.hyperlinks {
            caps.hyperlinks = hyperlinks;
        }
        if let Some(flush) = self.sync_flush {
            caps.private.sync_flush = Some(flush);
        }
        if caps.private.identity.is_none() {
            caps.private.identity = self.name.map(str::to_string);
        }
    }
}
