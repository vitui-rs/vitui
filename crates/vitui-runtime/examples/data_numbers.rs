//! **Runtime ticket 01's reports**, on this machine, against the shipped `data` module.
//!
//! ```text
//! cargo run --release --example data_numbers -p vitui-runtime
//! ```
//!
//! Two numbers are owed and a third is refused:
//!
//! 1. **What the memo is worth** — the chart's scale fold against a memo hit. Spec §14 measured
//!    84.91 µs against 1.28 ns, which is 66 301×.
//! 2. **The three source shapes** — one row drawer over indexed, chunked and linked access, at
//!    19.77 µs, 320.85 µs and 78.40 ms. This is the number the ticket wants *in the repository rather
//!    than only in the spec*, because the reasonable-looking chunked source is already **321% of the
//!    whole frame budget** and nothing in a type says so.
//! 3. **The unchanged frame's 23.58 µs is not measurable here and is not attempted.** It is the
//!    composition residue — the view function running, the rows being formatted, the verbs being
//!    issued — and there is no frame yet. The sentence and its number are in [`Memo`]'s rustdoc,
//!    which is where the ticket asks for them; this file would only be able to fake it.
//!
//! # The gate is the step count, not the timing
//!
//! §14's rule, and it decides the shape of this file: **a gate is a count, a ratio, an equality or a
//! compile outcome; a timing is a report.** So the three source shapes are *gated* on how many
//! access steps each one takes to draw the same seventy-eight rows, and *reported* in microseconds.
//!
//! That is not a formality. **All three sources allocate nothing in a steady frame**, so the
//! difference between them is invisible to every heap tool, and the timings are three orders of
//! magnitude apart on one machine and would be three different orders on another. The step count is
//! the same number everywhere.
//!
//! # The inner iteration counts are not arbitrary
//!
//! `Bench::case` takes an iteration count and divides, and the first run of this file reported
//! **0.00 ns** for the memo hit, both read arms and the indexed source. One memo hit is about a
//! nanosecond and seventy-eight index reads are a few tens; `Instant` cannot resolve either, and a
//! benchmark that reports zero has measured its own clock. The counts below put every arm above that
//! floor, which is why they differ per case rather than being one number.
//!
//! # Provenance
//!
//! **R 01** took these numbers and **R 20** re-ran the report on 2026-08-24: Apple M1 Max, macOS
//! 26.5.2, rustc 1.97.1, `--release`, unloaded, minimum of 40 rounds and 20 for the three sources.
//! The fold reads 87.88 µs against §14's 84.91 and the memo hit 1.33 ns against its 1.28 — a few
//! per cent on two figures four orders of magnitude apart, which is the instrument agreeing rather
//! than the mechanism moving. **No row of `crate::ledger` belongs to this file**, and that is the
//! correct outcome rather than an omission: nothing measured here is on the frame path, because the
//! memo is what keeps the fold off it.
//!
//! The 100 µs the percentages divide by is **read from `crate::ledger` and is not this file's to
//! choose**. Spec §19 inherits it from the engine map, and **a budget figure may not move without a
//! new map decision** — which is the sentence the chunked source at 227% of it is evidence for. What
//! has to move there is the data shape, not the divisor.

use std::cell::Cell;
use std::hint::black_box;

use vitui_bench::Bench;
use vitui_runtime::data::{Memo, Versioned};

/// The canonical scene's row count for the memo arm: a fold over a million rows is the thing the
/// memo exists to skip.
const MILLION: usize = 1_000_000;

/// The row count for the three source shapes: **the same million as the memo arm**, because the
/// whole point of the chunked row is that it is over the frame budget at the size an application
/// actually has, and a smaller list hides that.
///
/// The first version of this file used ten thousand and reported the chunked source at **5% of a
/// frame instead of 321%** — a hundredfold understatement of the one trap the ticket asks to have in
/// the repository. A benchmark scaled down for its own convenience answers a different question.
const ROWS: usize = MILLION;

/// A screenful, on a 300x80 with a header and a footer.
const VISIBLE: usize = 78;

/// Halfway down, which is the spec's own offset and the honest one for a walked source: the cost of
/// reaching a row is the whole subject, and the top of the list is the one place all three shapes
/// agree.
const OFFSET: usize = MILLION / 2;

/// Chunk width for the paged source. A rope, a paged cursor, a `Vec<Vec<T>>` walked chunk by chunk.
const CHUNK: usize = 64;

// **The frame budget every ratio here is against, read rather than written.** See the provenance
// note above: this file used to declare its own copy of the figure.
#[allow(dead_code)]
#[path = "../src/ledger.rs"]
mod ledger;

