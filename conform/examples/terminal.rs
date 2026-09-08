//! The Terminal.app arm: drive the engine inside macOS's own terminal and read its screen back.
//!
//! **One executable, two halves**, the shape every arm here has: with no arguments this is the
//! driver, with `--scene 01` it is the scene, and the driver launches the scene by re-running its
//! own [`std::env::current_exe`]. The scene, the readiness handshake and the comparison come from
//! [`common`] and are byte-for-byte the ones the other three arms run.
//!
//! ```sh
//! cd conform && cargo run --example terminal       # writes REPORT-terminal.md
//! ```
//!
//! # Why this arm exists, and it is the one thing the other three could not supply
//!
//! Ghostty 1.3.1, kitty 0.48.2 and tmux 3.7c are recent, they all implement UAX #29 clustering, and
//! on 2026-08-29 they answered scene 05 **identically** — twelve surveyed rows, three columns, no
//! disagreement anywhere. `conform/FINDINGS.md` recorded what that costs: *four identical columns is
//! not a result*, and a survey whose every arm agrees cannot say whether it is measuring the
//! terminals or measuring its own tables. It named two candidates for an arm that would disagree, and
//! this is the first of them — **a different VT lineage**, shipped on every Mac, whose emoji handling
//! predates most of what the other three implement.
//!
//! It disagrees. See `REPORT-terminal.md`.
//!
//! # What it cannot do, declared from the `sdef` and then from a run
//!
//! `contents` and `history` on Terminal.app's `tab` class are `type="text" access="r"` and there is
//! no styled variant anywhere on that class. **So every capture from this arm is plain text**, and
//! that is a fact about the *capture surface* rather than about what Terminal.app renders — it
//! renders bold perfectly well and this arm cannot see that it did. [`NO_STYLE`] is where that is
//! declared once, and [`common::Arm::no_style`] is what carries it into the two scenes it reaches:
//! all eleven rows of scene 01, and scene 04's one row whose value is a style it reports.
//!
//! a later pass recorded this arm as *glyph-grid scenes only* on the strength of that `sdef`
//! line, and then scene 05 was built and made the sentence too small. A cursor report comes back **in
//! band on the scene's own tty**, so the capture surface is not in its path at all — which is why
//! this arm answers scene 05 in full, scene 04 as text, scene 06 in full, and only scene 01 not at
//! all. Every scene of [`SCENES`] is run and every row of every scene is printed, because a missing
//! row reads as a win.
//!
//! # It is the only *emulator* arm that can be handed a size and then insist on it
//!
//! `number of rows` and `number of columns` on the `window` class carry no `access="r"`, and
//! [`REQUESTED`] is set on the window **before the scene is started in it** — a two-step launch
//! this arm does for that reason alone. kitty can ask for a size on its command line and report back
//! what it got; this one can set it afterwards and be refused if it did not take. The quiescence
//! handshake is kept regardless, for the reason the kitty arm gives: a declaration is not an
//! observation, and the handshake is what observes that the window stopped moving.
//!
//! # The refusals, written before the first assertion
//!
//! Five, and the fifth is this arm's own.
//!
//! 1. creating the window must add **exactly one** terminal, or the set difference is not an address.
//! 2. the geometry read back must be the geometry that was asked for. No other emulator arm can make
//!    this check, and it replaces nothing — the handshake still runs.
//! 3. the environment the scene runs in is **built rather than inherited**. See [`NEUTRALISED`].
//! 4. an empty capture is `FAILED` and never a match — `screen -X hardcopy`'s zero bytes, which this
//!    whole directory's first line of code was written against.
//! 5. **scene 06's silence is an observation and only silence is.** Terminal.app answers no DECRQM at
//!    all, and [`vitui_conform::ModeError::Unanswered`] is the refusal that says so without saying
//!    *the terminal lost an answer*. A short batch stays a refusal.

mod common;

use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

use common::{
    AnswersInBand, Arm, Excluded, SCENES, clear_handshake, header, publish, save_if_asked, scene,
    scene_argv, section, trailer, wait_for_quiescence,
};
use vitui_conform::Dialect;

/// The window size this arm asks for, in cells, and then **insists on**.
///
/// Eleven rows plus room so nothing scrolls, and wide enough that an attribute leaking past its
/// label would have somewhere to leak to. Set on the window before the scene is started in it, which
/// is why `drive` opens the window and runs the command in two steps rather than one.
const REQUESTED: (u16, u16) = (80, 24);

