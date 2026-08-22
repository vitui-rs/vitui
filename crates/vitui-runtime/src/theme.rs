//! Theme: a component names a **role**, and can never construct a **paint**.
//!
//! Spec §3 and §10; ADR 0018 (a component names no colour), ADR 0021 (the runtime does not edit a
//! declared interest), ADR 0010 (the glyph ladder is declared, never probed).
//!
//! ```
//! use vitui_engine::ColorDepth;
//! use vitui_runtime::theme::{Density, Distinction, Role, Theme};
//!
//! // **A theme is resolved for a terminal before it answers anything about one.** Unresolved, it
//! // claims no distinction at all — which is the conservative answer and not a bug: a theme nobody
//! // has told about the terminal must not promise a component a colour the terminal may not have.
//! let unresolved = Theme::default();
//! assert!(!unresolved.roles_differ_on_wire(Role::Body, Role::Danger));
//! assert!(!unresolved.shows(Distinction::Hover));
//!
//! let theme = Theme::default().resolve(ColorDepth::TrueColor);
//! let _title = theme.paint(Role::Title);
//! assert!(theme.roles_differ_on_wire(Role::Body, Role::Danger));
//! assert!(theme.shows(Distinction::Hover));
//! assert_eq!(theme.density(), Density::Cosy);
//! ```
//!
//! # `Paint` is the mechanism, and it is one field's visibility
//!
//! [`Paint`] is a newtype over the engine's `Style` whose field is `pub(crate)`. **Only a [`Theme`]
//! hands one out.** That makes *no colour literals in a component* a compile outcome rather than a
//! lint, and it costs **1.002× and eight bytes against eight** — at the noise floor, since three
//! whole-frame measurements differ among themselves by more than that.
//!
//! The thing worth knowing is what the type does *not* forbid. `Theme::custom` mints a paint from two
//! arbitrary colours, and that is deliberate: read literally, *no route to an arbitrary paint* makes a
//! chart's fifth series, a photograph's pixel and a sentinel probe inexpressible. **What ADR 0018
//! protects is a component minting a palette**, and the narrower invariant that actually holds is
//! *a paint cannot exist without a live `&Theme`* — which is why `custom` takes `&self` and why a
//! theme swap cannot be bypassed.
//!
//! # A role names a paint, not a colour
//!
//! This is the sentence that killed the proposal's own role list. A component handed `surface` and
//! `on_surface` must *pair* them, and pairing is constructing a style — the literal the same section
//! forbids three bullets earlier. So the thirteen roles are pairs already: [`Role::Body`] is a
//! foreground *on* a background, and there is no `on_body`.
//!
//! # `resolve(tier)` is where a colour fact becomes a routing fact
//!
//! A theme's `hover_distinct` bit used to describe the **authored** palette. On a 256-colour terminal
//! that meant the frame paid `Motion` — every pointer move a wakeup and a whole 32.96 µs frame — for
//! a highlight nobody can see. [`Theme::resolve`] joins the two sentences into one mechanism at
//! **4.16 ns, once, at construction**: it narrows every role to what the terminal will show and sets
//! the [`Distinction`] bits from *that*, so a component branches on a bool and names neither axis.
//!
//! **The runtime does not edit a declared interest** (ADR 0021). The component reads
//! [`Theme::hover_interest`] and writes what it says; a component that ignores it still gets
//! `Motion`, and is therefore **visibly wrong rather than invisibly corrected.**

pub mod wire;

use vitui_engine::{Color, ColorDepth, LinkId, Restyle, Rgb, Style};

pub use vitui_engine::GlyphSet;

use crate::data::Revision;

/// A style a component cannot construct.
///
/// The field is `pub(crate)`, so the only way to hold one is to have been given one by a [`Theme`].
/// Eight bytes, and the same eight the engine's `Style` occupies.
///
/// # The two doors, both shut, both with a twin
///
/// **It cannot be constructed** — the newtype is not a way in:
///
/// ```compile_fail,E0423
/// use vitui_engine::{Color, Style};
/// use vitui_runtime::theme::Paint;
/// let _ = Paint(Style::new().fg(Color::rgb(1, 2, 3)));
/// ```
///
/// **And it cannot be taken apart** — so a component cannot read a colour out of one either:
///
/// ```compile_fail,E0616
/// use vitui_runtime::{Role, Theme};
/// let paint = Theme::default().paint(Role::Body);
/// let _ = paint.0;
/// ```
///
/// The twin, naming both protected items by path and doing the one thing a component may:
///
/// ```
/// use vitui_runtime::theme::Paint;
/// use vitui_runtime::{Role, Theme};
///
/// let theme = Theme::default();
/// let paint: Paint = theme.paint(Role::Body);
/// // Two paints for the same role are the same paint, which is all a component can learn.
/// assert_eq!(paint, theme.paint(Role::Body));
/// assert_ne!(paint, theme.paint(Role::Danger));
/// ```
///
/// **`Style::pack` is not the door and never was.** The theme documents record the hostile line as
/// `Theme::imported` plus `Style::pack` plus `Box::leak`; `Style::pack` does not exist in the engine
/// at all, so that line fails with `E0599` — *no such method* — and would start compiling the day
/// somebody added a packer for an unrelated reason. The line that has to be shut is the one that
/// works: `Style::new().fg(..).bg(..)`, which is public, is three characters shorter, and is what the
/// two cases above and [`Roles`] are written against.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Paint(pub(crate) Style);

impl Paint {
    /// The style, for the crate's own drawing verbs to hand to the engine.
    // `allow` and not `expect`, because under `cfg(test)` the lint does not fire and an unfulfilled
    // expectation is itself an error. The engine's `budget.rs` makes the same choice for the same
    // shape: different callers use different parts.
    //
    // **The caller is ticket 08's `Ctx` and it does not exist yet.** This is the seam — a component
    // writes a `Repaint` over roles and the crate lowers it — and until the draw context lands there
    // is nothing above it to call. Made `pub(crate)` rather than `pub` deliberately: a public `lower`
    // would hand a component an `engine::Restyle` with `Color`s in it, which is the thing ADR 0018
    // keeps out of a component's hands.
    #[allow(dead_code)]
    pub(crate) const fn style(self) -> Style {
        self.0
    }
}

/// A runtime newtype over the engine's opaque link handle. **No `u32` crosses the seam.**
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Link(pub(crate) LinkId);

impl Link {
    /// Wrap the engine's handle. `pub(crate)` — a component receives one from `Ctx::link` and can do
    /// nothing else with it, which is what keeps `u32` off the seam.
    pub(crate) const fn from_engine(id: LinkId) -> Link {
        Link(id)
    }
}

/// The thirteen roles.
///
/// Thirteen and not the proposal's list: `on_surface` and `on_accent` do not appear, because once a
/// role is a paint the pairing has already happened. There is no `Accent` and no `Shadow` — a shadow
/// is an operator layer, not a colour.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Role {
    /// Ordinary text on the page.
    Body,
    /// A heading. Carries `BOLD`, and that bit is load-bearing rather than decorative — see
    /// [`Roles::from_palette`].
    Title,
    /// Text that is present but secondary.
    Dim,
    /// Text that cannot be interacted with.
    Disabled,
    /// A rule, a frame, a divider.
    Border,
    /// The face of something interactive at rest.
    Face,
    /// The same face under the pointer.
    FaceHover,
    /// The same face while pressed.
    FaceActive,
    /// A selected row or region.
    Selection,
    /// The focus ring.
    Focus,
    /// Something has gone wrong.
    Danger,
    /// Something needs attention.
    Warn,
    /// Something succeeded.
    Ok,
}

