//! **The crate line, from the side that matters: a consumer that cannot name the engine.**
//!
//! Runtime ticket 17. Every gate written on tickets 01–16 lives in one of three places, and two of
//! them are already across a crate boundary: `crates/vitui-runtime/tests/` and the sixteen
//! `examples/*_numbers.rs` are separate crates that link `vitui-runtime` as an rlib and can name
//! nothing it does not export. The third place is `#[cfg(test)] mod tests` inside `src/`, and that
//! is where the debt was — **a gate compiled inside the library can reach a `pub(crate)` item, and
//! seventy-eight of them exist.** What stood in for the build was a count of those items on the
//! seam; this file is the build.
//!
//! # Why this file is in the components package and not in the runtime's
//!
//! `crates/vitui-runtime/tests/` is across the *runtime's* line but not across the *component's*:
//! `vitui-engine` is a dependency of that package, so a test there may name `vitui_engine::Rect`,
//! and every one of them does. The components package is the only crate in this workspace whose
//! dependency list is `vitui-runtime` **and nothing else** — components spec §0 and §21, constraint
//! C6 — so here a `use vitui_engine::…` is `error[E0432]: unresolved import` before any gate runs.
//! The boundary is enforced by cargo rather than by review, which is the whole point of building it
//! rather than counting it.
//!
//! # What the build found, and it is not what the ticket expected
//!
//! **Seventeen engine types are named by the runtime's public surface and reachable through none of
//! it**, `Rect` in twenty-seven public declarations. A component-facing crate can *hold* such a
//! value — `cx.area()` hands one over and inference carries it — and cannot *write its type*, which
//! is exactly what components spec §1 asks every component to do:
//!
//! ```text
//! pub fn button(cx: &mut Ctx, area: Rect, label: &str) -> Response;
//! ```
//!
//! So the tests below are what the component-facing line reaches **today**, and the ones that are
//! missing are missing for a stated reason rather than for lack of effort:
//!
//! | ticket | reachable here | why not |
//! |---|---|---|
//! | 01 `data` | yes | |
//! | 02 `layout` | yes | a band comes from `Ctx::area` |
//! | 03 `layout::text` | yes | |
//! | 04 `theme` | yes | |
//! | 05 `theme::registry` | yes | |
//! | 06 `anim` | yes | |
//! | 07 `keys` | **no** | a `Key` has a `mods: Mods` field and `Mods` is not nameable here, so no key can be pressed |
//! | 08 the frame | partly | `Driver::headless` and `Driver::frame` are reachable; `post_key` and `post_mouse` take engine types |
//! | 09 `id` | yes | |
//! | 10 the hit index | **no** | it is answered from a posted mouse event |
//! | 11 `route` | **no** | `edge_of` and `batch_len` take `&Event` |
//! | 12 `focus` | partly | the ring is reachable, `Tab` is a posted key |
//! | 13 `overlay` | yes | placement is pure, and its rectangles are derived from `Ctx::area` |
//! | 14 `scroll` | yes | the four direction bits are arithmetic; the wheel is a posted event |
//! | 15 `sizing` | yes | `check` brings its own driver |
//! | 16 `work` | yes | `Worker::queueing` needs no `WakeHandle` |
//!
//! **Six of sixteen do not cross**, and every one of the six is blocked by the same fact: the
//! component-facing surface cannot construct or name an engine value. That is the finding filed
//! against runtime spec §4 as architecture issue 22, with the inventory as a value and a gate over
//! it in `crate::line::ENGINE_NAMES`; nothing here works around it.
//!
//! # What it is not
//!
//! It is not a second copy of tickets 01–16's suites — those are 359 tests inside the library and
//! they stay there. It is **one headline property per ticket**, chosen so that the property is the
//! mechanism rather than an example, and every one of them is written the way a component would have
//! to write it. Two of the fifteen were wrong the first time in a way no in-crate gate could have
//! shown: `Frame`'s accessors are `Driver::inspect`'s and not `Ctx`'s, and three lanes drawn from one
//! call site are one id — see `a_frame_runs_and_the_id_table_holds_what_drew`.

