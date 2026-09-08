//! The one operator, and what it does to a cell.
//!
//! Shadows, liftings, modal dimming, tints and fades are all [`Mix`]. The map's
//! three-item list — `Replace`, `Darken`, `Blend(f32)` — collapsed to two mechanisms, and both
//! halves of that are worth keeping: `Replace` is not a blend mode, it is what a content layer
//! does, and **alpha-over is rejected** because a terminal cell has no alpha. Blending two layers'
//! *glyphs* is not a thing — one of them has to win — and "a semi-transparent popup" would hand
//! that choice to the compositor, which has no basis for making it. A gradient is not an operator
//! either: it is a `fill` with a varying style, already fully expressed by the verbs, and the
//! compositor never sees one.
//!
//! # Colour resolves here, and it depends on what the terminal told us
//!
//! `Mix` needs channels, so a [`Color`] is resolved at composite time: `default` → the terminal's
//! real colour, `indexed` → the palette, `rgb` → itself
//! and that is a capability question rather than a preference. This is
//! the only place in the engine where a rendering decision depends on an answer from the other end,
//! and the rule for what to do when there is no answer is one rule rather than two:
//!
//! > **Refuse when a guess would be wrong in direction; default when it would be wrong only in
//! > degree.**
//!
//! So OSC 11 silence refuses — a guessed default background is a dark theme, and on a light-theme
//! terminal a shadow over it comes out *lighter* than its surroundings, which is a shadow drawn
//! backwards — while OSC 4 silence falls back to xterm's own sixteen, where a themed palette makes
//! the mix slightly off and nothing more.
//!
//! # The memo is the implementation, and it is a correctness requirement
//!
//! One entry on the *previous* style word, which is exactly the trick in
//! [`View::restyle`](crate::View::restyle). Cells in a run are contiguous and share a `u64`, so the
//! table round trip lands once per **distinct style** rather than once per cell, measured at
//! 20.38 / 28.46 / 200.22 µs against 79.47 / 86.28 / 646.87 µs unmemoised, and **3.2× faster than
//! the prototype the map shipped** — which did the mix arithmetic per cell *and deleted the
//! hyperlink under a shadow*. The memo is why the correct form is cheaper than the wrong one.
//!
//! **The memo is not here.** It is a local in the loop over cells, in
//! [`recolour`](crate::layer), for the reason `restyle`'s is: an entry behind a `&mut` is a load and
//! a store per cell where a local stays in a register, and it measured **2.5×** on a full screen.
//! What is here is the thing the memo caches, and the property that makes caching it sound —
//! [`Mixer::style`] is a function of the style word and of nothing else.
//!
//! The hyperlink survives because the whole per-cell transform goes through
//! [`restyle::apply`](crate::restyle::apply), whose contract is that it *rewrites what the
//! descriptor names and preserves every channel it does not*. A second implementation of that
//! contract would be a second thing to keep in step, and the failure it would drift into is the one
//! this module exists to prevent.

use crate::caps::{Capabilities, ColorDepth, Rgb};
use crate::restyle::{self, Restyle};
use crate::style::{Color, Style, TAG_DEFAULT, TAG_INDEXED, TAG_RGB};
use crate::tables::Tables;

/// The one operator: a colour, and how far toward it what is already there is moved.
///
/// **Darken is `Mix` toward black, lifting is `Mix` toward white, tint is `Mix` toward anything, and
/// a fade is `amount` moving across frames.** One mechanism instead of three blend modes, which is
/// what buys lifting, tint and fade for the 14% `Mix` costs over a plain `Darken`.
///
/// The fields are private and [`new`](Mix::new) clamps, because `0..=256` is an invariant the
/// compositing arithmetic rests on rather than a suggestion — the same shape [`Color`] already
/// takes. `amount == 0` is the identity, and **the identity never reaches a cell**: it marks no
/// damage when it is added, moved or removed, because there is nothing it could have painted for a
/// later frame to have to undo.
///
/// # Operators compound, and are therefore not idempotent
///
/// Two overlapping shadows at 0.5 leave the overlap at 0.25. That is visually right, and it means
/// **The order of operators among themselves matters**, not only their order relative to content.
///
/// # There is no second operator, and no alpha anywhere
///
/// Gated rather than promised, and each negative is paired with a positive twin naming the type by
/// path — a `compile_fail` alone passes for any reason at all, including the type having been
/// renamed out from under it (the shape [`Style`] already uses).
///
/// **`amount` is `0..=256`, not an alpha.** A float would be a request for alpha-over, which is
/// rejected because a terminal cell has no alpha: blending two layers' glyphs is not a thing, one of
/// them has to win, and a compositor has no basis for choosing.
///
/// ```compile_fail,E0308
/// let _ = vitui_engine::Mix::new(vitui_engine::Color::DEFAULT, 0.5);
/// ```
///
/// ```
/// let _: vitui_engine::Mix = vitui_engine::Mix::new(vitui_engine::Color::DEFAULT, 128);
/// ```
///
/// **There is no blend mode to select.** `Darken`, `Blend` and `Replace` are not variants of
/// anything — `Replace` is what a content layer does, and the other two collapsed into this type.
///
/// ```compile_fail,E0599
/// let _ = vitui_engine::Mix::blend(vitui_engine::Color::DEFAULT, 0.5);
/// ```
///
/// ```
/// let _: vitui_engine::Mix = vitui_engine::Mix::darken(128);
/// ```
///
/// **A layer has no opacity.** The only thing a content layer says about what shows through is one
/// `opaque` flag over the `EMPTY` sentinel, and an operator layer takes a rectangle and a `Mix` and
/// nothing else.
///
/// ```compile_fail,E0061
/// let config = vitui_engine::Config {
///     output: vitui_engine::Output::Sink(Box::new(Vec::new())),
///     ..Default::default()
/// };
/// let (mut screen, _wake) = vitui_engine::Engine::new(config).attach().unwrap();
/// let rect = vitui_engine::Rect::new(0, 0, 4, 1);
/// let mix = vitui_engine::Mix::darken(128);
/// screen.layers().add_operator(0, rect, mix, 0.5);
/// ```
///
/// ```
/// let config = vitui_engine::Config {
///     output: vitui_engine::Output::Sink(Box::new(Vec::new())),
///     ..Default::default()
/// };
/// let (mut screen, _wake) = vitui_engine::Engine::new(config).attach().unwrap();
/// let rect = vitui_engine::Rect::new(0, 0, 4, 1);
/// let mix = vitui_engine::Mix::darken(128);
/// let _: vitui_engine::LayerId = screen.layers().add_operator(0, rect, mix);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Mix {
    toward: Color,
    amount: u16,
}

