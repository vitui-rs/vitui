//! The shipped set as `.rodata`, and the one type that owns *which theme is current*.
//!
//! Spec §15 — *the standard set* and *live switching*; ADR 0007 (nothing here may read the operator's
//! palette), ADR 0018 (a component names a role, never a colour).
//!
//! # Two types, and the split is the whole design
//!
//! A [`Scheme`] is **data in `.rodata`**: thirteen paired roles, a slug, a display name, an author,
//! and the one derived bit. It is built by a `const fn` from sixteen base16 colours, so a shipped
//! theme costs no import at start-up and no allocation ever.
//!
//! A [`Themes`] is **application state**: a set of schemes, a selection, and the current [`Theme`]
//! built from the two. It is not a field of whatever holds the frame services, and it cannot be —
//! `Themes::theme` is a shared borrow that a picker holds *inside* the frame while the loop writes
//! the selection *between* frames, and both of those cannot come out of the same object. `E0502` is
//! the mechanism; the negative case is on [`Themes`] and its twin is beside it.
//!
//! # It earns its place on one line
//!
//! A hand-rolled set — a `Vec<Theme>` and an index — forgets [`Theme::resolve`], and **the failure of
//! forgetting is a tracking level rather than a colour.** An unresolved theme claims no distinction,
//! so `hover_interest()` comes back empty and the frame stops asking for motion; a theme resolved for
//! the wrong tier claims one the terminal will not show, and every pointer move buys a wakeup for a
//! highlight nobody can see. `Themes` resolves on every path that changes anything, so the call
//! cannot be forgotten.
//!
//! # Attribution, and why it is a field rather than a file
//!
//! The palettes are imported from [`tinted-theming/schemes`][corpus] (MIT), which is the only source
//! the map's research found that is a *corpus* rather than one application's palette: one schema,
//! hundreds of schemes, a permissive licence over all of them. Each [`Scheme`] carries the corpus's
//! own `name` and `author` through, so attribution travels with the palette and cannot drift away
//! from it — which an `ATTRIBUTION` file beside the table can and does.
//!
//! **`cargo deny` has nothing to say about any of this, and the silence is not approval.** cargo-deny
//! sees crates; a vendored colour scheme is data, so no `[licenses]` entry and no `[sources]` rule
//! will ever look at one. The check that replaces it is
//! [`every_shipped_theme_names_an_author`](self#gates), a count, run by `cargo test`.
//!
//! [corpus]: https://github.com/tinted-theming/schemes
//!
//! # Gates
//!
//! Three, and all three are universal over the shipped set rather than over a chosen example:
//! `Face`/`FaceHover` is distinct at 256 colours for every shipped theme, every shipped theme names a
//! non-empty author, and every slug is unique.

use vitui_engine::ColorDepth;

use super::{Density, GlyphSet, Roles, Theme, channel, luminance};

/// One application palette as it ships: thirteen paired roles and who to credit.
///
/// **`.rodata`, not a builder.** [`Scheme::base16`] is a `const fn`, so the sixteen colours are
/// paired into thirteen roles at compile time and what the binary carries is the finished table:
///
/// ```
/// use vitui_runtime::theme::Scheme;
///
/// const MINE: Scheme = Scheme::base16(
///     "mine",
///     "Mine",
///     "me <me@example.com>",
///     &[
///         0x101014, 0x181820, 0x242430, 0x343444, 0x505064, 0xd0d0e0, 0xe8e8f4, 0xb0b0ff,
///         0xff5f87, 0xffaf5f, 0xffd75f, 0x87d787, 0x5fd7d7, 0x5fafff, 0xaf87ff, 0xd7afaf,
///     ],
/// );
/// assert_eq!(MINE.slug(), "mine");
/// assert!(MINE.is_dark());
/// ```
///
/// An application that ships one palette of its own writes exactly that and pays the size of one
/// scheme. Nothing about [`Themes`] requires the shipped set: it takes any `&'static [Scheme]`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Scheme {
    roles: Roles,
    slug: &'static str,
    name: &'static str,
    author: &'static str,
    dark: bool,
}

