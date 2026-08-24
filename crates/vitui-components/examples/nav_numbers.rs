//! components ticket 08 — `nav::cursor`, as numbers.
//!
//! A **report**, not a gate: R15's refinement 2 says a row may cite one and may never rest on one,
//! and `cargo test` does not run a `fn main`. What gates every number below is `src/nav.rs`'s test
//! module, and each figure has its single home in that file's ledger block.
//!
//! It prints four things:
//!
//! 1. **The tab-stop count**, in two columns — this crate's screen and spec §3's.
//! 2. **Why the second column is not reproduced**, which is arithmetic and not an excuse.
//! 3. **The type-ahead deadline**, and the census that tells the two verbs apart.
//! 4. **What a `Group` does not do**, which is the half a walk-only reading cannot see.

use std::time::{Duration, Instant};

use vitui_components::nav::{
    self, CHROME, Cursor, PANELS, RING_ENTRIES, ROWS_MIN, SPEC_GROUPED, SPEC_UNGROUPED,
    STOPS_GROUPED, STOPS_UNGROUPED, TypeAhead, WINDOW, stops,
};
use vitui_runtime::Id;
use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{Chord, Code};

const LABELS: [&str; 6] = ["ant", "abbot", "bee", "Cormorant", "dove", "eagle"];

