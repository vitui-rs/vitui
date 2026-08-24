//! The tmux arm: drive the engine inside a tmux pane and read tmux's grid back.
//!
//! **One executable, two halves**, the same shape as the Ghostty arm: with no arguments this is the
//! driver, with `--scene 01` it is the scene, and the driver launches the scene by re-running its own
//! [`std::env::current_exe`]. The scene, the readiness handshake and the comparison come from
//! [`common`] and are byte-for-byte the ones the Ghostty arm runs — which is the whole reason this
//! file exists as a sibling rather than as a copy. Production ticket 05 asks for the eleven attribute
//! facts *on at least two families*; two copies of the scene would make a disagreement between the
//! arms unattributable, since it could be the software or it could be the drift.
//!
//! ```sh
//! cd conform && cargo run --example tmux          # writes REPORT-tmux.md
//! ```
//!
//! # What this arm measures, and it is not the emulator behind it
//!
//! `capture-pane -p -e` re-serialises **tmux's** grid. The engine's bytes are parsed by tmux, stored
//! in tmux's cells, and handed back by tmux — the emulator on the other side of tmux never sees them
//! and is not being measured. That is a legitimate target and never a proxy: tmux is in spec §10's
//! own tier-1 list, and `quirks.rs` already cites its 1 s synchronised-output limit. `SCENES.md` says
//! so at the top, and so does every report this writes.
//!
//! # Three things it can do that the Ghostty arm cannot
//!
//! 1. **Set the geometry.** `new-session -x -y` is a real size, where `surface configuration` offers a
//!    font size and nothing else. So this arm's rows are not at the mercy of a window that settles
//!    its size after the process inside it starts — the defect that made stage 1's first run
//!    photograph a screen whose top two rows had scrolled off.
//! 2. **Run headless.** No window server, no macOS automation grant, no focus stolen for a second
//!    and a half. This is the arm that could run somewhere other than one developer's machine.
//! 3. **Answer from a config nobody edited.** `-f /dev/null`, so the result is about tmux and not
//!    about `~/.tmux.conf` — a `terminal-features` line in a user's config could change what comes
//!    back and would make the run unreproducible for anyone else.
//!
//! It still **reports and does not gate**, for the reason `conform/README.md` gives: the result
//! depends on tmux's version, which is the thing being measured. `cargo test` over `fixtures/` is the
//! gate.
//!
//! # The refusals, and this arm needs one the Ghostty arm does not
//!
//! A private socket per run, named for the process, and **the server on it must not already exist**.
//! Ghostty's arm addresses a window by set difference and insists the difference is exactly one; the
//! equivalent here is that the server we talk to is one we created, because a `-L` name that happened
//! to collide would put the scene in a stranger's session and photograph their screen. The rest are
//! shared with the other arm: exactly one pane, a pane that is not dead, and a dump that goes through
//! [`vitui_conform::parse`] with the row count the scene declared.

mod common;

use std::path::Path;
use std::process::Command;
use std::time::Instant;

use common::{
    Arm, Excluded, publish, render, save_if_asked, scene, scene_argv, wait_for_quiescence,
};
use vitui_conform::{Dialect, parse};

/// The pane size this arm asks for.
///
/// Wide enough that an attribute leaking past its label has somewhere to leak *to* — the check that
/// makes a reverse block running to the right edge a disagreement rather than a pass — and eleven
/// rows plus room, so nothing scrolls. Twenty-four is also tmux's own default, so a run that ignored
/// `-y` would be invisible here; the report prints what the scene observed rather than what was
/// asked for, which is where that would show.
const SIZE: (u16, u16) = (80, 24);

/// The session, window and pane this arm addresses.
///
/// A name and not an index: tmux indices depend on `base-index`, which is exactly the kind of thing a
/// user's config sets. `-f /dev/null` means ours cannot be set, and addressing by name means it would
/// not matter if it were.
const SESSION: &str = "conform";

