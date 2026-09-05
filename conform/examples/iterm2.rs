//! The iTerm2 arm: drive the engine inside iTerm2 and read its buffer back over the Python API.
//!
//! **One executable, two halves**, the shape every arm here has: with no arguments this is the
//! driver, with `--scene 01` it is the scene, and the driver launches the scene by re-running its
//! own [`std::env::current_exe`]. The scene, the readiness handshake and the comparison come from
//! [`common`] and are byte-for-byte the ones the other six arms run.
//!
//! ```sh
//! cd conform && cargo run --example iterm2      # writes REPORT-iterm2.md
//! ```
//!
//! # The capture surface was established rather than assumed, and there were two candidates
//!
//! Production ticket 13 named both and refused to pick one in advance, because the last time this
//! repository took a citation on trust it was wrong. Both were read before a line of this file was
//! written, and they are not close.
//!
//! **AppleScript carries no style.** `iTerm2.sdef` offers `contents` and `text` on the `session`
//! class, both `type="text"`, and there is no styled variant anywhere on that class — Terminal.app's
//! shape exactly, and it would have made all eleven rows of scene 01 `cannot ask`.
//!
//! **The Python API carries the cell.** `GetBufferRequest` has an `include_styles` flag, and with it
//! set each `LineContents` carries a run-length-encoded `CellStyle` per cell: `bold`, `faint`,
//! `italic`, `blink`, `underline`, `strikethrough`, `invisible`, `inverse`, plus `underlineColor` as
//! a colour of its own. That is **eight of this scene's eleven rows answerable** where AppleScript
//! answers none, so this arm speaks the API and uses AppleScript only for the three things the API
//! is not needed for: the window's life, its geometry, and the cookie.
//!
//! # What the API still cannot be asked, and the two rows are not the same kind of thing
//!
//! `CellStyle` is a **projection** of iTerm2's own cell rather than the cell itself, and the two
//! shortfalls sit on opposite sides of that projection.
//!
//! `underline` is a `bool`. iTerm2's `screen_char_t` carries `underlineStyle0:2` and
//! `underlineStyle1:1` — three bits, and its own help text names `4:3  Curly underline` — so the
//! terminal renders a double and a dotted underline perfectly well and this message says only
//! *underlined*. **A fact about the instrument**, kitty's `CSI 4 : m` in a different alphabet, and
//! [`Excluded::CannotAsk`] is what says so.
//!
//! Overline is **not in the cell at all**. `screen_char_t`'s bit field is `bold faint italic blink
//! underline image strikethrough underlineStyle0 invisible inverse guarded virtualPlaceholder
//! rtlStatus underlineStyle1`, and the string `overline` appears **zero** times in the whole
//! 88 MB binary — not as an attribute, not as a help string, not as a preference key. That is a
//! second source outside the capture saying the terminal does not implement SGR 53, which is
//! Alacritty's situation and not kitty's `cannot ask`. It earned `quirks.rs` its **eighth** entry,
//! and the row is [`Excluded::ByDesign`] because the engine now withholds the bit.
//! `fixtures/iterm2-3.6.11-scene01-attrs.pb` is the capture taken while it still sent it — see
//! [`common::Excluded::ByDesign`], and do not regenerate that fixture.
//!
//! # The transport is a websocket over a unix socket, and it is hand-rolled for this crate's reason
//!
//! `conform/Cargo.toml` has no dependencies, matching the engine's and the runtime's posture, and a
//! protobuf request behind an RFC 6455 handshake does not need a crate to be one. The client sends
//! one masked binary frame and reads one back; [`vitui_conform::Dialect::Iterm2Buffer`] does the
//! decoding, in the **library**, so the committed fixture is gated by `cargo test` the way every
//! other arm's is.
//!
//! **The handshake's `Sec-WebSocket-Accept` is deliberately not verified**, and that is a decision
//! rather than an omission: verifying it is a SHA-1 of a key this client chose, over a unix socket
//! in this user's own home directory, and it would prove that the peer can hash. What is checked is
//! the status line, the negotiated subprotocol, and — the one that matters — that the answer decodes
//! as the message that was asked for.
//!
//! # The refusals, written before the first assertion
//!
//! Six, and the sixth is this arm's own.
//!
//! 1. creating the window must add **exactly one** terminal, or the set difference is not an address.
//! 2. the geometry read back must be the geometry that was asked for. iTerm2's `session` class has
//!    `columns` and `rows` read-write, so this is Terminal.app's check on a second emulator.
//! 3. the environment the scene runs in is **built rather than inherited**. See [`NEUTRALISED`], and
//!    note what is deliberately **not** in it.
//! 4. an empty capture is `FAILED` and never a match — `screen -X hardcopy`'s zero bytes, which this
//!    whole directory's first line of code was written against.
//! 5. the API's own `status` must be `OK`. A session that has gone away answers `SESSION_NOT_FOUND`
//!    with no contents, which is an empty capture wearing a successful reply's clothes.
//! 6. **the cookie is requested per run and never cached.** `request cookie and key for app named`
//!    mints a fresh pair, and a run that reused a stale one would fail its handshake with a 401 that
//!    reads exactly like the API being switched off — two different operator actions behind one
//!    message.

