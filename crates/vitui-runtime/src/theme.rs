//! Theme: a component names a **role**, and can never construct a **paint**.
//!
//! A component names a role and never a colour; a declared interest is never rewritten; the glyph
//! ladder is declared and never probed.
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
//! chart's fifth series, a photograph's pixel and a sentinel probe inexpressible. **What the rule
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
//! **The runtime does not edit a declared interest**. The component reads
//! [`Theme::hover_interest`] and writes what it says; a component that ignores it still gets
//! `Motion`, and is therefore **visibly wrong rather than invisibly corrected.**
//!
//! # The standard set and live switching
//!
//! [`Scheme`] is one shipped application palette as `.rodata` — [`Roles::from_palette`] is a `const
//! fn`, so the sixteen colours are paired into thirteen roles at compile time — and [`Themes`] is the
//! registry that holds a set of them, **as application state and never as a field beside the frame
//! services**. [`STANDARD`] is the fourteen this crate ships. See [`registry`] for the `E0502` that
//! decides where the registry lives, and [`Theme::memo_key`] for the one rule a memo owes a swap.

pub mod registry;
pub mod schemes;
pub mod wire;

use vitui_engine::{Color, ColorDepth, Restyle, Rgb, Style};

pub use vitui_engine::GlyphSet;

pub use registry::{Scheme, Themes};
pub use schemes::STANDARD;

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

// ── the `Link` newtype is not here, and it has nothing left to wrap ──────────────────────────────
//
// It was `Link(LinkId)`, a runtime newtype over the engine's opaque handle, and its whole reason was
// *no `u32` crosses the seam*. Engine architecture ticket 21 put the URI at the drawing verb, so the
// engine's own `Link<'a>` is `None | Uri(&str)` — there is no handle, no number, and nothing for a
// newtype to hide. Wrapping it would only mean a component could not write the URI it already has.
//
// So the engine's type is re-exported here under the name this crate's own spec §3 uses for it. The
// name was taken by the newtype and this frees it, which is why `Hyperlink<'a>` — the sanctioned
// fallback spelling — is not needed.
pub use vitui_engine::Link;

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
    ///
    /// # It is a `const fn`, and that is what puts a shipped theme in `.rodata`
    ///
    /// Every step above — the pick, the derivation, the luminance pairing and the 256-colour
    /// quantiser they all consult — runs at compile time, so [`crate::theme::Scheme`] holds a
    /// finished [`Roles`] rather than sixteen numbers waiting for an importer. There is no parser in
    /// this crate, no `Vec` on the path, and the heap-free requirement is met by construction rather
    /// than by care:
    ///
    /// ```
    /// use vitui_runtime::theme::Roles;
    ///
    /// const NORD: Roles = Roles::from_palette(&[
    ///     0x2e3440, 0x3b4252, 0x434c5e, 0x4c566a, 0xd8dee9, 0xe5e9f0, 0xeceff4, 0x8fbcbb,
    ///     0xbf616a, 0xd08770, 0xebcb8b, 0xa3be8c, 0x88c0d0, 0x81a1c1, 0xb48ead, 0x5e81ac,
    /// ]);
    /// assert_eq!(NORD, Roles::from_palette(&[
    ///     0x2e3440, 0x3b4252, 0x434c5e, 0x4c566a, 0xd8dee9, 0xe5e9f0, 0xeceff4, 0x8fbcbb,
    ///     0xbf616a, 0xd08770, 0xebcb8b, 0xa3be8c, 0x88c0d0, 0x81a1c1, 0xb48ead, 0x5e81ac,
    /// ]));
    /// ```
    pub const fn from_palette(palette: &[u32; 16]) -> Roles {
        let page = channel(palette[0]);
        let text = channel(palette[5]);

        // Colours already spent, so `pick` can avoid them. Sixteen is the most there can be.
        let mut spent = [0u8; 16];
        let mut n = 0usize;
        spend(page, &mut spent, &mut n);
        spend(text, &mut spent, &mut n);

        // The monotone rungs, in the order the mapping table gives them. Written out rather than
        // built by a helper because a closure cannot be *called* inside a `const fn`, and the table
        // is the part a reader wants to see anyway.
        let dim = pick(ramp(palette, 4, 3, 6), page, text, &mut spent, &mut n);
        let disabled = pick(ramp(palette, 3, 2, 4), page, text, &mut spent, &mut n);
        let border = pick(ramp(palette, 2, 1, 3), page, text, &mut spent, &mut n);
        let face_bg = pick(ramp(palette, 1, 2, 3), page, text, &mut spent, &mut n);
        let hover_bg = pick(ramp(palette, 2, 3, 4), page, text, &mut spent, &mut n);
        let active_bg = pick(ramp(palette, 3, 4, 2), page, text, &mut spent, &mut n);
        let selection_bg = channel(palette[13]);

        Roles([
            Spec::new(text, page),                                          // Body
            Spec::new(channel(palette[6]), page).bold(),                    // Title
            Spec::new(dim, page),                                           // Dim
            Spec::new(disabled, page),                                      // Disabled
            Spec::new(border, page),                                        // Border
            Spec::new(readable_on(face_bg, text, page), face_bg),           // Face
            Spec::new(readable_on(hover_bg, text, page), hover_bg),         // FaceHover
            Spec::new(readable_on(active_bg, text, page), active_bg),       // FaceActive
            Spec::new(readable_on(selection_bg, text, page), selection_bg), // Selection, on base0D
            Spec::new(selection_bg, page),                                  // Focus
            Spec::new(channel(palette[8]), page),                           // Danger
            Spec::new(channel(palette[10]), page),                          // Warn
            Spec::new(channel(palette[11]), page),                          // Ok
        ])
    }
}

/// One `0xRRGGBB` of a base16 palette as a colour.
pub(super) const fn channel(v: u32) -> Rgb {
    // Three masked bytes, so every cast is exact.
    Rgb::new(
        ((v >> 16) & 0xff) as u8,
        ((v >> 8) & 0xff) as u8,
        (v & 0xff) as u8,
    )
}

/// Three slots of a palette as a candidate list, in preference order.
const fn ramp(palette: &[u32; 16], i: usize, j: usize, k: usize) -> [Rgb; 3] {
    [
        channel(palette[i]),
        channel(palette[j]),
        channel(palette[k]),
    ]
}

/// Record what a colour becomes at 256 colours, so a later `pick` can avoid it.
const fn spend(rgb: Rgb, spent: &mut [u8; 16], n: &mut usize) {
    if *n < 16 {
        spent[*n] = wire::at_256(rgb);
        *n += 1;
    }
}

/// Whether an index is already spent.
const fn is_spent(spent: &[u8; 16], n: usize, idx: u8) -> bool {
    let mut i = 0;
    while i < n {
        if spent[i] == idx {
            return true;
        }
        i += 1;
    }
    false
}

