//! The performance budget, expressed as the harness that will measure it.
//!
//! Run it: `cargo run --release --example budget -p vitui-engine`
//!
//! Ticket 03 filled this in: six scenes, each one a whole frame through the public surface, three
//! of them gated. It exists so that both the budget and the **scene list** are visible in the
//! repository rather than only on the map. The scene list is not decoration:
//! ticket 07 found that the three scenes it had named itself scored identically on every candidate
//! damage structure, and that the two which discriminated were not on the list. A scene list is
//! part of a gate, and omitting a scene validates the wrong design while reporting success.
//!
//! # The budget
//!
//! | Scene                                        | Budget           |
//! |----------------------------------------------|------------------|
//! | Full-screen composition, 300x80 (~24k cells) | < 1 ms           |
//! | Typical damage-tracked frame                 | < 100 us         |
//! | Steady-state 60 fps animation                | < 5% of one core |
//! | 1M elements against 1k elements              | the same time    |
//! | In-loop overrun detector                     | < 50 ns          |
//!
//! Allocation gates and idle-cost gates are not measured here — they are assertions and process
//! measurements respectively, and both live elsewhere. See the verification-strategy ticket.
//!
//! # The scenes, and which ticket each one decided
//!
//! | Scene                                     | What it caught                              |
//! |-------------------------------------------|---------------------------------------------|
//! | Caret blink, one cell                     | equality filter, 3.0x bytes (08)            |
//! | Progress tick, one row                    | equality filter, 30.8x bytes (08)           |
//! | Scrolling list, one row                   | scroll region, 1726 -> 60 bytes (08)        |
//! | Three dialogs standing apart              | per-row spans, 2.53x overdraw (07)          |
//! | Sparse sub-cell chart                     | 37.07x overdraw (07), 4.88x walk (09),      |
//! |                                           | 166.76 us watchdog threshold (18)           |
//! | Twenty stacked popups                     | depth cost, discriminates nothing (07)      |
//! | Full-screen operator layer                | 78.2 us of the 107 us worst screen (06, 11) |
//! | Virtualised 1M-row tree                   | the data-volume invariant (05, 14)          |
//! | Hyperlinked page under an animating       | the only shape that grows a handle table    |
//! | operator                                  | without bound (16)                          |
//!
//! The last one has no measurement behind it yet and is the one this file exists to make sure
//! nobody forgets.
//!
//! # What ticket 03 can measure, and what it cannot
//!
//! Everything here goes through the public surface, because the public surface is all an example
//! has. That decides the shape of the report: spec §6's table separates mark, scan, union and
//! clear, and from outside the engine those four are one call. So each case below names which of
//! §6's numbers it contains rather than pretending to isolate one.
//!
//! In the deterministic single-thread mode `present` also serialises and writes, which on the
//! shipped three-thread path is the render thread's work and not the app thread's. The full-screen
//! number is therefore the sum of two budgets, and it is gated against the larger of them.
//!
//! The gates are counts and cliff-granularity ceilings, per §14's register. A 10% timing regression
//! is not detectable on a shared runner, and a gate that claims to detect one is a flaky test
//! wearing a budget's clothes.

use std::io::{Result, Write};
use std::time::Duration;

use vitui_bench::Bench;
use vitui_engine::{Color, Config, Engine, LayerId, Output, Rect, Screen, Style};

const W: u16 = 300;
const H: u16 = 80;

/// Takes everything, keeps none of it: a recording sink would measure a `Vec` growing.
struct Discard;

