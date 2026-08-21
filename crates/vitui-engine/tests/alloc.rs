//! The zero-allocation gate.
//!
//! The map's budget requires **zero allocations during frame composition**, and prior art shows
//! this is exactly the kind of guarantee that decays silently: ratatui shipped a per-frame `Vec`
//! allocation that was only noticed when it fragmented the heap on embedded devices.
//!
//! # The attribution window, which is a general rule and not a footnote
//!
//! **The probe counts `alloc` calls, not `alloc` calls by the app thread.** The counter is a
//! process-global atomic inside a global allocator, and a global allocator has no idea who is
//! calling it. So a window opened around a frame while a worker is running attributes the worker's
//! growth to the frame, and the gate goes red for something the engine did not do — or, worse, a
//! future version of it stays green because someone widened the window until it did.
//!
//! Spec §14 states the rule that follows, and it applies to every allocation gate this repository
//! ever adds, not only to the three below: **an allocation gate with a background job in it runs on
//! a deterministic spawner, or joins before it measures.** There is no third option, and "it
//! probably finished by then" is not one of them.
//!
//! Nothing here has a background job, and that is not luck: the deterministic single-thread mode is
//! what makes it true. `present` composites, packs, serialises and writes inline on the calling
//! thread, so the window below contains exactly one thread's work. Ticket 18 brings the render
//! thread, and it is the first ticket that has to obey the rule rather than satisfy it by
//! construction — its gate is register entry #5, pinned red in `src/register.rs` until then.
//!
//! # Why this is one test and not three
//!
//! The probe's counter is process-global, so a test asserting on it must not run beside another
//! test that allocates. CI runs the whole suite under `--test-threads=1`, which is enough there —
//! but a plain `cargo test` runs a binary's tests on several threads, and three tests in this file
//! observed each other's allocations and failed. A gate that is red on the repository's own
//! documented command is not a gate, so the three phases live in one test. Separate binaries are
//! separate processes and do not interfere.

use std::io::{Result, Write};

use vitui_alloc_probe::{CountingAllocator, assert_no_alloc};
use vitui_engine::{Color, Config, Engine, LayerId, Mix, Output, Rect, Restyle, Screen, Style};

#[global_allocator]
static ALLOC: CountingAllocator = CountingAllocator::new();

/// A sink that takes everything and keeps none of it. A recording sink would grow a `Vec` and the
/// gate would be measuring the test rather than the engine.
struct Discard;

