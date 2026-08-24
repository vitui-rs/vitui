//! **Runtime ticket 15's report**: what a sizing function costs, what the dry run costs for the same
//! answer, and what the one expensive sizing function costs when it is treated as a sizing question
//! instead of as a derived aggregate.
//!
//! ```text
//! cargo run --release --example sizing_numbers -p vitui-runtime
//! ```
//!
//! Four arms, and each of them is a number somebody would otherwise assume:
//!
//! 1. **Sizing a dialog to its contents.** Spec §12 measured 569 ns, and the point of the number is
//!    that it is *small* — the stated cost of having no measure pass is not what makes the decision
//!    hard. Drift is.
//! 2. **The same answer from a dry run**, which is a second whole frame. §12 measured **13.3×**.
//! 3. **The honest question a dry run cannot be asked.** A bounded list of 65 535 rows — every row
//!    `u16` can express — sized by arithmetic against the same list sized by drawing it. §12
//!    measured **5.81 ms against 0.96 ns, 6 051 331×**.
//! 4. **Column auto-fit**, which is not a sizing question at all: **17.60 µs at 1k, 17.69 ms at 1M,
//!    and 1.73 ns memoised — 10 231 195×.** The fold is at the call site here, where a container
//!    with a `measure` method would have hidden it.
//!
//! And one gate rather than a number, run here as well as in the module's own tests, because a
//! report whose fixture has drifted from its own sizing function is measuring two different layouts:
//! [`vitui_runtime::sizing::check`] over the dialog, at every width the report uses.

use std::hint::black_box;

use vitui_bench::Bench;
use vitui_engine::Rect;
use vitui_runtime::ctx::Driver;
use vitui_runtime::data::{Memo, Versioned};
use vitui_runtime::layout::text;
use vitui_runtime::sizing;
use vitui_runtime::{Ctx, Role};

/// The frame budget every ratio here is against.
const FRAME_NS: f64 = 100_000.0;

/// The width the dialog is sized at, which is the width §12's 569 ns was measured at.
const DIALOG_W: u16 = 78;

/// Every row `u16` can express, which is the largest question a dry run can honestly be asked.
const HONEST_ROWS: u16 = u16::MAX;

// ── the dialog: a sizing function, and a component laid out from the same arithmetic ────────────

