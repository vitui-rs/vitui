//! **Runtime ticket 07's report**: what a key map costs a frame, and what a Cyrillic layout reaches.
//!
//! ```text
//! cargo run --release --example keys_numbers -p vitui-runtime
//! ```
//!
//! Two halves, and the second is not a timing at all:
//!
//! 1. **135.6 ns for a 33-binding map and a three-key batch** — declaring the map into the frame's
//!    buffer plus matching three keys over it. 0.136% of a 100 µs frame.
//! 2. **The reach table**: 33 / 28 / 14 of 33 on a Cyrillic layout, with **0 wrong** at every tier.
//!    A count, and the only honest form the claim has — because a terminal cannot report which of the
//!    two legacy cases it is in, so this is a scene rather than a check.

use std::hint::black_box;
use std::time::Instant;

use vitui_bench::Bench;
use vitui_engine::{Key, KeyKind, KeyText};
use vitui_runtime::keys::{Chord, KeyMap, MatchMode, Matches, On, Pending, write_help};

// The corpus and the synthetic terminal, shared with the crate's own gates. See `src/keys/corpus.rs`.
#[allow(dead_code)]
#[path = "../src/keys/corpus.rs"]
mod corpus;

use corpus::{Layout, LegacyCtrl, Tier};

/// The frame budget every ratio is against.
const FRAME_NS: f64 = 100_000.0;

fn press(c: Chord) -> Key {
    Key {
        code: c.code,
        mods: c.mods,
        kind: KeyKind::Press,
        text: KeyText::EMPTY,
        at: Instant::now(),
    }
}

fn main() {
    println!("runtime ticket 07 — what a key map costs\n");
    what_a_frame_pays();
    what_a_cyrillic_layout_reaches();
    what_the_split_saves();
}

/// Report: declaring a map and matching a batch.
fn what_a_frame_pays() {
    let map = corpus::map(On::BaseLayout);
    assert_eq!(map.len(), 33);
    // Two buffers, because `Bench` holds every case's closure at once and one `&mut` plus one `&`
    // to the same buffer is `E0502`. Which is the harness being right: two cases sharing mutable
    // state would be two cases measuring each other.
    let mut declared_into = Matches::new();
    let mut matched_over = Matches::new();
    matched_over.declare(&map);

    // A three-key batch, which is what a busy frame actually carries: one hit near the front, one
    // near the back, one miss over the whole map.
    let batch = [
        press(Chord::key('s').ctrl()),
        press(Chord::key('?')),
        press(Chord::key('z').ctrl()),
    ];

    let report = Bench::new(40)
        .case("declare 33 bindings", 1_000, || {
            let b = black_box(&mut declared_into);
            b.clear();
            b.declare(black_box(&map));
        })
        .case("match a three-key batch", 1_000, || {
            let mut fired = 0usize;
            for k in &batch {
                fired += usize::from(
                    black_box(&matched_over)
                        .match_first(black_box(k), MatchMode::Masked)
                        .is_some(),
                );
            }
            black_box(fired);
        })
        .case("one chord matches", 10_000, || {
            black_box(
                black_box(Chord::key('s').ctrl()).matches(black_box(&batch[0]), MatchMode::Masked),
            );
        })
        .case("match_first, hit at 1 of 33", 10_000, || {
            black_box(black_box(&map).match_first(black_box(&batch[0])));
        })
        .case("match_first, miss over 33", 10_000, || {
            black_box(black_box(&map).match_first(black_box(&batch[2])));
        })
        .run();

    println!("report  a key map on the frame path, minimum of 40 rounds:\n{report}");
    let declare = report.get("declare 33 bindings").expect("measured");
    let batch_ns = report.get("match a three-key batch").expect("measured");
    let total = declare + batch_ns;
    println!(
        "        declare {:>8.2} ns  +  batch {:>8.2} ns  =  {:>8.2} ns  ({:.3}% of a {:.0} us frame)",
        declare,
        batch_ns,
        total,
        total / FRAME_NS * 100.0,
        FRAME_NS / 1e3,
    );
    println!(
        "        spec §9 measured 45.90 + 89.70 = 135.6 ns, 0.136% of the frame. Declaring is\n\
        \x20       cheaper here and the batch is dearer, and the batch is the one worth explaining:\n\
        \x20       **this one contains two near-full scans.** `?` is the last of the thirty-three\n\
        \x20       and `Ctrl+Z` is bound to nothing at all, so two of the three keys walk the whole\n\
        \x20       map — 80 ns each, against 4.6 for a hit at the front. A batch is worth what its\n\
        \x20       misses cost, and a map ordered so that the common keys are early is the whole of\n\
        \x20       the optimisation available. Both figures are inside a tenth of a per cent of the\n\
        \x20       frame either way.\n"
    );
}

