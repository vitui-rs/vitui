//! **The order, the index and the memo, as numbers.**
//!
//! Components ticket 13. The convention is the runtime's — a file in `examples/` named
//! `<subject>_numbers.rs` that prints the numbers a human reads — and so is the rule about what an
//! example may be: **`cargo test` does not run this file**, and no row of
//! [`vitui_components::gates::REGISTER`] rests on it. Rows 75–78 *cite* it and each names a
//! `#[test]` in `src/order.rs` beside the citation.
//!
//! # What it prints
//!
//! 1. **One structure, five names**, with which of `Entry`'s four fields each of them spends.
//! 2. **The splice against the permutation**, over §10's own edit: a million rows, 349 524 removed,
//!    a half-tree contiguous selection. The span counts are the finding and the microseconds are
//!    beside them.
//! 3. **The three real policies**, all `O(spans)` and all free, so the choice between them is
//!    behavioural rather than a cost.
//! 4. **The 300 → 120 resize**, where the memo with the wrong key recomputes **less**, runs
//!    **faster**, and draws a document at the previous width.
//! 5. **What does not reproduce**, said out loud rather than engineered away.

use std::time::Instant;

use vitui_components::collect::Selection;
use vitui_components::order::{
    self, ASKED_BYTES, Entry, Keyed, NARROW, Order, Policy, REVISION_BYTES, SCREEN_ROWS, USES,
    WIDE, WrapIndex,
};
use vitui_runtime::Revision;

/// How many rounds a reconcile is timed over, minimum-of-N.
const ROUNDS: u32 = 5;

/// §10's own interval: a million rows with 349 524 removed, which is a 52%-folded tree.
const LEN: usize = 1_000_000;
/// The interval a collapse takes out. See [`LEN`].
const REMOVED: std::ops::Range<usize> = 100_000..449_524;

fn main() {
    println!("The order, the index and the memo — one structure, five names\n");

    the_five_names();
    the_splice_and_the_permutation();
    the_policies();
    the_resize();
    what_does_not_reproduce();
}

/// **One structure, five names, and which fields each spends.**
fn the_five_names() {
    println!("== one structure, five names ==\n");
    println!("  owner        what                                    fields");
    for use_of in USES {
        println!(
            "  {:<11}  {:<38}  {}",
            use_of.owner,
            use_of.name,
            use_of.fields.join(", ")
        );
    }
    println!(
        "\n  size_of::<Entry>()  {} B      size_of::<Rows>()'s revision  {REVISION_BYTES} B      \
         size_of::<Asked>()  {ASKED_BYTES} B\n",
        size_of::<Entry>()
    );
    assert_eq!(USES.len(), 5);
}

/// **A splice against a permutation, over §10's own edit.**
fn the_splice_and_the_permutation() {
    println!(
        "== one edit, two ways: {LEN} rows, {} removed, a half-tree contiguous selection ==\n",
        REMOVED.end - REMOVED.start
    );
    let (spliced, permuted) = order::splice_vs_permutation(LEN, REMOVED, ROUNDS);
    println!("  reconciled as    spans before  spans after       bytes         us");
    println!(
        "  a splice         {:>12}  {:>11}  {:>10}  {:>9.2}",
        spliced.before,
        spliced.after,
        spliced.bytes,
        spliced.nanos as f64 / 1000.0
    );
    println!(
        "  a permutation    {:>12}  {:>11}  {:>10}  {:>9.2}",
        permuted.before,
        permuted.after,
        permuted.bytes,
        permuted.nanos as f64 / 1000.0
    );
    println!(
        "\n  ratio: {}x the spans, {:.0}x the time.",
        permuted.after / spliced.after.max(1),
        permuted.nanos as f64 / spliced.nanos.max(1) as f64
    );
    println!(
        "  §10: 1 run / 0.04 us / 0 KB against 297 180 runs / 24 338 us / 8 550 KB — 608 000x.\n  \
         **A subtree is contiguous in pre-order display coordinates**, so under a splice a sorted\n  \
         span list transforms in O(spans) and cannot shatter. Under a permutation it shatters into\n  \
         one span a row, which is what makes `Clear` the honest default there.\n"
    );
    assert_eq!(spliced.after, 1, "a splice cannot shatter");

    // Select-all, which escapes both at O(1) — and not by special-casing.
    let mut all = Selection::new();
    all.select_all(LEN);
    let at = Instant::now();
    order::reconcile_permutation(&mut all, LEN, Policy::Remap, &|i| Some((i * 3) % (LEN + 1)));
    let took = at.elapsed().as_nanos();
    println!(
        "  select-all under the same permutation: {} span, {} B, {took} ns — `[0, len)` is\n  \
         invariant under any permutation of that length, so the answer is known without touching\n  \
         a row.\n",
        all.span_count(),
        all.bytes()
    );
}