/// The row this arm no longer compares, and why it stopped.
///
/// **`capture-pane -e` spells every one of the eleven, overline included** — this arm reported 11/11
/// on 2026-08-23 and the committed fixture holds all eleven. What changed is not tmux: production
/// ticket 10 wired `attrs_dropped` on to the wire, so the engine consults `quirks.rs`, sees the
/// fourth entry, and does not send SGR 53 to a terminal that answers XTVERSION `tmux …`.
///
/// So the row is the quirk table working. `FAILED` would blame tmux for a decision of ours, and
/// **the instrument has lost the measurement that earned the entry** — which is not a defect to fix
/// here: an arm that asked the engine what to expect would be checking the engine against itself.
/// `fixtures/tmux-3.7c-scene01-attrs.vt` is the capture taken before the entry existed, and that is
/// what the rule against regenerating a fixture is protecting.
const NOT_COMPARED: &[(&str, Excluded, &str)] = &[(
    "overline",
    Excluded::ByDesign,
    "`quirks.rs`'s fourth entry: tmux accepts SGR 53, stores it, hands it back to `capture-pane` \
     and never forwards it, so the engine stopped sending it when production ticket 10 wired the \
     mask. This arm reported 11/11 while it still did, and \
     `fixtures/tmux-3.7c-scene01-attrs.vt` is that capture",
)];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--scene") => scene(args.get(1).map(String::as_str)),
        _ => match drive() {
            Ok(()) => {}
            Err(why) => {
                eprintln!("FAILED: {why}");
                std::process::exit(1);
            }
        },
    }
}

fn drive() -> Result<(), String> {
    // `new-session` takes a shell command line, so the shared argv is joined here — see
    // `common::scene_argv`, where the launchers' disagreement about that is written down.
    let command = scene_argv()?.join(" ");
    let ready = std::env::temp_dir().join(format!("conform-ready-{}", std::process::id()));
    let _ = std::fs::remove_file(&ready);

    // Named for this process, so two runs cannot meet. `-L` is a socket name under tmux's own
    // directory rather than a path, which keeps the permissions tmux chose for it.
    let socket = format!("conform-{}", std::process::id());
    let version = tmux_version()?;

    // **The refusal this arm needs and the other does not.** A server already listening on our
    // socket name is somebody else's, and `new-session` would join it rather than create one — so
    // the pane the scene lands in, and the screen this photographs, would be theirs.
    if tmux(&socket, &["list-sessions"]).is_ok() {
        return Err(format!(
            "a tmux server is already listening on socket {socket:?}; this run cannot say whose \
             session it would be photographing"
        ));
    }

    let launched = Instant::now();
    tmux(
        &socket,
        &[
            "new-session",
            "-d",
            "-s",
            SESSION,
            "-x",
            &SIZE.0.to_string(),
            "-y",
            &SIZE.1.to_string(),
            "-e",
            &format!("CONFORM_READY={}", ready.display()),
            &command,
        ],
    )?;

    let outcome = capture_and_compare(&socket, &version, &ready, launched);

    // Killed before the result is unwrapped, like the Ghostty arm closes its window: a failed run
    // that leaves a server behind is a run that needs a human before the next one can start. This is
    // *our* socket, so `kill-server` cannot reach the user's own tmux.
    let _ = tmux(&socket, &["kill-server"]);
    let _ = std::fs::remove_file(&ready);

    let (arm, report, failures) = outcome?;
    publish(&arm, &report, failures)
}

