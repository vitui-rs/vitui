//! The kitty arm: drive the engine inside a real kitty window and read kitty's screen back.
//!
//! **One executable, two halves**, the same shape as the other arms: with no arguments this is the
//! driver, with `--scene 01` it is the scene, and the driver launches the scene by re-running its own
//! [`std::env::current_exe`]. The scene, the readiness handshake and the comparison come from
//! [`common`] and are byte-for-byte the ones the Ghostty and tmux arms run, which is what lets a row
//! say *kitty dropped this bit and Ghostty did not* and mean it.
//!
//! ```sh
//! cd conform && cargo run --example kitty          # writes REPORT-kitty.md
//! ```
//!
//! # It is a better instrument than the Ghostty arm, on three axes
//!
//! Production ticket 04 predicted two of these and underestimated the third.
//!
//! 1. **No TCC grant and no window index.** `kitten @ --to unix:…` is a real remote-control socket,
//!    so there is no macOS automation consent dialog, no clipboard to clobber and no z-order to
//!    address around. The Ghostty arm needs a grant that cannot be obtained non-interactively
//!    anywhere but on the machine it was obtained on.
//! 2. **No config file in the loop.** `--listen-on` and `-o allow_remote_control=yes` on the command
//!    line are the whole of the setup, so the tmux arm's `-f /dev/null` equivalent comes free.
//!    `-o remember_window_size=no` is the other half of that: without it kitty restores whatever
//!    size the user last dragged one of its windows to, and the run would be about their window.
//! 3. **It can be handed a size, and it is the first *emulator* arm that can.** `-o
//!    initial_window_width=80c -o initial_window_height=24c` is a real geometry in cells, and
//!    `kitten @ ls` reports back what it got. Ghostty's `surface configuration` offers a font size
//!    and no rows or columns; tmux can set a pane size but is not an emulator. **The quiescence
//!    handshake stays anyway** — see [`REQUESTED`].
//!
//! # What it cannot ask, declared before the run
//!
//! kitty's serialiser writes a **dotted** underline as `CSI 4 : m`, an empty sub-parameter, and a
//! dashed one identically; the shipped `kitty.fast_data_types.so` contains the strings `4:2;` and
//! `4:3;` and neither `4:4;` nor `4:5;`. ECMA-48 reads an omitted parameter as the default and SGR
//! 4's default is 1, so the capture says *single* for a cell holding *dotted*.
//!
//! That is a limit of the **capture format** and not a misbehaviour of the terminal, and the two must
//! not be reported as one thing: a row compared anyway would say *kitty does not render dotted
//! underlines*, which is false, and would earn a `quirks.rs` entry the table exists to keep out. So
//! [`CANNOT_ASK`] declares the row, with the reason, before a single frame is drawn.
//!
//! **The separation is not an assumption either.** `CSI 4 : 0 m` comes back as nothing at all, so a
//! `4:` in the capture proves the cell holds a *non-zero* decoration and only its number was lost.
//! The instrument can still see that something is underlined; it cannot see which of two things.
//!
//! # What it *can* answer, and two of the answers are `quirks.rs` rows
//!
//! Overline and conceal come back bare, and for those two the dump is not the evidence — the shipped
//! binary is. kitty's `Cursor` repr enumerates every formatting attribute the cursor carries:
//!
//! ```text
//! Cursor(x, y, shape, blink, fg, bg, bold, italic, reverse, strikethrough, dim,
//!        decoration, decoration_fg, text_blink)
//! ```
//!
//! No conceal and no overline, and the attribute constants beside it — `BOLD ITALIC REVERSE MARK
//! STRIKETHROUGH DECORATION BLINK` — say the same thing a second time. **SGR is a mutation of the
//! cursor**, so an attribute the cursor cannot carry is one no cell can hold and no paint can
//! consult. `FINDINGS.md` had left conceal open on the possibility that kitty resolved the
//! foreground to the background at paint time; that needs a flag stored somewhere, and there is
//! none.
//!
//! **Those two rows were `FAILED` for exactly one run**, which is the arm's whole point and also the
//! end of its ability to make it. They earned `quirks.rs` its fifth entry, and the engine now
//! withholds both — so the rows read `by design` and the evidence lives in the committed fixture,
//! captured while the engine still sent them. See [`common::Excluded::ByDesign`], and do not
//! regenerate that fixture to make something pass.

