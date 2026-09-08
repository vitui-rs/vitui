//! **What a colour becomes on the way to the terminal**, in the runtime's own arithmetic.
//!
//! # Why this file exists at all, which is the uncomfortable part
//!
//! `Theme::roles_differ_on_wire` and every `Distinction` bit are questions about what the *terminal*
//! shows, not about what the palette says. Answering them needs quantisation — and the engine's
//! quantiser is `pub(crate)`, its `Color` is an opaque newtype whose channels are `pub(crate)`, and
//! there is no public API anywhere that narrows a colour or reads one back. **So the runtime cannot
//! ask the engine this question, and there is no arrangement of the public surface that lets it.**
//!
//! That leaves two honest options and one dishonest one. The dishonest one is to answer from the
//! authored palette and call it the wire — which is precisely the defect `Theme::resolve` exists to
//! fix, a bit describing the palette the author typed rather than the one the terminal shows. Of the
//! two honest ones, asking the engine to export a quantiser is a change to the engine's public
//! surface that the engine map has not decided, and **this backlog does not reopen decisions**; so
//! the arithmetic is copied here, and the copy is *gated against the original*.
//!
//! # The copy is gated against the engine, not against a written-down number
//!
//! This is the whole difference between this and the second-copy-of-the-UCD failure that
//! `crate::layout::text` refuses. A copy checked against a constant somebody transcribed drifts
//! silently the day the original changes. `tests/wire.rs` drives the **real engine** through a real
//! `Screen` at each tier, reads the SGR bytes it actually emitted, and requires this file to agree.
//! If the engine's quantiser changes, that test fails — which is what a copy owes its original.
//!
//! **It found something on its first run.** The theme documents record `Face`'s background arriving
//! as **index 59** at 256 colours; the engine emits **237**. 59 is the nearest point of the 6×6×6
//! cube and 237 is the nearest of the twenty-four greys, which is four times closer. So the recorded
//! number came from a prototype whose quantiser did not consult the grey ramp — and the pair it was
//! the evidence for does not collapse at all.
//!
//! Provenance: `crates/vitui-engine/src/quant.rs` as of impl 26 — `ANSI16`, `CUBE`, the `2:4:3`
//! weights, `cube_level`, `nearest_cube`, `nearest_grey` and the choice between the last two.

use vitui_engine::{ColorDepth, Rgb};