use vitui_runtime::anim::{Easing, Spring, Steps, Tween};
use vitui_runtime::ctx::Driver;
use vitui_runtime::layout::{
    Col,
    Constraint::{Fixed, Weight},
    Row, text,
};
use vitui_runtime::overlay::{Align, Placement, Side, place};
use vitui_runtime::scroll::Scrollable;
use vitui_runtime::theme::{Density, Glyph, Themes};
use vitui_runtime::work::{Drain, Task, Worker};
use vitui_runtime::{Ctx, Edit, Interest, Memo, Revision, Role, Versioned, sizing};

/// The dense screen's width and height, as every report on this backlog uses them.
const W: u16 = 300;
/// See [`W`].
const H: u16 = 80;

// ── 01 · the data contract ───────────────────────────────────────────────────────────────────────

/// **A memo recomputes on the revision and never on the value.** Ticket 01, spec §14.
///
/// The count is the gate: two `get` calls at one revision compute once, and the third computes only
/// because the `Edit` guard bumped the revision on drop. A memo keyed on the value would compute
/// three times and pass every equality in this test.
#[test]
fn a_memo_recomputes_on_the_revision_and_not_on_the_value() {
    let mut rows = Versioned::new(vec![1u32, 2, 3]);
    let mut memo: Memo<u32> = Memo::new();
    let mut computes = 0u32;

    let first = *memo.get(rows.revision(), || {
        computes += 1;
        rows.get().iter().sum()
    });
    let again = *memo.get(rows.revision(), || {
        computes += 1;
        rows.get().iter().sum()
    });
    assert_eq!((first, again, computes), (6, 6, 1));

    {
        let mut edit: Edit<'_, Vec<u32>> = rows.edit();
        edit.push(4);
    }
    let after = *memo.get(rows.revision(), || {
        computes += 1;
        rows.get().iter().sum()
    });
    assert_eq!((after, computes), (10, 2));
    assert_ne!(rows.revision(), Revision::UNKNOWN);
}

// ── 02 · layout ──────────────────────────────────────────────────────────────────────────────────

/// **Lanes tile the band the frame handed over**, with no gap and no overlap. Ticket 02, spec §11.
///
/// The band is `Ctx::area()` rather than a constructed rectangle, which is the only way a
/// component-facing crate can get one: see this file's header.
#[test]
fn the_lanes_tile_the_band_the_frame_handed_over() {
    let mut driver = Driver::headless(W, H).expect("a sink cannot fail to attach");
    driver.frame(|cx| {
        let band = cx.area();
        let [header, body, footer] = Col::new().split(band, [Fixed(3), Weight(1), Fixed(1)]);
        assert_eq!((header.h, footer.h), (3, 1));
        assert_eq!(header.h + body.h + footer.h, band.h);
        assert_eq!((header.y, body.y, footer.y), (0, 3, i32::from(H) - 1));

        let mut lanes = [band; 24];
        let n = Col::new().split_into(band, &[Weight(1); 24], &mut lanes);
        assert_eq!(n, 24);
        let total: u16 = lanes[..n].iter().map(|r| r.h).sum();
        assert_eq!(total, band.h);
    });
}

// ── 03 · text measurement ────────────────────────────────────────────────────────────────────────

/// **Two columns a cluster for CJK, one for a family emoji's whole sequence.** Ticket 03, spec §11
/// over ADR 0005's exports.
#[test]
fn text_measures_a_cluster_and_not_a_char() {
    assert_eq!(text::width("漢字漢字"), 8);
    assert_eq!(text::width("e\u{301}quipe"), 6);
    assert_eq!(text::wrap_height("one two three four", 9), 3);
    assert_eq!(text::truncate("supercalifragilistic", 5), "super");
    assert_eq!(text::truncate("漢字漢字", 3), "漢");
}

// ── 04 · theme ───────────────────────────────────────────────────────────────────────────────────

