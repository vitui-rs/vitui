//! **`layout` allocates nothing, at every arity.**
//!
//! A count, which is what §14's rule wants: *a gate is a count, a ratio, an equality or a compile
//! outcome.* Zero is a count, it is the same number on every machine, and it is the one property of
//! this module that a stopwatch could not have told us.
//!
//! # Why this is its own binary
//!
//! [`vitui_alloc_probe::CountingAllocator`] is a **process-global** allocator and the count is a
//! process-global counter. `cargo test` runs a binary's tests on several threads, so an allocating
//! sibling lands in the number — which is why the workspace's one test command is
//! `cargo test --workspace -- --test-threads=1`, and why this lives beside the library rather than
//! inside it.
//!
//! # What is actually being asserted
//!
//! Not *layout is fast*. **That the solver has no scratch space**, which is a structural claim: the
//! `Max` fixpoint keeps its frozen bit in the caller's output buffer (`out[i].x`), and
//! largest-remainder distribution ranks each lane against the others instead of sorting them. Both
//! of those are choices that would have been easier the other way, and this is the gate that stops
//! either one being quietly undone — a `Vec` for the remainders would pass every other test in this
//! crate.

use vitui_alloc_probe::{CountingAllocator, steady};
use vitui_engine::Rect;
use vitui_runtime::layout::{
    Col, Constraint,
    Constraint::{Fixed, Max, Min, Percent, Ratio, Weight},
    Grid, Row, Stack, rect, solve, text,
};

#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

// The one realistic screen, shared with the two reports. See `src/screen.rs`.
#[allow(dead_code)]
#[path = "../src/screen.rs"]
mod screen;
use screen::screen_frame;

/// `split` at every arity from one to twelve, which is the widest any corpus in this crate
/// generates.
///
/// Twelve separate const generic instantiations, written out rather than looped, because the arity
/// *is* the thing being varied and a loop cannot vary a const generic.
#[test]
fn split_allocates_zero_at_every_arity() {
    let band = Rect::new(0, 0, 300, 80);
    steady(|| {
        let _ = Row::new().split(band, [Weight(1)]);
        let _ = Row::new().split(band, [Fixed(20), Weight(1)]);
        let _ = Row::new().split(band, [Fixed(20), Weight(1), Min(10)]);
        let _ = Row::new().split(band, [Fixed(20), Weight(1), Min(10), Max(30)]);
        let _ = Row::new().split(
            band,
            [Percent(25), Percent(25), Weight(1), Fixed(4), Max(9)],
        );
        let _ = Row::new().split(
            band,
            [
                Ratio(1, 6),
                Ratio(1, 6),
                Weight(1),
                Weight(2),
                Fixed(3),
                Min(2),
            ],
        );
        let _ = Row::new().split(band, [Weight(1); 7]);
        let _ = Row::new().split(band, [Weight(1); 8]);
        let _ = Row::new().split(band, [Weight(1); 9]);
        let _ = Row::new().split(band, [Weight(1); 10]);
        let _ = Row::new().split(band, [Weight(1); 11]);
        let _ = Row::new().split(band, [Weight(1); 12]);
    });
}

/// The dynamic form, which is the one that could plausibly have needed a buffer and does not.
#[test]
fn the_dynamic_split_allocates_zero() {
    let band = Rect::new(0, 0, 300, 80);
    let spec = [
        Fixed(20),
        Weight(1),
        Min(10),
        Max(30),
        Percent(10),
        Ratio(1, 8),
        Weight(3),
    ];
    let mut out = [Rect::default(); 7];
    steady(|| {
        let n = Row::new()
            .spacing(1)
            .margin(1)
            .split_into(band, &spec, &mut out);
        assert_eq!(n, 7);
        let (n, fit) = Col::new().measure_into(band, &spec, &mut out);
        assert_eq!(n, 7);
        assert_eq!(fit.gaps.margin, 0);
    });
}

