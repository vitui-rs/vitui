//! **The file preview pane's three screens, as numbers.**
//!
//! Components tickets 31 and 32. Spec §15, §21. A report and never a gate — `crate::preview::tests` is where
//! every one of these is asserted, and `cargo test` does not run an example.
//!
//! ```text
//! cargo run --release --example preview_numbers -p vitui-components
//! ```

use std::time::{Duration, Instant};

use vitui_components::counters::{Allocations, Counter, Counters};
use vitui_components::preview::{
    self, BATCHES, Build, Bump, CELLS, CONTENT_WRITES, DecodeAt, Key, MAP_AT_A_MILLION, Offset,
    PICKER_DECOMPOSED, REVISION_FRAMES, Repeat, SECTION_15_DECODE_UNITS, SECTION_15_PHOTOGRAPH,
    SECTION_15_STEADY, SECTION_15_TOTAL_MB, SECTION_15_WRITES, SLOT_BYTES, TAB_SWITCHES, Taken,
    VOLUMES, Wire,
};

fn main() {
    println!("the file preview pane — components 31 and 32, spec §15 and §21\n");

    println!("the screen");
    let shape = preview::shape(Build::correct());
    println!(
        "  {} x {} = {} cells; the body is {} x {} = {} ({:.2}% of the screen)",
        preview::W,
        preview::H,
        CELLS,
        preview::BODY_W,
        preview::BODY_H,
        preview::BODY_CELLS,
        preview::body_percent(),
    );
    println!(
        "  writes {}   distinct {}   verbs {}   regions {}   content {}",
        shape.writes, shape.distinct, shape.verbs, shape.regions, shape.content_writes,
    );
    println!(
        "  §15 states 1 650 writes against 4 166 on a screen that did not partition its rectangle;\n\
         \x20 the difference is {} = {} rows x {} columns, which is where the document's width comes\n\
         \x20 from. On a steady frame the partition holds, so `writes` is {} on every arm and\n\
         \x20 `content` is the counter that is not blind: {}. The one frame where `writes` moves is\n\
         \x20 the shrink, below.",
        SECTION_15_WRITES[1] - SECTION_15_WRITES[0],
        preview::BODY_H,
        preview::LINE_COLUMNS,
        CELLS,
        CONTENT_WRITES,
    );

    println!("\nscene 25 — a re-sort under a preview pane, 200 files");
    let position = preview::resort(Build::correct().keyed(Key::Position));
    let identity = preview::resort(Build::correct().keyed(Key::Identity));
    println!(
        "  {} of {} positions move, and the three that do not are stated",
        position.moved,
        preview::FILES
    );
    println!(
        "  {:<16}  {:>8}  {:>8}  {:>8}",
        "the key is", "asks", "spawns", "wrong"
    );
    for run in [&position, &identity] {
        println!(
            "  {:<16}  {:>8}  {:>8}  {:>8}",
            format!("{:?}", run.key),
            run.requests,
            run.spawns,
            run.wrong,
        );
    }
    println!(
        "  both arms ask on every frame; the question still matches, so nothing posts, so nothing\n\
         \x20 wakes, so no frame corrects it."
    );

    println!("\nscene 24 — a directory streamed in seven batches under a sort");
    println!(
        "  {:<16}  {:>12}  {:>14}  {:>14}",
        "the key is", "wrong after", "wrong frames", "longest run"
    );
    for key in [Key::Position, Key::Identity] {
        let run = preview::batches(Build::correct().keyed(key));
        println!(
            "  {:<16}  {:>9} of {}  {:>14}  {:>14}",
            format!("{key:?}"),
            run.wrong_after,
            BATCHES,
            run.wrong_frames,
            run.longest_wrong_run,
        );
    }
    println!("  no user is in it: seven deliveries and not one keystroke.");

    println!("\nscene 23 — twenty selections through a directory of photographs");
    print!("{}", preview::wire_table());
    let slow = preview::photographs(Repeat::SlowerThanTheDecode);
    println!(
        "  a picture is {} bytes = {:.2} KiB, which §15 prints as 536 KB by truncating it;\n\
         \x20 twenty of them is {:.3} MB, which is what {:.1} MB/s over {} ms requires.\n\
         \x20 §15 prints {} MB beside that rate — the truncated 536 read as decimal kB, times twenty.",
        preview::PICTURE_BYTES,
        Wire::picture_kib(),
        slow.total_mb(),
        slow.rate() as f64 / 1_000_000.0,
        slow.millis,
        SECTION_15_TOTAL_MB,
    );

    println!("\nthe offset belongs to neither side — a 4 000-row file, an 800-row file, 74 rows");
    print!("{}", preview::offset_table());
    println!(
        "  a per-file map is right in both directions and nothing releases it: {} bytes at a\n\
         \x20 million entries against one slot's {}, and the shipped `HashMap` is {} — buckets\n\
         \x20 round to a power of two and the pair pads to sixteen bytes.",
        MAP_AT_A_MILLION,
        SLOT_BYTES,
        preview::measured_map_bytes(1_000_000),
    );

    println!("\nthe torn frame — a status row on each side of the pane");
    println!(
        "  taken inside the draw: {} torn frames of {}\n  taken at the top of the view: {}",
        preview::torn(Taken::InsideTheDraw),
        preview::SELECTIONS,
        preview::torn(Taken::AtTheTopOfTheView),
    );

    println!("\nthe frame the document shrinks on — the only one the unclamped spelling shows on");
    println!(
        "  {:<24}  {:>8}  {:>9}  {:>8}",
        "the offset is", "writes", "content", "verbs"
    );
    for spelling in Offset::ALL {
        let shape = preview::shape_after_the_shrink(spelling);
        println!(
            "  {:<24}  {:>8}  {:>9}  {:>8}",
            spelling.name(),
            shape.writes,
            shape.content_writes,
            shape.verbs,
        );
    }
    println!(
        "  §15 prices it at {} writes against {}; this screen writes {} against {}. The magnitudes\n\
         \x20 are two different screens and **the gap is {} in both** — 74 rows of 34 columns,\n\
         \x20 which is where the document's width comes from. Over an unbounded extent the area has\n\
         \x20 no tail, so the cells the body cannot write are cells nobody writes.",
        SECTION_15_WRITES[1],
        SECTION_15_WRITES[0],
        CELLS,
        preview::UNCLAMPED_WRITES,
        CONTENT_WRITES,
    );

    println!("\nthe answer's own identity — eight bytes on the payload");
    println!(
        "  {:<10}  {:>9}  {:>8}  {:>9}  {:>8}",
        "the job is", "answered", "refused", "landings", "content"
    );
    for honest in [true, false] {
        let m = preview::mismatched(honest);
        println!(
            "  {:<10}  {:>9}  {:>8}  {:>9}  {:>8}",
            if honest { "honest" } else { "lying" },
            m.answered,
            m.refused,
            m.landings,
            m.content_writes,
        );
    }
    println!(
        "  a question that was never asked is not out of order, so no test on the answer's arrival\n\
         \x20 could have seen it."
    );

    println!("\nrequirement 9 — decode units on the app thread");
    println!(
        "  on the worker {}   called from the view {}   (§15 states {} for a larger document)",
        preview::decoded(DecodeAt::OnTheWorker),
        preview::decoded(DecodeAt::InTheView),
        SECTION_15_DECODE_UNITS,
    );

    println!("\nR02's sweep, refused twice — {TAB_SWITCHES} tab switches");
    println!(
        "  {:<28}  {:>8}  {:>8}  {:>7}",
        "the slot lives", "spawns", "decodes", "folds"
    );
    for swept in [false, true] {
        let s = preview::sweeps(swept);
        println!(
            "  {:<28}  {:>8}  {:>8}  {:>7}",
            if swept {
                "in the widget registry"
            } else {
                "in application state"
            },
            s.spawns,
            s.decodes,
            s.folds,
        );
    }
    println!(
        "  a job's lifetime is the question's, a memo's is the data's, neither is the widget's."
    );

    println!(
        "\nR09's edit — a landing that did not happen is still a drop ({REVISION_FRAMES} frames)"
    );
    println!(
        "  {:<24}  {:>9}  {:>7}  {:>10}",
        "the revision advances", "landings", "folds", "writes"
    );
    for bump in [Bump::OnTheLanding, Bump::EveryFrame] {
        let r = preview::revisions(bump);
        println!(
            "  {:<24}  {:>9}  {:>7}  {:>10}",
            match bump {
                Bump::OnTheLanding => "on the landing",
                Bump::EveryFrame => "every frame",
            },
            r.landings,
            r.folds,
            r.writes,
        );
    }
    println!(
        "  the two screens are cell-identical, which is why no counter that reads a cell sees it."
    );

    println!("\nthe steady frame, and the one with a photograph in it");
    println!(
        "  {:>10}  {:>8}  {:>7}  {:>8}  {:>9}  {:>8}",
        "entries", "writes", "verbs", "regions", "content", "customs"
    );
    for volume in VOLUMES {
        let shape = preview::steady(volume);
        println!(
            "  {:>10}  {:>8}  {:>7}  {:>8}  {:>9}  {:>8}",
            volume, shape.writes, shape.verbs, shape.regions, shape.content_writes, shape.customs,
        );
    }
    let photo = preview::photograph_frame(VOLUMES[2]);
    println!(
        "  {:>10}  {:>8}  {:>7}  {:>8}  {:>9}  {:>8}   (a photograph selected)",
        VOLUMES[2], photo.writes, photo.verbs, photo.regions, photo.content_writes, photo.customs,
    );
    println!(
        "  §15 states {} writes / {} verbs steady and {} / {} / {} with a photograph. The two frame\n\
         \x20 magnitudes are a prototype's screen; the customs are exact, because the pane's\n\
         \x20 viewport is 198 x 74 and a picture spends one a cell.",
        SECTION_15_STEADY[0],
        SECTION_15_STEADY[1],
        SECTION_15_PHOTOGRAPH[0],
        SECTION_15_PHOTOGRAPH[1],
        SECTION_15_PHOTOGRAPH[2],
    );

    println!("\nwhat a frame costs — a report and never a gate (§21: a timing is a report)");
    let rounds = 60;
    let (text, photo) = round_robin(rounds);
    println!(
        "  {:<34}  {:>10}  {:>10}",
        "the pane is showing", "minimum", "of budget"
    );
    for (what, took, budget) in [
        ("a text document", text, FULL_SCREEN_BUDGET),
        ("a photograph, 14 652 customs", photo, FULL_SCREEN_BUDGET),
    ] {
        println!(
            "  {:<34}  {:>8.1} us  {:>9.2}x",
            what,
            took.as_secs_f64() * 1e6,
            took.as_secs_f64() * 1e6 / f64::from(budget),
        );
    }
    println!(
        "  minimum of {rounds} interleaved rounds, because a machine warming up is not a difference\n\
         \x20 between arms.\n\
         \x20 **§15's *238 us against a 100 us budget* does not reproduce, and it is the class that\n\
         \x20 moved rather than the clock.** A pane that writes every cell of its rectangle is\n\
         \x20 §20's **full-screen** class and not its typical-damage one, so the budget is\n\
         \x20 {FULL_SCREEN_BUDGET} us — and the photograph frame is *inside* it. The ticket says *over budget,\n\
         \x20 damaging nothing, not fixed here*; nothing here is over budget, and reclassifying it\n\
         \x20 the other way to make the sentence true would be `crate::ledger`'s forbidden move\n\
         \x20 one crate down."
    );

    println!("\nthe nine counters, and the one this crate cannot read");
    let alloc = Allocations::over(1, 0);
    let counters = shape_counters(alloc);
    for counter in Counter::ALL {
        println!("  {:<16}  {:?}", counter.word(), counters.get(counter));
    }
    println!(
        "  `marked` is the engine's alone and no re-export reaches it — `crate::counters`'s own\n\
         \x20 barrier, and it is why §15's *0 marked on a steady frame* is recorded here as\n\
         \x20 unreachable rather than printed as a zero. A figure defaulted to zero is a counter\n\
         \x20 that prints 0 when it means nobody counted.\n\
         \x20 `allocations` is 0 and it is gated in `tests/budget.rs`, over the **component** and\n\
         \x20 not over this screen: `Screen` formats a String a row and allocates {} a frame, which\n\
         \x20 is what an instrument does.",
        preview::SCREEN_ALLOCATIONS_A_FRAME,
    );

    println!("\n`file_picker` = `collection` + `overlay` + the pane");
    let shut = preview::picker(false);
    let open = preview::picker(true);
    println!("  shut  {shut:?}");
    println!("  open  {open:?}");
    println!(
        "  and the six decompose as {PICKER_DECOMPOSED:?}: the owner's shut face, the shell's blur\n\
         \x20 position, the collection's one entry however many rows it has, and the pane's three."
    );

    println!("\nwhat the three screens stand on");
    println!("  {:?}", preview::standing());
}

