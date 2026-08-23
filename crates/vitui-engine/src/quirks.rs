//! The layer for terminals that answer the query correctly and then misbehave.
//!
//! Level 5 of spec §10's precedence, and the whole reason it sits **after** detection: a terminal
//! that reports a capability it then mis-renders is not a detection failure, and asking again more
//! carefully cannot fix it. Detection says what the other end claims; this says what it does.
//!
//! # It is a mechanism, and it is deliberately not enumerated
//!
//! §15 puts populating this table in the fog on purpose. **A quirk table is field work**: each entry
//! is one terminal, one version range and one observed misbehaviour, and none of that can be
//! established from a document. What ships is the mechanism plus the three entries libvaxis's
//! production experience is the evidence for — and the comment saying so, because a table that
//! silently stayed at three entries would be indistinguishable from a table nobody needed.
//!
//! The three, and how each is recognised, which is the part that matters:
//!
//! | terminal | recognised by | quirk |
//! |---|---|---|
//! | ConPTY | the target is Windows and nothing answered XTVERSION | legacy SGR, and its own underline-colour form |
//! | Termux | `$TERMUX_VERSION` is set | legacy SGR |
//! | VSCode's integrated terminal | `$TERM_PROGRAM` is `vscode` | legacy SGR |
//!
//! **None of the three is recognised by a query**, and that is not an oversight either: they are
//! recognised the way libvaxis recognises them, because the misbehaviour is not something the
//! terminal will admit to. This is the one place `$TERM_PROGRAM`-shaped evidence is legitimate, and
//! it is legitimate precisely because it is not being used to *detect a capability* — spec §10's
//! refusal of terminfo is a refusal to infer capabilities from a name, and an entry here overrides
//! a capability that was measured.

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
/// **Every implementation force-flushes and the limits differ by an order of magnitude** — Alacritty
/// 150 ms and 2 MiB, kitty 2000 ms, tmux 1 s — so this is not one number with a safe default. §8 is
/// where it binds, and it is private because nothing above the engine opens a block.
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
        // produced it. Three is where the evidence stops, not where the need does.
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
