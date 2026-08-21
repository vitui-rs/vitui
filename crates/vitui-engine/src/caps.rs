//! What the terminal can do, what was asked of it, and the order the two are resolved in.
//!
//! # Two types, and the asymmetry between them
//!
//! [`Capabilities`] is **what is true**: built during `attach`, immutable for the life of the
//! `Screen`, and read through [`Screen::capabilities`](crate::Screen::capabilities). [`Overrides`]
//! is **what was asked for**: set before `attach`, every field an `Option`, and `None` means *let
//! detection decide*.
//!
//! The asymmetry underneath them is spec §10's, and it is about detectability. Colour is measured —
//! XTGETTCAP `RGB`, DA, OSC 10, OSC 11, OSC 4. **Glyph repertoire cannot be detected at all**: no
//! query asks whether `U+28FF` is in the font, the terminal accepts braille and draws tofu, and none
//! of the detection machinery sees it. So `--ascii` is not a discovery, it is an operator's promise.
//!
//! > **A detected axis must be flat booleans; a declared axis may be an ordered ladder.** The world
//! > does not sort; a promise is downward-closed by whoever makes it.
//!
//! See [ADR 0010](../../../docs/adr/0010-detected-axes-are-flat-declared-axes-are-ordered.md).
//! [`ColorDepth`] is ordered *and* detected, because the wire formats genuinely nest.
//!
//! # The word *tier* does not survive on the read side
//!
//! A tier was "a named combination of capabilities a scene can be rendered against", and after this
//! shape no such combination is read by anybody — **fourteen separate facts** are, two of them
//! ordered. The named combination survives only as [`Overrides::plain`], because `--ascii` and
//! `--no-color` genuinely travel together and a constructor is the right size for that.
//!
//! **Every public field had to pass one test: someone above can act on it.** What that test excludes
//! is the finding, and the excluded facts are not deleted — synchronised output, `DECSLRM`, legacy
//! SGR, the ConPTY underline-colour form, the force-flush limits and the sixteen palette entries are
//! all known, all private, and surfaced only through [`Capabilities::report`].
//!
//! **The eleven attribute bits are not here at all**, and that is the same test applied a second
//! time: there is no query for "do you render italic", DECRQM does not cover SGR, and XTGETTCAP was
//! narrowed to the single `RGB` capability precisely because terminfo descriptions are what
//! detection refused. Eleven attribute booleans would be eleven inventions. They live in the quirk
//! table ([`crate::quirks`]), and an unsupported one is dropped silently at serialise time —
//! nothing above needs to ask, because an absent attribute still draws the correct text.

use std::fmt::Write as _;

use crate::quirks::{Quirks, SyncFlush, Underlines};
use crate::style::Color;

/// A 24-bit colour as the terminal reported it.
///
/// Distinct from [`Color`], which is a *request*: this is an answer to OSC 10, OSC 11 or OSC 4, and
/// the only thing it can be is three channels. It converts into a `Color` for the one place that
/// needs it — spec §5's colour resolution, where `default` has to become channels before `Mix` can
/// touch it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Rgb {
    /// Red.
    pub r: u8,
    /// Green.
    pub g: u8,
    /// Blue.
    pub b: u8,
}

impl Rgb {
    /// A colour from its three channels.
    pub const fn new(r: u8, g: u8, b: u8) -> Rgb {
        Rgb { r, g, b }
    }
}

impl From<Rgb> for Color {
    fn from(c: Rgb) -> Color {
        Color::rgb(c.r, c.g, c.b)
    }
}

/// How much colour reaches the wire.
///
/// **Ordered, and detected** — the one axis that is both, because the wire formats genuinely nest:
/// every terminal that speaks `38;2;r;g;b` speaks `38;5;n`, and every terminal that speaks `38;5;n`
/// speaks `SGR 30`. That is not true of anything else here, which is why nothing else is a ladder.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Hash)]
pub enum ColorDepth {
    /// No colour at all. `reverse` is the one mechanism left, and it works everywhere.
    #[default]
    None,
    /// The sixteen ANSI colours. Quantising into them is a bet on the user's theme, which is why
    /// OSC 4 is worth its sixteen queries.
    Ansi16,
    /// The 256-colour cube. Indices 16..255 are fixed by specification and identical everywhere.
    Indexed256,
    /// 24-bit RGB.
    TrueColor,
}

impl ColorDepth {
    /// The word [`Capabilities::report`] prints.
    pub(crate) fn word(self) -> &'static str {
        match self {
            ColorDepth::None => "none",
            ColorDepth::Ansi16 => "ansi16",
            ColorDepth::Indexed256 => "indexed256",
            ColorDepth::TrueColor => "truecolor",
        }
    }
}

/// What a component may assume is in the font.
///
/// **Declared, never detected**, and therefore allowed to be a ladder — a promise is downward-closed
/// by whoever makes it. The engine reads this exactly nowhere: it emits no glyphs of its own, §4
/// leaves it three verbs and no box-drawing primitive, and boxes are `fill`s. A component branches
/// on it, and that is the only place the promise can be kept.
///
/// The three levels were read off a real chart rather than chosen because three felt right:
/// **braille is 256 states per cell, block elements are 8 and bottom-anchored, ASCII is 2** — so a
/// line chart is inexpressible in block elements and the component becomes an area chart, and
/// inexpressible again in ASCII and becomes something else again.
///
/// > **Choosing a glyph before drawing is legitimate; replacing one after it is drawn is
/// > forbidden.**
///
/// A border character set the runtime hands a component according to this value is a choice at the
/// source and is fine. The same table applied by the engine to cells a component already wrote is
/// what turns a chart into noise while every budget stays green.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Hash)]
pub enum GlyphSet {
    /// Printable ASCII, and nothing else.
    Ascii,
    /// Unicode a normal text font covers.
    Unicode,
    /// Braille, block elements, emoji, powerline — whatever the font actually has.
    ///
    /// **The default**, and it is the trade rather than the safe answer: `Unicode` is safer —
    /// braille and emoji are exactly what fonts lack, and the failure is tofu — but it would ship
    /// the flagship capability switched off, and requirement 6 is *UTF-8 first, with `--ascii` as
    /// the degradation path*. Since nothing can be queried, the lever is one environment variable.
    #[default]
    Extended,
}

impl GlyphSet {
    /// The word [`Capabilities::report`] prints, and the word `VITUI_GLYPHS` accepts.
    pub(crate) fn word(self) -> &'static str {
        match self {
            GlyphSet::Ascii => "ascii",
            GlyphSet::Unicode => "unicode",
            GlyphSet::Extended => "extended",
        }
    }

    fn parse(s: &str) -> Option<GlyphSet> {
        match s.trim().to_ascii_lowercase().as_str() {
            "ascii" => Some(GlyphSet::Ascii),
            "unicode" => Some(GlyphSet::Unicode),
            "extended" => Some(GlyphSet::Extended),
            _ => None,
        }
    }
}

