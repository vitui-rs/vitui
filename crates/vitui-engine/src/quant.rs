//! Colour narrowed to what the terminal can express, and the one placement that keeps the equality
//! filter exact.
//!
//! # Where it runs, and why there is only one answer
//!
//! **On the render thread, inside the run scan, *before* the comparison with the mirror.** Three
//! things force that and any one of them would be enough:
//!
//! - Quantisation is wire-specific, and everything wire-specific lives below the packet.
//! - **A mirror holding colours the terminal was never sent is not a mirror.**
//! - The app thread's budget is the scarce one (spec §8); the render thread has slack.
//!
//! # The correction nobody was looking for
//!
//! > **"Quantising colour saves almost nothing" is right about frame *size* and silent about frame
//! > *membership*.** If the mirror held canonical colours, two RGB values that quantise to the same
//! > index would compare **unequal** and be re-emitted for no visible change — the equality filter
//! > would under-filter by exactly the amount quantisation collapses. Quantising first makes the
//! > filter exact with respect to the wire, and an animated gradient on a 16-colour terminal nearly
//! > disappears from it.
//!
//! So `--no-color` is a **compatibility feature and not a bandwidth one**, and must not be defended
//! as an optimisation: quantising saves 6% on the three dialogs and 0.1% on a full-screen change,
//! because style-run coalescing already collected the win.
//!
//! # The algorithm is weighted integer distance in RGB
//!
//! CIELAB does not earn a dependency the policy forbids anyway (`deny.toml`): the choice is among
//! **fixed points**, where a perceptual ordering rarely changes the winner. The weights are
//! [`W_R`], [`W_G`] and [`W_B`] — luminance-shaped, and integer, so the whole search is exact
//! arithmetic on `u32`.
//!
//! Neither target set is searched exhaustively, and that is a budget rather than a preference: the
//! adversarial scene is 24 000 **distinct** style words a frame, so a 240-candidate scan per colour
//! would be eleven million weighted distances a frame. The 6×6×6 cube is a product set and weighted
//! Euclidean distance decomposes per channel, so the nearest cube point is the nearest level of each
//! channel taken independently — see [`nearest_cube`]. The twenty-four greys are a line, so the
//! nearest grey is the weighted mean snapped to a step — see [`nearest_grey`]. Two candidates are
//! then compared, which is O(1). Only [`ColorDepth::Ansi16`] scans, and it scans sixteen.
//!
//! # What each depth may target
//!
//! - **At [`ColorDepth::Indexed256`], indices 0..16 are never a quantisation target.** 16..256 are
//!   fixed by specification and identical on every terminal; 0..16 are repainted by the user's
//!   theme. Quantising into the cube is deterministic; quantising into 0..16 is a bet on somebody
//!   else's colour scheme. A colour the *caller* named as index 3 still goes out as index 3 — that
//!   is a request being carried, not a target being chosen.
//! - **At [`ColorDepth::Ansi16`] there is no such escape, which is what makes OSC 4 worth its
//!   sixteen queries.** The targets are the terminal's own sixteen where it answered and xterm's
//!   compiled-in table where it did not.
//! - At [`ColorDepth::None`] every colour is the terminal's own. `reverse` is the one mechanism left
//!   and it is an attribute, so nothing here touches it.
//!
//! # The silence asymmetry is one rule
//!
//! > **Refuse when a guess would be wrong in direction; default when it would be wrong only in
//! > degree.**
//!
//! OSC 11 silence → refuse to mix (`crate::mix`, ADR 0025), because a guessed background inverts a
//! shadow on the opposite theme. OSC 4 silence → use the standard table, because a themed palette
//! makes the nearest match slightly off. One rule, two answers, and it is the rule for any future
//! query.
//!
//! # Contrast preservation is refused, and the reason is mechanical
//!
//! See [`Quantiser::color`], which is where somebody would add it.

