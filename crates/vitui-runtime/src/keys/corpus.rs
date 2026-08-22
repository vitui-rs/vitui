//! A thirty-three-binding corpus and the synthetic terminal that presses it.
//!
//! **None of this ships as runtime behaviour.** It is the scene the ticket asks for, and it has to be
//! a scene rather than a live capability check for the reason the module comment gives: a terminal
//! cannot report which of the two legacy cases it is in, so the only way to know what a map reaches
//! is to press it against a model.
//!
//! `#[path]`-included by the report and compiled under `cfg(test)` in the library, which is the
//! arrangement `crate::screen` uses and for the same reason: an instrument is not part of the
//! library.
//!
//! **The runtime ships no layout table and never will** — it has no dependencies and the terminal
//! already knows.

use std::time::Instant;

use vitui_engine::{Key, KeyCode, KeyKind, KeyText, Mods};

use vitui_runtime::keys::{ActionId, Chord, KeyMap, On};

/// Save.
pub const SAVE: ActionId = 1;
/// Quit.
pub const QUIT: ActionId = 2;
/// Top of file, for the `g g` sequence.
pub const TOP: ActionId = 100;
/// Bottom of file.
pub const BOTTOM: ActionId = 101;

/// Fourteen `Ctrl`+letter accelerators. **The family that a terminal without a Latin fallback
/// loses entirely.**
pub const CTRL: [(char, ActionId, &str); 14] = [
    ('s', SAVE, "Save"),
    ('q', QUIT, "Quit"),
    ('o', 3, "Open"),
    ('n', 4, "New"),
    ('w', 5, "Close"),
    ('f', 6, "Find"),
    ('g', 7, "Find next"),
    ('r', 8, "Replace"),
    ('p', 9, "Command palette"),
    ('b', 10, "Toggle sidebar"),
    ('e', 11, "Recent files"),
    ('d', 12, "Duplicate line"),
    ('k', 13, "Delete line"),
    ('l', 14, "Select line"),
];

/// Six function keys. **Escape sequences, so whole at every tier.**
pub const FKEYS: [(u8, ActionId, &str); 6] = [
    (1, 20, "Help"),
    (2, 21, "Rename"),
    (3, 22, "Find next"),
    (5, 23, "Refresh"),
    (9, 24, "Breakpoint"),
    (12, 25, "Go to definition"),
];

/// Four named keys. Also escape sequences.
pub const NAMED: [(KeyCode, ActionId, &str); 4] = [
    (KeyCode::Escape, 30, "Cancel"),
    (KeyCode::Enter, 31, "Confirm"),
    (KeyCode::Delete, 32, "Delete"),
    (KeyCode::Backspace, 33, "Back"),
];

/// Three `Alt`+named. Escape sequences too.
pub const ALT: [(KeyCode, ActionId, &str); 3] = [
    (KeyCode::Left, 40, "Back"),
    (KeyCode::Right, 41, "Forward"),
    (KeyCode::Enter, 42, "Open in split"),
];

/// Six bare letters. **The family that goes from 6 of 6 to 1 of 6 below flag 4.**
///
/// The survivor is `?`, and the reason is worth having: it is the one key in this family that the
/// Cyrillic layout does not remap at all. `/` looks like it should survive — its Cyrillic position
/// prints `.`, which is still ASCII — and it does not, because below flag 4 the engine infers `code`
/// from the text and a binding on `Char('/')` meets `Char('.')`. **Still-ASCII is not still-the-same-
/// key**, and that distinction is the difference between one survivor and two.
pub const LETTERS: [(char, ActionId, &str); 6] = [
    ('j', 50, "Down"),
    ('k', 51, "Up"),
    ('n', 52, "Next match"),
    ('/', 53, "Search"),
    ('q', 54, "Close"),
    ('?', 55, "Help"),
];

