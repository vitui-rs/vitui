//! **The one number on the overlay family's screen that needs something to count: the frame's
//! allocation total.**
//!
//! Components ticket 25. `src/popup.rs` holds every other gate on that screen; this file exists
//! because [`vitui_alloc_probe::CountingAllocator`] is a **process-global** allocator over a
//! process-global counter and a library cannot install one on a consumer's behalf. It is its own
//! binary for `tests/budget.rs`'s reason, and it needs no new CI invocation:
//! `cargo test --workspace -- --test-threads=1` is what the `test` job already runs.
//!
//! # What it settles, and it contradicts a closed map
//!
//! Spec §12's table reads `allocations = 0` in all five of its rows. **That is not the shipped
//! number and it cannot be made one.** The table was measured on the prototype's crate-private bump
//! arena, which runtime ticket 21 deleted (ADR 0034) because it was seven `unsafe` blocks in a
//! workspace that has none. A body is now one `Box` in a queue the frame call owns, so:
//!
//! | overlays standing | allocations a frame |
//! |---|---|
//! | 0 | **0** — an empty `Vec` allocates nothing |
//! | 1 | 2 |
//! | 2 | 3 |
//!
//! The runtime's own gate is the **marginal equality** — one more overlay standing is exactly one
//! more allocation a frame — and that is what is asserted here, over a screen of 312 chips rather
//! than over a fixture: the marginal cost is the claim, and a base pass that allocated would show up
//! as an offset in the first row rather than as a slope in the difference.
//!
//! The `+ 1` is not slack. A stored body is `+ 'f`, and safe Rust cannot put a `'f`-bounded value
//! inside the thing borrowed for `'f` — which `Frame` and `Driver` both are — so the queue is a local
//! of `Driver::frame` rather than a field, and the queue itself is the extra allocation. **That is
//! exactly why the arena erased types in the first place**: an offset into a byte buffer mentions no
//! lifetime.

use vitui_alloc_probe::{CountingAllocator, count_allocations};
use vitui_components::counters::Allocations;
use vitui_components::popup::{self, Config};

#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

/// Frames the window is opened over. A **total**, never a mean (§21's refinement 2).
const FRAMES: u32 = 60;

/// One configuration's allocation total over [`FRAMES`] steady frames.
///
/// Warmed with two frames before the window opens, for the reason `vitui_alloc_probe::steady`
/// documents: the five frame structures take their allocation on the first frame that needs one and
/// keep it, and the overlay's *layer* is allocated on the frame it is first placed on — which is
/// [`popup::opening`]'s subject and not this one.
fn total(config: Config) -> Allocations {
    let mut driver = vitui_components::runner::driver_at(popup::W, popup::H, Default::default());
    let mut held = popup::Held::new();
    let mut ink = vitui_components::ink::Direct;
    for _ in 0..2 {
        driver.frame(|cx| {
            popup::draw_into(&mut ink, cx, &mut held, config);
        });
    }
    let (_, allocated) = count_allocations(|| {
        for _ in 0..FRAMES {
            driver.frame(|cx| {
                popup::draw_into(&mut ink, cx, &mut held, config);
            });
        }
    });
    Allocations::over(FRAMES, allocated as u64)
}

/// **A frame with no overlay allocates nothing, as a total over sixty frames.**
///
/// This is §12's `0` and it is true of exactly one of its five rows.
#[test]
fn a_frame_with_no_overlay_allocates_nothing() {
    let seen = total(Config::Nothing);
    assert_eq!(
        seen.total(),
        0,
        "{} allocations over {} frames on a screen of 312 chips with nothing open, and the budget \
         is a total of zero",
        seen.total(),
        seen.frames()
    );
}

/// **One more overlay standing is exactly one more allocation a frame.**
///
/// The marginal equality, which is the form the runtime's own gate takes and the only form that says
/// anything about the mechanism rather than about this screen. Read as absolutes: 0, 2 and 3.
#[test]
fn one_more_overlay_standing_is_exactly_one_more_allocation_a_frame() {
    let none = total(Config::Nothing).total();
    let one = total(Config::OneSelect).total();
    let two = total(Config::MenuAndSubmenu).total();

    assert_eq!(none, 0, "a frame with no overlay allocates nothing at all");
    assert_eq!(
        one,
        u64::from(FRAMES) * 2,
        "one body plus the queue that holds it"
    );
    assert_eq!(two, u64::from(FRAMES) * 3, "two bodies plus the queue");
    assert_eq!(
        two - one,
        u64::from(FRAMES),
        "one more overlay standing is one more allocation a frame"
    );

    // And the shape the module states, per frame, for every configuration on the screen.
    for config in Config::ALL {
        let seen = total(config).total();
        assert_eq!(
            seen,
            u64::from(FRAMES) * popup::overlay_allocs(config.bodies()) as u64,
            "{}",
            config.word()
        );
    }
}

/// **A body that captures nothing costs no allocation to box, and that is how `n + 1` goes green
/// while meaning something else.**
///
/// `Box::new` of a zero-sized value does not allocate, and a body written as a bare `fn` item — or as
/// a closure that happens to capture nothing — *is* zero-sized. So a screen standing two such
/// overlays costs **one** allocation a frame, the queue's, and a gate reading *at most `n + 1`* is
/// satisfied by a frame that boxed nothing at all.
///
/// This is not hypothetical: `crate::popup`'s menu and submenu were written that way and read 1
/// against the 3 the marginal equality asks for. The bodies now carry the row the submenu hangs off,
/// which is the owner's data and what a real body captures. The fact is kept here because it belongs
/// to `Box<dyn FnMut>` rather than to the runtime, and a reader who does not know it will read the
/// gate above as measuring something it does not.
#[test]
fn a_body_that_captures_nothing_costs_no_allocation_to_box() {
    fn empty(_cx: &mut vitui_runtime::Ctx<'_, '_>) {}

    let mut driver = vitui_components::runner::driver_at(40, 8, Default::default());
    let anchor = vitui_runtime::Rect::new(0, 0, 4, 1);
    let opts = vitui_runtime::overlay::OverlayOpts::sized(8, 2);
    let frame = |driver: &mut vitui_runtime::ctx::Driver| {
        driver.frame(|cx| {
            cx.overlay(vitui_runtime::Id::named("zst.a"), anchor, opts, empty);
            cx.overlay(vitui_runtime::Id::named("zst.b"), anchor, opts, empty);
        });
    };
    frame(&mut driver);
    frame(&mut driver);
    assert_eq!(driver.inspect().overlay_bodies_boxed(), 2, "two bodies");

    let (_, allocated) = count_allocations(|| {
        for _ in 0..FRAMES {
            frame(&mut driver);
        }
    });
    assert_eq!(
        allocated as u64,
        u64::from(FRAMES),
        "two zero-sized bodies cost the queue and nothing else — `n + 1` would have said three"
    );
}

/// **§12's zero is not reachable, and saying so is the point of this file.**
///
/// A gate that asserted `0` here would fail on four of §12's five rows, and the edit that would make
/// it pass is the arena. So the disagreement is asserted rather than resolved: the table's figure and
/// the shipped figure are both written down, and the one that moves is the table's.
#[test]
fn the_specs_zero_is_the_arena_that_was_deleted_and_not_the_shipped_number() {
    let with_one_open = total(Config::OneSelect).total();
    assert_ne!(
        with_one_open, 0,
        "§12's table reads 0 with one select open. It is 2 a frame, and the arena that made it 0 is \
         gone — see this file's header and ADR 0034"
    );
}
