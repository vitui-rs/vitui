//! **Components ticket 03's report**: the gate register, the nine counters, and which of them this
//! crate can read.
//!
//! ```text
//! cargo run --release --example gates_numbers -p vitui-components
//! ```
//!
//! # Why this file exists at all
//!
//! Spec §21's table records **`*_numbers.rs` examples on this lineage before C11: 0, against the
//! runtime's 19**. The convention is the runtime's and it has two halves that are easy to conflate:
//!
//! - **It prints numbers a human reads.** Nothing here is a gate — this workspace's CI runs
//!   `cargo test` and three named examples, and this is not one of them, so an `assert!` below is
//!   compiled by `cargo clippy --all-targets` and evaluated by nobody. `gates::REGISTER` cites this
//!   file as an [`Instrument::Report`](vitui_components::gates::Instrument::Report) beside a real
//!   test and never instead of one, which is R15's refinement 2.
//! - **What it does assert is the shape**, which is a count: forty-four rows, eighteen evaluated, nine
//!   counters of which eight are readable. A report that has quietly started measuring something
//!   smaller fails instead of looking good.
//!
//! # There is no timing here, and that is not an omission
//!
//! `vitui-bench` is the workspace's measurement instrument and it is not a dependency of this
//! package — spec §19's constraint C6 is `vitui-runtime` **and nothing else**, and the one
//! dev-dependency that exists (`vitui-alloc-probe`) is there because *no allocation* is a count and a
//! count needs something that counts. So the numbers below are counts, which is what §21 says a gate
//! is made of anyway; the timings for this map live in §20's tables, taken on the prototypes.
//!
//! For the same reason nothing here divides by the frame budget. Runtime ticket 20 made the sixteen
//! `*_numbers.rs` reports read that figure from `crate::ledger` rather than declare it — **a budget
//! figure may not move without a new map decision** — and `ledger` is a private module of
//! `vitui-runtime`. A report on this side that divided by 100 µs would be declaring the figure
//! again, in a nineteenth place, which is the defect ticket 20 removed.

use vitui_alloc_probe::{CountingAllocator, count_allocations};
use vitui_components::counters::{Allocations, Counter, Counters, Tally};
use vitui_components::gates::{EVALUATED, REGISTER, Standing, table};
use vitui_components::ink::Ink as _;
use vitui_runtime::ctx::{Ctx, Driver};
use vitui_runtime::layout::{
    Col,
    Constraint::{Fixed, Weight},
};
use vitui_runtime::{Interest, Role};

#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

/// The screen the counters are taken on.
const W: u16 = 120;
/// See [`W`].
const H: u16 = 40;
/// How many frames the allocation total is taken over. Never divided by.
const FRAMES: u32 = 200;

/// One frame a component could write: a header, twenty lanes with a label and a region each, a
/// footer. Every rectangle comes from `Ctx::area` and the solver, because a package whose dependency
/// list is `vitui-runtime` and nothing else has no other way to obtain one.
fn panel(cx: &mut Ctx<'_, '_>, tally: Option<&mut Tally>) {
    let band = cx.area();
    let body = cx.theme().paint(Role::Body);
    let [header, rows, footer] = Col::new().split(band, [Fixed(1), Weight(1), Fixed(1)]);

    // **The tally is optional and that is load-bearing.** A `Tally` owns a `BTreeSet`, so drawing
    // through one inside an allocation window prices the instrument rather than the frame. The two
    // hundred frames the total is taken over pass `None`, and the counted frame is a further one.
    let mut tally = tally;
    let mut draw = |cx: &mut Ctx<'_, '_>, x: i32, y: i32, s: &str| match tally.as_deref_mut() {
        Some(t) => {
            t.text(cx, x, y, s, body);
        }
        None => {
            cx.text(x, y, s, body);
        }
    };

    draw(cx, header.x, header.y, "a header that does not move");

    let mut lanes = [rows; 20];
    let n = Col::new().split_into(rows, &[Weight(1); 20], &mut lanes);
    for (i, lane) in lanes[..n].iter().enumerate() {
        draw(cx, lane.x, lane.y, "a row that does not move");
        cx.with_key(i as u64, |cx| {
            let id = cx.id();
            let _ = cx.interact(id, *lane, Interest::CLICK.with(Interest::FOCUS));
        });
    }

    draw(cx, footer.x, footer.y, "a footer");
}