impl Role {
    /// Every role, which is what the pair gate and the reports iterate.
    pub const ALL: [Role; 13] = [
        Role::Body,
        Role::Title,
        Role::Dim,
        Role::Disabled,
        Role::Border,
        Role::Face,
        Role::FaceHover,
        Role::FaceActive,
        Role::Selection,
        Role::Focus,
        Role::Danger,
        Role::Warn,
        Role::Ok,
    ];

    /// Its index into a [`Roles`] table.
    const fn index(self) -> usize {
        self as usize
    }
}

/// One role's authored appearance: two colours and the attribute bits.
///
/// **The theme keeps this and not only the `Style` it builds**, and the reason is a hard limit rather
/// than a preference: the engine's `Color` is an opaque newtype whose channels are `pub(crate)`, so a
/// `Style` cannot be taken apart from outside the engine. A theme that stored only styles could not
/// answer *what will the terminal show* about its own palette — which is
/// [`Theme::roles_differ_on_wire`] and every [`Distinction`] bit. See [`wire`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Spec {
    fg: Rgb,
    bg: Rgb,
    attrs: u16,
}

impl Spec {
    const fn new(fg: Rgb, bg: Rgb) -> Spec {
        Spec { fg, bg, attrs: 0 }
    }

    const fn bold(mut self) -> Spec {
        self.attrs |= Restyle::BOLD;
        self
    }

    /// The engine style this is, built through the engine's own public builders.
    fn style(self) -> Style {
        let mut s = Style::new()
            .fg(Color::rgb(self.fg.r, self.fg.g, self.fg.b))
            .bg(Color::rgb(self.bg.r, self.bg.g, self.bg.b));
        if self.attrs & Restyle::BOLD != 0 {
            s = s.bold();
        }
        if self.attrs & Restyle::DIM != 0 {
            s = s.dim();
        }
        if self.attrs & Restyle::ITALIC != 0 {
            s = s.italic();
        }
        if self.attrs & Restyle::REVERSE != 0 {
            s = s.reverse();
        }
        s
    }

    /// What this role becomes at `tier`, as a value two roles can be compared by.
    ///
    /// **The attribute bits are in the key**, not only the colours. That is what makes `Title`'s
    /// `BOLD` load-bearing: in much of the base16 corpus `base06 == base05`, so without the bold bit
    /// `Title`'s key collapses onto `Body`'s and the gate would be reporting a real defect as a
    /// palette accident.
    fn key(self, tier: ColorDepth) -> (u32, u32, u16) {
        (
            wire::key(self.fg, tier),
            wire::key(self.bg, tier),
            self.attrs,
        )
    }
}

/// Thirteen roles, paired inside the crate from sixteen colours.
///
/// **The door, and it is the only one.** A `Roles` cannot be built from thirteen styles — that would
/// be a route to thirteen arbitrary paints, which is the hole this type closed. It is built from
/// sixteen *colours*, and the pairing — which colour is a foreground, which is a background, which
/// carries an attribute — happens here.
///
/// ```compile_fail,E0423
/// use vitui_engine::Style;
/// use vitui_runtime::theme::Roles;
/// let _ = Roles([Style::new(); 13]);
/// ```
///
/// and the twin, which is the whole of what an application may do — including the `Box::leak` the
/// documents were worried about, which buys nothing here because there is no `&'static` in the
/// signature to leak *into*:
///
/// ```
/// use vitui_runtime::theme::{Density, GlyphSet, Role, Roles, Theme};
///
/// let palette: [u32; 16] = *Box::leak(Box::new([0x101010; 16]));
/// let roles = Roles::from_palette(&palette);
/// let theme = Theme::imported(roles, GlyphSet::Extended, Density::Cosy);
/// // Sixteen colours in, thirteen paired roles out, and the pairing was not the caller's.
/// assert_eq!(theme.paint(Role::Body), theme.paint(Role::Body));
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Roles([Spec; 13]);

impl Roles {
    /// Pair sixteen colours into thirteen roles.
    ///
    /// The sixteen are base16's `base00`–`base0F`: `base00`–`base07` a monotone ramp from page to
    /// foreground, `base08`–`base0F` eight accents. Each is `0xRRGGBB`.
    ///
    /// # The two mechanisms that are not renames
    ///
    /// **`pick`** takes the first candidate whose 256-colour index differs from every colour already
    /// spent. Three roles — [`Role::Dim`], [`Role::Disabled`] and [`Role::Border`] — are drawn from
    /// the middle of the ramp, and in much of the corpus two of those rungs are the same colour. The
    /// stub palette this replaced had `Dim` and `Border` both on `indexed(8)`: **two role names for
    /// one colour, which nine tickets drew through without noticing.** `pick` is what stops the
    /// default theme shipping its own gate's founding defect.
    ///
    /// **`readable_on`** returns whichever of `base05` and `base00` is further from the background in
    /// luminance. Nailing `base05` to every face puts `FaceActive` below a 3:1 contrast ratio in 218
    /// of 338 corpus schemes; picking per rung takes it to 55.
    ///
    /// `Density` and `GlyphSet` are **parameters and not derived** — no imported scheme carries
    /// either. `dark` *is* derived, from the page colour's luminance.
    pub fn from_palette(palette: &[u32; 16]) -> Roles {
        let c = |i: usize| {
            let v = palette[i];
            Rgb::new(
                u8::try_from((v >> 16) & 0xff).unwrap_or(0),
                u8::try_from((v >> 8) & 0xff).unwrap_or(0),
                u8::try_from(v & 0xff).unwrap_or(0),
            )
        };
        let page = c(0);
        let text = c(5);

        // Colours already spent, so `pick` can avoid them. Sixteen is the most there can be.
        let mut spent = [0u8; 16];
        let mut spent_n = 0usize;
        let spend = |rgb: Rgb, spent: &mut [u8; 16], n: &mut usize| {
            if *n < spent.len() {
                spent[*n] = wire::at_256(rgb);
                *n += 1;
            }
        };
        spend(page, &mut spent, &mut spent_n);
        spend(text, &mut spent, &mut spent_n);

        let pick = |candidates: &[Rgb], spent: &mut [u8; 16], n: &mut usize| -> Rgb {
            for &cand in candidates {
                let idx = wire::at_256(cand);
                if !spent[..*n].contains(&idx) {
                    spend(cand, spent, n);
                    return cand;
                }
            }
            // Nothing in the scheme is left. Derive one by stepping a mix of the scheme's own page
            // and text colours until it separates — 6% at a time, which is the smallest step that
            // moves a 256-colour index across the grey ramp.
            let mut t = 6u32;
            while t < 100 {
                let mixed = lerp(page, text, t);
                let idx = wire::at_256(mixed);
                if !spent[..*n].contains(&idx) {
                    spend(mixed, spent, n);
                    return mixed;
                }
                t += 6;
            }
            candidates.first().copied().unwrap_or(text)
        };

        // The monotone rungs, in the order the mapping table gives them.
        let dim = pick(&[c(4), c(3), c(6)], &mut spent, &mut spent_n);
        let disabled = pick(&[c(3), c(2), c(4)], &mut spent, &mut spent_n);
        let border = pick(&[c(2), c(1), c(3)], &mut spent, &mut spent_n);
        let face_bg = pick(&[c(1), c(2), c(3)], &mut spent, &mut spent_n);
        let hover_bg = pick(&[c(2), c(3), c(4)], &mut spent, &mut spent_n);
        let active_bg = pick(&[c(3), c(4), c(2)], &mut spent, &mut spent_n);

        let readable_on = |bg: Rgb| {
            if luminance(text).abs_diff(luminance(bg)) >= luminance(page).abs_diff(luminance(bg)) {
                text
            } else {
                page
            }
        };

        Roles([
            Spec::new(text, page),                        // Body
            Spec::new(c(6), page).bold(),                 // Title
            Spec::new(dim, page),                         // Dim
            Spec::new(disabled, page),                    // Disabled
            Spec::new(border, page),                      // Border
            Spec::new(readable_on(face_bg), face_bg),     // Face
            Spec::new(readable_on(hover_bg), hover_bg),   // FaceHover
            Spec::new(readable_on(active_bg), active_bg), // FaceActive
            Spec::new(readable_on(c(13)), c(13)),         // Selection, on base0D
            Spec::new(c(13), page),                       // Focus
            Spec::new(c(8), page),                        // Danger
            Spec::new(c(10), page),                       // Warn
            Spec::new(c(11), page),                       // Ok
        ])
    }
}