mod common;

use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use common::{
    AnswersInBand, Arm, Excluded, SCENES, clear_handshake, header, publish, save_if_asked, scene,
    scene_argv, section, trailer, wait_for_quiescence,
};
use vitui_conform::Dialect;

/// Where kitty is, absolutely.
///
/// **Not the bare name**, for the reason the Ghostty arm's `tmux_path` gives one level along:
/// Homebrew's `/opt/homebrew/bin` is a symlink farm into a Caskroom, and a run started from
/// somewhere without it on `PATH` should fail naming what is missing rather than fail as a frame
/// that never came. The bundle path is where the cask puts the binary and is what `kitty --version`
/// resolves to anyway.
const KITTY: &str = "/Applications/kitty.app/Contents/MacOS/kitty";

/// The `kitten` beside it, which is the remote-control client.
const KITTEN: &str = "/Applications/kitty.app/Contents/MacOS/kitten";

/// The window size this arm asks for, in cells.
///
/// **The first emulator arm that can ask.** Eleven rows plus room so nothing scrolls, and wide enough
/// that an attribute leaking past its label has somewhere to leak to — the check that makes a reverse
/// block running to the right edge a disagreement rather than a pass.
const REQUESTED: (u16, u16) = (80, 24);

/// The rows this arm does not compare, and why.
///
/// Declared here, above the run, because a limitation discovered after seeing the answer is an
/// excuse rather than a declaration — and hand-written rather than asked of the engine, because an
/// arm that consulted `quirks.rs` for what to expect would be checking the engine against itself.
///
/// **Both kinds are here, and they are the two halves of this arm's story.** One row the instrument
/// cannot see; two rows the engine deliberately no longer sends, because the evidence below earned
/// `quirks.rs` its fifth entry and the entry then took the measurement away.
const NOT_COMPARED: &[(&str, Excluded, &str)] = &[
    (
        "under-dot",
        Excluded::CannotAsk,
        "kitty's serialiser has a string for `4:2` and `4:3` and none for `4:4` or `4:5`, so a \
         dotted underline is written into the capture as `CSI 4 : m` — an empty sub-parameter, \
         which ECMA-48 reads as SGR 4's default of *single*. `CSI 4:0 m` comes back as nothing at \
         all, so the `4:` here does prove the cell holds a non-zero decoration; what is lost is \
         which one. A fact about the **capture format**, not about what kitty renders",
    ),
    (
        "conceal",
        Excluded::ByDesign,
        "`quirks.rs`'s fifth entry, earned by this arm's own first run and by kitty's shipped \
         binary: its `Cursor` carries no conceal attribute, so SGR 8 has nothing to set. The engine \
         now withholds it, and `fixtures/kitty-0.48.2-scene01-attrs.vt` is the capture taken while \
         it still sent it",
    ),
    (
        "overline",
        Excluded::ByDesign,
        "the same entry's second bit. kitty's `Cursor` carries no overline attribute either, and a \
         raw-`printf` control probe with no engine in it says the same thing independently",
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

/// A kitty started by this run, killed when it goes out of scope.
///
/// Not tidiness: the guards below return early, and a kitty left running holds a window on the
/// user's screen and a socket the *next* run's collision refusal would fire on. The Ghostty arm
/// closes its window before unwrapping the result for the same reason; here the child is ours, so
/// `Drop` can say it once.
struct Instance {
    child: Child,
    socket: PathBuf,
}

impl Drop for Instance {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_file(&self.socket);
    }
}

/// Every scene, each in its own kitty window, into one report.
///
/// **A window per scene rather than a scene switch inside one**, for the reason the whole directory
/// is arranged around: a window that has been written to once is a window whose state is now part of
/// the measurement. A fresh one costs a second and owes nothing to the run before it.
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
    // **An argv and not a command line.** kitty execs what it is given; handed one string it looks
    // for a file whose name ends in `--scene 01`, finds none, and says nothing — which arrives here
    // as the readiness timeout twenty seconds later, blaming the scene for the launcher.
    let command = scene_argv(which)?;
    let ready = std::env::temp_dir().join(format!("conform-ready-{}-{which}", std::process::id()));
    clear_handshake(&ready);

    // Named for this process, so two runs cannot meet — and **refused if it already exists**, which
    // is this arm's equivalent of the Ghostty arm insisting the set difference is exactly one
    // terminal. A socket we did not create is somebody else's kitty, and `kitten @ --to` would
    // photograph their screen and report it as ours.
    let socket =
        std::env::temp_dir().join(format!("conform-kitty-{}-{which}.sock", std::process::id()));
    if socket.exists() {
        return Err(format!(
            "{} already exists; this run cannot say whose kitty it would be photographing",
            socket.display()
        ));
    }
    // **A unix socket path has a length limit and kitty does not fail on it**, which cost a probe to
    // find. `sockaddr_un::sun_path` is 104 bytes on macOS; given a longer one kitty logs `Invalid
    // listen_on=…, ignoring` **and starts anyway**, so the window opens, no socket appears, and
    // every `kitten @` fails with a connection error that reads like a permissions problem.
    //
    // So the refusal is here, where it can name the cause, rather than twenty seconds later where it
    // could only say a socket never appeared. The path above is under `$TMPDIR`, which is short on
    // this machine — but `$TMPDIR` is somebody's environment variable and this is one line.
    const SUN_PATH: usize = 104;
    if socket.as_os_str().len() >= SUN_PATH {
        return Err(format!(
            "the control socket path is {} bytes and a unix socket path may be {SUN_PATH}: {}. \
             kitty would ignore `--listen-on` and start anyway, so this would arrive as a window \
             that never answers",
            socket.as_os_str().len(),
            socket.display()
        ));
    }

    let version = version()?;
    let launched = Instant::now();
    let child = Command::new(KITTY)
        .args(["--listen-on", &format!("unix:{}", socket.display())])
        .args(["-o", "allow_remote_control=yes"])
        // Geometry in cells, which is the thing the Ghostty arm cannot ask for.
        .args(["-o", &format!("initial_window_width={}c", REQUESTED.0)])
        .args(["-o", &format!("initial_window_height={}c", REQUESTED.1)])
        // Otherwise kitty restores the size of whatever window the user last resized, and the run is
        // about their window rather than about the size above.
        .args(["-o", "remember_window_size=no"])
        .args(["--title", "vitui-conform"])
        .env("CONFORM_READY", &ready)
        // The scene's own output goes to the pty kitty gives it, never to ours. What kitty itself
        // says goes nowhere: a warning on our stderr would be mistaken for the driver's.
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .args(&command)
        .spawn()
        .map_err(|e| format!("{KITTY}: {e} — this arm needs kitty installed at that path"))?;
    let instance = Instance {
        child,
        socket: socket.clone(),
    };

    let outcome = capture_and_compare(which, &socket, &version, &ready, launched);
    // Killed before the result is unwrapped, like the Ghostty arm closes its window: a failed run
    // that leaves a window open — and a socket behind it — is a run that needs a human before the
    // next one can start.
    drop(instance);
    clear_handshake(&ready);
    outcome
}

