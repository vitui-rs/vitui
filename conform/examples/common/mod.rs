//! The half of the live arm that is not the terminal: the scene, the handshake, and the comparison.
//!
//! **Not a target.** Cargo builds `examples/*.rs` and `examples/*/main.rs`; a directory with only a
//! `mod.rs` in it is neither, so this is a module two examples include rather than a third example.
//!
//! # Why it exists, and it is production ticket 05 that forced it
//!
//! Stage 1 had one arm and one file, and the scene lived in it. Ticket 05 asks for the eleven
//! attribute facts *on at least two families*, which makes the scene and the comparison the parts
//! that must be **identical** across arms while the launching, capturing and closing are the parts
//! that cannot be. A second copy of `scene01` would make a disagreement between the two arms
//! unattributable: it could be the emulators, or it could be the two copies having drifted.
//!
//! So the split here is not tidiness. It is the thing that lets a row say *tmux dropped this bit and
//! Ghostty did not* and mean it.
//!
//! # The arm-specific half, stated as an interface
//!
//! An arm brings four things and nothing else: a way to start [`scene_command`] somewhere a terminal
//! can see it, a way to read that terminal's screen back as bytes, a way to shut it down, and an
//! [`Arm`] describing itself for the report. Everything else — what is drawn, when it is safe to
//! photograph, and what counts as agreement — is here.

use std::fmt::Write as _;
use std::path::Path;
use std::time::{Duration, Instant};

use vitui_conform::{Attrs, Colour, Dump, Style, Underline, default_colours};
use vitui_engine::{Clock, Config, Engine, Rect, Style as EngineStyle, Wake};

/// How long to wait for the scene to present a frame and then stand still.
pub const READY_TIMEOUT: Duration = Duration::from_secs(20);

/// How long the scene's stamp must stay unchanged before the screen counts as settled.
///
/// **Not tuned to a measurement**, which would make it the flaky-test shape this repository refuses.
/// It is a floor: a window-server resize storm is tens of milliseconds, and this is an order of
/// magnitude above it. If it ever needs raising, the run says so — it fails with the reason rather
/// than photographing a moving screen.
pub const QUIESCENT: Duration = Duration::from_millis(500);

/// The arguments that turn one of these executables from its driver half into its scene half.
///
/// **One executable, two halves**, and the driver launches the scene by re-running its own
/// [`std::env::current_exe`]. That is not a trick to save a file: it makes the two halves the same
/// build by construction, where a sibling binary path can silently be yesterday's.
pub const SCENE_ARGS: &str = "--scene 01";

