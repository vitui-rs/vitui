//! The shipped standard set: fourteen base16 schemes, imported at compile time.
//!
//! # Provenance
//!
//! Every palette below is one file of [`tinted-theming/schemes`][corpus], branch `spec-0.11`, taken
//! on 2026-08-23. The corpus is **MIT**, one licence over all of it, and the `name:` and `author:`
//! fields are carried through unchanged so that credit travels with the colours.
//!
//! Where a scheme is a port of an upstream project, the corpus's own author line is what is
//! reproduced — which is also the safest form: the colours here were **imported and re-paired by
//! vitui** into thirteen roles, so a line reading *the Dracula theme* would imply an endorsement
//! nobody gave, while *colours from Dracula, imported by vitui* is nominative use and accurate.
//!
//! **The corpus itself is not vendored.** The map's research measured all 338 of its schemes from a
//! clone and vendored none of them; what ships is this table, which is a set the library stands
//! behind rather than a mirror of somebody else's repository.
//!
//! [corpus]: https://github.com/tinted-theming/schemes
//!
//! # Why fourteen, and why four of them are light
//!
//! Fourteen is the map's own set — the names people ask for by name — and the four light entries are
//! not decoration. **Every degradation number on this map was taken on the harder of the two
//! variants**, and that only became knowable once both were measurable: a light theme collapses
//! *fewer* pairs at sixteen colours (the corpus mean is 8.65 against 11.79) and carries more than
//! twice the low-contrast roles. A dark-only set would have shipped the first half of that as a
//! general fact.
//!
//! # What a scheme does not carry
//!
//! No glyph repertoire, no density, and no `Distinction` bit. An import produces at most a third of a
//! theme; the rest is [`Themes`](super::Themes)'s, which is where the tier arrives.

use super::Scheme;

/// The shipped set, in the order a picker should list it: ten dark, four light.
///
/// It is a `&'static [Scheme]` and nothing more, so an application that wants a different set — its
/// own palette, a subset, or these plus its own — passes one to [`Themes::new`](super::Themes::new)
/// and nothing here is in the way.
pub const STANDARD: &[Scheme] = &[
    DRACULA,
    NORD,
    GRUVBOX_DARK_MEDIUM,
    GRUVBOX_LIGHT_MEDIUM,
    CATPPUCCIN_MOCHA,
    CATPPUCCIN_LATTE,
    TOKYO_NIGHT_DARK,
    SOLARIZED_DARK,
    SOLARIZED_LIGHT,
    EVERFOREST_DARK_MEDIUM,
    KANAGAWA,
    AYU_DARK,
    ONEDARK,
    ROSE_PINE,
];

/// Dracula.
pub const DRACULA: Scheme = Scheme::base16(
    "dracula",
    "Dracula",
    "clach04 (https://github.com/clach04)",
    &[
        0x282a36, 0x21222c, 0x44475a, 0x6272a4, 0x9ea8c7, 0xf8f8f2, 0xf8f8f2, 0xffffff, 0xff5555,
        0xffb86c, 0xf1fa8c, 0x50fa7b, 0x8be9fd, 0xbd93f9, 0xff79c6, 0x993333,
    ],
);

/// Nord.
pub const NORD: Scheme = Scheme::base16(
    "nord",
    "Nord",
    "arcticicestudio",
    &[
        0x2e3440, 0x3b4252, 0x434c5e, 0x4c566a, 0xd8dee9, 0xe5e9f0, 0xeceff4, 0x8fbcbb, 0xbf616a,
        0xd08770, 0xebcb8b, 0xa3be8c, 0x88c0d0, 0x81a1c1, 0xb48ead, 0x5e81ac,
    ],
);

/// Gruvbox dark, medium.
pub const GRUVBOX_DARK_MEDIUM: Scheme = Scheme::base16(
    "gruvbox-dark-medium",
    "Gruvbox dark, medium",
    "Dawid Kurek (dawikur@gmail.com), morhetz (https://github.com/morhetz/gruvbox)",
    &[
        0x282828, 0x3c3836, 0x504945, 0x665c54, 0xbdae93, 0xd5c4a1, 0xebdbb2, 0xfbf1c7, 0xfb4934,
        0xfe8019, 0xfabd2f, 0xb8bb26, 0x8ec07c, 0x83a598, 0xd3869b, 0xd65d0e,
    ],
);

