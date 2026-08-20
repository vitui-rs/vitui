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
use vitui_engine::{Config, Engine, LayerId, Output, Rect, Screen, Style};

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