/// Which table decides how wide a cluster is.
///
/// **Nothing reads this yet, and that is the honest state rather than an oversight** — the same
/// position [`Clock`](crate::Clock) was declared in. Our own UAX #29 / #11 tables are authoritative
/// (spec §10) and `Wcwidth` is the lever for the field case where a terminal disagrees with them so
/// badly that matching its mistake is better than being right; building the second table is not
/// this ticket's work. It is declared now because a knob that appears later is a breaking change to
/// every `Overrides` literal.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum WidthSource {
    /// The engine's generated UAX #29 / #11 tables. Authoritative.
    #[default]
    Tables,
    /// Legacy `wcwidth` semantics, for a terminal that measures that way and cannot be persuaded.
    Wcwidth,
}

impl WidthSource {
    pub(crate) fn word(self) -> &'static str {
        match self {
            WidthSource::Tables => "tables",
            WidthSource::Wcwidth => "wcwidth",
        }
    }
}

/// What was asked for, before anything was detected.
///
/// Every field **lowers or pins**, and `None` means *let detection decide*. This is where an
/// application's own `--ascii` and `--no-color` land: **the engine parses no argv**, and it never
/// will, because an engine that read `std::env::args` would be deciding what an application's flags
/// are called.
///
/// # `Some` is never overridden
///
/// The only contested pair in spec §10's precedence was the API against the environment, and this
/// shape is what removes the conflict rather than adjudicating it: **the environment fills only the
/// `None`s.** A stale `VITUI_GLYPHS` cannot silently override a flag the user just typed, and a
/// variable still works wherever the application offered no flag.
///
/// ```
/// use vitui_engine::{ColorDepth, GlyphSet, Overrides};
///
/// let plain = Overrides::plain();
/// assert_eq!(plain.colors, Some(ColorDepth::None));
/// assert_eq!(plain.glyphs, Some(GlyphSet::Ascii));
/// assert_eq!(Overrides::default().colors, None);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct Overrides {
    /// Pin the colour depth. Raising is allowed as well as lowering — that is what makes truecolor
    /// reachable with detection switched off, which is what a headless golden needs.
    pub colors: Option<ColorDepth>,
    /// Pin what may be assumed of the font. The only lever there is, since nothing can be queried.
    pub glyphs: Option<GlyphSet>,
    /// Force the pre-ITU-T colon form of SGR 38/48, for a terminal that mis-parses the modern one.
    pub legacy_sgr: Option<bool>,
    /// Pin which table decides a cluster's width.
    pub width: Option<WidthSource>,
}

impl Overrides {
    /// `--ascii --no-color`, which genuinely travel together.
    ///
    /// The **only** named combination that survives, and the reason it survives is that it is a
    /// constructor rather than a type: nothing reads "is this the plain tier", so nothing can
    /// branch on a name instead of on a fact.
    pub fn plain() -> Overrides {
        Overrides {
            colors: Some(ColorDepth::None),
            glyphs: Some(GlyphSet::Ascii),
            ..Overrides::default()
        }
    }

    /// Fill only the `None`s. The one mechanism the whole precedence order rests on.
    fn fill(&mut self, weaker: Overrides) {
        self.colors = self.colors.or(weaker.colors);
        self.glyphs = self.glyphs.or(weaker.glyphs);
        self.legacy_sgr = self.legacy_sgr.or(weaker.legacy_sgr);
        self.width = self.width.or(weaker.width);
    }
}

/// The environment, read once, as data rather than as a syscall in the middle of a decision.
///
/// A struct instead of `std::env::var` at the point of use, for one reason that is a test rather
/// than a preference: *`Some` is never overridden* has to be asserted **in both directions**, and a
/// test that sets a process-wide variable to assert it is a test that races every other test in the
/// binary.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub(crate) struct Env {
    /// `VITUI_GLYPHS` — `ascii`, `unicode` or `extended`.
    pub(crate) glyphs: Option<String>,
    /// `VITUI_FORCE_COLOR` — `none`/`0`, `16`, `256` or `truecolor`.
    pub(crate) force_color: Option<String>,
    /// `VITUI_FORCE_LEGACY_SGR`.
    pub(crate) force_legacy_sgr: Option<String>,
    /// `VITUI_FORCE_WCWIDTH`.
    pub(crate) force_wcwidth: Option<String>,
    /// `NO_COLOR` — any non-empty value, by its own specification.
    pub(crate) no_color: Option<String>,
    /// `TERM`.
    pub(crate) term: Option<String>,
    /// `COLORTERM` — a cheap fast path and nothing more.
    pub(crate) colorterm: Option<String>,
    /// `TERM_PROGRAM`, which is how the quirk table recognises VSCode's integrated terminal.
    pub(crate) term_program: Option<String>,
    /// `TERMUX_VERSION`, which is how the quirk table recognises Termux.
    pub(crate) termux: Option<String>,
}

impl Env {
    /// Read the process's environment. The one place `std::env` is touched.
    pub(crate) fn from_process() -> Env {
        let var = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        Env {
            glyphs: var("VITUI_GLYPHS"),
            force_color: var("VITUI_FORCE_COLOR"),
            force_legacy_sgr: var("VITUI_FORCE_LEGACY_SGR"),
            force_wcwidth: var("VITUI_FORCE_WCWIDTH"),
            no_color: var("NO_COLOR"),
            term: var("TERM"),
            colorterm: var("COLORTERM"),
            term_program: var("TERM_PROGRAM"),
            termux: var("TERMUX_VERSION"),
        }
    }

    /// Whether `$COLORTERM` claims 24-bit colour. A convention, unstandardised, lost across ssh and
    /// sudo and clobbered by tmux and screen — which is why it is corroborated rather than believed.
    pub(crate) fn claims_truecolor(&self) -> bool {
        matches!(
            self.colorterm.as_deref().map(str::trim),
            Some("truecolor") | Some("24bit") | Some("24-bit")
        )
    }

    /// `TERM=dumb`, which is a terminal saying it speaks no escape sequences at all.
    pub(crate) fn term_is_dumb(&self) -> bool {
        self.term.as_deref() == Some("dumb")
    }

    /// Level 2 of the precedence order, as an [`Overrides`] so that it can only ever fill a `None`.
    fn as_overrides(&self) -> Overrides {
        Overrides {
            colors: self.force_color.as_deref().and_then(parse_depth),
            glyphs: self.glyphs.as_deref().and_then(GlyphSet::parse),
            legacy_sgr: self.force_legacy_sgr.as_deref().map(truthy),
            width: self.force_wcwidth.as_deref().map(|v| {
                if truthy(v) {
                    WidthSource::Wcwidth
                } else {
                    WidthSource::Tables
                }
            }),
        }
    }
}

fn parse_depth(s: &str) -> Option<ColorDepth> {
    match s.trim().to_ascii_lowercase().as_str() {
        "0" | "none" | "off" => Some(ColorDepth::None),
        "16" | "ansi16" => Some(ColorDepth::Ansi16),
        "256" | "indexed256" => Some(ColorDepth::Indexed256),
        "truecolor" | "24bit" | "24-bit" | "rgb" => Some(ColorDepth::TrueColor),
        _ => None,
    }
}