/// Wait for the frame, read the screen back, parse it, compare it, and render the report.
fn capture_and_compare(
    which: &str,
    socket: &Path,
    version: &str,
    ready: &Path,
    launched: Instant,
) -> Result<(Arm, common::Capture, Vec<u8>, String), String> {
    wait_for_socket(socket)?;

    // Insisted on before the wait for the same reason the tmux arm counts panes first: a scene that
    // never started leaves the readiness timeout to explain it, and "the scene never reported a
    // presented frame" is twenty seconds spent saying something this line says at once.
    let windows = window_geometry(socket)?;
    if windows.len() != 1 {
        return Err(format!(
            "{} windows in this kitty, not one — this run cannot say which one it captured",
            windows.len()
        ));
    }

    let size = wait_for_quiescence(ready, which)?;

    // **`Ecma48`, and that was probed rather than assumed.** kitty re-serialises its own grid, which
    // is what tmux does too and tmux needed a dialect of its own — so this was checked with a raw
    // `printf` control probe before the arm was written. Every construct kitty emits means what
    // ECMA-48 says: `CSI m` per row, `22;1` for bold, `4:2`/`4:3`, and the colon colour forms. See
    // the parser's module docs.
    //
    // **`get-text` is not called for scene 05**, and that is `common::capture`'s decision rather
    // than this arm's: the terminal answers that scene in band on the scene's own tty, so the
    // remote-control socket is not in its path.
    let mut capture_elapsed = Duration::ZERO;
    let (captured, bytes) = common::capture(which, ready, Dialect::Ecma48, || {
        let started = Instant::now();
        let bytes = capture(socket)?;
        capture_elapsed = started.elapsed();
        Ok(bytes)
    })?;
    save_if_asked(which, &bytes)?;

    let arm = Arm {
        title: "kitty",
        version: version.to_string(),
        mechanism: "`kitten @ get-text --extent screen --ansi`, over a unix socket",
        measures: "kitty. Its own cell state, re-serialised by the emulator that holds it — the \
                   same kind of evidence as the Ghostty arm's and gathered over a remote-control \
                   socket rather than an AppleScript surface, so no automation grant and no window \
                   z-order is in the loop",
        answers_in_band: AnswersInBand {
            who: "kitty",
            why: "kitty is an endpoint, so nothing sits between the scene's tty and it. It is also \
                  the arm where the two channels are most obviously different instruments: the \
                  photograph goes through a serialiser with a string for `4:2` and none for `4:4`, \
                  and an in-band reply goes through none",
        },
        not_compared: NOT_COMPARED,
        // Every capture from this arm is a re-serialisation of the emulator's own cell state, so
        // the style is in the capture and scene 01 is answerable. See [`Arm::no_style`].
        no_style: None,
        notes: vec![
            format!(
                "**Geometry: asked for and got.** `-o initial_window_width={}c -o \
                 initial_window_height={}c`, and `kitten @ ls` reported {}. **The first *emulator* \
                 arm that can be handed a size** — Ghostty's `surface configuration` offers a font \
                 size and nothing else. The quiescence handshake is kept anyway: it is what proves \
                 the window stopped moving, and one command-line option is not a run",
                REQUESTED.0, REQUESTED.1, windows[0]
            ),
            "**Config:** none. `--listen-on` and `-o allow_remote_control=yes` are given on the \
             command line, so there is no `kitty.conf` in the result — the tmux arm's `-f \
             /dev/null` equivalent, for free. `-o remember_window_size=no` is the other half: \
             without it kitty restores the size of the last window the user dragged"
                .to_string(),
            match which {
                "05" | "06" => format!(
                    "**Launch to answer:** {} ms — reported, never gated. **This scene is not \
                     photographed:** the terminal answers in band on the scene's own tty, so \
                     `get-text` and the socket under it are out of the path and this arm's whole \
                     job was to have launched the scene",
                    launched.elapsed().as_millis()
                ),
                _ => format!(
                    "**Launch to capture:** {} ms, of which `get-text` itself was {} ms — reported, \
                     never gated. No automation consent dialog, no clipboard, no z-order",
                    launched.elapsed().as_millis(),
                    capture_elapsed.as_millis()
                ),
            },
            "**Trailing blanks: kept, and that is the opposite of the tmux arm.** Every row the \
             engine painted comes back at its full width, so the check that an attribute stopped \
             where its label did has real padding to look at here. kitty closes each label with an \
             explicit off-code — `22`, `23`, `24`, `27`, `29` — rather than a reset, which is what \
             makes that check answerable at all. The one exception is the final row, which arrives \
             as a bare `CSI m` with no cells where Ghostty's dump gives it painted; the parser \
             drops a trailing blank row after counting the rows, so nothing turns on it"
                .to_string(),
            "**A never-written cell is not a painted blank**, and only the first is trimmed. A \
             probe screen kitty had never had written to came back with its unpainted rows absent \
             entirely — which is what an empty capture looks like from this arm, and why the \
             refusal is the first thing it does rather than the last"
                .to_string(),
            "**Rows are LF-separated**, where Ghostty's dump is CRLF. So `core.autocrlf` has \
             nothing to rewrite in this arm's fixture — which is luck rather than safety, and \
             `.gitattributes` still marks `*.vt` as `-text`"
                .to_string(),
        ],
    };
    Ok((arm, captured, bytes, size))
}