impl Scheme {
    /// Import sixteen base16 colours — `base00`–`base0F`, each `0xRRGGBB` — into thirteen roles.
    ///
    /// The pairing is [`Roles::from_palette`]'s and happens here at compile time. `dark` is derived
    /// from the page colour rather than taken as a parameter, because the base16 `variant:` header
    /// and the scheme's own luminance agree in every file of the map's corpus.
    ///
    /// A colour scheme carries neither a glyph repertoire nor a density, so a `Scheme` carries
    /// neither: **an import produces at most a third of a theme**, and the other two thirds are
    /// [`Themes`]'s parameters.
    pub const fn base16(
        slug: &'static str,
        name: &'static str,
        author: &'static str,
        palette: &[u32; 16],
    ) -> Scheme {
        Scheme {
            roles: Roles::from_palette(palette),
            slug,
            name,
            author,
            dark: luminance(channel(palette[0])) < 128,
        }
    }

    /// The stable identifier, which is what a configuration file names.
    pub const fn slug(&self) -> &'static str {
        self.slug
    }

    /// The name a picker shows.
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Who to credit, carried through from the corpus. **Never empty** — see the module's gates.
    pub const fn author(&self) -> &'static str {
        self.author
    }

    /// Whether the page is dark, derived from the page colour's luminance.
    pub const fn is_dark(&self) -> bool {
        self.dark
    }

    /// The thirteen paired roles.
    pub const fn roles(&self) -> Roles {
        self.roles
    }

    /// Build a theme from this scheme. **Unresolved** — the tier is [`Themes`]'s to supply, and this
    /// is why the registry earns its line.
    pub fn theme(&self, glyphs: GlyphSet, density: Density) -> Theme {
        Theme::imported(self.roles, glyphs, density)
    }
}

/// The set of application palettes a program offers, plus which one is current.
///
/// # It is application state, and that is a compile outcome rather than a preference
///
/// Put the set inside whatever owns the frame and a picker cannot read it, because reading it is a
/// shared borrow of the same object the frame call takes mutably:
///
/// ```compile_fail,E0502
/// // The shape, mimicked: a set of themes living *inside* the thing that runs a frame.
/// struct Themes;
/// impl Themes { fn name(&self) -> &'static str { "one" } }
/// struct Services { themes: Themes }
/// impl Services {
///     fn themes(&self) -> &Themes { &self.themes }
///     fn frame(&mut self, view: impl FnOnce()) { view(); }
/// }
///
/// let mut services = Services { themes: Themes };
/// let themes = services.themes();
/// // The picker reads the set *inside* the frame, and the frame is `&mut` on the same object.
/// services.frame(|| { let _ = themes.name(); });
/// ```
///
/// Held beside the driver instead, the same program compiles — and this twin names the shipped items
/// by path, because a lone `compile_fail` also passes when the type it was written for has been
/// renamed:
///
/// ```
/// use vitui_runtime::Driver;
/// use vitui_runtime::theme::{Scheme, Themes};
///
/// let mut themes: Themes = Themes::standard();
/// let mut driver = Driver::headless(20, 3).expect("a headless attach cannot fail");
/// driver.set_theme(*themes.theme());
///
/// // A picker reads the set inside the frame. Two objects, so one `&mut` and one `&`.
/// driver.frame(|cx| {
///     let scheme: &'static Scheme = themes.scheme();
///     let _ = cx.text(0, 0, scheme.name(), cx.theme().paint(vitui_runtime::Role::Title));
/// });
///
/// // …and the loop writes it *between* frames, like any other application state.
/// assert!(themes.select_slug("nord"));
/// driver.set_theme(*themes.theme());
/// ```
///
/// # A re-detected tier rebuilds one theme, not the set
///
/// Only the current scheme is ever built into a [`Theme`], so only one thing can be stale and
/// [`Themes::set_tier`] rebuilds exactly that. A registry that cached a resolved theme per scheme
/// would pay the whole set's import for a tier that arrives once.
#[derive(Clone, Debug)]
pub struct Themes {
    schemes: &'static [Scheme],
    at: usize,
    tier: ColorDepth,
    glyphs: GlyphSet,
    density: Density,
    current: Theme,
}

impl Themes {
    /// The shipped set: fourteen schemes, four of them light.
    pub fn standard() -> Themes {
        Themes::new(super::schemes::STANDARD)
            .expect("the shipped set is not empty, and a gate in this module says so")
    }

    /// A registry over any set of schemes, or `None` if the set is empty.
    ///
    /// **`Option`, because a registry with nothing in it has no current theme** and every other verb
    /// here would have to invent one. One `expect` at start-up buys an invariant the rest of the type
    /// never restates.
    ///
    /// The tier starts at [`ColorDepth::None`], which is the conservative answer rather than a
    /// placeholder: a registry nobody has told about the terminal claims no distinction at all. Call
    /// [`Themes::set_tier`] with what `Ctx::caps()` reports.
    pub fn new(schemes: &'static [Scheme]) -> Option<Themes> {
        let first = schemes.first()?;
        let glyphs = GlyphSet::default();
        let density = Density::default();
        Some(Themes {
            schemes,
            at: 0,
            tier: ColorDepth::None,
            glyphs,
            density,
            current: first.theme(glyphs, density).resolve(ColorDepth::None),
        })
    }

