//! The Ghostty arm: drive the engine inside a real Ghostty window and read its screen back.
//!
//! **One executable, two halves.** With no arguments this is the *driver*; with `--scene 01` it is
//! the *scene*, and the driver launches the scene by re-running its own [`std::env::current_exe`].
//! That is not a trick to save a file: it makes the two halves the same build by construction, where
//! a sibling binary path can silently be yesterday's.
//!
//! ```sh
//! cargo run --example ghostty          # writes REPORT-ghostty.md, opens and closes one window
//! ```
//!
//! The scene, the readiness handshake and the comparison are in [`common`], shared with the tmux arm
//! — because ticket 05 asks for the eleven attribute facts on *two* families, and two copies of the
//! scene would make a disagreement between the arms unattributable. What is here is what only this
//! arm can do: AppleScript, a window server, and a temp directory nobody reports the path of.
//!
//! # It reports, it does not gate
//!
//! It needs a window server, a macOS automation grant, and a Ghostty whose `vt` dump format is
//! **undocumented** — found by probing `+validate-config`, so a 1.4 may change it. `compare/`
//! already argued this case: *four external projects' versions cannot gate this repository's pull
//! requests.* Substituting "when Ghostty ships 1.4" leaves the sentence unchanged. The gate is
//! `cargo test` over the committed fixtures, and this is the soak that produces them.
//!
//! # The refusals, written before the first assertion
//!
//! Ticket 04 predicted the failure mode and it is the reason this file is shaped around guards:
//! `screen -X hardcopy` exits 0 and writes a zero-byte file, and *an empty capture compares equal
//! against a blank region and the row goes green.* Every step below therefore fails loudly rather
//! than continuing with less:
//!
//! - the scene signals readiness by **writing a file**, and the driver waits for it. A fixed delay
//!   is how a capture races the paint, and a raced capture is the empty one.
//! - creating the window must add **exactly one** terminal, or the set difference is not an address.
//! - the capture must produce **exactly one** new `screen.txt`, or the file found is somebody else's.
//! - the dump goes through [`vitui_conform::parse`] with the row count the scene declared, which is
//!   where an empty or short screen becomes `FAILED`.
//!
//! # Window geometry is not ours to set
//!
//! `surface configuration` offers `font size` and no rows or columns, and `TIOCSWINSZ` changes what
//! the *program* believes rather than what the emulator renders. So the scene declares eleven rows
//! and draws them at the top left of whatever window it is given; the rest of the alt screen is
//! default-styled blanks, which the `vt` dump keeps. The size is recorded in the report rather than
//! demanded. **The tmux arm is the one that can set it**, which is the sharpest difference between
//! the two and is why both exist.

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime};

use common::{
    AnswersInBand, Arm, Excluded, SCENES, clear_handshake, header, publish, save_if_asked, scene,
    scene_argv, section, trailer, wait_for_quiescence,
};
use vitui_conform::Dialect;

/// What stands between the engine and Ghostty in the window this photographs.
///
/// # Why the second variant exists, and it is a finding that put it there
///
/// The tmux arm found that `capture-pane -e` re-serialises **SGR 53 as `5:3`** — tmux stored the
/// overline and wrote it back as a parameter that means blink. A control probe with no engine in it
/// reproduces that from a raw `printf`, so it is tmux's capture writer and not the engine.
///
/// That leaves the question ticket 05 actually needs answered, and no dump of tmux's own grid can
/// answer it: **does tmux *forward* overline to the terminal it is running inside?** `capture-pane`
/// and the redraw path are different code in tmux, and `attrs_dropped` is about what a terminal
/// renders. So this variant puts tmux in the middle and photographs **Ghostty**: tmux parses the
/// engine's bytes and Ghostty parses tmux's, and since Ghostty alone already agrees 11/11, a
/// disagreement here is tmux's forwarding.
///
/// It is not a fourth arm, it is this arm with a different thing inside the window — which is the
/// point. The scene, the handshake and the comparison are unchanged, so the two runs are comparable
/// by construction.
enum Through {
    /// Nothing. Ghostty parses the engine's bytes directly.
    Nothing,
    /// A tmux session on a socket named for this process.
    Tmux(String),
}