use ledger::frame_budget_ns;

/// The struct-of-vectors every measurement on this map rests on, and the shape a slice-handing trait
/// cannot be implemented for.
struct Indexed {
    /// Never read here, and kept anyway: **two columns is what makes it a struct-of-vectors**, and a
    /// struct-of-vectors is the shape the slice-handing trait could not be implemented for. A
    /// one-column version would measure a `Vec<f32>` and quietly stop being the scene.
    #[expect(
        dead_code,
        reason = "the second column is what makes the shape the shape"
    )]
    pid: Vec<u32>,
    cpu: Vec<f32>,
}

impl Indexed {
    fn build(rows: usize) -> Indexed {
        Indexed {
            pid: (0..rows)
                .map(|i| u32::try_from(i).unwrap_or(u32::MAX))
                .collect(),
            cpu: (0..rows).map(|i| i as f32).collect(),
        }
    }
}

/// A paged source: chunks reached by walking from the first one.
struct Chunked {
    chunks: Vec<Vec<f32>>,
}

impl Chunked {
    fn build(rows: usize) -> Chunked {
        Chunked {
            chunks: (0..rows)
                .map(|i| i as f32)
                .collect::<Vec<f32>>()
                .chunks(CHUNK)
                .map(<[f32]>::to_vec)
                .collect(),
        }
    }

    /// Walked rather than indexed, which is what makes it O(k/64): a rope does not know where its
    /// chunks are without following them.
    fn get(&self, index: usize, steps: &Cell<u64>) -> f32 {
        let mut seen = 0;
        for chunk in &self.chunks {
            steps.set(steps.get() + 1);
            if seen + chunk.len() > index {
                return chunk[index - seen];
            }
            seen += chunk.len();
        }
        f32::NAN
    }
}

/// A linked source, walked from the head for every row.
struct Linked {
    head: Option<Box<Node>>,
}

struct Node {
    cpu: f32,
    next: Option<Box<Node>>,
}

/// **A million `Box<Node>` cannot be dropped recursively.** The derived drop walks `next` by
/// recursion and overflows the stack at this length, so the list frees itself in a loop. This is not
/// incidental to the point being made: the shape that is expensive to *read* is also the one that is
/// awkward to own, and both come from the same absence of an index.
impl Drop for Linked {
    fn drop(&mut self) {
        let mut node = self.head.take();
        while let Some(mut n) = node {
            node = n.next.take();
        }
    }
}

impl Linked {
    fn build(rows: usize) -> Linked {
        let mut head = None;
        for i in (0..rows).rev() {
            head = Some(Box::new(Node {
                cpu: i as f32,
                next: head,
            }));
        }
        Linked { head }
    }

    fn get(&self, index: usize, steps: &Cell<u64>) -> f32 {
        let mut node = self.head.as_deref();
        let mut at = 0;
        while let Some(n) = node {
            steps.set(steps.get() + 1);
            if at == index {
                return n.cpu;
            }
            at += 1;
            node = n.next.as_deref();
        }
        f32::NAN
    }
}

/// The one row drawer. **Eighty verbs, the same hit index, and it never learns what it is reading
/// from** — which is the sentence that killed the trait: *closures cover every shape a trait would.*
///
/// The body **formats** the row rather than adding it to a total, and that is what makes the
/// absolute figures comparable with the spec's: a real row is `{:>5.1}` into a buffer, which costs a
/// couple of hundred nanoseconds, and an access cost has to be read against that rather than against
/// nothing. It is also why the wrapper's own cost is measured by a *different* arm above with no
/// formatting in it — the spec found that differencing two formatted screens produces a **negative**
/// marginal cost for a sub-nanosecond index, because the `{:>5.1}` swamps it.
///
/// `impl Fn`, not `impl FnMut`, because that is the shape the spec's own `bar_chart` commits to — an
/// accessor is a read and a component has no business mutating through one. It has a consequence this
/// file ran into: **instrumenting the accessor needs a `Cell`**, since an `Fn` closure cannot hold a
/// `&mut` counter. That is the signature being right and the instrument paying for it, not the other
/// way round.
fn draw_rows(len: usize, value: impl Fn(usize) -> f32) -> f32 {
    use std::fmt::Write as _;

    let mut row = String::with_capacity(32);
    let mut sink = 0.0f32;
    for r in 0..len {
        let v = value(r);
        row.clear();
        let _ = write!(row, "{v:>5.1}");
        sink += row.len() as f32;
    }
    sink
}