/// **The `Max` fixpoint driven to its bound**, which is where a scratch buffer would have been most
/// tempting: every round has to know which lanes are frozen, and the frozen set changes as it goes.
#[test]
fn the_cap_fixpoint_allocates_zero_when_it_is_driven_to_its_bound() {
    // Twelve descending ceilings over a band far larger than their sum, so a lane freezes on every
    // round and the fixpoint runs as long as it can.
    let spec: [Constraint; 12] = [
        Max(1),
        Max(2),
        Max(3),
        Max(4),
        Max(5),
        Max(6),
        Max(7),
        Max(8),
        Max(9),
        Max(10),
        Max(11),
        Weight(1),
    ];
    let mut out = [Rect::default(); 12];
    let (_, fit) = solve(4000, &spec, &mut out);
    assert!(fit.rounds >= 2, "the fixpoint did not engage: {fit:?}");
    steady(|| {
        for _ in 0..1_000 {
            let (n, fit) = solve(4000, &spec, &mut out);
            assert_eq!(n, 12);
            assert!(fit.rounds <= 12);
        }
    });
}

/// The rect algebra, `Align`, `Stack` and `Grid`.
///
/// `Grid` is on this list deliberately: it holds two sixty-four-element buffers **on the stack**, and
/// a convenience that quietly heap-allocated would be the easiest way to lose this property without
/// touching the solver at all.
#[test]
fn the_rect_algebra_and_the_grid_allocate_zero() {
    let a = Rect::new(4, 4, 40, 20);
    let b = Rect::new(0, 0, 30, 30);
    steady(|| {
        let _ = rect::inset(a, 2);
        let _ = rect::shrink(a, 1, 2, 3, 4);
        let _ = rect::expand(a, 3);
        let _ = rect::intersect(a, b);
        let _ = rect::clamp_to(a, b);
        let _ = rect::split_at_h(a, 7);
        let _ = rect::split_at_v(a, 7);
        let _ = Stack::center(a, 10, 5);
        let grid = Grid::new(8, 6).spacing(1).margin(1);
        for x in 0..8 {
            for y in 0..6 {
                let _ = grid.cell(a, x, y, 2, 2);
            }
        }
    });
}

/// **The whole realistic screen**, which is the shape the report times: thirty-two splits and one
/// hundred and nineteen lanes.
///
/// The one that matters, because the others each exercise a mechanism and this exercises the
/// composition of them — and a layout pass that allocated once per split would still pass every test
/// above while allocating thirty-two times a frame.
#[test]
fn a_whole_screen_of_layout_allocates_zero() {
    steady(|| {
        for _ in 0..100 {
            let (splits, lanes) = screen_frame(300, 80);
            assert_eq!((splits, lanes), (32, 119));
        }
    });
}

/// **Text measurement allocates nothing either**, which is the less obvious half of it.
///
/// `wrap` hands back subslices of its input and holds two indices; `truncate` is a prefix of its
/// input; `wrap_height` is a `count` over the first. The tempting implementation of every one of
/// those builds a `Vec<String>` — and it would be *correct*, which is why this is a gate and not a
/// review note. Measuring twenty-four wrapped rows is 2.5x the cost of the whole screen's layout
/// already; doing it with an allocation a line would put it somewhere else entirely.
#[test]
fn text_measurement_allocates_zero() {
    let paragraph = "The quick brown fox jumps over the lazy dog, and 漢字 as well, \
                     with an e\u{301} and a \u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467} in it.";
    steady(|| {
        for w in 1..=80u16 {
            let mut columns = 0u32;
            for line in text::wrap(paragraph, w) {
                columns += u32::from(text::width(line));
            }
            assert!(columns > 0);
            assert_eq!(
                text::wrap_height(paragraph, w),
                text::wrap(paragraph, w).count()
            );
            let cut = text::truncate(paragraph, w);
            assert!(text::width(cut) <= w);
        }
    });
}

/// The realistic case: a column of twenty-four wrapped rows, which is what the report times.
#[test]
fn a_column_of_wrapped_rows_allocates_zero() {
    let rows: [&str; 4] = [
        "a short line",
        "a considerably longer line that will certainly need to be wrapped at any sensible width",
        "漢字漢字漢字漢字漢字漢字漢字漢字漢字漢字",
        "e\u{301}quipe \u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467} and a\u{2764}\u{FE0F}b",
    ];
    let column = || {
        let mut lines = 0usize;
        for _ in 0..6 {
            for row in rows {
                lines += text::wrap_height(row, 38);
            }
        }
        assert!(lines >= 24);
    };

    steady(column);
}