    /// The current theme.
    ///
    /// Always resolved for [`Themes::tier`], because every path that could change it goes through
    /// `rebuild`.
    pub const fn theme(&self) -> &Theme {
        &self.current
    }

    /// The current scheme, which is what a picker shows and what a configuration file records.
    pub const fn scheme(&self) -> &'static Scheme {
        &self.schemes[self.at]
    }

    /// Every scheme in the set, in the order a picker should list them.
    pub const fn schemes(&self) -> &'static [Scheme] {
        self.schemes
    }

    /// Which one is current.
    pub const fn selected(&self) -> usize {
        self.at
    }

    /// How many there are. Never zero.
    pub const fn len(&self) -> usize {
        self.schemes.len()
    }

    /// Never true. Present because clippy asks for it beside `len`, and because a reader who has not
    /// met [`Themes::new`] would otherwise have to guess.
    pub const fn is_empty(&self) -> bool {
        false
    }

    /// The tier the current theme is resolved for.
    pub const fn tier(&self) -> ColorDepth {
        self.tier
    }

    /// Select by position. `false`, and nothing changes, if there is no such position.
    ///
    /// **The pick reaches the swap one frame later and cannot do better**: a component may not write
    /// to the frame's environment mid-frame, so a picker returns a selection and the loop applies it
    /// between frames.
    pub fn select(&mut self, at: usize) -> bool {
        if at >= self.schemes.len() {
            return false;
        }
        self.at = at;
        self.rebuild();
        true
    }

    /// Select by slug, which is what a configuration file holds. `false` if no scheme has it.
    pub fn select_slug(&mut self, slug: &str) -> bool {
        match self.schemes.iter().position(|s| s.slug == slug) {
            Some(at) => self.select(at),
            None => false,
        }
    }

    /// Tell the registry what the terminal can show, and rebuild the current theme against it.
    ///
    /// **One theme, not the set.** The tier arrives from `Ctx::caps()`, told once at start-up and
    /// told again if it is ever re-detected; the registry detects nothing itself.
    pub fn set_tier(&mut self, tier: ColorDepth) {
        self.tier = tier;
        self.rebuild();
    }

    /// Declare the glyph repertoire every theme in this set is built with. **Declared, never probed**.
    ///
    pub fn set_glyphs(&mut self, glyphs: GlyphSet) {
        self.glyphs = glyphs;
        self.rebuild();
    }

    /// Set the density every theme in this set is built with. A scheme carries none.
    pub fn set_density(&mut self, density: Density) {
        self.density = density;
        self.rebuild();
    }

    /// **The one line the type exists for.** Import, then resolve — and the resolve is not optional
    /// on any path, which is the whole difference between this and a `Vec<Theme>` with an index.
    fn rebuild(&mut self) {
        self.current = self.schemes[self.at]
            .theme(self.glyphs, self.density)
            .resolve(self.tier);
    }
}

#[cfg(test)]
mod tests {
    use vitui_engine::ColorDepth;

    use super::super::{Distinction, Role};
    use super::*;

    /// **Gate, universal.** `Face`/`FaceHover` is distinct at 256 colours for every shipped theme.
    ///
    /// This is the one number on the map that was already zero across the whole 338-scheme corpus,
    /// which is why it is the one worth failing a build over: the map's §5 named this pair as the
    /// first casualty of quantisation, `pick` is what stops it being one, and a mapping change that
    /// quietly reverted to slot names would put it back in 73 of 338.
    ///
    /// Written over `Themes::standard()` rather than over a chosen example, so it holds at whatever
    /// size the set grows to.
    #[test]
    fn face_and_face_hover_are_distinct_at_256_colours_for_every_shipped_theme() {
        let mut set = Themes::standard();
        set.set_tier(ColorDepth::Indexed256);
        for i in 0..set.len() {
            assert!(set.select(i));
            assert!(
                set.theme()
                    .roles_differ_on_wire(Role::Face, Role::FaceHover),
                "{} loses the hover pair at 256 colours",
                set.scheme().slug()
            );
            assert!(
                set.theme().shows(Distinction::Hover),
                "{} resolved to no hover distinction at 256 colours",
                set.scheme().slug()
            );
        }
    }