/// Every environment variable the engine's detection reads that Terminal.app does not set itself.
///
/// # This is a refusal, and the shape of it is the tmux arm's `-f /dev/null`
///
/// `do script` runs the command in a **login shell**, so the user's profile has already run by the
/// time the scene starts. On the machine this arm was built on that profile contains
/// `export COLORTERM=truecolor` unconditionally — and Terminal.app 2.15 is a 256-colour terminal.
/// Inherited, the run would have recorded a truecolor capability that belongs to a line in somebody's
/// `.zshrc`, and the result would be about that file rather than about Terminal.app.
///
/// So the command line is an explicit `env` with these unset. `TERM`, `TERM_PROGRAM` and
/// `TERM_PROGRAM_VERSION` are deliberately **not** here: those three are Terminal.app's own and
/// clearing them would be a different lie.
///
/// **It moves no row in any scene as this suite stands, and that is luck rather than safety.** None
/// of the four scenes reads a colour the engine chose — scene 01 draws attributes and no colour, and
/// scene 04 writes its `SGR 41` as a raw byte with no engine in the path. The refusal is here for the
/// run after the one that adds a scene which does.
const NEUTRALISED: &[&str] = &[
    "COLORTERM",
    "TERMINAL_EMULATOR",
    "TERMUX_VERSION",
    "TMUX",
    "NO_COLOR",
    "VITUI_FORCE_LEGACY_SGR",
    "VITUI_FORCE_COLOR",
    "VITUI_GLYPHS",
];

/// Why this arm's capture surface carries no style, declared once. See [`common::Arm::no_style`].
const NO_STYLE: &str = "see **Style is not readable here**, above the tables";

/// The same fact at length, as a note above the tables, said once instead of once per row.
///
/// Eleven rows carrying ninety words each is a table nobody reads, and a reader who skips it has
/// skipped the declaration rather than the decoration. [`NO_STYLE`] points here.
const NO_STYLE_NOTE: &str = "**Style is not readable here, and that is a fact about the capture \
                             surface.** Terminal.app's scripting surface offers `contents` and \
                             `history` as `type=\"text\" access=\"r\"`, and the `tab` class has no \
                             styled variant anywhere — confirmed from the application's own `sdef` \
                             and then from every capture this arm has taken. So scene 01's eleven \
                             rows are `cannot ask` and scene 04's one reported style is \
                             unreportable. **Nothing here may be read as a claim about what \
                             Terminal.app renders**: it draws bold, and no property reachable over \
                             AppleScript can say that it did. Seeing that would take a \
                             `--through-tmux`-shaped arm with a styled reader on the far side, and \
                             Terminal.app is an endpoint. What the rows still say is whether the \
                             *text* survived, which is why a scrolled or mis-sized screen is as \
                             loud on this arm as on any other";