impl Through {
    /// The report's heading, and the name of the file it is written to.
    fn title(&self) -> &'static str {
        match self {
            Self::Nothing => "Ghostty",
            Self::Tmux(_) => "Ghostty-via-tmux",
        }
    }

    /// What this run's rows are evidence about.
    fn measures(&self) -> &'static str {
        match self {
            Self::Nothing => {
                "Ghostty. Its own cell state, re-serialised by the emulator that holds it"
            }
            Self::Tmux(_) => {
                "**tmux's forwarding**, read through Ghostty. tmux parses the engine's \
                              bytes and Ghostty parses tmux's, and Ghostty alone agrees 11/11 — so a \
                              disagreement here is what tmux passed on. This is the only instrument \
                              here that can see it: `capture-pane` and tmux's redraw path are \
                              different code, and `attrs_dropped` is about what is rendered"
            }
        }
    }

    /// Which terminal answers `CSI 6n` for this variant, and the two answers are **different
    /// subjects**.
    ///
    /// Scene 05's answers come back in band on the scene's own tty, so they come from the innermost
    /// terminal in the path — and for the tmux variant that is tmux, not the Ghostty window it is
    /// drawn in. This is where the two axes come apart: the *photograph* sees what tmux forwarded
    /// to Ghostty, which is the only thing this variant exists to see, and the *cursor report*
    /// never leaves tmux and duplicates the plain tmux arm's subject exactly.
    ///
    /// Printed rather than excluded, because the rows are real answers about a real terminal. What
    /// would be dishonest is the heading over them, and this is the field that stops it saying
    /// Ghostty.
    fn answers_in_band(&self) -> AnswersInBand {
        match self {
            Self::Nothing => AnswersInBand {
                who: "Ghostty",
                why: "Ghostty is an endpoint, so nothing sits between the scene's tty and it",
            },
            Self::Tmux(_) => AnswersInBand {
                who: "tmux",
                why: "**not Ghostty, and this is where the arm's two halves come apart.** A \
                      question asked in band is answered by the innermost terminal, so it never \
                      leaves tmux — \
                      where this arm's *photograph* is the only instrument in this directory that \
                      can see what tmux forwarded onward. These rows duplicate the plain tmux \
                      arm's exactly, and the column is headed with who answered rather than with \
                      this arm's title",
            },
        }
    }

    /// The command line the window runs.
    fn command(&self, ready: &Path, which: &str) -> Result<String, String> {
        // Joined here rather than in `common`: this one goes inside an AppleScript string
        // literal and tmux's goes inside a tmux command line, and neither quoting rule is one a
        // shared helper could guess. The argv is the shared part.
        let scene = scene_argv(which)?.join(" ");
        Ok(match self {
            Self::Nothing => scene,
            // `-f /dev/null` for the same reason the tmux arm gives: the result must be about tmux
            // and not about a `terminal-features` line in somebody's config. `-e` rather than trusting
            // the server to inherit our environment, because a server that already existed would not
            // have.
            Self::Tmux(socket) => format!(
                "{} -L {socket} -f /dev/null new-session -e CONFORM_READY={} {scene}",
                tmux_path()?,
                ready.display()
            ),
        })
    }

    /// The rows this run will not compare, and why.
    ///
    /// **The tmux variant lost its whole point to the entry it earned.** It was built to answer
    /// *does tmux forward overline*, it answered no, and production ticket 10 then wired
    /// `attrs_dropped` on to the wire — so the engine no longer sends SGR 53 to a terminal that
    /// answers XTVERSION `tmux …`, and this arm can no longer take the measurement. `FAILED` would
    /// blame tmux for a decision of ours; `by design` says whose it is.
    ///
    /// Not a defect to fix here. An arm that asked the engine what to expect would be checking the
    /// engine against itself, which is the arrangement `conform/` exists to break, and
    /// `fixtures/ghostty-1.3.1-via-tmux-3.7c-scene01-attrs.vt` is the capture taken while the bit
    /// was still on the wire.
    fn not_compared(&self) -> &'static [(&'static str, Excluded, &'static str)] {
        match self {
            // Ghostty's `vt` dump was probed with a raw `printf` before this arm was written and it
            // carries all eleven bits; Ghostty has no `quirks.rs` entry, so the engine withholds
            // nothing. Every row is compared and a disagreement means what it says.
            Self::Nothing => &[],
            Self::Tmux(_) => &[(
                "overline",
                Excluded::ByDesign,
                "`quirks.rs`'s fourth entry, which **this arm is what earned** — and production \
                 ticket 10 then wired the mask, so the engine stopped sending SGR 53 to tmux and \
                 this run can no longer see what tmux would have done with it. The capture that \
                 can is committed",
            )],
        }
    }

    /// Anything the reader needs that only this variant knows.
    fn notes(&self) -> Vec<String> {
        match self {
            Self::Nothing => Vec::new(),
            Self::Tmux(_) => vec![
                "**Two parsers in series.** The engine detected *tmux's* capabilities, not \
                 Ghostty's, so a disagreement has two candidate causes — what the engine chose to \
                 emit, and what tmux passed on. The row tells them apart: an attribute the engine \
                 never sent is missing, where one tmux mangled arrives wrong"
                    .to_string(),
            ],
        }
    }

    /// Shut down anything this variant started that closing the window does not.
    ///
    /// A tmux **server** outlives the client in it. Closing the Ghostty window kills the client and
    /// leaves the session detached, so without this a run leaves a server behind — and the next
    /// run's socket-collision refusal would fire on our own litter.
    fn cleanup(&self) {
        if let Self::Tmux(socket) = self {
            let _ = Command::new("tmux")
                .args(["-L", socket, "kill-server"])
                .output();
        }
    }
}