/// One row of scene 01: an attribute, the label under it, and what the dump must say.
pub struct Case {
    /// The label written into the row. Also the row's identity in the report.
    pub label: &'static str,
    /// The bit this row lights, named the way the engine's public surface names it.
    pub verb: &'static str,
    /// The engine style the scene draws with.
    pub draw: fn(EngineStyle) -> EngineStyle,
    /// The flags the dump must report, **exactly** — an extra one is a disagreement too.
    pub attrs: Attrs,
    /// The underline the dump must report.
    pub underline: Underline,
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
pub fn scene01() -> [Case; 11] {
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

/// The command line an arm has to get a terminal to run, as one string.
///
/// Built from [`std::env::current_exe`] so the scene is this same build. An arm that has to quote it
/// for a shell quotes what it is given; nothing here guesses at a quoting rule for a launcher it
/// cannot see.
///
/// # Errors
///
/// Whatever [`std::env::current_exe`] failed with.
pub fn scene_command() -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    Ok(format!("{} {SCENE_ARGS}", exe.display()))
}

// ── The scene half: inside the terminal ──────────────────────────────────────────────────────────

/// Draw the scene, say so, and keep it right until the arm shuts the terminal down.
///
/// **It redraws rather than drawing once**, and that is not defensive coding — it is the answer to a
/// defect the first live run produced. A Ghostty window settles its size *after* the process inside
/// it starts, so the first frame can be painted at one geometry and then reflowed at another; that
/// run photographed a screen whose top two rows had scrolled off, and every row of the report was
/// wrong by two. The readiness handshake proves a frame was presented. It cannot prove the window
/// has stopped moving, so the stamp below carries a frame counter and the driver waits for it to
/// stand still.
///
/// The tmux arm does not need that — `capture-pane` runs against a pane whose size the driver set
/// with `-x` and `-y` — and it runs the same code anyway. An arm that only *believes* it set the
/// size still gets told when it did not.
pub fn scene(which: Option<&str>) {
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
        // Drained and ignored. A window steals focus when it opens, so whatever the user types lands
        // here; a scene that echoed it would photograph the typing. A resize arrives through the same
        // drain and is answered by the redraw below, which is why nothing inspects the event kind —
        // every wake redraws, and a redraw is what makes the stamp stand still.
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
/// *a frame exists* from *the terminal has stopped changing shape*.
fn stamp(ready: &str, frame: &mut u64, screen: &vitui_engine::Screen) {
    *frame += 1;
    let (w, h) = screen.size();
    let _ = std::fs::write(ready, format!("{frame} {w}x{h}\n"));
}

/// Block until the scene has presented *and stopped changing*, or give up loudly.
///
/// **This replaces the fixed delay** — the mechanism by which a capture races the paint, and a raced
/// capture is exactly the empty one ticket 04 predicted would read as agreement. It waits for two
/// distinct things, because the first live run proved one was not enough:
///
/// 1. a stamp exists at all, so a frame has been presented;
/// 2. the stamp has stood still for [`QUIESCENT`], so the terminal has finished settling its size.
///
/// Quiescence is an *observed* condition and not a sleep: the scene redraws and re-stamps on every
/// wake, and an idle vitui application costs zero wakeups, so a stamp that stops moving is a screen
/// that has stopped moving.
///
/// Returns the surface size the last stamp reported.
///
/// # Errors
///
/// A message naming which of the two conditions was never met, because they have different causes:
/// no stamp at all is a scene that failed to attach, and a stamp that never stands still is a
/// terminal still changing shape.
pub fn wait_for_quiescence(ready: &Path) -> Result<String, String> {
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
             failed to attach, or the terminal may have refused the command"
        ),
        Some(_) => format!(
            "the scene presented but never stood still for {QUIESCENT:?} within {READY_TIMEOUT:?} — \
             something is redrawing it, and a capture of a moving screen is not evidence"
        ),
    })
}

// ── The report ───────────────────────────────────────────────────────────────────────────────────

