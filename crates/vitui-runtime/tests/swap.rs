//! **The swap frame: what the runtime can promise about it, and what it deliberately cannot.**
//!
//! Runtime ticket 05, spec §15 *live switching*. Four properties, all of them counts or equalities:
//!
//! 1. `Ctx::theme_changed()` is true for **exactly one frame**.
//! 2. Consulted, the stale count is **0** — an application that repaints on the flag leaves nothing
//!    carrying the previous palette.
//! 3. Ignored, it is **not** 0, and the runtime cannot fix that for anybody.
//! 4. The pair-key rule holds **in both directions**, over a corpus of memos rather than an example.
//!
//! # Why a separate binary, and why the stale count is the application's own
//!
//! **Nothing can force the repaint.** Damage is marked at write time and cleared by `present`, and
//! neither is reachable from outside the engine — deliberately, because marking the rest of the
//! screen damaged would re-send bytes that are still correct. So the runtime does what ADR 0007 has
//! the engine do one layer down and **tells the truth** instead of acting: `theme_changed()` is a
//! fact, and what to do with it is the application's.
//!
//! That shapes how the count below is taken. `Surface::damaged_cells` is `pub(crate)` in the engine,
//! so a stale count cannot be read out of the frame; what a caller *can* count is what it painted and
//! with which theme, which is what [`Painted`] does. The number is therefore the application's own
//! ledger of its own writes, and it is stated that way rather than implied to be the engine's damage.
//! The engine's one observable is `Presented::submitted`, and it is asserted here beside it.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use vitui_engine::ColorDepth;
use vitui_runtime::data::{Memo, Revision, Versioned};
use vitui_runtime::theme::{Density, GlyphSet, Themes};
use vitui_runtime::{Driver, Role, Theme};

// The one realistic screen, shared with the reports and the other gates. See `src/screen.rs`.
#[allow(dead_code)]
#[path = "../src/screen.rs"]
mod screen;

/// The dense screen the map's numbers are taken against.
const W: u16 = 300;
/// See [`W`].
const H: u16 = 80;

/// An application's own ledger of what it painted and with which theme.
///
/// One entry per cell, holding the revision of the theme whose paints were written there. **A cell
/// nothing has painted is not stale** — it holds no palette at all — which is why the value is an
/// `Option` rather than a sentinel revision.
struct Painted {
    at: Vec<Option<Revision>>,
}

impl Painted {
    fn new() -> Painted {
        Painted {
            at: vec![None; usize::from(W) * usize::from(H)],
        }
    }

    fn mark(&mut self, x: u16, y: u16, rev: Revision) {
        self.at[usize::from(y) * usize::from(W) + usize::from(x)] = Some(rev);
    }

    /// How many painted cells carry a theme other than `now`.
    fn stale(&self, now: Revision) -> usize {
        self.at
            .iter()
            .filter(|slot| matches!(slot, Some(rev) if *rev != now))
            .count()
    }

    fn painted(&self) -> usize {
        self.at.iter().filter(|slot| slot.is_some()).count()
    }
}

/// Paint every row of the screen, and record what was painted with what.
fn paint_all(cx: &mut vitui_runtime::Ctx<'_, '_>, ledger: &mut Painted) {
    let rev = cx.theme().revision();
    let body = cx.theme().paint(Role::Body);
    for y in 0..H {
        cx.fill(vitui_engine::Rect::new(0, i32::from(y), W, 1), "x", body);
        for x in 0..W {
            ledger.mark(x, y, rev);
        }
    }
}

/// **Gate: `theme_changed()` is true for exactly one frame, and the stale count is 0 when consulted.**
#[test]
fn a_consulted_swap_leaves_nothing_carrying_the_previous_palette() {
    let mut themes = Themes::standard();
    themes.set_tier(ColorDepth::TrueColor);
    let mut driver = Driver::headless(W, H).expect("a headless attach cannot fail");
    driver.set_theme(*themes.theme());

    let mut ledger = Painted::new();
    let mut flags = Vec::new();

    // Frame 1: the first draw. `set_theme` was called before it, so the flag is true here too — the
    // theme did change for this frame, and an application that repaints on it repaints once.
    driver.frame(|cx| {
        flags.push(cx.theme_changed());
        paint_all(cx, &mut ledger);
    });
    assert_eq!(ledger.painted(), usize::from(W) * usize::from(H));

    // Frame 2: nothing changed.
    driver.frame(|cx| flags.push(cx.theme_changed()));

    // The swap, written between frames like any other application state.
    assert!(themes.select_slug("solarized-light"));
    driver.set_theme(*themes.theme());
    let now = themes.theme().revision();

    // Frame 3: the swap frame. The application consults the flag and repaints.
    driver.frame(|cx| {
        let changed = cx.theme_changed();
        flags.push(changed);
        if changed {
            paint_all(cx, &mut ledger);
        }
    });
    assert_eq!(
        ledger.stale(now),
        0,
        "an application that consulted the flag left cells carrying the previous palette"
    );

    // Frame 4: exactly one frame later, and the flag is down again.
    driver.frame(|cx| flags.push(cx.theme_changed()));

    assert_eq!(
        flags,
        vec![true, false, true, false],
        "the flag is true for exactly the frame after each swap and no other"
    );
}