/// The first candidate the quantiser can still separate from everything already spent, or a colour
/// **derived** from the scheme's own two ends when it can separate none of them.
///
/// The derivation is forbidden to the engine and permitted here: the engine may never
/// synthesise a colour, because it would be lying about what the terminal holds, and an offline
/// import mixing two of the scheme's own colours is doing once, at compile time, what the author
/// would have done with a fourth slot. Six per cent at a time is the smallest step that moves a
/// 256-colour index across the grey ramp.
///
/// The last resort is to return the first candidate and collide. A loud failure would be worse: an
/// import that refuses a scheme ships a set the user's favourite theme is missing from, and the pair
/// count already says which themes are affected.
const fn pick(
    candidates: [Rgb; 3],
    page: Rgb,
    text: Rgb,
    spent: &mut [u8; 16],
    n: &mut usize,
) -> Rgb {
    let mut i = 0;
    while i < candidates.len() {
        let cand = candidates[i];
        if !is_spent(spent, *n, wire::at_256(cand)) {
            spend(cand, spent, n);
            return cand;
        }
        i += 1;
    }
    let mut t = 6u32;
    while t < 100 {
        let mixed = lerp(page, text, t);
        if !is_spent(spent, *n, wire::at_256(mixed)) {
            spend(mixed, spent, n);
            return mixed;
        }
        t += 6;
    }
    candidates[0]
}

/// Whichever of the scheme's own text and page colours is further from `bg` in luminance.
///
/// Nailing `base05` to every face background — the obvious reading of *the scheme's foreground* —
/// puts `FaceActive` below a 3:1 contrast ratio in 218 of the map's 338 corpus schemes; choosing the
/// readable end per rung takes it to 55.
const fn readable_on(bg: Rgb, text: Rgb, page: Rgb) -> Rgb {
    let l = luminance(bg);
    if luminance(text).abs_diff(l) >= luminance(page).abs_diff(l) {
        text
    } else {
        page
    }
}

/// Integer luminance, on the same `2:4:3` weights the quantiser uses.
pub(super) const fn luminance(c: Rgb) -> u32 {
    (2 * c.r as u32 + 4 * c.g as u32 + 3 * c.b as u32) / 9
}

/// `a` toward `b` by `t` per cent, where `t` is at most a hundred.
const fn lerp(a: Rgb, b: Rgb, t: u32) -> Rgb {
    Rgb::new(
        mix_channel(a.r, b.r, t),
        mix_channel(a.g, b.g, t),
        mix_channel(a.b, b.b, t),
    )
}

/// One channel of [`lerp`]. A weighted mean of two bytes is a byte, so the cast is exact.
const fn mix_channel(x: u8, y: u8, t: u32) -> u8 {
    ((x as u32 * (100 - t) + y as u32 * t) / 100) as u8
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
///
/// # Ten, and the split is a rule rather than a taxonomy
///
/// > **A distinction survives the whole matrix iff it is carried on both axes.**
///
/// [`Distinction::carried_by`] is that sentence as a value. **Six of the nine name a glyph pair and
/// three do not**, and the three that do not are exactly the three that die: [`Distinction::Fade`]
/// below truecolor, [`Distinction::Status`] where the palette's three signals quantise together, and
/// [`Distinction::Hover`] wherever the two faces land on one index. Everything glyph-bearing survives
/// all nine cells of the repertoire × tier matrix, **because the glyph carries it** — a spelling is
/// never blank and never other than one cell, so the carrier cannot go quiet the way a colour can.
///
/// The carrier pairs are **cross-family wherever a family can collapse**. All nine box-drawing
/// entries spell `+` at ASCII on purpose, so no distinction is carried by two of them: a tree's
/// *last child* — `TeeLeft` against `BottomLeft` — is a real difference that ASCII loses, and it is
/// not on this list for that reason.
///
/// **A tree used to ask one more and no longer asks anything here.** `Distinction::Guide` was
/// `VLine` against `TeeLeft`, which crosses a family and survives — and the components decision
/// then established that no component draws an indent guide at all, because a guide column at depth
/// *d* is a fact about *d* ancestors and every route to it is refused. A
/// distinction is a claim about a screen somebody draws, and this one was struck.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Distinction {
    /// A face under the pointer looks different from one at rest.
    Hover,
    /// The *animation into* a hover is visible — which is a different question, and the pair is why.
    Fade,
    /// [`Role::Danger`], [`Role::Warn`] and [`Role::Ok`] are three different things.
    Status,
    /// A scrollbar's two steppers point opposite ways.
    Stepper,
    /// A disclosure marker says open or closed — the same arrow family as the
    /// steppers, and **entering it twice under two names would be the collapse the pair gate
    /// catches**.
    Disclosure,
    /// A rule across is not a rule down.
    Separator,
    /// A scrollbar's thumb is distinguishable from its track.
    Thumb,
    /// A label that was **cut** is distinguishable from one that ended — and, specifically, from a
    /// collapsed-node marker.
    ///
    /// **This is C09's defect as a bit.** The shadow table spelled the ASCII ellipsis `>`, which is
    /// exactly an ASCII `ArrowRight`, so 468 truncated labels ended in the collapsed-node marker and
    /// `tree` draws both. Spelled `~`, the carrier crosses a family and the bit stands.
    Truncation,
    /// A list marker is not a checkmark.
    Marker,
}

impl Distinction {
    /// Every distinction. **Nine**, which is the denominator the distinction matrix reports against.
    ///
    /// **It was ten until 2026-09-05.** `Distinction::Guide` was *a
    /// tree's indent guide says this row has a sibling below rather than merely there is depth
    /// here*, carried by `(VLine, TeeLeft)` — and it was established that this
    /// library **does not draw an indent guide and cannot**: a guide column at depth *d* is a fact
    /// about *d* ancestors, and all four routes to it are refused. A distinction is a claim
    /// about what a narrowed repertoire can still tell apart on a screen somebody draws; a bit
    /// whose drawing does not exist is not one. `TeeLeft` reaching no screen was the symptom, and
    /// the components-side gate that would have caught it was passing vacuously because it read
    /// *declares* where it meant *draws*.
    pub const ALL: [Distinction; 9] = [
        Distinction::Hover,
        Distinction::Fade,
        Distinction::Status,
        Distinction::Stepper,
        Distinction::Disclosure,
        Distinction::Separator,
        Distinction::Thumb,
        Distinction::Truncation,
        Distinction::Marker,
    ];

