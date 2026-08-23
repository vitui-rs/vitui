//! The live arm: drive the engine inside a real Ghostty window and read its screen back.
//!
//! **One executable, two halves.** With no arguments this is the *driver*; with `--scene 01` it is
//! the *scene*, and the driver launches the scene by re-running its own [`std::env::current_exe`].
//! That is not a trick to save a file: it makes the two halves the same build by construction, where
//! a sibling binary path can silently be yesterday's.
//!
//! ```sh
//! cargo run --example ghostty          # writes REPORT.md, opens and closes one window
//! ```
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
//! default-styled blanks, which the `vt` dump trims. The size is recorded in the report rather than
//! demanded.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime};

use vitui_conform::{Attrs, Colour, Dump, Style, Underline, default_colours, parse};
use vitui_engine::{Clock, Config, Engine, Rect, Style as EngineStyle, Wake};

/// How long to wait for the scene to present a frame and then stand still.
const READY_TIMEOUT: Duration = Duration::from_secs(20);

/// How long the scene's stamp must stay unchanged before the screen counts as settled.
///
/// **Not tuned to a measurement**, which would make it the flaky-test shape this repository refuses.
/// It is a floor: a window-server resize storm is tens of milliseconds, and this is an order of
/// magnitude above it. If it ever needs raising, the run says so — it fails with the reason rather
/// than photographing a moving screen.
const QUIESCENT: Duration = Duration::from_millis(500);

/// One row of scene 01: an attribute, the label under it, and what the dump must say.
struct Case {
    /// The label written into the row. Also the row's identity in the report.
    label: &'static str,
    /// The bit this row lights, named the way the engine's public surface names it.
    verb: &'static str,
    /// The engine style the scene draws with.
    draw: fn(EngineStyle) -> EngineStyle,
    /// The flags the dump must report, **exactly** — an extra one is a disagreement too.
    attrs: Attrs,
    /// The underline the dump must report.
    underline: Underline,
}

/// Scene 01, one row per attribute bit of the engine's style word.
///
/// **Eleven rows because the style word spends eleven bits on attributes** — eight flags and a
/// three-bit underline field — and each row here lights exactly one of them. That is what makes the
/// assertion eleven independent booleans rather than eleven overlapping ones, and it is why the
/// three underline rows are single, double and dotted: `4:1`, `4:2` and `4:4` are the three values
/// with one bit set. Curly (`4:3`) and dashed (`4:5`) light two bits each and so cannot be a
/// per-bit row; they are covered by the committed-fixture tests instead.
///
/// `attrs_dropped` is the field this feeds, and it is the field with **no query**: the eleven facts
/// are in the capability set precisely because nothing can ask a terminal for them. A dump is the
/// only thing that can, which is why this is scene 01 and not scene 07.
fn scene01() -> [Case; 11] {
    [
        Case {
            label: "bold",
            verb: "Style::bold",
            draw: EngineStyle::bold,
            attrs: Attrs::BOLD,
            underline: Underline::None,
        },
        Case {
            label: "dim",
            verb: "Style::dim",
            draw: EngineStyle::dim,
            attrs: Attrs::DIM,
            underline: Underline::None,
        },
        Case {
            label: "italic",
            verb: "Style::italic",
            draw: EngineStyle::italic,
            attrs: Attrs::ITALIC,
            underline: Underline::None,
        },
        Case {
            label: "reverse",
            verb: "Style::reverse",
            draw: EngineStyle::reverse,
            attrs: Attrs::REVERSE,
            underline: Underline::None,
        },
        Case {
            label: "blink",
            verb: "Style::blink",
            draw: EngineStyle::blink,
            attrs: Attrs::BLINK,
            underline: Underline::None,
        },
        Case {
            label: "strikethru",
            verb: "Style::strikethrough",
            draw: EngineStyle::strikethrough,
            attrs: Attrs::STRIKE,
            underline: Underline::None,
        },
        Case {
            label: "conceal",
            verb: "Style::conceal",
            draw: EngineStyle::conceal,
            attrs: Attrs::HIDDEN,
            underline: Underline::None,
        },
        Case {
            label: "overline",
            verb: "Style::overline",
            draw: EngineStyle::overline,
            attrs: Attrs::OVERLINE,
            underline: Underline::None,
        },
        Case {
            label: "under-sgl",
            verb: "Style::underline",
            draw: EngineStyle::underline,
            attrs: Attrs::default(),
            underline: Underline::Single,
        },
        Case {
            label: "under-dbl",
            verb: "Style::underline_double",
            draw: EngineStyle::underline_double,
            attrs: Attrs::default(),
            underline: Underline::Double,
        },
        Case {
            label: "under-dot",
            verb: "Style::underline_dotted",
            draw: EngineStyle::underline_dotted,
            attrs: Attrs::default(),
            underline: Underline::Dotted,
        },
    ]
}

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

