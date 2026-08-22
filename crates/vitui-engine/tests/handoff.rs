//! The handoff's allocation gates: register entries #5 and #6.
//!
//! # Why this is a second binary and not two more phases of `alloc.rs`
//!
//! The probe's counter is process-global, so a test asserting on it must not run beside another test
//! that allocates — and `alloc.rs` says the rest of that sentence: separate binaries are separate
//! processes and do not interfere. These two gates need a **render thread running inside the
//! window**, which is the one thing every gate in that file is constructed to avoid, so they get a
//! process of their own rather than a phase in a test whose whole design is one thread.
//!
//! # The attribution rule, and the one gate in this repository that answers it rather than avoiding it
//!
//! `alloc.rs` states it: **an allocation gate with a background job in it runs on a deterministic
//! spawner, or joins before it measures.** There is no third option, and it exists because the
//! counter is a process-global atomic inside a global allocator, which has no idea who is calling it
//! — so a window opened around a frame while an unrelated worker is running attributes the worker's
//! growth to the frame.
//!
//! [`the_handoff_allocates_nothing`] cannot obey either clause: joining the render thread would end
//! the handoff it is measuring, and the spawner is `std::thread`. What makes it a gate anyway is that
//! **the background job is the subject**. The window contains exactly two threads, the app thread's
//! `lease → pack → submit` and the render thread's `take → serialise → write → finish`, and both of
//! those are what the gate is about. Nothing else runs in this process.
//!
//! That is the distinction the rule is actually drawing, and it is worth writing down because the
//! wrong reading of it — *never have a thread in the window* — would make this property untestable
//! for ever, and a property that cannot be tested is one that decays silently. What the rule forbids
//! is a window containing work the gate is **not** about.

use std::io::{Result, Write};

use vitui_alloc_probe::{CountingAllocator, steady};
use vitui_engine::{
    Clock, Color, ColorDepth, Config, Engine, InputConfig, LayerId, Output, Overrides, Rect,
    Restyle, Screen, Style,
};

#[global_allocator]
static ALLOC: CountingAllocator = CountingAllocator::new();

const W: u16 = 300;
const H: u16 = 80;

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

/// Truecolor and OSC 8, because a caller-supplied sink is asked nothing and a headless screen is
/// therefore at `ColorDepth::None` — where a hyperlink stops making a cell extended at all and the
/// dense arms below would be measuring a plain screen.
fn declared() -> Overrides {
    Overrides {
        colors: Some(ColorDepth::TrueColor),
        hyperlinks: Some(true),
        ..Default::default()
    }
}

fn screen(clock: Clock) -> (Screen, LayerId) {
    let (mut screen, _wake) = Engine::new(Config {
        size: (W, H),
        output: Output::Sink(Box::new(Discard)),
        clock,
        // Unpaced, because nothing here calls `wait`: the gate is the handoff, and a gap would only
        // be a number these frames never read.
        max_frame_rate: f32::INFINITY,
        overrides: declared(),
        overrun_threshold: None,
        overrun_report: None,
        input: InputConfig::default(),
    })
    .attach()
    .expect("attaching to a sink cannot fail");
    let id = screen.layers().add_content(0, Rect::new(0, 0, W, H), true);
    (screen, id)
}

/// Present until a frame is handed on, spinning while the renderer has not taken the last one.
///
/// The spin is what a caller has instead of a parking point until ticket 19's `wait`, and it is also
/// the right shape for this gate: a frame the renderer is not ready for is **not composited**, so the
/// spin costs a lock acquisition and a compare and touches no allocator.
fn present_eventually(screen: &mut Screen) {
    while !screen.present().submitted {
        std::hint::spin_loop();
    }
}

#[test]
fn the_steady_state_of_the_handoff_allocates_nothing() {
    the_handoff_allocates_nothing();
    pack_allocates_nothing_at_every_density();
}