fn main() {
    println!("components ticket 08 — a list is one tab stop\n");

    // ── 1. the tab-stop count ────────────────────────────────────────────────────────────────────
    let loose = stops(false);
    let grouped = stops(true);
    println!("report  tab stops, in two columns:");
    println!(
        "  {:<26}  {:>12}  {:>12}  {:>7}",
        "screen", "no Group", "with Group", "ratio"
    );
    for (name, ungrouped, gathered) in [
        ("this crate's fixture", loose.stops, grouped.stops),
        ("spec §3 / C02's gallery", SPEC_UNGROUPED, SPEC_GROUPED),
    ] {
        println!(
            "  {name:<26}  {ungrouped:>12}  {gathered:>12}  {:>7.2}",
            ungrouped as f64 / gathered as f64
        );
    }
    let per_panel = CHROME + 1;
    println!(
        "\n  The fixture is {PANELS} panels; panel *i* holds a header, a collection of {ROWS_MIN} + i\n  \
         rows and a footer. With each collection in its own `Group` the walk is exactly\n  \
         {per_panel} a panel — header, collection, footer — **whatever any collection holds**,\n  \
         which is the claim §3's sentence makes and the one the gate asserts.\n"
    );

    // ── 2. why the second column is not reproduced ───────────────────────────────────────────────
    println!("report  why 266 → 69 is recorded and not reproduced:");
    for line in [
        "  266 = c·r + s  and  69 = c + s   forces  c·(r − 1) = 197,  and 197 is prime.",
        "  So the only *uniform* screen yielding the pair is one collection of 198 rows beside",
        "  68 loose focusables — which is not a gallery and not anything else. C02's screen was",
        "  heterogeneous, none of the components on it is declared in this crate, and it is not",
        "  recoverable from the two numbers it left behind.",
        "",
        "  What reproduces is the **identity**: a collection is one tab stop. The magnitude is a",
        "  property of a screen this ticket does not own, and a fixture aimed at a remembered",
        "  number is a fixture that has stopped measuring anything.",
    ] {
        println!("{line}");
    }
    println!();

    // ── 3. the type-ahead deadline ───────────────────────────────────────────────────────────────
    let id = Id::named("collection");
    let t0 = Instant::now();
    println!(
        "report  a type-ahead buffer over {:?}, window {} ms:",
        LABELS,
        WINDOW.as_millis()
    );
    println!(
        "  {:<10}  {:>8}  {:<8}  {:>7}  {:>10}",
        "arm", "gap ms", "buffer", "hit", "asked_by"
    );
    for (name, gap) in [("armed", 400u64), ("lapsed", 1500)] {
        let mut driver = Driver::headless(60, 8).expect("a sink cannot fail to attach");
        let mut ahead = TypeAhead::new();
        driver.pin_clock(t0);
        let mut hit = None;
        driver.frame(|cx| {
            hit = nav::seek(
                cx,
                id,
                &mut ahead,
                &vitui_components::keys::press(Chord::key('a')),
                &LABELS,
            );
        });
        driver.advance(Duration::from_millis(gap));
        driver.frame(|cx| {
            hit = nav::seek(
                cx,
                id,
                &mut ahead,
                &vitui_components::keys::press(Chord::key('b')),
                &LABELS,
            );
        });
        let quoted = format!("{:?}", ahead.buffer());
        println!(
            "  {name:<10}  {gap:>8}  {quoted:<8}  {:>7}  {:>10}",
            hit.map(|i| LABELS[i]).unwrap_or("-"),
            driver.inspect().wakes().asked_by(id)
        );
    }
    println!(
        "\n  Inside the window the second key **refined** the search — `ab` is abbot and not bee.\n  \
         Past it the buffer had lapsed and `b` started afresh. Both arms arm one deadline a key,\n  \
         which is what makes the lapse happen at the deadline instead of at the next keypress.\n"
    );

    println!("report  the two verbs, on fresh drivers:");
    println!(
        "  {:<16}  {:>10}  {:>12}  {:>12}",
        "verb", "asked_by", "census_len", "line_count"
    );
    for (name, uses_id) in [("deadline_for", true), ("deadline", false)] {
        let mut driver = Driver::headless(20, 3).expect("a sink cannot fail to attach");
        let at = t0 + WINDOW;
        driver.frame(|cx| match uses_id {
            true => cx.deadline_for(id, at),
            false => cx.deadline(at),
        });
        let wakes = driver.inspect().wakes();
        println!(
            "  {name:<16}  {:>10}  {:>12}  {:>12}",
            wakes.asked_by(id),
            wakes.census_len(),
            wakes.line_count()
        );
    }
    println!(
        "\n  **The `id` is the census and not the attribution.** Runtime 06 argues that a deadline\n  \
         is attributed to its call site, which is true of both verbs — `line_count` is 1 either\n  \
         way. What the `id` buys is `asked_by`, the counter that proves which widget asked, and a\n  \
         collection has an id to count against. Ticket 22's tween has no widget and takes the\n  \
         other half. This is the ticket's own correction to itself, as two numbers.\n"
    );

    // ── 4. what a Group does not do ──────────────────────────────────────────────────────────────
    println!("report  what a `Group` collapses, and what it keeps:");
    println!(
        "  {:<14}  {:>8}  {:>8}  {:>8}",
        "arm", "ring", "stops", "walk"
    );
    println!(
        "  {:<14}  {:>8}  {:>8}  {:>8}",
        "no Group", loose.ring, loose.stops, loose.walk
    );
    println!(
        "  {:<14}  {:>8}  {:>8}  {:>8}",
        "with Group", grouped.ring, grouped.stops, grouped.walk
    );
    println!(
        "\n  The ring is **unchanged**, which is what makes the helper coherent rather than a way\n  \
         of hiding widgets: `Tab` reaches the list once, and once inside it `nav::cursor`'s arrows\n  \
         move over {} entries the walk will never stop on. A reading that took only the walk would\n  \
         report a screen whose contents had disappeared.\n",
        grouped.ring - grouped.walk
    );

    // A cursor over the same ring, so the arrows are not only asserted in a test module.
    let cur = Cursor::new(grouped.ring, 10);
    println!(
        "report  `nav::cursor`'s arithmetic over those {} entries:",
        cur.len
    );
    for (code, from) in [
        (Code::Down, 0),
        (Code::PageDown, 0),
        (Code::End, 0),
        (Code::PageUp, cur.last()),
        (Code::Home, cur.last()),
        (Code::Up, 0),
    ] {
        let k = vitui_components::keys::press(Chord::new(code));
        println!(
            "  {:<10}  {from:>4} → {:>4}",
            format!("{code:?}"),
            nav::step(&k, cur.to(from)).expect("a cursor key")
        );
    }
    let ctrl_home = vitui_components::keys::press(Chord::new(Code::Home).ctrl());
    println!(
        "  {:<10}  declined — a chord on a named key is the application's, and it is the same\n  \
         {:<10}  predicate a field declines `Ctrl+S` with",
        "Ctrl+Home", ""
    );
    assert_eq!(nav::step(&ctrl_home, cur), None);

    // The **shape**, so that a report which has quietly started measuring something smaller fails
    // rather than looking good.
    assert_eq!(loose.stops, STOPS_UNGROUPED);
    assert_eq!(grouped.stops, STOPS_GROUPED);
    assert_eq!(loose.ring, RING_ENTRIES);
    assert_eq!(grouped.ring, RING_ENTRIES);
    assert_eq!(grouped.walk, PANELS * (CHROME + 1));
}