mod common;

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime};

use common::{
    AnswersInBand, Arm, Excluded, SCENES, clear_handshake, header, publish, save_if_asked, scene,
    scene_argv, section, trailer, wait_for_quiescence,
};
use vitui_conform::Dialect;

/// The window size this arm asks for, in cells, and then **insists on**.
///
/// The Terminal.app arm's number, for the Terminal.app arm's reasons: eleven rows plus room so
/// nothing scrolls, and wide enough that an attribute leaking past its label would have somewhere to
/// leak to. Set on the *session* rather than the window — iTerm2 splits a window into sessions and
/// it is the session that has a grid.
const REQUESTED: (u16, u16) = (80, 24);

/// The name this run tells iTerm2 it is, in the cookie request and in the websocket handshake.
///
/// iTerm2 prints it in its own script console, so a person watching a run sees which client opened
/// the connection rather than an anonymous one.
const ADVISORY: &str = "vitui-conform";

/// Every environment variable the engine's detection reads that iTerm2 does not set itself.
///
/// # What is missing from this list is the interesting half
///
/// The Terminal.app arm unsets `COLORTERM` because `do script` runs a **login shell** and the user's
/// profile exports `truecolor` to a 256-colour terminal. Neither half of that sentence is true here.
/// iTerm2's `create window … command` execs the command directly, with no shell and no profile; and
/// `COLORTERM=truecolor` arrives anyway, because **iTerm2 sets it itself** — the string and the
/// value are both in its binary, and `launchctl getenv COLORTERM` is empty on this machine, so it is
/// not inherited from the session either. Unsetting it would be the lie the Terminal.app arm avoids
/// by leaving `TERM_PROGRAM` alone: it is the terminal's own variable, about itself, and it is true.
///
/// `TERM`, `TERM_PROGRAM`, `TERM_PROGRAM_VERSION`, `LC_TERMINAL` and `LC_TERMINAL_VERSION` are
/// iTerm2's own for the same reason and are left alone. What is unset is everything a *different*
/// terminal or a *previous* run could have left in the environment this process was started from.
const NEUTRALISED: &[&str] = &[
    "TERMINAL_EMULATOR",
    "TERMUX_VERSION",
    "TMUX",
    "ALACRITTY_WINDOW_ID",
    "NO_COLOR",
    "VITUI_FORCE_LEGACY_SGR",
    "VITUI_FORCE_COLOR",
    "VITUI_GLYPHS",
];