/// **A role is the only road to a paint.** Ticket 04, spec §3 and §15; ADR 0018.
///
/// `Paint` has no public constructor at all, so this is what a consumer can check about it: the same
/// role gives the same paint, two roles that differ on the wire give different ones, and the hover
/// answer is the theme's rather than the component's.
#[test]
fn a_role_is_the_only_road_to_a_paint() {
    let themes = Themes::standard();
    let theme = themes.theme();
    assert_eq!(theme.paint(Role::Body), theme.paint(Role::Body));
    assert_ne!(theme.paint(Role::Body), theme.paint(Role::Danger));
    assert!(theme.hover_interest() == Interest::HOVER || theme.hover_interest() == Interest::NONE);
    assert!(!theme.glyph(Glyph::VLine).is_empty());
}

// ── 05 · the standard set ────────────────────────────────────────────────────────────────────────

/// **Fourteen shipped schemes, and a slug round-trips.** Ticket 05, spec §15.
#[test]
fn the_standard_set_is_fourteen_schemes_and_a_slug_round_trips() {
    let mut themes = Themes::standard();
    assert_eq!(themes.len(), 14);
    assert_eq!(themes.schemes().len(), 14);

    let last = themes.schemes()[13].slug();
    assert!(themes.select_slug(last));
    assert_eq!(themes.selected(), 13);
    assert_eq!(themes.scheme().slug(), last);
    assert!(!themes.select_slug("no-such-scheme"));

    themes.set_density(Density::Compact);
    assert_eq!(themes.theme().density(), Density::Compact);
}

// ── 06 · animation ───────────────────────────────────────────────────────────────────────────────

/// **Every easing is pinned at both ends**, which is the property that makes a closed form a
/// substitute for an animation object. Ticket 06, spec §16.
#[test]
fn every_easing_is_pinned_at_both_ends() {
    for ease in [Easing::Linear, Easing::In, Easing::Out, Easing::InOut] {
        assert!(ease.at(0.0).abs() < 1e-6, "{ease:?} does not start at 0");
        assert!(
            (ease.at(1.0) - 1.0).abs() < 1e-6,
            "{ease:?} does not end at 1"
        );
    }
}

/// **A tween is the component's own state and the runtime holds none of it.** Ticket 06, spec §16.
#[test]
fn a_tween_is_a_closed_form_over_a_clock_it_does_not_own() {
    let t0 = std::time::Instant::now();
    let span = std::time::Duration::from_millis(200);
    let tween = Tween::new(t0, span, 0i32, 100i32).eased(Easing::Linear);
    assert_eq!(tween.value(t0), 0);
    assert_eq!(tween.value(t0 + span), 100);
    assert!(tween.done(t0 + span));
    assert_eq!(tween.wake(t0 + span), None);

    let steps = Steps::new(t0, std::time::Duration::from_millis(80));
    assert_eq!(steps.index(t0), 0);
    assert_eq!(steps.index(t0 + std::time::Duration::from_millis(160)), 2);

    let spring = Spring::new(t0, 0.0, 1.0);
    assert!(spring.value(t0).abs() < 1e-6);
    assert!(spring.settled(t0 + std::time::Duration::from_secs(4), 0.01));
}

// ── 08 and 09 · the frame and identity ───────────────────────────────────────────────────────────