/// The same drawer with no formatting in it, for the arm that is about the wrapper rather than about
/// access. See [`draw_rows`] for why the two cannot be one function.
fn read_rows(len: usize, value: impl Fn(usize) -> f32) -> f32 {
    let mut sink = 0.0f32;
    for r in 0..len {
        sink += value(r);
    }
    sink
}

fn main() {
    println!("runtime ticket 01 — the data contract's two reports");
    println!(
        "scene: {VISIBLE} visible rows at offset {OFFSET}, and a {MILLION}-row fold for the memo\n"
    );

    what_the_memo_is_worth();
    what_the_wrapper_costs();
    the_three_source_shapes();
}

/// Report: the chart's scale fold against a memo hit.
///
/// The bars are O(visible) and the scale is O(rows) — a maximum over every row, not over the ones on
/// screen — which is why this one fold is the whole argument for the type.
fn what_the_memo_is_worth() {
    let data = Versioned::new(Indexed::build(MILLION));
    let mut cold: Memo<f32> = Memo::new();
    let mut warm: Memo<f32> = Memo::new();
    warm.get(data.revision(), || {
        data.cpu.iter().copied().fold(f32::MIN, f32::max)
    });

    let report = Bench::new(40)
        .case("fold/1M rows", 1, || {
            let max = data.cpu.iter().copied().fold(f32::MIN, f32::max);
            black_box(max);
        })
        .case("memo/hit", 100_000, || {
            // A fresh `Memo` every round would measure the miss; this one is warm and the closure is
            // never called, which is the whole of what is being timed.
            let max = *warm.get(data.revision(), || unreachable!("warm"));
            black_box(max);
        })
        .run();

    println!("report  what the memo is worth, minimum of 40 rounds:\n{report}");
    let fold = report.get("fold/1M rows").expect("measured");
    let hit = report.get("memo/hit").expect("measured");
    println!(
        "            fold {:.2} us = {:.0}% of a {:.0} us frame · hit {hit:.2} ns · {:.0}x",
        fold / 1e3,
        fold / frame_budget_ns() * 100.0,
        frame_budget_ns() / 1e3,
        fold / hit,
    );
    println!("            spec §14 measured 84.91 us, 85% of the budget, 1.28 ns and 66 301x");

    // A count beside the timing, because the timing is the report and the count is what a gate could
    // be made of: one fold for the whole run, however many frames it was asked for.
    for _ in 0..10 {
        cold.get(data.revision(), || {
            data.cpu.iter().copied().fold(f32::MIN, f32::max)
        });
    }
    println!(
        "            and the count: 10 frames, {} recompute — the memo hit is not a faster fold, \
         it is no fold\n",
        cold.recomputes
    );
}

/// Report: what `Versioned<T>` costs on the read side.
///
/// Spec §14 writes it as **1.005%** and both ADR 0019 and the architecture ticket write it as
/// **1.005×** — the same measurement stated two ways, one of which is a hundredfold larger than the
/// other. 19 202 ns against 19 302 ns is a ratio of 1.005 and an overhead of 0.5%, so **the ADR's
/// form is the right one and §14's per-cent sign is a slip.** Recorded here rather than corrected in
/// the spec, because this file is where the number is now taken.
fn what_the_wrapper_costs() {
    let bare = Indexed::build(MILLION);
    let wrapped = Versioned::new(Indexed::build(MILLION));

    let report = Bench::new(40)
        .case("read/bare", 1_000, || {
            let sum = read_rows(VISIBLE, |r| bare.cpu[OFFSET + r]);
            black_box(sum);
        })
        .case("read/versioned", 1_000, || {
            let sum = read_rows(VISIBLE, |r| wrapped.cpu[OFFSET + r]);
            black_box(sum);
        })
        .run();

    println!("report  what the wrapper costs on the read side, minimum of 40 rounds:\n{report}");
    let bare_ns = report.get("read/bare").expect("measured");
    let wrapped_ns = report.get("read/versioned").expect("measured");
    println!(
        "            versioned / bare = {:.3}x, and zero characters at the call site, because \
         `Deref`\n            is implemented and `DerefMut` deliberately is not.",
        wrapped_ns / bare_ns
    );
    println!(
        "            spec §14 measured 19 202 ns against 19 302 ns. It writes the ratio as \
         `1.005%`\n            and ADR 0019 writes it as `1.005x`; the two differ by a hundredfold \
         and the ADR\n            is right — 1.005x is an overhead of 0.5%.\n"
    );
}