use crate::caps::{Capabilities, ColorDepth, Rgb};
use crate::cell::Cell;
use crate::exts::ExtStyle;
use crate::style::{Color, Style, TAG_DEFAULT, TAG_INDEXED, TAG_RGB};

/// xterm's default sixteen, used for an entry OSC 4 did not answer for.
///
/// **Wrong only in degree**, which is what makes defaulting right here and refusing right for the
/// default background (spec §10). Entries 0..8 are xterm's dimmed set and 8..16 its bright one;
/// these are the values `xterm` itself compiles in, not a re-derivation.
///
/// One copy, read by both the compositor — which *resolves* an index a caller named — and the
/// quantiser, which *chooses* one. A second copy could drift, and a drifted xterm table is a wrong
/// answer nothing would fail on.
pub(crate) const ANSI16: [Rgb; 16] = [
    Rgb::new(0x00, 0x00, 0x00),
    Rgb::new(0xcd, 0x00, 0x00),
    Rgb::new(0x00, 0xcd, 0x00),
    Rgb::new(0xcd, 0xcd, 0x00),
    Rgb::new(0x00, 0x00, 0xee),
    Rgb::new(0xcd, 0x00, 0xcd),
    Rgb::new(0x00, 0xcd, 0xcd),
    Rgb::new(0xe5, 0xe5, 0xe5),
    Rgb::new(0x7f, 0x7f, 0x7f),
    Rgb::new(0xff, 0x00, 0x00),
    Rgb::new(0x00, 0xff, 0x00),
    Rgb::new(0xff, 0xff, 0x00),
    Rgb::new(0x5c, 0x5c, 0xff),
    Rgb::new(0xff, 0x00, 0xff),
    Rgb::new(0x00, 0xff, 0xff),
    Rgb::new(0xff, 0xff, 0xff),
];

/// The six levels of the 6×6×6 cube, which is fixed by specification and identical everywhere.
const CUBE: [u8; 6] = [0, 95, 135, 175, 215, 255];

/// The first index of the cube, and the first index a quantisation may target.
const CUBE_BASE: u8 = 16;

/// The first of the twenty-four greys.
const GREY_BASE: u8 = 232;

/// How many greys there are.
const GREYS: u8 = 24;

/// The weights of the distance, luminance-shaped and integer.
///
/// Green carries most of the perceived luminance, blue least, which is the whole of what a
/// perceptual space would tell us about a choice among fixed points. `2 : 4 : 3` sums to nine, so a
/// weighted mean of three channels divides by nine — see [`nearest_grey`].
const W_R: u32 = 2;
/// See [`W_R`].
const W_G: u32 = 4;
/// See [`W_R`].
const W_B: u32 = 3;

/// Palette entry `i`, as channels.
///
/// **0..16 come from OSC 4 where the terminal answered and from [`ANSI16`] where it did not**, and
/// 16..256 are not asked about at all: the cube and the twenty-four greys are fixed by
/// specification and identical on every terminal, which is also why spec §10 refuses to *quantise*
/// into 0..16 while it is happy to *read* them here. Quantising into somebody else's theme is a bet;
/// resolving a colour the caller explicitly named is not.
pub(crate) fn index_channels(caps: &Capabilities, i: u8) -> Rgb {
    match i {
        0..=15 => caps.palette(i).unwrap_or(ANSI16[i as usize]),
        16..=231 => {
            let n = (i - CUBE_BASE) as usize;
            Rgb::new(CUBE[n / 36], CUBE[(n / 6) % 6], CUBE[n % 6])
        }
        // The greys, 232..=255, at 8 and then every tenth value.
        _ => {
            let level = 8 + 10 * (i as u16 - GREY_BASE as u16);
            Rgb::new(level as u8, level as u8, level as u8)
        }
    }
}

/// The weighted square distance between two colours. Integer, and never overflows a `u32`:
/// the largest term is `9 * 255²`.
fn distance(a: Rgb, b: Rgb) -> u32 {
    let d = |x: u8, y: u8| {
        let d = x.abs_diff(y) as u32;
        d * d
    };
    W_R * d(a.r, b.r) + W_G * d(a.g, b.g) + W_B * d(a.b, b.b)
}