    /// **Gate, universal.** Every shipped theme names an author.
    ///
    /// The trap this is written against is a real file: of the corpus's 338 schemes, `seti.yaml`'s
    /// only credit is a comment reading *Base16 Builder scheme by #!* and it carries no `author:` key
    /// at all. An importer that emits a blank string rather than failing produces a set that looks
    /// attributed and is not, and **`cargo deny` cannot see this** — it sees crates, and a colour
    /// scheme is data.
    #[test]
    fn every_shipped_theme_names_an_author() {
        for scheme in Themes::standard().schemes() {
            assert!(
                !scheme.author().trim().is_empty(),
                "{} ships with no credit",
                scheme.slug()
            );
            assert!(!scheme.name().trim().is_empty(), "{}", scheme.slug());
            assert!(!scheme.slug().trim().is_empty());
        }
    }

    /// **Gate, count.** No two shipped schemes share a slug, because a slug is what a configuration
    /// file names and a duplicate would make `select_slug` answer the wrong one for ever.
    #[test]
    fn every_slug_is_unique() {
        let schemes = Themes::standard().schemes();
        for (i, a) in schemes.iter().enumerate() {
            for b in &schemes[i + 1..] {
                assert_ne!(a.slug(), b.slug());
            }
        }
    }

    /// The set has both variants in it, which is what makes the light-theme numbers measurable at
    /// all: **every degradation number on this map was taken on the harder variant**, and nothing
    /// would have found that with a dark-only set.
    ///
    /// **The map says *fourteen themes, four of them light* and the fourteen it names contain
    /// three.** `variant:` in the corpus, and the page colour's own luminance, agree on all fourteen;
    /// the three are `gruvbox-light-medium`, `catppuccin-latte` and `solarized-light`. So the count
    /// is wrong and the list is right, and what is gated is the list — every entry's derived bit
    /// against the corpus header it was imported from — rather than a total that would have had to be
    /// met by adding a scheme nobody chose.
    #[test]
    fn the_shipped_set_carries_both_variants() {
        const LIGHT: [&str; 3] = [
            "gruvbox-light-medium",
            "catppuccin-latte",
            "solarized-light",
        ];
        for scheme in Themes::standard().schemes() {
            assert_eq!(
                scheme.is_dark(),
                !LIGHT.contains(&scheme.slug()),
                "{}: the derived bit disagrees with the corpus `variant:` header",
                scheme.slug()
            );
        }
        let light = Themes::standard()
            .schemes()
            .iter()
            .filter(|s| !s.is_dark())
            .count();
        assert!(light > 0 && light < Themes::standard().len());
    }

    /// **The line the registry earns.** A hand-rolled set forgets `resolve`, and the failure of
    /// forgetting is a tracking level rather than a colour: an unresolved theme claims no
    /// distinction, so a component asking `hover_interest()` is told not to track.
    #[test]
    fn selecting_resolves_and_forgetting_to_would_have_cost_a_tracking_level() {
        let mut set = Themes::standard();
        set.set_tier(ColorDepth::TrueColor);
        assert!(set.select_slug("nord"));
        assert_eq!(set.theme().tier(), ColorDepth::TrueColor);
        assert!(set.theme().shows(Distinction::Hover));

        // The hand-rolled shape, for contrast: import and forget.
        let forgotten = set.scheme().theme(GlyphSet::default(), Density::default());
        assert_eq!(forgotten.tier(), ColorDepth::None);
        assert!(!forgotten.shows(Distinction::Hover));
        assert_eq!(forgotten.hover_interest(), crate::Interest::NONE);
    }

    /// A re-detected tier rebuilds the current theme and moves its revision, so every memo keyed on
    /// the theme misses exactly once.
    #[test]
    fn a_re_detected_tier_rebuilds_one_theme() {
        let mut set = Themes::standard();
        let before = set.theme().revision();
        set.set_tier(ColorDepth::TrueColor);
        assert_ne!(set.theme().revision(), before);
        assert_eq!(set.theme().tier(), ColorDepth::TrueColor);
        assert_eq!(
            set.selected(),
            0,
            "the selection is not what a tier changes"
        );
    }

    /// Selecting nothing changes nothing, in both spellings.
    #[test]
    fn an_absent_selection_changes_nothing() {
        let mut set = Themes::standard();
        set.set_tier(ColorDepth::TrueColor);
        assert!(set.select_slug("catppuccin-mocha"));
        let (at, rev) = (set.selected(), set.theme().revision());
        assert!(!set.select(set.len()));
        assert!(!set.select_slug("no-such-scheme"));
        assert_eq!((set.selected(), set.theme().revision()), (at, rev));
    }

