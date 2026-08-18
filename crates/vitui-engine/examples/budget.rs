//! The performance budget, expressed as the harness that will measure it.
//!
//! Run it: `cargo run --release --example budget -p vitui-engine`
//!
//! This is a placeholder that measures nothing yet. It exists so the harness and the CI wiring are
//! in place before the first implementation lands, and so both the budget and the **scene list**
//! are visible in the repository rather than only on the map. The scene list is not decoration:
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

use std::time::Duration;
use vitui_bench::Bench;

fn main() {
    let report = Bench::new(40)
        .case("placeholder/noop", 1_000, || {
            std::hint::black_box(0u8);
        })
        .run();

    println!("budget harness, minimum of 40 rounds:\n{report}");
    report.assert_under("placeholder/noop", Duration::from_millis(1));
}
