//! **The reactivity layer is not on the frame path, and the gate is a delta rather than a zero.**
//!
//! Runtime ticket 18. The budget says *zero allocations during frame composition*, and this screen
//! cannot assert that number: `board::footer` builds its line with `format!`, which is one
//! allocation a frame and is the fixture's, not the graph's. A gate that asserted zero here would be
//! a gate about a `format!`.
//!
//! So the question is asked the way it is actually meant: **how many allocations does a signal graph
//! add to a frame the direct application already draws?** The answer is the same number for TEA, and
//! it is zero — the whole of a graph's per-frame work is three `Rc` derefs, a twelve-byte copy and a
//! comparison, none of which touch the allocator.
//!
//! # Why this is its own binary
//!
//! [`vitui_alloc_probe::CountingAllocator`] is a **process-global** allocator and the count is a
//! process-global counter, so an allocating sibling lands in the number. That is why the workspace's
//! one test command is `cargo test --workspace -- --test-threads=1`, and why the runtime keeps its
//! allocation gates in binaries of their own. This file is the same arrangement one crate up.

#[allow(dead_code)]
#[path = "../src/board.rs"]
mod board;

use board::{Data, Direct, H, Ink, Press, Signals, Tea, W};
use vitui_alloc_probe::{CountingAllocator, count_allocations};
use vitui_runtime::ctx::Driver;

#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

/// Frames measured, after an identical warm pass.
///
/// **The warm pass is the whole workload rather than a prefix of it**, which is the rule
/// `vitui_alloc_probe::steady` exists to carry: two allocation gates flaked in CI on a warm-up that
/// touched fewer buffers than the measurement did. `steady` itself is not used here because what is
/// wanted is a *count* to subtract, not an assertion of zero.
const FRAMES: usize = 64;

/// Draw `FRAMES` frames with `step`, warm first, and return the allocations of the measured pass.
fn frames(mut step: impl FnMut(&mut Ink)) -> usize {
    let mut ink = Ink::new();
    for _ in 0..FRAMES {
        step(&mut ink);
    }
    let (_, count) = count_allocations(|| {
        for _ in 0..FRAMES {
            step(&mut ink);
        }
    });
    count
}

/// **A signal graph adds nothing to the frame, and neither does a TEA pump.**
///
/// The direct driver's count is the fixture's own — one `format!` a frame, plus whatever the runtime
/// and the engine do behind a quiet frame — and it is the baseline rather than the claim. The claim
/// is the two deltas.
#[test]
fn the_reactive_drivers_add_no_allocation_to_a_quiet_frame() {
    let data = Data::default();

    let mut d_driver = Driver::headless(W, H).expect("a sink cannot fail to attach");
    let mut d_app = Direct::default();
    let direct = frames(|ink| {
        let _ = d_driver.frame(|cx| {
            let _ = d_app.view(cx, ink, &data, Press::Quiet);
        });
    });

    let mut t_driver = Driver::headless(W, H).expect("a sink cannot fail to attach");
    let mut t_app = Tea::default();
    let tea = frames(|ink| {
        let _ = t_driver.frame(|cx| {
            let _ = t_app.view_and_pump(cx, ink, &data, Press::Quiet);
        });
    });

    let mut s_driver = Driver::headless(W, H).expect("a sink cannot fail to attach");
    let mut s_app = Signals::default();
    let signals = frames(|ink| {
        let _ = s_driver.frame(|cx| {
            let _ = s_app.flush(cx, ink, &data, Press::Quiet);
        });
    });

    assert_eq!(
        signals, direct,
        "the signal graph allocates on the frame path: {signals} against {direct} over {FRAMES} frames"
    );
    assert_eq!(
        tea, direct,
        "the TEA pump allocates on the frame path: {tea} against {direct} over {FRAMES} frames"
    );
}

/// **Building the graph is a bounded, one-time cost.**
///
/// One `Rc` for the dirty flag, then two a signal — the value and the write counter — and nothing for
/// the [`vitui_signals::Computed`], whose `Memo` is empty until something asks. Seven, and the gate
/// is `≤ 8` rather than `== 7` because the number belongs to *how many signals this screen has*
/// rather than to the mechanism: a relation, not an equality, which is this backlog's rule for a
/// number that belongs to the data.
#[test]
fn building_the_graph_costs_at_most_eight_allocations() {
    // Warm: the first `Rc` in a process touches the allocator's own bookkeeping.
    let _warm = Signals::new();
    let (app, count) = count_allocations(Signals::new);
    assert!(
        count <= 8,
        "building a three-signal graph cost {count} allocations"
    );
    assert_eq!(
        app.max.recomputes(),
        0,
        "a fresh Computed has folded nothing"
    );
}