// ── The scene half: inside the Ghostty window ────────────────────────────────────────────────────

/// Draw the scene, say so, and keep it right until the driver closes the window.
///
/// **It redraws rather than drawing once**, and that is not defensive coding — it is the answer to a
/// defect the first live run produced. A Ghostty window settles its size *after* the process inside
/// it starts, so the first frame can be painted at one geometry and then reflowed at another; that
/// run photographed a screen whose top two rows had scrolled off, and every row of the report was
/// wrong by two. The readiness handshake proves a frame was presented. It cannot prove the window
/// has stopped moving, so the stamp below carries a frame counter and the driver waits for it to
/// stand still.
fn scene(which: Option<&str>) {
    assert_eq!(which, Some("01"), "only scene 01 exists");
    let ready = std::env::var("CONFORM_READY").expect("the driver sets CONFORM_READY");

    let engine = Engine::new(Config {
        clock: Clock::System,
        max_frame_rate: 60.0,
        // The terminal this writes to is the one being photographed, so a warning printed here would
        // become content in the capture.
        overrun_report: Some(Box::new(std::io::sink())),
        ..Default::default()
    });
    let Ok((mut screen, _wake)) = engine.attach() else {
        // Nothing can be drawn and nothing can be said on this terminal. The driver's readiness
        // timeout is what turns this into a `FAILED` row.
        return;
    };

    let mut frame = 0u64;
    {
        // Same declaration the app template makes, and for the same reason: a debug build panics on
        // the first unexcused overrun, and the cold-start frame is legitimately slower than a budget.
        let permit = screen.permit_slow("the cold-start frame");
        draw(&mut screen);
        drop(permit);
    }
    stamp(&ready, &mut frame, &screen);

    loop {
        if screen.wait() == Wake::Quit {
            return;
        }
        // Drained and ignored. The window steals focus when it opens, so whatever the user types
        // lands here; a scene that echoed it would photograph the typing. A resize arrives through
        // the same drain and is answered by the redraw below, which is why nothing inspects the
        // event kind — every wake redraws, and a redraw is what makes the stamp stand still.
        while screen.next_event().is_some() {}
        draw(&mut screen);
        stamp(&ready, &mut frame, &screen);
    }
}

/// One frame of scene 01: eleven rows at the top left of whatever surface there is.
fn draw(screen: &mut vitui_engine::Screen) {
    let (w, h) = screen.size();
    let layer = match screen.layers().is_empty() {
        true => screen.layers().add_content(0, Rect::new(0, 0, w, h), true),
        false => screen
            .layers()
            .topmost_at(0, 0)
            .expect("the one layer covers the screen"),
    };
    if let Some(mut view) = screen.layers().view(layer) {
        for (row, case) in scene01().iter().enumerate() {
            // Column 0 deliberately: the dump has no column grid, so a left margin would only add
            // leading spaces the comparison then has to strip.
            view.text(0, row as i32, case.label, (case.draw)(EngineStyle::new()));
        }
    }
    screen.present();
}

/// Tell the driver what has been presented, and how many times.
///
/// **After the frame is on the wire, never before.** The counter is what lets the driver distinguish
/// *a frame exists* from *the window has stopped changing shape*.
fn stamp(ready: &str, frame: &mut u64, screen: &vitui_engine::Screen) {
    *frame += 1;
    let (w, h) = screen.size();
    let _ = std::fs::write(ready, format!("{frame} {w}x{h}\n"));
}

// ── The driver half ──────────────────────────────────────────────────────────────────────────────

