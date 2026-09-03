//! The WezTerm arm: drive the engine inside a real WezTerm window and read WezTerm's screen back.
//!
//! **One executable, two halves**, the same shape as the other four: with no arguments this is the
//! driver, with `--scene 01` it is the scene, and the driver launches the scene by re-running its
//! own [`std::env::current_exe`]. The scene, the readiness handshake and the comparison come from
//! [`common`] and are byte-for-byte the ones the other arms run, which is what lets a row say
//! *WezTerm dropped this bit and kitty did not* and mean it.
//!
//! ```sh
//! cd conform && cargo run --example wezterm      # writes REPORT-wezterm.md
//! ```
//!
//! # It is the fifth family, and the two claims it was aimed at
//!
//! Production ticket 11 opened with spec §10's inference: WezTerm ships kitty's keyboard *encoding*
//! while implementing none of the flag stack, and has the protocol off by default. **Neither is a
//! question this instrument asks.** All four scenes are about cell state, bisected pairs, in-band
//! width verdicts and mode 2026; none of them sends a keystroke, and the keyboard protocol has no
//! capture surface here at all. That is recorded rather than worked around — see the report's
//! trailer — and it is a limit of the four scenes and not of the terminal.
//!
//! # It is addressed by socket path, and getting that wrong photographs a different terminal
//!
//! **This is the sharpest instance of *a missing row reads as a win* this directory has met**, and
//! it is worth the paragraph because the wrong version *succeeds*.
//!
//! `wezterm cli` prefers a **background mux server** at `~/.local/share/wezterm/sock`, not the GUI
//! this run started. Handed no socket it connects there, and if nothing is listening it runs
//! `wezterm-mux-server --daemonize` and connects to *that* — which spawns a default shell in a
//! default-sized pane. `wezterm cli list` then exits 0 with well-formed JSON, `get-text` returns a
//! screenful of somebody's shell prompt, and the comparison reports eleven disagreements about a
//! terminal the scene was never drawn into. It was observed doing exactly that during the probe
//! that preceded this file.
//!
//! Two things stop it, and both are load-bearing:
//!
//! 1. **`WEZTERM_UNIX_SOCKET` is set to this run's own GUI socket**, `gui-sock-<pid>` under
//!    WezTerm's runtime directory, and the pid is the one [`std::process::Child`] handed back —
//!    `wezterm start` **execs** `wezterm-gui` in place, so the pid is preserved. Named for our own
//!    child rather than found by scanning, so a WezTerm the user starts while this runs cannot be
//!    the one photographed, and **refused if the path already exists** for the reason the kitty arm
//!    refuses its socket.
//! 2. **`--no-auto-start` on every command.** Without it a wrong or missing socket is *repaired* by
//!    starting a daemon, which is the failure above wearing a success. With it, a socket that is not
//!    there is an error naming the path.
//!
//! **`--class` is not the answer, and it was probed.** `wezterm cli --class` is documented for
//! finding a GUI instance started with `--class`, and on macOS it routes nowhere: the class is a
//! windowing-system class, X11 and Windows and Wayland, and the run that passed it landed on the
//! auto-started mux server above.
//!
//! **A killed WezTerm leaves its socket file behind**, observed. `Drop` removes it, or the next
//! run's collision refusal fires on a terminal that no longer exists.
//!
//! # What it cannot ask, declared before the run
//!
//! `get-text --escapes` re-serialises WezTerm's own grid into a **classic** SGR repertoire, and
//! three things fall outside it. All three were established with a raw `printf` control probe with
//! no engine anywhere in it, before this file existed:
//!
//! | sent | came back |
//! |---|---|
//! | `CSI 4 m` | `4` |
//! | `CSI 4:1 m` | `4` — **the sub-parameter is normalised away** |
//! | `CSI 4:2 m` | `21`, ECMA-48's *doubly underlined* |
//! | `CSI 4:3 m`, `4:4`, `4:5` | nothing at all |
//! | `CSI 53 m` | nothing at all |
//! | `CSI 58;5;9 m`, `CSI 58:2::255:0:0 m` | nothing at all |
//! | `CSI 38;5;9 m` | `91`, the aixterm bright form |
//! | `CSI 48:5:21 m` | `48;5;21` — colon in, semicolon out |
//! | `CSI 38:2::255:0:0 m` | `38:2::255:0:0`, verbatim |
//!
//! So the capture has **no spelling** for `4:3`, `4:4`, `4:5`, SGR 53 or SGR 58, and it is a
//! re-speller rather than an echo — it rewrites the forms it does have. Two of scene 01's eleven
//! rows land in that gap, `overline` and `under-dot`, and [`NOT_COMPARED`] declares both before a
//! frame is drawn.
//!
//! **The separation is not an assumption.** `CSI 4:3 ; 1 m` comes back as `CSI 0 ; 1 m` and
//! `CSI 53 ; 1 m` comes back as `CSI 0 ; 1 m`: the bold survives, so the serialiser reached a cell
//! it agrees is styled and dropped one attribute of it. What it cannot say is whether the cell was
//! holding the attribute — and that is the whole of why these are [`Excluded::CannotAsk`] and not a
//! `quirks.rs` entry.
//!
//! # This arm cannot promote either row to a quirk, and that is the finding rather than a gap
//!
//! kitty's two `by design` rows are `quirks.rs`'s fifth entry, and they earned it **from a second
//! source**: the shipped `kitty.fast_data_types.so` prints a `Cursor` repr enumerating every
//! attribute the cursor carries, and neither conceal nor overline is in it. *Not stored* and *not
//! serialised* look identical in a capture, and only a second source separates them.
//!
//! There is no second source here. WezTerm is an endpoint, so there is no far side to read from —
//! the tmux arm's three-path attribution is unavailable to every emulator arm. And the shipped
//! binary is one blob whose string table holds every Unicode character name, so the technique that
//! settled kitty returns `overline` from `OVERLINE` inside a run of eight-letter glyph names. A
//! `grep -c` there is noise, not evidence, and reporting it as evidence would earn `quirks.rs` an
//! entry describing a misbehaviour that may not be happening — which is the one thing that table is
//! most careful to keep out.
//!
//! So this arm's answer is `cannot ask` twice, and **the difference from kitty is entirely in what
//! evidence was available outside the capture**. Ticket 11 asked for a quirk entry if a real
//! misbehaviour was observed; none was, because on these two rows this instrument cannot see far
//! enough to observe one.
//!
//! # What it *can* answer, and conceal is the interesting one
//!
//! Nine of eleven, and the ninth is `conceal` — which kitty has nowhere to put. WezTerm accepts
//! SGR 8, stores it, and hands it back: `CSI 8 ; 31 m` comes back as `CSI 0 ; 8 m` then `CSI 31 m`,
//! so the flag and the foreground both survive and neither was resolved on the way in. It is the
//! second family to hold all of blink, conceal and strikethrough, and the first that is not Ghostty.
//!
//! # Two parser gaps it found before it drew a frame
//!
//! Both in `src/lib.rs`, both ECMA-48-correct, and **neither grew the [`Dialect`] enum** — a
//! dialect is a disagreement about what a colon means, and there is none here.
//!
//! - **`ESC ( B` leads every row.** WezTerm designates US-ASCII as G0 before each row's SGR. The
//!   escape handler's two-byte fallback consumed `ESC (` and left the `B` as content, so every row
//!   of this scene arrived as `Bbold` — eleven rows red, blaming the terminal for the instrument's
//!   arithmetic.
//! - **SGR 21 is *doubly underlined*.** ECMA-48 spends 21 on that and 22 on normal intensity;
//!   *bold off* is xterm's reading and a deviation. Nothing needed the arm while every capture here
//!   spelled the style `4:2`.