/// Block until kitty has created its control socket, or say that it never did.
///
/// A `kitten @` against a socket that does not exist yet fails with a connection error, and retrying
/// that in the caller would blur *kitty has not started* into *kitty refused the command*. They have
/// different causes and this one has a name.
fn wait_for_socket(socket: &Path) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if socket.exists() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Err(format!(
        "kitty never created {} — it may have refused `allow_remote_control`, or failed to start \
         at all",
        socket.display()
    ))
}

/// Every window this kitty has, as `<columns>x<lines>`.
///
/// Read out of `kitten @ ls`'s JSON by hand rather than with a parser crate, because this directory
/// depends on nothing and the two numbers are one `"columns": 80` each. A malformed reply produces
/// an empty list, which the caller refuses as *not one window* rather than proceeding with a guess.
fn window_geometry(socket: &Path) -> Result<Vec<String>, String> {
    let json = String::from_utf8_lossy(&kitten_raw(socket, &["ls"])?).into_owned();
    let numbers = |key: &str| -> Vec<String> {
        json.split(&format!("\"{key}\":"))
            .skip(1)
            .filter_map(|rest| {
                let value: String = rest
                    .trim_start()
                    .chars()
                    .take_while(char::is_ascii_digit)
                    .collect();
                (!value.is_empty()).then_some(value)
            })
            .collect()
    };
    let (columns, lines) = (numbers("columns"), numbers("lines"));
    Ok(columns
        .iter()
        .zip(&lines)
        .map(|(c, l)| format!("{c}x{l}"))
        .collect())
}