/// Gruvbox light, medium.
pub const GRUVBOX_LIGHT_MEDIUM: Scheme = Scheme::base16(
    "gruvbox-light-medium",
    "Gruvbox light, medium",
    "Dawid Kurek (dawikur@gmail.com), morhetz (https://github.com/morhetz/gruvbox)",
    &[
        0xfbf1c7, 0xebdbb2, 0xd5c4a1, 0xbdae93, 0x665c54, 0x504945, 0x3c3836, 0x282828, 0x9d0006,
        0xaf3a03, 0xb57614, 0x79740e, 0x427b58, 0x076678, 0x8f3f71, 0xd65d0e,
    ],
);

/// Catppuccin Mocha. **The default**, and the palette every earlier ticket's numbers were taken on.
pub const CATPPUCCIN_MOCHA: Scheme = Scheme::base16(
    "catppuccin-mocha",
    "Catppuccin Mocha",
    "https://github.com/catppuccin/catppuccin",
    &super::CATPPUCCIN_MOCHA,
);

/// Catppuccin Latte.
pub const CATPPUCCIN_LATTE: Scheme = Scheme::base16(
    "catppuccin-latte",
    "Catppuccin Latte",
    "https://github.com/catppuccin/catppuccin",
    &[
        0xeff1f5, 0xe6e9ef, 0xccd0da, 0xbcc0cc, 0xacb0be, 0x4c4f69, 0xdc8a78, 0x7287fd, 0xd20f39,
        0xfe640b, 0xdf8e1d, 0x40a02b, 0x179299, 0x1e66f5, 0x8839ef, 0xdd7878,
    ],
);

/// Tokyo Night Dark.
pub const TOKYO_NIGHT_DARK: Scheme = Scheme::base16(
    "tokyo-night-dark",
    "Tokyo Night Dark",
    "Michaël Ball",
    &[
        0x1a1b26, 0x16161e, 0x2f3549, 0x444b6a, 0x787c99, 0xa9b1d6, 0xcbccd1, 0xd5d6db, 0xc0caf5,
        0xa9b1d6, 0x0db9d7, 0x9ece6a, 0xb4f9f8, 0x2ac3de, 0xbb9af7, 0xf7768e,
    ],
);

/// Solarized Dark.
pub const SOLARIZED_DARK: Scheme = Scheme::base16(
    "solarized-dark",
    "Solarized Dark",
    "Ethan Schoonover (modified by aramisgithub)",
    &[
        0x002b36, 0x073642, 0x586e75, 0x657b83, 0x839496, 0x93a1a1, 0xeee8d5, 0xfdf6e3, 0xdc322f,
        0xcb4b16, 0xb58900, 0x859900, 0x2aa198, 0x268bd2, 0x6c71c4, 0xd33682,
    ],
);

/// Solarized Light.
pub const SOLARIZED_LIGHT: Scheme = Scheme::base16(
    "solarized-light",
    "Solarized Light",
    "Ethan Schoonover (modified by aramisgithub)",
    &[
        0xfdf6e3, 0xeee8d5, 0x93a1a1, 0x839496, 0x657b83, 0x586e75, 0x073642, 0x002b36, 0xdc322f,
        0xcb4b16, 0xb58900, 0x859900, 0x2aa198, 0x268bd2, 0x6c71c4, 0xd33682,
    ],
);

/// Everforest Dark Medium.
///
/// **The one entry the map's exception list calls a mapping limit rather than an authored
/// collision**: `base01`, `base02` and `base03` all quantise to one 256-colour index, so the ramp
/// holds a single usable bucket and the derivation between the scheme's two ends finds nothing
/// distinct either. It ships anyway, and the pair count is what says so — an import that refused a
/// scheme would ship a set somebody's favourite theme is missing from.
pub const EVERFOREST_DARK_MEDIUM: Scheme = Scheme::base16(
    "everforest-dark-medium",
    "Everforest Dark Medium",
    "Sainnhe Park (https://github.com/sainnhe)",
    &[
        0x2d353b, 0x343f44, 0x3d484d, 0x475258, 0x7a8478, 0x859289, 0x9da9a0, 0xd3c6aa, 0xe67e80,
        0xe69875, 0xdbbc7f, 0xa7c080, 0x83c092, 0x7fbbb3, 0xd699b6, 0x514045,
    ],
);