/// The nearest level of the cube to one channel, as its index 0..6.
///
/// A six-way comparison rather than arithmetic, because the levels are not evenly spaced: the gap
/// from 0 to 95 is more than twice any other, and `v / 51` — the arithmetic that looks right — puts
/// 47 in the wrong bucket.
fn cube_level(v: u8) -> usize {
    let mut best = 0;
    let mut best_d = u32::MAX;
    for (i, &level) in CUBE.iter().enumerate() {
        let d = v.abs_diff(level) as u32;
        if d < best_d {
            best_d = d;
            best = i;
        }
    }
    best
}

/// The nearest point of the 6×6×6 cube, as an index.
///
/// **Exact, and O(1), because the cube is a product set and the distance decomposes.** Minimising
/// `W_R·(r−x)² + W_G·(g−y)² + W_B·(b−z)²` over `x`, `y` and `z` drawn independently from [`CUBE`] is
/// three independent minimisations, so the weights do not enter at all — which is why this needs no
/// scan and the grey line, where they do enter, needs one comparison.
fn nearest_cube(c: Rgb) -> u8 {
    let (r, g, b) = (cube_level(c.r), cube_level(c.g), cube_level(c.b));
    CUBE_BASE + (36 * r + 6 * g + b) as u8
}

/// The nearest of the twenty-four greys, as an index.
///
/// The greys are a line, so the closest point on it is the **weighted mean** — `(2r + 4g + 3b) / 9`,
/// which is where the weights summing to nine is worth having — snapped to the nearest step of ten
/// from eight and clamped to the ends.
fn nearest_grey(c: Rgb) -> u8 {
    let mean = (W_R * c.r as u32 + W_G * c.g as u32 + W_B * c.b as u32) / (W_R + W_G + W_B);
    // Rounded rather than truncated: the steps are ten apart, so truncation is a bias of half a step
    // toward black on every grey in the picture.
    let step = (mean.saturating_sub(8) + 5) / 10;
    GREY_BASE + (step.min(GREYS as u32 - 1)) as u8
}

/// What the terminal can express, and what everything else becomes.
///
/// Built once per `serialize` from [`Capabilities`], which are immutable for the life of the
/// `Screen` — so this is a fixed-size value, holds no allocation, and can be rebuilt per frame
/// rather than invalidated.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Quantiser {
    depth: ColorDepth,
    /// The sixteen the terminal actually paints: OSC 4 where it answered, [`ANSI16`] where it was
    /// silent. **Only [`ColorDepth::Ansi16`] reads it as a target set**; every depth reads it to
    /// resolve an index a caller named.
    palette: [Rgb; 16],
    /// Whether OSC 8 reaches the wire. Read here so the instrument that compares a frame against
    /// what the terminal shows can drop a link the terminal never got — the *engine* drops it one
    /// layer up, at intern time, which is [`crate::tables::Tables::links_in_key`].
    hyperlinks: bool,
}

impl Quantiser {
    /// A quantiser that narrows nothing.
    ///
    /// What a [`Serializer`](crate::serial::Serializer) holds before it has been handed a terminal:
    /// `new` builds a mirror and a buffer and is told nothing about the far end, and the real one
    /// arrives at the top of every `serialize`. It is *transparent* rather than *default* because
    /// there is no sense in which narrowing nothing is the conservative answer — the conservative
    /// answer is [`ColorDepth::None`], and a serializer that quietly held it would emit a
    /// colourless first frame on a truecolor terminal.
    pub(crate) const fn transparent() -> Quantiser {
        Quantiser {
            depth: ColorDepth::TrueColor,
            palette: ANSI16,
            hyperlinks: true,
        }
    }