/// The rows this arm does not compare, and why.
///
/// Declared here, above the run, because a limitation discovered after seeing the answer is an
/// excuse rather than a declaration. **All five are `cannot express`, and this arm is the first here
/// to construct that kind at all** — `compare/` supplied the word, `SCENES.md` inherited it on the
/// day this directory was laid out, and the three emulator families that came first all had every
/// capability the scenes ask about.
///
/// Terminal.app 2.15 has no synchronised output. A control probe with no engine in it — five raw
/// `CSI ? 2026 $ p` writes and a `CSI c` behind them — brought back the device-attributes reply and
/// nothing else, and the seven `p`s the engine's own capability batch leaves on the screen say why:
/// Terminal.app's parser does not take `$` as an intermediate byte, so this is not a query it
/// declines but a sequence it never finishes reading. `detect.rs` reaches the same conclusion from
/// the same silence and reports `sync_output false`, which is the engine behaving correctly on a
/// terminal that has no such mode.
///
/// A `FAILED` on these five would be this suite demanding a feature of a terminal that never claimed
/// one. The rows are still printed, with `no reply` beside them.
const NOT_COMPARED: &[(&str, Excluded, &str)] = &[
    (
        "before",
        Excluded::CannotExpress,
        "Terminal.app 2.15 has no synchronised output and does not answer DECRQM about it. Not \
         `not recognised (0)`, which is DEC's own way of declining — **nothing at all**, because \
         its parser does not take `$` as an intermediate and never finishes reading the query. The \
         `CSI c` behind the batch is what makes that an observation rather than a timeout, and the \
         seven `p`s the engine's own capability batch leaves on this terminal's screen are the same \
         fact seen from the other side",
    ),
    (
        "while-open",
        Excluded::CannotExpress,
        "the same silence. A terminal with no mode has no state machine to track it",
    ),
    ("after-close", Excluded::CannotExpress, "the same silence"),
    ("opened-twice", Excluded::CannotExpress, "the same silence"),
    (
        "closed-once",
        Excluded::CannotExpress,
        "the same silence — and this is the row the engine depends on, so it is worth saying what \
         its absence does and does not mean. §8 wraps every frame in a balanced pair, and a \
         terminal that counted the sets would hold a frame past the close sent for it. A terminal \
         with no mode at all cannot: there is nothing to hold it, `detect.rs` reports \
         `sync_output false` from this same silence, and `serial.rs` then wraps nothing. The row \
         is unanswerable here and the property is not at risk",
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

/// A Terminal.app window this run opened, closed when it goes out of scope.
///
/// The guards below return early, and a window left open leaves the scene's process running on the
/// user's screen — a redraw loop that never ends, and for scenes 05 and 06 a tty the scene put into
/// raw mode and deliberately never restores, because the terminal it runs in is one the arm creates
/// and destroys. **Not for the addressing**, which is the reason the Ghostty arm gives: a stray
/// window here would be in the *next* run's `was` snapshot as well as in its second one, and would
/// cancel. See `one_scene`.
struct Window(String);

impl Drop for Window {
    fn drop(&mut self) {
        let _ = osascript(&format!(
            r#"tell application "Terminal" to close window id {}"#,
            self.0
        ));
    }
}

/// Every scene, each in its own window, into one report.
///
/// A window per scene rather than a scene switch inside one, for the reason the kitty arm's `drive`
/// gives: a window that has been written to once is a window whose state is now part of the
/// measurement.
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
    let ready = std::env::temp_dir().join(format!("conform-ready-{}-{which}", std::process::id()));
    clear_handshake(&ready);

    let was = window_ids()?;
    let launched = Instant::now();
    // **Two steps, and the size is the reason.** `do script "cmd"` would open the window and start
    // the command in one call, at whatever size Terminal.app's default profile gives — and the
    // engine would then be told a geometry that changes underneath it a moment later. An empty
    // `do script` opens a window running only the login shell; the size goes on before anything
    // that cares about it exists.
    let id = osascript(
        r#"tell application "Terminal"
             set t to do script ""
             return id of (first window whose selected tab is t) as text
           end tell"#,
    )?;
    let window = Window(id.clone());

    // The set difference is still checked, even though the line above named the window directly:
    // `first window whose selected tab is t` is a query, and a query that matched two windows would
    // have returned one of them rather than failing. An earlier pass's rule, and it costs one round trip.
    //
    // **`id of every window` is cumulative, not live**, and that was measured rather than assumed:
    // a closed window stays in the list for the life of the Terminal.app process, with `visible`
    // false and its name still readable. Thirty-five entries after `close every window` on the
    // machine this was built on.
    //
    // The difference is still an address, and it is worth saying why rather than leaving a reader
    // to work it out. `was` is read **immediately before** the open, so a stale id is in both
    // snapshots and cancels; what the difference contains is what appeared between the two calls.
    // A window the user opened in that instant makes it two and fails the run, which is the
    // outcome the check exists for. Filtering on `visible` would make the list live and would also
    // put *a window that has not finished appearing* into the refusal's path, which is a race this
    // one does not have.
    let fresh: Vec<String> = window_ids()?
        .into_iter()
        .filter(|w| !was.contains(w))
        .collect();
    if fresh.len() != 1 || fresh[0] != id {
        return Err(format!(
            "opening the window added {} terminals rather than exactly one, so the set difference \
             is not an address and this run cannot say which window it captured",
            fresh.len()
        ));
    }

    let size = set_geometry(&id)?;
    run_scene(&id, &ready, which)?;

    let outcome = capture_and_compare(which, &id, &ready, launched, &size);
    // Closed before the result is unwrapped, like the other two windowed arms.
    drop(window);
    clear_handshake(&ready);
    outcome
}

/// Hand the window a size and refuse it if it did not take.
///
/// **The check no other emulator arm can make.** Ghostty's `surface configuration` offers a font size
/// and no rows or columns; kitty's `initial_window_width` is a request reported back by `kitten @ ls`
/// rather than an assignment. Here the two properties are read-write on the `window` class, so the
/// arm can set them and then read them, and a window that came back some other size is a run whose
/// every row would be about a screen the scene did not design.
fn set_geometry(id: &str) -> Result<String, String> {
    osascript(&format!(
        r#"tell application "Terminal"
             set number of columns of window id {id} to {}
             set number of rows of window id {id} to {}
           end tell"#,
        REQUESTED.0, REQUESTED.1
    ))?;
    let got = osascript(&format!(
        r#"tell application "Terminal" to return ((number of columns of window id {id}) as text) & "x" & ((number of rows of window id {id}) as text)"#
    ))?;
    let want = format!("{}x{}", REQUESTED.0, REQUESTED.1);
    if got != want {
        return Err(format!(
            "the window was asked for {want} and reports {got} — this arm is the one that can \
             insist, so it does"
        ));
    }
    Ok(got)
}

/// Type the scene's command line into the window that is already the right size.
///
/// The environment is built here rather than inherited: see [`NEUTRALISED`]. `CONFORM_READY` goes on
/// the same `env` for the reason the other arms pass it their own way — three launchers, three
/// mechanisms, and `common` supplies the argv rather than a guess at anyone's quoting rule.
fn run_scene(id: &str, ready: &Path, which: &str) -> Result<(), String> {
    let mut command = String::from("/usr/bin/env");
    for name in NEUTRALISED {
        command.push_str(&format!(" -u {name}"));
    }
    command.push_str(&format!(" CONFORM_READY={}", ready.display()));
    for word in scene_argv(which)? {
        // Single quotes for the shell, inside a double-quoted AppleScript literal. The path comes
        // from `current_exe` under this workspace's `target/`, so there is nothing here for either
        // quoting rule to trip on — and quoting it anyway is what keeps that true.
        command.push_str(&format!(" '{word}'"));
    }
    osascript(&format!(
        r#"tell application "Terminal" to do script "{command}" in selected tab of window id {id}"#
    ))?;
    Ok(())
}

/// Wait for the frame, read the screen back, compare it, and render the report.
fn capture_and_compare(
    which: &str,
    id: &str,
    ready: &Path,
    launched: Instant,
    geometry: &str,
) -> Result<(Arm, common::Capture, Vec<u8>, String), String> {
    let size = wait_for_quiescence(ready, which)?;

    // **`Ecma48` over a capture with no escape sequences in it at all**, which is not a contradiction:
    // the dialects differ over what a colon means inside SGR, and a plain-text capture has no SGR to
    // disagree about. Naming the ECMA-48 one says the parser is reading the bytes as written rather
    // than folding anything.
    //
    // **`contents` is not called for scenes 05 and 06** — `common::capture`'s decision rather than
    // this arm's, and the one that makes this arm worth having: those two are answered in band on the
    // scene's own tty, where a capture surface that carries no style is not in the path at all.
    let mut capture_elapsed = Duration::ZERO;
    let (captured, bytes) = common::capture(which, ready, Dialect::Ecma48, || {
        let started = Instant::now();
        let bytes = capture(id)?;
        capture_elapsed = started.elapsed();
        Ok(bytes)
    })?;
    save_if_asked(which, Dialect::Ecma48, &bytes)?;

    let arm = Arm {
        title: "Terminal.app",
        version: osascript(r#"tell application "Terminal" to get version"#)
            .unwrap_or_else(|_| "unknown".into()),
        mechanism: "`contents of selected tab`, over AppleScript",
        measures: "Terminal.app — **a different VT lineage from the other three arms**, and the \
                   only one here that is not a recent reimplementation. Its screen, read back as \
                   plain text, and its own answers to questions asked in band",
        answers_in_band: AnswersInBand {
            who: "Terminal.app",
            why: "Terminal.app is an endpoint, so nothing sits between the scene's tty and it. It \
                  is also the arm where the two channels are furthest apart: the photograph carries \
                  no style whatever, and an in-band reply carries everything the terminal knows",
        },
        not_compared: NOT_COMPARED,
        no_style: Some(NO_STYLE),
        notes: vec![
            NO_STYLE_NOTE.to_string(),
            format!(
                "**Geometry: set, and then insisted on.** `number of rows` and `number of columns` \
                 are read-write on the `window` class, so this arm assigns {} and reads it back, \
                 and a window that came back some other size fails the run. **The only *emulator* \
                 arm that can** — Ghostty offers a font size and kitty offers a request. The window \
                 is opened empty and sized *before* the scene is started in it, which is why the \
                 launch is two AppleScript calls rather than one. The quiescence handshake is kept \
                 anyway: a declaration is not an observation",
                format_args!("{}x{}", REQUESTED.0, REQUESTED.1)
            ),
            format!(
                "**Environment: built, not inherited.** `do script` runs a login shell, so the \
                 user's profile has run before the scene starts — and on this machine that profile \
                 exports `COLORTERM=truecolor` to a 256-colour terminal. The command line unsets \
                 the {} variables the engine's detection reads that Terminal.app does not set \
                 itself, and leaves `TERM`, `TERM_PROGRAM` and `TERM_PROGRAM_VERSION` alone \
                 because those three are the terminal's own. It moves no row in this suite as it \
                 stands, and that is luck rather than safety",
                NEUTRALISED.len()
            ),
            match which {
                "05" | "06" => format!(
                    "**Launch to answer:** {} ms — reported, never gated. **This scene is not \
                     photographed:** the terminal answers in band on the scene's own tty, so the \
                     `contents` property — the half of this arm that carries no style — is out of \
                     its path entirely, and what the arm did was open a window, size it and start \
                     the scene. **The automation grant is still in the path**, three Apple Events \
                     before the scene runs; what an in-band scene does without is the *capture \
                     surface*",
                    launched.elapsed().as_millis()
                ),
                _ => format!(
                    "**Launch to capture:** {} ms, of which the `contents` round trip alone was \
                     {} ms — reported, never gated. It is AppleScript round trips and a window \
                     opening",
                    launched.elapsed().as_millis(),
                    capture_elapsed.as_millis()
                ),
            },
            format!(
                "**Surface, as the scene reported it: {size}**, against a window this arm set to \
                 {geometry}. The two are printed side by side because they are different \
                 measurements — one is what AppleScript says the window is, the other is what \
                 `TIOCGWINSZ` told the process inside it, and an arm that printed one of them twice \
                 would not notice them coming apart"
            ),
            "**A never-written cell is not a painted blank, and only the first is trimmed** — \
             the kitty arm's finding, reproduced here through a different surface. Scene 01's \
             capture comes back at the full 80 columns because the engine paints every cell of the \
             alt screen; scene 04's rows come back at their written length, because a raw-byte \
             probe writes six short rows on to a primary screen whose remaining cells were never \
             touched. Both fixtures are committed and they disagree about padding, which is why \
             this is stated as a rule about cells rather than as one about this arm. Nothing in \
             these scenes turns on it: scene 04 compares with `trim_end`"
                .to_string(),
            "**The alt screen is what `contents` reads**, so scene 01's capture is the engine's \
             page and not the shell's. Scene 04 writes raw bytes to the primary screen with no \
             alt-screen switch at all, and erases each of its rows before writing it — so the \
             login banner and the echoed command line are gone from the six rows it is judged on"
                .to_string(),
        ],
    };
    Ok((arm, captured, bytes, size))
}

/// Read the screen back, refusing a reply of no bytes at all.
///
/// **The refusal that comes first**, and it is this directory's oldest: `screen -X hardcopy` exits 0
/// and writes a zero-byte file, and an empty capture compares equal against a blank region — the row
/// goes green and the missing result hides inside a passing one. [`vitui_conform::parse`] refuses an
/// empty dump too; this refuses it one step earlier, where the message can still name the property
/// that produced nothing.
fn capture(id: &str) -> Result<Vec<u8>, String> {
    let bytes = osascript_raw(&format!(
        r#"tell application "Terminal" to return contents of selected tab of window id {id}"#
    ))?;
    if bytes.iter().all(u8::is_ascii_whitespace) {
        return Err(format!(
            "`contents of selected tab of window id {id}` came back with nothing but whitespace — \
             an empty capture compares equal against a blank region and must never reach the \
             comparison"
        ));
    }
    Ok(bytes)
}

/// The id of every Terminal.app window, as text.
///
/// A `text item delimiters` of its own, because AppleScript coerces a list of integers to text by
/// running them together: two windows numbered 8559 and 8561 arrive as `85598561`, which is one
/// address that exists nowhere.
fn window_ids() -> Result<Vec<String>, String> {
    let out = osascript(
        r#"set text item delimiters to ","
           tell application "Terminal" to return (id of every window) as text"#,
    )?;
    Ok(out
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect())
}

/// One AppleScript, as trimmed text.
fn osascript(script: &str) -> Result<String, String> {
    Ok(String::from_utf8_lossy(&osascript_raw(script)?)
        .trim()
        .to_string())
}

/// The same, as bytes.
///
/// **Bytes and not a `String` for the capture**, for the reason the tmux arm gives: the capture is
/// evidence, and a lossy conversion on the way through would replace a malformed sequence here,
/// where the parser is the thing that is supposed to notice it. Scene 04 writes `漢`, so this path
/// carries non-ASCII on every run.
fn osascript_raw(script: &str) -> Result<Vec<u8>, String> {
    let out = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| format!("osascript: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "osascript failed: {} — this arm needs a window server and a macOS automation grant \
             for Terminal.app, neither of which can be obtained non-interactively",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(out.stdout)
}