/// Wait for the frame, read the grid back, parse it, compare it, and render the report.
fn capture_and_compare(
    socket: &str,
    version: &str,
    ready: &Path,
    launched: Instant,
) -> Result<(Arm, String, usize), String> {
    // Insisted on before the wait, not after: a scene that never started leaves the readiness timeout
    // to explain it, and "the scene never reported a presented frame" is twenty seconds spent saying
    // something this line says at once.
    let panes = tmux(socket, &["list-panes", "-t", SESSION])?;
    let count = panes.lines().filter(|l| !l.trim().is_empty()).count();
    if count != 1 {
        return Err(format!(
            "{count} panes in the session, not one — this run cannot say which one it captured"
        ));
    }

    let size = wait_for_quiescence(ready)?;

    // **A dead pane's capture is not evidence.** If the scene exited, tmux keeps the pane's last
    // grid and hands it back exactly as it hands back a live one, so `capture-pane` succeeds and the
    // rows may even compare equal. That is the same shape as the empty capture that reads as
    // agreement, one level along: not an absent screen but a *stale* one.
    let dead = tmux(
        socket,
        &["display-message", "-p", "-t", SESSION, "#{pane_dead}"],
    )?;
    if dead.trim() != "0" {
        return Err(
            "the pane is dead — the scene exited, so the grid tmux would hand back is whatever was \
             on it when the process died and not a screen anything is presenting"
                .into(),
        );
    }

    let started = Instant::now();
    let bytes = tmux_bytes(socket, &["capture-pane", "-p", "-e", "-t", SESSION])?;
    let capture_elapsed = started.elapsed();
    save_if_asked(&bytes)?;

    // **`TmuxCapturePane` and not `Ecma48`, and the difference is a whole attribute.** tmux writes
    // any attribute code of two digits as `code/10 : code%10`, so overline arrives as `5:3` — which
    // read as ECMA-48 is *blink*, an attribute tmux never rendered and the instrument would have
    // invented. See the parser's module docs and `FINDINGS.md`.
    let dump = parse(&bytes, common::scene01().len(), Dialect::TmuxCapturePane)
        .map_err(|e| format!("the capture is not a screen: {e}"))?;

    let default_terminal = tmux(socket, &["show-options", "-gv", "default-terminal"])
        .unwrap_or_else(|_| "unknown".into());

    let arm = Arm {
        title: "tmux",
        version: version.to_string(),
        mechanism: "`capture-pane -p -e`",
        measures: "**tmux**, and never the emulator behind it. `capture-pane` re-serialises tmux's \
                   own grid, so the engine's bytes were parsed and stored by tmux and handed back by \
                   tmux. A legitimate target — tmux is in spec §10's tier-1 list — and never a proxy \
                   for the terminal it is running inside",
        size,
        not_compared: NOT_COMPARED,
        notes: vec![
            format!(
                "**Geometry: asked for and got.** `new-session -x {} -y {}`, which is the one thing \
                 this arm can do that the Ghostty arm cannot. The size above is what the *scene* \
                 reported, so the two disagreeing would be visible here",
                SIZE.0, SIZE.1
            ),
            format!(
                "**Config:** `-f /dev/null`, so this is tmux's own defaults and not a user's \
                 `~/.tmux.conf`. `default-terminal` was `{}`",
                default_terminal.trim()
            ),
            format!(
                "**Launch to capture:** {} ms, of which `capture-pane` itself was {} ms — reported, \
                 never gated. Headless: no window server, no automation grant, no focus taken",
                launched.elapsed().as_millis(),
                capture_elapsed.as_millis()
            ),
            "**Trailing blanks:** `capture-pane` trims the default-styled ones the engine painted, \
             where Ghostty's `vt` dump keeps them. It does **not** trim a *styled* blank, so an \
             attribute leaking past its label is still counted as the disagreement it is"
                .to_string(),
        ],
    };
    let (report, failures) = render(&arm, &dump, &bytes);
    Ok((arm, report, failures))
}

/// What tmux says its version is. Asked of the binary that is about to run, never assumed.
fn tmux_version() -> Result<String, String> {
    let out = Command::new("tmux")
        .arg("-V")
        .output()
        .map_err(|e| format!("tmux -V: {e} — this arm needs tmux on PATH"))?;
    if !out.status.success() {
        return Err("tmux -V failed".into());
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .trim()
        .trim_start_matches("tmux ")
        .to_string())
}

/// One tmux command against our own socket, as text.
fn tmux(socket: &str, args: &[&str]) -> Result<String, String> {
    let bytes = tmux_bytes(socket, args)?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

/// One tmux command against our own socket, as bytes.
///
/// **Bytes and not a `String` for the capture**, because the capture is evidence: `from_utf8_lossy`
/// on the way through would replace a malformed sequence here, where the parser is the thing that is
/// supposed to notice it.
fn tmux_bytes(socket: &str, args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("tmux")
        // `-L` before the command, and `-f /dev/null` with it: a config file is read when the server
        // starts, which is the first command that needs one.
        .args(["-L", socket, "-f", "/dev/null"])
        .args(args)
        .output()
        .map_err(|e| format!("tmux {}: {e}", args.join(" ")))?;
    if !out.status.success() {
        return Err(format!(
            "tmux {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(out.stdout)
}