/// Anything but an explicit denial. `VITUI_FORCE_LEGACY_SGR=0` is somebody switching it off.
fn truthy(s: &str) -> bool {
    !matches!(
        s.trim().to_ascii_lowercase().as_str(),
        "" | "0" | "false" | "no" | "off"
    )
}

/// What is on the other end, which decides whether anything is asked at all.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Ground {
    /// A real terminal. The queries go out.
    Tty,
    /// Standard output is a pipe or a file. **This is not headless**: it is a terminal nothing is
    /// known about, so no queries are fired — there is nobody to answer, and the DA1 sentinel would
    /// burn the whole ceiling for nothing.
    NotATty,
    /// A caller-supplied sink. **A fully declared tier, not the lowest one**: detection off, every
    /// axis pinned through [`Overrides`], so *any* tier is testable, truecolor included. A floor
    /// tier could never have provided that.
    Headless,
}

impl Ground {
    pub(crate) fn word(self) -> &'static str {
        match self {
            Ground::Tty => "tty",
            Ground::NotATty => "not-a-tty",
            Ground::Headless => "headless",
        }
    }
}

/// What the live queries came back with. Everything here is an answer or an absence, never a guess.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub(crate) struct Detected {
    /// Whether the DA1 sentinel arrived, which is what ends the exchange early.
    pub(crate) answered: bool,
    /// The identity string XTVERSION reported, if it did.
    pub(crate) version: Option<String>,
    /// DA2's three parameters, the fallback identity probe.
    pub(crate) da2: Option<(u32, u32, u32)>,
    /// XTGETTCAP's answer about `RGB`: `Some(true)` affirmed, `Some(false)` denied, `None` silent.
    /// The three states are not the same and collapsing them is how a terminal loses truecolor.
    pub(crate) rgb: Option<bool>,
    /// OSC 10.
    pub(crate) default_fg: Option<Rgb>,
    /// OSC 11.
    pub(crate) default_bg: Option<Rgb>,
    /// OSC 4, entries 0..16.
    pub(crate) palette: [Option<Rgb>; 16],
    /// The kitty keyboard flags that survived a push of all five, or `None` if the protocol is not
    /// there. **Read back rather than assumed**, which is the whole reason these are four booleans
    /// and not a tier: tmux forwards the `CSI u` encoding while implementing none of the stack.
    pub(crate) kitty_flags: Option<u32>,
    /// DECRQM answers, keyed by mode. A `Pm` of zero means *not recognised*.
    pub(crate) modes: Vec<(u32, u32)>,
}

impl Detected {
    /// Whether **anything** came back, sentinel or not.
    ///
    /// Distinct from [`answered`](Detected::answered) on purpose, and the distinction is a defect
    /// that was fixed here. A terminal can answer OSC 10, OSC 11, sixteen palette entries and every
    /// DECRQM and still lose its DA1 — the *answers and then goes quiet* row detection already
    /// handles. Keying the ambiguous colour case off the sentinel put such a terminal at
    /// `ColorDepth::None`, which is *no colour at all*, on the strength of one missing reply while
    /// it was demonstrably speaking escape sequences and had just reported its real default
    /// foreground. The rule is that what arrived is kept; this is what makes the colour ladder obey
    /// it too.
    pub(crate) fn heard_anything(&self) -> bool {
        self.answered
            || self.version.is_some()
            || self.da2.is_some()
            || self.rgb.is_some()
            || self.default_fg.is_some()
            || self.default_bg.is_some()
            || self.kitty_flags.is_some()
            || !self.modes.is_empty()
            || self.palette.iter().any(Option::is_some)
    }

    /// Whether a DEC private mode is available on this terminal.
    ///
    /// DECRQM answers with five states and only two of them are *no*. `0` is **not recognised** —
    /// the terminal has never heard of the mode. `4` is **permanently reset**: it knows the mode and
    /// will never let it be set, which is a denial and not a current value. `1` and `2` are set and
    /// reset, both of which mean *available*, because the question here is whether the capability
    /// exists and not whether it happens to be on right now. `3` is permanently set.
    ///
    /// Reading `4` as available is the failure worth naming: the engine would send the escape every
    /// frame, the terminal would ignore it every frame, and a component above would branch on a
    /// capability it does not have.
    pub(crate) fn mode(&self, which: u32) -> bool {
        self.modes
            .iter()
            .rev()
            .find(|(m, _)| *m == which)
            .is_some_and(|(_, state)| *state != NOT_RECOGNISED && *state != PERMANENTLY_RESET)
    }
}

/// The facts that are known and that nobody above can act on.
///
/// Every one of them failed the *someone above can act on it* test and none of them is therefore
/// deleted: they decide bytes, and the serializer reads them. They reach a human through
/// [`Capabilities::report`] and reach nothing else.
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct Private {
    pub(crate) ground: Ground,
    pub(crate) identity: Option<String>,
    /// Mode 2026. §8 is where its force-flush limits bind.
    pub(crate) sync_output: bool,
    /// Mode 69, `DECSLRM`. Nothing in the serializer depends on it (§15); it is queried because the
    /// query is free inside the batch and because "we never asked" is a worse answer than "no".
    pub(crate) decslrm: bool,
    /// The pre-ITU-T colon form of SGR 38/48.
    pub(crate) legacy_sgr: bool,
    /// Which escape spells an underline colour.
    pub(crate) underlines: Underlines,
    /// Which table decides a cluster's width.
    pub(crate) width: WidthSource,
    /// Palette entries 0..16, for quantisation at [`ColorDepth::Ansi16`]. On silence the xterm
    /// default table is used — **OSC 4 silence is wrong in degree, where OSC 11 silence would be
    /// wrong in direction**, and that one rule decides both.
    pub(crate) palette: [Option<Rgb>; 16],
    /// Attribute bits this terminal does not render, in the style word's own positions. Dropped
    /// silently at serialise time, because an absent attribute still draws the correct text.
    pub(crate) attrs_dropped: u64,
    /// When this terminal force-flushes an open synchronised-output block.
    pub(crate) sync_flush: Option<SyncFlush>,
}