/// The corpus as a map. Thirty-three bindings in five families.
pub fn map(on: On) -> KeyMap {
    let mut m = KeyMap::new();
    for (c, a, h) in CTRL {
        m = m.bind(&[Chord::key(c).ctrl()], a, h);
    }
    for (n, a, h) in FKEYS {
        m = m.bind(&[Chord::new(KeyCode::F(n))], a, h);
    }
    for (code, a, h) in NAMED {
        m = m.bind(&[Chord::new(code)], a, h);
    }
    for (code, a, h) in ALT {
        m = m.bind(&[Chord::new(code).alt()], a, h);
    }
    for (c, a, h) in LETTERS {
        let chord = match on {
            On::BaseLayout => Chord::key(c),
            On::Typed => Chord::typed(c),
        };
        m = m.bind(&[chord], a, h);
    }
    m
}

/// The corpus plus two sequences.
pub fn map_with_seqs(on: On) -> KeyMap {
    map(on)
        .bind_seq(&[Chord::key('g'), Chord::key('g')], TOP, "Top of file")
        .bind_seq(
            &[Chord::key('x').ctrl(), Chord::key('s').ctrl()],
            SAVE,
            "Save (emacs)",
        )
        .bind_seq(&[Chord::key('G')], BOTTOM, "Bottom of file")
}

/// A modal's four bindings, for the scoping gate.
pub fn modal_map() -> KeyMap {
    KeyMap::new()
        .bind(&[Chord::new(KeyCode::Enter)], 60, "Confirm")
        .bind(&[Chord::new(KeyCode::Escape)], 61, "Cancel")
        .bind(&[Chord::key('s').ctrl()], 62, "Save and close")
        .bind(&[Chord::new(KeyCode::Tab)], 63, "Next field")
}

/// A keyboard layout, as a table from where a key is to what it prints.
pub struct Layout {
    /// For the report.
    pub name: &'static str,
    /// Position to product. Empty means they agree.
    pub map: &'static [(char, char)],
}

impl Layout {
    /// The layout every test suite is written on, and the reason nine tickets noticed nothing.
    pub const US: Layout = Layout {
        name: "us",
        map: &[],
    };

    /// A Cyrillic layout: thirty-three positions that print something else.
    pub const RU: Layout = Layout {
        name: "ru",
        map: &[
            ('q', 'й'),
            ('w', 'ц'),
            ('e', 'у'),
            ('r', 'к'),
            ('t', 'е'),
            ('y', 'н'),
            ('u', 'г'),
            ('i', 'ш'),
            ('o', 'щ'),
            ('p', 'з'),
            ('a', 'ф'),
            ('s', 'ы'),
            ('d', 'в'),
            ('f', 'а'),
            ('g', 'п'),
            ('h', 'р'),
            ('j', 'о'),
            ('k', 'л'),
            ('l', 'д'),
            ('z', 'я'),
            ('x', 'ч'),
            ('c', 'с'),
            ('v', 'м'),
            ('b', 'и'),
            ('n', 'т'),
            ('m', 'ь'),
            (';', 'ж'),
            ('\'', 'э'),
            ('[', 'х'),
            (']', 'ъ'),
            (',', 'б'),
            ('.', 'ю'),
            ('/', '.'),
        ],
    };

    /// What the key at `base`'s position prints.
    pub fn prints(&self, base: char) -> char {
        self.map
            .iter()
            .find(|(pos, _)| *pos == base)
            .map_or(base, |(_, out)| *out)
    }
}

/// Which of the two indistinguishable legacy behaviours a terminal has.
///
/// **This is the axis that cannot be observed from inside the process**, and it is why
/// `base_layout_reported` is a boolean rather than a tier: both arms are silence on the wire.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LegacyCtrl {
    /// A control chord on a non-Latin keycap still arrives, spelled with the Latin letter.
    LatinFallback,
    /// It does not arrive at all.
    None,
}

/// How much of the keyboard protocol the terminal has. **A test-rig parameter and never a readable
/// capability** — ADR 0010 refuses a keyboard tier by name, and the middle rungs are not computable.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tier {
    /// Kitty flag 4: the physical layout is reported alongside what was typed.
    BaseLayout,
    /// Below it: `code` is inferred from `text`, and `text` is what the terminal chose to send.
    Legacy,
}