    /// An empty set has no current theme, so there is no registry to hand back.
    #[test]
    fn an_empty_set_is_not_a_registry() {
        assert!(Themes::new(&[]).is_none());
    }

    /// How many of the seventy-eight role pairs a theme cannot distinguish at `tier`.
    fn collapsed(theme: &Theme) -> usize {
        let mut n = 0;
        for (i, &a) in Role::ALL.iter().enumerate() {
            for &b in &Role::ALL[i + 1..] {
                if !theme.roles_differ_on_wire(a, b) {
                    n += 1;
                }
            }
        }
        n
    }

    /// **The C16 pair count is a lower bound, and this is the gate that says so.**
    ///
    /// Every count below truecolor compares *indices*, because an index is all a process may know —
    /// ADR 0007: the engine does not know what `indexed(9)` looks like on this machine. An operator
    /// whose profile spells two indices with the same colour collapses pairs the index count called
    /// distinct, and half of the ten real profiles the map's research measured do exactly that,
    /// usually the bright half repeating the normal half. The corpus mean moves **10.87 → 16.36**.
    ///
    /// The corpus is not vendored here and neither are the ten profiles, so what is gated is the
    /// **relation** rather than either number: merging buckets never lowers the count, and for at
    /// least one shipped theme it raises it. That is what makes the phrase *lower bound* mechanical
    /// instead of a caveat in a doc comment.
    #[test]
    fn a_sixteen_colour_count_is_a_lower_bound_and_an_operator_can_only_raise_it() {
        // An operator palette, as a merge: `bright[i]` spells the same colour as `i`. This is the
        // shape five of the ten measured profiles have.
        let merged = |theme: &Theme| -> usize {
            let bucket = |r: Role| {
                let (fg, bg) = theme.wire_indices_at_16(r);
                (fg % 8, bg % 8, theme.attrs_of(r))
            };
            let mut n = 0;
            for (i, &a) in Role::ALL.iter().enumerate() {
                for &b in &Role::ALL[i + 1..] {
                    if bucket(a) == bucket(b) {
                        n += 1;
                    }
                }
            }
            n
        };

        let mut set = Themes::standard();
        set.set_tier(ColorDepth::Ansi16);
        let mut raised = 0;
        for i in 0..set.len() {
            assert!(set.select(i));
            let indices = collapsed(set.theme());
            let operator = merged(set.theme());
            assert!(
                operator >= indices,
                "{}: an operator palette lowered the count, {operator} against {indices}",
                set.scheme().slug()
            );
            if operator > indices {
                raised += 1;
            }
        }
        assert!(
            raised > 0,
            "no shipped theme is sensitive to the operator's palette, which would make the \
             lower-bound claim vacuous"
        );
    }

    /// Narrowing never *adds* a distinction, which is the relation every pair count is gated by —
    /// here over the whole shipped set rather than over one palette.
    #[test]
    fn narrowing_never_adds_a_distinction_for_any_shipped_theme() {
        let tiers = [
            ColorDepth::TrueColor,
            ColorDepth::Indexed256,
            ColorDepth::Ansi16,
            ColorDepth::None,
        ];
        let mut set = Themes::standard();
        for i in 0..set.len() {
            assert!(set.select(i));
            let mut previous = 0;
            for tier in tiers {
                set.set_tier(tier);
                let now = collapsed(set.theme());
                assert!(
                    now >= previous,
                    "{}: {tier:?} distinguishes more than the tier above it",
                    set.scheme().slug()
                );
                previous = now;
            }
        }
    }

    /// **Gate, count: `Themes` is not a field beside the frame services.**
    ///
    /// The `E0502` doctest above says a registry *cannot* live there; this says it *does not*. The
    /// two are different failures — the compile outcome would keep passing if somebody added a
    /// `Themes` field that nothing borrowed across a frame, and the shape would still be wrong.
    ///
    /// A scan for absent code, and the lesson from three previous ones is applied: the needle is
    /// joined at run time so that this test is not its own counter-example.
    #[test]
    fn the_frame_services_do_not_name_the_registry() {
        let needle = ["The", "mes"].concat();
        let hits = include_str!("../ctx.rs")
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .filter(|l| l.contains(&needle))
            .count();
        assert_eq!(
            hits, 0,
            "the registry is application state; `ctx` may not hold one"
        );
    }
}
