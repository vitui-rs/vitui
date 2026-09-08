//! The Alacritty arm: drive the engine inside a real Alacritty window and read Alacritty's own grid.
//!
//! **One executable, two halves**, the shape every arm here has: with no arguments this is the
//! driver, with `--scene 01` it is the scene, and the driver launches the scene by re-running its
//! own [`std::env::current_exe`]. The scene, the readiness handshake and the comparison come from
//! [`common`] and are byte-for-byte the ones the other five arms run.
//!
//! ```sh
//! cd conform && cargo run --example alacritty      # writes REPORT-alacritty.md
//! ```
//!
//! # The capture surface, established before anything was written
//!
//! a later pass predicted this arm would have **no** capture surface — Alacritty has no
//! remote-control socket, no screen-dump facility, and no AppleScript dictionary — and told the
//! session to establish that rather than assume it. It is wrong, and the thing that makes it wrong
//! is on Alacritty's own command line: **`--ref-test`**. Alacritty's reference tests are captured by
//! serialising the `Term`'s grid to `./grid.json` when the last window closes, one JSON object per
//! cell, and `--ref-test` is what turns that on for any run.
//!
//! **It is the strongest capture surface in this directory, and it is the weakest.** Strongest,
//! because no serialiser of the emulator's stands between the cell and the reader: kitty's *the dump
//! has no spelling for a dotted underline* and WezTerm's *the serialiser dropped one attribute of a
//! styled cell* cannot happen here, and an underline **colour** is a field of the cell rather than a
//! sequence that has to survive a re-serialisation — where WezTerm's capture has no spelling for
//! SGR 58 at all. Weakest, because a grid is what the terminal **stores** — which is the tmux arm's caveat
//! arriving on an emulator, and the reason this arm's two missing attributes are argued from the
//! shipped source and a debug log rather than from their absence.
//!
//! # Three things about that surface that had to be found by running it
//!
//! 1. **The dump is written when the last window closes**, so the capture is taken by *ending* the
//!    run rather than by asking a socket during it. See [`photograph`]: the scene is stopped, the
//!    window closes, Alacritty writes the file and exits. The engine is never detached, which is
//!    what keeps the alternate screen — and the scene on it — in the grid that gets written.
//! 2. **It writes to `./`, and Alacritty `chdir`s to the home directory on macOS** before the event
//!    loop starts (`main.rs`, unconditional). So the driver hands it a `HOME` of its own under
//!    `$TMPDIR`, which is also this arm's `-f /dev/null`: no user config file is found there either.
//! 3. **`scrolling.history` must be zero or the file is the scrollback.** `write_ref_test_results`
//!    calls `Grid::initialize_all` first, so a default 10 000-line history is materialised in full:
//!    the first probe wrote **111 MB** for a four-row screen.
//!
//! # The environment is built rather than inherited, and here that is load-bearing
//!
//! The Terminal.app arm neutralises its environment and records that it moves no row *as that suite
//! stands*. On this arm it moves everything. Alacritty spawns the command it is given directly, so
//! the child inherits the launching terminal's variables — and this workspace is developed inside
//! Ghostty, which exports `TERM_PROGRAM=ghostty`. Run without [`NEUTRALISED`], the engine inside the
//! Alacritty window detects **Ghostty**, consults `quirks.rs` for Ghostty, and the report is about a
//! terminal that is not in the window. See that constant.
//!
//! # What it found, and none of the three is a number
//!
//! - **Blink and overline are not stored**, and this is the first arm where *cannot ask* was ruled
//!   out rather than settled for: `alacritty_terminal::term::cell::Flags` has no bit for either, and
//!   `alacritty -vvv` prints `Term got unhandled attr: BlinkSlow` while SGR 53 produces no
//!   `Setting attribute` line at all. Two mechanisms, one outcome, both observed. `quirks.rs`'s
//!   seventh entry.
//! - **Mode 2026 is reported reset while it is set**, which WezTerm did first — and here the cause
//!   is readable: `Term::report_private_mode` answers `NamedPrivateMode::SyncUpdate` with a
//!   **constant** `ModeState::Reset`, while synchronised output is implemented one layer down in
//!   `vte`'s parser. The flag and the reporter are in different crates. No quirk entry, for
//!   WezTerm's reason: there is no route to take around it.
//! - **This terminal answers no XTVERSION**, so the quirk table cannot recognise it by a query the
//!   way it recognises tmux and kitty — and `TERM` is `xterm-256color` here, because Alacritty falls
//!   back to that when no `alacritty` terminfo entry exists and this machine has none. The
//!   recognition rule is `ALACRITTY_WINDOW_ID`.