/// What one arm has to say about itself, so a row can say where it came from.
///
/// **A row that does not name its arm is not a result**, and with two arms that stopped being a
/// slogan: `capture-pane` re-serialises *tmux's* grid, so the tmux arm's rows are about tmux and
/// never about the emulator behind it. [`Arm::measures`] is where that is written, in the report,
/// rather than only in `SCENES.md` where a reader of the numbers may not go.
pub struct Arm {
    /// The heading, and the name in the file this writes.
    pub title: &'static str,
    /// What the software under test says its version is. Asked of it, never assumed.
    pub version: String,
    /// The capture mechanism, spelled as the reader would have to type it.
    pub mechanism: &'static str,
    /// What this arm's rows are evidence *about*.
    pub measures: &'static str,
    /// The surface size the scene reported, which is not necessarily the one the arm asked for.
    pub size: String,
    /// Anything else this arm knows that the reader needs. One bullet per entry, already worded.
    pub notes: Vec<String>,
}

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
/// terminal would be showing an underline or a reverse block running to the right edge — a real
/// defect, and one that a comparison of the first cluster alone would report as a clean pass.
///
/// The two arms differ in what they hand back here and the check survives both. Ghostty's `vt` dump
/// keeps the default-styled blanks the engine painted; tmux's `capture-pane` trims them, so the
/// padding count is zero for a reason that is about the *capture format* rather than about the
/// screen. What matters is that neither trims a blank that is **styled** — a leaked reverse block
/// is not default-styled, so it survives the trim and is still counted.
fn judge(dump: &Dump, i: usize, case: &Case) -> Verdict {
    let Some(row) = dump.rows.get(i) else {
        return Verdict::Missing;
    };
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

/// Compare the eleven rows and render one arm's report. Returns the text and the disagreement count.
pub fn render(arm: &Arm, dump: &Dump, bytes: &[u8]) -> (String, usize) {
    let cases = scene01();
    let (fg, bg) = default_colours(bytes);

    let mut out = String::new();
    let _ = writeln!(out, "# The conformance suite — {}\n", arm.title);
    let _ = writeln!(
        out,
        "**Generated. It reports; it does not block.** One file per arm, because two arms writing \
         one file means the rows of whichever ran first are gone — and *a missing row reads as a \
         win*.\n"
    );
    let _ = writeln!(
        out,
        "The version of software this repository does not control cannot gate its pull requests. So \
         this file is committed and regenerated, and a disagreement arrives as a review-visible diff \
         rather than as a red build. The gate is `cargo test` over `fixtures/`.\n"
    );
    let _ = writeln!(out, "Read [`SCENES.md`](SCENES.md) first.\n");
    let _ = writeln!(
        out,
        "- **Arm:** {} {}, {}",
        arm.title, arm.version, arm.mechanism
    );
    let _ = writeln!(out, "- **These rows are evidence about:** {}", arm.measures);
    let _ = writeln!(
        out,
        "- **Surface:** {} cells, as the scene reported it",
        arm.size
    );
    let _ = writeln!(
        out,
        "- **Default colours, from the dump's own OSC 10/11:** fg {}, bg {}",
        show(fg),
        show(bg)
    );
    for note in &arm.notes {
        let _ = writeln!(out, "- {note}");
    }
    let _ = writeln!(out);

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

/// A style in words, for a table cell.
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

/// A default colour the capture may not have carried.
fn show(c: Option<Colour>) -> String {
    match c {
        Some(Colour::Rgb(r, g, b)) => format!("`#{r:02x}{g:02x}{b:02x}`"),
        Some(other) => format!("`{other:?}`"),
        // A fact about the run, in `compare/`'s vocabulary: the capture format carries no header.
        None => "`cannot express` — this capture format has no OSC 10/11 header".into(),
    }
}

/// Write one arm's report where the arm's own name puts it, and say how it went.
///
/// # Errors
///
/// The write failing, or a disagreement — which is a non-zero exit so a human running this notices,
/// and **not** a CI gate: nothing runs these on a pull request.
pub fn publish(arm: &Arm, report: &str, failures: usize) -> Result<(), String> {
    let path = format!("REPORT-{}.md", arm.title.to_lowercase());
    std::fs::write(&path, report).map_err(|e| format!("writing {path}: {e}"))?;
    println!("{report}");
    let total = scene01().len();
    match failures {
        0 => {
            println!("{path} written. {total}/{total} agreed.");
            Ok(())
        }
        n => Err(format!("{n} of {total} disagreed — {path} written")),
    }
}

/// The capture an arm produced, saved out only when the operator asked for it.
///
/// **Opt-in, and never automatic.** `fixtures/` is evidence: a test that fails against it is a
/// parser defect or a claim that stopped being true, never a reason to regenerate. A driver that
/// rewrote its own fixtures on every run would turn the gate into a mirror.
///
/// # Errors
///
/// The write failing. A capture that cannot be saved when saving was asked for is a failed run, not
/// a run with a missing side effect.
pub fn save_if_asked(bytes: &[u8]) -> Result<(), String> {
    if let Ok(to) = std::env::var("CONFORM_SAVE_CAPTURE") {
        std::fs::write(&to, bytes).map_err(|e| format!("saving the capture: {e}"))?;
        eprintln!("capture saved to {to}");
    }
    Ok(())
}