/// **A frame runs from a crate that cannot name the engine**, and the id table holds exactly what
/// drew. Tickets 08 and 09, spec §1 and §5; ADR 0012 and 0013.
///
/// # The first thing a consumer gets wrong, and the surface is right about it
///
/// `Ctx::id()` is `#[track_caller]`, so **three lanes drawn from one call site are one id**: the
/// naive loop below lands `live() == 1` and two merges rather than three stops, and the keyed loop
/// beside it is what a container owes. Spec §5 calls duplicate detection part of the design rather
/// than a debug aid, and this is what that reads like from outside — the count is here so that the
/// wrong shape stays visibly wrong.
#[test]
fn a_frame_runs_and_the_id_table_holds_what_drew() {
    let mut driver = Driver::headless(W, H).expect("a sink cannot fail to attach");
    driver.frame(|cx| {
        let band = cx.area();
        let lanes = Row::new().split(band, [Weight(1), Weight(1), Weight(1)]);
        for lane in lanes {
            let id = cx.id();
            let _ = cx.interact(id, lane, Interest::CLICK.with(Interest::FOCUS));
        }
    });
    assert_eq!(
        (driver.inspect().ids().live(), driver.inspect().stop_count()),
        (1, 1),
        "one call site is one id, however many times the loop turns"
    );
    assert_eq!(driver.inspect().ids().merges(), 2);

    driver.frame(|cx| {
        let band = cx.area();
        let lanes = Row::new().split(band, [Weight(1), Weight(1), Weight(1)]);
        for (i, lane) in lanes.into_iter().enumerate() {
            cx.with_key(i as u64, |cx| {
                let id = cx.id();
                let _ = cx.interact(id, lane, Interest::CLICK.with(Interest::FOCUS));
            });
        }
    });
    // **Read after the draw, not inside it.** The five frame structures are `Frame`'s and a `Ctx`
    // does not carry them: `Driver::inspect` is the door, which is a fact about the surface that
    // only a consumer finds — a gate inside the crate reaches the fields.
    assert_eq!(
        (driver.inspect().ids().live(), driver.inspect().stop_count()),
        (3, 3)
    );

    // A third frame draws nothing, and nothing survives it: what the runtime keeps is rebuilt from
    // the draw rather than retained (ADR 0012).
    driver.frame(|_cx| {});
    assert_eq!(
        (driver.inspect().ids().live(), driver.inspect().stop_count()),
        (0, 0)
    );
    assert_eq!(driver.inspect().frames(), 3);
}

/// **A key is a `u64` and a call site, and neither needs the engine.** Ticket 09, spec §5.
#[test]
fn a_keyed_id_is_stable_and_a_named_one_is_a_constant() {
    let a = vitui_runtime::Id::named("sidebar");
    let b = vitui_runtime::Id::named("sidebar");
    let c = vitui_runtime::Id::named("detail");
    assert_eq!(a, b);
    assert_ne!(a, c);
    assert_eq!(
        vitui_runtime::Id::keyed(a, 7),
        vitui_runtime::Id::keyed(a, 7)
    );
    assert_ne!(
        vitui_runtime::Id::keyed(a, 7),
        vitui_runtime::Id::keyed(a, 8)
    );
}

// ── 12 · focus, as far as it reaches without a keyboard ──────────────────────────────────────────

/// **`Interest::FOCUS` declares a tab stop and costs no tracking.** Ticket 12, spec §8.
///
/// The ring is reachable from here; `Tab` is not, because it is a posted key.
#[test]
fn a_focus_bit_declares_a_stop_and_asks_for_no_tracking() {
    let mut driver = Driver::headless(W, H).expect("a sink cannot fail to attach");
    driver.frame(|cx| {
        let band = cx.area();
        let [left, right] = Row::new().split(band, [Weight(1), Weight(1)]);
        let a = cx.id();
        let _ = cx.interact(a, left, Interest::FOCUS);
        let b = cx.id();
        let _ = cx.interact(b, right, Interest::FOCUS);
    });
    let frame = driver.inspect();
    assert_eq!(frame.stop_count(), 2);
    assert_eq!(frame.ring().len(), 2);
}

// ── 13 · overlays ────────────────────────────────────────────────────────────────────────────────

/// **Placement flips to the opposite side when the bound refuses, and never leaves the bound.**
/// Ticket 13, spec §10.
#[test]
fn placement_flips_when_the_bound_refuses_and_stays_inside_it() {
    let mut driver = Driver::headless(W, H).expect("a sink cannot fail to attach");
    driver.frame(|cx| {
        let bounds = cx.area();
        let [top, _mid, bottom] = Col::new().split(bounds, [Fixed(4), Weight(1), Fixed(4)]);

        let below = place(top, (20, 6), bounds, Placement::BELOW);
        assert!(below.y >= top.y + i32::from(top.h));

        let flipped = place(bottom, (20, 6), bounds, Placement::BELOW);
        assert!(
            flipped.y + i32::from(flipped.h) <= i32::from(bounds.h),
            "an overlay under the last row must flip above it"
        );

        for side in [Side::Below, Side::Above, Side::Left, Side::Right] {
            for align in [Align::Start, Align::Center, Align::End] {
                let r = place(top, (24, 5), bounds, Placement::new(side, align));
                assert!(r.x >= 0 && r.y >= 0, "{side:?}/{align:?} left the bound");
                assert!(
                    r.x + i32::from(r.w) <= i32::from(bounds.w)
                        && r.y + i32::from(r.h) <= i32::from(bounds.h),
                    "{side:?}/{align:?} left the bound"
                );
            }
        }
    });
}