/// **A theme is heap-free, and a swap allocates nothing.**
///
/// The obvious `Vec<Style>` palette allocates on **exactly the frame a swap becomes visible** — which
/// is the one frame where an allocation is least affordable and most likely to be blamed on something
/// else. Two swaps and two full frames' worth of lookups, and the count is zero.
///
/// `Theme` owning its thirteen roles rather than borrowing `&'static` ones is what costs the +0.32 ns
/// a lookup, and it is what buys a theme that can come off a disk without `Box::leak`.
#[test]
fn a_theme_and_two_swaps_allocate_zero() {
    use vitui_engine::ColorDepth;
    use vitui_runtime::theme::{
        CATPPUCCIN_MOCHA, Density, Distinction, Glyph, GlyphSet, Role, Theme,
    };

    let mut light = CATPPUCCIN_MOCHA;
    light[0] = 0xff_ffff;
    light[5] = 0x00_0000;

    // Built outside the window: construction pairs sixteen colours and is not what is being measured.
    let dark = Theme::authored(&CATPPUCCIN_MOCHA, GlyphSet::Extended, Density::Cosy)
        .resolve(ColorDepth::TrueColor);
    let pale =
        Theme::authored(&light, GlyphSet::Ascii, Density::Compact).resolve(ColorDepth::Indexed256);

    // **One untimed pass first**, which is what `vitui-bench` does before it measures anything and
    // for the same reason: the first touch of a code path in a process pays costs that are not the
    // steady state — ten allocations, on the run that found this, and zero on every one after. A
    // window that includes first-touch is measuring the loader.
    let warm = |mut current: Theme| {
        for swap in 0..2 {
            for _ in 0..2 {
                let mut sink = 0usize;
                for _ in 0..48 {
                    for r in Role::ALL {
                        sink += usize::from(current.paint(r) == current.paint(Role::Body));
                    }
                }
                for g in Glyph::ALL {
                    sink += current.glyph(g).len();
                }
                for d in Distinction::ALL {
                    sink += usize::from(current.shows(d));
                }
                sink += usize::from(current.roles_differ_on_wire(Role::Face, Role::FaceHover));
                assert!(sink > 0);
            }
            current = if swap == 0 { pale } else { dark };
        }
    };
    warm(dark);

    steady(|| {
        let mut current = dark;
        for swap in 0..2 {
            // Two full frames of lookups: 48 role paints is what a dense screen asks for.
            for _ in 0..2 {
                let mut sink = 0usize;
                for _ in 0..48 {
                    for r in Role::ALL {
                        sink += usize::from(current.paint(r) == current.paint(Role::Body));
                    }
                }
                for g in Glyph::ALL {
                    sink += current.glyph(g).len();
                }
                for d in Distinction::ALL {
                    sink += usize::from(current.shows(d));
                }
                sink += usize::from(current.roles_differ_on_wire(Role::Face, Role::FaceHover));
                assert!(sink > 0);
            }
            // The swap itself: a move, and nothing else.
            current = if swap == 0 { pale } else { dark };
        }
    });
}

/// `resolve`, `with_glyphs` and `mix` allocate nothing either.
#[test]
fn resolve_with_glyphs_and_mix_allocate_zero() {
    use vitui_engine::ColorDepth;
    use vitui_runtime::theme::{GlyphSet, Role, Theme};

    let theme = Theme::default();
    steady(|| {
        for tier in [
            ColorDepth::TrueColor,
            ColorDepth::Indexed256,
            ColorDepth::Ansi16,
            ColorDepth::None,
        ] {
            let t = theme.resolve(tier).with_glyphs(GlyphSet::Unicode);
            for step in 0..=10u32 {
                let _ = t.mix(Role::Body, Role::Danger, step as f32 / 10.0);
            }
        }
    });
}

