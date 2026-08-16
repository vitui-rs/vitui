//! The performance budget, expressed as benchmarks.
//!
//! These are placeholders that measure nothing yet — they exist so the harness, the profile and the
//! CI wiring are in place before the first real implementation lands, and so the budget is visible
//! in the repository rather than only on the map.
//!
//! The budget, from `.scratch/vitui-engine-architecture/map.md`:
//!
//! | Scene                                     | Budget   |
//! |-------------------------------------------|----------|
//! | Full-screen composition, 300x80 (~24k cells) | < 1 ms   |
//! | Typical damage-tracked frame              | < 100 us |
//! | Steady-state 60 fps animation             | < 5% of one core |
//! | Scene of 1M elements vs 1k elements       | same time |
//!
//! Allocations during frame composition must be zero; that is asserted in tests via
//! `vitui-alloc-probe`, not measured here.
//!
//! The three scenes the damage-model ticket calls for — a single blinking cursor, a scrolling list,
//! and twenty stacked animated popups — become the real benchmarks here.

use criterion::{Criterion, criterion_group, criterion_main};

/// Placeholder. Replace with full-screen composition of a 300x80 buffer once a `Surface` exists.
fn compose_full_screen(c: &mut Criterion) {
    c.bench_function("compose/full_screen_300x80", |b| {
        b.iter(|| std::hint::black_box(0u8));
    });
}

criterion_group!(benches, compose_full_screen);
criterion_main!(benches);