// ── 14 · scrolling ───────────────────────────────────────────────────────────────────────────────

/// **Four direction bits refuse what a pair of axis bools admits.** Ticket 14, spec §13; ADR 0015.
///
/// A list at its bottom can still go up, so an axis matcher answers *yes, vertically* and swallows
/// the notch. The bits answer per direction and hand it to the enclosing area.
#[test]
fn four_direction_bits_refuse_what_an_axis_pair_admits() {
    let at_bottom = Scrollable::between((0, 100), (0, 100));
    assert!(at_bottom.contains(Scrollable::UP));
    assert!(!at_bottom.contains(Scrollable::DOWN));
    assert!(
        !at_bottom.admits((0, 1)),
        "a full list must refuse another notch down"
    );
    assert!(at_bottom.admits((0, -1)));

    let axis_would_admit = |d: (i32, i32)| !at_bottom.is_empty() && d.1 != 0;
    let mut differ = 0u32;
    for dx in -1i32..=1 {
        for dy in -1i32..=1 {
            if axis_would_admit((dx, dy)) != at_bottom.admits((dx, dy)) {
                differ += 1;
            }
            assert!(
                !(at_bottom.admits((dx, dy)) && !axis_would_admit((dx, dy))),
                "the bits are never looser than the axis pair"
            );
        }
    }
    assert_eq!(
        differ, 3,
        "the two disagree on exactly the three downward deltas"
    );
}

// ── 15 · sizing ──────────────────────────────────────────────────────────────────────────────────

/// **A sizing function is a shape, and `check` is the equality per width every component owes.**
/// Ticket 15, spec §12; ADR 0014.
#[test]
fn a_sizing_function_agrees_with_its_draw_at_every_width() {
    let label = "a moderately long label";
    let agreement = sizing::check(
        [4u16, 8, 12, 24, 48, 300],
        |w| u16::try_from(text::wrap_height(label, w)).unwrap_or(u16::MAX),
        |cx: &mut Ctx<'_, '_>| {
            let band = cx.area();
            let body = cx.theme().paint(Role::Body);
            for (y, line) in text::wrap(label, band.w).enumerate() {
                cx.text(0, i32::try_from(y).unwrap_or(i32::MAX), line, body);
            }
        },
    );
    agreement.assert();
    assert!(agreement.agrees());
}

// ── 16 · async work ──────────────────────────────────────────────────────────────────────────────

/// **A stale landing cannot displace a fresh one**, which is what the eight bytes of `Generation`
/// are for. Ticket 16, spec §17.
#[test]
fn a_stale_landing_cannot_displace_a_fresh_one() {
    let worker = Worker::queueing();
    let task: Task<u32> = Task::new(&worker);
    let _ = task.request(1, |_cancel| 10u32);
    let first = task.generation().expect("a request mints a generation");
    let _ = task.request(2, |_cancel| 20u32);
    let second = task.generation().expect("a request mints a generation");
    assert_eq!(task.asking(), Some(2));
    assert_ne!(first, second, "two questions are two generations");

    // Run the second question and then the first: the older answer must not land on the newer.
    assert!(worker.run(1));
    assert!(!worker.run(0), "a superseded question does not land");
    assert_eq!(task.take(), Some(20));
    assert_eq!(task.take(), None);
    assert!(second.raw() > first.raw());
}

/// **A drain takes a batch without a worker at all.** Ticket 16, spec §17.
#[test]
fn a_drain_takes_a_batch_without_a_worker() {
    let drain: Drain<u32> = Drain::new();
    drain.extend([1u32, 2, 3]);
    drain.push(4);
    let mut out = Vec::new();
    assert_eq!(drain.drain_into(&mut out), 4);
    assert_eq!(out, vec![1, 2, 3, 4]);
    assert_eq!(drain.drain_into(&mut out), 0);
}
