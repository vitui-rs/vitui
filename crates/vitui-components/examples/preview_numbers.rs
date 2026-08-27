//! **The file preview pane's three screens, as numbers.**
//!
//! Components ticket 31. Spec §15, §21. A report and never a gate — `crate::preview::tests` is where
//! every one of these is asserted, and `cargo test` does not run an example.
//!
//! ```text
//! cargo run --release --example preview_numbers -p vitui-components
//! ```

use vitui_components::preview::{
    self, BATCHES, Build, CELLS, CONTENT_WRITES, Key, MAP_AT_A_MILLION, Repeat,
    SECTION_15_TOTAL_MB, SECTION_15_WRITES, SLOT_BYTES, Taken, Wire,
};

fn main() {
    println!("the file preview pane — components 31, spec §15 and §21\n");

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
         \x20 from. Here the partition holds, so `writes` is {} either way and `content` is the\n\
         \x20 counter that is not blind: {} against 0.",
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
        preview::torn(Taken::InsideDraw),
        preview::SELECTIONS,
        preview::torn(Taken::TopOfView),
    );

    println!("\nwhat the three screens are waiting for");
    println!("  {:?}", preview::standing());
}