/// Where `tmux` is, absolutely.
///
/// **Not the bare name, and this cost a run to learn.** A Ghostty window's `command` is executed with
/// the environment the GUI application was launched with, whose `PATH` is `/usr/bin:/bin:/usr/sbin:
/// /sbin` — Homebrew's `/opt/homebrew/bin` is not on it. The first `--through-tmux` run therefore
/// opened a window, failed to exec `tmux`, and reported *the scene never presented a frame*, which is
/// the readiness timeout doing its job and saying nothing about the cause.
///
/// So the path is resolved **here**, in a process that has the developer's `PATH`, and the refusal is
/// about the thing that is missing rather than about a frame that never came.
///
/// # Errors
///
/// tmux not being on this process's `PATH` at all.
fn tmux_path() -> Result<String, String> {
    let out = Command::new("sh")
        .args(["-c", "command -v tmux"])
        .output()
        .map_err(|e| format!("looking for tmux: {e}"))?;
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    match out.status.success() && !path.is_empty() {
        true => Ok(path),
        false => Err("tmux is not on PATH, and `--through-tmux` is the arm that needs it".into()),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--scene") => scene(args.get(1).map(String::as_str)),
        Some("--through-tmux") => {
            run(Through::Tmux(format!("conform-{}", std::process::id())));
        }
        _ => run(Through::Nothing),
    }
}

fn run(through: Through) {
    if let Err(why) = drive_all(&through) {
        eprintln!("FAILED: {why}");
        std::process::exit(1);
    }
}

/// Every scene, each in its own window, into one report. See the kitty arm's `drive` for why a
/// window per scene rather than a scene switch inside one.
fn drive_all(through: &Through) -> Result<(), String> {
    let mut report = String::new();
    let mut sections = String::new();
    let (mut asked, mut failures) = (0usize, 0usize);
    let mut arm: Option<Arm> = None;

    for which in SCENES {
        let (this, captured, bytes, size) = drive(through, which)?;
        if arm.is_none() {
            report.push_str(&header(&this, &bytes));
        }
        let (text, a, f) = section(&this, which, &captured, &size);
        sections.push_str(&text);
        asked += a;
        failures += f;
        arm = Some(this);
    }

    let arm = arm.expect("SCENES is not empty");
    report.push_str(&sections);
    report.push_str(&trailer());
    publish(&arm, &report, asked, failures)
}