mod common;

use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use common::{
    AnswersInBand, Arm, Excluded, SCENES, clear_handshake, header, publish, save_if_asked, scene,
    scene_argv, section, trailer, wait_for_quiescence,
};
use vitui_conform::Dialect;

/// Where this arm expects to find Alacritty.
///
/// The Homebrew cask was **disabled on 2026-09-01** for failing the macOS Gatekeeper check, so the
/// binary this arm was built against is `cargo install alacritty --locked` — which is a build from
/// the published source rather than a signed bundle, and is recorded in the report as such.
const ALACRITTY: &str = "alacritty";

/// The window size this arm asks for, in cells, and then insists on.
///
/// Eleven rows plus room so nothing scrolls, and wide enough that an attribute leaking past its
/// label would have somewhere to leak to. **The third emulator arm that can be handed a size** —
/// `window.dimensions` is in cells — and the second that can check it, though it checks it from a
/// different place than Terminal.app does: the scene's own stamp reports the surface the engine was
/// given, which is what Alacritty told the pty. See [`insist_on_size`].
const REQUESTED: (u16, u16) = (80, 24);

/// Every environment variable the engine's detection reads that Alacritty does not set itself.
///
/// # On this arm it is load-bearing, and on the Terminal.app arm it was luck
///
/// That arm's own note says its neutralisation moves no row *as this suite stands*. This one moves
/// every row of every scene. Alacritty execs the command it is handed with the environment it was
/// started with, and this workspace is developed inside Ghostty — which exports
/// `TERM_PROGRAM=ghostty` and `TERM_PROGRAM_VERSION=1.3.1`. Inherited, `detect.rs` identifies the
/// terminal in the Alacritty window as **Ghostty**, `quirks.rs` is consulted for Ghostty, and every
/// number here is about a terminal that is not on the screen. Observed on the first probe.
///
/// **`TERM` and `COLORTERM` are deliberately not here.** Alacritty sets both itself, after this list
/// has been applied and before it spawns anything: `TERM` to `alacritty` where that terminfo entry
/// exists and to `xterm-256color` where it does not, and `COLORTERM` to `truecolor` unconditionally.
/// Removing them would change nothing and would read as a claim that it does not.
///
/// **`TERMINFO` is here for a different reason than the rest**, and it is the reason `TERM` is not:
/// it is not read by the engine at all, and it is what Alacritty's own `terminfo_exists` check
/// consults. Left inherited, Ghostty's terminfo directory decides which `TERM` this run gets.
const NEUTRALISED: &[&str] = &[
    "TERM_PROGRAM",
    "TERM_PROGRAM_VERSION",
    "TERMINFO",
    "TERMINAL_EMULATOR",
    "TERMUX_VERSION",
    "TMUX",
    "NO_COLOR",
    "VITUI_FORCE_LEGACY_SGR",
    "VITUI_FORCE_COLOR",
    "VITUI_GLYPHS",
];

/// Scene 01's two rows this arm will not compare, declared before the run.
///
/// **They were `FAILED` for exactly one run**, which is the kitty arm's shape and the same trade:
/// that run earned `quirks.rs` its seventh entry, the engine now withholds both bits from a terminal
/// it recognises as Alacritty, and the rows read `by design` from here on. The measurement that
/// earned the entry lives in `fixtures/alacritty-0.17.0-scene01-attrs.json`, captured while the
/// engine still sent them — see [`common::Excluded::ByDesign`], and do not regenerate that fixture
/// to make something pass.
const BY_DESIGN: &[(&str, Excluded, &str)] = &[
    (
        "blink",
        Excluded::ByDesign,
        "`Flags` has no bit for blink, and `alacritty -vvv` prints `Term got unhandled attr: \
         BlinkSlow` for the SGR that asks for one — parsed by `vte`, discarded by the `Term`. \
         `quirks.rs`'s seventh entry, so the engine no longer sends it here",
    ),
    (
        "overline",
        Excluded::ByDesign,
        "`Flags` has no bit for overline and `vte`'s `Attr` has no variant for it either, so SGR 53 \
         produces no `Setting attribute` line at all — not parsed rather than parsed and dropped. \
         The same quirk entry",
    ),
];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--scene") => scene(args.get(1).map(String::as_str)),
        _ => {
            if let Err(why) = drive() {
                eprintln!("FAILED: {why}");
                std::process::exit(1);
            }
        }
    }
}