/// **The other half, and it is the one that has to be a count rather than a promise.**
///
/// A partial-drawing application ignores the flag, repaints twenty cells because that is what its own
/// data said had changed, and leaves every other painted cell carrying the previous palette. The
/// runtime cannot fix this and must not try: marking the rest damaged would re-send bytes that are
/// still correct for cells the application has not redrawn.
#[test]
fn an_ignored_swap_leaves_the_rest_of_the_screen_stale_and_the_runtime_may_not_fix_it() {
    let mut themes = Themes::standard();
    themes.set_tier(ColorDepth::TrueColor);
    let mut driver = Driver::headless(W, H).expect("a headless attach cannot fail");
    driver.set_theme(*themes.theme());

    let mut ledger = Painted::new();
    driver.frame(|cx| paint_all(cx, &mut ledger));
    let painted = ledger.painted();

    assert!(themes.select_slug("nord"));
    driver.set_theme(*themes.theme());
    let now = themes.theme().revision();

    // Twenty cells, because twenty is what this application's own data said had moved.
    driver.frame(|cx| {
        let rev = cx.theme().revision();
        let paint = cx.theme().paint(Role::Focus);
        for x in 0..20u16 {
            let _ = cx.text(i32::from(x), 0, "y", paint);
            ledger.mark(x, 0, rev);
        }
    });

    assert_eq!(
        ledger.stale(now),
        painted - 20,
        "the runtime repainted something it was not asked to"
    );
}

/// **`Presented::submitted` cannot see a swap, and finding that out is worth more than the gate it
/// replaced.**
///
/// The obvious shape for *a swap frame is a cliff* is an equality on `submitted`: a steady redraw
/// submits nothing, a swap frame submits. It does not hold, and the reason is a decision the engine
/// took and wrote down. **There is no write-time equality filter on a surface** — it was measured and
/// refused, because `clear`-then-draw defeats it: an immediate-mode component blanks a region before
/// drawing into it, so the filter sees blank-over-text and then text-over-blank and both are changes.
/// Damage is therefore marked by *every* write, and `submitted` is true for every frame that drew.
///
/// The equality that survives is the narrow one below. The place a swap actually becomes visible is
/// one layer further down — the serialiser's mirror, where the two writes have already collapsed into
/// the one cell the frame ends up holding — and that is **bytes on the wire**, not a flag. It is a
/// timing and a byte count, so it is [`the report`](../examples/theme_set_numbers.rs) rather than a
/// gate.
#[test]
fn a_frame_that_draws_nothing_submits_nothing_and_every_frame_that_draws_submits() {
    let mut themes = Themes::standard();
    themes.set_tier(ColorDepth::TrueColor);
    let mut driver = Driver::headless(W, H).expect("a headless attach cannot fail");
    driver.set_theme(*themes.theme());

    let mut ledger = Painted::new();
    assert!(
        driver.frame(|cx| paint_all(cx, &mut ledger)).submitted,
        "the first draw is a change from a blank screen"
    );
    assert!(
        driver.frame(|cx| paint_all(cx, &mut ledger)).submitted,
        "an identical redraw still marks damage: there is no write-time equality filter"
    );
    assert!(
        !driver.frame(|_| {}).submitted,
        "a frame that wrote nothing has nothing to submit"
    );

    assert!(themes.select_slug("gruvbox-light-medium"));
    driver.set_theme(*themes.theme());
    assert!(driver.frame(|cx| paint_all(cx, &mut ledger)).submitted);
}

/// A sink the gate can read back, so a frame can be priced in bytes.
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

