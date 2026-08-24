//! **Runtime ticket 11's report**: what routing costs, and what the two shapes it replaced cost.
//!
//! ```text
//! cargo run --release --example routing_numbers -p vitui-runtime
//! ```
//!
//! Three numbers the ticket asks for — **routing a realistic batch at 221 ns, 0.221%**, **folding at
//! 5.0 ns an event**, and **bubbling at 2.99 ns a key a level** — and one it asks for as a contrast:
//! the drain written the obvious way, which is **11.75 ms against 53.79 µs** on an eight-thousand
//! key paste.
//!
//! That last pair is the reason the gate in `crate::route` is a **growth ratio** and not a budget.
//! 11.75 ms is 117× the frame budget and would be caught by anything — but it is 8 000 keys, and
//! nobody pastes 8 000 keys into a test. At the sizes a test actually uses, the quadratic form is
//! comfortably inside 100 µs and looks fine. **It is a slope, not a cliff.**

use std::hint::black_box;
use std::time::Instant;

use vitui_bench::Bench;
use vitui_engine::{
    Button, Buttons, Event, Key, KeyCode, KeyKind, KeyText, Mods, Mouse, MouseKind, Rect,
};
use vitui_runtime::ctx::{Ctx, Driver, Interest};
use vitui_runtime::focus::ScopeKind;
use vitui_runtime::id::Id;
use vitui_runtime::route;

/// The frame budget every ratio is against.
const FRAME_NS: f64 = 100_000.0;

/// The spec's dense screen.
const DENSE: u64 = 312;

/// The paste the quadratic hides in.
const PASTE: u32 = 8_000;

fn key(code: KeyCode) -> Key {
    Key {
        code,
        mods: Mods::NONE,
        kind: KeyKind::Press,
        text: KeyText::EMPTY,
        at: Instant::now(),
    }
}

fn mouse(kind: MouseKind) -> Mouse {
    Mouse {
        x: 4,
        y: 2,
        kind,
        buttons: Buttons::NONE,
        mods: Mods::NONE,
        at: Instant::now(),
    }
}

/// **A realistic batch**: what arrives while somebody types a word and then clicks.
///
/// Twelve keys, four pointer moves and a press. The press is the routing edge, so this is one
/// frame's worth — and choosing a batch with exactly one edge in it is not a convenience, it is what
/// the rule guarantees.
fn realistic_batch() -> Vec<Event> {
    let mut batch: Vec<Event> = "hello, world"
        .chars()
        .map(|c| Event::Key(key(KeyCode::Char(c))))
        .collect();
    for _ in 0..4 {
        batch.push(Event::Mouse(mouse(MouseKind::Move)));
    }
    batch.push(Event::Mouse(mouse(MouseKind::Down(Button::Left))));
    batch
}

fn main() {
    println!("runtime ticket 11 — what routing costs\n");
    the_split();
    routing_a_realistic_batch();
    folding();
    bubbling();
    the_drain_written_the_obvious_way();
}

/// Report: the batch splitter itself, which is a scan over a slice and nothing else.
fn the_split() {
    let batch = realistic_batch();
    let report = Bench::new(40)
        .case("split a realistic batch", 2_000, || {
            black_box(route::batch_len(black_box(&batch)));
        })
        .run();
    let ns = report.get("split a realistic batch").expect("measured");
    println!("report  the batch splitter:\n{report}");
    println!(
        "        {} events classified in {ns:.2} ns, {:.3} ns an event. It is a scan that stops at\n\
        \x20       the first edge, so a batch whose edge is at the front costs one classification.\n",
        batch.len(),
        ns / batch.len() as f64
    );
}