/// Integer luminance, on the same `2:4:3` weights the quantiser uses.
fn luminance(c: Rgb) -> u32 {
    (2 * u32::from(c.r) + 4 * u32::from(c.g) + 3 * u32::from(c.b)) / 9
}

/// `a` toward `b` by `t` per cent.
fn lerp(a: Rgb, b: Rgb, t: u32) -> Rgb {
    let m = |x: u8, y: u8| {
        let v = (u32::from(x) * (100 - t) + u32::from(y) * t) / 100;
        u8::try_from(v).unwrap_or(u8::MAX)
    };
    Rgb::new(m(a.r, b.r), m(a.g, b.g), m(a.b, b.b))
}

/// How much room a theme leaves around things. **It stays because it changes rectangles**, not
/// because it is comfortable.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Density {
    /// One cell of padding. The same 40×10 panel gives `(1, 1, 38, 8)`.
    Compact,
    /// Two cells. The same panel gives `(2, 2, 36, 6)`.
    #[default]
    Cosy,
}

impl Density {
    /// Horizontal padding, in cells.
    pub const fn pad_x(self) -> u16 {
        match self {
            Density::Compact => 1,
            Density::Cosy => 2,
        }
    }

    /// Vertical padding, in cells.
    pub const fn pad_y(self) -> u16 {
        match self {
            Density::Compact => 1,
            Density::Cosy => 2,
        }
    }

    /// The gap between two adjacent things.
    pub const fn gap(self) -> u16 {
        match self {
            Density::Compact => 0,
            Density::Cosy => 1,
        }
    }
}

/// A visual difference a component may want to rely on, narrowed to one bit by [`Theme::resolve`].
///
/// **A component branches on a bool and names neither axis** — not the colour depth, not the glyph
/// repertoire. That is the whole point: branching on the axes is what produced twenty-four
/// `GlyphSet::` occurrences across four component crates and nine divergent private fallback tables.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Distinction {
    /// A face under the pointer looks different from one at rest.
    Hover,
    /// The *animation into* a hover is visible — which is a different question, and the pair is why.
    Fade,
    /// [`Role::Danger`], [`Role::Warn`] and [`Role::Ok`] are three different things.
    Status,
}

impl Distinction {
    /// Every distinction. The set is deliberately small: one per mechanism already on the map, and
    /// components ticket 05 owns growing it.
    pub const ALL: [Distinction; 3] = [Distinction::Hover, Distinction::Fade, Distinction::Status];

    const fn bit(self) -> u16 {
        1 << (self as u16)
    }
}

// `Interest` used to be defined here, and it is `ctx`'s — spec §4 puts it there, and ticket 08 grew
// it from two bits to five plus `tracking()`. Re-exported rather than redefined, because **two types
// with one name across a module boundary is the review finding `GlyphSet` already carries**, and
// because `hover_interest` returns one: a theme answers *what should be declared*, and what a
// declaration means is the frame's business.
pub use crate::ctx::Interest;

/// The glyphs a theme spells for a component, so that no component owns a fallback table.
///
/// Seven entries, and the count is not this ticket's to grow: components ticket 05 declares the
/// demand set. **The theme owns the table because there was nowhere else to put one** — nine crates
/// of twelve grew a private `mod missing` with six glyph literals and a `match` on `GlyphSet`,
/// byte-identical in all nine.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Glyph {
    /// A vertical rule.
    VLine,
    /// A horizontal rule.
    HLine,
    /// A list marker.
    Bullet,
    /// A scrollbar thumb.
    Thumb,
    /// A scrollbar track.
    Track,
    /// A downward arrow.
    ArrowDown,
    /// A checkmark.
    Tick,
}

impl Glyph {
    /// Every glyph, which is what the two spelling gates iterate.
    pub const ALL: [Glyph; 7] = [
        Glyph::VLine,
        Glyph::HLine,
        Glyph::Bullet,
        Glyph::Thumb,
        Glyph::Track,
        Glyph::ArrowDown,
        Glyph::Tick,
    ];

    /// The spelling at each rung.
    ///
    /// **Two rows, not three.** The `Unicode` and `Extended` spellings are identical for every entry,
    /// and that is a decision rather than an omission: anything that would distinguish the top two
    /// rungs is a *branch* — the sub-rows per cell change, 8 for braille against 2 for blocks against
    /// 1 for ASCII, and so does the number of samples asked of the data. No table can carry that.
    const fn spell(self, set: GlyphSet) -> &'static str {
        match (self, set) {
            (Glyph::VLine, GlyphSet::Ascii) => "|",
            (Glyph::VLine, _) => "│",
            (Glyph::HLine, GlyphSet::Ascii) => "-",
            (Glyph::HLine, _) => "─",
            (Glyph::Bullet, GlyphSet::Ascii) => "*",
            (Glyph::Bullet, _) => "•",
            (Glyph::Thumb, GlyphSet::Ascii) => "#",
            (Glyph::Thumb, _) => "█",
            (Glyph::Track, GlyphSet::Ascii) => ":",
            (Glyph::Track, _) => "░",
            (Glyph::ArrowDown, GlyphSet::Ascii) => "v",
            (Glyph::ArrowDown, _) => "▼",
            (Glyph::Tick, GlyphSet::Ascii) => "x",
            (Glyph::Tick, _) => "✓",
        }
    }
}