/// The rows this arm does not compare, and why.
///
/// Declared here, above the run, because a limitation discovered after seeing the answer is an
/// excuse rather than a declaration. An arm that consulted `quirks.rs` for what to expect would be
/// checking the engine against itself, which is the arrangement `conform/` exists to break.
///
/// **Both kinds are here and they are this arm's whole story**, which is why the module docs argue
/// them at length: two rows the instrument cannot see, one row the engine deliberately no longer
/// sends. The three of them are the difference between a projection dropping a bit and a cell never
/// having had one.
const NOT_COMPARED: &[(&str, Excluded, &str)] = &[
    (
        "overline",
        Excluded::ByDesign,
        "`quirks.rs`'s eighth entry, earned by this arm's first run and by iTerm2's own binary. \
         `screen_char_t` — the cell struct, readable from the shipped binary's Objective-C type \
         encoding — enumerates `bold faint italic blink underline image strikethrough \
         underlineStyle0 invisible inverse guarded virtualPlaceholder rtlStatus underlineStyle1`, \
         and there is no bit for an overline; the string `overline` appears **zero** times in the \
         binary. The engine now withholds SGR 53, and \
         `fixtures/iterm2-3.6.11-scene01-attrs.pb` is the capture taken while it still sent it",
    ),
    (
        "under-dbl",
        Excluded::CannotAsk,
        "`CellStyle.underline` is a `bool`. iTerm2's cell carries a three-bit `underlineStyle` and \
         its own help text names `4:3  Curly underline`, so the terminal renders a double underline \
         and the API says only *underlined*. A fact about the **capture format** and not about what \
         iTerm2 draws — kitty's `CSI 4 : m` reached through a completely different surface, which \
         is what makes the pair worth reading together",
    ),
    (
        "under-dot",
        Excluded::CannotAsk,
        "the same `bool`, and the same three-bit field behind it. This is the row Alacritty's grid \
         could answer and neither kitty's serialiser nor this projection can, so the suite now has \
         all three readings of one question: rendered and spelled, rendered and unspelled, and \
         stored as a bit of its own",
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

/// An iTerm2 window this run opened, closed when it goes out of scope.
///
/// The guards below return early, and a window left open leaves the scene's process running on the
/// user's screen — a redraw loop that never ends, and for scenes 05 and 06 a tty the scene put into
/// raw mode and deliberately never restores, because the terminal it runs in is one the arm creates
/// and destroys. It is also the addressing, unlike the Terminal.app arm's: iTerm2's `id of every
/// window` is **live**, so a stray window from a previous run would be in the next run's `was`
/// snapshot and would not cancel — it would simply be there, and the set difference would still be
/// one. What a stray window costs here is the user's screen rather than the address.
struct Window(String);

impl Drop for Window {
    fn drop(&mut self) {
        let _ = osascript(&format!(
            r#"tell application "iTerm2" to close (every window whose id is {})"#,
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
    // **One step, where the Terminal.app arm needs two.** `create window … command` execs the
    // command rather than typing it into a login shell, so there is no profile between the scene and
    // the terminal — and the geometry can go on afterwards, because iTerm2 reflows the session and
    // the scene redraws on the resize. The quiescence handshake is what observes that it settled.
    let id = osascript(&format!(
        r#"tell application "iTerm2"
             set w to (create window with default profile command "{}")
             return id of w as text
           end tell"#,
        applescript_string(&command_line(&ready, which)?)
    ))?;
    let window = Window(id.clone());

    // The set difference is still checked, even though the line above named the window directly:
    // ticket 04's rule, and it costs one round trip. `was` is read immediately before the open, so a
    // window the user opened in that instant makes it two and fails the run — which is the outcome
    // the check exists for.
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

    let session = osascript(&format!(
        r#"tell application "iTerm2" to return id of current session of (first window whose id is {id})"#
    ))?;
    let size = set_geometry(&id)?;

    let outcome = capture_and_compare(which, &session, &ready, launched, &size);
    // Closed before the result is unwrapped, like the other windowed arms.
    drop(window);
    clear_handshake(&ready);
    outcome
}

/// The command line the window runs, as one shell-free `env` invocation.
///
/// The environment is built here rather than inherited: see [`NEUTRALISED`]. `CONFORM_READY` goes on
/// the same `env` for the reason the other arms pass it their own way — six launchers, six
/// mechanisms, and `common` supplies the argv rather than a guess at anyone's quoting rule.
fn command_line(ready: &Path, which: &str) -> Result<String, String> {
    let mut command = String::from("/usr/bin/env");
    for name in NEUTRALISED {
        command.push_str(&format!(" -u {name}"));
    }
    command.push_str(&format!(" CONFORM_READY={}", ready.display()));
    for word in scene_argv(which)? {
        // Single quotes for the shell iTerm2 does not use — `create window … command` execs a
        // command line it splits itself, and the quoting rule is the same one. The path comes from
        // `current_exe` under this workspace's `target/`, so there is nothing here for it to trip
        // on, and quoting it anyway is what keeps that true.
        command.push_str(&format!(" '{word}'"));
    }
    Ok(command)
}

/// Hand the session a size and refuse it if it did not take.
///
/// **Terminal.app's check, on a second emulator.** `columns` and `rows` carry no `access="r"` on
/// iTerm2's `session` class, so the arm can assign them and then read them back, and a session that
/// came back some other size is a run whose every row would be about a screen the scene did not
/// design. It is set *after* the command starts rather than before, which Terminal.app's arm cannot
/// do — there the window is opened empty and sized first, because `do script` would otherwise start
/// the command at the profile's geometry. Here the scene is an engine application that redraws on
/// the resize, and the handshake is what waits for it.
fn set_geometry(id: &str) -> Result<String, String> {
    osascript(&format!(
        r#"tell application "iTerm2"
             tell current session of (first window whose id is {id})
               set columns to {}
               set rows to {}
             end tell
           end tell"#,
        REQUESTED.0, REQUESTED.1
    ))?;
    let got = osascript(&format!(
        r#"tell application "iTerm2" to tell current session of (first window whose id is {id}) to return ((columns as text) & "x" & (rows as text))"#
    ))?;
    let want = format!("{}x{}", REQUESTED.0, REQUESTED.1);
    if got != want {
        return Err(format!(
            "the session was asked for {want} and reports {got} — this arm is one of the two that \
             can insist, so it does"
        ));
    }
    Ok(got)
}

/// Wait for the frame, read the buffer back, compare it, and render the report.
fn capture_and_compare(
    which: &str,
    session: &str,
    ready: &Path,
    launched: Instant,
    geometry: &str,
) -> Result<(Arm, common::Capture, Vec<u8>, String), String> {
    let size = wait_for_quiescence(ready, which)?;

    // **`Iterm2Buffer` and not `Ecma48`**, and the difference is not a dialect: what comes back is a
    // protobuf `GetBufferResponse` with no escape sequence anywhere in it.
    //
    // **The API is not called for scenes 05 and 06** — `common::capture`'s decision rather than this
    // arm's: those two are answered in band on the scene's own tty, where the capture surface is not
    // in the path at all.
    let mut capture_elapsed = Duration::ZERO;
    let (captured, bytes) = common::capture(which, ready, Dialect::Iterm2Buffer, || {
        let started = Instant::now();
        let bytes = capture(session)?;
        capture_elapsed = started.elapsed();
        Ok(bytes)
    })?;
    save_if_asked(which, Dialect::Iterm2Buffer, &bytes)?;

    let arm = Arm {
        title: "iTerm2",
        version: osascript(r#"tell application "iTerm2" to get version"#)
            .unwrap_or_else(|_| "unknown".into()),
        mechanism: "`GetBufferRequest` with `include_styles`, over the Python API's unix socket",
        measures: "iTerm2 — **what iTerm2 stores**, read as its own cells rather than as an escape \
                   stream it re-serialised. The second capture surface here of that kind and the \
                   first that is a *projection* of the cell rather than the cell itself",
        answers_in_band: AnswersInBand {
            who: "iTerm2",
            why: "iTerm2 is an endpoint, so nothing sits between the scene's tty and it. The two \
                  channels are further apart here than on any arm but Terminal.app's: the buffer \
                  is a message iTerm2 composed about its own memory, and an in-band reply is the \
                  terminal answering a question on the wire",
        },
        not_compared: NOT_COMPARED,
        // The whole point of choosing this surface over AppleScript: the capture carries style.
        no_style: None,
        notes: vec![
            "**The capture surface was chosen, and the loser is worth naming.** `iTerm2.sdef` \
                 offers `contents` and `text` on the `session` class as `type=\"text\"`, with no \
                 styled variant anywhere on it — Terminal.app's surface exactly, and it would have \
                 made all eleven rows of scene 01 `cannot ask`. The Python API's \
                 `GetBufferRequest` takes an `include_styles` flag and answers with one `CellStyle` \
                 per run of cells, so **eight of the eleven are answerable**. This arm speaks the \
             API and uses AppleScript for the window's life, its geometry and the cookie"
                .to_string(),
            format!(
                "**Geometry: set, and then insisted on.** `columns` and `rows` are read-write on \
                 the `session` class, so this arm assigns {} and reads it back, and a session that \
                 came back some other size fails the run. The second *emulator* arm that can — \
                 Terminal.app is the other, and this one sets the size **after** the command starts \
                 rather than before, because `create window … command` execs the scene directly \
                 where `do script` types into a login shell. The quiescence handshake is what \
                 observes the reflow",
                format_args!("{}x{}", REQUESTED.0, REQUESTED.1)
            ),
            format!(
                "**Environment: built, not inherited — and `COLORTERM` is deliberately left \
                 alone.** The command line unsets the {} variables a *different* terminal or a \
                 previous run could have left behind. It does **not** unset `COLORTERM`, which the \
                 Terminal.app arm does: there the value comes from a login shell's profile, and \
                 here it comes from iTerm2 itself — the string and the value `truecolor` are both \
                 in the shipped binary, `launchctl getenv COLORTERM` is empty, and no shell runs \
                 between the terminal and the scene. Unsetting a terminal's own true statement \
                 about itself would be the lie the other arm avoids by leaving `TERM_PROGRAM` alone",
                NEUTRALISED.len()
            ),
            match which {
                "05" | "06" => format!(
                    "**Launch to answer:** {} ms — reported, never gated. **This scene is not \
                     captured over the API:** the terminal answers in band on the scene's own tty, \
                     so the websocket and the protobuf are out of its path entirely, and what the \
                     arm did was open a window, size it and start the scene. The automation grant \
                     is still in the path, three Apple Events before the scene runs",
                    launched.elapsed().as_millis()
                ),
                _ => format!(
                    "**Launch to capture:** {} ms, of which the API round trip alone was {} ms — \
                     reported, never gated. The round trip is a cookie request over AppleScript, a \
                     websocket handshake on a unix socket, one frame out and one frame back",
                    launched.elapsed().as_millis(),
                    capture_elapsed.as_millis()
                ),
            },
            format!(
                "**Surface, as the scene reported it: {size}**, against a session this arm set to \
                 {geometry}. The two are printed side by side because they are different \
                 measurements — one is what the API says the session is, the other is what \
                 `TIOCGWINSZ` told the process inside it, and an arm that printed one of them twice \
                 would not notice them coming apart"
            ),
            "**A never-written cell is not reported at all**, which is the kitty arm's finding \
             reaching a third surface. A row's `code_points_per_cell` covers the cells that were \
             written and stops: `bold` on an eighty-column screen comes back as four cells where \
             the engine's own alt-screen paint comes back as eighty, because the engine paints \
             every cell and a raw-byte probe writes six short rows. Nothing in these scenes turns \
             on it — scene 04 compares with `trim_end` — and nothing here pads a short row, because \
             the scene's declared row count is the refusal"
                .to_string(),
            "**The alt screen is what the buffer reads**, so scene 01's capture is the engine's \
             page and not the shell's — and there is no shell here at all, because \
             `create window … command` execs the scene. Scene 04 writes raw bytes to the primary \
             screen with no alt-screen switch, and erases each of its rows before writing it"
                .to_string(),
        ],
    };
    Ok((arm, captured, bytes, size))
}

// ── The capture: a websocket over a unix socket, and one protobuf message each way ───────────────

/// Ask iTerm2 for the session's screen, styles included, and hand back the reply's bytes.
///
/// **The bytes and not a decoded screen.** A fixture is evidence, and what is committed here is
/// exactly what iTerm2 sent — the decoding lives in the library, where `cargo test` gates it over
/// that fixture the way it gates every other arm's capture.
fn capture(session: &str) -> Result<Vec<u8>, String> {
    let (cookie, key) = cookie()?;
    let mut socket = connect(&cookie, &key)?;
    write_frame(&mut socket, &get_buffer_request(session))?;
    let bytes = read_frame(&mut socket)?;
    if bytes.is_empty() {
        return Err(
            "the API answered with a frame of no bytes at all — an empty capture compares equal \
             against a blank region and must never reach the comparison"
                .into(),
        );
    }
    Ok(bytes)
}

/// A fresh cookie and key, minted per run.
///
/// **Never cached**, which is refusal six: a stale pair fails the handshake with a 401 that reads
/// exactly like the API being switched off, and those are two different things for an operator to
/// go and do.
fn cookie() -> Result<(String, String), String> {
    let out = osascript(&format!(
        r#"tell application "iTerm2" to request cookie and key for app named "{ADVISORY}""#
    ))?;
    let mut parts = out.split_whitespace();
    match (parts.next(), parts.next()) {
        (Some(cookie), Some(key)) => Ok((cookie.to_string(), key.to_string())),
        _ => Err(format!(
            "`request cookie and key` answered {out:?} rather than a cookie and a key — the \
             Python API is switched off unless *Preferences → General → Magic → Enable Python API* \
             is on, and this is what that looks like from here"
        )),
    }
}

/// The API's unix socket, as the client library locates it.
fn socket_path() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|e| format!("HOME: {e}"))?;
    Ok(PathBuf::from(home).join("Library/Application Support/iTerm2/private/socket"))
}

/// Open the socket and complete an RFC 6455 handshake on it.
///
/// The headers are the client library's, name for name, because the server checks three of them:
/// the cookie and key are the authorisation, and `x-iterm2-disable-auth-ui` is what makes a refusal
/// arrive as a status code rather than as a dialog nobody is there to click.
fn connect(cookie: &str, key: &str) -> Result<UnixStream, String> {
    let path = socket_path()?;
    let mut socket = UnixStream::connect(&path).map_err(|e| {
        format!(
            "connecting to {}: {e} — the API's socket exists only while iTerm2 is running with the \
             Python API enabled",
            path.display()
        )
    })?;
    socket
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| format!("setting a read timeout: {e}"))?;

    // **A constant nonce, and it is a decision.** RFC 6455 wants sixteen random bytes so that a
    // *proxy* cannot replay a cached response; there is no proxy on a unix socket in this user's
    // home directory, and this client does not verify the accept it would be keyed to. See the
    // module docs.
    let request = format!(
        "GET / HTTP/1.1\r\n\
         Host: localhost\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
         Sec-WebSocket-Version: 13\r\n\
         Sec-WebSocket-Protocol: api.iterm2.com\r\n\
         Origin: ws://localhost/\r\n\
         x-iterm2-library-version: rust {}\r\n\
         x-iterm2-disable-auth-ui: true\r\n\
         x-iterm2-advisory-name: {ADVISORY}\r\n\
         x-iterm2-cookie: {cookie}\r\n\
         x-iterm2-key: {key}\r\n\
         \r\n",
        env!("CARGO_PKG_VERSION")
    );
    socket
        .write_all(request.as_bytes())
        .map_err(|e| format!("writing the handshake: {e}"))?;

    let head = read_head(&mut socket)?;
    let status = head.lines().next().unwrap_or_default();
    if !status.contains(" 101") {
        return Err(format!(
            "the handshake was answered {status:?} rather than 101 — a 401 here is the cookie \
             being refused and a connection reset is the API being switched off"
        ));
    }
    if !head.to_lowercase().contains("api.iterm2.com") {
        return Err(format!(
            "the handshake negotiated no `api.iterm2.com` subprotocol: {head:?}"
        ));
    }
    Ok(socket)
}

/// Read the handshake response's head, one byte at a time up to the blank line.
///
/// **One byte at a time, and not a buffered read**, because the bytes after the blank line are the
/// first websocket frame: a reader that filled a buffer would swallow the answer it is about to go
/// and ask for.
fn read_head(socket: &mut UnixStream) -> Result<String, String> {
    let mut head = Vec::new();
    let mut byte = [0u8; 1];
    while !head.ends_with(b"\r\n\r\n") {
        match socket.read(&mut byte) {
            Ok(0) => {
                return Err(
                    "the socket closed during the handshake — iTerm2 refuses an unauthorised \
                     connection this way when the auth UI is disabled"
                        .into(),
                );
            }
            Ok(_) => head.push(byte[0]),
            Err(e) => return Err(format!("reading the handshake: {e}")),
        }
        if head.len() > 8192 {
            return Err("the handshake response has no blank line in eight kilobytes".into());
        }
    }
    Ok(String::from_utf8_lossy(&head).to_string())
}

/// One masked binary frame out.
fn write_frame(socket: &mut UnixStream, payload: &[u8]) -> Result<(), String> {
    let mut frame = vec![0x82u8];
    let len = payload.len();
    match len {
        0..=125 => frame.push(0x80 | len as u8),
        126..=65535 => {
            frame.push(0x80 | 126);
            frame.extend_from_slice(&(len as u16).to_be_bytes());
        }
        _ => {
            frame.push(0x80 | 127);
            frame.extend_from_slice(&(len as u64).to_be_bytes());
        }
    }
    let mask = frame_mask();
    frame.extend_from_slice(&mask);
    frame.extend(payload.iter().enumerate().map(|(i, b)| b ^ mask[i % 4]));
    socket
        .write_all(&frame)
        .map_err(|e| format!("writing the request frame: {e}"))
}

/// Frames in until a whole binary message, which is the answer.
///
/// **Fragmentation is reassembled rather than ignored, and that is not hypothetical here.** A styled
/// 80×24 screen with `include_styles` is tens of kilobytes, and RFC 6455 lets a server split a
/// message across a `0x2` frame with FIN clear and any number of `0x0` continuations. A reader that
/// took the first frame as the whole message would hand `buffer::parse` a truncated
/// length-delimited field — which it refuses, so nothing would be *wrong*, but the refusal would
/// blame the protobuf for a framing bug one layer down. iTerm2 has not been observed to fragment;
/// the cost of being ready is this loop's accumulator.
///
/// A ping is answered and a close is a refusal. Neither has been observed either — the client
/// library turns keepalives off and so does this.
fn read_frame(socket: &mut UnixStream) -> Result<Vec<u8>, String> {
    let mut message: Option<Vec<u8>> = None;
    for _ in 0..64 {
        let (fin, opcode, payload) = read_one_frame(socket)?;
        match opcode {
            // A new data frame. Text is not something this protocol sends, and it is accumulated
            // the same way rather than skipped: a reader that dropped it would go on to read a
            // *continuation* of it as though it continued something else.
            0x1 | 0x2 => message = Some(payload),
            0x0 => match message.as_mut() {
                Some(so_far) => so_far.extend_from_slice(&payload),
                None => {
                    return Err(
                        "a continuation frame arrived with nothing to continue — this connection \
                         is not carrying the messages it says it is"
                            .into(),
                    );
                }
            },
            0x9 => {
                // **The ping's payload is echoed**, which RFC 6455 §5.5.3 requires: a server that
                // validates the echo would close the connection on an empty pong, and the failure
                // would arrive here as a close with no explanation.
                write_control(socket, 0x8a, &payload)?;
                continue;
            }
            // A pong. Nothing here sends a ping, so this is a server keepalive and is ignored.
            0xa => continue,
            0x8 => return Err("the API closed the connection before answering".into()),
            n => return Err(format!("opcode {n:#x} is not one RFC 6455 defines")),
        }
        if fin {
            return Ok(message.take().unwrap_or_default());
        }
    }
    Err("sixty-four frames arrived and none of them completed a message".into())
}

/// One masked control frame, payload and all.
fn write_control(socket: &mut UnixStream, opcode: u8, payload: &[u8]) -> Result<(), String> {
    // A control frame's payload is at most 125 bytes by RFC 6455 §5.5, so there is no extended
    // length to write and a longer one is the peer misbehaving rather than something to forward.
    if payload.len() > 125 {
        return Err(format!(
            "a control frame carries {} bytes, where §5.5 allows 125",
            payload.len()
        ));
    }
    let mask = frame_mask();
    let mut frame = vec![opcode, 0x80 | payload.len() as u8];
    frame.extend_from_slice(&mask);
    frame.extend(payload.iter().enumerate().map(|(i, b)| b ^ mask[i % 4]));
    socket
        .write_all(&frame)
        .map_err(|e| format!("writing a control frame: {e}"))
}

/// The four mask bytes a client frame must carry.
///
/// A framing requirement rather than a secret — RFC 6455 §5.3 exists to stop a *cache* being
/// poisoned by a frame that looks like a request, which a unix socket has no room for. It still has
/// to vary, or a middlebox is entitled to treat two frames as one.
fn frame_mask() -> [u8; 4] {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or_default()
        .to_be_bytes()
}

/// One frame, whole, with its payload unmasked if the server masked it.
///
/// Returns the FIN bit beside the opcode, because a data frame without it is half a message and the
/// caller is the only thing that can hold the other half.
fn read_one_frame(socket: &mut UnixStream) -> Result<(bool, u8, Vec<u8>), String> {
    let mut two = [0u8; 2];
    read_exact(socket, &mut two)?;
    let fin = two[0] & 0x80 != 0;
    let opcode = two[0] & 0x0f;
    let masked = two[1] & 0x80 != 0;
    let len = match two[1] & 0x7f {
        126 => {
            let mut n = [0u8; 2];
            read_exact(socket, &mut n)?;
            u64::from(u16::from_be_bytes(n))
        }
        127 => {
            let mut n = [0u8; 8];
            read_exact(socket, &mut n)?;
            u64::from_be_bytes(n)
        }
        n => u64::from(n),
    };
    // A screen with styles is tens of kilobytes; anything of this order is a framing error being
    // read as a length, and allocating it would be the last thing this process did.
    if len > 64 * 1024 * 1024 {
        return Err(format!("a frame claims {len} bytes, which is not a screen"));
    }
    let mut mask = [0u8; 4];
    if masked {
        read_exact(socket, &mut mask)?;
    }
    let mut payload = vec![0u8; len as usize];
    read_exact(socket, &mut payload)?;
    if masked {
        for (i, b) in payload.iter_mut().enumerate() {
            *b ^= mask[i % 4];
        }
    }
    Ok((fin, opcode, payload))
}

/// Fill the buffer, refusing a short read rather than reporting a truncated frame as a frame.
fn read_exact(socket: &mut UnixStream, into: &mut [u8]) -> Result<(), String> {
    socket.read_exact(into).map_err(|e| {
        format!(
            "reading {} bytes of a frame: {e} — a truncated frame must be a refusal and never a \
             short screen",
            into.len()
        )
    })
}

/// The one request this arm sends, encoded by hand.
///
/// ```text
/// ClientOriginatedMessage { id: 1, get_buffer_request: GetBufferRequest {
///     session, line_range: LineRange { screen_contents_only: true }, include_styles: true } }
/// ```
///
/// **`screen_contents_only` and not a coordinate range**, because the scenes are about what is on
/// the screen: a range would put this arm in the business of deciding which rows count, which is
/// the scene's job and `SCENES.md`'s.
fn get_buffer_request(session: &str) -> Vec<u8> {
    let line_range = tagged_varint(1, 1);
    let mut request = Vec::new();
    request.extend(tagged_bytes(1, session.as_bytes()));
    request.extend(tagged_bytes(2, &line_range));
    request.extend(tagged_varint(3, 1));

    let mut message = tagged_varint(1, 1);
    message.extend(tagged_bytes(100, &request));
    message
}

/// A base-128 varint field.
fn tagged_varint(number: u32, value: u64) -> Vec<u8> {
    let mut out = varint(u64::from(number) << 3);
    out.extend(varint(value));
    out
}

/// A length-delimited field.
fn tagged_bytes(number: u32, value: &[u8]) -> Vec<u8> {
    let mut out = varint((u64::from(number) << 3) | 2);
    out.extend(varint(value.len() as u64));
    out.extend_from_slice(value);
    out
}

fn varint(mut value: u64) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(byte);
            return out;
        }
        out.push(byte | 0x80);
    }
}

// ── AppleScript, for the three things the API is not needed for ──────────────────────────────────

/// The id of every iTerm2 window, as text.
///
/// A `text item delimiters` of its own, because AppleScript coerces a list of integers to text by
/// running them together: two windows numbered 8559 and 8561 arrive as `85598561`, which is one
/// address that exists nowhere. The Terminal.app arm's finding, and it is the same coercion.
fn window_ids() -> Result<Vec<String>, String> {
    let out = osascript(
        r#"set text item delimiters to ","
           tell application "iTerm2" to return (id of every window) as text"#,
    )?;
    Ok(out
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect())
}

/// A string as an AppleScript double-quoted literal's contents.
///
/// The command line is built from `current_exe` and a temporary directory, so nothing in it needs
/// escaping today. Escaping it anyway is what keeps that true when one of those two moves.
fn applescript_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// One AppleScript, as trimmed text.
fn osascript(script: &str) -> Result<String, String> {
    let out = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| format!("osascript: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "osascript failed: {} — this arm needs a window server and a macOS automation grant \
             for iTerm2, neither of which can be obtained non-interactively",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}