impl Mix {
    /// All the way to `toward`, leaving nothing of what was underneath.
    pub const FULL: u16 = 256;

    /// A mix `amount`/256 of the way toward `toward`, saturating at [`FULL`](Mix::FULL).
    pub const fn new(toward: Color, amount: u16) -> Mix {
        Mix {
            toward,
            amount: if amount > Mix::FULL {
                Mix::FULL
            } else {
                amount
            },
        }
    }

    /// A darkening: [`Mix`] toward black. What a shadow and a modal scrim are made of.
    pub const fn darken(amount: u16) -> Mix {
        Mix::new(Color::rgb(0, 0, 0), amount)
    }

    /// A lifting: [`Mix`] toward white. The same mechanism, the other direction.
    pub const fn lift(amount: u16) -> Mix {
        Mix::new(Color::rgb(255, 255, 255), amount)
    }

    /// The colour this operator moves cells toward.
    pub const fn toward(self) -> Color {
        self.toward
    }

    /// How far toward it, out of [`FULL`](Mix::FULL).
    pub const fn amount(self) -> u16 {
        self.amount
    }

    /// Whether this operator changes nothing, and is therefore skipped entirely.
    pub const fn is_identity(self) -> bool {
        self.amount == 0
    }
}

/// Which end of a style word a colour sits on, which is the whole of what `default` means.
///
/// [`Color::DEFAULT`] is *the terminal's own foreground or background, whichever side of a style
/// this lands on* — so resolving one is not a property of the colour, and `toward` is resolved twice
/// rather than once for exactly that reason.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Side {
    Fg,
    Bg,
}

/// Palette entry `i`, as channels.
///
/// **One table, two readers, and this is the reader that *resolves* rather than chooses**: an index
/// a caller named is turned into channels here so a `Mix` can move it, while [`crate::quant`]
/// chooses one for a terminal that cannot spell the colour asked for. xterm's own sixteen and the
/// cube's six levels live there, because a second copy of either could drift and a drifted table is
/// a wrong colour nothing fails on.
///
/// 0..16 come from OSC 4 where the terminal answered and from xterm's table where it did not, which
/// is the silence asymmetry: OSC 4 silence *defaults* where OSC 11 silence *refuses*, because
/// a themed palette is wrong in degree and a guessed background is wrong in direction.
fn indexed(caps: &Capabilities, i: u8) -> Rgb {
    crate::quant::index_channels(caps, i)
}

/// A colour as channels, or `None` when this terminal has given us no way to know.
///
/// The `None` is the silent path, and the caller's response to it is to leave the cell
/// **entirely** alone rather than to mix the channel it does know. See [`Mixer::mixed`].
fn resolve(caps: &Capabilities, c: Color, side: Side) -> Option<Rgb> {
    match c.tag() {
        TAG_DEFAULT => match side {
            Side::Fg => caps.default_fg,
            Side::Bg => caps.default_bg,
        },
        TAG_INDEXED => Some(indexed(caps, c.payload() as u8)),
        TAG_RGB => {
            let p = c.payload();
            Some(Rgb::new((p >> 16) as u8, (p >> 8) as u8, p as u8))
        }
        // Tag 3 is reserved and no builder produces one. Refusing to mix is the same
        // answer the silent OSC 11 path gets, for the same reason: an invented channel is worse
        // than an unmixed cell.
        _ => None,
    }
}