impl Write for Discard {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

const W: u16 = 300;
const H: u16 = 80;

fn screen() -> (Screen, LayerId) {
    let (mut screen, _wake) = Engine::new(Config {
        size: (W, H),
        output: Output::Sink(Box::new(Discard)),
        ..Default::default()
    })
    .attach()
    .expect("attaching to a sink cannot fail");
    let id = screen.layers().add_content(0, Rect::new(0, 0, W, H), true);
    (screen, id)
}

fn full_screen(screen: &mut Screen, id: LayerId, row: &str, style: Style) {
    let mut view = screen.layers().view(id).expect("the layer was just added");
    for y in 0..H as i32 {
        view.text(0, y, row, style);
    }
}

#[test]
fn the_steady_state_allocates_nothing() {
    a_thousand_compose_cycles_allocate_nothing();
    a_full_screen_frame_allocates_nothing();
    an_idle_frame_allocates_nothing();
    a_frame_of_clusters_allocates_nothing();
    a_settled_restyle_over_a_hyperlinked_screen_allocates_nothing();
    a_settled_operator_over_a_hyperlinked_screen_allocates_nothing();
}

/// Ticket 12 puts the **first** intern on the frame path, and this is what bounds it.
///
/// Every other allocation site in this crate is a drawing verb or a topology change, where §3
/// permits one. `composite_run` → `recolour` → `Mixer::style` → `restyle::apply` reaches the
/// extended-style table *while a frame is being composited*, and CLAUDE.md's budget is **zero
/// allocations during frame composition**. So the property has to be that a **settled** operator
/// asks for nothing new: the table deduplicates, so once the entries its result needs exist, every
/// later frame finds them.
///
/// This is not register entry #7, which is about a *fading* operator and is red against impl 08 with
/// the sweep. It is the half impl 12 can be held to, and it is the same shape as
/// `a_settled_restyle_over_a_hyperlinked_screen_allocates_nothing` one layer up.
///
/// Hyperlinked, because that is what makes the mix's result need a table entry at all: an inline
/// cell mixes to an inline word and reaches no table, so a screen of those would pass whatever
/// `recolour` did.
fn a_settled_operator_over_a_hyperlinked_screen_allocates_nothing() {
    let (mut screen, id) = screen();
    let row: String = std::iter::repeat_n('m', W as usize).collect();
    // Explicit colours: a cell with a default background is left unmixed on a terminal silent on
    // OSC 11 (spec §5), and a sink is silent — so a default-coloured screen would measure an
    // operator that never touched a cell.
    full_screen(
        &mut screen,
        id,
        &row,
        Style::new().fg(Color::indexed(15)).bg(Color::indexed(8)),
    );
    let link = screen.link("https://example.com/vitui");
    {
        let mut v = screen.layers().view(id).expect("the layer was just added");
        v.restyle(
            Rect::new(0, 0, W, H),
            &Restyle {
                link: Some(link),
                ..Default::default()
            },
        );
    }
    screen
        .layers()
        .add_operator(1, Rect::new(0, 0, W, H), Mix::darken(Mix::FULL / 2));

    // Two frames of warm-up: the first mints the one entry the mix needs and grows the frame's own
    // buffers, the second proves the steady state has started.
    for _ in 0..2 {
        full_screen(
            &mut screen,
            id,
            &row,
            Style::new().fg(Color::indexed(15)).bg(Color::indexed(8)),
        );
        screen.present();
    }

    assert_no_alloc(|| {
        for _ in 0..100 {
            full_screen(
                &mut screen,
                id,
                &row,
                Style::new().fg(Color::indexed(15)).bg(Color::indexed(8)),
            );
            screen.present();
        }
    });
}

/// The memo's property, as an allocation count rather than as a stopwatch.
///
/// `restyle` reaches the extended-style table once per **distinct** style word, and the table is
/// deduplicated — so once a descriptor has been applied, applying it again finds every entry it
/// needs already there and mints nothing. That is what makes a settled operator converge after one
/// frame, and it is why a permanently *changing* one is the case eviction exists for.
///
/// This is not register entry #7, which is about the operator layer and is red against impl 08. It
/// is the half of that property `restyle` can be held to today.
///
/// It stops short of `present` on purpose: the serializer does not emit SGR 58/59 or OSC 8 yet
/// (impl 13), and `Style`'s colour accessors carry a `debug_assert` that says so rather than
/// reading a handle as two colours.
fn a_settled_restyle_over_a_hyperlinked_screen_allocates_nothing() {
    let (mut screen, id) = screen();
    let row: String = std::iter::repeat_n('m', W as usize).collect();
    full_screen(&mut screen, id, &row, Style::new());
    let link = screen.link("https://example.com/vitui");
    let all = Rect::new(0, 0, W, H);
    let hyperlink = Restyle {
        link: Some(link),
        ..Default::default()
    };
    let shadow = Restyle {
        bg: Some(Color::indexed(0)),
        ..Default::default()
    };

    {
        // Warm-up: the whole screen becomes extended, and then settles on the one style the loop
        // below asks for over and over. Both entries exist by the end of this block.
        let mut v = screen.layers().view(id).expect("the layer was just added");
        v.restyle(all, &hyperlink);
        v.restyle(all, &shadow);
    }

    assert_no_alloc(|| {
        let mut v = screen.layers().view(id).expect("the layer is still there");
        for _ in 0..1_000 {
            v.restyle(all, &shadow);
        }
    });
}

fn a_thousand_compose_cycles_allocate_nothing() {
    let (mut screen, id) = screen();
    let row: String = std::iter::repeat_n('x', W as usize).collect();

    // Two full-screen frames first, so every buffer the steady state uses has reached the capacity
    // it will keep. What is being gated is the steady state, not the first frame — a first frame
    // that allocates its buffers is the design, not a defect.
    for _ in 0..2 {
        full_screen(&mut screen, id, &row, Style::new());
        screen.present();
    }

    assert_no_alloc(|| {
        for i in 0..1_000u32 {
            let mut view = screen.layers().view(id).expect("the layer is still there");
            view.text(0, (i % H as u32) as i32, &row, Style::new());
            let presented = screen.present();
            assert!(presented.submitted, "frame {i} had nothing to say");
        }
    });
}

fn a_full_screen_frame_allocates_nothing() {
    let (mut screen, id) = screen();
    let row: String = std::iter::repeat_n('W', W as usize).collect();

    for _ in 0..2 {
        full_screen(&mut screen, id, &row, Style::new());
        screen.present();
    }

    assert_no_alloc(|| {
        for i in 0..10u8 {
            // A different style every frame, so every one of the 24 000 cells changes and the
            // serializer walks the whole screen.
            full_screen(
                &mut screen,
                id,
                &row,
                Style::new().fg(vitui_engine::Color::indexed(i)),
            );
            assert!(screen.present().submitted);
        }
    });
}

/// Spec §4's whole frame: a clear, 80 rows of text, a ZWJ family emoji, a box-drawn popup and CJK.
///
/// The clusters are what makes this different from the frames above. Segmentation borrows slices of
/// the caller's string and never builds a `Vec<&str>`; the interner mints a handle once per
/// *distinct* cluster and never again; and the packet's cluster arena is cleared rather than freed,
/// so it keeps the capacity it reached. Each of those is a place a `Vec` could have gone, and this
/// is the test that says none of them did.
fn a_frame_of_clusters_allocates_nothing() {
    let (mut screen, id) = screen();
    let family = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";
    let latin: String = std::iter::repeat_n('m', W as usize).collect();
    let cjk: String = std::iter::repeat_n('漢', W as usize / 2).collect();
    let combining = "e\u{301}a\u{308}o\u{302}u\u{308}i\u{301}";

    let frame = |screen: &mut Screen, tint: u8| {
        let style = Style::new().fg(vitui_engine::Color::indexed(tint));
        let mut v = screen.layers().view(id).expect("the layer is still there");
        v.fill(Rect::new(0, 0, W, H), " ", style);
        for y in 0..H as i32 {
            v.text(0, y, if y % 2 == 0 { &latin } else { &cjk }, style);
        }
        v.text(4, 2, family, style);
        v.text(8, 2, combining, style);
        // A box-drawn popup: four degenerate fills and four corners, which is what spec §4 says a
        // box is instead of a primitive.
        let (x, y, w, h) = (40i32, 10i32, 30i32, 8i32);
        v.fill(Rect::new(x, y, w as u16, 1), "─", style);
        v.fill(Rect::new(x, y + h - 1, w as u16, 1), "─", style);
        v.fill(Rect::new(x, y, 1, h as u16), "│", style);
        v.fill(Rect::new(x + w - 1, y, 1, h as u16), "│", style);
        v.set(x, y, "┌", style);
        v.set(x + w - 1, y, "┐", style);
        v.set(x, y + h - 1, "└", style);
        v.set(x + w - 1, y + h - 1, "┘", style);
    };

    // Two frames of warm-up: what is gated is the steady state, and every distinct cluster this
    // scene contains has entered the interner and the packet's arena by the end of the first.
    for tint in 0..2u8 {
        frame(&mut screen, tint);
        screen.present();
    }

    assert_no_alloc(|| {
        for tint in 2..12u8 {
            frame(&mut screen, tint);
            assert!(
                screen.present().submitted,
                "frame {tint} had nothing to say"
            );
        }
    });
}

fn an_idle_frame_allocates_nothing() {
    let (mut screen, id) = screen();
    let row: String = std::iter::repeat_n('i', W as usize).collect();
    full_screen(&mut screen, id, &row, Style::new());
    screen.present();

    assert_no_alloc(|| {
        for _ in 0..1_000 {
            assert!(!screen.present().submitted);
        }
    });
}