mod common;

use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use common::{
    AnswersInBand, Arm, Excluded, SCENES, clear_handshake, header, publish, save_if_asked, scene,
    scene_argv, section, trailer, wait_for_quiescence,
};
use vitui_conform::Dialect;

/// Where WezTerm is, absolutely.
///
/// **Not the bare name**, for the kitty arm's reason: `/opt/homebrew/bin/wezterm` is a symlink into
/// a Caskroom, and a run started from somewhere without it on `PATH` should fail naming what is
/// missing rather than fail as a frame that never came.
const WEZTERM: &str = "/Applications/WezTerm.app/Contents/MacOS/wezterm";

/// The window size this arm asks for, in cells.
///
/// **The second *emulator* arm that can ask**, and it asks the way kitty's does — a real geometry in
/// cells, reported back by `wezterm cli list`. Eleven rows plus room so nothing scrolls, and wide
/// enough that an attribute leaking past its label has somewhere to leak to.
const REQUESTED: (u16, u16) = (80, 24);

/// The rows this arm does not compare, and why.
///
/// Declared here, above the run, because a limitation discovered after seeing the answer is an
/// excuse rather than a declaration — and hand-written rather than asked of the engine, because an
/// arm that consulted `quirks.rs` for what to expect would be checking the engine against itself.
///
/// **Both are [`Excluded::CannotAsk`] and neither is a quirk**, which is this arm's headline. See
/// the module docs: the capture has no spelling for either attribute, a cell carrying it alongside
/// bold comes back carrying the bold, and there is no second source that could separate *not
/// stored* from *not serialised* the way kitty's shipped binary did.
const NOT_COMPARED: &[(&str, Excluded, &str)] = &[
    (
        "overline",
        Excluded::CannotAsk,
        "`get-text --escapes` has no SGR 53 in its repertoire: `CSI 53 m` comes back as nothing, \
         and `CSI 53 ; 1 m` comes back as `CSI 0 ; 1 m` — the bold survives, so the serialiser \
         reached a cell it agrees is styled and wrote none of the overline. A fact about the \
         **capture format**, and one this arm cannot promote to a `quirks.rs` row: WezTerm is an \
         endpoint, so there is no far side to attribute from, and its shipped binary's string \
         table holds `OVERLINE` as a Unicode character name, so the technique that settled kitty's \
         two rows returns noise here",
    ),
    (
        "under-dot",
        Excluded::CannotAsk,
        "the same repertoire, one axis along. `CSI 4:1 m` comes back as bare `4` and `CSI 4:2 m` \
         as `21`, so sub-parameters are normalised away entirely — and `4:3`, `4:4` and `4:5` come \
         back as nothing at all, where kitty at least emits `CSI 4 : m` and proves a non-zero \
         decoration is there. This capture cannot distinguish a dotted underline from no underline, \
         so a row compared anyway would report *WezTerm does not render dotted underlines*, which \
         is what the `quirks.rs` table exists to keep out",
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

/// A WezTerm started by this run, killed when it goes out of scope.
///
/// Not tidiness, and not optional: **`std::process::Child`'s own `Drop` does not kill**, so every
/// `?` between the spawn and this struct leaks a WezTerm window onto the user's screen. That is why
/// the two things that can fail — the runtime directory and the socket snapshot — are read *before*
/// the spawn rather than between it and here.
///
/// **The socket is removed by hand** because WezTerm does not remove it — a killed GUI was observed
/// leaving `gui-sock-<pid>` behind, and the next run's collision refusal would then fire on a
/// terminal that no longer exists.
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

/// Every scene, each in its own WezTerm window, into one report.
///
/// **A window per scene rather than a scene switch inside one**, for the reason the whole directory
/// is arranged around: a window that has been written to once is a window whose state is now part
/// of the measurement.
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
    // An argv and not a command line: WezTerm execs what it is given after `--`.
    let command = scene_argv(which)?;
    let ready = std::env::temp_dir().join(format!("conform-ready-{}-{which}", std::process::id()));
    clear_handshake(&ready);

    let version = version()?;
    // **Both of these are read before the spawn, and both for the same reason.** A `Child` that
    // goes out of scope is *not* killed, so a `?` between the spawn and the `Instance` below leaks
    // a WezTerm window — and the socket names cannot be compared after the spawn, because by then
    // our own child has created one. See [`Instance`] and the refusal below.
    let dir = runtime_dir()?;
    let before = sockets_in(&dir);
    let launched = Instant::now();
    // **`--config` before `start`, and that is not a style choice.** They are top-level options;
    // after the subcommand WezTerm exits at once with `unexpected argument '--config' found`, which
    // arrives here as a socket that never appears rather than as anything about the geometry.
    //
    // `-n` is `--skip-config`: no `~/.wezterm.lua` in the result, which is the tmux arm's
    // `-f /dev/null` and the kitty arm's bare command line, for free.
    //
    // `--always-new-process` is what makes the pid below ours. Without it `start` asks an existing
    // GUI instance to open the window and returns, so there is no child to name a socket after and
    // no child to kill.
    let child = Command::new(WEZTERM)
        .arg("-n")
        .args(["--config", &format!("initial_cols={}", REQUESTED.0)])
        .args(["--config", &format!("initial_rows={}", REQUESTED.1)])
        .arg("start")
        .arg("--always-new-process")
        .env("CONFORM_READY", &ready)
        // The scene's own output goes to the pty WezTerm gives it, never to ours. What WezTerm
        // itself says goes nowhere: a warning on our stderr would be mistaken for the driver's.
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .arg("--")
        .args(&command)
        .spawn()
        .map_err(|e| format!("{WEZTERM}: {e} — this arm needs WezTerm installed at that path"))?;

    // **Named for our own child, never found by scanning.** `wezterm start` execs `wezterm-gui` in
    // place, so this pid is the GUI's, and a WezTerm the user starts while this runs cannot become
    // the one photographed.
    let name = format!("gui-sock-{}", child.id());
    let socket = dir.join(&name);
    let instance = Instance {
        child,
        socket: socket.clone(),
    };
    // **Compared against the snapshot taken before the spawn, not against the filesystem now.**
    // This is the kitty arm's *whose socket would I be photographing* refusal, and it cannot be
    // written the same way: that arm names its socket after its own process id and can therefore
    // ask before it starts anything, where this one has to spawn a child to learn the name. Asking
    // afterwards would race our own WezTerm, which creates the file within half a second — so a
    // literal `socket.exists()` here refuses a correct run about as often as a broken one.
    //
    // A socket at our child's pid that existed **before our child did** is a stale file from a
    // crashed run or somebody else's terminal, and this run cannot tell which.
    if before.contains(&name) {
        return Err(format!(
            "{} already existed before this run's WezTerm was spawned; it is a stale socket or \
             another terminal's, and this run cannot say which one it would be photographing",
            socket.display()
        ));
    }

    let outcome = capture_and_compare(which, &socket, &version, &ready, launched);
    // Killed before the result is unwrapped, like the kitty arm kills its child: a failed run that
    // leaves a window open is a run that needs a human before the next one can start.
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

    // Insisted on before the wait, for the reason the kitty arm insists on one window and the tmux
    // arm counts panes: "the scene never reported a presented frame" is twenty seconds spent saying
    // something this line says at once. It is also where the pane id comes from — `get-text` with
    // no `--pane-id` and no `$WEZTERM_PANE` refuses outright, which is the good failure, but a
    // guessed `--pane-id 0` in a WezTerm with two panes would be the bad one.
    let panes = panes(socket)?;
    let [pane] = panes.as_slice() else {
        return Err(format!(
            "{} panes in this WezTerm, not one — this run cannot say which one it captured",
            panes.len()
        ));
    };

    let size = wait_for_quiescence(ready, which)?;

    // **`Ecma48`, and that was probed rather than assumed.** WezTerm re-serialises its own grid,
    // which is what tmux does too and tmux needed a dialect of its own — so this was checked with a
    // raw `printf` control probe before the arm was written. Every construct WezTerm emits means
    // what ECMA-48 says: `CSI 0 ; n m` per row, bare `4` for a single underline, **`21` for a
    // double** one, `ESC ( B` designating US-ASCII, and both colour spellings. The last two cost
    // the parser an arm each and neither is a disagreement about meaning — see the module docs.
    //
    // **`get-text` is not called for scenes 05 and 06**, and that is `common::capture`'s decision
    // rather than this arm's: the terminal answers those in band on the scene's own tty, so the
    // control socket is not in their path.
    let mut capture_elapsed = Duration::ZERO;
    let (captured, bytes) = common::capture(which, ready, Dialect::Ecma48, || {
        let started = Instant::now();
        let bytes = capture(socket, &pane.id)?;
        capture_elapsed = started.elapsed();
        Ok(bytes)
    })?;
    save_if_asked(which, Dialect::Ecma48, &bytes)?;

    let arm = Arm {
        title: "wezterm",
        version: version.to_string(),
        mechanism: "`wezterm cli get-text --escapes`, over this run's own GUI socket",
        measures: "WezTerm. Its own cell state, re-serialised by the emulator that holds it — the \
                   same kind of evidence as the Ghostty and kitty arms', gathered over a control \
                   socket rather than an AppleScript surface, so no automation grant and no window \
                   z-order is in the loop",
        answers_in_band: AnswersInBand {
            who: "WezTerm",
            why: "WezTerm is an endpoint, so nothing sits between the scene's tty and it. It is \
                  also the arm where the two channels are furthest apart: the photograph goes \
                  through a serialiser with no spelling for SGR 53 or SGR 58 and none for a \
                  sub-parameter, and an in-band reply goes through none",
        },
        not_compared: NOT_COMPARED,
        // Every capture from this arm is a re-serialisation of the emulator's own cell state, so
        // the style is in the capture and scene 01 is answerable. See [`Arm::no_style`].
        no_style: None,
        notes: vec![
            format!(
                "**Geometry: asked for and got.** `--config initial_cols={}` and `--config \
                 initial_rows={}`, and `wezterm cli list` reported {}x{}. **The options come \
                 before `start`** — they are top-level, and after the subcommand WezTerm exits \
                 with `unexpected argument '--config' found` before opening a window at all. The \
                 quiescence handshake is kept anyway: it is what proves the window stopped moving, \
                 and one command-line option is not a run",
                REQUESTED.0, REQUESTED.1, pane.cols, pane.rows
            ),
            "**Config:** none. `-n` is `--skip-config`, so no `~/.wezterm.lua` is in the result — \
             the tmux arm's `-f /dev/null` equivalent. There is no `remember_window_size` to \
             disable: the two options above are a geometry rather than a request against a \
             remembered one"
                .to_string(),
            format!(
                "**Addressed by socket path, and this is the row worth reading twice.** \
                 `WEZTERM_UNIX_SOCKET={}` and `--no-auto-start` on every command. Without them \
                 `wezterm cli` prefers the background mux server at \
                 `~/.local/share/wezterm/sock`, **starts one with `wezterm-mux-server \
                 --daemonize` if none is listening**, and photographs that daemon's default shell \
                 pane — exiting 0, with well-formed JSON and a screenful of somebody's prompt. \
                 Observed. `--class` does not fix it on macOS: that is a windowing-system class \
                 and routes nowhere here",
                socket.display()
            ),
            match which {
                "05" | "06" => format!(
                    "**Launch to answer:** {} ms — reported, never gated. **This scene is not \
                     photographed:** the terminal answers in band on the scene's own tty, so \
                     `get-text` and the socket under it are out of the path and this arm's whole \
                     job was to have launched the scene",
                    launched.elapsed().as_millis()
                ),
                _ => format!(
                    "**Launch to capture:** {} ms, of which `get-text` itself was {} ms — \
                     reported, never gated. No automation consent dialog, no clipboard, no z-order",
                    launched.elapsed().as_millis(),
                    capture_elapsed.as_millis()
                ),
            },
            "**Trailing blanks: trimmed, like tmux's and unlike kitty's.** Each row ends where its \
             content does, so the check that an attribute stopped where its label did has no \
             padding to look at on this arm — what carries it instead is that WezTerm opens every \
             row with `CSI 0 ; n m`, an explicit reset before the attribute, so a style leaking \
             across a row boundary would be visible as the reset being absent"
                .to_string(),
            "**Both SGR spellings were sent, and 38 and 58 answer differently.** A raw `printf` \
             control probe, no engine in it. **38:** `CSI 38;2;255;0;0 m` and `CSI 38:2::255:0:0 m` \
             resolve to the same channel — WezTerm's serialiser emits nothing between two \
             consecutive rows painted with the two spellings, which is a stateful serialiser saying \
             *no change* — and the capture writes truecolour back as `38:2::255:0:0` and an indexed \
             colour as `48;5;21` or the aixterm `91`, so it re-spells rather than echoing. **58:** \
             neither `CSI 58;5;9 m` nor `CSI 58:2::255:0:0 m` comes back at all, in either \
             spelling, so this capture cannot report an underline colour and nothing here says \
             whether WezTerm parsed one. Asked separately because a terminal can want the semicolon \
             form for 58 and never be asked about 38"
                .to_string(),
            "**Rows are CRLF-separated**, like Ghostty's dump and unlike kitty's, and the capture \
             ends with a bare `ESC ( B CSI 0 m` after the blank rows. So `core.autocrlf` has \
             something to rewrite in this arm's fixture and `.gitattributes`' `*.vt -text` is what \
             stops it — the same line that lost twenty-three carriage returns out of the Ghostty \
             capture before it existed"
                .to_string(),
            "**The two spec §10 claims this terminal carries are not askable here, and that is \
             recorded rather than worked around.** *It ships kitty's keyboard encoding while \
             implementing none of the flag stack*, and *the protocol is off by default*, are both \
             about the keyboard. No scene in `SCENES.md` sends a keystroke — scene 01 is cell \
             state, 04 a bisected pair, 05 an in-band width verdict, 06 mode 2026 — so this run \
             neither confirms nor corrects either. A sixth scene could ask the second one with \
             `CSI ? u`; the first needs a key pressed, and nothing here presses one"
                .to_string(),
        ],
    };
    Ok((arm, captured, bytes, size))
}

/// Block until WezTerm has created its GUI socket, or say that it never did.
///
/// A `wezterm cli` against a socket that does not exist yet fails with a connection error, and
/// retrying that in the caller would blur *WezTerm has not started* into *WezTerm refused the
/// command*. They have different causes and this one has a name.
fn wait_for_socket(socket: &Path) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if socket.exists() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Err(format!(
        "WezTerm never created {} — it may have failed to start at all, or rejected an option \
         before opening a window",
        socket.display()
    ))
}