fn main() {
    println!("components ticket 03 — the gate register and the nine counters\n");

    // ── the register ─────────────────────────────────────────────────────────────────────────────
    let mut evaluated = 0usize;
    let mut red = 0usize;
    let mut unreachable = 0usize;
    let mut unsubjected = 0usize;
    for row in REGISTER {
        match row.standing {
            Standing::Evaluated { .. } => evaluated += 1,
            Standing::Red { .. } => red += 1,
            Standing::Unreachable { .. } => unreachable += 1,
            Standing::Unsubjected { .. } => unsubjected += 1,
        }
    }
    // **No second copy of the row count here.** This line used to read `assert_eq!(REGISTER.len(),
    // 47, ...)`, and the register reached 56 without it noticing — because `cargo test` compiles an
    // example and evaluates none of it, which is the defect this whole register was built to name.
    // A hardcoded length in a file nothing runs is not a gate, it is a second home for a number, and
    // `gates::tests` already owns that one (`the_numbers_are_contiguous_and_unique` and the spec /
    // lineage split both read `REGISTER.len()`). What stays is the assertion that reads the figure
    // from its single home rather than restating it.
    assert_eq!(
        evaluated, EVALUATED,
        "`EVALUATED` disagrees with the register it counts"
    );

    println!("register  {} rows", REGISTER.len());
    println!("          {evaluated:>3} evaluated      anything `cargo test` runs");
    println!("          {red:>3} red, pinned    a failing set asserted in both directions");
    println!(
        "          {unreachable:>3} unreachable    needs an engine name this crate cannot say"
    );
    println!("          {unsubjected:>3} unsubjected    no component exists to run it over");
    println!(
        "\n          §21 counted 2 of 18 at the branch point and 11 of 18 after C11, both over\n\
         \x20         the prototypes. The line above is the first count taken over shipped code.\n"
    );
    print!("{}", table());

    obligations();

    // **Nothing under the heading is the report, not an empty section.** Row 112 was the last red
    // row and runtime architecture 31 inverted it; a heading with nothing under it reads as a gap in
    // the instrument rather than as a green register, so the zero case says so out loud.
    if red == 0 {
        println!(
            "\nred and pinned: none. Row 112 was the last, and runtime architecture 31 inverted it"
        );
    } else {
        println!("\nred and pinned, each with the set it is pinned on:");
    }
    for row in REGISTER {
        if let Standing::Red {
            failing,
            inverted_by,
            ..
        } = row.standing
        {
            println!("  {:>2}  {}", row.number, row.gate);
            println!("      inverted by `{inverted_by}`");
            for chunk in wrap(failing, 92) {
                println!("      {chunk}");
            }
        }
    }

    println!("\nunreachable across the crate line, with what would have to become public:");
    for row in REGISTER {
        if let Standing::Unreachable { needs, .. } = row.standing {
            println!("  {:>2}  {}", row.number, row.gate);
            for chunk in wrap(needs, 92) {
                println!("      {chunk}");
            }
        }
    }

    // ── the nine counters ────────────────────────────────────────────────────────────────────────
    let mut driver = Driver::headless(W, H).expect("a sink cannot fail to attach");
    driver.frame(|cx| panel(cx, None));
    driver.frame(|cx| panel(cx, None));

    // **A total over the run, never a mean over n**, and the window holds no instrument: a `Tally`
    // owns a `BTreeSet` and allocating one inside the window would price the probe rather than the
    // frame. So the two hundred frames below draw the same panel with no tally at all, and the tally
    // is taken on one further frame after the counter has been read.
    let (_, total) = count_allocations(|| {
        for _ in 0..FRAMES {
            driver.frame(|cx| panel(cx, None));
        }
    });

    let mut tally = Tally::new();
    driver.frame(|cx| panel(cx, Some(&mut tally)));
    let counters = Counters::of(&driver, &tally, Allocations::over(FRAMES, total as u64));

    println!("\ncounters  one frame, {W}x{H}, twenty lanes:");
    for counter in Counter::ALL {
        match counters.get(counter).measured() {
            Some(n) => println!("  {:>22}  {n:>8}", counter.word()),
            None => println!(
                "  {:>22}  {:>8}   unreachable across the crate line",
                counter.word(),
                "—"
            ),
        }
    }
    assert_eq!(
        counters.reachable(),
        8,
        "eight of the nine, and marked is not"
    );
    assert_eq!(
        counters.verbs.get(Counter::Verbs),
        22,
        "the shape is a count and it has changed, so this is a different screen"
    );
    assert_eq!(counters.regions.get(Counter::Regions), 20);
    assert_eq!(counters.merges.get(Counter::Merges), 0);

    println!(
        "\n          allocations is a TOTAL over {FRAMES} frames and is never divided by it: a mean\n\
         \x20         cannot see anything below n, and every prototype printed `allocs / n` with n\n\
         \x20         between 40 and 200. The gate is `tests/gates.rs`, which measures a frame with\n\
         \x20         no instrument in the window, which is what the {total} above is: the panel drawn\n\
         \x20         {FRAMES} times with no `Tally` in it. The counted frame is a further one."
    );
    println!(
        "\n          `marked` is the one counter nobody outside `vitui-engine` can read: its\n\
         \x20         damage structure is `pub(crate)` from top to bottom and `Presented` carries no\n\
         \x20         cell count. It is the last of the nine, and the sentinel beside it answers\n\
         \x20         since components 40 — see `counters::sentinel`."
    );

    // ── the sentinel, answering ──────────────────────────────────────────────────────────────────
    //
    // **It used to refuse here**, and the refusal was asserted: three barriers, `Unreachable`, and a
    // `catch_unwind` making sure an answer of 0 could not be mistaken for a green gate. Components
    // 40 read it off `crate::runner::Pen` instead of off the screen ADR 0023 will not open — the
    // recorder is the stricter instrument, because a verb that skips the caller's `Ink` makes this
    // number *larger* — so what is printed is a count, on a surface with a row nobody wrote.
    let mut driver = Driver::headless(20, 3).expect("a sink cannot fail to attach");
    let mut pen = vitui_components::runner::Pen::new(20, 3);
    driver.frame(|cx| {
        let body = cx.theme().paint(Role::Body);
        pen.pad_to(cx, 0, 0, "two rows of three", 20, body);
        pen.pad_to(cx, 0, 1, "written", 20, body);
    });
    let left = vitui_components::counters::sentinel(pen.canvas()).get(Counter::Distinct);
    assert_eq!(left, 20, "the third row of twenty");
    println!("\nsentinel  {left} cells of 60 written by nobody — the third row of a 20x3 surface,");
    println!("          which is `no cell never` as a count. See `REGISTER`'s row 7.");
}