fn drive(
    through: &Through,
    which: &str,
) -> Result<(Arm, common::Capture, Vec<u8>, String), String> {
    let ready = std::env::temp_dir().join(format!("conform-ready-{}-{which}", std::process::id()));
    clear_handshake(&ready);
    let command = through.command(&ready, which)?;

    let was = terminal_ids()?;
    let launched = Instant::now();
    osascript(&format!(
        r#"tell application "Ghostty"
             set cfg to new surface configuration from {{command:"{}", environment variables:{{"CONFORM_READY={}"}}}}
             new window with configuration cfg
           end tell"#,
        command,
        ready.display()
    ))?;

    // The set difference is the address. Ticket 04 established that window *indices* are z-order and
    // shift between `osascript` calls, and that `focused terminal` raises -1728 on a fresh window, so
    // neither is usable; the terminal id is stable for the surface's life.
    let terminal = new_terminal(&was)?;

    let outcome = capture_and_compare(through, which, &terminal, &ready, launched);

    // Closed before the result is unwrapped. A failed run that leaves a window open is a run that
    // needs a human before the next one can start.
    let _ = osascript(&format!(
        r#"tell application "Ghostty" to close terminal id "{terminal}""#
    ));
    through.cleanup();
    clear_handshake(&ready);
    outcome
}