/// A restyle written over **roles**, which is the only kind a component can write.
///
/// The engine's own `Restyle` cannot be passed through: its fields are `Option<Color>`, and a `Color`
/// is exactly what ADR 0018 forbids a component to name. The attribute masks are the engine's `u16`
/// unchanged, mirrored as constants here so a component never names `Restyle` either.
///
/// **A component names a role and nothing else**, which is where the third compile outcome lives.
/// Neither an engine `Style` nor an engine `Color` fits a field of this descriptor:
///
/// ```compile_fail,E0308
/// use vitui_engine::Style;
/// use vitui_runtime::Repaint;
/// let _ = Repaint { fg: Some(Style::new()), ..Default::default() };
/// ```
///
/// ```compile_fail,E0308
/// use vitui_engine::Color;
/// use vitui_runtime::Repaint;
/// let _ = Repaint { fg: Some(Color::rgb(1, 2, 3)), ..Default::default() };
/// ```
///
/// and the twin:
///
/// ```
/// use vitui_runtime::{Repaint, Role};
///
/// let repaint = Repaint {
///     fg: Some(Role::Danger),
///     set: Repaint::BOLD,
///     ..Default::default()
/// };
/// assert_eq!(repaint.fg, Some(Role::Danger));
/// ```
///
/// # And the fourth outcome is this type existing
///
/// The verb that used to take a *function over styles* is the one a colour could have escaped
/// through: six drawing verbs took a style as a parameter and were closed, and the seventh took a
/// closure that could return a style it invented. Replacing that closure with this descriptor means
/// **no verb returns a `Paint`**, so there is no hole left for a doctest to guard — the outcome is
/// structural. The closure's shape is kept as a regression case, so that reintroducing the verb
/// reintroduces the gate.
///
/// **`clear` is applied before `set`**, so an attribute named in both ends up set.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Repaint {
    /// The new foreground, or leave it.
    pub fg: Option<Role>,
    /// The new background, or leave it.
    pub bg: Option<Role>,
    /// Attributes to add.
    pub set: u16,
    /// Attributes to remove, applied first.
    pub clear: u16,
    /// The new underline colour, or leave it.
    pub ul: Option<Role>,
    /// The new hyperlink, or leave it.
    pub link: Option<Link>,
}

impl Repaint {
    /// Bold.
    pub const BOLD: u16 = Restyle::BOLD;
    /// Dim.
    pub const DIM: u16 = Restyle::DIM;
    /// Italic.
    pub const ITALIC: u16 = Restyle::ITALIC;
    /// Reverse video.
    pub const REVERSE: u16 = Restyle::REVERSE;
    /// Blink.
    pub const BLINK: u16 = Restyle::BLINK;
    /// Strikethrough.
    pub const STRIKETHROUGH: u16 = Restyle::STRIKETHROUGH;
    /// Concealed.
    pub const CONCEAL: u16 = Restyle::CONCEAL;
    /// Overline.
    pub const OVERLINE: u16 = Restyle::OVERLINE;
    /// A single underline.
    pub const UNDERLINE: u16 = Restyle::UNDERLINE;
    /// A double underline.
    pub const UNDERLINE_DOUBLE: u16 = Restyle::UNDERLINE_DOUBLE;
    /// A curly underline.
    pub const UNDERLINE_CURLY: u16 = Restyle::UNDERLINE_CURLY;
    /// A dotted underline.
    pub const UNDERLINE_DOTTED: u16 = Restyle::UNDERLINE_DOTTED;
    /// A dashed underline.
    pub const UNDERLINE_DASHED: u16 = Restyle::UNDERLINE_DASHED;

    /// Lower to the engine's descriptor, resolving every role against `theme`.
    ///
    /// `pub(crate)` — this is the seam, and a component on either side of it never sees both types.
    // `allow` and not `expect`, because under `cfg(test)` the lint does not fire and an unfulfilled
    // expectation is itself an error. The engine's `budget.rs` makes the same choice for the same
    // shape: different callers use different parts.
    //
    // **The caller is ticket 08's `Ctx` and it does not exist yet.** This is the seam — a component
    // writes a `Repaint` over roles and the crate lowers it — and until the draw context lands there
    // is nothing above it to call. Made `pub(crate)` rather than `pub` deliberately: a public `lower`
    // would hand a component an `engine::Restyle` with `Color`s in it, which is the thing ADR 0018
    // keeps out of a component's hands.
    #[allow(dead_code)]
    pub(crate) fn lower(self, theme: &Theme) -> Restyle {
        let colour = |r: Option<Role>| r.map(|r| theme.spec(r).style_fg());
        Restyle {
            fg: colour(self.fg),
            bg: self.bg.map(|r| theme.spec(r).style_bg()),
            set: self.set,
            clear: self.clear,
            ul: colour(self.ul),
            link: self.link.map(|l| l.0),
        }
    }
}

impl Spec {
    #[allow(
        dead_code,
        reason = "read by `Repaint::lower`, whose caller is ticket 08's `Ctx`"
    )]
    fn style_fg(self) -> Color {
        Color::rgb(self.fg.r, self.fg.g, self.fg.b)
    }

    #[allow(
        dead_code,
        reason = "read by `Repaint::lower`, whose caller is ticket 08's `Ctx`"
    )]
    fn style_bg(self) -> Color {
        Color::rgb(self.bg.r, self.bg.g, self.bg.b)
    }
}

/// A palette, a glyph repertoire, a density, and what the terminal will actually show of them.
///
/// **Heap-free.** Thirteen role specs, thirteen precomputed styles, two enums, a bitmask, a bool, a
/// tier and a [`Revision`] — no `Vec`, because the obvious `Vec<Style>` palette allocates on exactly
/// the frame a swap becomes visible. A swap is **14.30 ns and zero allocations**; owning the styles
/// rather than borrowing `&'static` ones costs **+0.32 ns a lookup and +0.015 µs a frame**, and buys
/// a theme that can come off a disk without `Box::leak`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Theme {
    specs: [Spec; 13],
    styles: [Style; 13],
    /// What each role becomes at [`Theme::tier`], precomputed by [`Theme::resolve`].
    ///
    /// **Measured into existence.** The first version quantised inside
    /// [`Theme::roles_differ_on_wire`], which put it at **42.4 ns a call against `shows`'s 0.93** —
    /// and `shows` is a bit test, so a 45× gap meant the two answers a component chooses between
    /// were not comparable at all. Quantising thirteen roles once at `resolve` makes the question two
    /// array reads and a compare, which is what *resolved once* was supposed to mean. It costs 156
    /// bytes a theme and the theme is still heap-free.
    keys: [(u32, u32, u16); 13],
    glyphs: GlyphSet,
    density: Density,
    dark: bool,
    tier: ColorDepth,
    distinctions: u16,
    rev: Revision,
}

/// Catppuccin Mocha, as base16. The default palette, and **data rather than a decision**: components
/// ticket 05 replaces it and no gate here reads a literal from it.
pub const CATPPUCCIN_MOCHA: [u32; 16] = [
    0x1e1e2e, 0x181825, 0x313244, 0x45475a, 0x585b70, 0xcdd6f4, 0xf5e0dc, 0xb4befe, 0xf38ba8,
    0xfab387, 0xf9e2af, 0xa6e3a1, 0x94e2d5, 0x89b4fa, 0xcba6f7, 0xf2cdcd,
];