fn drive() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let ready = std::env::temp_dir().join(format!("conform-ready-{}", std::process::id()));
    let _ = std::fs::remove_file(&ready);

    let was = terminal_ids()?;
    let launched = Instant::now();
    osascript(&format!(
        r#"tell application "Ghostty"
             set cfg to new surface configuration from {{command:"{} --scene 01", environment variables:{{"CONFORM_READY={}"}}}}
             new window with configuration cfg
           end tell"#,
        exe.display(),
        ready.display()
    ))?;

    // The set difference is the address. Ticket 04 established that window *indices* are z-order and
    // shift between `osascript` calls, and that `focused terminal` raises -1728 on a fresh window, so
    // neither is usable; the terminal id is stable for the surface's life.
    let terminal = new_terminal(&was)?;

    let outcome = capture_and_compare(&terminal, &ready, launched);

    // Closed before the result is unwrapped. A failed run that leaves a window open is a run that
    // needs a human before the next one can start.
    let _ = osascript(&format!(
        r#"tell application "Ghostty" to close terminal id "{terminal}""#
    ));
    let _ = std::fs::remove_file(&ready);

    let (report, failures) = outcome?;
    std::fs::write("REPORT.md", &report).map_err(|e| format!("writing REPORT.md: {e}"))?;
    println!("{report}");
    match failures {
        0 => {
            println!("REPORT.md written. 11/11 agreed.");
            Ok(())
        }
        // A non-zero exit so a human running this notices, and **not** a CI gate: nothing runs this
        // on a pull request. See the module docs.
        n => Err(format!("{n} of 11 disagreed — REPORT.md written")),
    }
}

/// Wait for the frame, photograph it, parse it, compare it, and render the report.
fn capture_and_compare(
    terminal: &str,
    ready: &Path,
    launched: Instant,
) -> Result<(String, usize), String> {
    let size = wait_for_quiescence(ready)?;
    let bytes = capture(terminal)?;
    let dump =
        parse(&bytes, scene01().len()).map_err(|e| format!("the capture is not a screen: {e}"))?;
    Ok(render(&dump, &bytes, &size, launched.elapsed()))
}