/// WezTerm's runtime directory, where the GUI socket lives.
///
/// **`$HOME` and not `$XDG_RUNTIME_DIR`**, which was probed: WezTerm 20240203 ignores the latter on
/// macOS and puts the socket under `~/.local/share/wezterm` regardless, so a run that trusted the
/// variable would wait ten seconds for a socket in a directory nothing writes to.
fn runtime_dir() -> Result<PathBuf, String> {
    let home = std::env::var_os("HOME")
        .ok_or("$HOME is not set, so WezTerm's runtime directory cannot be named")?;
    Ok(PathBuf::from(home).join(".local/share/wezterm"))
}

/// Every `gui-sock-*` in WezTerm's runtime directory, by file name.
///
/// **Not a `Result`.** A directory that is not there yet is a machine with no WezTerm sockets in it,
/// which is the empty set and not a failure — WezTerm creates the directory on its first run, and a
/// refusal here would make a clean machine the one case this arm cannot start on.
fn sockets_in(dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with("gui-sock-"))
        .collect()
}

/// One pane of this WezTerm, as the id to address and the size it reported.
struct Pane {
    id: String,
    cols: String,
    rows: String,
}

/// Every pane this WezTerm has.
///
/// Read out of `wezterm cli list --format json`'s output by hand rather than with a parser crate,
/// because this directory depends on nothing and the three fields are one `"pane_id": 0` each. A
/// malformed reply produces an empty list, which the caller refuses as *not one pane* rather than
/// proceeding with a guess.
fn panes(socket: &Path) -> Result<Vec<Pane>, String> {
    let json =
        String::from_utf8_lossy(&cli_raw(socket, &["list", "--format", "json"])?).into_owned();
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
    let (ids, cols, rows) = (numbers("pane_id"), numbers("cols"), numbers("rows"));
    Ok(ids
        .into_iter()
        .zip(cols)
        .zip(rows)
        .map(|((id, cols), rows)| Pane { id, cols, rows })
        .collect())
}