impl Default for Theme {
    fn default() -> Theme {
        Theme::authored(&CATPPUCCIN_MOCHA, GlyphSet::default(), Density::default())
    }
}

impl Theme {
    /// Build a theme from paired roles.
    ///
    /// Takes [`Roles`] and not `[Style; 13]`, and that is the door being shut: thirteen styles in
    /// would be thirteen arbitrary paints out.
    pub fn imported(roles: Roles, glyphs: GlyphSet, density: Density) -> Theme {
        let specs = roles.0;
        let dark = luminance(specs[Role::Body.index()].bg) < 128;
        Theme {
            specs,
            styles: core::array::from_fn(|i| specs[i].style()),
            // At `ColorDepth::None` until `resolve` says otherwise, which is what makes an unresolved
            // theme claim nothing rather than claim everything.
            keys: core::array::from_fn(|i| specs[i].key(ColorDepth::None)),
            glyphs,
            density,
            dark,
            // Unresolved until `resolve` is called: the conservative tier, so a theme nobody resolved
            // claims no distinctions rather than claiming all of them.
            tier: ColorDepth::None,
            distinctions: 0,
            rev: Revision::fresh(),
        }
    }

    /// Build a theme from sixteen colours, which is the convenience spelling.
    ///
    /// `dark` is derived from the palette rather than taken as a parameter.
    pub fn authored(palette: &[u32; 16], glyphs: GlyphSet, density: Density) -> Theme {
        Theme::imported(Roles::from_palette(palette), glyphs, density)
    }

    /// Narrow every role to what `tier` will actually show, and set the [`Distinction`] bits from
    /// that.
    ///
    /// **Called once, at construction, and never per frame** — 4.16 ns. It is the whole of what turns
    /// a colour fact into a routing fact: see the module documentation.
    pub fn resolve(mut self, tier: ColorDepth) -> Theme {
        self.tier = tier;
        // Thirteen quantisations, once. Everything below reads them.
        self.keys = core::array::from_fn(|i| self.specs[i].key(tier));
        let differs = |a: Role, b: Role| self.keys[a.index()] != self.keys[b.index()];
        let mut bits = 0u16;
        if differs(Role::Face, Role::FaceHover) {
            bits |= Distinction::Hover.bit();
        }
        // **Two answers, not one.** A hover being visible does not mean the *animation into* it is:
        // a fade is a sequence of intermediate colours, and at 256 colours or fewer the intermediates
        // collapse onto the endpoints. Of 338 corpus themes 338 show a hover at C256 and **165** show
        // the animation, so a component reading only the first pays nineteen wakeups for a switch in
        // 173 of them.
        if differs(Role::Face, Role::FaceHover) && tier == ColorDepth::TrueColor {
            bits |= Distinction::Fade.bit();
        }
        if differs(Role::Danger, Role::Warn) && differs(Role::Warn, Role::Ok) {
            bits |= Distinction::Status.bit();
        }
        self.distinctions = bits;
        self.rev = Revision::fresh();
        self
    }

    /// Declare a glyph repertoire. **Declared, never probed** (ADR 0010).
    pub fn with_glyphs(mut self, set: GlyphSet) -> Theme {
        self.glyphs = set;
        self.rev = Revision::fresh();
        self
    }

    /// The paint for a role.
    pub fn paint(&self, r: Role) -> Paint {
        Paint(self.styles[r.index()])
    }

    /// A paint from two colours the caller names.
    ///
    /// # Two obligations, and they are the reason this is documented rather than just provided
    ///
    /// **A paint made this way carries no tier guarantee.** [`Theme::resolve`] narrowed the thirteen
    /// roles; it knows nothing about this. So the component that calls `custom` owes a branch on the
    /// terminal's own capabilities — the two colours it picked may be one colour on the wire, and no
    /// [`Distinction`] bit covers them.
    ///
    /// **It is a per-call cost and not a lookup.** [`Theme::paint`] reads an array; this builds a
    /// style. A component that calls it per cell is paying per cell, so a component that calls it at
    /// all should publish its own call census.
    ///
    /// It exists because *no route to an arbitrary paint*, read literally, makes a chart's fifth
    /// series, a photograph's pixel and a sentinel probe inexpressible. `&self` is the mechanism that
    /// keeps the real invariant: **a paint cannot exist without a live `&Theme`.**
    pub fn custom(&self, fg: Rgb, bg: Rgb) -> Paint {
        Paint(Spec::new(fg, bg).style())
    }

    /// A paint `t` of the way from `a` to `b`, where `t` is clamped to `0.0..=1.0`.
    ///
    /// Here rather than as `impl Lerp for Paint` or `Paint::derive`, because both of those fail: a
    /// trait implementation cannot construct the newtype (`E0423`) and an inherent method cannot read
    /// its field from outside the crate (`E0624`). A tween needs the theme, so the theme owns the
    /// verb — 7.58 ns, zero allocations.
    pub fn mix(&self, a: Role, b: Role, t: f32) -> Paint {
        let t = u32::try_from((t.clamp(0.0, 1.0) * 100.0) as i64)
            .unwrap_or(0)
            .min(100);
        let (x, y) = (self.specs[a.index()], self.specs[b.index()]);
        Paint(
            Spec {
                fg: lerp(x.fg, y.fg, t),
                bg: lerp(x.bg, y.bg, t),
                // The endpoint's attributes, not a blend of them: half of bold is not a thing.
                attrs: if t >= 50 { y.attrs } else { x.attrs },
            }
            .style(),
        )
    }