    /// The **glyph pair** that carries this distinction, where one does.
    ///
    /// `None` is not *uncarried* — it is *carried on the colour axis alone*, which is the half that
    /// narrowing can take away. A bit whose carrier is a glyph pair is set when the theme's declared
    /// repertoire spells the two differently; a bit with no carrier is set by [`Theme::resolve`]'s
    /// rule over the quantised roles.
    pub const fn carried_by(self) -> Option<(Glyph, Glyph)> {
        match self {
            Distinction::Hover | Distinction::Fade | Distinction::Status => None,
            Distinction::Stepper => Some((Glyph::ArrowUp, Glyph::ArrowDown)),
            Distinction::Disclosure => Some((Glyph::ArrowRight, Glyph::ArrowDown)),
            Distinction::Separator => Some((Glyph::HLine, Glyph::VLine)),
            Distinction::Thumb => Some((Glyph::Thumb, Glyph::Track)),
            Distinction::Truncation => Some((Glyph::Ellipsis, Glyph::ArrowRight)),
            Distinction::Marker => Some((Glyph::Bullet, Glyph::Tick)),
        }
    }

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
/// **Twenty entries**, grown from seven when the demand set became a value.
/// **The theme owns the table because there was nowhere else to put one** — nine
/// crates of twelve grew a private `mod missing` with six glyph literals and a `match` on
/// `GlyphSet`, byte-identical in all nine.
///
/// # The twenty are six families, and the family is what the collapse gate is written over
///
/// | family | entries | ASCII |
/// |---|---|---|
/// | arrow | [`Glyph::ArrowUp`] [`Glyph::ArrowDown`] [`Glyph::ArrowLeft`] [`Glyph::ArrowRight`] | `^` `v` `<` `>` |
/// | rule | [`Glyph::VLine`] [`Glyph::HLine`] | `\|` `-` |
/// | box | the four corners, the four tees and [`Glyph::Cross`] | `+` ×9 |
/// | mark | [`Glyph::Bullet`] [`Glyph::Tick`] | `*` `x` |
/// | fill | [`Glyph::Thumb`] [`Glyph::Track`] | `#` `:` |
/// | cut | [`Glyph::Ellipsis`] | `~` |
///
/// **The nine box-drawing entries all spell `+` at ASCII on purpose**, which is exactly the 36 of
/// 190 glyph pairs measured collapsing there — `9 × 8 / 2`, and every one of them inside one
/// family. A corner collapsing onto a corner loses nothing; a corner collapsing onto an arrow would
/// be C09's defect, and that is why the gate the component crate keeps is **cross-family** collapse
/// and not the pairwise version.
///
/// **The arrows are one family serving two mechanisms.** Scrollbar steppers and disclosure
/// markers are the same four ends, and entering them twice under two names would be a collapse the
/// pair gate catches rather than two useful entries.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Glyph {
    /// An upward arrow — a stepper, or a sort ascending.
    ArrowUp,
    /// A downward arrow — a stepper, or an expanded disclosure.
    ArrowDown,
    /// A leftward arrow — a stepper, or a page back.
    ArrowLeft,
    /// A rightward arrow — a stepper, or a collapsed disclosure.
    ArrowRight,
    /// A vertical rule.
    VLine,
    /// A horizontal rule.
    HLine,
    /// The top-left corner of a frame.
    TopLeft,
    /// The top-right corner of a frame.
    TopRight,
    /// The bottom-left corner of a frame, and a tree's last child.
    BottomLeft,
    /// The bottom-right corner of a frame.
    BottomRight,
    /// The tee on a frame's **top** edge, where a column rule meets it.
    TeeTop,
    /// The tee on a frame's **bottom** edge.
    TeeBottom,
    /// The tee on a frame's **left** edge, and a tree's non-last child.
    TeeLeft,
    /// The tee on a frame's **right** edge.
    TeeRight,
    /// Where a column rule and a row rule cross.
    Cross,
    /// A list marker.
    Bullet,
    /// A checkmark.
    Tick,
    /// A scrollbar thumb.
    Thumb,
    /// A scrollbar track.
    Track,
    /// The one cell a truncated label ends in.
    ///
    /// **One cell, and that is the entry's whole point.** A three-cell `...` where one was reserved
    /// moves 468 cells over 78 rows with `writes / verbs / marked` identical either way: inside a
    /// narrowed context the clip discards the two extras silently and the reader gets a hard cut
    /// that looks deliberate, and outside one they overrun the neighbour.
    Ellipsis,
}

impl Glyph {
    /// Every glyph, which is what the two spelling gates iterate.
    pub const ALL: [Glyph; 20] = [
        Glyph::ArrowUp,
        Glyph::ArrowDown,
        Glyph::ArrowLeft,
        Glyph::ArrowRight,
        Glyph::VLine,
        Glyph::HLine,
        Glyph::TopLeft,
        Glyph::TopRight,
        Glyph::BottomLeft,
        Glyph::BottomRight,
        Glyph::TeeTop,
        Glyph::TeeBottom,
        Glyph::TeeLeft,
        Glyph::TeeRight,
        Glyph::Cross,
        Glyph::Bullet,
        Glyph::Tick,
        Glyph::Thumb,
        Glyph::Track,
        Glyph::Ellipsis,
    ];

    /// The spelling at each rung.
    ///
    /// **Two rows, not three.** The `Unicode` and `Extended` spellings are identical for every entry,
    /// and that is a decision rather than an omission: anything that would distinguish the top two
    /// rungs is a *branch* — the sub-rows per cell change, 8 for braille against 2 for blocks against
    /// 1 for ASCII, and so does the number of samples asked of the data. No table can carry that.
    ///
    /// **The ASCII ellipsis is `~` and not `>`**, which is not a spelling preference: `>` is exactly
    /// the ASCII [`Glyph::ArrowRight`], and the shadow table that spelled it that way put the
    /// collapsed-node marker on the end of 468 truncated labels in a `tree` that draws both.
    const fn spell(self, set: GlyphSet) -> &'static str {
        match (self, set) {
            (Glyph::ArrowUp, GlyphSet::Ascii) => "^",
            (Glyph::ArrowUp, _) => "▲",
            (Glyph::ArrowDown, GlyphSet::Ascii) => "v",
            (Glyph::ArrowDown, _) => "▼",
            (Glyph::ArrowLeft, GlyphSet::Ascii) => "<",
            (Glyph::ArrowLeft, _) => "◀",
            (Glyph::ArrowRight, GlyphSet::Ascii) => ">",
            (Glyph::ArrowRight, _) => "▶",
            (Glyph::VLine, GlyphSet::Ascii) => "|",
            (Glyph::VLine, _) => "│",
            (Glyph::HLine, GlyphSet::Ascii) => "-",
            (Glyph::HLine, _) => "─",
            // The nine that all spell `+`, deliberately and in one block, so that a reader who
            // wonders whether the collapse is an accident can see that it is not.
            (
                Glyph::TopLeft
                | Glyph::TopRight
                | Glyph::BottomLeft
                | Glyph::BottomRight
                | Glyph::TeeTop
                | Glyph::TeeBottom
                | Glyph::TeeLeft
                | Glyph::TeeRight
                | Glyph::Cross,
                GlyphSet::Ascii,
            ) => "+",
            (Glyph::TopLeft, _) => "┌",
            (Glyph::TopRight, _) => "┐",
            (Glyph::BottomLeft, _) => "└",
            (Glyph::BottomRight, _) => "┘",
            (Glyph::TeeTop, _) => "┬",
            (Glyph::TeeBottom, _) => "┴",
            (Glyph::TeeLeft, _) => "├",
            (Glyph::TeeRight, _) => "┤",
            (Glyph::Cross, _) => "┼",
            (Glyph::Bullet, GlyphSet::Ascii) => "*",
            (Glyph::Bullet, _) => "•",
            (Glyph::Tick, GlyphSet::Ascii) => "x",
            (Glyph::Tick, _) => "✓",
            (Glyph::Thumb, GlyphSet::Ascii) => "#",
            (Glyph::Thumb, _) => "█",
            (Glyph::Track, GlyphSet::Ascii) => ":",
            (Glyph::Track, _) => "░",
            (Glyph::Ellipsis, GlyphSet::Ascii) => "~",
            (Glyph::Ellipsis, _) => "…",
        }
    }
}