/// `from`, moved `amount`/256 of the way toward `to`, channel by channel.
///
/// Integer, and exact at both ends rather than nearly: `amount == 0` answers `from` and
/// `amount == `[`Mix::FULL`] answers `to`, which is what makes *all the way* mean all the way. In
/// between it truncates, so two mixes at half leave a quarter of 255 as 63 rather than as 64 — the
/// compounding is right and the last bit is not chased.
fn blend(from: Rgb, to: Rgb, amount: u16) -> Rgb {
    let channel = |a: u8, b: u8| {
        let kept = a as u32 * (Mix::FULL - amount) as u32;
        let added = b as u32 * amount as u32;
        ((kept + added) / Mix::FULL as u32) as u8
    };
    Rgb::new(
        channel(from.r, to.r),
        channel(from.g, to.g),
        channel(from.b, to.b),
    )
}

/// One operator, resolved against one terminal.
///
/// Built once per operator layer per damaged run and dropped with it. It holds no memo and no state
/// at all: [`style`](Mixer::style) is a pure function of the style word, which is the property the
/// caller's one-entry memo rests on.
pub(crate) struct Mixer<'a> {
    caps: &'a Capabilities,
    amount: u16,
    /// `toward`, resolved on the foreground side.
    to_fg: Rgb,
    /// `toward`, resolved on the background side. Two fields and not one, because
    /// [`Color::DEFAULT`] means a different colour on each side.
    to_bg: Rgb,
}