    /// How this theme spells a glyph. 0.661 ns.
    pub const fn glyph(&self, g: Glyph) -> &'static str {
        g.spell(self.glyphs)
    }

    /// Whether a distinction is visible on the terminal this theme was resolved for. 0.524 ns.
    ///
    /// A bit test against a mask, against **8.22 ns** for deciding it per draw — 15.7×. But the cost
    /// is the smaller argument: the screens are cell-identical at all nine matrix cells, so the choice
    /// is **cost and vocabulary, never correctness**. Deciding per draw forces a component to name the
    /// axis, and naming the axis is what produced nine divergent fallback tables.
    pub const fn shows(&self, d: Distinction) -> bool {
        self.distinctions & d.bit() != 0
    }

    /// Whether two roles are distinguishable on the wire, at the tier this theme was resolved for.
    ///
    /// # The obligation, which is the point of exposing this
    ///
    /// **A distinction survives the whole matrix if and only if it is carried on both axes.** A
    /// component reading `false` owes a *second axis* — a glyph, a rule, a position — and **never a
    /// darker colour**, because a darker colour is the same axis again and will collapse in the same
    /// place.
    ///
    /// And a count taken at sixteen colours is a **lower bound rather than a measurement**: nothing
    /// in this process can read the operator's palette, and half of ten real terminal profiles spell
    /// at least one index twice.
    pub const fn roles_differ_on_wire(&self, a: Role, b: Role) -> bool {
        let (x, y) = (self.keys[a.index()], self.keys[b.index()]);
        x.0 != y.0 || x.1 != y.1 || x.2 != y.2
    }

    /// What a component should declare when it registers a hoverable region.
    ///
    /// **The runtime does not edit a declared interest** (ADR 0021). This is an answer, not a
    /// correction: a component that writes `Interest::HOVER` against a flat theme still gets motion
    /// tracking, and is therefore visibly wrong rather than invisibly fixed.
    pub const fn hover_interest(&self) -> Interest {
        if self.shows(Distinction::Hover) {
            Interest::HOVER
        } else {
            Interest::NONE
        }
    }

    /// Whether the page is dark. Ticket 13's scrim reads this and nothing else mints it.
    pub const fn is_dark(&self) -> bool {
        self.dark
    }

    /// The density.
    pub const fn density(&self) -> Density {
        self.density
    }

    /// The glyph repertoire this theme was told about.
    pub const fn glyphs(&self) -> GlyphSet {
        self.glyphs
    }

    /// The tier this theme was resolved for.
    pub const fn tier(&self) -> ColorDepth {
        self.tier
    }

    /// A revision that changes whenever this theme does, for a memo whose value is made of paints.
    ///
    /// From the crate's **process-global** counter, deliberately: a per-theme counter would put two
    /// themes both at revision one, which is the defect [`crate::data::Revision`] exists on the other
    /// side of.
    pub const fn revision(&self) -> Revision {
        self.rev
    }

    #[allow(
        dead_code,
        reason = "read by `Repaint::lower`, whose caller is ticket 08's `Ctx`"
    )]
    fn spec(&self, r: Role) -> Spec {
        self.specs[r.index()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tier the reports and most gates run at.
    const TIERS: [ColorDepth; 4] = [
        ColorDepth::TrueColor,
        ColorDepth::Indexed256,
        ColorDepth::Ansi16,
        ColorDepth::None,
    ];

    fn resolved(tier: ColorDepth) -> Theme {
        Theme::default().resolve(tier)
    }

    /// A paint is eight bytes, the same eight the engine's style occupies. **The newtype is free.**
    #[test]
    fn a_paint_is_the_same_size_as_a_style() {
        assert_eq!(size_of::<Paint>(), size_of::<Style>());
        assert_eq!(size_of::<Paint>(), 8);
    }

    /// **The default theme does not ship its own gate's founding defect.**
    ///
    /// The palette this replaced had `Dim` and `Border` both on `indexed(8)` — two role names for one
    /// colour, which nine tickets drew through without noticing. `pick` is what separates them, and
    /// this is the assertion that says it worked.
    #[test]
    fn the_default_theme_has_no_truecolor_collapse() {
        let theme = resolved(ColorDepth::TrueColor);
        let mut collapsed = Vec::new();
        for (i, &a) in Role::ALL.iter().enumerate() {
            for &b in &Role::ALL[i + 1..] {
                if !theme.roles_differ_on_wire(a, b) {
                    collapsed.push((a, b));
                }
            }
        }
        assert!(
            collapsed.is_empty(),
            "the default palette collapses at truecolor: {collapsed:?}"
        );
    }

    /// **The pair count is a relation, not an equality**, because the number belongs to the palette
    /// and components ticket 05 replaces the palette.
    ///
    /// What is gated is the shape: narrowing never *adds* a distinction, and no colour at all
    /// collapses everything a colour could have carried.
    #[test]
    fn narrowing_never_adds_a_distinction() {
        let counts: Vec<usize> = TIERS
            .iter()
            .map(|&tier| {
                let theme = resolved(tier);
                let mut n = 0;
                for (i, &a) in Role::ALL.iter().enumerate() {
                    for &b in &Role::ALL[i + 1..] {
                        if !theme.roles_differ_on_wire(a, b) {
                            n += 1;
                        }
                    }
                }
                n
            })
            .collect();
        let [t_true, t_256, t_16, t_none] = [counts[0], counts[1], counts[2], counts[3]];
        assert!(
            t_true <= t_256 && t_256 <= t_16,
            "collapses must be monotone in the narrowing: {t_true} / {t_256} / {t_16}"
        );
        // 78 unordered pairs of thirteen roles, and `Title`'s BOLD is why this is not all of them:
        // with no colour at all, two roles differ only if their attributes do.
        let total = Role::ALL.len() * (Role::ALL.len() - 1) / 2;
        assert_eq!(total, 78);
        assert!(
            t_none < total,
            "even with no colour, BOLD keeps Title apart from something"
        );
        assert!(
            t_none > t_16,
            "no colour collapses more than sixteen colours"
        );
    }

    /// `Title`'s bold bit is load-bearing rather than decorative.
    ///
    /// In much of the base16 corpus `base06 == base05`, so without the attribute in the key `Title`
    /// collapses onto `Body` and the gate above would report a palette accident as a real defect.
    #[test]
    fn titles_bold_bit_is_what_keeps_it_apart_from_body() {
        // A palette where the two colours are deliberately identical, which is the corpus case.
        let mut palette = CATPPUCCIN_MOCHA;
        palette[6] = palette[5];
        let theme = Theme::authored(&palette, GlyphSet::Extended, Density::Cosy)
            .resolve(ColorDepth::TrueColor);
        assert!(
            theme.roles_differ_on_wire(Role::Body, Role::Title),
            "identical colours, and the bold bit is the whole difference"
        );
    }

    /// **A theme nobody resolved claims no distinction**, rather than claiming all of them.
    #[test]
    fn an_unresolved_theme_claims_nothing() {
        let theme = Theme::default();
        for d in Distinction::ALL {
            assert!(!theme.shows(d), "{d:?} was claimed before resolve");
        }
        assert_eq!(theme.hover_interest(), Interest::NONE);
    }

    /// `resolve` sets the hover bit when the two faces are distinguishable, and `hover_interest`
    /// follows it.
    #[test]
    fn hover_interest_follows_what_the_terminal_can_show() {
        let rich = resolved(ColorDepth::TrueColor);
        assert!(rich.shows(Distinction::Hover));
        assert!(rich.hover_interest().wants_hover());

        let flat = resolved(ColorDepth::None);
        assert!(!flat.shows(Distinction::Hover));
        assert!(!flat.hover_interest().wants_hover());
        assert_eq!(flat.hover_interest(), Interest::NONE);
    }

    /// **Hover and fade stay two answers**, and a theme that shows the first need not show the
    /// second.
    ///
    /// A fade is a sequence of intermediate colours; at 256 colours the intermediates collapse onto
    /// the endpoints. A component reading only `shows(Hover)` pays wakeups for an animation nobody
    /// sees.
    #[test]
    fn hover_and_fade_are_two_answers() {
        let c256 = resolved(ColorDepth::Indexed256);
        assert!(
            c256.shows(Distinction::Hover),
            "the two faces are 237 and 239"
        );
        assert!(
            !c256.shows(Distinction::Fade),
            "and the colours between them are not"
        );
        let truecolor = resolved(ColorDepth::TrueColor);
        assert!(truecolor.shows(Distinction::Hover) && truecolor.shows(Distinction::Fade));
    }

    /// A traffic light is monochrome at sixteen colours, which is what `Status` is for.
    #[test]
    fn a_traffic_light_is_monochrome_at_sixteen_colours() {
        assert!(resolved(ColorDepth::TrueColor).shows(Distinction::Status));
        let c16 = resolved(ColorDepth::Ansi16);
        assert!(
            !c16.shows(Distinction::Status)
                || !c16.roles_differ_on_wire(Role::Danger, Role::Warn)
                || !c16.roles_differ_on_wire(Role::Warn, Role::Ok),
            "at C16 the three status colours are not three colours"
        );
    }

    /// `Repaint` lowers to the engine's descriptor with the roles resolved.
    ///
    /// The gate is that the lowered colours are the theme's and not the caller's: a component wrote
    /// `Role::Danger` and what reaches the engine is a `Color`, which the component never named.
    #[test]
    fn a_repaint_lowers_to_the_engines_descriptor() {
        let theme = resolved(ColorDepth::TrueColor);
        let repaint = Repaint {
            fg: Some(Role::Danger),
            bg: Some(Role::Body),
            set: Repaint::BOLD | Repaint::UNDERLINE,
            clear: Repaint::DIM,
            ul: Some(Role::Warn),
            link: None,
        };
        let lowered = repaint.lower(&theme);
        assert_eq!(lowered.set, Repaint::BOLD | Repaint::UNDERLINE);
        assert_eq!(lowered.clear, Repaint::DIM);
        assert!(lowered.fg.is_some() && lowered.bg.is_some() && lowered.ul.is_some());
        assert!(lowered.link.is_none());
        // A `Repaint` that names nothing lowers to a descriptor that changes nothing.
        assert_eq!(Repaint::default().lower(&theme), Restyle::default());
    }

    /// The paint a role hands out is the style the theme built for it.
    #[test]
    fn a_role_paints_the_style_the_theme_built() {
        let theme = Theme::default();
        for r in Role::ALL {
            assert_eq!(theme.paint(r).style(), theme.styles[r.index()]);
        }
    }

    /// `custom` mints a paint from two colours, and needs a live theme to do it.
    #[test]
    fn custom_mints_a_paint_from_two_colours() {
        let theme = Theme::default();
        let a = theme.custom(Rgb::new(1, 2, 3), Rgb::new(4, 5, 6));
        let b = theme.custom(Rgb::new(1, 2, 3), Rgb::new(4, 5, 6));
        assert_eq!(a, b, "the same two colours are the same paint");
        assert_ne!(a, theme.custom(Rgb::new(1, 2, 4), Rgb::new(4, 5, 6)));
    }

    /// `mix` interpolates, clamps its parameter, and hits both endpoints exactly.
    #[test]
    fn mix_hits_both_endpoints_and_clamps() {
        let theme = Theme::default();
        assert_eq!(
            theme.mix(Role::Body, Role::Danger, 0.0),
            theme.paint(Role::Body)
        );
        assert_eq!(
            theme.mix(Role::Body, Role::Danger, 1.0),
            theme.paint(Role::Danger)
        );
        assert_eq!(
            theme.mix(Role::Body, Role::Danger, -5.0),
            theme.paint(Role::Body),
            "clamped below"
        );
        assert_eq!(
            theme.mix(Role::Body, Role::Danger, 5.0),
            theme.paint(Role::Danger),
            "clamped above"
        );
        // And the middle is neither end.
        let half = theme.mix(Role::Body, Role::Danger, 0.5);
        assert_ne!(half, theme.paint(Role::Body));
        assert_ne!(half, theme.paint(Role::Danger));
    }

    /// **Two counts over `Glyph::ALL × GlyphSet`, both zero**: no spelling is other than one cell
    /// wide, and none is blank.
    ///
    /// **Absence is not representable**, and the reason it needs a gate is that every counter
    /// approves of a blank fallback: it is no slower, it writes 77 fewer cells, it marks the same
    /// damage and it allocates the same. Only the rendered surface disagrees — 545 cells against 135,
    /// and 80 of 80 rows touched at ASCII against 77.
    #[test]
    fn no_glyph_spelling_is_blank_or_wider_than_one_cell() {
        let mut blank = 0;
        let mut wrong_width = 0;
        for set in [GlyphSet::Ascii, GlyphSet::Unicode, GlyphSet::Extended] {
            let theme = Theme::default().with_glyphs(set);
            for g in Glyph::ALL {
                let s = theme.glyph(g);
                if s.is_empty() {
                    blank += 1;
                }
                if crate::layout::text::width(s) != 1 {
                    wrong_width += 1;
                }
            }
        }
        assert_eq!(blank, 0, "a blank spelling is an absence nobody can see");
        assert_eq!(wrong_width, 0, "every spelling is exactly one cell");
    }

    /// The table has **two rows, not three**: `Unicode` and `Extended` spell every entry the same.
    #[test]
    fn the_glyph_table_has_two_rows_and_not_three() {
        let unicode = Theme::default().with_glyphs(GlyphSet::Unicode);
        let extended = Theme::default().with_glyphs(GlyphSet::Extended);
        let ascii = Theme::default().with_glyphs(GlyphSet::Ascii);
        let mut differ = 0;
        for g in Glyph::ALL {
            if unicode.glyph(g) != extended.glyph(g) {
                differ += 1;
            }
            assert!(
                ascii.glyph(g).is_ascii(),
                "{g:?} is not ASCII at the ASCII rung"
            );
        }
        assert_eq!(
            differ, 0,
            "anything that would distinguish the top two rungs is a branch, not a table row"
        );
    }

    /// The glyph ladder is downward-closed, which is what `Ord` on `GlyphSet` is for.
    #[test]
    fn the_glyph_ladder_is_ordered() {
        assert!(GlyphSet::Ascii < GlyphSet::Unicode);
        assert!(GlyphSet::Unicode < GlyphSet::Extended);
    }

    /// A density changes rectangles, which is the only reason it exists.
    #[test]
    fn a_density_changes_rectangles() {
        assert_eq!(Density::Compact.pad_x(), 1);
        assert_eq!(Density::Cosy.pad_x(), 2);
        assert_ne!(Density::Compact.gap(), Density::Cosy.gap());
        // The panel from the theme ticket: 40x10 inset by each density.
        use crate::layout::rect;
        let panel = vitui_engine::Rect::new(0, 0, 40, 10);
        assert_eq!(
            rect::inset(panel, Density::Compact.pad_x()),
            vitui_engine::Rect::new(1, 1, 38, 8)
        );
        assert_eq!(
            rect::inset(panel, Density::Cosy.pad_x()),
            vitui_engine::Rect::new(2, 2, 36, 6)
        );
    }

    /// A theme is dark or light according to its page, and `is_dark` is what a scrim reads.
    #[test]
    fn is_dark_follows_the_page() {
        assert!(Theme::default().is_dark(), "Mocha is a dark scheme");
        let mut light = CATPPUCCIN_MOCHA;
        light[0] = 0xffffff;
        assert!(!Theme::authored(&light, GlyphSet::Extended, Density::Cosy).is_dark());
    }

    /// **Every change to a theme is a new revision, and it comes from the process-global counter.**
    ///
    /// A per-theme counter would put two themes both at revision one, which is the defect
    /// `crate::data::Revision` exists on the other side of.
    #[test]
    fn every_change_is_a_new_revision_from_the_global_counter() {
        let a = Theme::default();
        let b = Theme::default();
        assert_ne!(a.revision(), b.revision(), "two themes, two revisions");
        let resolved = a.resolve(ColorDepth::TrueColor);
        assert_ne!(resolved.revision(), a.revision());
        assert_ne!(
            resolved.with_glyphs(GlyphSet::Ascii).revision(),
            resolved.revision()
        );
    }

    /// A swap is a move, and the new theme's roles are the new theme's.
    #[test]
    fn a_swap_replaces_every_role() {
        let mut light = CATPPUCCIN_MOCHA;
        light[0] = 0xffffff;
        light[5] = 0x000000;
        let dark = resolved(ColorDepth::TrueColor);
        let pale = Theme::authored(&light, GlyphSet::Extended, Density::Compact)
            .resolve(ColorDepth::TrueColor);
        assert_ne!(dark.paint(Role::Body), pale.paint(Role::Body));
        assert_ne!(dark.density(), pale.density());
        assert!(dark.is_dark() && !pale.is_dark());
    }

    /// `resolve` is idempotent at one tier and re-derivable at another.
    #[test]
    fn resolve_is_idempotent_at_one_tier() {
        let once = resolved(ColorDepth::Indexed256);
        let twice = once.resolve(ColorDepth::Indexed256);
        for d in Distinction::ALL {
            assert_eq!(once.shows(d), twice.shows(d));
        }
        assert_eq!(once.tier(), twice.tier());
    }
}

#[cfg(test)]
mod seam {
    //! Gates about what this module is *not*, which is the half that needs a scan rather than a call.

    /// **Nothing in `theme` names crossterm** (ADR 0001), and nothing in it names a Unicode table
    /// either.
    ///
    /// A source scan, because both are claims about absent code. The needles are split and joined at
    /// run time for the reason `crate::layout::text`'s scanner found the hard way: a scanner whose
    /// needles are plain literals fails on its own source file, and excluding its own file is the easy
    /// wrong fix.
    #[test]
    // The name says `the_backend` rather than spelling the crate, and that is the third round of the
    // same joke: the needles are split, the comments are skipped, and the scan still found the word —
    // in this function's own identifier. A scanner for absent code keeps finding itself, and each time
    // the fix is to stop being an exception rather than to add one.
    fn nothing_here_names_the_backend() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/src/theme");
        let mut files = vec![std::path::PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/theme.rs"
        ))];
        if let Ok(dir) = std::fs::read_dir(root) {
            for entry in dir {
                let path = entry.expect("a readable entry").path();
                if path.extension().is_some_and(|e| e == "rs") {
                    files.push(path);
                }
            }
        }
        assert!(files.len() >= 2, "only {} files scanned", files.len());
        for path in files {
            let source = std::fs::read_to_string(&path).expect("readable");
            // **Code, not prose.** The needles are already split and joined at run time, and that is
            // still not enough: this module's own documentation says the word, because saying what a
            // module does not name is half of saying what it does. So the scan is over lines that are
            // not comments — which is the same shape `layout::text`'s no-trait scan uses, arrived at
            // for the same reason one file later.
            let source: String = source
                .lines()
                .map(str::trim_start)
                .filter(|line| !line.starts_with("//"))
                .collect::<Vec<_>>()
                .join("\n");
            for (head, tail) in [("cross", "term"), ("Cross", "term"), ("Key", "Modifiers")] {
                let needle = format!("{head}{tail}");
                assert!(
                    !source.contains(&needle),
                    "{} names `{needle}` — the engine is the only thing that may, and a theme is \
                     data (ADR 0001)",
                    path.display()
                );
            }
        }
    }

    /// **What a traffic light actually loses at sixteen colours**, pair by pair, with one of the two
    /// documented pairs reproducing and the other not.
    ///
    /// The theme documents say `Danger`, `Warn` and `Ok` "all quantise to bright white at sixteen
    /// colours, so a traffic light is monochrome". Against this palette and the shipped engine's
    /// arithmetic:
    ///
    /// | role | base16 | C16 index |
    /// |---|---|---|
    /// | `Danger` | `base08` `#f38ba8` | **8** |
    /// | `Warn` | `base0A` `#f9e2af` | **7** |
    /// | `Ok` | `base0B` `#a6e3a1` | **7** |
    ///
    /// So **`(Warn, Ok)` collapses and `(Danger, Warn)` does not**, and neither lands on *bright*
    /// white — index 7 is white and 15 is the bright one. Mocha's pink is dark enough to fall on the
    /// grey instead.
    ///
    /// What is asserted is therefore the part that is a property of the mechanism: **a traffic light
    /// loses at least one of its two distinctions**, and it had both at truecolor. Which pair goes is
    /// palette data, and spec §20's own rule is that a number belonging to the data must be a relation
    /// rather than an equality — so asserting `(Danger, Warn)` here would be a gate that gets edited
    /// when components ticket 05 replaces the palette, rather than one that gets fixed.
    #[test]
    fn a_traffic_light_loses_a_distinction_at_sixteen_colours() {
        use super::{ColorDepth, Role, Theme};

        let rich = Theme::default().resolve(ColorDepth::TrueColor);
        assert!(rich.roles_differ_on_wire(Role::Danger, Role::Warn));
        assert!(rich.roles_differ_on_wire(Role::Warn, Role::Ok));

        let c16 = Theme::default().resolve(ColorDepth::Ansi16);
        let kept = usize::from(c16.roles_differ_on_wire(Role::Danger, Role::Warn))
            + usize::from(c16.roles_differ_on_wire(Role::Warn, Role::Ok));
        assert!(
            kept < 2,
            "a traffic light kept both distinctions at sixteen colours, which no palette on the \
             corpus does — either the arithmetic is wrong or this palette is not a palette"
        );
        // The pair the documents name second is the one that reproduces.
        assert!(
            !c16.roles_differ_on_wire(Role::Warn, Role::Ok),
            "Warn and Ok are both index 7 at C16"
        );
    }

    /// `hover_interest` is a pure function of the theme, asked and never told.
    ///
    /// The compile outcomes on `Interest` are what make this structural; this is the behavioural half:
    /// the same theme gives the same answer, and there is no argument by which a caller could ask for
    /// a different one.
    #[test]
    fn hover_interest_is_asked_and_never_told() {
        use super::{ColorDepth, Distinction, Interest, Theme};

        for tier in [ColorDepth::TrueColor, ColorDepth::Ansi16, ColorDepth::None] {
            let theme = Theme::default().resolve(tier);
            let once = theme.hover_interest();
            assert_eq!(
                once,
                theme.hover_interest(),
                "not a pure function at {tier:?}"
            );
            assert_eq!(
                once.wants_hover(),
                theme.shows(Distinction::Hover),
                "the answer is the distinction and nothing else"
            );
            assert!(once == Interest::HOVER || once == Interest::NONE);
        }
    }
}