struct Dialog {
    title: &'static str,
    body: &'static str,
    buttons: [&'static str; 2],
}

const DIALOG: Dialog = Dialog {
    title: "Unsaved changes",
    body: "This document has changes that have not been written to disk. Closing it now discards \
           them, and there is no undo once the window is gone.",
    buttons: ["Discard", "Cancel"],
};

/// The sizing function: the same `&data`, the width it is about to be given, integers out, **and no
/// `Ctx`** — so it cannot draw, cannot claim an identity and cannot route.
fn dialog_height(d: &Dialog, width: u16) -> u16 {
    // A one-column frame either side, and a blank row above and below the buttons.
    let inner = width.saturating_sub(2);
    let body = u16::try_from(text::wrap_height(d.body, inner)).unwrap_or(u16::MAX);
    // frame · title · rule · body · blank · buttons · frame
    1 + 1 + 1 + body + 1 + 1 + 1
}

/// The component, laid out **from the arithmetic its sizing function publishes** — one expression
/// and not two, which is the obligation that makes the detector below able to pass at all.
fn dialog(cx: &mut Ctx<'_, '_>, d: &Dialog) {
    let title = cx.theme().paint(Role::Title);
    let body = cx.theme().paint(Role::Body);
    let rule = cx.theme().glyph(vitui_runtime::Glyph::HLine);
    let width = cx.area().w;
    let inner = width.saturating_sub(2);

    cx.text(1, 1, d.title, title);
    cx.fill(Rect::new(1, 2, inner, 1), rule, body);
    let mut row = 3i32;
    for line in text::wrap(d.body, inner) {
        cx.text(1, row, line, body);
        row += 1;
    }
    // The blank row, then the buttons — placed from the same count the sizing function published.
    row += 1;
    let mut x = 1i32;
    for label in d.buttons {
        cx.text(x, row, label, body);
        x += i32::from(text::width(label)) + 2;
    }
    // The frame's own last row, which is what makes the total one more than the buttons' row.
    cx.fill(Rect::new(0, row + 1, width, 1), rule, body);
}

// ── the honest question: a bounded list, sized by arithmetic ────────────────────────────────────

/// The arithmetic sizing function a list has. **This is the 0.96 ns**, and there is nothing in it to
/// make cheaper.
fn list_height(rows: u16, row_height: u16) -> u16 {
    rows.saturating_mul(row_height)
}

fn list(cx: &mut Ctx<'_, '_>) {
    let body = cx.theme().paint(Role::Body);
    for row in cx.visible_rows() {
        cx.text(0, row, "a row of a bounded list", body);
    }
}

fn main() {
    println!("runtime ticket 15 — sizing functions, and what the dry run costs\n");

    // The gate first: a report whose fixture drifted from its own sizing function would be timing
    // two different layouts and calling the difference a ratio.
    sizing::check(
        20..=120,
        |w| dialog_height(&DIALOG, w),
        |cx| dialog(cx, &DIALOG),
    )
    .assert();
    println!(
        "gate    the dialog's sizing function agrees with its component at every width from 20\n\
        \x20       to 120. That is the gate every component publishing a sizing function owes,\n\
        \x20       and it is one call: `sizing::check(widths, claim, draw).assert()`.\n"
    );

    let claimed = dialog_height(&DIALOG, DIALOG_W);
    println!("        the dialog is {claimed} rows at {DIALOG_W} columns\n");

    // ── arms 1 and 2, round-robined in one bench so the ratio is between two moments of the same
    //    machine rather than two moments of its life. ─────────────────────────────────────────────
    let mut driver = Driver::headless(DIALOG_W, claimed).expect("a sink cannot fail to attach");
    let mut report = None;
    driver.frame(|cx| {
        report = Some(
            Bench::new(40)
                .case("size/sizing function", 1_000, || {
                    black_box(dialog_height(black_box(&DIALOG), black_box(DIALOG_W)));
                })
                .case("size/dry run", 100, || {
                    let measured = cx.measured(DIALOG_W, claimed, |inner| dialog(inner, &DIALOG));
                    black_box(measured.extent.h);
                })
                .run(),
        );
    });
    let report = report.expect("the frame ran");

    println!("report  one dialog, two ways of asking its height, minimum of 40 rounds:\n{report}");
    let fun = report.get("size/sizing function").expect("measured");
    let dry = report.get("size/dry run").expect("measured");
    println!(
        "        sizing function {fun:>10.2} ns   {:>6.3}% of a {:.0} us frame\n\
        \x20       dry run         {dry:>10.2} ns   {:>6.3}% of a {:.0} us frame\n\
        \x20       dry run / sizing function = {:.1}x, and spec §12 measured 13.3x.",
        fun / FRAME_NS * 100.0,
        FRAME_NS / 1e3,
        dry / FRAME_NS * 100.0,
        FRAME_NS / 1e3,
        dry / fun,
    );
    println!(
        "        **The dry run is a second whole frame**, and the ratio is what it costs for an\n\
        \x20       answer the function beside the component already had. It is kept as the\n\
        \x20       detector and never as the layout mechanism — a component that reached for it\n\
        \x20       to lay something out would pay this on every frame.\n\
        \x20       Both arms here are the *same* dialog, which is the honest pairing and the\n\
        \x20       smaller ratio: §12's 13.3x puts a dense screen's dry run against a dialog's\n\
        \x20       sizing function. The ratio grows with the screen, because the left-hand side\n\
        \x20       does not — a sizing function measures one component's text and a dry run\n\
        \x20       redraws everything that is on the screen.\n"
    );

    // ── arm 3: the honest question. ─────────────────────────────────────────────────────────────
    let mut driver = Driver::headless(40, 4).expect("a sink cannot fail to attach");
    let mut report = None;
    driver.frame(|cx| {
        report = Some(
            Bench::new(5)
                .case("bounded/arithmetic", 10_000, || {
                    black_box(list_height(black_box(HONEST_ROWS), black_box(1)));
                })
                .case("bounded/dry run", 1, || {
                    let measured = cx.measured(40, HONEST_ROWS, list);
                    black_box(measured.extent.h);
                })
                .run(),
        );
    });
    let report = report.expect("the frame ran");

    println!(
        "report  a bounded list of {HONEST_ROWS} rows — every row `u16` can express — minimum of 5 rounds:\n{report}"
    );
    let arithmetic = report.get("bounded/arithmetic").expect("measured");
    let drawn = report.get("bounded/dry run").expect("measured");
    println!(
        "        arithmetic {arithmetic:>12.2} ns\n\
        \x20       dry run    {drawn:>12.2} ns   {:.2} ms\n\
        \x20       ratio      {:>12.0}x, and spec §12 measured 6 051 331x.\n\
        \x20       The dry run here allocates its own {HONEST_ROWS}-row discard surface every\n\
        \x20       time, which a dry run does — the surface is what the answer is read off.",
        drawn / 1e6,
        drawn / arithmetic,
    );
    println!(
        "        **And the dry run measures the clip, not the content.** This one is honest only\n\
        \x20       because the surface was made as tall as the list; a virtualised million-row\n\
        \x20       list under an eighty-row clip reports eighty, and `u16` cannot hold a million\n\
        \x20       anyway — which is also the trait form's second death.\n"
    );

    // ── arm 4: column auto-fit, and the fold at the call site. ──────────────────────────────────
    let thousand = Versioned::new(column(1_000));
    let million = Versioned::new(column(1_000_000));
    let mut memo: Memo<u16> = Memo::new();
    // Warm, so that the third arm measures a hit and not the fold it is being compared with.
    let warm = *memo.get(million.revision(), || {
        sizing::auto_fit(million.iter().map(String::as_str))
    });

    let report = Bench::new(20)
        .case("auto-fit/1k rows", 10, || {
            black_box(sizing::auto_fit(thousand.iter().map(String::as_str)));
        })
        .case("auto-fit/1M rows", 1, || {
            black_box(sizing::auto_fit(million.iter().map(String::as_str)));
        })
        .case("auto-fit/memo hit", 10_000, || {
            black_box(*memo.get(million.revision(), || unreachable!("the memo is warm")));
        })
        .run();

    println!("report  column auto-fit, minimum of 20 rounds:\n{report}");
    let k = report.get("auto-fit/1k rows").expect("measured");
    let m = report.get("auto-fit/1M rows").expect("measured");
    let hit = report.get("auto-fit/memo hit").expect("measured");
    println!(
        "        1k rows   {:>10.2} us   {:>7.2}% of a {:.0} us frame\n\
        \x20       1M rows   {:>10.2} ms   {:>7.0}% of a {:.0} us frame\n\
        \x20       memo hit  {hit:>10.2} ns\n\
        \x20       1M / memo hit = {:.0}x, and spec §12 measured 10 231 195x.\n\
        \x20       the column fits to {warm} columns, and across every read the memo arm made\n\
        \x20       the fold ran {} time — which is the count, and the count is the gate.",
        k / 1e3,
        k / FRAME_NS * 100.0,
        FRAME_NS / 1e3,
        m / 1e6,
        m / FRAME_NS * 100.0,
        FRAME_NS / 1e3,
        m / hit,
        memo.recomputes,
    );
    println!(
        "        **The one expensive sizing function is not a sizing question.** It is a derived\n\
        \x20       aggregate over the data, it recomputes when the data changes rather than when a\n\
        \x20       frame runs, and the fold is written at the call site — where a `measure` method\n\
        \x20       on a container would have hidden a million-row walk behind a layout call.\n"
    );

    // ── and what the extent costs, which is why a real frame does not maintain one. ─────────────
    let strings = column(24);
    let report = Bench::new(40)
        .case("extent/off — the same bytes, unmeasured", 1_000, || {
            let mut total = 0usize;
            for s in &strings {
                total += s.len();
            }
            black_box(total);
        })
        .case("extent/on — one display width a verb", 1_000, || {
            let mut total = 0u32;
            for s in &strings {
                total += u32::from(text::width(s));
            }
            black_box(total);
        })
        .run();
    let off = report
        .get("extent/off — the same bytes, unmeasured")
        .expect("measured");
    let on = report
        .get("extent/on — one display width a verb")
        .expect("measured");
    println!("report  what maintaining the drawn extent adds, minimum of 40 rounds:\n{report}");
    println!(
        "        {} verbs cost {:.2} us of grapheme walking they would not otherwise do —\n\
        \x20       {:.2}% of a {:.0} us frame, against {:.3} us of touching the same bytes. Per\n\
        \x20       verb that is {:.1} ns, so a screen with the dense scene's few hundred verbs\n\
        \x20       is the single-digit per cent the glossary states.\n\
        \x20       **That is why the extent is `None` in a real frame**: it is maintained only\n\
        \x20       while something asks, and today the only thing that asks is `Ctx::measured`.",
        strings.len(),
        (on - off) / 1e3,
        (on - off) / FRAME_NS * 100.0,
        FRAME_NS / 1e3,
        off / 1e3,
        (on - off) / strings.len() as f64,
    );
}

/// A column of cells whose widest is the one the fold has to find, and it is **last** — a fold that
/// stopped early would look correct against a column whose widest was first.
fn column(rows: usize) -> Vec<String> {
    let mut cells: Vec<String> = (0..rows).map(|i| format!("row-{i}")).collect();
    if let Some(last) = cells.last_mut() {
        *last = String::from("the widest cell in the whole column, and it is at the end");
    }
    cells
}
