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
use std::sync::{Arc, Mutex};

use vitui_alloc_probe::{CountingAllocator, assert_no_alloc};
use vitui_engine::{
    Color, ColorDepth, Config, Engine, LayerId, Mix, Output, Overrides, Rect, Restyle, Screen,
    Style,
};

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
    screen_with(Overrides::default())
}

/// What every gate here whose extended cells are **hyperlinks** has to pin, and the whole of why.
///
/// Truecolor because §5 skips an operator layer outright at `ColorDepth::None` — see [`screen_with`]
/// for the instance that shipped without it. And `hyperlinks`, because OSC 8 reaches the wire only
/// where the terminal has it, and a caller-supplied sink is asked nothing.
///
/// The second pin is architecture ticket 22's and it is not yet load-bearing: **the gates below would
/// go vacuous the moment impl 17 lands §10's intern-key collapse**, because a hyperlink on a screen
/// where OSC 8 is inexpressible stops making a cell extended, and a `Mix` over an inline cell reaches
/// no table at all. So the gate that says a settled operator mints nothing would be asserting that a
/// table nobody reached stayed empty.
///
/// > **A gate can be written today and be vacuous later, and nothing in the gate changes.** What
/// > changes is a capability arriving with a reader. So the audit is not *did I pin the axes that
/// > exist* but *will the subject still be reached once every ticket that reads a capability has
/// > landed*.
///
/// The audit that finds these is a grep and not a list — `rg 'screen\.link\('` over `src/`, `tests/`
/// and `examples/` — and **two of the six it finds are in this file**, which is not `src/`.
fn hyperlinks_and_truecolor() -> Overrides {
    Overrides {
        colors: Some(ColorDepth::TrueColor),
        hyperlinks: Some(true),
        ..Default::default()
    }
}

/// The same screen, on a terminal with whatever `overrides` pins.
///
/// **A headless screen is at `ColorDepth::None`**, because a caller-supplied sink is asked nothing
/// and spec §10 will not invent a colour for one — and §5 skips an operator layer **outright** at
/// that depth. So a gate about an operator that does not pin a depth is a gate about the depth: it
/// asserts that a layer which was never visited allocated nothing, which is true of every layer that
/// was never visited.
///
/// That is not hypothetical. `a_settled_operator_over_a_hyperlinked_screen_allocates_nothing` shipped
/// with impl 12 without the pin and was green for exactly that reason; impl 08 found it while writing
/// the fading half of the same register entry, and the pin is what makes both halves about the mix.
fn screen_with(overrides: Overrides) -> (Screen, LayerId) {
    let (mut screen, _wake) = Engine::new(Config {
        size: (W, H),
        output: Output::Sink(Box::new(Discard)),
        overrides,
        ..Default::default()
    })
    .attach()
    .expect("attaching to a sink cannot fail");
    let id = screen.layers().add_content(0, Rect::new(0, 0, W, H), true);
    (screen, id)
}

