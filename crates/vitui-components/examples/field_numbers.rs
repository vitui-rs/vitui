//! **The field: the cluster corpus, the caret, the wrap index and the resize, as numbers.**
//!
//! Components ticket 23. The convention is the runtime's — a file in `examples/` named
//! `<subject>_numbers.rs` that prints the numbers a human reads — and so is the rule about what an
//! example may be: **`cargo test` does not run this file**, and no row of
//! [`vitui_components::gates::REGISTER`] rests on it. The rows that cite it each name a `#[test]` in
//! `src/document.rs` or `src/clusters.rs` beside the citation.
//!
//! # What it prints
//!
//! 1. **The two scenes**, with where each stands and which ticket inverts them.
//! 2. **The corpus**, kind by kind, with §11's remembered figures in the column beside.
//! 3. **The `char` caret**, which is the negative case and is *faster per step*.
//! 4. **`Left` at the end of a pasted megabyte**, both ways.
//! 5. **The wrap index**: 625 against 875, the recompute count that points the wrong way, and the
//!    five hundred splices.
//! 6. **The four gates**, each with the cells it moves and the counters that can see it — which is
//!    the ticket's headline, because the answer is *none*.
//! 7. **What does not reproduce**, said out loud rather than engineered away.
//!
//! # It asserts the shape and not the timings
//!
//! R15's rule, inherited: *a gate is a count, a ratio, an equality or a compile outcome; a timing is
//! a report*. Every `assert!` below is a count or an equality, and every one of them is asserted in
//! `src/` as well — an example is compiled and never evaluated, which is the defect components
//! ticket 08 found inside the register's own report.

use std::time::{Duration, Instant};

use vitui_alloc_probe::{CountingAllocator, count_allocations};
use vitui_components::clusters::{self, Kind};
use vitui_components::counters::{Allocations, Counter};
use vitui_components::document::{self, Caret, Defect, Index};
use vitui_components::gates::Standing;
use vitui_components::scenes::{SCENES, scenes_for};

// **The probe, because `allocations` is a total and a total needs something that counts.**
#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

fn main() {
    println!(
        "The document — {} lines, {} visual rows at {} columns and {} at {}, every line built out \
         of a cluster corpus\n",
        document::LINES,
        document::LINES,
        document::WIDE,
        document::WRAPPED,
        document::NARROW
    );

    scene_list();
    the_corpus();
    the_char_caret();
    left_at_a_megabyte();
    the_wrap_index();
    the_four_gates();
    the_component();
    the_ring();
    the_frame();
    what_does_not_reproduce();
}

/// 8. **The component, and the two counts §11 states about it.**
///
/// Components ticket 24. `field` exists now, so the two figures §11 gives for the widget itself can
/// be measured rather than argued: the regions it declares against the per-cluster spelling, and
/// the `Left` at the end of a pasted megabyte with the index the wrapping already needs.
fn the_component() {
    use vitui_components::counters::Tally;
    use vitui_components::edit::{Text, WrapKind, defective as bad};
    use vitui_components::input::{FieldOpts, defective::Regions, defective::field_regions};
    use vitui_runtime::Rect;
    use vitui_runtime::ctx::Driver;

    println!("report  the component — §11's *79 regions against 621*, over this crate's screen:");
    let text = clusters::corpus().text().repeat(4);
    let (w, h) = (60u16, 8u16);
    let area = Rect::new(0, 0, w, h);
    let opts = FieldOpts::default();
    let declared = |regions: Regions| {
        let mut st = Text::of(text.clone(), WrapKind::Words);
        let mut tally = Tally::new();
        let mut driver = Driver::headless(w, h).expect("a sink attaches");
        driver.frame(|cx| {
            field_regions(&mut tally, cx, area, &mut st, &opts, regions);
        });
        (driver.inspect().hits().len(), tally.writes(), tally.verbs())
    };
    let (one, one_writes, one_verbs) = declared(Regions::Widget);
    let (many, many_writes, many_verbs) = declared(Regions::PerCluster);
    println!(
        "  {one} region against {many} for one per visible cluster, on a {w}x{h} field — and the \
         two draw the same screen:"
    );
    println!(
        "    writes {one_writes} against {many_writes}, verbs {one_verbs} against {many_verbs}"
    );
    println!(
        "  §11 states 79 against 621. Both are a prototype's screen; what reproduces is the shape — \
         one entry for the widget,"
    );
    println!("  and hundreds for the spelling that gives each cluster its own.\n");

    println!("report  one `Left` at the end of a pasted megabyte, through the component:");
    let mega = document::pasted();
    let mut with = Text::of(mega.clone(), WrapKind::Words);
    let _ = with.index(document::WIDE);
    let last = with.index(document::WIDE).rows() - 1;
    with.click(document::WIDE, last, u16::MAX, false);
    let end = with.caret();
    let started = Instant::now();
    with.left(document::WIDE, false);
    let indexed = started.elapsed();

    let started = Instant::now();
    let blind = bad::blind_left(&mega, end);
    let without = started.elapsed();
    println!(
        "  {:>12.3} us with an index against {:>12.3} us without one — {:.0}x, and §11 remembers \
         {:.0}x",
        indexed.as_secs_f64() * 1e6,
        without.as_secs_f64() * 1e6,
        without.as_secs_f64() / indexed.as_secs_f64().max(f64::MIN_POSITIVE),
        document::REMEMBERED_LEFT_RATIO
    );
    println!(
        "  and the two agree about where it lands: byte {} either way.\n",
        blind.byte()
    );
}