/// **The three real policies, all `O(spans)` and all free.**
fn the_policies() {
    println!("== the four policies ==\n");
    println!("  policy   permutation  splice   default for");
    for policy in Policy::ALL {
        let default_for = match policy {
            Policy::Clear => "a permutation",
            Policy::Drop => "a splice",
            _ => "-",
        };
        println!(
            "  {:<7}  {:>11}  {:>6}   {default_for}",
            policy.word(),
            policy.admits(order::EditKind::Permutation),
            policy.admits(order::EditKind::Splice),
        );
    }
    let (clear, drop, stash) = order::policy_costs(LEN, REMOVED, ROUNDS);
    println!(
        "\n  cost over the same edit: clear {:.2} us · drop {:.2} us · stash {:.2} us",
        clear as f64 / 1000.0,
        drop as f64 / 1000.0,
        stash as f64 / 1000.0
    );
    println!(
        "  §10: clear 0.00 · drop 0.08 · stash 0.04. All three are O(spans) and all three are\n  \
         free, **so the choice is recorded as behavioural and not as cost** — and `Clear` under a\n  \
         splice is refused by name, because it throws away spans that were nowhere near the fold\n  \
         for no saving at all.\n"
    );
}

/// **The 300 → 120 resize, where the wrong key recomputes less and runs faster.**
fn the_resize() {
    println!("== a memo's key is every input: the 300 -> 120 resize ==\n");
    let lines = order::document();
    let rev = Revision::fresh();

    let mut right: Keyed<(Revision, u16), WrapIndex> = Keyed::new();
    let at = Instant::now();
    let _ = right.get((rev, WIDE), || WrapIndex::built(&lines, WIDE));
    let narrow = right
        .get((rev, NARROW), || WrapIndex::built(&lines, NARROW))
        .clone();
    let right_ns = at.elapsed().as_nanos();

    let mut wrong: Keyed<Revision, WrapIndex> = Keyed::new();
    let at = Instant::now();
    let _ = wrong.get(rev, || WrapIndex::built(&lines, WIDE));
    let stale = wrong.get(rev, || WrapIndex::built(&lines, NARROW)).clone();
    let wrong_ns = at.elapsed().as_nanos();

    let differ = (0..SCREEN_ROWS)
        .filter(|&i| stale.line_at(i) != narrow.line_at(i))
        .count();

    println!("  keyed on               recomputes    rows drawn   width built at        us");
    println!(
        "  (revision, width)      {:>10}    {:>10}   {:>13}  {:>8.2}",
        right.recomputes,
        narrow.rows(),
        narrow.width(),
        right_ns as f64 / 1000.0
    );
    println!(
        "  the revision alone     {:>10}    {:>10}   {:>13}  {:>8.2}",
        wrong.recomputes,
        stale.rows(),
        stale.width(),
        wrong_ns as f64 / 1000.0
    );
    println!(
        "\n  {} of {SCREEN_ROWS} visible rows carry a different source line.",
        differ
    );
    println!(
        "  **The natural detector points the wrong way**: the wrong key recomputes {} against {}\n  \
         and runs faster. `recomputes` reports an improvement. The two that work are the memoised\n  \
         object recording the input it was built at — `{}` against `{}` above — and the rendered\n  \
         surface.\n",
        wrong.recomputes,
        right.recomputes,
        stale.width(),
        narrow.width()
    );
}

/// **What does not reproduce, said out loud.**
fn what_does_not_reproduce() {
    println!("== what does not reproduce ==\n");
    let lines = order::document();
    let wide = WrapIndex::built(&lines, WIDE);
    let narrow = WrapIndex::built(&lines, NARROW);
    println!(
        "  1. §10 records **625 rows drawn where 875 are needed; 69 of 80 rows differ**, over\n     \
         C06's document, which is not in this repository. `order::document` is stated rather than\n     \
         fitted at those two numbers, and this corpus gives {} against {}. Fitting a corpus to a\n     \
         remembered pair is a fixture that has stopped measuring anything.",
        wide.rows(),
        narrow.rows()
    );
    println!(
        "  2. The permutation column is `Remap` and not `Clear`, because `Clear` is O(1) and would\n     \
         price nothing. What §10 measures is what reconciling a permutation *exactly* costs, which\n     \
         is the number that makes `Clear` the honest default."
    );
    println!(
        "  3. The permutation is `i * 2 mod (len | 1)` — deterministic, so the figure is\n     \
         replayable, and a full cycle, so it really is a permutation. §10's shuffle is not\n     \
         recorded and the magnitude depends on it; **the shattering is what does not.**\n"
    );

    let order = Order::built(vec![Entry::default(); 8]);
    assert_eq!(order.rows().len, 8);
}