/// A restyle written over **roles**, which is the only kind a component can write.
///
/// The engine's own `Restyle` cannot be passed through: its fields are `Option<Color>`, and a `Color`
/// is exactly what a component may not name. The attribute masks are the engine's `u16`
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
pub struct Repaint<'a> {
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
    /// The new hyperlink, as a URI, or leave it.
    ///
    /// [`Link::None`] clears it and `None` leaves it alone. **The URI is the datum**, which is what
    /// gives this descriptor its lifetime: the engine interns it at the verb, so a component names
    /// the address it already has instead of a handle it would have had to get from somewhere it
    /// cannot reach. See the note where `Ctx::link` used to be
    /// explained why it could not exist.
    pub link: Option<Link<'a>>,
}

impl<'a> Repaint<'a> {
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
    pub(crate) fn lower(self, theme: &Theme) -> Restyle<'a> {
        let colour = |r: Option<Role>| r.map(|r| theme.spec(r).style_fg());
        Restyle {
            fg: colour(self.fg),
            bg: self.bg.map(|r| theme.spec(r).style_bg()),
            set: self.set,
            clear: self.clear,
            ul: colour(self.ul),
            // Straight across: the URI is the engine's own `Link` and there is nothing to resolve.
            // A role is the runtime's vocabulary and an address is not.
            link: self.link,
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
    /// Whether [`Theme::resolve`] has been called.
    ///
    /// **Not derivable from `tier`**, and that stopped being an
    /// implementation detail. `ColorDepth::None` is both *nobody has said* and *a terminal with no
    /// colour*, and while every distinction was carried on the colour axis the two answered the
    /// same — an unresolved theme claimed nothing, and so did a monochrome one. Seven of the ten
    /// distinctions are now carried by a **glyph** pair, which a monochrome terminal keeps and an
    /// unresolved theme must still not claim. So the flag exists, and it is what
    /// `tests::an_unresolved_theme_claims_nothing` still holds over ten entries.
    resolved: bool,
    distinctions: u16,
    rev: Revision,
}

/// Catppuccin Mocha, as base16. The default palette, and **data rather than a decision** — no gate
/// here reads a literal from it.
///
/// **The work that grew the glyph table deliberately left it alone.** It grew the glyph
/// table and the distinction set, and the only reason to have swapped the palette as well would have
/// been to make an old `1 / 2 / 13 of 78` reproduce — which is tuning the data until the report
/// prints the remembered number. The pair count is gated as a **relation** for exactly that reason,
/// and what the shipped palette actually measures (`0 / 0 / 18`) is recorded beside the expectation
/// in `crates/vitui-components/examples/glyph_numbers.rs` rather than engineered away.
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
            resolved: false,
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
        self.resolved = true;
        // Thirteen quantisations, once. Everything below reads them.
        self.keys = core::array::from_fn(|i| self.specs[i].key(tier));
        self.distinctions = self.narrow();
        self.rev = Revision::fresh();
        self
    }

    /// Declare a glyph repertoire. **Declared, never probed**.
    ///
    /// It re-narrows the [`Distinction`] bits, because six of the nine are carried by a glyph pair
    /// and the repertoire is half of what *both axes* names. **Order is therefore not
    /// load-bearing**: `with_glyphs(..).resolve(..)` and `resolve(..).with_glyphs(..)` reach the same
    /// theme, which the version that narrowed only inside `resolve` did not.
    pub fn with_glyphs(mut self, set: GlyphSet) -> Theme {
        self.glyphs = set;
        self.distinctions = self.narrow();
        self.rev = Revision::fresh();
        self
    }

    /// Every [`Distinction`] bit, from the palette as it arrives at the terminal and the repertoire
    /// as it was declared.
    ///
    /// The rule as arithmetic: a distinction with a glyph carrier is set when the declared
    /// repertoire spells its two glyphs differently, and one without a carrier is set by the rule
    /// over the quantised roles. **An unresolved theme is zero**, which is the conservative answer:
    /// a theme nobody has told about the terminal must not promise a component anything.
    fn narrow(&self) -> u16 {
        if !self.resolved {
            return 0;
        }
        let differs = |a: Role, b: Role| self.keys[a.index()] != self.keys[b.index()];
        let mut bits = 0u16;
        for d in Distinction::ALL {
            let carried = match d.carried_by() {
                // The glyph axis. It cannot go quiet the way a colour can — no spelling is blank
                // and none is other than one cell — so what this asks is only whether the declared
                // rung tells these two apart, which is where a within-family pair would fail.
                Some((a, b)) => a.spell(self.glyphs) != b.spell(self.glyphs),
                // The colour axis alone, which is the half narrowing takes away.
                None => match d {
                    Distinction::Hover => differs(Role::Face, Role::FaceHover),
                    // **Two answers, not one.** A hover being visible does not mean the *animation
                    // into* it is: a fade is a sequence of intermediate colours, and at 256 colours
                    // or fewer the intermediates collapse onto the endpoints. Of 338 corpus themes
                    // 338 show a hover at C256 and **165** show the animation, so a component
                    // reading only the first pays nineteen wakeups for a switch in 173 of them.
                    Distinction::Fade => {
                        differs(Role::Face, Role::FaceHover) && self.tier == ColorDepth::TrueColor
                    }
                    Distinction::Status => {
                        differs(Role::Danger, Role::Warn) && differs(Role::Warn, Role::Ok)
                    }
                    // Unreachable by construction: `carried_by` returns `None` for exactly these
                    // three. Spelled as a `false` rather than as a panic because warnings are denied
                    // workspace-wide and an arm nothing reaches still has to compile.
                    _ => false,
                },
            };
            if carried {
                bits |= d.bit();
            }
        }
        bits
    }

    /// The paint for a role.
    pub fn paint(&self, r: Role) -> Paint {
        Paint(self.styles[r.index()])
    }

    /// **The colour the page is painted in.**
    ///
    /// [`Theme::custom`] takes two [`Rgb`] and the thirteen roles are [`Paint`]s rather than
    /// colours, so a component that needs an arbitrary foreground *over the page* had no way to ask
    /// what the page is. `crate::chart` substituted `Rgb::new(0, 0, 0)` when [`Theme::is_dark`] said
    /// dark — the one bit about the page a component could read — and the substitution is visible on
    /// a real terminal: every plotted cell went out as `48:5:16`, pure black, while the text around
    /// it went out as `48:5:235`. A black halo around every curve and every bar, on a page that is
    /// `#1e1e2e`.
    ///
    /// **`Role::Body`'s background is the page**, which is what makes this a read rather than a
    /// thirteenth role: a palette says what the body is painted on, and every other role is painted
    /// on the same ground.
    #[must_use]
    pub fn page(&self) -> Rgb {
        self.specs[Role::Body.index()].bg
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
    /// **The verb that discharges it is [`Theme::colours_differ_on_wire`]**, and it is here because
    /// this sentence stood for four tickets with nothing to answer it: the one question the surface
    /// had about the wire was over the thirteen **roles**, and a `custom` cell is outside them by
    /// construction.
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
    ///
    /// # `Hover` and `Fade` are two questions, and reading only the first is the defect
    ///
    /// [`Distinction::Hover`] says *a face under the pointer looks different*. [`Distinction::Fade`]
    /// says *the animation into it is visible*, which is a strictly harder thing to ask of a palette:
    /// a fade is a run of intermediate colours, and below truecolor the intermediates collapse onto
    /// the endpoints. Over the map's corpus every one of 338 themes shows a hover at 256 colours and
    /// **165** show the animation.
    ///
    /// > **A component that reads `shows(Hover)` and then cross-fades pays nineteen wakeups for a
    /// > switch it cannot show, in 173 themes of 338.** The two are separate readers because they are
    /// > separate capabilities; asking the first and assuming the second is the wakeup budget spent
    /// > on a picture nobody receives.
    ///
    /// The correct shape is both, in that order: `shows(Hover)` decides whether there is a hover
    /// state at all, and `shows(Fade)` decides whether to animate into it or to snap.
    ///
    /// ```
    /// use vitui_engine::ColorDepth;
    /// use vitui_runtime::theme::{Distinction, Theme};
    ///
    /// let theme = Theme::default().resolve(ColorDepth::Indexed256);
    /// if theme.shows(Distinction::Hover) {
    ///     if theme.shows(Distinction::Fade) {
    ///         // animate into the hover
    ///     } else {
    ///         // snap to it, and ask for no wakeups
    ///     }
    /// }
    /// ```
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

    /// Whether two **colours** are distinguishable on the wire, at the tier this theme was resolved
    /// for.
    ///
    /// [`Theme::roles_differ_on_wire`] asked of two colours instead of two roles, and it exists
    /// because the one caller [`Theme::custom`]'s obligation is addressed to is the one caller the
    /// role-pair question cannot serve: **a paint made from two `Rgb` is outside the thirteen roles
    /// by construction**, so the branch `custom` says the component owes had nothing to branch on.
    /// Found by the first screen in this workspace whose every cell is
    /// `custom`.
    ///
    /// # It publishes no index, and that is what keeps the honesty rule intact
    ///
    /// *Do these two look the same* is a fact a component may act on. *Which index this landed on*
    /// is not — nothing in this process may read the sixteen colours the terminal is configured
    /// with, so an index handed out would be an answer that is wrong on half of ten real terminal
    /// profiles. The return is a `bool` for that reason and not for convenience.
    ///
    /// The obligation is [`Theme::roles_differ_on_wire`]'s, verbatim: a component reading `false`
    /// owes a **second axis** — a glyph, a rule, a position — and never a darker colour, because a
    /// darker colour is the same axis again and collapses in the same place.
    ///
    /// ```
    /// use vitui_runtime::{ColorDepth, Rgb, Theme};
    ///
    /// let (a, b) = (Rgb::new(0x10, 0x20, 0x30), Rgb::new(0x10, 0x20, 0x31));
    /// assert!(Theme::default().resolve(ColorDepth::TrueColor).colours_differ_on_wire(a, b));
    /// assert!(!Theme::default().resolve(ColorDepth::Ansi16).colours_differ_on_wire(a, b));
    /// ```
    #[must_use]
    pub const fn colours_differ_on_wire(&self, a: Rgb, b: Rgb) -> bool {
        wire::key(a, self.tier) != wire::key(b, self.tier)
    }

    /// What a component should declare when it registers a hoverable region.
    ///
    /// **The runtime does not edit a declared interest**. This is an answer, not a
    /// correction: a component that writes `Interest::HOVER` against a flat theme still gets motion
    /// tracking, and is therefore visibly wrong rather than invisibly fixed.
    pub const fn hover_interest(&self) -> Interest {
        if self.shows(Distinction::Hover) {
            Interest::HOVER
        } else {
            Interest::NONE
        }
    }

    /// Whether the page is dark. An overlay scrim reads this and nothing else mints it.
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

    /// A memo key that carries this theme as well as the data. **+0.312 ns, 1.247×.**
    ///
    /// # The theme is a second memo input the data contract does not cover
    ///
    /// [`crate::data::Memo`] takes one [`Revision`] and nothing else, and on a swap frame a
    /// data-keyed memo answers with the **previous theme's output** — because the data did not
    /// change. If the value is made of paints, that is a stale picture that persists until the data
    /// happens to move, which on a settings pane is never.
    ///
    /// ```
    /// use vitui_runtime::data::{Memo, Revision, Versioned};
    /// use vitui_runtime::theme::{Role, Theme};
    /// use vitui_engine::ColorDepth;
    ///
    /// let rows = Versioned::new(vec![1u32, 2, 3]);
    /// let dark = Theme::default().resolve(ColorDepth::TrueColor);
    /// let mut paints: Memo<Vec<_>> = Memo::new();
    ///
    /// let build = |t: &Theme| t.paint(Role::Body);
    /// let first = *paints
    ///     .get(dark.memo_key(rows.revision()), || vec![build(&dark)])
    ///     .first()
    ///     .expect("one row");
    ///
    /// // Same data, different theme. The key moves, so the memo misses and rebuilds.
    /// let light = Theme::authored(&[0xffffff; 16], Default::default(), Default::default());
    /// let second = *paints
    ///     .get(light.memo_key(rows.revision()), || vec![build(&light)])
    ///     .first()
    ///     .expect("one row");
    /// assert_eq!(paints.recomputes, 2);
    /// assert_ne!(first, second);
    /// ```
    ///
    /// # The rule is narrow, and reading it as *every memo* is itself the defect
    ///
    /// > **A memo carries the theme in its key exactly when its value is made of paints.**
    ///
    /// Both directions. A memo whose value moves with the theme and does not carry it is stale after
    /// a swap; a memo whose value does not move with the theme and carries it anyway pays a full
    /// recomputation on every swap **for a value that is bit-identical afterwards** — the map priced
    /// that at 221 µs and 399 µs with two and four such memos on the dense screen, which is two and
    /// four times the whole frame budget.
    ///
    /// It works only because the theme's revision comes from [`Revision`]'s own **process-global**
    /// counter. A per-theme counter would put two themes both at revision one, which is exactly the
    /// two-values-both-at-1 defect the data contract refused.
    ///
    /// [`Revision::UNKNOWN`] is preserved rather than folded: *memoise nothing* means nothing,
    /// including nothing about the theme.
    pub const fn memo_key(&self, data: Revision) -> Revision {
        if !data.is_known() {
            return Revision::UNKNOWN;
        }
        // **A rotate, a multiply and an xor — not FNV-1a, and the reason is a measurement.** The
        // crate's own hash is FNV-1a and `id.rs` uses it, but FNV eats one byte at a time, so folding
        // two words is sixteen rounds: **5.76 ns against this form's fraction of one**, on a value
        // that is taken once per memo per frame. The map priced the pair key at +0.312 ns and that is
        // the budget this fits in.
        //
        // What it buys instead of a hash's diffusion is a property that can be *stated*: both halves
        // are bijections of `u64`, so the fold is **injective in each argument with the other held
        // fixed**. A swap therefore always moves the key and an edit always moves the key, which is
        // the whole of what a memo needs. It is not a hash and makes no claim to be one — two
        // *different* pairs may still collide, exactly as they may under FNV.
        const GOLDEN: u64 = 0x9e37_79b9_7f4a_7c15;
        let mixed = data.raw().rotate_left(32) ^ self.rev.raw().wrapping_mul(GOLDEN);
        // Zero is `UNKNOWN`, which would silently mean *memoise nothing*. One value of 2^64 is worth
        // a branch rather than a footnote.
        Revision::from_raw(if mixed == 0 { 1 } else { mixed })
    }

    #[allow(
        dead_code,
        reason = "read by `Repaint::lower`, whose caller is ticket 08's `Ctx`"
    )]
    fn spec(&self, r: Role) -> Spec {
        self.specs[r.index()]
    }
}