/// 9. **The undo ring: two bounds, the coalescing ratio, and the entry that is one paste.**
fn the_ring() {
    use vitui_components::edit::{RING_BYTES, RING_ENTRIES, Ring, Text, WrapKind};

    println!("report  the undo ring — bounded twice, and coalescing is an entry count:");
    let sentence = "the quick brown fox jumps over the lazy dog and it keeps going for a while ";
    let mut coalesced =
        Text::of(String::new(), WrapKind::Ruler).with_ring(Ring::bounded(usize::MAX, usize::MAX));
    let mut flat = Text::of(String::new(), WrapKind::Ruler)
        .with_ring(Ring::bounded(usize::MAX, usize::MAX).uncoalesced());
    let mut keystrokes = 0u32;
    for c in sentence.repeat(14).chars() {
        let s = c.to_string();
        coalesced.insert(200, &s);
        flat.insert(200, &s);
        keystrokes += 1;
    }
    println!(
        "  {keystrokes} keystrokes is {} entries coalesced against {} flat — {:.2}x, and §11 \
         remembers 1 012 -> 414, which is 2.44x",
        coalesced.ring().len(),
        flat.ring().len(),
        flat.ring().len() as f64 / coalesced.ring().len().max(1) as f64
    );
    println!(
        "  {} B against {} B. The rule is the **word** break and not the line break: §11 gives the \
         reason in the same sentence as",
        coalesced.ring().bytes(),
        flat.ring().bytes()
    );
    println!(
        "  the ratio — *undoing a sentence is 414 presses instead of 1 012* — and a run closing \
         only at a line makes it one press, which is a"
    );
    println!("  checkpoint and not a history.");

    let mut bounded = Text::of(String::from("start"), WrapKind::Ruler)
        .with_ring(Ring::bounded(16, usize::MAX).uncoalesced());
    bounded.end(40, false);
    for i in 0..64u32 {
        bounded.insert(40, &format!(" {i}"));
    }
    let mut steps = 0;
    while bounded.undo() {
        steps += 1;
    }
    println!(
        "  sixty-four edits through a sixteen-entry ring: {steps} undos each answering yes, and the \
         document is {} — `truncated` is {}",
        if bounded.text() == "start" {
            "back"
        } else {
            "**not** back"
        },
        bounded.ring().truncated()
    );
    println!("  the shipped bounds are {RING_ENTRIES} entries and {RING_BYTES} B.\n");
}