/// An Alacritty started by this run, with the private home it writes into.
///
/// Not tidiness: the guards below return early, and an Alacritty left running holds a window on the
/// user's screen. The home directory goes with it — it is this run's alone, named for the process,
/// and a stale one would be the next run's capture.
struct Instance {
    child: Child,
    home: PathBuf,
}

impl Drop for Instance {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.home);
    }
}

/// Every scene, each in its own Alacritty window, into one report.
///
/// **A window per scene**, for the reason the whole directory is arranged around and one this arm
/// has no choice about anyway: the capture is taken by closing the window.
fn drive() -> Result<(), String> {
    let mut report = String::new();
    let mut sections = String::new();
    let (mut asked, mut failures) = (0usize, 0usize);
    let mut arm: Option<Arm> = None;

    for which in SCENES {
        let (this, captured, bytes, size) = one_scene(which)?;
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

fn one_scene(which: &str) -> Result<(Arm, common::Capture, Vec<u8>, String), String> {
    // An argv, for the kitty arm's reason: Alacritty's `-e` takes a program and its arguments, and
    // handed one string it would look for a file whose name ends in `--scene 01`.
    let command = scene_argv(which)?;
    let ready = std::env::temp_dir().join(format!("conform-ready-{}-{which}", std::process::id()));
    clear_handshake(&ready);

    // **Named for this process, and refused if it already exists.** This is where `grid.json` will
    // land, so a directory this run did not create is a directory whose `grid.json` may be somebody
    // else's screen — the kitty arm's refusal about a socket, one file down.
    let home =
        std::env::temp_dir().join(format!("conform-alacritty-{}-{which}", std::process::id()));
    if home.exists() {
        return Err(format!(
            "{} already exists; this run cannot say whose grid it would be reading",
            home.display()
        ));
    }
    std::fs::create_dir_all(&home).map_err(|e| format!("creating {}: {e}", home.display()))?;

    let version = version()?;
    let mut alacritty = Command::new(ALACRITTY);
    alacritty
        .arg("--ref-test")
        // Or `grid.json` is the scrollback rather than the screen: 111 MB on the first probe.
        .args(["-o", "scrolling.history=0"])
        // Geometry in cells, checked afterwards against what the engine was given.
        .args(["-o", &format!("window.dimensions.columns={}", REQUESTED.0)])
        .args(["-o", &format!("window.dimensions.lines={}", REQUESTED.1)])
        .args(["--title", "vitui-conform"])
        // Where `./grid.json` lands, and the config file this run does not read. See the module docs.
        .env("HOME", &home)
        .env("CONFORM_READY", &ready)
        // What Alacritty itself says goes nowhere: a warning on our stderr would be mistaken for the
        // driver's. The scene's own output goes to the pty Alacritty gives it.
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    for name in NEUTRALISED {
        alacritty.env_remove(name);
    }
    let child = alacritty
        .arg("-e")
        .args(&command)
        .spawn()
        .map_err(|e| format!("{ALACRITTY}: {e} — this arm needs alacritty on PATH"))?;
    // **Mutable because the photograph reaps it**, which is the one thing this arm's capture needs
    // that no other arm's does: the grid is written as Alacritty exits, so the capture is complete
    // when the child has been waited for. See [`photograph`].
    let mut instance = Instance {
        child,
        home: home.clone(),
    };
    let outcome = capture_and_compare(which, &version, &ready, &home, &mut instance);
    // Dropped before the result is unwrapped, like every other arm closes its window: a failed run
    // that leaves a window on the user's screen is a run that needs a human before the next one.
    drop(instance);
    clear_handshake(&ready);
    outcome
}

/// Wait for the frame, close the window, read the grid it wrote, compare it, render the report.
fn capture_and_compare(
    which: &str,
    version: &str,
    ready: &Path,
    home: &Path,
    instance: &mut Instance,
) -> Result<(Arm, common::Capture, Vec<u8>, String), String> {
    let size = wait_for_quiescence(ready, which)?;
    insist_on_size(&size)?;

    let dialect = Dialect::AlacrittyGrid;
    let (captured, bytes) = common::capture(which, ready, dialect, || {
        photograph(&mut instance.child, home)
    })?;
    save_if_asked(which, dialect, &bytes)?;

    Ok((arm(version), captured, bytes, size))
}

/// Refuse a window that is not the size this arm asked for.
///
/// **Checked from inside the window rather than from the window server**, which is the only place
/// this arm can check it: the scene's stamp carries the surface the engine was given, and that is
/// what Alacritty put in the pty. Scene 04 stamps `raw` because a raw-byte probe has no engine to
/// ask, so this passes anything that is not a geometry — the ruler row is what stands in there, as
/// that scene's own note says.
fn insist_on_size(size: &str) -> Result<(), String> {
    let Some((columns, lines)) = size.split_once('x') else {
        return Ok(());
    };
    let want = format!("{}x{}", REQUESTED.0, REQUESTED.1);
    if (columns.parse(), lines.parse()) != (Ok(REQUESTED.0), Ok(REQUESTED.1)) {
        return Err(format!(
            "the scene was given a {size} surface and this arm asked for {want}; a capture at a \
             size nobody asked for is a capture of somebody else's window"
        ));
    }
    Ok(())
}

/// Take the photograph, which on this arm means ending the run.
///
/// # The capture is the window closing, and that is the surface rather than a choice
///
/// `WindowContext::write_ref_test_results` is called from one place in `alacritty`'s event loop:
/// the branch where the last window has gone and the process is about to exit. There is no socket to
/// ask during the run and no signal that triggers it. So the scene is stopped, Alacritty sees the
/// pty close, the window goes, the file is written, and the process exits — in that order, and this
/// function waits for the end of it rather than for a delay.
///
/// # It is stopped and then killed, and the two signals are not one gesture
///
/// `SIGSTOP` first, because scene 04 repaints every [`common::REPAINT`] and a process killed
/// mid-repaint could leave a half-erased screen in the grid. Stopping it freezes the writer at once
/// and leaves whatever it had already written in the pty, which Alacritty then drains and paints
/// during the settle below. `SIGKILL` after, because a stopped process still has to go.
///
/// **Never a clean exit**, and that is the load-bearing half. The engine leaves the alternate screen
/// when a `Screen` is dropped, so a scene asked to finish politely would hand back a grid of the
/// primary screen — an empty shell — and the capture would be of nothing at all. What is wanted is
/// the screen as it stood, which is the screen a killed process leaves behind.
fn photograph(alacritty: &mut Child, home: &Path) -> Result<Vec<u8>, String> {
    let scene = child_of(alacritty.id())?;
    signal("STOP", scene)?;
    // Long enough for Alacritty to drain the pty and paint what the scene had already written, and
    // it is a settle rather than a measurement — nothing here is timed.
    std::thread::sleep(common::QUIESCENT);
    signal("KILL", scene)?;

    let grid = home.join("grid.json");
    let deadline = Instant::now() + Duration::from_secs(20);
    while Instant::now() < deadline {
        // **The file, and then the process.** Alacritty writes the grid and exits, so a `grid.json`
        // that exists is a grid that was written — but a run that read it the instant it appeared
        // could read a partial write. Waiting for the process to be gone is what makes the file
        // whole, and the file existing is what says the branch that writes it ran at all.
        //
        // **`try_wait` and never `kill -0`**, which was written first and hung every run to its
        // twenty-second ceiling: an exited child this process has not reaped is a zombie, its pid is
        // still a pid, and `kill -0` answers *there* for ever. The reap is the observation.
        let exited = alacritty
            .try_wait()
            .map_err(|e| format!("waiting for Alacritty: {e}"))?
            .is_some();
        if grid.exists() && exited {
            return std::fs::read(&grid).map_err(|e| format!("reading {}: {e}", grid.display()));
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Err(format!(
        "{} never appeared, or Alacritty never exited — the grid is written from the branch where \
         the last window closes, so this is a window that did not",
        grid.display()
    ))
}

/// The scene process, which is Alacritty's own child.
///
/// **Refused unless there is exactly one**, for the reason the Ghostty arm insists its set
/// difference is one terminal: a second child is a process this run cannot name, and stopping the
/// wrong one would photograph a screen nothing wrote.
fn child_of(parent: u32) -> Result<u32, String> {
    let out = Command::new("pgrep")
        .args(["-P", &parent.to_string()])
        .output()
        .map_err(|e| format!("pgrep: {e}"))?;
    let pids: Vec<u32> = String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .filter_map(|p| p.parse().ok())
        .collect();
    match pids.as_slice() {
        [one] => Ok(*one),
        other => Err(format!(
            "Alacritty has {} children and the scene is one of them; this run cannot say which",
            other.len()
        )),
    }
}

fn signal(which: &str, pid: u32) -> Result<(), String> {
    let status = Command::new("kill")
        .args([&format!("-{which}"), &pid.to_string()])
        .status()
        .map_err(|e| format!("kill -{which} {pid}: {e}"))?;
    match status.success() {
        true => Ok(()),
        false => Err(format!("kill -{which} {pid} exited {status}")),
    }
}

/// What Alacritty says its version is. Asked of it, never assumed.
fn version() -> Result<String, String> {
    let out = Command::new(ALACRITTY)
        .arg("--version")
        .output()
        .map_err(|e| format!("{ALACRITTY} --version: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    // `alacritty 0.17.0 (…)` on a build from a checkout, `alacritty 0.17.0` from a published crate.
    text.split_whitespace()
        .nth(1)
        .map(str::to_string)
        .ok_or_else(|| format!("{ALACRITTY} --version said {text:?}, which has no version in it"))
}

/// This arm, as the report needs to describe it.
fn arm(version: &str) -> Arm {
    Arm {
        title: "Alacritty",
        version: version.to_string(),
        mechanism: "`alacritty --ref-test`, and the `grid.json` it writes when its last window closes",
        measures: "**what Alacritty stores.** The grid is serialised out of the `Term`, so no \
                   serialiser of the emulator's is in the path — an underline colour is a *field* \
                   of the cell here rather than a sequence that has to survive a re-serialisation, \
                   which is what makes every row of it a statement about storage rather than about \
                   paint",
        answers_in_band: AnswersInBand {
            who: "Alacritty",
            why: "the innermost terminal in the path, and the only one — nothing is multiplexed here",
        },
        not_compared: BY_DESIGN,
        no_style: None,
        notes: vec![
            "**No XTVERSION.** `CSI > 0 q` is answered with nothing at all, so this terminal cannot \
             be recognised by the query that recognises tmux and kitty. DA1 is `CSI ? 6 c` — a \
             VT102, and a sixth distinct sentinel in this directory."
                .into(),
            "**`TERM` is `xterm-256color` here and that is Alacritty's own doing.** `setup_env` \
             picks `alacritty` only where that terminfo entry exists, and this machine has none. So \
             the quirk table's recognition rule for this terminal is `ALACRITTY_WINDOW_ID`, which \
             is the variable Alacritty sets for every window it opens."
                .into(),
            "**Scene 06's part B measures the reply and not the flag, and on this terminal that is \
             the only thing left to measure.** DECRQM here answers `reset` inside an open block and \
             outside one alike, so there is no *when did it stop saying set* to bisect. What the \
             floor reports instead is when the **reply** arrived: a `CSI ? 2026 $ p` written 50 ms \
             into an open block came back at **150, 151 and 171 ms** on three runs of this arm — so \
             the question sat in the parser's buffer for about a tenth of a second and came out \
             when the block force-flushed. `vte`'s shipped `SYNC_UPDATE_TIMEOUT` is 150 ms, which \
             makes this the first row of that table with a **measurement** beside it. **It is \
             a bound and not the paint**, and the arming instant is unobservable from inside for \
             Ghostty's reason: the terminal starts its timer when it parses the `h` and this scene's \
             clock starts when it wrote one."
                .into(),
            "**Three control probes, raw `printf` with no engine in any of them, separate the \
             scene-06 rows from every innocent reading** — WezTerm's three, asked again of a second \
             family. `CSI ? 9999 $ p` answers **`0`**, so this terminal says *not recognised* when \
             it means it and its `2` for 2026 is a real *reset*. `CSI ? 2004 $ p` answers `2`, then \
             **`1`** after an `h`, then `2` after an `l`, so its DECRQM tracks a mode it \
             implements. And it genuinely holds a block, which is the reply above arriving a tenth \
             of a second late. What is different here is that the cause is **readable**: \
             `Term::report_private_mode` answers `NamedPrivateMode::SyncUpdate` with a constant \
             `ModeState::Reset`, and synchronised output is implemented one crate down in `vte`, \
             which the `Term` never sees."
                .into(),
            "**The binary is a source build.** The Homebrew cask was disabled on 2026-09-01 for \
             failing the macOS Gatekeeper check, so this is `cargo install alacritty --locked` — \
             the published crate, built here, rather than the project's signed bundle."
                .into(),
        ],
    }
}