/// Draw the whole screen through one layer.
///
/// # This reaches a door that is allowed to allocate, and that is not an accident here
///
/// `Screen::layers` is where the mark-and-compact sweep runs (impl 08), and a sweep allocates —
/// two marker vectors and one of surface pointers. It cannot fire in any window below, because the
/// high-water mark's floor is 256 table entries and the heaviest fixture here holds two: one
/// extended style for the hyperlink and one for the mix's result.
///
/// **It is worth saying out loud rather than relying on**, because the failure mode is confusing:
/// a future fixture that crossed the floor would fail *this* gate, whose subject is frame
/// composition, for something that is neither in a frame nor composition. The fix if that ever
/// happens is to widen the fixture's warm-up until the mark has moved past it, not to widen the
/// window until the gate goes green.
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
    a_filtered_frame_with_gaps_allocates_nothing();
    an_idle_frame_allocates_nothing();
    a_frame_of_clusters_allocates_nothing();
    a_settled_restyle_over_a_hyperlinked_screen_allocates_nothing();
    a_settled_operator_over_a_hyperlinked_screen_allocates_nothing();
    the_operator_reaches_the_wire_at_the_depth_the_gate_pins();
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
/// This **is** register entry #7's settled half — impl 08 wired the fading one, in
/// `crate::gates::a_fading_operator_mints_per_distinct_style_and_never_per_cell` — and it is the same
/// shape as `a_settled_restyle_over_a_hyperlinked_screen_allocates_nothing` one layer up.
///
/// Hyperlinked, because that is what makes the mix's result need a table entry at all: an inline
/// cell mixes to an inline word and reaches no table, so a screen of those would pass whatever
/// `recolour` did.
fn a_settled_operator_over_a_hyperlinked_screen_allocates_nothing() {
    // Two axes, both of which make this gate about something else if they are missing. See
    // `hyperlinks_and_truecolor`.
    let (mut screen, id) = screen_with(hyperlinks_and_truecolor());
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

/// A sink that keeps what it is given, so a test can prove the operator changed the wire.
#[derive(Clone)]
struct Tap(Arc<Mutex<Vec<u8>>>);

impl Write for Tap {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.0
            .lock()
            .expect("never poisoned")
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

/// **Assert the operator moved a cell before asserting anything about which ones.**
///
/// A gate that says an operator allocated nothing is worthless if the operator was skipped, and §5
/// skips one outright at `ColorDepth::None` — which is what a headless screen is by default. This is
/// the positive half, from outside the crate, where cells are not visible (ADR 0023): the same frame
/// with and without the operator layer has to reach the wire as **different bytes**.
///
/// It is here rather than folded into the gate above because the gate's window may not contain a
/// growing `Vec`, and a sink that keeps its bytes is one.
fn the_operator_reaches_the_wire_at_the_depth_the_gate_pins() {
    fn one_frame(operator: bool) -> Vec<u8> {
        let tap = Tap(Arc::new(Mutex::new(Vec::new())));
        let bytes = Arc::clone(&tap.0);
        let (mut screen, _wake) = Engine::new(Config {
            size: (W, H),
            output: Output::Sink(Box::new(tap)),
            overrides: hyperlinks_and_truecolor(),
            ..Default::default()
        })
        .attach()
        .expect("attaching to a sink cannot fail");
        let id = screen.layers().add_content(0, Rect::new(0, 0, W, H), true);
        let row: String = std::iter::repeat_n('m', W as usize).collect();
        full_screen(
            &mut screen,
            id,
            &row,
            Style::new().fg(Color::indexed(15)).bg(Color::indexed(8)),
        );
        if operator {
            screen
                .layers()
                .add_operator(1, Rect::new(0, 0, W, H), Mix::darken(Mix::FULL / 2));
        }
        screen.present();
        let out = bytes.lock().expect("never poisoned").clone();
        assert!(!out.is_empty(), "the frame wrote nothing at all");
        out
    }

    assert_ne!(
        one_frame(false),
        one_frame(true),
        "the operator layer changed no byte, so the gate above is about the colour depth"
    );
}

/// The memo's property, as an allocation count rather than as a stopwatch.
///
/// `restyle` reaches the extended-style table once per **distinct** style word, and the table is
/// deduplicated — so once a descriptor has been applied, applying it again finds every entry it
/// needs already there and mints nothing. That is what makes a settled operator converge after one
/// frame, and it is why a permanently *changing* one is the case eviction exists for.
///
/// This is not register entry #7, which is about the operator layer and lives one file across in
/// `crate::gates`. It is the half of that property `restyle` can be held to on its own.
///
/// It stops short of `present` on purpose, and the reason changed at impl 13: the serializer emits
/// SGR 58/59 and OSC 8 now, so the frame path is no longer what is unavailable. What this gate is
/// about is `restyle`'s own arithmetic — the memo, one layer above the frame — and putting a
/// `present` inside its window would fold the serializer's buffers into the measurement.
///
/// It still pins both axes, and the second one is why: after impl 17's intern-key collapse a
/// hyperlink on a screen where OSC 8 is inexpressible stops making a cell extended at all, and this
/// fixture's only extended channel is a hyperlink.
fn a_settled_restyle_over_a_hyperlinked_screen_allocates_nothing() {
    let (mut screen, id) = screen_with(hyperlinks_and_truecolor());
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

/// **Composition, and since impl 14 that is all it is**, which is worth saying rather than leaving
/// for somebody to discover.
///
/// Every frame below writes the same row with the same style, so the equality filter finds nothing
/// changed and the frame reaches the wire as zero bytes. What is still measured is what the name says
/// — a thousand damage-mark, composite and pack cycles — and the serializer's own path is measured by
/// [`a_full_screen_frame_allocates_nothing`], which varies the style, and by
/// [`a_filtered_frame_with_gaps_allocates_nothing`], which varies a quarter of it.
///
/// The rotation over rows is what this one has that the others do not: a thousand frames each
/// damaging a different row, which is a thousand different run shapes through `pack`.
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

/// The filter's own path, **with gaps in it.**
///
/// The two full-screen gates vary the style every frame, so every cell changes and the filter skips
/// nothing: the comparison runs and the gap merge never does. This is the frame shape that exercises
/// it — every fourth column changes and the three columns between them are priced against the move
/// they would avoid, at three bytes against four, so the merge fires on every gap of every row.
///
/// The property is that pricing a gap and painting through it reaches no allocator, and it has none
/// to reach by construction: the plan *is* the run, and the walk that prices a gap stops after the
/// bytes of the move it is priced against. There is no per-row scratch buffer, which is the thing
/// this gate would have caught somebody adding.
fn a_filtered_frame_with_gaps_allocates_nothing() {
    let (mut screen, id) = screen();
    // Built **before** the window: building a `String` allocates, and the gate would be measuring
    // the fixture rather than the frame.
    let rows: Vec<String> = (0..10u8)
        .map(|i| {
            (0..W as u32)
                .map(|x| {
                    if x % 4 == 0 {
                        char::from(b'a' + i)
                    } else {
                        '.'
                    }
                })
                .collect()
        })
        .collect();

    for row in &rows[..2] {
        full_screen(&mut screen, id, row, Style::new());
        screen.present();
    }

    assert_no_alloc(|| {
        for row in &rows[2..] {
            full_screen(&mut screen, id, row, Style::new());
            assert!(screen.present().submitted, "the frame had nothing to say");
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