/// Report: **routing a realistic batch on a realistic screen — the ticket's 221 ns.**
///
/// Measured as a difference, because routing does not have a frame of its own: the same dense screen
/// drawn with the batch and without it, and the gap is what the batch cost.
fn routing_a_realistic_batch() {
    let focused = Id::keyed(Id::ROOT, 7);
    let dense = move |cx: &mut Ctx<'_, '_>| {
        let interest = Interest::CLICK.with(Interest::FOCUS);
        for i in 0..DENSE {
            let y = i32::try_from(i % 80).unwrap_or(0);
            let x = i32::try_from((i / 80) * 40).unwrap_or(0);
            cx.interact(Id::keyed(Id::ROOT, i), Rect::new(x, y, 40, 1), interest);
        }
        while cx.next_key(focused).is_some() {}
    };

    let quiet = {
        let mut driver = Driver::headless(300, 80).expect("sink");
        driver.plant(None, Some(focused), None);
        let report = Bench::new(40)
            .case("a dense frame, nothing queued", 20, || {
                black_box(&mut driver).frame(dense);
            })
            .run();
        report
            .get("a dense frame, nothing queued")
            .expect("measured")
    };

    let busy = {
        let mut driver = Driver::headless(300, 80).expect("sink");
        driver.plant(None, Some(focused), None);
        let batch = realistic_batch();
        let report = Bench::new(40)
            .case("a dense frame, a realistic batch", 20, || {
                let d = black_box(&mut driver);
                for event in &batch {
                    match event {
                        Event::Key(k) => d.post_key(*k),
                        Event::Mouse(m) => d.post_mouse(*m),
                        _ => {}
                    }
                }
                // One frame: the batch has exactly one edge and it is at the end.
                d.frame(dense);
                // `assert_eq!`, not `debug_assert_eq!`: the module doc runs this example in
                // release, where a debug assertion is compiled out — and the one-frame premise is
                // what the 221 ns figure means.
                assert_eq!(d.queued(), 0, "the whole batch went in one frame");
            })
            .run();
        report
            .get("a dense frame, a realistic batch")
            .expect("measured")
    };

    let routing = busy - quiet;
    println!(
        "report  routing on the dense screen:\n\
        \x20       quiet frame            {quiet:>10.2} ns\n\
        \x20       frame with a batch     {busy:>10.2} ns\n\
        \x20       routing                {routing:>10.2} ns   {:.3}% of a {:.0} us budget\n\
        \x20       spec §7 measured **221 ns, 0.221%**. Posting the batch is in this figure and is\n\
        \x20       not free, which is why the difference is quoted rather than the drain alone.\n",
        routing / FRAME_NS * 100.0,
        FRAME_NS / 1000.0
    );
}

/// Report: **folding — 5.0 ns an event.**
///
/// Eight thousand ordinary keys are one frame, because not one of them is a routing edge. This is
/// the direction "one event a frame" gets wrong that nobody notices until somebody pastes.
fn folding() {
    let editor = Id::from_raw(1);
    let mut driver = Driver::headless(300, 80).expect("sink");
    driver.plant(None, Some(editor), None);

    // **Built once, outside the measurement.** `Instant::now()` is ~20 ns on this machine and a
    // `Key` carries one, so constructing the paste inside the timed loop would have measured the
    // clock five times over for every nanosecond of routing.
    let paste: Vec<Key> = (0..PASTE).map(|_| key(KeyCode::Char('x'))).collect();
    let report = Bench::new(20)
        .case("fold an eight-thousand key paste", 1, || {
            let d = black_box(&mut driver);
            for k in black_box(&paste) {
                d.post_key(*k);
            }
            d.frame(|cx| {
                cx.interact(editor, Rect::new(0, 0, 40, 1), Interest::FOCUS);
                while cx.next_key(editor).is_some() {}
            });
            assert_eq!(d.queued(), 0, "a paste is one frame");
        })
        .run();
    let ns = report
        .get("fold an eight-thousand key paste")
        .expect("measured");
    println!("report  folding:\n{report}");
    println!(
        "        {PASTE} keys posted, routed and drained in {:.2} us — **{:.2} ns an event**, and\n\
        \x20       ONE frame. ADR 0016 quotes 5.0 ns for the fold itself; this figure also carries\n\
        \x20       `post_key` and the copy into the queue, which is two moves of a 48-byte `Key`.\n\
        \x20       Under \"one event a frame\" the same paste would be {PASTE} frames, which at the\n\
        \x20       100 us budget is {:.1} seconds of nothing else.\n",
        ns / 1000.0,
        ns / f64::from(PASTE),
        f64::from(PASTE) * FRAME_NS / 1e9
    );
}