/// What is true about the terminal on the other end.
///
/// Fourteen public facts, two of them ordered, and **no field that nobody above can act on**. Built
/// during `attach` and **immutable for the life of the `Screen`**, which is why
/// [`Screen::capabilities`](crate::Screen::capabilities) takes `&self` and why no component ever has
/// to handle a capability changing between frames.
///
/// The price of sampling once is explicit rather than discovered: **a terminal that changed
/// underneath the process — a reconnected ssh session, a SIGTSTP/SIGCONT cycle — cannot be
/// re-detected without a fresh `attach`.** That is spec §15's terminal lifecycle, not a degradation
/// question.
///
/// ```
/// let config = vitui_engine::Config {
///     // Headless, because a doctest must not reach for the developer's terminal.
///     output: vitui_engine::Output::Sink(Box::new(Vec::new())),
///     ..Default::default()
/// };
/// let (screen, _wake) = vitui_engine::Engine::new(config).attach().unwrap();
/// // A caller-supplied sink is a declared tier, and nothing was detected.
/// assert_eq!(screen.capabilities().colors, vitui_engine::ColorDepth::None);
/// assert_eq!(screen.capabilities().glyphs, vitui_engine::GlyphSet::Extended);
/// ```
///
/// # Two absences, gated rather than asserted in prose
///
/// **The eleven attribute bits are not here**, and each `compile_fail` below is paired with a
/// positive twin that names a field which *is*: a `compile_fail` alone passes for any reason at
/// all, including the type having been renamed out from under it.
///
/// ```compile_fail,E0609
/// let config = vitui_engine::Config {
///     // Headless, because a doctest must not reach for the developer's terminal.
///     output: vitui_engine::Output::Sink(Box::new(Vec::new())),
///     ..Default::default()
/// };
/// let (screen, _wake) = vitui_engine::Engine::new(config).attach().unwrap();
/// let _ = screen.capabilities().italic;
/// ```
///
/// ```
/// let config = vitui_engine::Config {
///     // Headless, because a doctest must not reach for the developer's terminal.
///     output: vitui_engine::Output::Sink(Box::new(Vec::new())),
///     ..Default::default()
/// };
/// let (screen, _wake) = vitui_engine::Engine::new(config).attach().unwrap();
/// let _: bool = screen.capabilities().hyperlinks;
/// ```
///
/// And **nothing outside the crate can construct one**: this is what `attach` found, never what a
/// caller asserts. [`Overrides`] is the door for asserting, and it is a different type on purpose.
///
/// ```compile_fail,E0451
/// let _ = vitui_engine::Capabilities {
///     colors: vitui_engine::ColorDepth::TrueColor,
///     glyphs: vitui_engine::GlyphSet::Extended,
///     default_fg: None,
///     default_bg: None,
///     hyperlinks: true,
///     grapheme_clusters: true,
///     key_release: true,
///     key_repeat: true,
///     alternate_keys: true,
///     associated_text: true,
///     mouse: true,
///     mouse_motion: true,
///     focus_events: true,
///     bracketed_paste: true,
/// };
/// ```
///
/// ```
/// let _: vitui_engine::Overrides = vitui_engine::Overrides {
///     colors: Some(vitui_engine::ColorDepth::TrueColor),
///     ..Default::default()
/// };
/// ```
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Capabilities {
    /// How much colour reaches the wire. Detected, and a ladder.
    pub colors: ColorDepth,
    /// What may be assumed of the font. Declared, never detected.
    pub glyphs: GlyphSet,
    /// The terminal's real default foreground, from OSC 10.
    pub default_fg: Option<Rgb>,
    /// The terminal's real default background, from OSC 11.
    ///
    /// **`None` is spec §5's silent path and it is load-bearing**: cells with a default background
    /// are left unmixed rather than mixed against a guess, because the guess is a dark theme and on
    /// a light-theme terminal it draws a shadow backwards. A shadow clipped to the explicitly
    /// coloured area is a visible imperfection; an inverted shadow is a bug. See
    /// [ADR 0025](../../../docs/adr/0025-compositing-depends-on-a-terminal-capability.md).
    pub default_bg: Option<Rgb>,
    /// OSC 8 hyperlinks.
    pub hyperlinks: bool,
    /// Mode 2027, requested once at startup and reported here. **Our tables stay authoritative
    /// either way** — this says what the terminal will do with a cluster, not what the engine will.
    pub grapheme_clusters: bool,
    /// Key releases are reported.
    pub key_release: bool,
    /// Auto-repeat is distinguishable from a fresh press.
    pub key_repeat: bool,
    /// The key the physical layout would have produced is reported alongside the shifted one.
    pub alternate_keys: bool,
    /// The text a key press produced is reported alongside the key.
    pub associated_text: bool,
    /// Mouse buttons are reported.
    pub mouse: bool,
    /// Mouse motion is reported, not only clicks.
    pub mouse_motion: bool,
    /// Focus in and focus out are reported.
    pub focus_events: bool,
    /// A paste arrives bracketed rather than as a burst of key presses.
    pub bracketed_paste: bool,
    pub(crate) private: Private,
}

impl Capabilities {
    /// Everything known, including the facts that are deliberately not fields.
    ///
    /// Diagnostics, and the reason there is not a public field per fact. A bug report pastes this;
    /// nothing branches on it, and nothing can, because it is a `String`.
    pub fn report(&self) -> String {
        let p = &self.private;
        let mut out = String::with_capacity(768);
        let _ = writeln!(out, "vitui capabilities ({})", p.ground.word());
        if let Some(id) = &p.identity {
            let _ = writeln!(out, "  identity          {id}");
        }
        let _ = writeln!(out, "  colors            {}", self.colors.word());
        let _ = writeln!(out, "  glyphs            {} (declared)", self.glyphs.word());
        let _ = writeln!(out, "  default_fg        {}", show(self.default_fg));
        let _ = writeln!(out, "  default_bg        {}", show(self.default_bg));
        let _ = writeln!(out, "  hyperlinks        {}", self.hyperlinks);
        let _ = writeln!(out, "  grapheme_clusters {}", self.grapheme_clusters);
        let _ = writeln!(
            out,
            "  keyboard          release {} repeat {} alternate {} text {}",
            self.key_release, self.key_repeat, self.alternate_keys, self.associated_text
        );
        let _ = writeln!(
            out,
            "  pointer           mouse {} motion {} focus {} paste {}",
            self.mouse, self.mouse_motion, self.focus_events, self.bracketed_paste
        );
        let _ = writeln!(out, "  -- not public, nobody above can act on these --");
        let _ = writeln!(out, "  sync_output       {}", p.sync_output);
        let _ = writeln!(out, "  decslrm           {}", p.decslrm);
        let _ = writeln!(out, "  legacy_sgr        {}", p.legacy_sgr);
        let _ = writeln!(out, "  underline_colour  {}", p.underlines.word());
        let _ = writeln!(out, "  width             {}", p.width.word());
        let _ = writeln!(out, "  attrs_dropped     {:#013x}", p.attrs_dropped);
        match p.sync_flush {
            Some(f) => {
                let _ = writeln!(
                    out,
                    "  sync_flush        {} ms {:?} bytes",
                    f.after_ms, f.after_bytes
                );
            }
            None => {
                let _ = writeln!(out, "  sync_flush        -");
            }
        }
        let _ = write!(out, "  palette          ");
        for entry in &p.palette {
            let _ = write!(out, " {}", show(*entry));
        }
        out.push('\n');
        out
    }