/// **The cliff, as an equality and a relation: a steady frame is zero bytes and a swap frame is a
/// screen.**
///
/// This is what the flag above could not say. Damage is marked by every write, so `submitted` cannot
/// tell a steady redraw from a swap; the serialiser's mirror can, because by then the frame's writes
/// have collapsed into the one cell each of them ends up holding. A steady frame therefore re-sends
/// **nothing at all** — an equality, on every machine — and a swap frame re-sends the screen.
///
/// **A swap frame needs no permission to exceed the frame budget**: it is a steady frame plus a full
/// repaint, and a full repaint is what a swap *is*. The microseconds are in the report, because a
/// timing is a report; the bytes are here, because zero is a count.
#[test]
fn a_steady_frame_is_zero_bytes_on_the_wire_and_a_swap_frame_is_a_screen() {
    let buf = Arc::new(Mutex::new(Vec::new()));
    let mut themes = Themes::standard();
    themes.set_tier(ColorDepth::TrueColor);
    let mut driver = Driver::attach(
        vitui_engine::Config {
            clock: vitui_engine::Clock::Manual,
            output: vitui_engine::Output::Sink(Box::new(Shared(Arc::clone(&buf)))),
            size: (W, H),
            overrides: vitui_engine::Overrides {
                colors: Some(ColorDepth::TrueColor),
                ..Default::default()
            },
            ..Default::default()
        },
        *themes.theme(),
    )
    .expect("attaching to a sink cannot fail");

    let mut ledger = Painted::new();
    let bytes = |driver: &mut Driver, ledger: &mut Painted| {
        buf.lock().expect("no panic holds this lock").clear();
        driver.frame(|cx| paint_all(cx, ledger));
        buf.lock().expect("no panic holds this lock").len()
    };

    let first = bytes(&mut driver, &mut ledger);
    assert!(first > 0, "the first draw is a change from a blank screen");
    assert_eq!(
        bytes(&mut driver, &mut ledger),
        0,
        "an identical redraw re-sends nothing: the mirror filter is the whole of it"
    );

    assert!(themes.select_slug("solarized-light"));
    driver.set_theme(*themes.theme());
    let swap = bytes(&mut driver, &mut ledger);
    assert!(
        swap > first / 2,
        "a swap frame re-sends the screen: {swap} bytes against a first draw's {first}"
    );
    assert_eq!(
        bytes(&mut driver, &mut ledger),
        0,
        "and the frame after a swap is a steady frame again"
    );
}

/// One case in the memo corpus: what it computes, and whether it says the theme is in its key.
struct Case {
    what: &'static str,
    /// Whether the value this memo computes is made of paints.
    declared: bool,
    /// The key it actually uses, given the data's revision and the theme.
    key: fn(Revision, &Theme) -> Revision,
    /// The value, as bytes, so two themes' outputs can be compared without naming a type.
    value: fn(&Theme) -> Vec<u8>,
}

/// The corpus the detector runs over. Four cases and **two of them are wrong on purpose**, because a
/// detector that only ever sees correct code is a detector nobody has run.
const CORPUS: [Case; 4] = [
    Case {
        what: "a row's paints, keyed on the pair",
        declared: true,
        key: |data, theme| theme.memo_key(data),
        value: |theme| format!("{:?}", theme.paint(Role::Body)).into_bytes(),
    },
    Case {
        what: "a row's paints, keyed on the data alone",
        declared: false,
        key: |data, _| data,
        value: |theme| format!("{:?}", theme.paint(Role::Body)).into_bytes(),
    },
    Case {
        what: "a column width, keyed on the data alone",
        declared: false,
        key: |data, _| data,
        value: |_| b"widest cell in the column".to_vec(),
    },
    Case {
        what: "a column width, keyed on the pair",
        declared: true,
        key: |data, theme| theme.memo_key(data),
        value: |_| b"widest cell in the column".to_vec(),
    },
];