/// 10. **The frame, at a hundred kilobytes and at a megabyte.**
fn the_frame() {
    use vitui_components::counters::{Counter, Counters, Tally};
    use vitui_components::edit::{Text, WrapKind};
    use vitui_components::input::{FieldOpts, field_into};
    use vitui_runtime::{Density, Rect};

    println!("report  the frame — §11's *82.4 us / 19 634 writes / 631 verbs / 79 regions, flat*:");
    println!(
        "  {:<16}  {:>10}  {:>8}  {:>7}  {:>8}  {:>7}",
        "document", "us/frame", "writes", "verbs", "regions", "merges"
    );
    let mega = document::pasted();
    let (w, h) = (document::WIDE, document::H);
    for (name, content) in [
        ("100 kB clusters", mega[..100_000].to_string()),
        ("1 MB clusters", mega.clone()),
        (
            "1 MB ASCII",
            "the quick brown fox jumps over the lazy dog\n".repeat(24_000),
        ),
    ] {
        let mut st = Text::of(content, WrapKind::Words);
        let _ = st.index(w);
        let mut driver = vitui_components::runner::driver_at(w, h, Density::default());
        // **The timing is the shipped path and the counters are the instrument's.** A `Tally`
        // unions a span per verb into a cell set, which is thirty times the frame it is measuring
        // — so a minimum taken over it would be a report about `crate::counters`. `Direct` is what
        // a component gets (`crate::ink`), and it is what is timed.
        let mut best = f64::MAX;
        for _ in 0..8 {
            let started = Instant::now();
            driver.frame(|cx| {
                field_into(
                    &mut vitui_components::ink::Direct,
                    cx,
                    Rect::new(0, 0, w, h),
                    &mut st,
                    &FieldOpts::default(),
                );
            });
            best = best.min(started.elapsed().as_secs_f64() * 1e6);
        }
        let worst = best;
        let mut tally = Tally::new();
        driver.frame(|cx| {
            field_into(
                &mut tally,
                cx,
                Rect::new(0, 0, w, h),
                &mut st,
                &FieldOpts::default(),
            );
        });
        let counters = Counters::of(&driver, &tally, Allocations::over(1, 0));
        println!(
            "  {:<16}  {:>10.2}  {:>8}  {:>7}  {:>8}  {:>7}",
            name,
            worst,
            tally.writes(),
            tally.verbs(),
            counters
                .get(Counter::Regions)
                .measured()
                .map_or_else(|| "-".into(), |v| v.to_string()),
            counters
                .get(Counter::Merges)
                .measured()
                .map_or_else(|| "-".into(), |v| v.to_string()),
        );
    }
    println!(
        "  a minimum of eight, and a **report**: R15. What is gated is that the first two rows are \
         the same frame — the field costs its"
    );
    println!(
        "  visible window and never its content, which is the engine's own invariant arriving \
         on the component that holds the most data"
    );
    println!(
        "  in the freeze. **The third row is why the first two are what they are**: the same \
         rectangle over ASCII is a different frame, because"
    );
    println!(
        "  every cell of the first two is a grapheme cluster and the cost is text measurement over \
         the engine's tables. §11's 82.4 us is a"
    );
    println!("  prototype's screen and this one writes 24 000 cells of the cluster corpus.\n");
}

/// 1. The two scenes, and the one ticket that inverted both.
fn scene_list() {
    println!("report  the two scenes, and the one fact they stand on:");
    println!(
        "  {:>3}  {:<44}  {:<28}  inverted by",
        "#", "scene", "standing"
    );
    let mut red = 0usize;
    for scene in SCENES.iter().filter(|s| s.stands.contains(&"field")) {
        let (word, ticket) = match scene.standing {
            Standing::Red { inverted_by, .. } => {
                red += 1;
                ("red, pinned", inverted_by)
            }
            Standing::Evaluated { .. } => ("evaluated", "-"),
            Standing::Unsubjected { inverted_by } => ("unsubjected", inverted_by),
            Standing::Unreachable { inverted_by, .. } => ("unreachable", inverted_by),
        };
        println!(
            "  {:>3}  {:<44}  {word:<28}  {ticket}",
            scene.number, scene.name
        );
    }
    assert_eq!(
        red, 0,
        "both scenes were pinned red for one ticket and components 24 stood them up, together with \
         the corpus scene they run over"
    );
    // **Four since components 41.** Components 40 stood the assembled gallery up and 41 stood the
    // palette-swap gallery up beside it, and scenes 26 and 27 both carry `STANDS_THE_GALLERY` — so
    // one screen carrying every built row of the freeze answers `scenes_for` **twice** for each of
    // them, through `stands` and never through `covers`.
    //
    // **This number was 3 for five tickets while the answer was 4**, and nothing said so: components
    // ticket 20's finding for the fourth time — `cargo test` does not run an example, so an
    // `assert!` here is compiled by `cargo clippy --all-targets` and evaluated by nobody. Found by
    // components 46 running all thirty of them.
    assert_eq!(scenes_for("field").count(), 4);
    println!(
        "  `scenes_for(\"field\")` answers {} scenes, and `crate::document::standing()` is {:?}\n",
        scenes_for("field").count(),
        document::standing()
    );
}