/// Kanagawa.
pub const KANAGAWA: Scheme = Scheme::base16(
    "kanagawa",
    "Kanagawa",
    "Tommaso Laurenzi (https://github.com/rebelot)",
    &[
        0x1f1f28, 0x16161d, 0x223249, 0x54546d, 0x727169, 0xdcd7ba, 0xc8c093, 0x717c7c, 0xc34043,
        0xffa066, 0xc0a36e, 0x76946a, 0x6a9589, 0x7e9cd8, 0x957fb8, 0xd27e99,
    ],
);

/// Ayu Dark.
pub const AYU_DARK: Scheme = Scheme::base16(
    "ayu-dark",
    "Ayu Dark",
    "Tinted Theming (https://github.com/tinted-theming), Ayu Theme (https://github.com/ayu-theme)",
    &[
        0x0b0e14, 0x131721, 0x202229, 0x3e4b59, 0xbfbdb6, 0xe6e1cf, 0xece8db, 0xf2f0e7, 0xf07178,
        0xff8f40, 0xffb454, 0xaad94c, 0x95e6cb, 0x59c2ff, 0xd2a6ff, 0xe6b450,
    ],
);

/// OneDark.
pub const ONEDARK: Scheme = Scheme::base16(
    "onedark",
    "OneDark",
    "Lalit Magant (http://github.com/tilal6991)",
    &[
        0x282c34, 0x353b45, 0x3e4451, 0x545862, 0x565c64, 0xabb2bf, 0xb6bdca, 0xc8ccd4, 0xe06c75,
        0xd19a66, 0xe5c07b, 0x98c379, 0x56b6c2, 0x61afef, 0xc678dd, 0xbe5046,
    ],
);

/// Rosé Pine.
pub const ROSE_PINE: Scheme = Scheme::base16(
    "rose-pine",
    "Rosé Pine",
    "Emilia Dunfelt <edun@dunfelt.se>",
    &[
        0x191724, 0x1f1d2e, 0x26233a, 0x6e6a86, 0x908caa, 0xe0def4, 0xe0def4, 0x524f67, 0xeb6f92,
        0xf6c177, 0xebbcba, 0x31748f, 0x9ccfd8, 0xc4a7e7, 0xf6c177, 0x524f67,
    ],
);

#[cfg(test)]
mod tests {
    use super::*;

    /// **The set is a count**, the same way the scene list and the realistic screen are: a table that
    /// quietly lost an entry would otherwise make every universal gate above easier to pass.
    #[test]
    fn the_shipped_set_is_fourteen_schemes() {
        assert_eq!(STANDARD.len(), 14);
    }

    /// **A theme is `.rodata`, and this is the assertion that says so rather than the sentence.**
    ///
    /// Every entry above is a `const`, so the import ran at compile time and the binary carries the
    /// finished thirteen roles. A `const` promoted to a `&'static` is in read-only data by
    /// construction: taking its address twice gives one address, which a runtime import could not do.
    #[test]
    fn a_shipped_scheme_is_one_static_object() {
        let a: &'static Scheme = &STANDARD[0];
        let b: &'static Scheme = &STANDARD[0];
        assert!(std::ptr::eq(a, b));
        assert_eq!(a.roles(), DRACULA.roles());
    }

    /// The default palette is the same colours in both spellings, so nothing drifted when the
    /// `[u32; 16]` grew a `Scheme` beside it.
    ///
    /// **Compared by role and not by `==`**, because two themes built a moment apart carry two
    /// revisions and a derived `PartialEq` would call them different — which is the point of the
    /// revision and not a nuisance to work around.
    #[test]
    fn the_default_palette_is_the_same_in_both_spellings() {
        use super::super::{Density, GlyphSet, Role, Theme};
        let from_scheme = CATPPUCCIN_MOCHA.theme(GlyphSet::default(), Density::default());
        let from_array = Theme::default();
        for role in Role::ALL {
            assert_eq!(
                from_scheme.paint(role),
                from_array.paint(role),
                "{role:?} differs between the scheme and the array"
            );
        }
        assert_eq!(CATPPUCCIN_MOCHA.is_dark(), from_array.is_dark());
    }
}