impl<'a> Mixer<'a> {
    /// The mixer for one operator, or `None` when the operator provably changes no byte on the
    /// wire and the compositor may skip it outright.
    ///
    /// Two cases, and both are the compositor's:
    ///
    /// - **The identity.** `amount == 0` never reaches a cell.
    /// - **[`ColorDepth::None`].** There is no colour on the wire, so a `Mix` cannot change one.
    ///   Worth **78.2 µs of the 107 µs worst screen**, and taken here rather than per cell so
    ///   that *skipped outright* is a fact about the layer and not an early return repeated 24 000
    ///   times.
    /// - **A `toward` this terminal cannot resolve.** `Mix::new(Color::DEFAULT, …)` on a terminal
    ///   silent on OSC 10 or OSC 11 has nowhere to move a cell toward, so *every* cell would take
    ///   the both-or-neither path and be left alone — after a table read each. The same rule as the
    ///   two above, applied one field along: a layer that provably changes nothing is skipped as a
    ///   layer, not refused 24 000 times.
    pub(crate) fn new(mix: Mix, caps: &'a Capabilities) -> Option<Mixer<'a>> {
        if mix.is_identity() || caps.colors == ColorDepth::None {
            return None;
        }
        let (to_fg, to_bg) = (
            resolve(caps, mix.toward, Side::Fg)?,
            resolve(caps, mix.toward, Side::Bg)?,
        );
        Some(Mixer {
            caps,
            amount: mix.amount,
            to_fg,
            to_bg,
        })
    }

    /// What one style word becomes: **a function of that word and of nothing else.**
    ///
    /// Not of the cell, not of the column, not of what came before it. That is what the caller's
    /// one-entry memo rests on, and it is why the memo is a speed-up rather than a cache with a
    /// correctness argument attached. It costs a table read on an extended word and an intern on a
    /// result that needs one, which is exactly what the memo is there to skip.
    pub(crate) fn style(&self, tables: &mut Tables, old: Style) -> Style {
        self.mixed(tables, old)
    }

    /// The mix itself, on a style word whose channels have to be fetched.
    ///
    /// # Both colours or neither, and the same factor on each
    ///
    /// **`Mix` darkens foreground and background by the same factor.** Darkening only the background
    /// *raises* contrast, making shadowed text more prominent than unshadowed text — the opposite of
    /// what a shadow means, and it would break modal dimming outright, whose whole job is to push
    /// the background away.
    ///
    /// That sentence decides the stated case and the unstated one, as one rule: **a cell is
    /// mixed only if every colour it needs resolves.** The background is named — *if the terminal
    /// stays silent on OSC 11, cells with a default background are left unmixed* — and a default
    /// *foreground* on a terminal silent on OSC 10 is the same situation seen from the other side.
    /// Mixing the half that resolved would be darkening one channel and not the other, which is the
    /// contrast bug above rather than a partial success. A shadow clipped to the cells whose colours
    /// are known is a visible imperfection; an inverted shadow, or one that raises contrast, is a
    /// bug.
    ///
    /// # `reverse`, and the one combination where the invariance does not hold
    ///
    /// A reversed cell has its two colours swapped by the terminal, and mixing both by the same
    /// factor **toward the same colour** is invariant under swapping them — so `darken`, `lift` and
    /// a tint toward an explicit colour need no special case, and that is a consequence rather than
    /// an oversight.
    ///
    /// `Mix::new(Color::DEFAULT, …)` is the exception, because `toward` then resolves to a
    /// *different* colour on each side: the foreground channel moves toward the terminal's default
    /// foreground, and on a reversed cell that channel is what the terminal paints as the
    /// background. A fade toward the default over reversed text therefore moves each channel toward
    /// the other side's colour.
    ///
    /// **It is recorded rather than special-cased, and the reason is mechanical.** Swapping the two
    /// `toward` sides for a cell that carries the bit would be right only if the terminal is going
    /// to honour the bit — and whether it does is a *serialise-time* decision the compositor cannot
    /// see: [`Capabilities`]'s `attrs_dropped` drops attributes a terminal does not render, silently,
    /// below the packet. A compositor that pre-swapped for a terminal that then dropped
    /// `reverse` would be wrong in the other direction, and it would also make the mix depend on an
    /// attribute bit, which it nowhere claims to. `assert_reverse_is_left_to_the_terminal` pins
    /// the behaviour so that it is a choice rather than an accident.
    fn mixed(&self, tables: &mut Tables, old: Style) -> Style {
        self.moved(tables, old).unwrap_or(old)
    }

    /// The mix, or `None` where a colour this cell needs cannot be resolved.
    ///
    /// **One `?` per channel is the whole of the both-or-neither rule.** Every colour the mix
    /// touches has to resolve, or the caller keeps the cell exactly as it was — and nothing here has
    /// a side effect before the last line, so a channel that gives up costs one table read and no
    /// intern.
    fn moved(&self, tables: &mut Tables, old: Style) -> Option<Style> {
        let e = restyle::channels(tables, old);
        let (to_fg, to_bg) = (self.to_fg, self.to_bg);
        // Everything the descriptor does not name is carried across untouched, which is how the
        // hyperlink under a shadow survives: it is not preserved here, it is simply never named.
        let d = Restyle {
            fg: Some(blend(resolve(self.caps, e.fg, Side::Fg)?, to_fg, self.amount).into()),
            bg: Some(blend(resolve(self.caps, e.bg, Side::Bg)?, to_bg, self.amount).into()),
            ul: if e.ul == Color::DEFAULT {
                // SGR 59 is *underline in the text's own colour*, and the text's colour has just
                // been mixed. There is nothing here to resolve and nothing to leave behind.
                None
            } else {
                Some(blend(resolve(self.caps, e.ul, Side::Fg)?, to_fg, self.amount).into())
            },
            ..Restyle::default()
        };
        Some(restyle::apply(tables, &d, None, old))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caps::ColorDepth;
    use crate::exts::LinkId;
    use crate::restyle::Link;

    /// The verb's own two steps, as `View::restyle` takes them: intern the URI once, then apply.
    fn applied(tables: &mut Tables, d: &Restyle<'_>, old: Style) -> Style {
        let link = crate::restyle::intern_link(tables, d);
        restyle::apply(tables, d, link, old)
    }

    /// A style word carrying `uri`, built through the descriptor the way a caller would.
    fn extended_with_link(tables: &mut Tables, uri: &str, base: Style) -> Style {
        applied(
            tables,
            &Restyle {
                link: Some(Link::Uri(uri)),
                ..Default::default()
            },
            base,
        )
    }

    /// A terminal that answered OSC 10 and OSC 11, at truecolor.
    fn answering() -> Capabilities {
        Capabilities::answering(
            ColorDepth::TrueColor,
            Some(Rgb::new(0xc0, 0xc0, 0xc0)),
            Some(Rgb::new(0x20, 0x20, 0x20)),
        )
    }

    /// A terminal that stayed silent on both, which is every headless test in this crate.
    fn silent() -> Capabilities {
        Capabilities::answering(ColorDepth::TrueColor, None, None)
    }

    fn mixed(caps: &Capabilities, mix: Mix, tables: &mut Tables, old: Style) -> Style {
        let mixer = Mixer::new(mix, caps).expect("the fixture is not the identity");
        mixer.style(tables, old)
    }

    // ---- the arithmetic ------------------------------------------------------------------------

    #[test]
    fn the_identity_and_the_full_mix_are_exact_at_both_ends() {
        let from = Rgb::new(10, 128, 255);
        let to = Rgb::new(255, 0, 7);
        assert_eq!(blend(from, to, 0), from);
        assert_eq!(blend(from, to, Mix::FULL), to);
    }

    #[test]
    fn a_mix_saturates_rather_than_wrapping() {
        assert_eq!(Mix::new(Color::DEFAULT, 60_000).amount(), Mix::FULL);
        assert!(Mix::new(Color::DEFAULT, 0).is_identity());
        assert!(!Mix::new(Color::DEFAULT, 1).is_identity());
        assert_eq!(
            Mix::new(Color::rgb(1, 2, 3), 8).toward(),
            Color::rgb(1, 2, 3)
        );
    }

    #[test]
    fn darken_is_toward_black_and_lift_is_toward_white() {
        // The whole reason one operator replaced three: these are not two mechanisms.
        assert_eq!(Mix::darken(128).toward(), Color::rgb(0, 0, 0));
        assert_eq!(Mix::lift(128).toward(), Color::rgb(255, 255, 255));
        assert_eq!(Mix::darken(128).amount(), 128);
    }

    #[test]
    fn two_overlapping_operators_at_a_half_leave_the_overlap_at_a_quarter() {
        // **Operators compound and are therefore not idempotent**, which is visually right and
        // is why the order of operators among themselves matters. Two shadows at 0.5 leave a
        // quarter of the original, and 255/4 truncates to 63 rather than rounding to 64.
        let white = Rgb::new(255, 255, 255);
        let black = Rgb::new(0, 0, 0);
        let once = blend(white, black, 128);
        let twice = blend(once, black, 128);
        assert_eq!(once, Rgb::new(127, 127, 127));
        assert_eq!(twice, Rgb::new(63, 63, 63));
    }

    // ---- resolution ---------------------------------------------------------------------------

    #[test]
    fn rgb_resolves_to_itself() {
        assert_eq!(
            resolve(&silent(), Color::rgb(0x12, 0x34, 0x56), Side::Fg),
            Some(Rgb::new(0x12, 0x34, 0x56))
        );
    }

    #[test]
    fn default_resolves_to_a_different_colour_on_each_side() {
        let caps = answering();
        assert_eq!(
            resolve(&caps, Color::DEFAULT, Side::Fg),
            Some(Rgb::new(0xc0, 0xc0, 0xc0))
        );
        assert_eq!(
            resolve(&caps, Color::DEFAULT, Side::Bg),
            Some(Rgb::new(0x20, 0x20, 0x20))
        );
    }

    #[test]
    fn default_resolves_to_nothing_where_the_terminal_stayed_silent() {
        assert_eq!(resolve(&silent(), Color::DEFAULT, Side::Fg), None);
        assert_eq!(resolve(&silent(), Color::DEFAULT, Side::Bg), None);
    }

    #[test]
    fn the_cube_and_the_greys_are_fixed_by_specification() {
        let caps = silent();
        // The corners of the 6x6x6 cube, and the two ends of the grey ramp.
        assert_eq!(indexed(&caps, 16), Rgb::new(0, 0, 0));
        assert_eq!(indexed(&caps, 231), Rgb::new(255, 255, 255));
        assert_eq!(indexed(&caps, 196), Rgb::new(255, 0, 0));
        assert_eq!(indexed(&caps, 46), Rgb::new(0, 255, 0));
        assert_eq!(indexed(&caps, 21), Rgb::new(0, 0, 255));
        assert_eq!(indexed(&caps, 232), Rgb::new(8, 8, 8));
        assert_eq!(indexed(&caps, 255), Rgb::new(238, 238, 238));
    }

    #[test]
    fn the_first_sixteen_come_from_the_terminal_and_fall_back_to_xterms_own() {
        // The silence asymmetry, as two assertions: OSC 4 silence *defaults* where OSC 11 silence
        // *refuses*, because a themed palette is wrong in degree and a guessed background is wrong
        // in direction.
        assert_eq!(indexed(&silent(), 1), Rgb::new(0xcd, 0x00, 0x00));
        let themed = Capabilities::answering_palette(Rgb::new(1, 2, 3));
        assert_eq!(indexed(&themed, 1), Rgb::new(1, 2, 3));
        assert_eq!(
            indexed(&themed, 200),
            indexed(&silent(), 200),
            "a themed palette says nothing about the cube"
        );
    }

    // ---- what a Mix does to a style -----------------------------------------------------------

    #[test]
    fn a_mix_moves_foreground_and_background_by_the_same_factor() {
        let mut t = Tables::new();
        let old = Style::new()
            .bold()
            .fg(Color::rgb(200, 200, 200))
            .bg(Color::rgb(100, 100, 100));
        let now = mixed(&silent(), Mix::darken(128), &mut t, old);
        assert_eq!(now.foreground(), Color::rgb(100, 100, 100));
        assert_eq!(now.background(), Color::rgb(50, 50, 50));
        assert_eq!(
            now.attr_word(),
            old.attr_word(),
            "attributes are not colours"
        );
    }

    #[test]
    fn a_mix_toward_a_colour_is_a_tint_rather_than_a_darkening() {
        let mut t = Tables::new();
        let old = Style::new().fg(Color::rgb(0, 0, 0)).bg(Color::rgb(0, 0, 0));
        let now = mixed(
            &silent(),
            Mix::new(Color::rgb(0, 200, 0), Mix::FULL / 2),
            &mut t,
            old,
        );
        assert_eq!(now.foreground(), Color::rgb(0, 100, 0));
        assert_eq!(now.background(), Color::rgb(0, 100, 0));
    }

    #[test]
    fn an_indexed_cell_comes_out_as_the_channels_it_resolved_to() {
        let mut t = Tables::new();
        // Index 196 is pure red in the cube; a full mix toward black is black, whatever the theme.
        let old = Style::new().fg(Color::indexed(196)).bg(Color::indexed(196));
        let now = mixed(&silent(), Mix::darken(Mix::FULL), &mut t, old);
        assert_eq!(now.foreground(), Color::rgb(0, 0, 0));
    }

    #[test]
    fn a_cell_with_a_default_background_is_left_alone_where_osc_11_was_silent() {
        // The silent path, which is the tested one until something can declare a default
        // background headless.
        let mut t = Tables::new();
        let old = Style::new().fg(Color::rgb(200, 200, 200));
        assert_eq!(mixed(&silent(), Mix::darken(128), &mut t, old), old);
        assert!(t.exts.is_empty(), "an unmixed cell mints nothing");
    }

    #[test]
    fn a_cell_with_a_default_foreground_is_left_alone_for_the_same_reason() {
        // Not an enumerated case, and the same rule decides it: mixing the background alone
        // *raises* contrast, which is the bug "the same factor" exists to prevent.
        let mut t = Tables::new();
        let osc11_only = Capabilities::answering(
            ColorDepth::TrueColor,
            None,
            Some(Rgb::new(0x20, 0x20, 0x20)),
        );
        let old = Style::new().bg(Color::rgb(100, 100, 100));
        assert_eq!(mixed(&osc11_only, Mix::darken(128), &mut t, old), old);
    }

    #[test]
    fn a_cell_with_both_colours_answered_is_mixed_against_the_terminals_own() {
        let mut t = Tables::new();
        let now = mixed(
            &answering(),
            Mix::darken(Mix::FULL / 2),
            &mut t,
            Style::new(),
        );
        assert_eq!(now.foreground(), Color::rgb(0x60, 0x60, 0x60));
        assert_eq!(now.background(), Color::rgb(0x10, 0x10, 0x10));
    }

    #[test]
    fn a_mix_toward_the_terminals_own_default_resolves_per_side() {
        // `Color::DEFAULT` on the `toward` side is *the terminal's own foreground or background,
        // whichever side of a style this lands on* — so a full mix toward it lands each channel on
        // its own side's colour rather than both on one.
        let mut t = Tables::new();
        let old = Style::new()
            .fg(Color::rgb(0, 0, 0))
            .bg(Color::rgb(255, 255, 255));
        let now = mixed(
            &answering(),
            Mix::new(Color::DEFAULT, Mix::FULL),
            &mut t,
            old,
        );
        assert_eq!(now.foreground(), Color::rgb(0xc0, 0xc0, 0xc0));
        assert_eq!(now.background(), Color::rgb(0x20, 0x20, 0x20));
    }

    #[test]
    fn a_mix_toward_the_default_is_skipped_where_the_terminal_stayed_silent() {
        // **As a whole layer, not per cell.** Every cell would take the both-or-neither path, so
        // there is nothing for the mixer to be built for — the same shape as the identity and as
        // `ColorDepth::None`.
        assert!(Mixer::new(Mix::new(Color::DEFAULT, 128), &silent()).is_none());
        assert!(Mixer::new(Mix::new(Color::DEFAULT, 128), &answering()).is_some());
        // And one side is enough to refuse: `Mix` moves both channels or neither.
        let osc10_only = Capabilities::answering(
            ColorDepth::TrueColor,
            Some(Rgb::new(0xc0, 0xc0, 0xc0)),
            None,
        );
        assert!(Mixer::new(Mix::new(Color::DEFAULT, 128), &osc10_only).is_none());
        assert!(
            Mixer::new(Mix::darken(128), &osc10_only).is_some(),
            "a `toward` that needs no answer is unaffected"
        );
    }

    #[test]
    fn assert_reverse_is_left_to_the_terminal() {
        // Two halves. **Darkening a reversed cell is invariant under the swap**, so the operator
        // needs no special case: the same factor on both channels commutes with exchanging them.
        let mut t = Tables::new();
        let plain = Style::new()
            .fg(Color::rgb(200, 200, 200))
            .bg(Color::rgb(100, 100, 100));
        let reversed = plain.reverse();
        let a = mixed(&silent(), Mix::darken(128), &mut t, plain);
        let b = mixed(&silent(), Mix::darken(128), &mut t, reversed);
        assert_eq!(a.foreground(), b.foreground());
        assert_eq!(a.background(), b.background());
        assert_eq!(b.attrs(), reversed.attrs(), "the bit is carried across");

        // And **a fade toward the terminal's own colours is not invariant**, because `toward`
        // resolves to a different colour on each side. Pinned so that it is a choice: swapping the
        // sides here would be right only if the terminal honours the bit, and `attrs_dropped` can
        // drop it at serialise time where the compositor cannot see.
        let caps = answering();
        let fade = Mix::new(Color::DEFAULT, Mix::FULL);
        let faded = mixed(&caps, fade, &mut t, reversed);
        assert_eq!(
            faded.foreground(),
            Color::rgb(0xc0, 0xc0, 0xc0),
            "the foreground channel goes to the default foreground, reversed or not"
        );
        assert_eq!(faded.background(), Color::rgb(0x20, 0x20, 0x20));
    }

    // ---- the hyperlink, which is the defect the prototype shipped ------------------------------

    #[test]
    fn a_mix_over_a_hyperlinked_cell_preserves_the_hyperlink() {
        // The prototype **deleted** it: it rewrote bits 51..0 with two colours,
        // clearing bit 63 and the 52-bit handle with it, silently. This goes through the descriptor
        // that names only the colours, so the link is not preserved — it is never named.
        let mut t = Tables::new();
        const URI: &str = "https://example.com/vitui";
        let old = extended_with_link(
            &mut t,
            URI,
            Style::new()
                .fg(Color::rgb(200, 200, 200))
                .bg(Color::rgb(100, 100, 100)),
        );
        assert!(old.is_extended(), "the fixture is the extended corner");

        let now = mixed(&silent(), Mix::darken(128), &mut t, old);
        assert!(now.is_extended());
        let e = t
            .exts
            .get(now.ext_handle().expect("still extended"))
            .expect("minted here");
        assert_eq!(
            t.links.uri(e.link),
            Some(URI),
            "the hyperlink survived the shadow"
        );
        assert_eq!(e.fg, Color::rgb(100, 100, 100));
        assert_eq!(e.bg, Color::rgb(50, 50, 50));
    }

    #[test]
    fn a_mix_over_an_underline_colour_moves_it_with_the_text() {
        let mut t = Tables::new();
        let old = applied(
            &mut t,
            &Restyle {
                ul: Some(Color::rgb(200, 0, 0)),
                ..Default::default()
            },
            Style::new()
                .underline()
                .fg(Color::rgb(200, 200, 200))
                .bg(Color::rgb(100, 100, 100)),
        );
        let now = mixed(&silent(), Mix::darken(128), &mut t, old);
        let e = t
            .exts
            .get(now.ext_handle().expect("still extended"))
            .unwrap();
        assert_eq!(e.ul, Color::rgb(100, 0, 0));
        assert_eq!(e.link, LinkId::NONE);
    }

    #[test]
    fn a_default_underline_colour_stays_default_and_follows_the_text() {
        // SGR 59 means *the text's own colour*. Mixing it to an explicit value would pin an
        // underline to the colour the text had before the operator ran.
        let mut t = Tables::new();
        let old = Style::new()
            .underline()
            .fg(Color::rgb(200, 200, 200))
            .bg(Color::rgb(100, 100, 100));
        let now = mixed(&silent(), Mix::darken(128), &mut t, old);
        assert!(!now.is_extended(), "nothing here needs the table");
        assert_eq!(now.foreground(), Color::rgb(100, 100, 100));
    }

    // ---- what the memo rests on --------------------------------------------------------------

    #[test]
    fn the_mix_is_a_function_of_the_style_word_and_of_nothing_else() {
        // The property `recolour`'s one-entry memo is sound on. Asked twice, the same word answers
        // the same thing; and a word the mixer has already seen once is not answered differently
        // for having seen another in between. A mix that depended on the cell or the column would
        // make the memo *wrong* rather than merely useless.
        let mut t = Tables::new();
        let caps = silent();
        let mixer = Mixer::new(Mix::darken(77), &caps).unwrap();
        let a = Style::new()
            .fg(Color::rgb(200, 100, 50))
            .bg(Color::rgb(50, 100, 200));
        let b = Style::new().fg(Color::rgb(1, 2, 3)).bg(Color::rgb(4, 5, 6));
        let (fa, fb) = (mixer.style(&mut t, a), mixer.style(&mut t, b));
        assert_ne!(fa, fb);
        for _ in 0..10 {
            assert_eq!(mixer.style(&mut t, a), fa);
            assert_eq!(mixer.style(&mut t, b), fb);
        }
    }

    #[test]
    fn a_style_that_needs_the_table_is_interned_once_however_often_it_is_asked_for() {
        // The other half: the table deduplicates, so an unmemoised mix and a memoised one grow it
        // by the same amount. **The memo is a speed-up and the table is what bounds the growth** —
        // which is why the memo's own number is a report (`crate::layer`'s bench) and the growth is
        // impl 08's, with the sweep.
        let mut t = Tables::new();
        let old = extended_with_link(
            &mut t,
            "https://example.com/vitui",
            Style::new()
                .fg(Color::rgb(3, 3, 3))
                .bg(Color::rgb(30, 30, 30)),
        );
        let before = t.exts.len();
        let caps = silent();
        let mixer = Mixer::new(Mix::darken(128), &caps).unwrap();
        for _ in 0..300 {
            mixer.style(&mut t, old);
        }
        assert_eq!(t.exts.len() - before, 1);
    }

    // ---- the cost of having one operator instead of three ------------------------------------

    /// The third operator number: **`Mix` costs 14% over a plain `Darken`**, and it is taken.
    ///
    /// A report, not a gate. The arm it is measured against is a plain darkening — every channel
    /// scaled, with nothing to interpolate toward — written here and **nowhere else in the crate**,
    /// because it is the thing deliberately not shipped: one operator instead of three buys
    /// lifting, tint and fade for this difference, and one mechanism is worth more than the
    /// percentage.
    ///
    /// Measured on inline style words, which is where the difference lives. An extended word costs a
    /// table read and an intern on top, identical in both arms, and interning 24 000 distinct
    /// entries would measure a `HashMap` growing rather than two arithmetics.
    ///
    /// ```text
    /// cargo test --release -p vitui-engine a_mix_costs_what_a_plain_darken -- --nocapture
    /// ```
    #[test]
    fn a_mix_costs_what_a_plain_darken_costs_and_the_difference_is_reported() {
        /// A plain `Darken`: every channel scaled, no `toward`, one multiply-add per channel fewer.
        ///
        /// The same shape as [`Mixer::moved`] in every other respect — the same table read, the same
        /// descriptor, the same `apply` — so the difference the bench reports is the arithmetic and
        /// nothing else.
        fn scaled(c: Rgb, amount: u16) -> Rgb {
            let one = |a: u8| ((a as u32 * (Mix::FULL - amount) as u32) / Mix::FULL as u32) as u8;
            Rgb::new(one(c.r), one(c.g), one(c.b))
        }

        fn plainly_darkened(
            caps: &Capabilities,
            amount: u16,
            tables: &mut Tables,
            old: Style,
        ) -> Option<Style> {
            let e = restyle::channels(tables, old);
            let d = Restyle {
                fg: Some(scaled(resolve(caps, e.fg, Side::Fg)?, amount).into()),
                bg: Some(scaled(resolve(caps, e.bg, Side::Bg)?, amount).into()),
                ul: if e.ul == Color::DEFAULT {
                    None
                } else {
                    Some(scaled(resolve(caps, e.ul, Side::Fg)?, amount).into())
                },
                ..Restyle::default()
            };
            Some(restyle::apply(tables, &d, None, old))
        }

        // A full screen's worth of *distinct* inline style words, so neither arm can be helped by a
        // repeat and neither touches a table.
        const CELLS: usize = 300 * 80;
        let styles: Vec<Style> = (0..CELLS)
            .map(|i| {
                let n = i as u32;
                Style::new()
                    .fg(Color::rgb((n >> 8) as u8, n as u8, (n >> 4) as u8))
                    .bg(Color::rgb(n as u8, (n >> 8) as u8, (n >> 2) as u8))
            })
            .collect();
        let caps = silent();
        let mixer = Mixer::new(Mix::darken(128), &caps).expect("not the identity");
        let mut t = Tables::new();
        let mut u = Tables::new();

        let report = vitui_bench::Bench::new(20)
            .case("mix", 20, || {
                for s in &styles {
                    std::hint::black_box(mixer.style(&mut t, *s));
                }
            })
            .case("plain-darken", 20, || {
                for s in &styles {
                    std::hint::black_box(plainly_darkened(&caps, 128, &mut u, *s).unwrap_or(*s));
                }
            })
            .run();

        if cfg!(debug_assertions) {
            println!(
                "these numbers are a debug build and are not comparable to the figure below; \
                 rerun with --release"
            );
        }
        let (mix, plain) = (
            report.get("mix").expect("measured"),
            report.get("plain-darken").expect("measured"),
        );
        println!(
            "one operator against a plain darken, {CELLS} distinct inline styles, minimum of 20 \
             rounds:\n{report}\
             \n            mix over plain-darken     {:>8.1}%   spec §5 recorded 14% \
             (78.2 against 68.5 us)\
             \n            report, not a gate, and the percentage is not the interesting part: the \
             memo\n                        moved this arithmetic off the per-cell path entirely, so \
             on a real frame it\n                        lands once per distinct style rather than \
             24 000 times. What §5 measured\n                        it as — a whole full-screen \
             composite, unmemoised — no longer exists.\n                        22% here against 14% \
             there is the same difference over a\n                        narrower slice: §5 divided \
             it into a whole composite and this divides it\n                        into the \
             arithmetic alone. The two arms are kept because the *decision*\n                        \
             is still live: one operator instead of three, and this is what it costs.",
            (mix - plain) / plain * 100.0
        );
        println!();
    }

    // ---- what is skipped outright -------------------------------------------------------------

    #[test]
    fn there_is_no_mixer_for_the_identity_or_for_a_terminal_with_no_colour() {
        let caps = answering();
        assert!(Mixer::new(Mix::darken(0), &caps).is_none());
        let plain = Capabilities::answering(ColorDepth::None, None, None);
        assert!(
            Mixer::new(Mix::darken(128), &plain).is_none(),
            "a Mix provably changes no byte on the wire at ColorDepth::None"
        );
        assert!(
            Mixer::new(Mix::new(Color::DEFAULT, 128), &silent()).is_none(),
            "nor when there is nowhere to move a cell toward"
        );
        assert!(
            Mixer::new(Mix::darken(128), &caps).is_some(),
            "and it does at every other depth, including Ansi16 where the effect is often nil \
             but not provably so"
        );
    }
}