    /// Mode 2026, which §8 wraps a frame in when it is there.
    #[allow(dead_code)]
    pub(crate) fn sync_output(&self) -> bool {
        self.private.sync_output
    }

    /// Whether SGR 38/48 must be spelled the pre-ITU-T way.
    #[allow(dead_code)]
    pub(crate) fn legacy_sgr(&self) -> bool {
        self.private.legacy_sgr
    }

    /// Which escape spells an underline colour on this terminal.
    #[allow(dead_code)]
    pub(crate) fn underlines(&self) -> Underlines {
        self.private.underlines
    }

    /// The attribute bits to drop at serialise time, in the style word's own positions.
    #[allow(dead_code)]
    pub(crate) fn attrs_dropped(&self) -> u64 {
        self.private.attrs_dropped
    }

    /// Palette entry `i`, or `None` where the terminal did not say and the xterm table applies.
    #[allow(dead_code)]
    pub(crate) fn palette(&self, i: u8) -> Option<Rgb> {
        self.private.palette.get(i as usize).copied().flatten()
    }

    /// Which table decides a cluster's width.
    #[allow(dead_code)]
    pub(crate) fn width(&self) -> WidthSource {
        self.private.width
    }

    /// When this terminal force-flushes an open synchronised-output block, where it says.
    #[allow(dead_code)]
    pub(crate) fn sync_flush(&self) -> Option<SyncFlush> {
        self.private.sync_flush
    }
}

fn show(c: Option<Rgb>) -> String {
    match c {
        Some(c) => format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b),
        None => "-".to_string(),
    }
}

/// Resolve the seven levels of spec §10's precedence into one immutable answer.
///
/// Highest wins, and the whole order is in the order of the statements below:
///
/// 1. **The explicit API** — `Config::overrides`.
/// 2. **`VITUI_*`** — a lever that does not require a new release, for when detection is wrong in
///    the field.
/// 3. **`NO_COLOR`**, any non-empty value.
/// 4. **`TERM=dumb`, or standard output is not a tty.**
/// 5. **The quirk table, applied *after* detection** — some terminals answer correctly and then
///    misbehave.
/// 6. **Live detection.**
/// 7. **Conservative defaults.**
///
/// Levels 1 to 4 are all the same mechanism, which is the point: each is an [`Overrides`] and each
/// may only **fill a `None`**, so a stale variable can never override a flag the user just typed and
/// `NO_COLOR` keeps its intent — an explicit request through the application's own flag is a *more*
/// specific expression of the same user's wish than the variable is.
///
/// Levels 5 to 7 build the base the forcing is applied on top of.
pub(crate) fn assemble(
    overrides: Overrides,
    env: &Env,
    ground: Ground,
    detected: &Detected,
    quirks: Quirks,
) -> Capabilities {
    // Levels 1 through 4, weakest last, each filling only what is still `None`.
    let mut forced = overrides;
    forced.fill(env.as_overrides());
    if env.no_color.is_some() {
        forced.fill(Overrides {
            colors: Some(ColorDepth::None),
            ..Overrides::default()
        });
    }
    if env.term_is_dumb() || ground != Ground::Tty {
        // Not `glyphs`. `TERM=dumb` and a pipe are both statements about escape sequences and
        // neither is a statement about the font, and lowering an axis nothing measured would be
        // exactly the invention this type exists to refuse.
        forced.fill(Overrides {
            colors: Some(ColorDepth::None),
            ..Overrides::default()
        });
    }

    // Levels 6 and 7: what the queries said, or conservative defaults where they said nothing.
    let speaks_escapes = ground == Ground::Tty && !env.term_is_dumb();
    let mut caps = Capabilities {
        colors: detect_depth(detected, env),
        glyphs: GlyphSet::default(),
        default_fg: detected.default_fg,
        default_bg: detected.default_bg,
        // OSC 8 has no query, so this is the one inference in the whole file and it is written down
        // rather than buried. See `implements_osc8`.
        hyperlinks: implements_osc8(detected.version.as_deref()),
        grapheme_clusters: detected.mode(MODE_GRAPHEME_CLUSTERS),
        key_release: kitty(detected, KITTY_EVENT_TYPES),
        key_repeat: kitty(detected, KITTY_EVENT_TYPES),
        alternate_keys: kitty(detected, KITTY_ALTERNATE_KEYS),
        associated_text: kitty(detected, KITTY_ASSOCIATED_TEXT),
        mouse: detected.mode(MODE_MOUSE),
        mouse_motion: detected.mode(MODE_MOUSE_MOTION),
        focus_events: detected.mode(MODE_FOCUS),
        bracketed_paste: detected.mode(MODE_BRACKETED_PASTE),
        private: Private {
            ground,
            identity: detected
                .version
                .clone()
                .or_else(|| detected.da2.map(|(a, b, c)| format!("DA2 {a};{b};{c}"))),
            sync_output: detected.mode(MODE_SYNC_OUTPUT),
            decslrm: detected.mode(MODE_DECSLRM),
            legacy_sgr: false,
            underlines: Underlines::Standard,
            width: WidthSource::default(),
            palette: detected.palette,
            attrs_dropped: 0,
            sync_flush: None,
        },
    };

    // Level 5: the quirk table, applied **after** detection, because that is the whole reason it
    // exists — a terminal that answers the query correctly and then misbehaves is not a detection
    // failure and cannot be fixed by asking again.
    quirks.apply(&mut caps);

    if !speaks_escapes {
        // A pipe, a file, a sink, or `TERM=dumb`. Nothing was asked and nothing may be assumed.
        caps.hyperlinks = false;
        caps.grapheme_clusters = false;
        caps.key_release = false;
        caps.key_repeat = false;
        caps.alternate_keys = false;
        caps.associated_text = false;
        caps.mouse = false;
        caps.mouse_motion = false;
        caps.focus_events = false;
        caps.bracketed_paste = false;
        caps.private.sync_output = false;
        caps.private.decslrm = false;
    }

    // And levels 1 to 4 on top of all of it.
    if let Some(colors) = forced.colors {
        caps.colors = colors;
    }
    if let Some(glyphs) = forced.glyphs {
        caps.glyphs = glyphs;
    }
    if let Some(legacy) = forced.legacy_sgr {
        caps.private.legacy_sgr = legacy;
    }
    if let Some(width) = forced.width {
        caps.private.width = width;
    }
    caps
}

/// DECRQM's `Pm`: the terminal has never heard of the mode.
pub(crate) const NOT_RECOGNISED: u32 = 0;
/// DECRQM's `Pm`: the terminal knows the mode and will never let it be set.
pub(crate) const PERMANENTLY_RESET: u32 = 4;

/// DEC private modes the batch asks about.
pub(crate) const MODE_MOUSE: u32 = 1000;
pub(crate) const MODE_MOUSE_MOTION: u32 = 1003;
pub(crate) const MODE_FOCUS: u32 = 1004;
pub(crate) const MODE_BRACKETED_PASTE: u32 = 2004;
pub(crate) const MODE_SYNC_OUTPUT: u32 = 2026;
pub(crate) const MODE_GRAPHEME_CLUSTERS: u32 = 2027;
pub(crate) const MODE_DECSLRM: u32 = 69;