/// **Gate, a count in both directions: a memo's value moves with the theme ⟺ the theme is in its key.**
///
/// Both halves are measured rather than declared. *Its value moves with the theme* is decided by
/// computing it under two themes and comparing the bytes; *the theme is in its key* is decided by
/// taking the key under the same two themes and comparing. Nothing here reads the `declared` field to
/// decide either one — that field is only what the case **claims**, and the last assertion is what
/// makes the corpus itself honest.
///
/// > **Read as *every memo*, the pair key is itself a defect.** It costs a full recomputation on
/// > every swap for values that are bit-identical afterwards — the map priced two and four such memos
/// > on the dense screen at 221 µs and 399 µs, which is two and four whole frame budgets. So the
/// > detector counts the false positives as well as the false negatives, and both counts are zero
/// > only for a corpus in which every case is right.
#[test]
fn a_memos_value_moves_with_the_theme_exactly_when_the_theme_is_in_its_key() {
    let data = Versioned::new(vec![1u32, 2, 3]);
    let dark = Theme::default().resolve(ColorDepth::TrueColor);
    let light = Themes::standard()
        .schemes()
        .iter()
        .find(|s| !s.is_dark())
        .expect("the shipped set has a light scheme")
        .theme(GlyphSet::default(), Density::default())
        .resolve(ColorDepth::TrueColor);

    let mut verdicts = HashMap::new();
    let (mut stale, mut wasted) = (0usize, 0usize);
    for case in &CORPUS {
        let moves = (case.value)(&dark) != (case.value)(&light);
        let keyed = (case.key)(data.revision(), &dark) != (case.key)(data.revision(), &light);
        verdicts.insert(case.what, (moves, keyed));
        match (moves, keyed) {
            // Its value is made of paints and its key does not say so: stale after a swap, for ever,
            // because the data will not move on a settings pane.
            (true, false) => stale += 1,
            // Its value is not made of paints and its key says it is: a full recomputation on every
            // swap for a value that is bit-identical afterwards.
            (false, true) => wasted += 1,
            _ => {}
        }
        // The corpus's own claim, checked against what was measured, so a case cannot rot into
        // agreeing with itself.
        assert_eq!(
            case.declared, keyed,
            "{}: the case claims one thing about its key and does another",
            case.what
        );
    }

    assert_eq!(
        (stale, wasted),
        (1, 1),
        "the corpus is meant to contain exactly one of each defect: {verdicts:?}"
    );

    // And the two correct cases, which is the direction a gate over real code would assert.
    assert_eq!(verdicts["a row's paints, keyed on the pair"], (true, true));
    assert_eq!(
        verdicts["a column width, keyed on the data alone"],
        (false, false)
    );
}

/// A pair-keyed memo misses **exactly once** per swap, and a data-keyed one never misses at all —
/// which is the same fact from the memo's side rather than the key's.
#[test]
fn a_pair_keyed_memo_misses_once_per_swap_and_a_data_keyed_one_never_does() {
    let data = Versioned::new(vec![1u32, 2, 3]);
    let mut themes = Themes::standard();
    themes.set_tier(ColorDepth::TrueColor);

    let mut paired: Memo<u8> = Memo::new();
    let mut data_only: Memo<u8> = Memo::new();

    for slug in ["dracula", "nord", "solarized-light", "kanagawa"] {
        assert!(themes.select_slug(slug));
        let theme = themes.theme();
        let _ = paired.get(theme.memo_key(data.revision()), || 0);
        let _ = data_only.get(data.revision(), || 0);
    }

    assert_eq!(paired.recomputes, 4, "one miss per swap and no more");
    assert_eq!(data_only.recomputes, 1, "the data never moved");
}

/// *Memoise nothing* means nothing, including nothing about the theme: an unknown data revision
/// stays unknown after the fold, so a memo that was told not to cache is not accidentally given a
/// cache by the theme.
#[test]
fn an_unknown_revision_survives_the_pair_key() {
    let theme = Theme::default();
    assert_eq!(theme.memo_key(Revision::UNKNOWN), Revision::UNKNOWN);
    assert!(!theme.memo_key(Revision::UNKNOWN).is_known());
    assert!(theme.memo_key(Revision::fresh()).is_known());
}

/// **Gate, two counts of zero: the pair key is injective in each argument with the other held
/// fixed.**
///
/// That is the whole property a memo needs, and it is stronger than *it hashes*: a swap always moves
/// the key, and an edit always moves the key. It is what lets the fold be a rotate, a multiply and an
/// xor instead of sixteen rounds of FNV — the map's budget for this is **+0.312 ns**, and a per-byte
/// hash does not fit in it.
///
/// It is deliberately *not* a claim that two different pairs never collide. No sixty-four-bit fold of
/// a hundred and twenty-eight bits can promise that, FNV included, and pretending otherwise is how a
/// footnote becomes a defect.
#[test]
fn the_pair_key_is_injective_in_each_argument() {
    let themes = Themes::standard();
    let one = *themes.theme();

    let mut keys: Vec<u64> = (1..2_000u64)
        .map(|d| one.memo_key(Revision::from_raw(d)).raw())
        .collect();
    keys.sort_unstable();
    let before = keys.len();
    keys.dedup();
    assert_eq!(before, keys.len(), "one theme, two thousand data revisions");

    // Two thousand *real* themes rather than two thousand invented revisions, because a revision
    // only ever arrives from the process-global counter and a test that made one up would be
    // checking a number the mechanism cannot produce.
    let data = Revision::from_raw(0x1234_5678_9abc_def0);
    let mut keys: Vec<u64> = (0..2_000)
        .map(|_| Theme::default().memo_key(data).raw())
        .collect();
    keys.sort_unstable();
    let before = keys.len();
    keys.dedup();
    assert_eq!(before, keys.len(), "one datum, two thousand themes");
}