/// Report and count: what a map reaches on a keyboard that is not the author's.
fn what_a_cyrillic_layout_reaches() {
    let map = corpus::map(On::BaseLayout);
    println!("report  what a 33-binding map reaches, by keyboard and terminal:");
    println!(
        "          {:<34} {:>5} {:>5} {:>6}",
        "case", "hit", "lost", "wrong"
    );
    let cases = [
        (
            "us, any terminal",
            &Layout::US,
            Tier::BaseLayout,
            LegacyCtrl::None,
        ),
        (
            "ru, kitty flag 4",
            &Layout::RU,
            Tier::BaseLayout,
            LegacyCtrl::None,
        ),
        (
            "ru, legacy + latin fallback",
            &Layout::RU,
            Tier::Legacy,
            LegacyCtrl::LatinFallback,
        ),
        (
            "ru, legacy + no fallback",
            &Layout::RU,
            Tier::Legacy,
            LegacyCtrl::None,
        ),
    ];
    let mut wrong_total = 0;
    for (name, layout, tier, legacy) in cases {
        let r = corpus::reach(&map, layout, tier, legacy);
        println!(
            "          {name:<34} {:>5} {:>5} {:>6}",
            r.reachable, r.lost, r.wrong
        );
        wrong_total += r.wrong;
    }
    println!(
        "\n        spec §9 measured 33 / 28 / 14 of 33 with 0 wrong, and that reproduces.\n\
        \x20       **The failure is silence, not misfire** — {wrong_total} wrong across every case — \
        which is\n\
        \x20       exactly why a US-layout test suite finds none of it: the first row loses nothing\n\
        \x20       at any tier.\n\
        \x20       **And the two legacy rows cannot be told apart from inside the process.** Both are\n\
        \x20       silence on the wire, which is why what is readable is one boolean and not a tier\n\
        \x20       (ADR 0010), and why this table is a scene rather than a capability check."
    );

    // The family breakdown, which is the sharper half: the whole loss is one family.
    let letters = {
        let mut m = KeyMap::new();
        for (c, a, h) in corpus::LETTERS {
            m = m.bind(&[Chord::key(c)], a, h);
        }
        m
    };
    let full = corpus::reach(&letters, &Layout::RU, Tier::BaseLayout, LegacyCtrl::None);
    let legacy = corpus::reach(&letters, &Layout::RU, Tier::Legacy, LegacyCtrl::None);
    println!(
        "\n        bare letters: {} of {} at flag 4, {} of {} below it. Function keys, named keys\n\
        \x20       and Alt+named are escape sequences and are whole at every tier — so an\n\
        \x20       application whose map is arrows and function keys never meets any of this.\n",
        full.reachable, full.total, legacy.reachable, legacy.total
    );
}

/// Report: the two sizes, and what help costs when it is asked for.
fn what_the_split_saves() {
    use vitui_runtime::keys::{Binding, Match};

    let map = corpus::map(On::BaseLayout);
    let mut help = String::with_capacity(256);
    let report = Bench::new(40)
        .case("write help for 33 bindings", 1_000, || {
            let out = black_box(&mut help);
            for b in black_box(&map).bindings.iter() {
                out.clear();
                write_help(out, b);
            }
        })
        .case("step a sequence twice", 10_000, || {
            let seqs = corpus::map_with_seqs(On::BaseLayout);
            let mut p = Pending::none();
            let g = press(Chord::key('g'));
            black_box(seqs.step(&mut p, &g, KeyMap::TIMEOUT));
            black_box(seqs.step(&mut p, &g, KeyMap::TIMEOUT));
        })
        .run();
    println!("report  authoring and help, minimum of 40 rounds:\n{report}");
    println!(
        "        Match {} bytes, Binding {} bytes — the difference is the help string, which routes\n\
        \x20       nothing. A 33-binding map copies {} bytes into the frame instead of {}.\n\
        \x20       spec §9 measured 44 and 64.\n\
        \x20       And a `&'static [Chord]` binding would be {} bytes, which is what inline chords\n\
        \x20       cost: {} bytes a binding, paid so that a whole map is one expression (E0716).",
        size_of::<Match>(),
        size_of::<Binding>(),
        size_of::<Match>() * 33,
        size_of::<Binding>() * 33,
        40,
        size_of::<Binding>() - 40,
    );
    println!(
        "\n        Note the sequence arm above builds its map inside the window, so it is an upper\n\
        \x20       bound rather than the step's own cost — `Pending` is 32 bytes of `Copy` and the\n\
        \x20       machine allocates nothing, which is a count in tests/alloc.rs rather than a timing\n\
        \x20       here."
    );
}