/// 2. The corpus, kind by kind.
fn the_corpus() {
    let corpus = clusters::corpus();
    println!(
        "report  the cluster corpus — {} whole turns of an eight-word rota:",
        clusters::CYCLES
    );
    println!(
        "  {:<26}  {:>8}  {:>6}  {:>7}  {:>7}",
        "kind", "clusters", "cps", "columns", "spec §11"
    );
    for kind in Kind::ALL {
        let sample = clusters::ROTA
            .iter()
            .find(|w| w.kind == kind)
            .map(|w| w.clusters.concat())
            .unwrap_or_default();
        println!(
            "  {:<26}  {:>8}  {:>6}  {:>7}  {:>7}",
            kind.word(),
            corpus.count_of(kind),
            sample.chars().count(),
            vitui_runtime::layout::text::width(&sample),
            if kind.named_by_spec() {
                "named"
            } else {
                "ours"
            }
        );
    }
    println!(
        "  totals: {} clusters (§11: {}), {} code points (§11: {}), {} landing inside a cluster \
         (§11: {}), {} columns",
        corpus.len(),
        document::REMEMBERED_CLUSTERS,
        corpus.chars(),
        document::REMEMBERED_STEPS,
        corpus.inside(),
        document::REMEMBERED_INSIDE,
        corpus.width()
    );
    println!(
        "  the boundary set is derived three ways and all three agree: by construction, by the \
         two-probe forward step, by the width sweep\n"
    );
    assert_eq!(corpus.len(), document::CORPUS_CLUSTERS);
    assert_eq!(corpus.inside(), corpus.chars() - corpus.len());
}

/// 3. The `char` caret, which is the negative case.
fn the_char_caret() {
    let corpus = clusters::corpus();
    let walk = document::char_walk(&corpus);
    let ascii = document::char_walk(&clusters::ascii_corpus());
    println!("report  the `char` caret against the cluster caret, over the same corpus:");
    println!(
        "  {:<40}  {:>8}  {:>8}  {:>10}",
        "arm", "steps", "inside", "surprises"
    );
    println!(
        "  {:<40}  {:>8}  {:>8}  {:>10}",
        "a `char` caret over the corpus", walk.steps, walk.inside, walk.width_surprises
    );
    println!(
        "  {:<40}  {:>8}  {:>8}  {:>10}",
        "a cluster caret over the corpus", walk.clusters, 0, 0
    );
    println!(
        "  {:<40}  {:>8}  {:>8}  {:>10}",
        "a `char` caret over ASCII only", ascii.steps, ascii.inside, ascii.width_surprises
    );
    println!(
        "  §11 remembers {} steps for {} clusters, {} inside, {} surprises. A *surprise* is an \
         insertion that changes",
        document::REMEMBERED_STEPS,
        document::REMEMBERED_CLUSTERS,
        document::REMEMBERED_INSIDE,
        document::REMEMBERED_SURPRISES
    );
    println!(
        "  the buffer's display width by something other than what was typed — the cluster came \
         apart. On ASCII the sweep reports NOTHING,\n  which is the whole reason the corpus is a \
         value rather than a string literal.\n"
    );
    assert!(walk.width_surprises > 0);
    assert_eq!(ascii.width_surprises, 0);
}