/// Wait for the frame, photograph it, parse it, compare it, and render the report.
fn capture_and_compare(
    through: &Through,
    which: &str,
    terminal: &str,
    ready: &Path,
    launched: Instant,
) -> Result<(Arm, common::Capture, Vec<u8>, String), String> {
    let size = wait_for_quiescence(ready, which)?;
    // **The window is not photographed for scene 05**, and that is `common::capture`'s decision
    // rather than this arm's: the terminal answers `CSI 6n` in band on the scene's own tty, so the
    // **capture** — the `write_screen_file` action, the undocumented `vt` writer and the hunt for
    // the file it left in a fresh temp directory — is out of its path entirely.
    //
    // **The automation grant is not**, and saying so was wrong the first time this comment was
    // written. This arm opens its window with `osascript`, addresses it by set difference over
    // `terminal_ids`, and closes it with `osascript` — three Apple Events before any scene runs. It
    // is the *capture surface* that scene 05 does without, and for an arm that could launch a
    // terminal some other way that is the whole of the requirement. Ghostty is not that arm.
    let mut capture_elapsed = Duration::ZERO;
    let (captured, bytes) = common::capture(which, ready, Dialect::Ecma48, || {
        let (bytes, elapsed) = capture(terminal)?;
        capture_elapsed = elapsed;
        Ok(bytes)
    })?;
    save_if_asked(which, Dialect::Ecma48, &bytes)?;
    let mut notes = vec![
        match which {
            "05" | "06" => format!(
                "**Launch to answer:** {} ms — reported, never gated. **This scene is not \
                 photographed:** the terminal answers in band on the scene's own tty, so the \
                 `write_screen_file` action and the undocumented `vt` writer are out of its path. \
                 **The automation grant is not** — this arm still opens, addresses and closes its \
                 window over AppleScript, which is three Apple Events before the scene runs. What \
                 an in-band scene does without is the *capture surface*",
                launched.elapsed().as_millis()
            ),
            _ => format!(
                "**Launch to capture:** {} ms, of which the capture round trip alone was {} ms — \
                 reported, never gated. It is AppleScript round trips and a window opening",
                launched.elapsed().as_millis(),
                capture_elapsed.as_millis()
            ),
        },
        "**Geometry:** not ours to set. `surface configuration` offers a font size and no rows \
         or columns, so the scene draws at the top left of whatever it is given"
            .to_string(),
    ];
    notes.extend(through.notes());
    let arm = Arm {
        title: through.title(),
        version: osascript(r#"tell application "Ghostty" to get version"#)
            .unwrap_or_else(|_| "unknown".into()),
        mechanism: "`write_screen_file:…,vt`",
        measures: through.measures(),
        answers_in_band: through.answers_in_band(),
        not_compared: through.not_compared(),
        // Every capture from this arm is a re-serialisation of the emulator's own cell state, so
        // the style is in the capture and scene 01 is answerable. See [`Arm::no_style`].
        no_style: None,
        notes,
    };
    Ok((arm, captured, bytes, size))
}

/// Ask Ghostty to write its screen out, and find the file it wrote.
///
/// `paste` and not `copy`: `copy` clobbers the user's system clipboard, and the path it pastes lands
/// in the scene's input, which the scene drains and ignores. The `plain|html|vt` argument grammar is
/// undocumented; `vt` is the only one of the three that keeps resolved colours — `html` renders
/// reverse as `filter: invert(100%)` and throws the colours away.
///
/// Returns the bytes and how long the round trip took. **The second half is not decoration**: it is
/// the measured reason production ticket 05 gives for `mode 2026`'s force-flush limits being out of
/// reach of this instrument. A limit of 150 ms cannot be bracketed by a capture that costs more than
/// that to ask for.
fn capture(terminal: &str) -> Result<(Vec<u8>, Duration), String> {
    // Ghostty writes `screen.txt` into a **fresh random directory** under the user's temp directory,
    // and nothing reports the path back to us. So: mark the time, act, and take what appeared —
    // with the count checked, because "the newest one" silently picks a stranger's file.
    let mark = SystemTime::now();
    let started = Instant::now();
    let ok = osascript(&format!(
        r#"tell application "Ghostty" to perform action "write_screen_file:paste,vt" on terminal id "{terminal}""#
    ))?;
    if ok.trim() != "true" {
        return Err(format!("Ghostty refused write_screen_file: {ok:?}"));
    }

    let mut found: Vec<PathBuf> = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        found = screen_files_since(mark);
        if !found.is_empty() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    match found.len() {
        0 => Err("Ghostty said it wrote a screen file and none appeared".into()),
        1 => {
            let bytes =
                std::fs::read(&found[0]).map_err(|e| format!("reading the capture: {e}"))?;
            Ok((bytes, started.elapsed()))
        }
        n => Err(format!(
            "{n} screen files appeared at once; none of them is safely ours"
        )),
    }
}

/// Every `<temp>/*/screen.txt` modified since `mark`.
fn screen_files_since(mark: SystemTime) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(std::env::temp_dir()) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|e| e.path().join("screen.txt"))
        .filter(|p| {
            std::fs::metadata(p)
                .and_then(|m| m.modified())
                .is_ok_and(|t| t >= mark)
        })
        .collect()
}

// ── Addressing ───────────────────────────────────────────────────────────────────────────────────

fn terminal_ids() -> Result<Vec<String>, String> {
    let out = osascript(r#"tell application "Ghostty" to get id of every terminal"#)?;
    Ok(out
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect())
}

/// The one terminal that was not there before, insisting there is exactly one.
fn new_terminal(was: &[String]) -> Result<String, String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut fresh: Vec<String> = Vec::new();
    while Instant::now() < deadline {
        fresh = terminal_ids()?
            .into_iter()
            .filter(|id| !was.contains(id))
            .collect();
        if !fresh.is_empty() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    match fresh.len() {
        0 => Err("no new terminal appeared; Ghostty may not have opened the window".into()),
        1 => Ok(fresh.remove(0)),
        n => Err(format!(
            "{n} terminals appeared at once — the set difference is not an address, so this run \
             cannot say which window it photographed"
        )),
    }
}

fn osascript(script: &str) -> Result<String, String> {
    let out = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| format!("osascript: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "osascript failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}