    /// Whether this terminal narrows anything at all.
    ///
    /// **The fast path for a comparison that has to be against the wire.** Where this is false,
    /// [`style`](Quantiser::style) is the identity on every word, so a caller comparing a packet's
    /// cells against the mirror's may compare the two **slices** — which is what
    /// `Serializer::row_lands_on` is built on and measured at 2.4x a per-column walk. Where it is
    /// true the walk is the only correct form, and it is bounded by a band rather than by a screen.
    pub(crate) fn narrows(self) -> bool {
        self.depth != ColorDepth::TrueColor
    }

    /// The quantiser for one terminal.
    pub(crate) fn for_terminal(caps: &Capabilities) -> Quantiser {
        let mut palette = ANSI16;
        for (i, entry) in palette.iter_mut().enumerate() {
            if let Some(answered) = caps.palette(i as u8) {
                *entry = answered;
            }
        }
        Quantiser {
            depth: caps.colors,
            palette,
            hyperlinks: caps.hyperlinks,
        }
    }

    /// One colour, narrowed to what this terminal can express.
    ///
    /// **Idempotent at every depth**, which is what lets the SGR path quantise a word the run scan
    /// has already quantised without a flag saying which it is holding.
    ///
    /// # Contrast preservation is refused, and this is where it would go
    ///
    /// A context-aware version of this function — one that took the cell's other colour, or its
    /// neighbours, and chose a target that stayed distinguishable from them — is **refused, and the
    /// reason is mechanical rather than aesthetic.** Two cells carrying an *identical* `u64` would
    /// then require different SGR, so a style run would have to break; and style-run coalescing is
    /// the mechanism that collected the entire wire win, so making quantisation contextual turns a
    /// −6% into a byte *increase*. It would also put a spatial concept into a serializer that has
    /// none and whose runs arrive one row at a time.
    ///
    /// **The consequence is stated rather than mitigated: at sixteen colours a shadow over a similar
    /// background disappears.** That is precisely the case
    /// [ADR 0009](../../../docs/adr/0009-degradation-happens-at-serialise-time.md) hands upward.
    pub(crate) fn color(self, c: Color) -> Color {
        match self.depth {
            // Nothing to narrow to. `reverse` is the one mechanism left and it is an attribute.
            ColorDepth::None => Color::DEFAULT,
            ColorDepth::TrueColor => c,
            ColorDepth::Indexed256 => match c.tag() {
                // A request the caller made, carried rather than chosen: every index 0..256 is
                // expressible here, so there is nothing to decide.
                TAG_DEFAULT | TAG_INDEXED => c,
                TAG_RGB => Color::indexed(self.nearest_indexed(channels(c))),
                _ => Color::DEFAULT,
            },
            ColorDepth::Ansi16 => match c.tag() {
                TAG_DEFAULT => c,
                // 0..16 are expressible; 16..256 are not, and are resolved through the fixed cube
                // before being narrowed like any other colour.
                TAG_INDEXED if c.payload() < 16 => c,
                TAG_INDEXED => Color::indexed(self.nearest_ansi16(self.indexed(c))),
                TAG_RGB => Color::indexed(self.nearest_ansi16(channels(c))),
                _ => Color::DEFAULT,
            },
        }
    }

    /// One style word, narrowed.
    ///
    /// **An extended word comes back unchanged, and that is not an omission.** Bits 51..0 are a
    /// handle, so there are no colours in the word to narrow — they are in the table, and they are
    /// narrowed by [`channels`](Quantiser::channels) on the way to the wire. The consequence is
    /// written down where it can be acted on: see [`crate::serial::Mirror`].
    pub(crate) fn style(self, s: Style) -> Style {
        if self.depth == ColorDepth::TrueColor || s.is_extended() {
            return s;
        }
        let (fg, bg) = (self.color(s.foreground()), self.color(s.background()));
        Style::inline(s.attr_word(), fg, bg)
    }

