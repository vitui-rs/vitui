//! **A picture per component, and the freeze is what says which ones.**
//!
//! Twenty-nine files under `docs/img/components/`, one per built row of `INVENTORY`, each the SVG of
//! a frame this engine composited: `gallery::picture` draws the panel the gallery draws, through the
//! shipped ink, into a headless driver, and asks the driver for its frame.
//!
//! # Gated the way a golden screen is, and for the same reason
//!
//! The file on disk is the expectation and the run is the observation. A component whose drawing
//! changes fails here with the name of the file to look at, and regenerating is a deliberate act —
//! `VITUI_BLESS=1 cargo test -p vitui-components shots` — whose diff a reviewer reads. In CI the
//! blessing is refused outright, because a job that can rewrite the file it compares against is not
//! a gate.
//!
//! # The population is the freeze, not the directory
//!
//! Both directions, because each catches what the other cannot: a built row with no file is a
//! component nobody can see, and a file with no built row is a picture of something this crate no
//! longer ships. The second is the one a rename produces, and it is silent everywhere else.

use std::fs;
use std::path::{Path, PathBuf};

use vitui_components::{INVENTORY, gallery};

/// The tile every component is drawn in.
///
/// One size for all twenty-nine, so that a gallery of pictures is a grid rather than a ransom note.
/// Wide enough for `table`'s four columns and tall enough for `collection`'s rows to show that they
/// scroll; the panel frame is inside it.
const W: u16 = 72;
const H: u16 = 20;

/// Frames before the picture is taken. Three, for the reason `gallery::shot` states: the hover index
/// and the ring settle on the second and the preview pane's answer arrives on a third.
const FRAMES: u32 = 3;

fn home() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("img")
        .join("components")
}

/// `VITUI_BLESS=1`, and never in CI — `crate::golden`'s rule, which this file is a second population
/// of rather than a second mechanism.
fn blessing() -> bool {
    let on = std::env::var_os("VITUI_BLESS").is_some_and(|v| v == "1");
    assert!(
        !(on && std::env::var_os("CI").is_some()),
        "VITUI_BLESS=1 in CI would rewrite the files the job exists to compare against"
    );
    on
}

#[test]
fn every_built_row_of_the_freeze_is_the_picture_on_file() {
    let dir = home();
    fs::create_dir_all(&dir).expect("the picture directory is this repository's own");
    let bless = blessing();

    let mut stale = Vec::new();
    for entry in fs::read_dir(&dir).expect("the directory was just created if it was missing") {
        let path = entry.expect("a directory entry").path();
        let Some(id) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        if path.extension().is_some_and(|e| e == "svg")
            && !INVENTORY.iter().any(|c| c.built && c.id == id)
        {
            stale.push(id.to_string());
        }
    }
    assert!(
        stale.is_empty(),
        "these pictures are of components the freeze does not have built rows for: {stale:?}"
    );

    let mut differ = Vec::new();
    for component in INVENTORY.iter().filter(|c| c.built) {
        let drawn = gallery::picture(component.id, W, H, FRAMES);
        let path = dir.join(format!("{}.svg", component.id));
        let on_file = fs::read_to_string(&path).ok();
        if on_file.as_deref() == Some(drawn.as_str()) {
            continue;
        }
        if bless {
            fs::write(&path, &drawn).expect("the picture directory is writable");
            continue;
        }
        differ.push(match on_file {
            None => format!("{}: no picture on file", component.id),
            Some(_) => format!("{}: the picture on file is a different frame", component.id),
        });
    }

    assert!(
        differ.is_empty(),
        "{}\n\nrun `VITUI_BLESS=1 cargo test -p vitui-components shots` and review the diff",
        differ.join("\n")
    );
}

/// **The size beside the decision.** A 300x80 screen has 24 000 cells and a picture with an element
/// per cell would be unreadable as a diff and unreasonable in a repository; runs are what make these
/// files small, and this is the number that says so.
#[test]
fn a_picture_is_a_few_kilobytes_and_the_whole_gallery_is_under_a_megabyte() {
    let dir = home();
    let mut total = 0u64;
    let mut largest = (String::new(), 0u64);
    for component in INVENTORY.iter().filter(|c| c.built) {
        let path = dir.join(format!("{}.svg", component.id));
        let Ok(meta) = fs::metadata(&path) else {
            continue;
        };
        total += meta.len();
        if meta.len() > largest.1 {
            largest = (component.id.to_string(), meta.len());
        }
    }
    assert!(
        largest.1 < 64 * 1024,
        "the largest picture is {} at {} bytes",
        largest.0,
        largest.1
    );
    assert!(
        total < 1024 * 1024,
        "the twenty-nine pictures are {total} bytes together"
    );
}
