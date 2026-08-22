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
use vitui_engine::Rect;
use vitui_runtime::layout::{
    Col,
    Constraint::{Fixed, Weight},
    Row, text,
};

/// The frame budget every ratio here is against.
const FRAME_NS: f64 = 100_000.0;

/// The width the rows are wrapped to: a detail pane in the screen the layout report uses.
const PANE: u16 = 38;

/// Twenty-four rows, which is a screenful of a list with wrapped cells.
///
/// **Deliberately not twenty-four copies of one string.** A row of Latin, a row that wraps four
/// times, a CJK row with nothing to break on and a row full of clusters cost different amounts, and a
/// measurement over one of them would be a measurement of that one.
const ROWS: [&str; 24] = [
    "a short line",
    "The quick brown fox jumps over the lazy dog and keeps going for a while yet",
    "漢字漢字漢字漢字漢字漢字漢字漢字漢字漢字漢字漢字",
    "e\u{301}quipe \u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467} and a\u{2764}\u{FE0F}b",
    "status: ready",
    "one two three four five six seven eight nine ten eleven twelve thirteen",
    "mixed 漢字 and latin words together in one line that has to wrap somewhere",
    "a\u{300}\u{301}\u{302}\u{303} stacked combining marks, and then some ordinary text after them",
    "/usr/local/share/doc/some-package/examples/a-rather-long-path-name.txt",
    "",
    "  leading and   internal   spaces  ",
    "supercalifragilisticexpialidocious",
    "warning: the value was truncated because it did not fit in the column provided",
    "42",
    "\u{2764}\u{FE0E} text presentation, and \u{2764}\u{FE0F} emoji presentation, side by side",
    "one\ntwo\nthree",
    "The second paragraph is here to make the corpus look like real data rather than a fixture",
    "漢字 mixed with a very long latin run that will definitely need more than one line",
    "id       name                 state     updated",
    "a b c d e f g h i j k l m n o p q r s t u v w x y z",
    "tab\tseparated\tvalues\there",
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor",
    "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}",
    "done",
];

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

/// The same realistic screen the layout report times, so the two arms are comparable.
///
/// A third copy of this function, and the third one is the one that made me stop and think about it:
/// `tests/alloc.rs` has it, `examples/layout_numbers.rs` has it, and now this. All three assert
/// `(32, 119)`, which is what keeps them honest — but the right answer at four copies is a
/// `#[path]`-included module the way the engine shares its scene list, and that is worth doing before
/// a fourth arrives rather than after.
fn screen_frame(w: u16, h: u16) -> (u32, u32) {
    let mut splits = 0;
    let mut lanes = 0;
    let screen = Rect::new(0, 0, w, h);

    let [header, body, footer] = Col::new().split(screen, [Fixed(3), Weight(1), Fixed(1)]);
    splits += 1;
    lanes += 3;
    let [sidebar, _gutter, detail] = Row::new()
        .spacing(1)
        .split(body, [Fixed(20), Fixed(1), Weight(1)]);
    splits += 1;
    lanes += 3;
    let _ = Col::new()
        .margin(1)
        .split(detail, [Weight(1), Weight(1), Weight(1)]);
    splits += 1;
    lanes += 3;
    let _ = Row::new().spacing(2).split(footer, [Weight(1); 6]);
    splits += 1;
    lanes += 6;
    let [left, mid, right] = Row::new().split(header, [Fixed(10), Weight(1), Fixed(12)]);
    splits += 1;
    lanes += 3;
    let _ = Row::new().split(left, [Weight(1), Weight(1)]);
    splits += 1;
    lanes += 2;
    let _ = Row::new().split(right, [Weight(1), Weight(1)]);
    splits += 1;
    lanes += 2;
    let _ = Col::new().split(mid, [Weight(1)]);
    splits += 1;
    lanes += 1;

    let mut rows = [Rect::default(); 24];
    let n = Col::new().split_into(sidebar, &[Weight(1); 24], &mut rows);
    for row in &rows[..n] {
        let _ = Row::new().split(*row, [Fixed(6), Weight(2), Weight(1), Fixed(8)]);
        splits += 1;
        lanes += 4;
    }
    (splits, lanes)
}