/// What kitty says its version is. Asked of the binary that is about to run, never assumed.
fn version() -> Result<String, String> {
    let out = Command::new(KITTY)
        .arg("--version")
        .output()
        .map_err(|e| format!("{KITTY} --version: {e}"))?;
    if !out.status.success() {
        return Err("kitty --version failed".into());
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .trim()
        .trim_start_matches("kitty ")
        .split_whitespace()
        .next()
        .unwrap_or("unknown")
        .to_string())
}

/// Read the screen back, refusing a reply of no bytes at all.
///
/// **The refusal that comes first.** `screen -X hardcopy` exits 0 and writes a zero-byte file, and an
/// empty capture compares equal against a blank region — the row goes green and the missing result
/// hides inside a passing one. [`vitui_conform::parse`] refuses an empty dump too; this refuses it
/// one step earlier, where the message can still name the command that produced nothing.
fn capture(socket: &Path) -> Result<Vec<u8>, String> {
    let bytes = kitten_raw(socket, &["get-text", "--extent", "screen", "--ansi"])?;
    if bytes.is_empty() {
        return Err(format!(
            "`kitten @ get-text` returned no bytes at all against {} — an empty capture compares \
             equal against a blank region and must never reach the comparison",
            socket.display()
        ));
    }
    Ok(bytes)
}

/// One remote-control command against our own socket, as bytes.
///
/// **Bytes and not a `String`**, for the reason the tmux arm gives: the capture is evidence, and
/// `from_utf8_lossy` on the way through would replace a malformed sequence here, where the parser is
/// the thing that is supposed to notice it.
fn kitten_raw(socket: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new(KITTEN)
        .arg("@")
        .args(["--to", &format!("unix:{}", socket.display())])
        .args(args)
        .output()
        .map_err(|e| format!("kitten @ {}: {e}", args.join(" ")))?;
    if !out.status.success() {
        return Err(format!(
            "kitten @ {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(out.stdout)
}