/// **Register entry #5: zero allocations over 1 000 `lease → pack → submit → take → finish`.**
///
/// On the real three-thread path, which is the only place those five words all exist: the first three
/// happen on the app thread inside `present` and the last two on the render thread, and a pool that
/// allocated a packet per frame — or a packet that allocated its side tables per frame — would show
/// up here and nowhere else.
///
/// The pool is two `Box<Packet>` made at `attach`. What the frames below have to reach is the state
/// where every buffer inside both of them is at its high-water mark: the runs, the cells, the arena,
/// and the generation-stamped marker vectors, whose one growth is to the size of the handle tables.
///
/// # Ten warm-up frames were not enough, and this gate flaked in CI because of it
///
/// It observed **nine allocations** on pipeline #17 having passed everywhere else. The moving cell is
/// at `(t % 300, t % 80)`, so the damage pattern has a period of twelve hundred frames — ten frames
/// visit ten positions near the diagonal and the measured thousand visit a thousand, several of which
/// split a row into more runs than anything in the warm-up did. The high-water mark was reached
/// *inside the window*.
///
/// So the warm pass is now **the identical workload** rather than a prefix of it: `steady` runs the
/// whole thousand once and asserts over the second thousand. Same `t` values, same damage patterns,
/// same buffers — and no guess about which prefix is representative. See
/// [`vitui_alloc_probe::steady`].
fn the_handoff_allocates_nothing() {
    let (mut screen, id) = screen(Clock::System);
    let row: String = std::iter::repeat_n('m', W as usize).collect();
    let ink = Style::new().fg(Color::indexed(15)).bg(Color::indexed(8));

    let frame = |screen: &mut Screen, t: u32| {
        {
            let mut view = screen.layers().view(id).expect("the layer was just added");
            for y in 0..H as i32 {
                view.text(0, y, &row, ink);
            }
            // One cell moves every frame, so the equality filter has something to emit and the
            // render thread has something to write. A frame of zero bytes would still cycle the
            // mailbox, but it would not exercise the serializer's own buffers.
            view.text(
                (t % u32::from(W)) as i32,
                (t % u32::from(H)) as i32,
                "x",
                ink.bold(),
            );
        }
        present_eventually(screen);
    };

    steady(|| {
        for t in 0..1_000 {
            frame(&mut screen, t);
        }
    });
}

/// **Register entry #6: `pack` allocates zero on warm tables, at every density.**
///
/// The four densities are spec §7's own — none of the 24 000 cells carrying a handle, one in a
/// hundred, all of them naming one entry, and all of them naming a distinct entry. What the entry is
/// about is the generation-stamped marker: its slots grow to the handle tables' high-water mark and
/// never again, so a frame on warm tables reaches no allocator however many handles it resolves. A
/// packet that cleared and refilled a `HashMap` instead would pass the plain arm and fail the dense
/// ones.
///
/// It is asserted over the whole frame rather than over `pack` alone, which is stronger and is the
/// only shape available from outside the crate (ADR 0023: a caller cannot read back cells, let alone
/// a packet). A frame that allocates nothing contains a `pack` that allocates nothing.
///
/// Deterministic, because nothing here is about the threads: `Clock::Manual` runs the whole round on
/// this thread, and the window then contains exactly one thread's work — which is the ordinary
/// reading of the attribution rule rather than the argued one above.
fn pack_allocates_nothing_at_every_density() {
    for density in ["plain", "realistic 1%", "linked 100%", "hostile 100%"] {
        let (mut screen, id) = screen(Clock::Manual);
        let row: String = std::iter::repeat_n('m', W as usize).collect();
        let link = screen.link("https://example.com/vitui");
        let ink = Style::new().fg(Color::indexed(15)).bg(Color::indexed(8));

        let frame = |screen: &mut Screen, t: u32| {
            let mut view = screen.layers().view(id).expect("the layer was just added");
            // A different weight every frame, so every cell changes and the whole screen is packed.
            let ink = if t % 2 == 0 { ink } else { ink.bold() };
            for y in 0..H as i32 {
                view.text(0, y, &row, ink);
            }
            // The density is applied **after** the text, because `text` writes the style word it is
            // given and would take an extended cell back to inline.
            match density {
                "plain" => {}
                "realistic 1%" => {
                    for y in (0..H as i32).step_by(4) {
                        for x in (0..W as i32).step_by(25) {
                            view.text(x, y, "e\u{301}", ink);
                            view.restyle(
                                Rect::new(x, y, 1, 1),
                                &Restyle {
                                    ul: Some(Color::rgb(1, 2, 3)),
                                    ..Default::default()
                                },
                            );
                        }
                    }
                }
                "linked 100%" => view.restyle(
                    Rect::new(0, 0, W, H),
                    &Restyle {
                        link: Some(link),
                        ..Default::default()
                    },
                ),
                "hostile 100%" => {
                    for y in 0..H {
                        for x in 0..W {
                            let n = u32::from(y) * u32::from(W) + u32::from(x);
                            view.restyle(
                                Rect::new(x as i32, y as i32, 1, 1),
                                &Restyle {
                                    ul: Some(Color::rgb((n >> 16) as u8, (n >> 8) as u8, n as u8)),
                                    ..Default::default()
                                },
                            );
                        }
                    }
                }
                other => unreachable!("no such density: {other}"),
            }
            assert!(
                screen.present().submitted,
                "{density}: frame {t} said nothing"
            );
        };

        // **The warm pass is the identical workload**, for the reason the gate above it now carries:
        // a six-frame prefix reaches the high-water marks that six frames reach, and the hundred
        // measured after it need not be a subset of those. The extended-style table peaks on the
        // first frame, the marker vectors grow to it on the second, and the mark-and-compact sweep in
        // `Screen::layers` settles once nothing is orphaned — all of which the first hundred does too,
        // and without anyone having to decide which prefix is representative.
        steady(|| {
            for t in 0..100 {
                frame(&mut screen, t);
            }
        });
    }
}