/// xterm's own sixteen, which is what a terminal that has not been asked will show.
///
/// **Only ever read, never quantised into** above `Ansi16` — the engine refuses to quantise into the
/// low sixteen at 256 colours because they are somebody else's theme, and this copy keeps that
/// refusal, because a copy that improved on its original would *be* the drift.
const ANSI16: [Rgb; 16] = [
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

/// The six levels of the 6×6×6 cube, fixed by specification and identical on every terminal.
const CUBE: [u8; 6] = [0, 95, 135, 175, 215, 255];

/// The first index of the cube.
const CUBE_BASE: u8 = 16;

/// The first of the twenty-four greys.
const GREY_BASE: u8 = 232;

/// How many greys there are.
const GREYS: u8 = 24;

/// Luminance-shaped integer weights, summing to nine so that a weighted mean divides by nine.
const W_R: u32 = 2;
/// See [`W_R`].
const W_G: u32 = 4;
/// See [`W_R`].
const W_B: u32 = 3;

/// The gap between two bytes, without `abs_diff`, which is not `const`.
const fn gap(x: u8, y: u8) -> u32 {
    if x > y {
        (x - y) as u32
    } else {
        (y - x) as u32
    }
}

/// The weighted square distance between two colours.
const fn distance(a: Rgb, b: Rgb) -> u32 {
    let (dr, dg, db) = (gap(a.r, b.r), gap(a.g, b.g), gap(a.b, b.b));
    W_R * dr * dr + W_G * dg * dg + W_B * db * db
}

/// The nearest level of the cube to one channel, as its index 0..6.
///
/// A six-way comparison rather than `v / 51`: the levels are not evenly spaced — the gap from 0 to 95
/// is more than twice any other — and the arithmetic that looks right puts 48 in the wrong bucket.
const fn cube_level(v: u8) -> usize {
    let mut best = 0;
    let mut best_d = u32::MAX;
    let mut i = 0;
    while i < CUBE.len() {
        let d = gap(v, CUBE[i]);
        if d < best_d {
            best_d = d;
            best = i;
        }
        i += 1;
    }
    best
}

/// The channels of a fixed index, 16..=255 — the cube and the greys, which no terminal is asked
/// about.
const fn fixed_channels(i: u8) -> Rgb {
    if i < GREY_BASE {
        let n = (i - CUBE_BASE) as usize;
        Rgb::new(CUBE[n / 36], CUBE[(n / 6) % 6], CUBE[n % 6])
    } else {
        // The last grey is `8 + 10 * 23 == 238`, so the byte cannot overflow and the cast is exact.
        let level = (8 + 10 * (i as u16 - GREY_BASE as u16)) as u8;
        Rgb::new(level, level, level)
    }
}

/// The nearest point of the cube, as an index.
const fn nearest_cube(c: Rgb) -> u8 {
    let (r, g, b) = (cube_level(c.r), cube_level(c.g), cube_level(c.b));
    // `36 * 5 + 6 * 5 + 5 == 215`, and `16 + 215 == 231`: both casts are exact by construction.
    CUBE_BASE + (36 * r + 6 * g + b) as u8
}

/// The nearest of the twenty-four greys, as an index.
///
/// The greys are a line, so the closest point on it is the weighted mean snapped to the nearest step
/// of ten from eight. **Rounded rather than truncated**: the steps are ten apart, so truncating is a
/// bias of half a step toward black on every grey in the picture.
const fn nearest_grey(c: Rgb) -> u8 {
    let mean = (W_R * c.r as u32 + W_G * c.g as u32 + W_B * c.b as u32) / (W_R + W_G + W_B);
    let step = (mean.saturating_sub(8) + 5) / 10;
    let last = GREYS as u32 - 1;
    let step = if step > last { last } else { step };
    GREY_BASE + step as u8
}

/// The index a 256-colour terminal is given: the nearer of the cube and the greys, **never one of
/// the low sixteen.**
///
/// **The grey ramp is why the recorded 59 was wrong.** Twenty-four greys ten apart resolve a
/// near-neutral colour four times more closely than a cube whose levels are 95 apart at the dark
/// end, so any colour near the diagonal lands on a grey. `#313244` is 237, not 59.
const fn index_256(c: Rgb) -> u8 {
    let cube = nearest_cube(c);
    let grey = nearest_grey(c);
    if distance(c, fixed_channels(cube)) <= distance(c, fixed_channels(grey)) {
        cube
    } else {
        grey
    }
}

/// The index a sixteen-colour terminal is given: a scan over xterm's own sixteen, which is short
/// enough that a scan is the whole algorithm.
const fn index_16(c: Rgb) -> u8 {
    let mut best = 0u8;
    let mut best_d = u32::MAX;
    let mut i = 0;
    while i < ANSI16.len() {
        let d = distance(c, ANSI16[i]);
        if d < best_d {
            best_d = d;
            // Sixteen entries, so the cast is exact by construction.
            best = i as u8;
        }
        i += 1;
    }
    best
}

/// What one colour becomes at one tier, as a value two colours can be compared by.
///
/// A `u32` rather than an enum because it is only ever compared, never read: two roles differ on the
/// wire exactly when their keys differ, and what the key *means* is this file's business. The tag in
/// the high byte is what stops an index colliding with a channel triple.
pub(super) const fn key(c: Rgb, tier: ColorDepth) -> u32 {
    match tier {
        // Nothing is narrowed, so the colour is its own key.
        ColorDepth::TrueColor => ((c.r as u32) << 16) | ((c.g as u32) << 8) | c.b as u32,
        ColorDepth::Indexed256 => 1 << 24 | index_256(c) as u32,
        ColorDepth::Ansi16 => 2 << 24 | index_16(c) as u32,
        // **No colour at all, so every colour is the same colour.** Not a degenerate case to be
        // guarded against: `--no-color` is a supported tier, and on it every pair of roles is
        // indistinguishable by colour, which is exactly what a component asking `shows` needs told.
        ColorDepth::None => 3 << 24,
    }
}

/// The index a 256-colour terminal is given. Used by `Roles::from_palette`'s `pick`, which needs to
/// know whether two candidate colours are the same colour on a 256-colour terminal.
///
/// **`const`, and that is what puts a shipped theme in `.rodata`.** The import walks the ramp by what
/// this function can still separate, so an import that runs at compile time needs a quantiser that
/// runs at compile time — every function in this file is `const` for that one reason. Nothing about
/// the arithmetic changed, and `against_the_engine` still drives the real engine to say so.
pub(super) const fn at_256(c: Rgb) -> u8 {
    index_256(c)
}

/// The index a sixteen-colour terminal is given, for the report that re-takes the C16 pair count
/// through an operator palette.
///
/// **Only ever an instrument, which is why it is `cfg(test)` and not on the public surface.**
/// It can never be more than one: nothing in this process may read the sixteen
/// colours the terminal is configured with, so a count taken at sixteen colours is a lower bound, and
/// an operator palette is a fixture that says *how much* of one — never a fact a component may read.
#[cfg(test)]
pub(super) const fn at_16(c: Rgb) -> u8 {
    index_16(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The four values this file was written against, taken from the **engine's own output** rather
    /// than from a document. `the_copy_agrees_with_the_engine_on_the_wire` below is the version that
    /// drives a real `Screen`; these are the same numbers as a fast unit check.
    #[test]
    fn the_two_face_colours_narrow_the_way_the_engine_narrows_them() {
        let face = Rgb::new(0x31, 0x32, 0x44);
        let hover = Rgb::new(0x45, 0x47, 0x5a);
        assert_eq!(index_256(face), 237, "the engine emits 38:5:237");
        assert_eq!(index_256(hover), 239, "the engine emits 38:5:239");
        assert_eq!(index_16(face), 0, "the engine emits SGR 30");
        assert_eq!(index_16(hover), 8, "the engine emits SGR 90");
    }

    /// **And the two do not collapse at 256 colours**, which the theme documents say they do.
    ///
    /// 237 against 239 — two steps of the grey ramp apart. The recorded collapse was at index 59,
    /// the nearest cube point, from a prototype that did not consult the greys.
    #[test]
    fn the_named_c256_collapse_does_not_reproduce() {
        let face = Rgb::new(0x31, 0x32, 0x44);
        let hover = Rgb::new(0x45, 0x47, 0x5a);
        assert_ne!(
            key(face, ColorDepth::Indexed256),
            key(hover, ColorDepth::Indexed256),
            "the named C256 casualty is distinguishable against the shipped engine"
        );
    }

    /// The unevenly-spaced cube levels, which is the arithmetic `v / 51` gets wrong.
    #[test]
    fn the_cube_levels_are_not_evenly_spaced() {
        assert_eq!(cube_level(0), 0);
        assert_eq!(cube_level(47), 0, "47 is nearer 0 than 95");
        assert_eq!(cube_level(48), 1, "but 48 is nearer 95, and 48 / 51 says 0");
        assert_eq!(cube_level(255), 5);
    }

    /// No colour at all makes every colour one colour, which is the answer a component needs.
    #[test]
    fn no_colour_at_all_makes_every_colour_the_same_colour() {
        assert_eq!(
            key(Rgb::new(0, 0, 0), ColorDepth::None),
            key(Rgb::new(255, 255, 255), ColorDepth::None)
        );
        assert_ne!(
            key(Rgb::new(0, 0, 0), ColorDepth::TrueColor),
            key(Rgb::new(255, 255, 255), ColorDepth::TrueColor)
        );
    }

    /// Truecolor narrows nothing, so two colours differ iff they differ.
    #[test]
    fn truecolor_narrows_nothing() {
        for (a, b) in [((0u8, 0u8, 0u8), (0, 0, 1)), ((49, 50, 68), (49, 50, 69))] {
            assert_ne!(
                key(Rgb::new(a.0, a.1, a.2), ColorDepth::TrueColor),
                key(Rgb::new(b.0, b.1, b.2), ColorDepth::TrueColor),
                "one channel apart is apart at truecolor"
            );
        }
    }
}

#[cfg(test)]
mod against_the_engine {
    use std::sync::{Arc, Mutex};

    use vitui_engine::{
        Clock, Color, ColorDepth, Config, Engine, Output, Overrides, Rect, Rgb, Style,
    };

    use super::{index_16, index_256};

    /// A sink the test can read back after the screen has been dropped.
    struct Shared(Arc<Mutex<Vec<u8>>>);

    impl std::io::Write for Shared {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0
                .lock()
                .expect("no panic holds this lock")
                .extend_from_slice(b);
            Ok(b.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// Draw one cell in `c` at `depth` and return every byte the engine wrote.
    fn wire(depth: ColorDepth, c: Rgb) -> String {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let (mut screen, _wake) = Engine::new(Config {
            clock: Clock::Manual,
            output: Output::Sink(Box::new(Shared(Arc::clone(&buf)))),
            size: (4, 1),
            overrides: Overrides {
                colors: Some(depth),
                // Declared, so the engine has a ground to narrow against and the answer is a property
                // of the arithmetic rather than of what it guessed about the terminal.
                default_fg: Some(Rgb::new(0xcd, 0xd6, 0xf4)),
                default_bg: Some(Rgb::new(0x1e, 0x1e, 0x2e)),
                ..Default::default()
            },
            ..Default::default()
        })
        .attach()
        .expect("attaching to a sink cannot fail");
        let layer = screen.layers().add_content(0, Rect::new(0, 0, 4, 1), true);
        screen
            .layers()
            .view(layer)
            .expect("the layer just added")
            .text(0, 0, "X", Style::new().fg(Color::rgb(c.r, c.g, c.b)));
        assert!(screen.present().submitted);
        drop(screen);
        let bytes = buf.lock().expect("no panic holds this lock").clone();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    /// **The copy is checked against the original, by driving the original.**
    ///
    /// This is what a copied table owes the thing it was copied from, and it is the difference between
    /// this file and the second-copy-of-the-UCD failure `crate::layout::text` refuses. A copy checked
    /// against a transcribed constant drifts silently the day the original changes; a copy checked
    /// against the engine's own emitted bytes cannot.
    ///
    /// **It is also how the recorded index 59 was found to be wrong.** The engine emits `38:5:237`.
    #[test]
    fn the_copy_agrees_with_the_engine_on_the_wire() {
        // A spread across the space rather than two convenient greys: near-neutrals (where the grey
        // ramp wins), saturated accents (where the cube wins), and the ends.
        let corpus = [
            Rgb::new(0x31, 0x32, 0x44),
            Rgb::new(0x45, 0x47, 0x5a),
            Rgb::new(0xcd, 0xd6, 0xf4),
            Rgb::new(0xf3, 0x8b, 0xa8),
            Rgb::new(0xa6, 0xe3, 0xa1),
            Rgb::new(0x00, 0x00, 0x00),
            Rgb::new(0xff, 0xff, 0xff),
            Rgb::new(0x80, 0x00, 0x00),
            Rgb::new(0x00, 0x5f, 0x87),
            Rgb::new(0x2f, 0x2f, 0x2f),
        ];
        for c in corpus {
            let emitted = wire(ColorDepth::Indexed256, c);
            let want = format!("38:5:{}m", index_256(c));
            assert!(
                emitted.contains(&want),
                "at 256 colours the engine emitted {emitted:?} for {c:?}, and this file says {want}"
            );
        }
        for c in corpus {
            let emitted = wire(ColorDepth::Ansi16, c);
            let i = index_16(c);
            // The engine spells 0..8 as SGR 30..38 and 8..16 as SGR 90..98, so the expectation is
            // written in the encoding rather than in the index — which is also the only form in which
            // this comparison is honest, because the index is this file's answer and the SGR is the
            // engine's.
            let sgr = if i < 8 {
                30 + u16::from(i)
            } else {
                90 + u16::from(i - 8)
            };
            let literal = format!("\u{1b}[{sgr}m");
            assert!(
                emitted.contains(&literal),
                "at sixteen colours the engine emitted {emitted:?} for {c:?}, and this file says \
                 index {i}, which is SGR {sgr}"
            );
        }
    }

    /// Truecolor narrows nothing, which is the control: if this fails, the harness is measuring the
    /// wrong thing rather than the arithmetic being wrong.
    #[test]
    fn the_engine_narrows_nothing_at_truecolor() {
        let c = Rgb::new(0x31, 0x32, 0x44);
        let emitted = wire(ColorDepth::TrueColor, c);
        assert!(
            emitted.contains("38:2::49:50:68m"),
            "truecolor should carry the channels through: {emitted:?}"
        );
    }
}
