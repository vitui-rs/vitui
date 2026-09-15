//! **A picture per application, and `APPS` is what says which ones.**
//!
//! The population is the example targets, joined through [`APPS`] — the same value the site's data
//! file and the README table are derived from, so an application added to `examples/` arrives owing
//! a picture and one that is renamed fails the join instead of leaving a stale file behind.
//!
//! # What this can gate and what it cannot
//!
//! `cargo test` does not run an example, so nothing here can *take* a picture: these are twenty-one
//! separate binaries and each writes its own under `--shot`. What a test can do is join the two
//! populations and read what is on disk, which is what catches the two failures that matter — an
//! application with no picture, and a picture of an application that no longer exists.
//!
//! Whether a picture is **current** is the third failure and it is `scripts/shots.sh --check`, which
//! re-takes all twenty-one into a temporary directory and diffs. That runs in CI beside the test
//! command, because a file nobody regenerates is a file that drifts.

use std::fs;
use std::path::{Path, PathBuf};

use vitui_apps::{APPS, pictures};

/// The one application whose picture is **not** checked for drift, and why.
///
/// `latency` draws a measured duration, so its screen differs between two runs of the same binary
/// by design. A timing is a report and never a gate; the picture is written and reviewed like every
/// other one, and what is not asserted is that two runs agree. `scripts/shots.sh` is where that
/// exclusion is spelled, and the test below is what keeps the two from drifting apart.
const CARRIES_A_STOPWATCH: &str = "latency";

/// The one application with no picture, and the reason it has none.
///
/// `caps` draws no frame at all: it attaches, reads what the terminal claimed, detaches and prints.
/// There is no composited frame for a picture to be of. Named here rather than skipped silently,
/// because an absence with no reason beside it is indistinguishable from an omission.
const DRAWS_NO_FRAME: &str = "caps";

fn dir() -> PathBuf {
    pictures::path("any")
        .parent()
        .expect("the picture path has a directory")
        .to_path_buf()
}

#[test]
fn every_application_that_draws_a_frame_has_a_picture() {
    let mut missing = Vec::new();
    for app in APPS {
        if app.name == DRAWS_NO_FRAME {
            assert!(
                !pictures::path(app.name).exists(),
                "{} draws no frame and has a picture anyway",
                app.name
            );
            continue;
        }
        if !pictures::path(app.name).exists() {
            missing.push(app.name);
        }
    }
    assert!(
        missing.is_empty(),
        "these applications have no picture: {missing:?}\n\nrun `scripts/shots.sh`"
    );
}

#[test]
fn no_picture_is_of_an_application_this_crate_no_longer_has() {
    let dir = dir();
    let Ok(entries) = fs::read_dir(&dir) else {
        panic!("{} is missing; run `scripts/shots.sh`", dir.display());
    };
    let mut stray = Vec::new();
    for entry in entries {
        let path = entry.expect("a directory entry").path();
        if path.extension().is_none_or(|e| e != "svg") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("an svg file has a name")
            .to_string();
        if !APPS.iter().any(|a| a.name == stem) {
            stray.push(stem);
        }
    }
    assert!(
        stray.is_empty(),
        "these pictures are of applications `APPS` does not list: {stray:?}"
    );
}

/// **The size beside the decision**, and the worst case is a real screen rather than a bound.
///
/// A picture is one element per *run*, so its size is a measure of how much of the screen is
/// distinctly coloured: `theatre` draws a photograph, where no two neighbouring cells share a style
/// and every cell is its own element. That is the honest ceiling and it is a fifth of a megabyte;
/// every other application is a few tens of kilobytes.
#[test]
fn the_pictures_are_text_sized_and_the_photograph_is_the_ceiling() {
    let mut total = 0u64;
    let mut largest = (String::new(), 0u64);
    for app in APPS {
        let Ok(meta) = fs::metadata(pictures::path(app.name)) else {
            continue;
        };
        total += meta.len();
        if meta.len() > largest.1 {
            largest = (app.name.to_string(), meta.len());
        }
    }
    assert_eq!(
        largest.0, "theatre",
        "the largest picture is no longer the photograph, which is the case the ceiling is about"
    );
    assert!(
        largest.1 < 512 * 1024,
        "{} is {} bytes",
        largest.0,
        largest.1
    );
    assert!(
        total < 2 * 1024 * 1024,
        "the twenty pictures are {total} bytes together"
    );
}

/// **The drift check's exclusion and its reason are joined.**
///
/// A gate that excludes something is owed the name of what it excluded, in the place a reader will
/// look — and a shell script and a Rust constant are two places. This is the join: the script must
/// exclude exactly the file this file names, so renaming the application fails here rather than
/// quietly leaving a second picture unchecked.
#[test]
fn the_script_excludes_the_one_picture_that_carries_a_stopwatch() {
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("scripts")
        .join("shots.sh");
    let source = fs::read_to_string(&script).expect("the script is this repository's own");
    let needle = format!("--exclude={CARRIES_A_STOPWATCH}.svg");
    assert!(
        source.contains(&needle),
        "{} does not exclude {needle}",
        script.display()
    );
    assert_eq!(
        source.matches("--exclude=").count(),
        1,
        "the script excludes something this file does not name"
    );
}

/// The pictures live where the repository's other images live, which is what the site stages from.
#[test]
fn the_pictures_are_under_the_repositorys_own_image_directory() {
    let dir = dir();
    assert!(
        dir.ends_with(Path::new("docs/img/apps")),
        "{}",
        dir.display()
    );
}