/// Report: **bubbling — 2.99 ns a key a level.**
///
/// One key, posted per frame, declined by the focused field and taken and declined again by every
/// enclosing scope on its way out. The id path on a realistic screen is 1 at the top and 2 inside a
/// keyed row, so the number worth having is the per-level slope rather than any particular depth.
///
/// **One key a frame, and that is not a shortcut.** A decline ends a level's turn at the queue —
/// it must, or the obvious drain loop hands the same key out for ever — so a level that declines
/// sends the whole tail of the batch outward with the key it declined. Posting a batch and dividing
/// by its length would divide by a number the work does not depend on, and understate the figure by
/// exactly that factor.
///
/// **A double difference, because a level is two costs and only one of them is bubbling.** Opening
/// a scope builds a child `Ctx`, claims an id and moves the routing target; carrying a key out
/// through it is a take and a decline. The same nest is measured with a key and with none, at two
/// depths, so the four numbers separate them.
fn bubbling() {
    const FRAMES: u32 = 64;
    let field = Id::from_raw(1);

    // A body nested `levels` deep, where every level is offered whatever is queued and hands it back.
    fn nest(cx: &mut Ctx<'_, '_>, level: u64, levels: u64, field: Id) {
        if level == levels {
            cx.interact(field, Rect::new(0, 0, 8, 1), Interest::FOCUS);
            // The `while let` shape, which terminates because a decline closes the queue to this
            // level. Written this way here deliberately: it is the shape an author reaches for.
            while let Some(k) = cx.next_key(field) {
                cx.decline(k);
            }
            return;
        }
        let id = Id::from_raw(100 + level);
        cx.scope(id, ScopeKind::Group, |cx| {
            nest(cx, level + 1, levels, field)
        });
        // The after-the-body moment: take what the level below declined, and decline it again so
        // the level above this one has something to take.
        while let Some(k) = cx.next_key(id) {
            cx.decline(k);
        }
    }

    let cost = |levels: u64, keyed: bool| {
        let mut driver = Driver::headless(300, 80).expect("sink");
        driver.plant(None, Some(field), None);
        let name = "a nest of scopes";
        let report = Bench::new(40)
            .case(name, FRAMES, || {
                let d = black_box(&mut driver);
                if keyed {
                    d.post_key(key(KeyCode::Char('x')));
                }
                d.frame(|cx| nest(cx, 0, levels, field));
                assert_eq!(d.queued(), 0, "one key is one frame");
            })
            .run();
        report.get(name).expect("measured")
    };

    let (shallow_bare, shallow_keyed) = (cost(1, false), cost(1, true));
    let (deep_bare, deep_keyed) = (cost(9, false), cost(9, true));

    // Per level: the scope itself, and — as a double difference, so that posting the key and
    // beginning the frame cancel — the key's journey through one level.
    let scope_per_level = (deep_bare - shallow_bare) / 8.0;
    let key_per_level = ((deep_keyed - deep_bare) - (shallow_keyed - shallow_bare)) / 8.0;
    println!(
        "report  bubbling:\n\
        \x20       1 level,  no key   {shallow_bare:>10.2} ns      1 level,  one key   {shallow_keyed:>10.2} ns\n\
        \x20       9 levels, no key   {deep_bare:>10.2} ns      9 levels, one key   {deep_keyed:>10.2} ns\n\
        \x20       opening a scope, a level                        {scope_per_level:>10.2} ns\n\
        \x20       **bubbling, a key a level**                     {key_per_level:>10.2} ns\n\
        \x20       The second figure is the one spec §7 quotes at **2.99 ns a key a level**, and it\n\
        \x20       is a take, a decline and a re-open against one cursor. Measured here it is about\n\
        \x20       {:.1}x that, and the reason is that a decline now also closes the queue to the level\n\
        \x20       that made it and the scope re-opens it — a branch and two writes the estimate did\n\
        \x20       not have, and the alternative to it was a drain loop that never terminates.\n\
        \x20       The first figure is the scope's own — a child `Ctx`, a claim and a routing target\n\
        \x20       — which a container pays whether or not a key ever reaches it. Both are reports;\n\
        \x20       neither is a gate, and at 43 tab stops the whole of it is under a microsecond.\n",
        key_per_level / 2.99
    );
}

/// Report: **the drain written the obvious way — 11.75 ms against 53.79 µs.**
///
/// `Vec::remove(0)` shifts the tail on every key. Both halves are timed here over the same keys so
/// that the ratio is the point rather than either number.
fn the_drain_written_the_obvious_way() {
    let keys: Vec<Key> = (0..PASTE).map(|_| key(KeyCode::Char('x'))).collect();

    // **Both halves clone the same input**, so the clone is in both figures and the difference is
    // the drain and nothing else.
    let linear_src = keys.clone();
    let quadratic_src = keys;

    let report = Bench::new(8)
        .case("the cursor", 1, || {
            let mut src = black_box(&linear_src).clone();
            let mut at = 0usize;
            while let Some(k) = src.get(at) {
                black_box(k);
                at += 1;
            }
            black_box(&mut src);
        })
        .case("Vec::remove(0)", 1, || {
            let mut src = black_box(&quadratic_src).clone();
            while !src.is_empty() {
                black_box(src.remove(0));
            }
        })
        .run();

    let linear = report.get("the cursor").expect("measured");
    let quadratic = report.get("Vec::remove(0)").expect("measured");
    println!("report  draining {PASTE} keys:\n{report}");
    println!(
        "        the cursor       {:>10.2} us\n\
        \x20       Vec::remove(0)   {:>10.2} us   {:.1}x\n\
        \x20       spec §7 measured **11.75 ms against 53.79 us**. Both halves clone the same input,\n\
        \x20       so the clone is in both figures and the gap is the drain — the shape is the point,\n\
        \x20       and the shape is a slope: at the sizes a test uses, the quadratic form\n\
        \x20       is well inside the 100 us budget and looks fine. That is why the gate in\n\
        \x20       `vitui_runtime::route` is a growth RATIO over a slot count.\n",
        linear / 1000.0,
        quadratic / 1000.0,
        quadratic / linear
    );
}