/// **A key map declares, matches and helps without allocating.**
///
/// Four windows, because they are four different claims: copying a map's routing half into the
/// frame's buffer, matching over it, stepping the sequence machine, and writing the whole corpus's
/// help into one reused `String`.
///
/// The buffer is the interesting one: it is **cleared and never freed**, so a frame that declares the
/// same maps every frame allocates on the first one and never again. A `Vec` that was dropped and
/// rebuilt would pass every other test in this file.
#[test]
fn a_key_map_declares_matches_and_helps_without_allocating() {
    use std::time::Instant;

    use vitui_engine::{Key, KeyCode, KeyKind, KeyText, Mods};
    use vitui_runtime::keys::{Chord, KeyMap, MatchMode, Matches, Pending, write_help};

    const SAVE: u32 = 1;
    let map = KeyMap::new()
        .bind(&[Chord::key('s').ctrl()], SAVE, "Save")
        .bind(&[Chord::key('q').ctrl(), Chord::key('c').ctrl()], 2, "Quit")
        .bind_seq(&[Chord::key('g'), Chord::key('g')], 3, "Top");

    let key = |code: KeyCode, mods: Mods| Key {
        code,
        mods,
        kind: KeyKind::Press,
        text: KeyText::EMPTY,
        at: Instant::now(),
    };

    let mut buf = Matches::new();
    let mut help = String::with_capacity(256);
    let mut pending = Pending::none();

    // The untimed pass, for the reason on the theme gate: a window that includes first-touch is
    // measuring the loader.
    buf.clear();
    buf.declare(&map);
    let _ = buf.match_first(&key(KeyCode::Char('s'), Mods::CTRL), MatchMode::Masked);
    let _ = map.step(
        &mut pending,
        &key(KeyCode::Char('g'), Mods::NONE),
        KeyMap::TIMEOUT,
    );
    map.abandon(&mut pending);
    for b in &map.bindings {
        help.clear();
        write_help(&mut help, b);
    }
    let capacity = buf.capacity();

    steady(|| {
        for _ in 0..100 {
            buf.clear();
            buf.declare(&map);
            assert_eq!(
                buf.match_first(
                    &key(KeyCode::Char('s'), Mods::CTRL.with(Mods::CAPS)),
                    MatchMode::Masked
                ),
                Some(SAVE)
            );
            // Sixty-four steps of the sequence machine, which holds one `Copy` value and no heap.
            for _ in 0..32 {
                let _ = map.step(
                    &mut pending,
                    &key(KeyCode::Char('g'), Mods::NONE),
                    KeyMap::TIMEOUT,
                );
                let _ = map.step(
                    &mut pending,
                    &key(KeyCode::Char('g'), Mods::NONE),
                    KeyMap::TIMEOUT,
                );
            }
            for b in &map.bindings {
                help.clear();
                write_help(&mut help, b);
                assert!(!help.is_empty());
            }
        }
    });
    assert_eq!(
        buf.capacity(),
        capacity,
        "the buffer was cleared, not freed"
    );
}

/// **A steady frame allocates nothing** — the five structures are swapped and cleared, not dropped
/// and rebuilt, and the scratch buffer keeps its capacity across frames.
///
/// The one that matters most on this backlog, because it is the property every later ticket inherits:
/// a frame that declares the same widgets every frame must not allocate for having done it before.
#[test]
fn a_steady_frame_allocates_nothing() {
    use vitui_engine::Rect;
    use vitui_runtime::ctx::{Driver, Id, Interest};
    use vitui_runtime::{Repaint, Role};

    let mut driver = Driver::headless(80, 24).expect("attaching to a sink cannot fail");

    // A realistic-ish frame: twenty-four rows, each with a formatted cell and an interactive region,
    // plus a restyle and a tab stop. Run once untimed, for the first-touch reason the other gates
    // give.
    let one_frame = |driver: &mut Driver| {
        driver.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            for row in 0..24i32 {
                let id = Id::from_raw(u64::try_from(row).unwrap_or(0));
                cx.label(
                    0,
                    row,
                    format_args!("row {row:>3}  {:>8.2}", f64::from(row) * 1.5),
                    body,
                );
                cx.interact(
                    id,
                    Rect::new(0, row, 80, 1),
                    Interest::CLICK.with(Interest::FOCUS),
                );
            }
            cx.restyle(
                Rect::new(0, 0, 80, 1),
                &Repaint {
                    fg: Some(Role::Title),
                    set: Repaint::BOLD,
                    ..Default::default()
                },
            );
        });
    };
    one_frame(&mut driver);
    one_frame(&mut driver);

    steady(|| {
        for _ in 0..50 {
            one_frame(&mut driver);
        }
    });
    assert_eq!(
        driver.inspect().hits().len(),
        24,
        "and it drew what it claimed"
    );
    assert_eq!(driver.inspect().ring().len(), 24);
}