    /// One cell, narrowed.
    ///
    /// The grapheme is never touched — the engine substitutes no glyph anywhere (spec §10), and a
    /// glyph set is not a colour depth.
    ///
    /// **It exists so that two comparisons are provably the same predicate.**
    /// `Serializer::damaged_span_is_on_the_terminal` compares a packet's cells against the mirror's
    /// with a slice `==` where nothing narrows and a walk where something does, and a walk written
    /// over *fields* is a walk that silently stops covering a field somebody adds — `Cell::_reserved`
    /// is zero in every cell today and is documented as not being slack to reclaim. Narrowing to a
    /// whole `Cell` keeps both arms total.
    pub(crate) fn cell(self, c: Cell) -> Cell {
        Cell::new(c.grapheme, self.style(c.style))
    }

    /// The four channels of an extended entry, narrowed — and the link dropped where the terminal
    /// cannot express one.
    ///
    /// The link is dropped here for the **instrument's** sake, not the wire's: the serializer
    /// already emits no OSC 8 without the capability, and the engine drops the channel from the
    /// intern *key* one layer up so the table never holds it. This is what lets a comparison of a
    /// frame against what the terminal shows be written once.
    pub(crate) fn channels(self, e: ExtStyle) -> ExtStyle {
        ExtStyle {
            fg: self.color(e.fg),
            bg: self.color(e.bg),
            ul: self.color(e.ul),
            link: if self.hyperlinks {
                e.link
            } else {
                crate::exts::LinkId::NONE
            },
        }
    }

    /// An indexed colour as channels, through this terminal's own palette.
    fn indexed(self, c: Color) -> Rgb {
        let i = c.payload() as u8;
        match i {
            0..=15 => self.palette[i as usize],
            // The cube and the greys, which no terminal is asked about.
            _ => index_channels_fixed(i),
        }
    }

    /// The nearest of the cube and the greys — **never one of the low sixteen.**
    fn nearest_indexed(self, c: Rgb) -> u8 {
        let cube = nearest_cube(c);
        let grey = nearest_grey(c);
        if distance(c, index_channels_fixed(cube)) <= distance(c, index_channels_fixed(grey)) {
            cube
        } else {
            grey
        }
    }

    /// The nearest of the terminal's own sixteen. The one place a scan is the whole algorithm, and
    /// sixteen is short enough that the memo carries it.
    fn nearest_ansi16(self, c: Rgb) -> u8 {
        let mut best = 0u8;
        let mut best_d = u32::MAX;
        for (i, &entry) in self.palette.iter().enumerate() {
            let d = distance(c, entry);
            if d < best_d {
                best_d = d;
                best = i as u8;
            }
        }
        best
    }
}

/// The channels of an index no terminal is asked about: 16..256.
///
/// Split out from [`index_channels`] because the quantiser reaches it without a `Capabilities` in
/// hand — it has already resolved the low sixteen into its own array — and because a caller that
/// passed a low index here would be asking the wrong question.
fn index_channels_fixed(i: u8) -> Rgb {
    debug_assert!(
        i >= CUBE_BASE,
        "the low sixteen are the terminal's, not the specification's"
    );
    match i {
        16..=231 => {
            let n = (i - CUBE_BASE) as usize;
            Rgb::new(CUBE[n / 36], CUBE[(n / 6) % 6], CUBE[n % 6])
        }
        _ => {
            let level = 8 + 10 * (i as u16 - GREY_BASE as u16);
            Rgb::new(level as u8, level as u8, level as u8)
        }
    }
}

/// One cell as the wire can say it: the grapheme, the eleven attribute bits, and four channels
/// resolved out of the handle tables and narrowed.
///
/// **The shape a frame is compared against the terminal in, and it exists because quantisation
/// makes raw cell equality the wrong question.** Ticket 03's round trip compared whole `Cell`s,
/// which was exact while every colour the frame held reached the wire unchanged. It does not any
/// more: a frame holds what the application asked for and the terminal holds what the depth could
/// express, so the two disagree by construction on any terminal below truecolor and agreeing would
/// mean the engine had not narrowed anything.
///
/// It is **not** a weaker comparison. Resolving the handle is *stronger* than comparing it — a
/// handle pointing at the wrong entry fails here and compares equal there — and the two channels
/// that forced the fold are compared by value, so a lost hyperlink or a lost underline colour is
/// still a failure. What it stops asserting is the *spelling*: whether one screen's cell reached
/// the wire as an inline word or as a table entry is the engine's business, and the terminal has no
/// opinion about it.
#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct OnTheWire {
    grapheme: crate::cell::GraphemeId,
    attrs: u64,
    channels: ExtStyle,
}