/// What WezTerm says its version is. Asked of the binary that is about to run, never assumed.
fn version() -> Result<String, String> {
    let out = Command::new(WEZTERM)
        .arg("--version")
        .output()
        .map_err(|e| format!("{WEZTERM} --version: {e}"))?;
    if !out.status.success() {
        return Err("wezterm --version failed".into());
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .trim()
        .trim_start_matches("wezterm ")
        .split_whitespace()
        .next()
        .unwrap_or("unknown")
        .to_string())
}

/// Read the screen back, refusing a reply of no bytes at all.
///
/// **The refusal that comes first.** `screen -X hardcopy` exits 0 and writes a zero-byte file, and
/// an empty capture compares equal against a blank region — the row goes green and the missing
/// result hides inside a passing one. [`vitui_conform::parse`] refuses an empty dump too; this
/// refuses it one step earlier, where the message can still name the command that produced nothing.
fn capture(socket: &Path, pane: &str) -> Result<Vec<u8>, String> {
    let bytes = cli_raw(socket, &["get-text", "--pane-id", pane, "--escapes"])?;
    if bytes.is_empty() {
        return Err(format!(
            "`wezterm cli get-text` returned no bytes at all against {} — an empty capture \
             compares equal against a blank region and must never reach the comparison",
            socket.display()
        ));
    }
    Ok(bytes)
}

/// One control command against our own GUI socket, as bytes.
///
/// **`--no-auto-start`, always.** See the module docs: without it a socket that is not there is
/// *repaired* by starting a mux-server daemon, and the daemon's own shell pane is then photographed
/// and reported as the scene.
///
/// **Bytes and not a `String`**, for the reason the tmux arm gives: the capture is evidence, and
/// `from_utf8_lossy` on the way through would replace a malformed sequence here, where the parser
/// is the thing that is supposed to notice it.
fn cli_raw(socket: &Path, args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new(WEZTERM)
        .arg("cli")
        .arg("--no-auto-start")
        .args(args)
        .env("WEZTERM_UNIX_SOCKET", socket)
        .output()
        .map_err(|e| format!("wezterm cli {}: {e}", args.join(" ")))?;
    if !out.status.success() {
        return Err(format!(
            "wezterm cli {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(out.stdout)
}