/// **Instruments, and they stay `cfg(test)`.**
///
/// Both of these read the wire arithmetic a role landed on rather than whether two roles agree, and
/// that is a different kind of answer: `roles_differ_on_wire` is a fact a component may act on, an
/// index is a fact nothing in this process may act on. They exist so that the
/// operator-palette gate in [`registry`] can merge two buckets and re-take the count, which is the
/// only way *a sixteen-colour count is a lower bound* becomes mechanical instead of a caveat.
#[cfg(test)]
impl Theme {
    /// The sixteen-colour indices this role's foreground and background land on.
    pub(crate) fn wire_indices_at_16(&self, r: Role) -> (u8, u8) {
        let spec = self.specs[r.index()];
        (wire::at_16(spec.fg), wire::at_16(spec.bg))
    }

    /// A role's attribute bits, which are the third component of every pair key.
    pub(crate) fn attrs_of(&self, r: Role) -> u16 {
        self.specs[r.index()].attrs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// WCAG relative luminance, which is a different curve from the quantiser's `2:4:3` weights and
    /// has to be: the quantiser asks *which index is nearest* and a contrast ratio asks *can this be
    /// read*. Floating point, because it is a report's arithmetic and never a gate's key.
    fn relative_luminance(c: Rgb) -> f64 {
        let channel = |v: u8| {
            let v = f64::from(v) / 255.0;
            if v <= 0.03928 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(c.r) + 0.7152 * channel(c.g) + 0.0722 * channel(c.b)
    }

    /// The contrast ratio between two colours, 1.0 to 21.0.
    fn contrast(fg: Rgb, bg: Rgb) -> f64 {
        let (a, b) = (relative_luminance(fg), relative_luminance(bg));
        let (hi, lo) = if a > b { (a, b) } else { (b, a) };
        (hi + 0.05) / (lo + 0.05)
    }

    /// **The page is the ground every role is painted on, and it is not black.**
    ///
    /// `crate::chart::series_paint` had no way to ask and substituted `Rgb::new(0, 0, 0)` for a dark
    /// theme. Catppuccin Mocha's page is `#1e1e2e`: on a real terminal every plotted cell went out
    /// as `48:5:16` and every glyph around it as `48:5:235`, which is a black halo around every
    /// curve on every dark theme this crate ships.
    ///
    /// Asserted over **all fourteen shipped schemes** rather than the default, and in both
    /// directions: the page is what every role without a ground of its own is painted on, and no
    /// scheme's page is either pure black or pure white — a palette whose ground really were
    /// `#000000` would make the old substitution correct by accident and this gate vacuous for it.
    ///
    /// **`Role::Body` is not on the list the equality reads.** [`Theme::page`] *is*
    /// `specs[Role::Body].bg`, so asserting one against the other is one declaration compared with
    /// itself — a gate that cannot fail, which is the trap this map has met more than once. The
    /// eight roles below are [`Roles::from_palette`]'s **other** users of the page, so the equality
    /// is a claim about the import rather than a restatement of the accessor: a pairing that gave
    /// `Focus` a ground of its own, or a `page()` that read some other role, goes red here.
    #[test]
    fn the_page_is_the_body_ground_and_no_shipped_scheme_paints_it_pure_black() {
        // The nine roles `from_palette` paints on the page, minus `Body` — which is the one
        // `page()` reads, and so the one that would make this compare a value with itself.
        const ON_THE_PAGE: [Role; 8] = [
            Role::Title,
            Role::Dim,
            Role::Disabled,
            Role::Border,
            Role::Focus,
            Role::Danger,
            Role::Warn,
            Role::Ok,
        ];
        for scheme in crate::theme::schemes::STANDARD {
            let theme = scheme.theme(GlyphSet::default(), Density::default());
            let name = scheme.name();
            let page = theme.page();
            for role in ON_THE_PAGE {
                assert_eq!(
                    theme.specs[role.index()].bg,
                    page,
                    "{name}: {role:?} carries no ground of its own and is painted on the page",
                );
            }
            assert_ne!(
                (page.r, page.g, page.b),
                (0, 0, 0),
                "{name}: a page of pure black would make the substitution this replaces correct \
                 by accident",
            );
            assert_ne!(
                (page.r, page.g, page.b),
                (0xff, 0xff, 0xff),
                "{name}: and pure white is the same accident on the other side",
            );
        }
    }

    /// **Contrast is a check the pair count structurally cannot make**, and this is the relation that
    /// keeps `readable_on` from being deleted as a complication.
    ///
    /// The pair count compares roles to *each other*; a contrast ratio compares a role to *itself*. A
    /// mapping judged only by the pair count will happily produce a theme in which nothing is
    /// readable and everything is distinct — which is what nailing `base05` to every face background
    /// does: over the map's 338-scheme corpus it puts `FaceActive` below 3:1 in **218** schemes, and
    /// choosing the readable end per rung takes it to **55**.
    ///
    /// The corpus is not vendored, so what is gated is the relation over the shipped set: **the
    /// shipped pairing is never worse than nailing the foreground, and for at least one theme it is
    /// better.** A relation rather than either number, because the number belongs to the palettes.
    #[test]
    fn pairing_by_luminance_never_costs_contrast_and_usually_buys_it() {
        // The four roles that carry a background of their own, which are the ones `readable_on`
        // decides. Every other role sits on the page and has nothing to choose between.
        const ON_THEIR_OWN_GROUND: [Role; 4] = [
            Role::Face,
            Role::FaceHover,
            Role::FaceActive,
            Role::Selection,
        ];
        let mut improved = 0;
        for scheme in Themes::standard().schemes() {
            let theme = scheme.theme(GlyphSet::default(), Density::default());
            let nailed_fg = theme.spec(Role::Body).fg;
            let (mut shipped_low, mut nailed_low) = (0, 0);
            for role in ON_THEIR_OWN_GROUND {
                let spec = theme.spec(role);
                shipped_low += usize::from(contrast(spec.fg, spec.bg) < 3.0);
                nailed_low += usize::from(contrast(nailed_fg, spec.bg) < 3.0);
            }
            assert!(
                shipped_low <= nailed_low,
                "{}: the readable end is worse than nailing the foreground, {shipped_low} against \
                 {nailed_low}",
                scheme.slug()
            );
            if shipped_low < nailed_low {
                improved += 1;
            }
        }
        assert!(
            improved > 0,
            "no shipped theme is helped by pairing on luminance, which would make `readable_on` a \
             complication with nothing behind it"
        );
    }

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
    /// and a later palette replaces this one.
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

    /// **A finding, recorded as a measurement rather than argued: the `Fade` bit is *tier-gated* and
    /// the palette does not require it to be.**
    ///
    /// [`Theme::resolve`] sets `Fade` when the hover pair differs **and** the tier is truecolor, so
    /// the bit is false below truecolor by construction. An earlier sketch was narrower —
    /// *338 themes can show a hover state at C256 and **165** can show the animation into it* — which
    /// is a statement about palettes, and no palette can reach it through a rule that names the tier.
    /// Over the shipped fourteen the split is therefore 14 / 0 at 256 colours rather than a
    /// proportion.
    ///
    /// It is measured here and **not changed here**, because the rule is shipped,
    /// documented and gated, and this backlog does not reopen a decision inside another ticket. What
    /// this test pins is the size of the gap, so that whoever does reopen it starts from a number:
    /// the default palette's two face backgrounds are two steps of the grey ramp apart, and the ramp
    /// between them contains a third index the terminal can show.
    ///
    /// The consequence is conservative in the safe direction — a component is told *do not animate*
    /// when it could have — so nothing renders wrong; what it costs is an animation, not a wakeup.
    #[test]
    fn the_fade_bit_is_tier_gated_and_the_palette_does_not_require_it() {
        let theme = resolved(ColorDepth::Indexed256);
        let (face, hover) = (theme.spec(Role::Face).bg, theme.spec(Role::FaceHover).bg);
        let mut seen: Vec<u32> = Vec::new();
        for t in 0..=100 {
            let key = wire::key(lerp(face, hover, t), ColorDepth::Indexed256);
            if !seen.contains(&key) {
                seen.push(key);
            }
        }
        assert!(
            seen.len() > 2,
            "the ramp between the two face backgrounds holds only its endpoints, which would make \
             the tier gate exact rather than conservative"
        );
        assert!(
            !theme.shows(Distinction::Fade),
            "and the shipped bit is false anyway, because the rule names the tier"
        );
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

        // **The URI goes straight across, and that is the whole of what replaced `Ctx::link`.** A
        // role is the runtime's vocabulary and is resolved here; an address is the component's own
        // datum and there is nothing to resolve it against. The engine interns it at the verb.
        let linked = Repaint {
            link: Some(Link::Uri("https://example.com/")),
            ..Default::default()
        }
        .lower(&theme);
        assert_eq!(linked.link, Some(Link::Uri("https://example.com/")));
        let cleared = Repaint {
            link: Some(Link::None),
            ..Default::default()
        }
        .lower(&theme);
        assert_eq!(cleared.link, Some(Link::None), "and a clear stays a clear");
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

    /// **The table is twenty entries and the distinction set is nine**, which are the two
    /// denominators the distinction matrix reports against.
    ///
    /// A count rather than a sentence, because the demand set is a value: the day an entry is added
    /// without the matrix moving, the number a report divides by
    /// stops being the number it prints.
    ///
    /// **The distinction set was ten until a later decision** struck
    /// `Distinction::Guide`. The **table** did not move with it: the five box junctions stay,
    /// because they are what a table's caller spells to draw its separators, and an
    /// entry a caller needs in order to degrade with the theme is not a claim about what any
    /// component draws. The two counts moving apart is the point — they answer different questions.
    #[test]
    fn the_table_is_twenty_entries_and_the_distinction_set_is_nine() {
        assert_eq!(Glyph::ALL.len(), 20);
        assert_eq!(Distinction::ALL.len(), 9);
        // No entry appears twice, which the four arrows make a live risk: §9's steppers and §7's
        // disclosure markers are one family, and entering them twice under two names is a collapse
        // rather than two entries.
        let mut spelled: Vec<&'static str> = Glyph::ALL
            .iter()
            .map(|g| g.spell(GlyphSet::Extended))
            .collect();
        spelled.sort_unstable();
        spelled.dedup();
        assert_eq!(
            spelled.len(),
            20,
            "two entries spell the same thing at the top rung, which is one entry written twice"
        );
    }

    /// **Six distinctions name a glyph pair and three do not, and the three are the ones that
    /// die.**
    ///
    /// Seven until `Distinction::Guide` was struck, whose pair was
    /// `(VLine, TeeLeft)` and whose drawing — a tree's indent guide — was
    /// established this library does not make.
    ///
    /// The rule as a count. The gate is the *partition*, not the membership: a
    /// distinction quietly losing its carrier would go on reading as carried on both axes while
    /// being carried on one.
    #[test]
    fn six_distinctions_are_carried_by_a_glyph_and_three_by_the_palette_alone() {
        let carried: Vec<Distinction> = Distinction::ALL
            .into_iter()
            .filter(|d| d.carried_by().is_some())
            .collect();
        assert_eq!(carried.len(), 6);
        let uncarried: Vec<Distinction> = Distinction::ALL
            .into_iter()
            .filter(|d| d.carried_by().is_none())
            .collect();
        assert_eq!(
            uncarried,
            vec![Distinction::Hover, Distinction::Fade, Distinction::Status]
        );

        // And every carrier is distinguishable at every rung, which is what *survives all nine
        // cells* means. A pair drawn from inside the box-drawing family would fail here at ASCII —
        // which is why `TeeLeft`/`BottomLeft`, a tree's real *last child* difference, is not a
        // `Distinction`. `VLine`/`TeeLeft` was one for the same reason and is gone for a different
        // one: it survived every rung and named a drawing no component makes.
        for set in [GlyphSet::Ascii, GlyphSet::Unicode, GlyphSet::Extended] {
            for d in carried.iter().copied() {
                let (a, b) = d.carried_by().expect("filtered above");
                assert_ne!(
                    a.spell(set),
                    b.spell(set),
                    "{d:?}'s carrier collapses at {set:?}, so a distinction the map says survives \
                     all nine cells does not"
                );
            }
        }
    }

    /// **The repertoire is a real input to the bits, and the order of the two builders is not.**
    ///
    /// The version that narrowed only inside `resolve` answered differently depending on whether the
    /// repertoire was declared before or after the terminal was — which would have made
    /// `Theme::with_glyphs` a live-switching verb that silently left seven of ten bits describing
    /// the previous rung.
    #[test]
    fn a_repertoire_swap_re_narrows_and_the_two_builders_commute() {
        let ascii_first = Theme::default()
            .with_glyphs(GlyphSet::Ascii)
            .resolve(ColorDepth::TrueColor);
        let ascii_last = Theme::default()
            .resolve(ColorDepth::TrueColor)
            .with_glyphs(GlyphSet::Ascii);
        for d in Distinction::ALL {
            assert_eq!(
                ascii_first.shows(d),
                ascii_last.shows(d),
                "{d:?} depends on the order the theme was built in"
            );
        }

        // A rung that spells a carrier's two halves alike takes the bit away, which is the half of
        // *both axes* the colour tier cannot reach. `Truncation` is the one to watch, because it is
        // C09's defect: spelling the ASCII ellipsis `>` is spelling it `ArrowRight`.
        assert!(ascii_first.shows(Distinction::Truncation));
        assert_eq!(
            Glyph::Ellipsis.spell(GlyphSet::Ascii),
            "~",
            "the ASCII ellipsis is `~`, and `>` is an `ArrowRight`"
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

    /// **Nothing in `theme` names crossterm**, and nothing in it names a Unicode table
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
    /// palette data, and a number belonging to the data must be a relation
    /// rather than an equality — so asserting `(Danger, Warn)` here would be a gate that gets edited
    /// when a later palette replaces this one, rather than one that gets fixed.
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

    /// **The colour-pair question agrees with the role-pair question wherever both can be asked.**
    ///
    /// `roles_differ_on_wire` compares three cached keys minted at
    /// `resolve`; `colours_differ_on_wire` mints one live. Two derivations of one fact, and the join
    /// is the pairs of roles that share a ground and an attribute set — over those, the two verbs
    /// must never disagree, which is what catches the arm that forgets `self.tier` and asks
    /// truecolor.
    ///
    /// **Sixty-nine of the hundred and sixty-nine role pairs qualify** on the shipped default —
    /// the rest carry a different ground or a different attribute set — so the join is a real
    /// fraction of the matrix rather than a corner, and the floor below is a relation because which
    /// pairs qualify is palette data.
    #[test]
    fn the_colour_pair_question_agrees_with_the_role_pair_question() {
        use super::{ColorDepth, Role, Theme};

        let mut asked = 0usize;
        for tier in [
            ColorDepth::TrueColor,
            ColorDepth::Indexed256,
            ColorDepth::Ansi16,
            ColorDepth::None,
        ] {
            let theme = Theme::default().resolve(tier);
            for a in Role::ALL {
                for b in Role::ALL {
                    let (x, y) = (theme.specs[a.index()], theme.specs[b.index()]);
                    // The join: only where the *other two* thirds of the key are equal is the role
                    // pair's answer a statement about the two foregrounds alone.
                    if x.bg != y.bg || x.attrs != y.attrs {
                        continue;
                    }
                    asked += 1;
                    assert_eq!(
                        theme.roles_differ_on_wire(a, b),
                        theme.colours_differ_on_wire(x.fg, y.fg),
                        "{a:?} and {b:?} at {tier:?}: the two verbs disagree about \
                         {:?} against {:?}",
                        x.fg,
                        y.fg
                    );
                }
            }
        }
        assert!(
            asked >= 250,
            "only {asked} role pairs shared a ground and an attribute set — 276 over the four \
             tiers on the shipped default — so this gate has become a corner rather than a \
             fraction of the matrix"
        );
    }

    /// **What the verb is for**: a pair of colours outside the theme, at four tiers.
    ///
    /// The one caller `Theme::custom`'s obligation is addressed to is the one caller
    /// `roles_differ_on_wire` cannot serve, so these answers are the whole point — and they are
    /// asserted **in both directions**, because a verb that always says *they differ* discharges the
    /// obligation and tells the caller nothing.
    #[test]
    fn two_colours_a_theme_never_heard_of_collapse_where_the_tier_says_they_do() {
        use super::{ColorDepth, Rgb, Theme};

        let near = (Rgb::new(0x10, 0x20, 0x30), Rgb::new(0x10, 0x20, 0x31));
        let far = (Rgb::new(0x10, 0x20, 0x30), Rgb::new(0x90, 0x20, 0x30));

        for (tier, near_differ) in [
            (ColorDepth::TrueColor, true),
            (ColorDepth::Indexed256, false),
            (ColorDepth::Ansi16, false),
            (ColorDepth::None, false),
        ] {
            let theme = Theme::default().resolve(tier);
            assert_eq!(
                theme.colours_differ_on_wire(near.0, near.1),
                near_differ,
                "one unit of blue apart, at {tier:?}"
            );
            // Far apart survives everywhere except the tier that has no colour at all.
            assert_eq!(
                theme.colours_differ_on_wire(far.0, far.1),
                tier != ColorDepth::None,
                "half the red channel apart, at {tier:?}"
            );
            // A colour never differs from itself, whatever the tier.
            assert!(!theme.colours_differ_on_wire(near.0, near.0));
        }
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