/// §20's full-screen class, in microseconds. This screen writes every cell of 300 x 80.
const FULL_SCREEN_BUDGET: u32 = 1_000;

/// **The two arms interleaved, minimum of each.** A machine warming up is not a difference between
/// arms — `crate::series` learned that with a 4% that turned out to be noise with a sign.
fn round_robin(rounds: u32) -> (Duration, Duration) {
    let mut text = preview::many(1_000);
    let mut photo = preview::many(1_000);
    photo[0].kind = preview::Kind::Photograph;
    photo[0].rows = u32::from(preview::BODY_H);

    let mut a = preview::Screen::open(Build::correct());
    let mut b = preview::Screen::open(Build::correct());
    for (screen, files) in [(&mut a, &text), (&mut b, &photo)] {
        screen.frame(files, 0);
        screen.settle();
        screen.frame(files, 0);
    }
    let (mut best_a, mut best_b) = (Duration::MAX, Duration::MAX);
    for _ in 0..rounds {
        for (screen, files, best) in [(&mut a, &text, &mut best_a), (&mut b, &photo, &mut best_b)] {
            let started = Instant::now();
            let _ = screen.frame(files, 0);
            *best = (*best).min(started.elapsed());
        }
    }
    text.clear();
    (best_a, best_b)
}

/// The nine counters over one steady frame, so that the one this crate cannot read says so out loud
/// rather than being absent from the table.
fn shape_counters(allocations: Allocations) -> Counters {
    preview::counters(allocations)
}