/// What the engine would deliver if this chord were pressed on this keyboard.
///
/// A model of what the engine's input ticket already settled, not a guess about terminals.
///
/// # Every key it makes has empty text, and that is a hard limit rather than a simplification
///
/// **`KeyText` has no public constructor.** Its fields are private and the engine ships only
/// `EMPTY`, `as_str` and `is_empty` — deliberately, because a component that could assemble one
/// could forge a key whose `code` and `text` disagree and make any binding fire. That gate is worth
/// more than this rig.
///
/// The consequence is written down rather than worked around: **this rig models `On::BaseLayout`
/// bindings only, and `On::Typed` cannot be exercised anywhere in this crate.** It costs the rig
/// nothing, because the counts are about `code` — below flag 4 the engine infers `code` *from* the
/// text, so a `BaseLayout` binding on `Char('j')` meets `Char('о')` and misses, which is the whole
/// loss being counted. What it does cost is that `Chord`'s `On::Typed` arm has no positive test in
/// this crate at all; only the negative one, that it never matches an empty text.
pub fn as_received(press: Chord, layout: &Layout, tier: Tier, legacy: LegacyCtrl) -> Option<Key> {
    let key = |code: KeyCode, mods: Mods, text: KeyText| {
        Some(Key {
            code,
            mods,
            kind: KeyKind::Press,
            text,
            at: Instant::now(),
        })
    };
    // A named key, a function key or anything with a non-shift modifier other than a bare letter
    // arrives as an escape sequence, which no layout touches.
    match press.code {
        KeyCode::Char(c) => {
            let printed = layout.prints(c);
            let has_ctrl_or_alt = press.mods.contains(Mods::CTRL) || press.mods.contains(Mods::ALT);
            if has_ctrl_or_alt {
                // A control chord produces no text at any tier, so below flag 4 there is nothing to
                // infer `code` from and the terminal's own fallback decides what arrives.
                //
                // **The fallback question only arises when the keycap is not already Latin.** On a US
                // layout the physical key prints the letter the binding names, so a control chord
                // arrives at every tier and there is nothing to fall back *from* — which is exactly
                // why nine tickets noticed none of this. The first version of this model suppressed
                // the chord on layout alone and made US lose nineteen bindings, which is a model that
                // proves too much.
                let latin = printed.is_ascii();
                return match tier {
                    Tier::BaseLayout => key(KeyCode::Char(c), press.mods, KeyText::EMPTY),
                    Tier::Legacy if latin => key(KeyCode::Char(c), press.mods, KeyText::EMPTY),
                    Tier::Legacy => match legacy {
                        LegacyCtrl::LatinFallback => {
                            key(KeyCode::Char(c), press.mods, KeyText::EMPTY)
                        }
                        LegacyCtrl::None => None,
                    },
                };
            }
            // A bare letter always produces text; what differs is whether `code` is the position or
            // is inferred from it. The text itself cannot be built here — see the doc comment — and
            // the counts do not need it.
            let code = match tier {
                Tier::BaseLayout => KeyCode::Char(c),
                Tier::Legacy => KeyCode::Char(printed),
            };
            key(code, press.mods, KeyText::EMPTY)
        }
        other => key(other, press.mods, KeyText::EMPTY),
    }
}

/// What a map reaches on a keyboard.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Reach {
    /// Bindings in the map.
    pub total: usize,
    /// Bindings whose first alternative fires.
    pub reachable: usize,
    /// Bindings that do not fire at all. **Silence.**
    pub lost: usize,
    /// Bindings that fire the *wrong* action. **Must be zero at every tier.**
    pub wrong: usize,
}

/// Press the first alternative of every binding — which is what a help bar would have shown — and
/// count what happens.
pub fn reach(map: &KeyMap, layout: &Layout, tier: Tier, legacy: LegacyCtrl) -> Reach {
    let mut r = Reach {
        total: map.len(),
        reachable: 0,
        lost: 0,
        wrong: 0,
    };
    for b in &map.bindings {
        let Some(&first) = b.keys().first() else {
            continue;
        };
        match as_received(first, layout, tier, legacy) {
            None => r.lost += 1,
            Some(k) => match map.match_first(&k) {
                Some(a) if a == b.action() => r.reachable += 1,
                Some(_) => r.wrong += 1,
                None => r.lost += 1,
            },
        }
    }
    r
}
