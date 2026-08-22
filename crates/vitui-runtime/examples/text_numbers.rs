//! **Runtime ticket 03's two reports**: what text measurement costs against what layout costs, and
//! what correctness costs against the wrong answer.
//!
//! ```text
//! cargo run --release --example text_numbers -p vitui-runtime
//! ```
//!
//! Both are here because both are numbers somebody would otherwise assume:
//!
//! 1. **Layout is not the cost; text is.** A whole screen of constraint solving is 32 splits and 119
//!    lanes; measuring twenty-four wrapped rows is *more than twice that*. §11 measured 4.41 µs
//!    against 1.76 µs. Put the two on one page and the assumption cannot survive.
//! 2. **Correctness costs 25.5×**, and the alternative is not cheaper — it is wrong. `chars().count()`
//!    is one pass over bytes; grapheme segmentation plus a width lookup a cluster is a great deal more
//!    than that. The number is here so that the trade is visible and so that nobody re-discovers the
//!    cheap version as an optimisation.

use std::hint::black_box;

use vitui_bench::Bench;
use vitui_runtime::layout::text;

// The one realistic screen and its row corpus, shared with the allocation gates and the layout
// report. See `src/screen.rs`.
#[allow(dead_code)]
#[path = "../src/screen.rs"]
mod screen;
use screen::{ROWS, screen_frame};

/// The frame budget every ratio here is against.
const FRAME_NS: f64 = 100_000.0;

/// The width the rows are wrapped to: a detail pane in the screen the layout report uses.
const PANE: u16 = 38;

fn main() {
    println!("runtime ticket 03 — what text measurement costs\n");
    let lines: usize = ROWS.iter().map(|r| text::wrap_height(r, PANE)).sum();
    let clusters: usize = ROWS
        .iter()
        .map(|r| vitui_engine::graphemes(r).count())
        .sum();
    println!(
        "corpus: {} rows wrapped to {PANE} columns -> {lines} lines, {clusters} clusters",
        ROWS.len()
    );

    let (splits, lanes) = screen_frame(300, 80);
    assert_eq!(
        (splits, lanes),
        (32, 119),
        "the layout arm's shape has changed"
    );

    // One bench, so the two arms are round-robined and share the same interference. Comparing two
    // separately-run benches would be comparing two moments in the machine's life, and the whole
    // point of this report is the ratio between them.
    let report = Bench::new(40)
        .case("layout/whole screen", 100, || {
            black_box(screen_frame(300, 80));
        })
        .case("text/24 wrapped rows", 100, || {
            let mut total = 0usize;
            for row in ROWS {
                total += text::wrap_height(row, PANE);
            }
            black_box(total);
        })
        .run();

    println!("\nreport  text against layout, minimum of 40 rounds:\n{report}");
    let layout_ns = report.get("layout/whole screen").expect("measured");
    let text_ns = report.get("text/24 wrapped rows").expect("measured");
    println!(
        "        layout  {:>8.2} us  {:>5.2}% of a {:.0} us frame\n\
        \x20       text    {:>8.2} us  {:>5.2}% of a {:.0} us frame\n\
        \x20       text / layout = {:.2}x",
        layout_ns / 1e3,
        layout_ns / FRAME_NS * 100.0,
        FRAME_NS / 1e3,
        text_ns / 1e3,
        text_ns / FRAME_NS * 100.0,
        FRAME_NS / 1e3,
        text_ns / layout_ns,
    );
    println!(
        "        spec §11 measured 4.41 us of text against 1.76 us of layout, 2.5x.\n\
        \x20       **The deleted measure pass was for components; this cost is real and it belongs\n\
        \x20       to whoever calls it.** A measure pass would pay it on every frame for every\n\
        \x20       component whether or not anything wrapped."
    );

    // The second report: what the right answer costs against the wrong one.
    let report = Bench::new(40)
        .case("width/correct", 1_000, || {
            let mut total = 0u32;
            for row in ROWS {
                total += u32::from(text::width(row));
            }
            black_box(total);
        })
        .case("width/chars, and wrong", 1_000, || {
            let mut total = 0usize;
            for row in ROWS {
                total += row.chars().count();
            }
            black_box(total);
        })
        .run();

    println!("\nreport  what correctness costs, minimum of 40 rounds:\n{report}");
    let correct = report.get("width/correct").expect("measured");
    let wrong = report.get("width/chars, and wrong").expect("measured");
    println!(
        "        correct / chars = {:.1}x, and spec §11 measured 25.5x.",
        correct / wrong
    );

    // And what the wrong answer actually says, which is the reason the ratio is not a trade.
    let family = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";
    println!(
        "        The wrong answer is not a cheaper approximation. On one family emoji:\n\
        \x20         columns {}   clusters {}   chars {}   bytes {}\n\
        \x20       Only the first is a caret column. `chars().count()` puts the caret of a family\n\
        \x20       emoji between two letters either side at column 9 where the engine's rule says\n\
        \x20       6 — and the engine is what draws it, so its rule is not an opinion.",
        text::width(family),
        vitui_engine::graphemes(family).count(),
        family.chars().count(),
        family.len(),
    );
    println!(
        "        **The 25.5x is bought, not paid.** Every function here goes through\n\
        \x20       `graphemes()` and `width_of()`, so the runtime ships no second copy of the UCD —\n\
        \x20       which is what ADR 0005 exists to prevent, because two copies disagree the day\n\
        \x20       their Unicode versions differ."
    );
}