/// The drawing verbs and the scratch buffer, in their own window.
///
/// **The scratch is worth twenty-four allocations a frame**, which is the reason it exists — not the
/// speed. A `format!` per row is one allocation per row; staging into a buffer the frame owns is none.
#[test]
fn formatting_a_frame_of_rows_allocates_nothing() {
    use vitui_runtime::Role;
    use vitui_runtime::ctx::Driver;

    let mut driver = Driver::headless(80, 24).expect("sink");
    let draw = |driver: &mut Driver| {
        driver.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            for row in 0..24i32 {
                let w = cx.stage(format_args!("{:>10}", row * 7));
                let _ = cx.blit(i32::from(80 - w), row, body);
            }
        });
    };
    draw(&mut driver);
    draw(&mut driver);
    steady(|| {
        for _ in 0..50 {
            draw(&mut driver);
        }
    });
}

/// **Routing a batch allocates nothing**, which is the half of *there are no per-id inboxes* that a
/// count can hold.
///
/// The per-id version costs at least one allocation a frame — a map entry, or a `Vec` per widget
/// that read a key — and this is what says the one queue was kept. Two allocations were hiding
/// behind it and neither showed up in the steady-frame gate above, because that gate posts no
/// input: the pointer batch was `std::mem::take`-n at award time and bought back on the next frame
/// that saw the pointer, and the key queue was drained with `Vec::remove(0)`.
#[test]
fn routing_a_batch_allocates_nothing() {
    use vitui_engine::{
        Button, Buttons, Key, KeyCode, KeyKind, KeyText, Mods, Mouse, MouseKind, Rect,
    };
    use vitui_runtime::ctx::{Driver, Id, Interest};

    let mut driver = Driver::headless(80, 24).expect("attaching to a sink cannot fail");
    let focused = Id::from_raw(1);
    driver.plant(None, Some(focused), None);

    let key = |c: char| Key {
        code: KeyCode::Char(c),
        mods: Mods::NONE,
        kind: KeyKind::Press,
        text: KeyText::EMPTY,
        at: std::time::Instant::now(),
    };
    let mouse = |kind| Mouse {
        x: 4,
        y: 2,
        kind,
        buttons: Buttons::NONE,
        mods: Mods::NONE,
        at: std::time::Instant::now(),
    };

    // A realistic burst: a dozen keys, a few moves and a click. The click is a routing edge, so this
    // is two frames' worth of queue and the drain loop is part of what is being measured.
    let one_burst = |driver: &mut Driver| {
        for c in "hello, world".chars() {
            driver.post_key(key(c));
        }
        for _ in 0..4 {
            driver.post_mouse(mouse(MouseKind::Move));
        }
        driver.post_mouse(mouse(MouseKind::Down(Button::Left)));
        driver.post_mouse(mouse(MouseKind::Up(Button::Left)));
        loop {
            driver.frame(|cx| {
                for row in 0..24i32 {
                    cx.interact(
                        Id::from_raw(u64::try_from(row).unwrap_or(0)),
                        Rect::new(0, row, 80, 1),
                        Interest::CLICK.with(Interest::FOCUS),
                    );
                }
                while cx.next_key(focused).is_some() {}
            });
            if driver.queued() == 0 {
                break;
            }
        }
    };

    one_burst(&mut driver);
    one_burst(&mut driver);
    steady(|| {
        for _ in 0..20 {
            one_burst(&mut driver);
        }
    });
}