/// 4. `Left` at the end of a pasted megabyte, both ways. **A report.**
fn left_at_a_megabyte() {
    let text = document::pasted();
    let index = Index::build(&text, document::WIDE, 0);
    let (blind, indexed, caret) = document::left_at_the_end(&text, &index);
    let ratio = blind.as_secs_f64() / indexed.as_secs_f64().max(f64::MIN_POSITIVE);
    println!(
        "report  one `Left` at the end of a pasted megabyte ({} bytes):",
        text.len()
    );
    println!("  {:<44}  {:>12}", "arm", "us");
    println!(
        "  {:<44}  {:>12.3}",
        "no index — segment forward from byte 0",
        micros(blind)
    );
    println!(
        "  {:<44}  {:>12.3}",
        "with the index — from the caret's own row",
        micros(indexed)
    );
    println!(
        "  ratio {ratio:.0}x   (§11: 3161.68 us against 0.327 — {:.0}x). A TIMING AND NEVER A GATE.",
        document::REMEMBERED_LEFT_RATIO
    );
    let row = index.row_of(text.len());
    println!(
        "  what is gated is the count beside it: the blind arm segments {} bytes and the indexed \
         arm {}\n",
        text.len(),
        text.len() - index.row_start(row.saturating_sub(1)),
    );
    assert!(caret.byte() < text.len());
    let _ = Caret::default();
}

/// 5. The wrap index: the resize, the recompute count, and the five hundred splices.
fn the_wrap_index() {
    let text = document::text();
    let wide = Index::build(&text, document::WIDE, 0);
    let narrow = Index::build(&text, document::NARROW, 0);
    println!("report  the wrap index over the same document at two widths:");
    println!(
        "  {} columns: {} visual rows   |   {} columns: {} visual rows   (§21: 625 rows drawn \
         where 875 are needed)",
        document::WIDE,
        wide.rows(),
        document::NARROW,
        narrow.rows()
    );

    let (correct, defective) = document::recomputes_over_the_resize();
    println!(
        "  recomputes over the resize: keyed on (revision, width) {correct}, keyed on the revision \
         alone {defective}   (§21: 1 against 2 — `recomputes` points the WRONG WAY)"
    );

    let diff = document::resized();
    println!("  the surface: {diff}   (§21: 69 of 80 rows differ)");

    let (splices, witness) = document::splice_sweep_with_witness();
    println!(
        "  {} splices: restarted at `row_of(at)` wrong {} times, restarted one row earlier wrong \
         {} times   (§11: 499 of 500 agreed)",
        splices.trials, splices.at_row_of, splices.one_row_earlier
    );
    println!(
        "  the witness the screen is drawn from is edit at byte {}\n",
        witness.map(|(_, at)| at).unwrap_or(0)
    );
    assert_eq!(wide.rows(), document::LINES);
    assert_eq!(narrow.rows(), document::WRAPPED);
    assert_eq!(diff.rows, document::STALE_ROWS);
    assert_eq!(splices.one_row_earlier, 0);
    assert!(splices.at_row_of > 0);
    println!(
        "  agreements {} of {} against §11's {} of 500 — a rate over this corpus and these five \
         insertions, not a property of the mechanism\n",
        splices.trials - splices.at_row_of,
        splices.trials,
        document::REMEMBERED_SPLICE_AGREEMENTS
    );
}

/// 6. **The four gates, and which of §20's nine counters can see each. None.**
fn the_four_gates() {
    println!("report  §11's four gates — *none of them visible on the rendered screen*, measured:");
    println!(
        "  {:<58}  {:>8}  {:>6}  {:<24}",
        "gate", "cells", "rows", "counters that separate them"
    );
    for defect in Defect::ALL {
        let (mut good, mut bad) = document::screens(defect);
        let a = steady_allocations(&mut good);
        let b = steady_allocations(&mut bad);
        let inert = Allocations::over(1, 0);
        let separating = document::counters_that_separate(defect, inert, inert);
        let with_probe = document::counters_that_separate(defect, a, b);
        let diff = document::surface_diff(defect);
        let words: Vec<&str> = separating.iter().map(|c: &Counter| c.word()).collect();
        println!(
            "  {:<58}  {:>8}  {:>6}  {:<24}  allocations {} / {}",
            defect.gate(),
            diff.cells,
            diff.rows,
            if words.is_empty() {
                "none".to_string()
            } else {
                words.join(", ")
            },
            a.total(),
            b.total()
        );
        // The eight counters this crate can read, with the allocation column inert on both arms:
        // none of them separates the two builds on any of the four gates.
        assert!(separating.is_empty());
        // **And the ninth, measured on both arms, separates exactly one of the four — in the
        // direction that APPROVES the defect.** A stale index has 625 rows where 875 are needed, so
        // the defective frame allocates *less*. §21's first refinement is this exact shape: a
        // counter on the wrong side of the question is not a weak gate, it is a green one.
        if !with_probe.is_empty() {
            assert_eq!(with_probe, vec![Counter::Allocations]);
            assert!(
                b.total() <= a.total(),
                "the allocation total is a separation only because the defective build is cheaper"
            );
        }
    }
    println!(
        "  TWO of the four move no cell at all — the caret is the terminal's cursor and not a \
         cell — and two move a great many."
    );
    println!(
        "  For the two that move cells, NO counter of §20's nine separates the arms: the equality \
         against a correct render is the only detector there is."
    );
    println!(
        "  The one column that CAN be made to differ is the allocation total, and it differs in \
         the direction that approves the defect — a stale"
    );
    println!(
        "  index has 625 rows where 875 are needed, so the defective frame allocates less. That is \
         §21's refinement 1 exactly: a counter on the"
    );
    println!("  wrong side of the question is not a weak gate, it is a green one.\n");
}