impl Write for Discard {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

/// A screen with a full-screen base layer, plus however many popups the scene wants.
struct Scene {
    screen: Screen,
    /// `layers[0]` is the full-screen base; the rest are popups. Two fields rather than a tuple,
    /// so that `scene.screen` and `scene.layers` are disjoint borrows and a case can draw into a
    /// layer it names in the same expression.
    layers: Vec<LayerId>,
}

fn scene(layers: u32) -> Scene {
    let (mut screen, _wake) = Engine::new(Config {
        size: (W, H),
        output: Output::Sink(Box::new(Discard)),
        ..Default::default()
    })
    .attach()
    .expect("attaching to a sink cannot fail");

    let mut ids = vec![screen.layers().add_content(0, Rect::new(0, 0, W, H), true)];
    for i in 0..layers {
        // Twenty popups, ninety columns wide, walking down the screen.
        let x = (i as i32 * 11) % (W as i32 - 90);
        let y = (i as i32 * 3) % (H as i32 - 14);
        ids.push(
            screen
                .layers()
                .add_content(i as i32 + 1, Rect::new(x, y, 90, 14), true),
        );
    }
    Scene {
        screen,
        layers: ids,
    }
}

fn main() {
    let row: String = std::iter::repeat_n('x', W as usize).collect();

    // Every case draws and presents, so the reported number is one whole frame: mark, union, scan,
    // composite, pack, serialise, write and clear.
    let mut idle = scene(0);
    idle.screen
        .layers()
        .view(idle.layers[0])
        .unwrap()
        .text(0, 0, &row, Style::new());
    idle.screen.present();

    let mut caret = scene(0);
    let mut full = scene(0);
    let mut chart = scene(0);
    let mut dialogs = scene(0);
    let mut popups = scene(20);

    // One counter per case: the bench holds every closure at once, so a shared counter would be
    // two mutable borrows of the same local.
    let (mut t_caret, mut t_dialogs, mut t_chart, mut t_popups, mut t_full) =
        (0u32, 0u32, 0u32, 0u32, 0u32);

    let report = Bench::new(40)
        .case("frame/idle", 1_000, || {
            // Nothing damaged: the summary word says so and `clear` touches zero rows.
            std::hint::black_box(idle.screen.present());
        })
        .case("frame/caret-blink", 1_000, || {
            t_caret = t_caret.wrapping_add(1);
            let style = if t_caret % 2 == 0 {
                Style::new().reverse()
            } else {
                Style::new()
            };
            caret
                .screen
                .layers()
                .view(caret.layers[0])
                .unwrap()
                .text(10, 5, " ", style);
            std::hint::black_box(caret.screen.present());
        })
        .case("frame/three-dialogs-apart", 200, || {
            t_dialogs = t_dialogs.wrapping_add(1);
            let style = Style::new().fg(Color::indexed((t_dialogs % 16) as u8));
            let mut view = dialogs.screen.layers().view(dialogs.layers[0]).unwrap();
            for y in 4..18 {
                view.text(2, y, "left dialog", style);
                view.text(140, y, "middle dialog", style);
                view.text(280, y, "right", style);
            }
            std::hint::black_box(dialogs.screen.present());
        })
        .case("frame/sparse-chart-400-points", 200, || {
            t_chart = t_chart.wrapping_add(1);
            let mut view = chart.screen.layers().view(chart.layers[0]).unwrap();
            for i in 0..400i32 {
                let x = (i * 7 + t_chart as i32) % W as i32;
                let y = (i * 13) % H as i32;
                view.text(x, y, "*", Style::new());
            }
            std::hint::black_box(chart.screen.present());
        })
        .case("frame/twenty-popups-union", 100, || {
            t_popups = t_popups.wrapping_add(1);
            let style = Style::new().fg(Color::indexed((t_popups % 16) as u8));
            for i in 0..popups.layers.len() {
                let id = popups.layers[i];
                popups
                    .screen
                    .layers()
                    .view(id)
                    .unwrap()
                    .text(0, 0, "popup", style);
            }
            std::hint::black_box(popups.screen.present());
        })
        .case("frame/full-screen-300x80", 20, || {
            t_full = t_full.wrapping_add(1);
            let style = Style::new().fg(Color::indexed((t_full % 16) as u8));
            let mut view = full.screen.layers().view(full.layers[0]).unwrap();
            for y in 0..H as i32 {
                view.text(0, y, &row, style);
            }
            std::hint::black_box(full.screen.present());
        })
        .run();

    println!("budget harness, minimum of 40 rounds:\n{report}");

    // Cliff-granularity gates, with the headroom stated rather than implied. Ticket 03 measured
    // roughly 10 ns, 3 us and 250 us for these three in release on an unloaded aarch64 laptop.
    report.assert_under("frame/idle", Duration::from_micros(10));
    report.assert_under("frame/caret-blink", Duration::from_micros(100));
    report.assert_under("frame/full-screen-300x80", Duration::from_millis(1));
}