/// The kitty keyboard flags, as the protocol numbers them.
pub(crate) const KITTY_DISAMBIGUATE: u32 = 0b0_0001;
pub(crate) const KITTY_EVENT_TYPES: u32 = 0b0_0010;
pub(crate) const KITTY_ALTERNATE_KEYS: u32 = 0b0_0100;
pub(crate) const KITTY_ALL_AS_ESCAPES: u32 = 0b0_1000;
pub(crate) const KITTY_ASSOCIATED_TEXT: u32 = 0b1_0000;
/// Every flag there is, which is what the batch pushes so that the read-back says what stuck.
pub(crate) const KITTY_ALL: u32 = KITTY_DISAMBIGUATE
    | KITTY_EVENT_TYPES
    | KITTY_ALTERNATE_KEYS
    | KITTY_ALL_AS_ESCAPES
    | KITTY_ASSOCIATED_TEXT;

/// Whether this terminal implements OSC 8, inferred from what XTVERSION reported.
///
/// **The one inference in this file, and the first draft of it was wrong.** It read *answered
/// XTVERSION at all* as *implements OSC 8*, on the belief that the sequence belongs to the four
/// terminals that popularised it. It does not: **XTVERSION is xterm's own** — patch #359 — and xterm
/// answers `DCS > | XTerm(NNN) ST` while having no OSC 8 support at all. So the rule produced
/// exactly the wrong `true` its own comment forbade, on the one terminal most likely to be behind a
/// bug report.
///
/// So it is an allow-list of identities rather than a test of whether anything replied, and the
/// failure it can still have is the safe one: a terminal that implements OSC 8 and is not on the
/// list gets a wrong `false`. VTE — gnome-terminal, Tilix — is that case today and answers no
/// XTVERSION to be recognised by, which is what the quirk table is for.
///
/// A wrong `false` costs a hyperlink that is not offered. A wrong `true` costs a component branching
/// on a capability it does not have, and an escape sent every frame that the terminal ignores every
/// frame. The two are not symmetric, and the list leans accordingly.
fn implements_osc8(version: Option<&str>) -> bool {
    let Some(version) = version else {
        return false;
    };
    // XTVERSION answers with a name and a parenthesised version. Match on the name only: the
    // version moves and the name does not.
    let name = version
        .split(['(', ' '])
        .next()
        .unwrap_or(version)
        .trim()
        .to_ascii_lowercase();
    matches!(
        name.as_str(),
        "kitty" | "wezterm" | "foot" | "ghostty" | "rio" | "contour" | "iterm2" | "mintty"
    )
}

fn kitty(detected: &Detected, flag: u32) -> bool {
    detected.kitty_flags.is_some_and(|f| f & flag != 0)
}