/// Break a long line for the report, on spaces, at `width`.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && line.len() + 1 + word.len() > width {
            out.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}

/// **The nine queries §17's obligations are, in one place.**
///
/// Row 30 of the register is *O1-O7 as queries over `INVENTORY`*, and until components ticket 44
/// each of them printed only in its own ticket's report — `doc_numbers` for O1, `gallery_numbers`
/// for O2, `golden_numbers` for O3, `contract_numbers` for O4, `scene_numbers` for O5 and
/// `volume_numbers` for O6. A reader who wanted *where do the obligations stand* had six files to
/// open, which is the shape §17 wrote the freeze to end.
///
/// Nine lines, not seven files. The per-obligation reports keep their own evidence; this is the
/// roll-up beside the register row that owns it. **O7's evidence is in another crate** —
/// `crates/vitui-apps/examples/` — which is what makes it the one line here nothing in
/// `vitui-components` could have produced on its own.
fn obligations() {
    use vitui_components::obligations::{
        APPLIED, AXIS_SCENES, DOC_TESTED, GOLDENS, KEYBOARD_DOCUMENTED, KEYBOARD_REGISTERED,
        PANELS, VOLUME_MEASURED, Verdict, o1, o2_everything_built_has_a_panel,
        o2_nothing_shown_is_absent_from_the_freeze, o3, o4, o5, o6,
        o7_everything_declared_has_an_application, o7_nothing_exercised_is_absent_from_the_freeze,
    };

    let declared = vitui_components::consumer::declared(|relative| {
        let path =
            std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join(relative);
        std::fs::read_to_string(&path).unwrap_or_default()
    });
    println!("\nthe obligations, as queries over the freeze (register row 30)");
    let all = [
        ("O1  a page with a compiled example", o1(DOC_TESTED)),
        (
            "O2a nothing shown is absent from it",
            o2_nothing_shown_is_absent_from_the_freeze(PANELS),
        ),
        (
            "O2b everything built has a panel  ",
            o2_everything_built_has_a_panel(PANELS),
        ),
        ("O3  a golden per construction     ", o3(GOLDENS)),
        (
            "O4  documented == registered      ",
            o4(KEYBOARD_DOCUMENTED, KEYBOARD_REGISTERED),
        ),
        ("O5  a scene per declared axis     ", o5(AXIS_SCENES)),
        ("O6  60 Hz at a million inputs     ", o6(VOLUME_MEASURED)),
        (
            "O7a nothing applied is absent     ",
            o7_nothing_exercised_is_absent_from_the_freeze(APPLIED),
        ),
        (
            "O7b everything declared is in one ",
            o7_everything_declared_has_an_application(&declared, APPLIED),
        ),
    ];
    for (name, verdict) in all {
        match verdict {
            Verdict::Met { over } => println!("  {name}  met over {over}"),
            Verdict::Unmet {
                over,
                failing,
                inverted_by,
                ..
            } => println!("  {name}  UNMET {failing} of {over}, inverted by `{inverted_by}`"),
        }
    }
    println!(
        "\n          O5 is the one left, and §17 says it is worth more than the other four\n\
         \x20         together. O6 is the sixth and O7 the seventh, both stated after the map\n\
         \x20         closed; O6's population is derived from the freeze and O7's is read out of\n\
         \x20         the source, because the `built` column is a claim and it has been wrong."
    );
}