/// The allocation total for one screen over two hundred steady frames, measured on **both** arms.
///
/// A figure handed to one arm and defaulted on the other is a separation the instrument invented.
fn steady_allocations(screen: &mut document::Screen) -> Allocations {
    const FRAMES: u32 = 8;
    let (_, total) = count_allocations(|| {
        for _ in 0..FRAMES {
            let _ = document::play_field(screen, Allocations::over(1, 0));
        }
    });
    Allocations::over(FRAMES, total as u64)
}

/// 7. What does not reproduce, and why.
fn what_does_not_reproduce() {
    let corpus = clusters::corpus();
    let walk = document::char_walk(&corpus);
    println!("report  what does not reproduce, said out loud:");
    println!(
        "  clusters {} against §11's {}: {} whole turns of an eight-word rota is 167 and there is \
         no arrangement of whole",
        corpus.len(),
        document::REMEMBERED_CLUSTERS,
        clusters::CYCLES
    );
    println!("  turns that is 166. Padding to 166 would be a corpus tuned to a remembered number.");
    println!(
        "  code points {} against §11's {}: this corpus carries {:.3} code points a cluster and \
         §11's carried {:.3} — its prose",
        corpus.chars(),
        document::REMEMBERED_STEPS,
        corpus.chars() as f64 / corpus.len() as f64,
        document::REMEMBERED_STEPS as f64 / document::REMEMBERED_CLUSTERS as f64
    );
    println!("  had more ASCII in it. One word of eight here is plain ASCII.");
    println!(
        "  inside-landings {} against §11's {}: NOT a third measurement. `inside == chars - \
         clusters` holds for every string, and",
        corpus.inside(),
        document::REMEMBERED_INSIDE
    );
    println!(
        "  §11's own three figures satisfy it: {} - {} == {}. The RELATION reproduces exactly; the \
         magnitudes are this corpus's.",
        document::REMEMBERED_STEPS,
        document::REMEMBERED_CLUSTERS,
        document::REMEMBERED_INSIDE
    );
    println!(
        "  width surprises {} against §11's {}: the same story one level down — a surprise is one \
         insertion per code point inside a",
        walk.width_surprises,
        document::REMEMBERED_SURPRISES
    );
    println!("  cluster that changes the width, so it scales with this corpus's cluster mix.");
    println!(
        "  625 / 875 / 69-of-80 DO reproduce, and they are the fixture's PARAMETERS rather than \
         its measurements — `SHORT_HEAD` says so."
    );
    println!(
        "  §11's *restarting one row earlier is provably enough* reproduces only because a defect \
         in `vitui_runtime::layout::text::wrap`"
    );
    println!(
        "  was fixed by this ticket: a row that exactly filled its band fell through to the \
         overhang branch and was cut at the FIRST space."
    );
    println!(
        "  Against the unfixed wrap, five of the five hundred splices restarted one row earlier \
         still disagreed with a rebuild.\n"
    );
}

/// A duration in microseconds.
fn micros(d: Duration) -> f64 {
    d.as_secs_f64() * 1_000_000.0
}