/// The colour ladder, from three sources that do not agree and are not equally trustworthy.
///
/// `$COLORTERM` is **a cheap fast path and nothing more** — unstandardised, lost across ssh and
/// sudo, clobbered by tmux and screen — so it is corroborated with a live XTGETTCAP `RGB` query, and
/// the answer is 256 when the two are genuinely ambiguous. The three states of that query are kept
/// apart deliberately: silence means *this terminal does not implement XTGETTCAP*, which is not the
/// same claim as *this terminal has no truecolor*, and collapsing them is how a terminal loses it.
fn detect_depth(detected: &Detected, env: &Env) -> ColorDepth {
    match (detected.rgb, env.claims_truecolor()) {
        // Affirmed by the terminal itself. Nothing else is consulted.
        (Some(true), _) => ColorDepth::TrueColor,
        // Denied by the terminal itself. `$COLORTERM` loses, which is the entire point of asking.
        (Some(false), _) => ColorDepth::Indexed256,
        // Silent, but the variable claims it and nothing contradicts it.
        (None, true) => ColorDepth::TrueColor,
        // Genuinely ambiguous, and something came back: 256, which every such terminal speaks.
        (None, false) if detected.heard_anything() => ColorDepth::Indexed256,
        // Nothing answered at all.
        (None, false) => ColorDepth::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A terminal that answered everything, so that a test about precedence is a test about
    /// precedence rather than about what happens when nothing is known.
    fn modern() -> Detected {
        Detected {
            answered: true,
            version: Some("kitty(0.32.2)".to_string()),
            da2: Some((1, 4000, 19)),
            rgb: Some(true),
            default_fg: Some(Rgb::new(0xc5, 0xc8, 0xc6)),
            default_bg: Some(Rgb::new(0x1d, 0x1f, 0x21)),
            palette: [Some(Rgb::new(1, 2, 3)); 16],
            kitty_flags: Some(KITTY_ALL),
            modes: vec![
                (MODE_GRAPHEME_CLUSTERS, 1),
                (MODE_SYNC_OUTPUT, 2),
                (MODE_MOUSE, 2),
                (MODE_MOUSE_MOTION, 2),
                (MODE_FOCUS, 2),
                (MODE_BRACKETED_PASTE, 2),
                (MODE_DECSLRM, 0),
            ],
        }
    }

    fn on_a_tty(overrides: Overrides, env: Env) -> Capabilities {
        assemble(
            overrides,
            &env,
            Ground::Tty,
            &modern(),
            Quirks::lookup(Some("kitty(0.32.2)"), &env),
        )
    }

    /// DECRQM has five answers and only two of them are *no*.
    ///
    /// `0` is *never heard of it* and `4` is *knows it and will never allow it* — a denial rather
    /// than a current value. `1` and `2` are set and reset, and both mean **available**, because the
    /// question is whether the capability exists and not whether it is on right now.
    ///
    /// Reading `4` as available is the failure worth naming: the engine would send the escape every
    /// frame, the terminal would ignore it every frame, and a component would branch on a capability
    /// it does not have.
    #[test]
    fn a_permanently_reset_mode_is_a_denial_and_not_a_value() {
        let asked = |state: u32| {
            let detected = Detected {
                modes: vec![(MODE_MOUSE, state)],
                answered: true,
                ..Detected::default()
            };
            assemble(
                Overrides::default(),
                &Env::default(),
                Ground::Tty,
                &detected,
                Quirks::default(),
            )
            .mouse
        };
        assert!(!asked(NOT_RECOGNISED), "0 is never heard of it");
        assert!(asked(1), "1 is set, and set is available");
        assert!(asked(2), "2 is reset, and reset is still available");
        assert!(asked(3), "3 is permanently set");
        assert!(!asked(PERMANENTLY_RESET), "4 is a denial");
    }

    #[test]
    fn a_terminal_that_answers_everything_is_read_at_face_value() {
        let caps = on_a_tty(Overrides::default(), Env::default());
        assert_eq!(caps.colors, ColorDepth::TrueColor);
        assert_eq!(
            caps.glyphs,
            GlyphSet::Extended,
            "declared, and the default is Extended"
        );
        assert_eq!(caps.default_bg, Some(Rgb::new(0x1d, 0x1f, 0x21)));
        assert!(caps.hyperlinks);
        assert!(caps.grapheme_clusters);
        assert!(caps.key_release && caps.key_repeat && caps.alternate_keys && caps.associated_text);
        assert!(caps.mouse && caps.mouse_motion && caps.focus_events && caps.bracketed_paste);
        assert!(caps.sync_output());
        assert!(!caps.private.decslrm, "a Pm of zero is not a capability");
    }

    /// **Gate, and it is the one contested pair in the whole order.** `Some` is never overridden by
    /// any environment variable, **in both directions** — the second half is what makes it a gate
    /// rather than a coincidence, because a rule that only ever lowers would pass the first half by
    /// accident.
    #[test]
    fn a_some_in_overrides_is_never_overridden_by_the_environment() {
        let env = Env {
            glyphs: Some("ascii".to_string()),
            force_color: Some("16".to_string()),
            ..Env::default()
        };

        // The variable would lower; the API says otherwise, and the API is level 1.
        let held = on_a_tty(
            Overrides {
                glyphs: Some(GlyphSet::Extended),
                colors: Some(ColorDepth::TrueColor),
                ..Overrides::default()
            },
            env.clone(),
        );
        assert_eq!(held.glyphs, GlyphSet::Extended);
        assert_eq!(held.colors, ColorDepth::TrueColor);

        // And the other direction: the variable would *raise* and the API lowers.
        let raised = Env {
            glyphs: Some("extended".to_string()),
            force_color: Some("truecolor".to_string()),
            ..Env::default()
        };
        let lowered = on_a_tty(Overrides::plain(), raised);
        assert_eq!(lowered.glyphs, GlyphSet::Ascii);
        assert_eq!(lowered.colors, ColorDepth::None);
    }

    /// The environment fills only the `None`s — which is the same statement seen from the other
    /// side, and the reason a variable is still worth having.
    #[test]
    fn the_environment_fills_a_none_and_only_a_none() {
        let env = Env {
            glyphs: Some("unicode".to_string()),
            force_legacy_sgr: Some("1".to_string()),
            force_wcwidth: Some("yes".to_string()),
            ..Env::default()
        };
        let caps = on_a_tty(
            Overrides {
                colors: Some(ColorDepth::Ansi16),
                ..Overrides::default()
            },
            env,
        );
        assert_eq!(caps.colors, ColorDepth::Ansi16, "the API's Some");
        assert_eq!(caps.glyphs, GlyphSet::Unicode, "the variable filled a None");
        assert!(caps.legacy_sgr());
        assert_eq!(caps.width(), WidthSource::Wcwidth);
    }

    /// `NO_COLOR` is level 3: under both `VITUI_*` and the API, over detection.
    #[test]
    fn no_color_sits_under_the_api_and_under_the_vitui_variables() {
        let bare = Env {
            no_color: Some("1".to_string()),
            ..Env::default()
        };
        assert_eq!(
            on_a_tty(Overrides::default(), bare.clone()).colors,
            ColorDepth::None,
            "over detection"
        );
        assert_eq!(
            on_a_tty(
                Overrides {
                    colors: Some(ColorDepth::TrueColor),
                    ..Overrides::default()
                },
                bare.clone()
            )
            .colors,
            ColorDepth::TrueColor,
            "an explicit request through the application's own flag is the same user, more specific"
        );
        let with_variable = Env {
            force_color: Some("256".to_string()),
            ..bare
        };
        assert_eq!(
            on_a_tty(Overrides::default(), with_variable).colors,
            ColorDepth::Indexed256,
            "`VITUI_FORCE_COLOR` is level 2 and `NO_COLOR` is level 3"
        );
    }

    /// `TERM=dumb` is a claim about escape sequences and **is not a claim about the font**.
    /// Lowering an axis nothing measured would be exactly the invention `GlyphSet` exists to refuse.
    #[test]
    fn term_dumb_takes_the_colour_and_leaves_the_font_alone() {
        let env = Env {
            term: Some("dumb".to_string()),
            ..Env::default()
        };
        let caps = on_a_tty(Overrides::default(), env);
        assert_eq!(caps.colors, ColorDepth::None);
        assert_eq!(caps.glyphs, GlyphSet::Extended);
        assert!(!caps.hyperlinks);
        assert!(!caps.mouse && !caps.bracketed_paste);
        assert!(!caps.sync_output());
    }

    /// A non-tty is **not headless**: it is a terminal nothing is known about, so nothing was asked
    /// and everything falls through to the declared defaults — escape codes in a log are noise, and
    /// lowering the glyph set unasked would be an invention.
    #[test]
    fn a_non_tty_is_a_terminal_nothing_is_known_about() {
        let caps = assemble(
            Overrides::default(),
            &Env::default(),
            Ground::NotATty,
            &Detected::default(),
            Quirks::default(),
        );
        assert_eq!(caps.colors, ColorDepth::None);
        assert_eq!(caps.glyphs, GlyphSet::Extended);
        assert!(!caps.sync_output());
        assert_eq!(
            caps.default_bg, None,
            "and §5's silent path, which is the honest one"
        );
        assert!(caps.report().contains("not-a-tty"));
    }

    /// **Gate.** Headless is a fully *declared* tier, not the lowest one: truecolor is reachable
    /// with detection switched off. A floor tier could never have provided this, and it is what
    /// ticket 05's goldens and the round trip's option sets need.
    #[test]
    fn headless_reaches_truecolor_with_detection_off() {
        let caps = assemble(
            Overrides {
                colors: Some(ColorDepth::TrueColor),
                glyphs: Some(GlyphSet::Extended),
                legacy_sgr: Some(false),
                width: Some(WidthSource::Tables),
            },
            &Env::default(),
            Ground::Headless,
            &Detected::default(),
            Quirks::default(),
        );
        assert_eq!(caps.colors, ColorDepth::TrueColor);
        assert_eq!(caps.glyphs, GlyphSet::Extended);
        assert!(!caps.legacy_sgr());
        assert!(caps.report().contains("headless"));
    }

    /// **Gate.** The quirk table is applied *after* detection, which is the whole reason it exists:
    /// a terminal that answers the query correctly and then misbehaves is not a detection failure.
    #[test]
    fn the_quirk_table_is_applied_after_detection() {
        let env = Env {
            term_program: Some("vscode".to_string()),
            ..Env::default()
        };
        let caps = assemble(
            Overrides::default(),
            &env,
            Ground::Tty,
            &modern(),
            Quirks::lookup(Some("kitty(0.32.2)"), &env),
        );
        assert!(
            caps.legacy_sgr(),
            "detection said nothing about SGR and the table is what decides it"
        );
        assert_eq!(
            caps.colors,
            ColorDepth::TrueColor,
            "and it touched nothing else"
        );

        // Level 1 still wins over level 5, in the direction that matters: switching it back off.
        let forced = assemble(
            Overrides {
                legacy_sgr: Some(false),
                ..Overrides::default()
            },
            &env,
            Ground::Tty,
            &modern(),
            Quirks::lookup(Some("kitty(0.32.2)"), &env),
        );
        assert!(!forced.legacy_sgr());
    }

    /// `$COLORTERM` is a cheap fast path and nothing more: it is corroborated, and it loses to a
    /// denial from the terminal itself. The three states of the `RGB` query are why.
    #[test]
    fn colorterm_is_corroborated_and_loses_to_a_denial() {
        let claiming = Env {
            colorterm: Some("truecolor".to_string()),
            ..Env::default()
        };
        let depth = |rgb: Option<bool>, env: &Env| {
            let detected = Detected { rgb, ..modern() };
            assemble(
                Overrides::default(),
                env,
                Ground::Tty,
                &detected,
                Quirks::default(),
            )
            .colors
        };
        assert_eq!(depth(Some(true), &Env::default()), ColorDepth::TrueColor);
        assert_eq!(
            depth(Some(false), &claiming),
            ColorDepth::Indexed256,
            "the terminal denied it and the variable does not get a vote"
        );
        assert_eq!(
            depth(None, &claiming),
            ColorDepth::TrueColor,
            "silent is not a denial, and the fast path is uncontradicted"
        );
        assert_eq!(
            depth(None, &Env::default()),
            ColorDepth::Indexed256,
            "genuinely ambiguous, and 256 is what every such terminal speaks"
        );
    }

    /// **XTVERSION is xterm's own sequence, and the first draft of this inference forgot it.**
    ///
    /// Reading *answered XTVERSION at all* as *implements OSC 8* produced the wrong `true` its own
    /// comment forbade, on xterm — the terminal most likely to be behind a bug report about it. The
    /// list leans the safe way: a wrong `false` costs a hyperlink that is not offered, a wrong `true`
    /// costs a component branching on a capability it does not have.
    #[test]
    fn osc8_is_an_allow_list_and_xterm_is_not_on_it() {
        let asked = |version: &str| {
            let detected = Detected {
                version: Some(version.to_string()),
                ..modern()
            };
            assemble(
                Overrides::default(),
                &Env::default(),
                Ground::Tty,
                &detected,
                Quirks::default(),
            )
            .hyperlinks
        };
        assert!(
            !asked("XTerm(390)"),
            "xterm answers XTVERSION and has no OSC 8"
        );
        assert!(
            !asked("xterm(370)"),
            "and the match is on the name, case-insensitively"
        );
        for yes in [
            "kitty(0.32.2)",
            "WezTerm(20240203)",
            "foot(1.16.2)",
            "ghostty 1.0.0",
            "iTerm2(3.5)",
        ] {
            assert!(asked(yes), "{yes} implements OSC 8 and was not recognised");
        }
        assert!(
            !asked("SomeTerminalNobodyHasHeardOf(1)"),
            "an unknown identity gets the safe answer"
        );

        // And silence is false, which is the same safe answer reached from the other side.
        let silent = Detected {
            version: None,
            ..modern()
        };
        assert!(
            !assemble(
                Overrides::default(),
                &Env::default(),
                Ground::Tty,
                &silent,
                Quirks::default()
            )
            .hyperlinks
        );
    }

    /// **A lost DA1 must not cost a terminal its colour.**
    ///
    /// Keying the ambiguous arm of the colour ladder off the *sentinel* meant a terminal that
    /// answered OSC 10, OSC 11, sixteen palette entries and every DECRQM — but whose DA1 went
    /// missing, which is the *answers and then goes quiet* row detection already handles — landed on
    /// `ColorDepth::None`. No colour at all, on the strength of one missing reply, from something
    /// demonstrably speaking escape sequences. The rule is that what arrived is kept.
    #[test]
    fn a_terminal_that_answered_everything_but_da1_still_has_colour() {
        let no_sentinel = Detected {
            answered: false,
            rgb: None,
            ..modern()
        };
        assert!(
            no_sentinel.heard_anything(),
            "it plainly said several things"
        );
        assert_eq!(
            assemble(
                Overrides::default(),
                &Env::default(),
                Ground::Tty,
                &no_sentinel,
                Quirks::default()
            )
            .colors,
            ColorDepth::Indexed256,
        );

        // And a terminal that said nothing at all is still `None`, which is what the arm is for.
        let mute = Detected::default();
        assert!(!mute.heard_anything());
        assert_eq!(
            assemble(
                Overrides::default(),
                &Env::default(),
                Ground::Tty,
                &mute,
                Quirks::default()
            )
            .colors,
            ColorDepth::None,
        );
    }

    /// The facts that failed the *someone above can act on it* test are not deleted — they are
    /// private, and `report` is the one door they reach a human through.
    #[test]
    fn report_names_the_facts_that_are_deliberately_not_fields() {
        let report = on_a_tty(Overrides::default(), Env::default()).report();
        for fact in [
            "sync_output",
            "decslrm",
            "legacy_sgr",
            "underline_colour",
            "attrs_dropped",
            "sync_flush",
            "palette",
            "identity",
        ] {
            assert!(
                report.contains(fact),
                "`report` says nothing about {fact}:\n{report}"
            );
        }
        assert!(report.contains("kitty(0.32.2)"));
    }

    /// The eleven attribute bits are not on `Capabilities`, because there is no query for any of
    /// them and eleven booleans would be eleven inventions. They live in the quirk table; their
    /// absence from the public surface is gated on [`Capabilities`] itself, where rustdoc runs it.
    #[test]
    fn the_attribute_bits_are_not_capabilities() {
        let caps = on_a_tty(Overrides::default(), Env::default());
        assert_eq!(
            caps.attrs_dropped(),
            0,
            "nothing is dropped until a quirk says so"
        );
    }

    #[test]
    fn plain_is_ascii_and_no_colour_and_nothing_else() {
        let plain = Overrides::plain();
        assert_eq!(plain.colors, Some(ColorDepth::None));
        assert_eq!(plain.glyphs, Some(GlyphSet::Ascii));
        assert_eq!(
            plain.legacy_sgr, None,
            "`--ascii --no-color` says nothing about SGR"
        );
        assert_eq!(plain.width, None);
    }

    /// The ladder is ordered, and it is the only public axis that is — which is what lets a caller
    /// write `caps.colors >= ColorDepth::Indexed256` and what a flat axis must never allow.
    #[test]
    fn the_two_ordered_axes_are_the_only_ordered_axes() {
        assert!(ColorDepth::None < ColorDepth::Ansi16);
        assert!(ColorDepth::Ansi16 < ColorDepth::Indexed256);
        assert!(ColorDepth::Indexed256 < ColorDepth::TrueColor);
        assert!(GlyphSet::Ascii < GlyphSet::Unicode);
        assert!(GlyphSet::Unicode < GlyphSet::Extended);
    }
}
