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

use std::time::Duration;

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
    what_does_not_reproduce();
}

/// 1. The two scenes, and the one ticket that inverts both.
fn scene_list() {
    println!("report  the two scenes, and the one reason they are red:");
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
    assert_eq!(red, 2, "both scenes are pinned red");
    assert_eq!(scenes_for("field").count(), 2);
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
    assert!(caret.byte < text.len());
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
        let (good, bad) = document::screens(defect);
        let a = steady_allocations(&good);
        let b = steady_allocations(&bad);
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
fn steady_allocations(screen: &document::Screen) -> Allocations {
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