/// Block until the scene has presented *and stopped changing*, or give up loudly.
///
/// **This replaces the fixed delay** — the mechanism by which a capture races the paint, and a raced
/// capture is exactly the empty one ticket 04 predicted would read as agreement. It waits for two
/// distinct things, because the first live run proved one was not enough:
///
/// 1. a stamp exists at all, so a frame has been presented;
/// 2. the stamp has stood still for [`QUIESCENT`], so the window has finished settling its size.
///
/// Quiescence is an *observed* condition and not a sleep: the scene redraws and re-stamps on every
/// wake, and an idle vitui application costs zero wakeups, so a stamp that stops moving is a screen
/// that has stopped moving.
///
/// Returns the surface size the last stamp reported.
fn wait_for_quiescence(ready: &Path) -> Result<String, String> {
    let deadline = Instant::now() + READY_TIMEOUT;
    let mut last: Option<(String, Instant)> = None;
    while Instant::now() < deadline {
        let now = std::fs::read_to_string(ready)
            .unwrap_or_default()
            .trim()
            .to_string();
        if !now.is_empty() {
            match &last {
                Some((seen, since)) if *seen == now => {
                    if since.elapsed() >= QUIESCENT {
                        // "<frame> <w>x<h>" — the report wants the size, not the counter.
                        return Ok(now
                            .split_whitespace()
                            .nth(1)
                            .unwrap_or("unknown")
                            .to_string());
                    }
                }
                _ => last = Some((now, Instant::now())),
            }
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    Err(match last {
        None => format!(
            "the scene never reported a presented frame within {READY_TIMEOUT:?} — it may have \
             failed to attach, or Ghostty may have refused the command"
        ),
        Some(_) => format!(
            "the scene presented but never stood still for {QUIESCENT:?} within {READY_TIMEOUT:?} — \
             something is redrawing it, and a capture of a moving screen is not evidence"
        ),
    })
}

/// Ask Ghostty to write its screen out, and find the file it wrote.
///
/// `paste` and not `copy`: `copy` clobbers the user's system clipboard, and the path it pastes lands
/// in the scene's input, which the scene drains and ignores. The `plain|html|vt` argument grammar is
/// undocumented; `vt` is the only one of the three that keeps resolved colours — `html` renders
/// reverse as `filter: invert(100%)` and throws the colours away.
fn capture(terminal: &str) -> Result<Vec<u8>, String> {
    // Ghostty writes `screen.txt` into a **fresh random directory** under the user's temp directory,
    // and nothing reports the path back to us. So: mark the time, act, and take what appeared —
    // with the count checked, because "the newest one" silently picks a stranger's file.
    let mark = SystemTime::now();
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
            // **Opt-in, and never automatic.** `fixtures/` is evidence: a test that fails against it
            // is a parser defect or a claim that stopped being true, never a reason to regenerate.
            // A driver that rewrote its own fixtures on every run would turn the gate into a mirror.
            if let Ok(to) = std::env::var("CONFORM_SAVE_CAPTURE") {
                std::fs::write(&to, &bytes).map_err(|e| format!("saving the capture: {e}"))?;
                eprintln!("capture saved to {to}");
            }
            Ok(bytes)
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

// ── The report ───────────────────────────────────────────────────────────────────────────────────

/// What the dump said about one row of the scene.
enum Verdict {
    /// The label is there, wearing exactly the attribute the scene drew it with, and the rest of the
    /// row is unstyled. Carries what was seen rather than a tick, so an agreeing row still prints
    /// the evidence for its own agreement.
    Agreed(Style),
    /// The row is not in the capture at all.
    Missing,
    /// The label did not survive. Printed rather than summarised, because a wrong label usually
    /// means the whole screen is offset and every other row is about to lie in the same way.
    WrongText(String),
    /// The label survived and the style did not.
    WrongStyle(Style),
    /// The attribute did not stop where the label did.
    Leaked(usize),
}

impl Verdict {
    fn observed(&self) -> String {
        match self {
            Self::Agreed(style) => describe(*style),
            Self::Missing => "no such row".into(),
            Self::WrongText(text) => format!("text was {text:?}"),
            Self::WrongStyle(style) => describe(*style),
            Self::Leaked(n) => format!(
                "the attribute continued past the label, over {n} of the row's padding cells"
            ),
        }
    }
}

/// The style the scene drew row `case` with, in the dump's own vocabulary.
fn expected(case: &Case) -> Style {
    Style {
        attrs: case.attrs,
        underline: case.underline,
        ..Style::default()
    }
}

/// Judge one row, checking three things rather than one.
///
/// **The padding check is not padding.** The engine paints the whole surface, so a row is the label
/// followed by spaces, and the label's style must stop where the label does. If it did not, the
/// terminal would be showing an underline or a reverse block running to column 156 — a real defect,
/// and one that a comparison of the first cluster alone would report as a clean pass.
fn judge(dump: &Dump, i: usize, case: &Case) -> Verdict {
    let Some(row) = dump.rows.get(i) else {
        return Verdict::Missing;
    };
    // The engine paints the surface, so the trailing cells are real spaces the dump keeps — unlike
    // the never-written cells a `printf` leaves, which it trims. Trimming here is reading the scene,
    // not being lenient: the scene wrote a label, and the padding is the surface underneath it.
    let text = row.text();
    if text.trim_end() != case.label {
        return Verdict::WrongText(text.trim_end().to_string());
    }
    let want = expected(case);
    let label_len = case.label.chars().count();
    if let Some(wrong) = row
        .clusters
        .iter()
        .take(label_len)
        .find(|c| c.style != want)
    {
        return Verdict::WrongStyle(wrong.style);
    }
    let leaked = row
        .clusters
        .iter()
        .skip(label_len)
        .filter(|c| c.style != Style::default())
        .count();
    match leaked {
        0 => Verdict::Agreed(want),
        n => Verdict::Leaked(n),
    }
}

/// Compare the eleven rows and render `REPORT.md`. Returns the text and the number of disagreements.
fn render(dump: &Dump, bytes: &[u8], size: &str, elapsed: Duration) -> (String, usize) {
    let cases = scene01();
    let (fg, bg) = default_colours(bytes);
    let version = osascript(r#"tell application "Ghostty" to get version"#)
        .unwrap_or_else(|_| "unknown".into());

    let mut out = String::new();
    let _ = writeln!(out, "# The conformance suite — Ghostty\n");
    let _ = writeln!(
        out,
        "**Generated by `cargo run --example ghostty`. It reports; it does not block.**\n"
    );
    let _ = writeln!(
        out,
        "An emulator's version, a window server and a macOS automation grant cannot gate this\n\
         repository's pull requests, and the `vt` dump format this reads is undocumented. So this\n\
         file is committed and regenerated, and a disagreement arrives as a review-visible diff\n\
         rather than as a red build. The gate is `cargo test` over `fixtures/`.\n"
    );
    let _ = writeln!(out, "Read [`SCENES.md`](SCENES.md) first.\n");
    let _ = writeln!(
        out,
        "- **Arm:** Ghostty {version}, `write_screen_file:…,vt`"
    );
    let _ = writeln!(out, "- **Surface:** {size} cells, as the window was given");
    let _ = writeln!(
        out,
        "- **Default colours, from the dump's own OSC 10/11:** fg {}, bg {}",
        show(fg),
        show(bg)
    );
    let _ = writeln!(
        out,
        "- **Launch to capture:** {} ms — reported, never gated; it is AppleScript round trips and \
         a window opening\n",
        elapsed.as_millis()
    );

    let _ = writeln!(
        out,
        "## Scene 01 — the eleven attribute bits, one per row\n"
    );
    let _ = writeln!(
        out,
        "Each row lights exactly one of the style word's eleven attribute bits. A row agrees only \
         when the dump reports that attribute **and nothing else** — an invented attribute is a \
         disagreement in the same way a missing one is.\n"
    );
    let _ = writeln!(out, "| bit | verb | expected | observed | |");
    let _ = writeln!(out, "|---|---|---|---|---|");

    let mut failures = 0;
    for (i, case) in cases.iter().enumerate() {
        let verdict = judge(dump, i, case);
        if !matches!(verdict, Verdict::Agreed(_)) {
            failures += 1;
        }
        let _ = writeln!(
            out,
            "| {} | `{}` | {} | {} | {} |",
            case.label,
            case.verb,
            describe(expected(case)),
            verdict.observed(),
            match verdict {
                Verdict::Agreed(_) => "\u{2713}",
                _ => "**FAILED**",
            }
        );
    }

    let _ = writeln!(
        out,
        "\n**{}/{} agreed.**\n",
        cases.len() - failures,
        cases.len()
    );
    let _ = writeln!(
        out,
        "A row that does not say which arm it came from is not a result, and a *missing* row reads \
         as a win — which is the single easiest way for this directory to become dishonest. Every \
         row of the scene is printed above whether it agreed or not, and a capture with fewer rows \
         than the scene declared never reaches this table: it is refused as `FAILED` by the parser."
    );
    (out, failures)
}

fn describe(s: Style) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for (attr, name) in [
        (Attrs::BOLD, "bold"),
        (Attrs::DIM, "dim"),
        (Attrs::ITALIC, "italic"),
        (Attrs::BLINK, "blink"),
        (Attrs::REVERSE, "reverse"),
        (Attrs::HIDDEN, "conceal"),
        (Attrs::STRIKE, "strike"),
        (Attrs::OVERLINE, "overline"),
    ] {
        if s.attrs.has(attr) {
            parts.push(name);
        }
    }
    let underline = match s.underline {
        Underline::None => None,
        Underline::Single => Some("underline".to_string()),
        Underline::Double => Some("underline:2".to_string()),
        Underline::Curly => Some("underline:3".to_string()),
        Underline::Dotted => Some("underline:4".to_string()),
        Underline::Dashed => Some("underline:5".to_string()),
        Underline::Other(n) => Some(format!("underline:{n}")),
    };
    let mut all: Vec<String> = parts.into_iter().map(str::to_string).collect();
    all.extend(underline);
    if s.fg != Colour::Default {
        all.push(format!("fg {:?}", s.fg));
    }
    if s.bg != Colour::Default {
        all.push(format!("bg {:?}", s.bg));
    }
    if s.underline_colour != Colour::Default {
        all.push(format!("underline colour {:?}", s.underline_colour));
    }
    match all.is_empty() {
        true => "nothing".into(),
        false => all.join(" + "),
    }
}

fn show(c: Option<Colour>) -> String {
    match c {
        Some(Colour::Rgb(r, g, b)) => format!("`#{r:02x}{g:02x}{b:02x}`"),
        Some(other) => format!("`{other:?}`"),
        // A fact about the run, in `compare/`'s vocabulary: the emulator sent no header.
        None => "not run here — no OSC 10/11 in the capture".into(),
    }
}