/// One cell, as the wire can say it. See [`OnTheWire`].
///
/// Applied to **both** sides of every comparison, and idempotence is what makes that sound: the
/// terminal's cells were minted from bytes that were already narrowed, so narrowing them again is
/// the identity, while the frame's cells are narrowed here for the first and only time.
#[cfg(test)]
pub(crate) fn on_the_wire(q: Quantiser, tables: &crate::tables::Tables, c: Cell) -> OnTheWire {
    OnTheWire {
        grapheme: c.grapheme,
        attrs: c.style.attr_word(),
        channels: q.channels(crate::restyle::channels(tables, c.style)),
    }
}

/// An RGB colour's three channels.
fn channels(c: Color) -> Rgb {
    let p = c.payload();
    Rgb::new((p >> 16) as u8, (p >> 8) as u8, p as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caps::Capabilities;

    fn at(depth: ColorDepth) -> Quantiser {
        Quantiser::for_terminal(&Capabilities::answering(depth, None, None))
    }

    /// The index a colour lands on, for a test that wants to name one number.
    fn index_of(q: Quantiser, c: Color) -> u8 {
        let out = q.color(c);
        assert_eq!(out.tag(), TAG_INDEXED, "expected an index, got {out:?}");
        out.payload() as u8
    }

    #[test]
    fn truecolor_narrows_nothing() {
        let q = at(ColorDepth::TrueColor);
        for c in [
            Color::DEFAULT,
            Color::indexed(0),
            Color::indexed(200),
            Color::rgb(1, 2, 3),
        ] {
            assert_eq!(q.color(c), c);
        }
    }

    #[test]
    fn no_colour_at_all_leaves_the_terminals_own() {
        let q = at(ColorDepth::None);
        for c in [Color::indexed(0), Color::indexed(200), Color::rgb(1, 2, 3)] {
            assert_eq!(q.color(c), Color::DEFAULT);
        }
    }

    #[test]
    fn every_depth_is_idempotent() {
        // What lets the SGR path narrow a word the run scan has already narrowed without a flag
        // saying which of the two it is holding.
        for depth in [
            ColorDepth::None,
            ColorDepth::Ansi16,
            ColorDepth::Indexed256,
            ColorDepth::TrueColor,
        ] {
            let q = at(depth);
            for c in [
                Color::DEFAULT,
                Color::indexed(0),
                Color::indexed(7),
                Color::indexed(15),
                Color::indexed(16),
                Color::indexed(200),
                Color::indexed(255),
                Color::rgb(0, 0, 0),
                Color::rgb(9, 8, 7),
                Color::rgb(255, 255, 255),
                Color::rgb(0x12, 0x34, 0x56),
            ] {
                let once = q.color(c);
                assert_eq!(q.color(once), once, "{depth:?} is not idempotent on {c:?}");
            }
        }
    }

    #[test]
    fn at_256_no_rgb_colour_ever_targets_an_index_under_sixteen() {
        // The gate, over the whole space rather than over a sample: every one of the 16 777 216 RGB
        // values, at a stride that visits every cube bucket and every grey step and is coprime with
        // neither, would be sixteen million quantisations. A stride over the cube's own resolution
        // is what makes this exhaustive in the dimension that matters — the *bucket*, not the value.
        let q = at(ColorDepth::Indexed256);
        for r in 0..=255u8 {
            for g in (0..=255u8).step_by(17) {
                for b in (0..=255u8).step_by(17) {
                    let i = index_of(q, Color::rgb(r, g, b));
                    assert!(
                        i >= CUBE_BASE,
                        "rgb({r}, {g}, {b}) quantised to index {i}, which is the user's theme"
                    );
                }
            }
        }
    }

    #[test]
    fn at_256_an_index_the_caller_named_is_carried_rather_than_chosen() {
        // The distinction the gate above is about: 0..16 is not a *target*, and an index a caller
        // asked for is a request rather than a choice. A quantiser that collapsed it would be
        // repainting the application's own colour scheme.
        let q = at(ColorDepth::Indexed256);
        for i in [0u8, 3, 7, 8, 15, 16, 231, 232, 255] {
            assert_eq!(q.color(Color::indexed(i)), Color::indexed(i));
        }
    }

    #[test]
    fn the_cube_and_the_greys_are_hit_exactly_where_a_colour_is_one_of_them() {
        let q = at(ColorDepth::Indexed256);
        for i in CUBE_BASE..=255 {
            let c = index_channels_fixed(i);
            let got = index_of(q, Color::rgb(c.r, c.g, c.b));
            assert_eq!(
                index_channels_fixed(got),
                c,
                "index {i} did not round-trip through its own channels"
            );
        }
    }

    #[test]
    fn a_grey_prefers_a_grey_and_a_colour_prefers_the_cube() {
        let q = at(ColorDepth::Indexed256);
        // 100, 100, 100 is between two cube levels and within two of grey step 9 (98).
        let grey = index_of(q, Color::rgb(100, 100, 100));
        assert!(grey >= GREY_BASE, "a near-grey chose cube index {grey}");
        let red = index_of(q, Color::rgb(240, 10, 10));
        assert!(
            (CUBE_BASE..GREY_BASE).contains(&red),
            "a saturated red chose index {red}"
        );
    }

    #[test]
    fn at_sixteen_the_targets_are_xterms_own_where_osc_4_was_silent() {
        let q = at(ColorDepth::Ansi16);
        // Exactly xterm's entries, so each one must choose itself.
        for (i, entry) in ANSI16.iter().enumerate() {
            assert_eq!(
                index_of(q, Color::rgb(entry.r, entry.g, entry.b)),
                i as u8,
                "xterm entry {i} did not choose itself"
            );
        }
    }

    #[test]
    fn at_sixteen_the_targets_are_the_terminals_own_where_osc_4_answered() {
        // The whole reason OSC 4 is worth sixteen queries. A terminal whose every palette entry is
        // one colour has one target, so *every* colour lands on entry 0 — which is the strongest
        // form of "the answer was read", because no default table could produce it.
        let themed = Capabilities::answering_palette(Rgb::new(0x11, 0x22, 0x33));
        let mut caps = themed;
        caps.colors = ColorDepth::Ansi16;
        let q = Quantiser::for_terminal(&caps);
        for c in [
            Color::rgb(255, 255, 255),
            Color::rgb(0, 0, 0),
            Color::rgb(0xcd, 0, 0),
        ] {
            assert_eq!(index_of(q, c), 0, "the terminal's own palette was not read");
        }
        // And the silent terminal disagrees with it, so the test above is not passing for a reason
        // that has nothing to do with the palette.
        assert_ne!(
            index_of(at(ColorDepth::Ansi16), Color::rgb(255, 255, 255)),
            0
        );
    }

    #[test]
    fn at_sixteen_an_index_above_fifteen_is_resolved_before_it_is_narrowed() {
        let q = at(ColorDepth::Ansi16);
        // 231 is the cube's white corner and 16 is its black one, so each has an obvious nearest.
        assert_eq!(index_of(q, Color::indexed(231)), 15);
        assert_eq!(index_of(q, Color::indexed(16)), 0);
        // And an index the terminal can already spell is left exactly as it is.
        for i in 0..16u8 {
            assert_eq!(q.color(Color::indexed(i)), Color::indexed(i));
        }
    }

    #[test]
    fn two_close_rgb_values_collapse_to_one_index_at_sixteen() {
        // The mechanism the equality filter's exactness rests on, as a statement about colour alone:
        // this is what makes two frames of an animated gradient compare *equal* once the mirror
        // holds what the wire said.
        let q = at(ColorDepth::Ansi16);
        assert_eq!(
            q.color(Color::rgb(0xcc, 0x02, 0x01)),
            q.color(Color::rgb(0xcf, 0x00, 0x03))
        );
    }

    #[test]
    fn an_extended_word_comes_back_untouched_and_its_channels_do_not() {
        let q = at(ColorDepth::Ansi16);
        let word = Style::extended(0, 7);
        assert_eq!(q.style(word), word, "the handle is not a colour");
        let e = ExtStyle {
            fg: Color::rgb(0xcd, 0, 0),
            bg: Color::DEFAULT,
            ul: Color::rgb(0, 0, 0xee),
            link: crate::exts::LinkId::NONE,
        };
        let narrowed = q.channels(e);
        assert_eq!(narrowed.fg, Color::indexed(1));
        assert_eq!(narrowed.bg, Color::DEFAULT);
        assert_eq!(narrowed.ul, Color::indexed(4));
    }

    #[test]
    fn an_inline_words_attributes_survive_being_narrowed() {
        // The contract impl 07 wrote for `restyle`, asked of this function: it rewrites the channels
        // it is about and preserves everything else.
        let q = at(ColorDepth::Ansi16);
        let s = Style::new()
            .bold()
            .italic()
            .fg(Color::rgb(0xcd, 0, 0))
            .bg(Color::rgb(0, 0, 0));
        let narrowed = q.style(s);
        assert_eq!(narrowed.attr_word(), s.attr_word());
        assert_eq!(narrowed.foreground(), Color::indexed(1));
        assert_eq!(narrowed.background(), Color::indexed(0));
    }

    #[test]
    fn the_index_tables_agree_with_the_compositors_own() {
        // Two readers, one table: `crate::mix` resolves an index a caller named and this module
        // chooses one. A second copy of xterm's sixteen or of the cube could drift, and a drifted
        // table is a wrong colour nothing fails on.
        let caps = Capabilities::answering(ColorDepth::TrueColor, None, None);
        for i in 0..=255u8 {
            let resolved = index_channels(&caps, i);
            if i >= CUBE_BASE {
                assert_eq!(resolved, index_channels_fixed(i));
            } else {
                assert_eq!(resolved, ANSI16[i as usize]);
            }
        }
    }

    #[test]
    fn a_grey_step_is_rounded_rather_than_truncated() {
        // Truncation would bias every grey in the picture half a step toward black.
        // Step 9 is 98 and step 10 is 108, so 104 is nearer the second.
        assert_eq!(nearest_grey(Rgb::new(104, 104, 104)), GREY_BASE + 10);
        assert_eq!(nearest_grey(Rgb::new(98, 98, 98)), GREY_BASE + 9);
        // And the ends clamp rather than wrapping.
        assert_eq!(nearest_grey(Rgb::new(0, 0, 0)), GREY_BASE);
        assert_eq!(
            nearest_grey(Rgb::new(255, 255, 255)),
            GREY_BASE + (GREYS - 1)
        );
    }

    #[test]
    fn a_cube_level_is_chosen_by_comparison_rather_than_by_division() {
        // `v / 51` is the arithmetic that looks right: it puts 47 at level 0 where the levels are
        // 0 and 95 and 47 is nearer 0 — and puts 48 at level 0 too, where 95 is nearer nothing.
        // The gap from 0 to 95 is more than twice any other, which is what breaks the division.
        assert_eq!(cube_level(47), 0);
        assert_eq!(cube_level(48), 1);
        assert_eq!(cube_level(0), 0);
        assert_eq!(cube_level(255), 5);
        assert_eq!(cube_level(116), 2);
    }
}