/// Gate and report: one row drawer, three access costs.
///
/// **The load-bearing assumption is carried by neither shape.** O(1) indexed access is prose in the
/// documentation; this is what keeps it honest, and the gate is the step count.
fn the_three_source_shapes() {
    let indexed = Indexed::build(ROWS);
    let chunked = Chunked::build(ROWS);
    let linked = Linked::build(ROWS);

    // The counts first, since they are the gate.
    let indexed_steps = Cell::new(0u64);
    let chunked_steps = Cell::new(0u64);
    let linked_steps = Cell::new(0u64);
    draw_rows(VISIBLE, |r| {
        indexed_steps.set(indexed_steps.get() + 1);
        indexed.cpu[OFFSET + r]
    });
    draw_rows(VISIBLE, |r| chunked.get(OFFSET + r, &chunked_steps));
    draw_rows(VISIBLE, |r| linked.get(OFFSET + r, &linked_steps));
    let (indexed_steps, chunked_steps, linked_steps) =
        (indexed_steps.get(), chunked_steps.get(), linked_steps.get());

    // Twenty rather than forty, and the linked arm is why: it is tens of milliseconds a round at
    // this size, which is the measurement rather than a problem with it.
    let report = Bench::new(20)
        .case("rows/indexed O(1)", 100, || {
            let sum = draw_rows(VISIBLE, |r| indexed.cpu[OFFSET + r]);
            black_box(sum);
        })
        .case("rows/chunked O(k/64)", 1, || {
            let ignored = Cell::new(0);
            let sum = draw_rows(VISIBLE, |r| chunked.get(OFFSET + r, &ignored));
            black_box(sum);
        })
        .case("rows/linked O(k)", 1, || {
            let ignored = Cell::new(0);
            let sum = draw_rows(VISIBLE, |r| linked.get(OFFSET + r, &ignored));
            black_box(sum);
        })
        .run();

    println!(
        "gate    the same row drawer over three access costs, minimum of 20 rounds:\n{report}"
    );
    println!("        steps to draw the same {VISIBLE} rows — this is the gate:");
    println!("          indexed  {indexed_steps:>10}");
    println!(
        "          chunked  {chunked_steps:>10}  {:.0}x",
        chunked_steps as f64 / indexed_steps as f64
    );
    println!(
        "          linked   {linked_steps:>10}  {:.0}x",
        linked_steps as f64 / indexed_steps as f64
    );
    println!(
        "        spec §14 quotes 78 and 780 078 for the two extremes, and **780 078 does not \
         belong to\n        the scene §14 states.** 780 078 is 78 x 10 001, which is a \
         ten-thousand-row list; the\n        scene is a million rows at offset 500 000, where a \
         walked source is 39 003 081 steps —\n        exactly what is measured above. §14's own \
         78.40 ms is consistent with the million (it is\n        100 ns a step, which is one cache \
         miss a node on a scattered list) and not with 780 078\n        (which would be 100 ns a \
         step over a list small enough to be resident). So the step\n        figure is off by 50x \
         against the timing beside it. The **ratio** is the claim either\n        way, and the \
         ratio is unaffected."
    );

    for (case, budget_note) in [
        ("rows/indexed O(1)", "of a 100 us frame"),
        ("rows/chunked O(k/64)", "of a 100 us frame"),
        ("rows/linked O(k)", "of a 100 us frame"),
    ] {
        let ns = report.get(case).expect("measured");
        println!(
            "        {case:<22} {:>10.2} us  {:>7.0}% {budget_note}",
            ns / 1e3,
            ns / frame_budget_ns() * 100.0
        );
    }
    println!(
        "        spec §14 measured 19.77 us (19.8%), 320.85 us (321%) and 78.40 ms (78 400%).\n\
        \x20       **The chunked source is the one to look at.** A rope, a paged cursor or a\n\
        \x20       `Vec<Vec<T>>` is a reasonable thing for an application to have, it is over the\n\
        \x20       whole frame budget on its own, and no type in this module says so. Ticket 19's\n\
        \x20       scene list is what keeps the O(1) assumption honest; this is the number under it."
    );

    // An equality rather than a ratio for the one that is a property of the mechanism: the indexed
    // source takes exactly one step a row, which is what O(1) *means* and is not a measurement.
    assert_eq!(
        indexed_steps, VISIBLE as u64,
        "an O(1) source takes one step a row, by definition"
    );
    assert!(
        linked_steps > indexed_steps * 1_000,
        "the walked source is three orders away, and if it ever is not, this file is measuring \
         something other than what it claims"
    );
}
