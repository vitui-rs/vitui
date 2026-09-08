//! The vitui-**runtime** arm of the comparative suite.
//!
//! `../../SCENES.md` is normative and `../../ARM-CONTRACT.md` is what the harness relies on. This
//! file draws the same nine pictures `arms/vitui` draws, through `vitui-runtime` instead of through
//! `vitui-engine`, so that the layer between them is priced by subtraction. `Cargo.toml` carries why
//! it is a second arm rather than a changed first one.
//!
//! # Three things this arm does differently, and every one of them is the finding
//!
//! **It redraws the whole picture every frame, on all nine scenes.** That is not a shortcut and it
//! is not laziness — it is what the runtime *is*. There is no scene tree and no retained structure
//! (the rule): the clip stack is the call stack and the id path is the closure tree, so a component
//! that wants to be on the screen on frame *n* is a function that runs on frame *n*. `arms/vitui`
//! writes only the cells that changed, because the engine's three verbs mark damage as they write
//! and a caller at that level knows what it changed. Both are ordinary code at their own layer, and
//! the gap between the two columns is the price of the equality filter absorbing an immediate-mode
//! redraw. It is the number this arm exists to produce.
//!
//! **It cannot say *the terminal's default colours*, and four scenes ask for them.** A `Paint` comes
//! from a `Theme` (the rule: a component names a role and can never construct a paint), and a
//! `Theme` is thirteen concrete colour pairs. There is no role, and no [`Theme::custom`] argument,
//! that means *leave it alone* — `Theme::custom` takes two `Rgb`, and `Rgb` is three channels. So
//! this arm names the two colours the suite has already declared for exactly this problem:
//! `SCENES.md` scene 5 fixes the terminal's defaults at `rgb(192,192,192)` on `rgb(0,0,0)`, because
//! no arm can read them from inside and one arm's dim was a silent no-op until they were written
//! down. That makes the *picture* the one described. It does not make the bytes the same, and it
//! must not be read as though it did: a cell in default colours costs this arm an SGR that costs
//! `arms/vitui` nothing, on every scene, at the truecolor tier. See `NOTES.md`.
//!
//! **There is no `cannot express` cell here and there could have been one.** The paragraph above is
//! the shape of a refusal — a scene says *default colours*, the framework has no such thing — and it
//! was resolved as an encoding difference rather than a refusal because the suite had already ruled
//! on it. `SCENES.md` declared those two colours to be what the terminal's defaults *are* for
//! measurement, which is the ruling; an arm that then refused would be refusing the suite's own
//! definition and would take four rows off the table for a picture it can draw. **A missing row
//! reads as a win**, and it reads as a win in whichever direction it is missing.
//!
//! `Clock::Manual`, for `arms/vitui`'s reason and unchanged: `present` composites, packs,
//! serialises and writes inline before returning, so the bytes are all on the wire when the process
//! exits and frame *n* is a pure function of *n*.

use std::io::Write as _;
use std::time::{Duration, Instant};

use vitui::engine::{Clock, Config, CursorShape, Mix, Output, Rect, Rgb};
use vitui::runtime::ctx::{Ctx, Driver};
use vitui::runtime::overlay::{Align, OverlayOpts, Placement, Scrim, Side, Z};
use vitui::runtime::theme::{Paint, Repaint};
use vitui::runtime::{Id, Interest, Theme};

/// The 64-character lorem row, verbatim from `SCENES.md`. The same literal `arms/vitui` uses,
/// because two arms of the same library measuring two strings would be the under-specification the
/// first run spent a section on.
const LOREM: &str = "lorem ipsum dolor sit amet consectetur adipiscing elit sed do ei";

/// `modal-over-list`'s spinner, one glyph a frame.
const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// How many rows the list scenes pretend to have. Never iterated.
const LIST_ROWS: i32 = 10_000;

/// The terminal's default foreground, as `SCENES.md` declares it.
const DEFAULT_FG: Rgb = Rgb::new(192, 192, 192);
/// The terminal's default background, as `SCENES.md` declares it.
const DEFAULT_BG: Rgb = Rgb::new(0, 0, 0);

/// The one paint every scene needs, resolved once per frame from the frame's own theme.
///
/// A value carried down rather than a call at each site, because [`Theme::custom`] is documented as
/// a per-call cost and not a lookup — it builds a style rather than reading an array — and a
/// component that calls it per cell is paying per cell. One call a frame is the honest shape; scene
/// 4 is the one place this arm cannot avoid paying per cell, and it says so where it does.
#[derive(Clone, Copy)]
struct Paints {
    /// The declared default pair.
    plain: Paint,
}

impl Paints {
    fn of(theme: &Theme) -> Paints {
        Paints {
            plain: theme.custom(DEFAULT_FG, DEFAULT_BG),
        }
    }
}

/// Turn a row of already-drawn cells into reverse video.
///
/// # The first version of this arm swapped two colours instead, and it was a false win
///
/// A `Paint` is two colours, so `theme.custom(DEFAULT_BG, DEFAULT_FG)` looks like reverse video and
/// is the described picture at the truecolor tier. At the `no-color` tier it is **nothing**: the
/// serializer narrows both colours to the terminal's default, the swap collapses, and the row comes
/// out identical to the rows around it.
///
/// The numbers that produced were the reason to look. At `no-color`, `list-scroll` read **40.0 bytes
/// a frame against the engine arm's 572.3** — a fourteenfold win, on a scene where the two arms are
/// the same compositor — because the highlight had silently stopped existing and the frames no
/// longer differed by a moving reverse-video row. ratatui and Textual keep theirs at that tier,
/// because SGR 7 is an *attribute* and survives having no colour. **A cell that is cheaper because
/// the picture lost an element is the missing-row defect committed inside a number**, and it is
/// worse than a blank, because a blank at least looks like an absence.
///
/// So the attribute is asked for as an attribute. [`Ctx::restyle`] is the engine's third verb
/// surfaced through the runtime and [`Repaint`] is the descriptor it takes; `set: Repaint::REVERSE`
/// is a bit on the cell rather than a pair of colours, and it survives the narrowing exactly the way
/// every other arm's does. The cost is one more verb over the same rectangle, which is in the number.
fn reverse_row(cx: &mut Ctx<'_, '_>, x: i32, y: i32, w: u16) {
    cx.restyle(
        Rect::new(x, y, w, 1),
        &Repaint {
            set: Repaint::REVERSE,
            ..Default::default()
        },
    );
}

fn main() {
    let mut scene = String::new();
    let mut frames: u32 = 120;
    let mut seconds: u64 = 10;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--scene" => scene = args.next().unwrap_or_default(),
            "--frames" => frames = args.next().and_then(|v| v.parse().ok()).unwrap_or(120),
            "--seconds" => seconds = args.next().and_then(|v| v.parse().ok()).unwrap_or(10),
            other => {
                eprintln!("arm-vitui-runtime: unknown argument {other}");
                std::process::exit(2);
            }
        }
    }

    let w: u16 = env_u16("COLUMNS", 120);
    let h: u16 = env_u16("LINES", 40);

    // The one line on stderr, before the first frame. The version is the runtime's, which is this
    // workspace's, which is the arm's — one number for all three, and it is the number the report
    // has to carry because a comparison against an unnamed version is not one.
    eprintln!(
        "arm=vitui-runtime version={} no_color=honoured alt_screen=yes",
        env!("CARGO_PKG_VERSION"),
    );

    match scene.as_str() {
        "caret" => picture(w, h, frames, caret),
        "status-line" => picture(w, h, frames, status_line),
        "list-scroll" => picture(w, h, frames, list_scroll),
        "full-repaint" => picture(w, h, frames, full_repaint),
        "modal-over-list" => picture(w, h, frames, modal_over_list),
        "unchanged" => picture(w, h, frames, unchanged),
        "fade" => picture(w, h, frames, fade),
        "scattered" => picture(w, h, frames, scattered),
        "filter-shrink" => picture(w, h, frames, filter_shrink),
        "latency" => latency(w, h),
        "cpu" => cpu(w, h, seconds),
        "" => {
            eprintln!("arm-vitui-runtime: --scene is required");
            std::process::exit(2);
        }
        other => {
            eprintln!("arm-vitui-runtime: no scene {other}");
            std::process::exit(2);
        }
    }
}

fn env_u16(name: &str, fallback: u16) -> u16 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(fallback)
}

/// A driver over a buffered stdout, with the inline clock.
///
/// [`Driver::headless`] exists and is not used: it writes into a `Vec` and picks its own theme, and
/// both of those are the measurement's business rather than the runtime's. The theme handed in is
/// the shipped default resolved at truecolor — its roles are never read, because every paint this
/// arm makes comes from [`Theme::custom`] over the two declared colours, but a `Driver` needs one
/// and the scrim asks it which direction to move (`Theme::is_dark`).
fn attach(w: u16, h: u16) -> Driver {
    let sink = std::io::BufWriter::with_capacity(1 << 16, std::io::stdout());
    Driver::attach(
        Config {
            clock: Clock::Manual,
            output: Output::Sink(Box::new(sink)),
            size: (w, h),
            ..Default::default()
        },
        Theme::default().resolve(vitui::engine::ColorDepth::TrueColor),
    )
    .expect("attaching to a sink cannot fail")
}

/// The nine picture scenes: draw frame `n` and present, `frames` times.
///
/// One `frame` call per frame and the whole picture inside it. There is no per-scene state carried
/// across the loop and there is nothing to build once, which is the difference from `arms/vitui`'s
/// `Stack`: the runtime has no retained structure to build.
fn picture(w: u16, h: u16, frames: u32, mut draw: impl FnMut(&mut Ctx<'_, '_>, u32, u16, u16)) {
    let mut driver = attach(w, h);
    for n in 0..frames {
        let presented = driver.frame(|cx| draw(cx, n, w, h));
        assert!(presented.submitted, "frame {n} was not submitted");
    }
    drop(driver);
    let _ = std::io::stdout().flush();
}

/// Scene 1. Row 0 and a caret.
///
/// # **Nothing focused means no caret**, so this arm has to focus something
///
/// The first version of this function was `arms/vitui`'s with one call renamed, and it wrote **no
/// caret at all** — not on the odd frames, on any of them. `Frame::settle_caret` drops the caret
/// when nothing holds the keyboard, and its reason is a good one: *a caret on a screen where no
/// widget holds the keyboard is a lie about where typing goes*. The engine has no such rule and
/// cannot have one; it does not know what focus is.
///
/// So the arm does what an application would: it declares row 0 a tab stop and focuses it. That is
/// the framework's idiom rather than a workaround, and it is the honest way to the described
/// picture — but it is **not free, and the difference belongs in the report rather than in a
/// comment nobody reads**. This row costs an id, a hit-index entry, a focus stop and a ring
/// membership that `arms/vitui` does not pay, and it is the clearest small example of what the two
/// columns are measuring differently: the engine arm places a cursor, and this one says who owns it.
fn caret(cx: &mut Ctx<'_, '_>, n: u32, w: u16, _h: u16) {
    const TITLE: &str = "vitui compare — caret";
    let p = Paints::of(cx.theme());
    cx.text(0, 0, TITLE, p.plain);
    let id = Id::from_raw(1);
    cx.interact(id, Rect::new(0, 0, w, 1), Interest::FOCUS);
    cx.focus(id);
    if n.is_multiple_of(2) {
        // `caret_with` and not a cell: the caret is the engine's (the rule) and the runtime hands it
        // through, so this row measures the same mechanism `arms/vitui` measures with one more call
        // in front of it.
        cx.caret_with(
            i32::from(vitui::engine::width_of(TITLE)),
            0,
            CursorShape::Block,
        );
    }
}

/// Scene 2. Thirty-nine static rows and a status line, all of it redrawn every frame.
fn status_line(cx: &mut Ctx<'_, '_>, n: u32, w: u16, h: u16) {
    let p = Paints::of(cx.theme());
    body_rows(cx, p, h);
    let elapsed = format!("{:.2}s", f64::from(n) / 60.0);
    let bar = format!(" frame {n:<6}elapsed {elapsed:<6}   cpu 12%    3 tasks");
    status_bar(cx, p, &bar, w, h);
}

/// Rows 0..h-2 of scenes 2 and 6.
fn body_rows(cx: &mut Ctx<'_, '_>, p: Paints, h: u16) {
    for row in 0..i32::from(h) - 1 {
        cx.text(0, row, &format!("line {row:02}  {LOREM}"), p.plain);
    }
}

/// The reverse-video row at the bottom of scenes 2, 6, 9 and `latency`.
fn status_bar(cx: &mut Ctx<'_, '_>, p: Paints, bar: &str, w: u16, h: u16) {
    let padded = format!("{bar:<width$}", width = usize::from(w));
    let row = i32::from(h) - 1;
    cx.text(0, row, &padded, p.plain);
    reverse_row(cx, 0, row, w);
}

/// Scene 3. A window onto ten thousand rows that advances one row a frame.
fn list_scroll(cx: &mut Ctx<'_, '_>, n: u32, w: u16, _h: u16) {
    let p = Paints::of(cx.theme());
    let top = i32::try_from(n).expect("120 frames fit in an i32");
    let mut list = cx.scrolled(0, -top);
    // The runtime's own culling query, which is the engine's under a different name: `visible_rows`
    // says which rows of the offset space the clip can still reach, and ten thousand never enter the
    // loop. The data-volume invariant is the runtime's to keep as much as the engine's — the engine's design's
    // scene 2 gates it at 15 876x, flat from a thousand rows to a million.
    for row in list.visible_rows() {
        if !(0..LIST_ROWS).contains(&row) {
            continue;
        }
        let line = format!("{row:05}  item-{row:05}---------");
        let padded = format!("{line:<width$}", width = usize::from(w));
        list.text(0, row, &padded, p.plain);
        if row == top + 12 {
            reverse_row(&mut list, 0, row, w);
        }
    }
}

/// Scene 4. Every cell differs from the frame before it.
///
/// **The one scene where this arm pays per cell and cannot not.** [`Theme::custom`] builds a style
/// rather than reading one, and its own documentation says a component that calls it per cell is
/// paying per cell and should publish the census. This is that census: 4 800 calls a frame, because
/// a 24-bit ramp is not thirteen roles and no theme has a role for *the colour at column c*. It is
/// the price of the rule on the one picture that is nothing but colour, and it is in the CPU column
/// rather than the byte column — the wire is the same wire.
fn full_repaint(cx: &mut Ctx<'_, '_>, n: u32, w: u16, h: u16) {
    let shift = i32::try_from(n).expect("120 frames fit in an i32");
    let theme = cx.theme();
    for y in 0..i32::from(h) {
        for x in 0..i32::from(w) {
            let source = (x + shift).rem_euclid(i32::from(w));
            let r = u8::try_from((source * 2).clamp(0, 255)).expect("clamped");
            let g = u8::try_from((y * 6).clamp(0, 255)).expect("clamped");
            // The background is the declared default rather than *left alone*, for the reason in the
            // module comment: this arm has no way to leave one alone.
            let paint = theme.custom(Rgb::new(r, g, 128), DEFAULT_BG);
            cx.set(x, y, "#", paint);
        }
    }
}

/// Scene 5. Scene 3's frame 0, dimmed, with an undimmed dialog over it.
///
/// **The dim is a `Scrim` and the dialog is an overlay**, which is the runtime's whole answer to
/// this picture and is one call rather than two layers reconciled by hand. The scrim is an operator
/// layer the driver puts at `z - 1` over the *whole screen*, which is why the anchor below decides
/// only where the dialog lands and nothing about the dim.
fn modal_over_list(cx: &mut Ctx<'_, '_>, n: u32, w: u16, h: u16) {
    list_scroll(cx, 0, w, h);
    let opts = OverlayOpts {
        z: Z::MODAL,
        size: (40, 12),
        // Below a full-width one-row anchor at row 13, centred: y = 14, x = (120 - 40) / 2 = 40.
        // Exactly the rectangle `SCENES.md` fixes, arrived at through `place` rather than asserted.
        placement: Placement::new(Side::Below, Align::Center),
        opaque: true,
        // Halfway toward black, which is what the scene declares the dim to be. `Scrim::shadow`
        // rather than `Scrim::for_theme`, because the direction here is the scene's and not the
        // theme's — a lift would be the described picture on no terminal at all.
        scrim: Some(Scrim::shadow(Mix::FULL / 2)),
    };
    let anchor = Rect::new(0, 13, w, 1);
    let glyph = SPINNER[usize::try_from(n).expect("120 frames") % SPINNER.len()];
    cx.overlay(Id::from_raw(1), anchor, opts, move |cx| {
        let p = Paints::of(cx.theme());
        let inner = 38usize;
        cx.text(0, 0, &format!("┌{}┐", "─".repeat(inner)), p.plain);
        for row in 1..11 {
            cx.text(0, row, &format!("│{}│", " ".repeat(inner)), p.plain);
        }
        cx.text(0, 11, &format!("└{}┘", "─".repeat(inner)), p.plain);
        cx.text(2, 1, &format!("{glyph} working"), p.plain);
        for (i, line) in [
            "Copying 4 of 12 files.",
            "This dialog is a layer, and the",
            "list behind it is another one.",
            "Nothing below has been rewritten.",
        ]
        .iter()
        .enumerate()
        {
            cx.text(2, 3 + i32::try_from(i).expect("four lines"), line, p.plain);
        }
    });
}

/// Scene 6. Scene 2's screen, drawn again on every frame, with nothing on it different.
///
/// **The scene that this arm is the point of.** Redrawing everything is not a choice here and it is
/// not a stress test: it is the only thing an immediate-mode runtime does. So this row is 4 800
/// cells built, laid out, id-ed and written on every one of 120 frames, against a wire that should
/// carry nothing at all — which is the runtime's own scene 16 (0 wakeups against 60 polled) and the
/// ledger's *an unchanged frame at 23.58 µs, nothing skips the composition*, asked of the wire.
fn unchanged(cx: &mut Ctx<'_, '_>, _n: u32, w: u16, h: u16) {
    let p = Paints::of(cx.theme());
    body_rows(cx, p, h);
    status_bar(
        cx,
        p,
        " frame 0     elapsed 0.00s    cpu 12%    3 tasks",
        w,
        h,
    );
}

/// Scene 7. A 60x20 panel of `=` fading up from black.
fn fade(cx: &mut Ctx<'_, '_>, n: u32, _w: u16, _h: u16) {
    let v = u8::try_from((n * 2).min(255)).expect("clamped to 255");
    let paint = cx.theme().custom(Rgb::new(v, v, v), DEFAULT_BG);
    cx.fill(Rect::new(30, 10, 60, 20), "=", paint);
}

/// Scene 8. Twelve panels, and one two-digit field in each of them moves.
fn scattered(cx: &mut Ctx<'_, '_>, n: u32, _w: u16, _h: u16) {
    let p = Paints::of(cx.theme());
    for k in 0..12u32 {
        let x = i32::try_from(30 * (k % 4)).expect("under 120");
        let y = i32::try_from(13 * (k / 4)).expect("under 40");
        // A panel is a child context, which is what a panel is at this layer: a rectangle and a clip
        // that the calls inside it are relative to. It costs one `Ctx` and no allocation.
        let mut panel = cx.child(Rect::new(x, y, 30, 13));
        panel.text(0, 0, &format!("panel {k:02}"), p.plain);
        panel.text(0, 2, &format!("count {:02}", (k + n) % 100), p.plain);
    }
}

/// Scene 9. A list that narrows and widens again, and rows that leave the screen blank.
fn filter_shrink(cx: &mut Ctx<'_, '_>, n: u32, w: u16, h: u16) {
    let p = Paints::of(cx.theme());
    let matches = 39 - (n % 39);
    for row in 0..i32::from(h) - 1 {
        let index = u32::try_from(row).expect("under 40");
        let line = if index < matches {
            format!("{row:05}  item-{row:05}---------")
        } else {
            String::new()
        };
        let padded = format!("{line:<width$}", width = usize::from(w));
        cx.text(0, row, &padded, p.plain);
    }
    status_bar(cx, p, &format!(" search: {matches:02} matches"), w, h);
}

/// The latency mode: scene 2's screen, then one status-line field per keystroke, flushed.
///
/// stdin is read on this thread rather than through the runtime's key queue, for `arms/vitui`'s
/// reason: the input thread needs a tty and the harness gives every arm a pipe. What is measured is
/// the interval from the byte arriving to the first output byte after it, and that is unaffected.
fn latency(w: u16, h: u16) {
    use std::io::Read as _;

    let mut driver = attach(w, h);
    driver.frame(|cx| status_line(cx, 0, w, h));
    let _ = std::io::stdout().flush();

    let mut stdin = std::io::stdin();
    let mut byte = [0u8; 1];
    let mut pending: Vec<u8> = Vec::new();
    while let Ok(1) = stdin.read(&mut byte) {
        pending.push(byte[0]);
        let name = match pending.as_slice() {
            [0x03] => Some("C-c".to_string()),
            [0x1b] | [0x1b, b'['] => None,
            [0x1b, b'[', b'A'] => Some("Up".to_string()),
            [0x1b, b'[', b'B'] => Some("Down".to_string()),
            [0x1b, b'[', b'C'] => Some("Right".to_string()),
            [0x1b, b'[', b'D'] => Some("Left".to_string()),
            [b] if b.is_ascii_graphic() => Some((*b as char).to_string()),
            _ => Some("?".to_string()),
        };
        let Some(name) = name else { continue };
        pending.clear();
        let bar = format!(" key {name:<10} elapsed 0.00s    cpu 12%    3 tasks");
        driver.frame(|cx| {
            let p = Paints::of(cx.theme());
            body_rows(cx, p, h);
            status_bar(cx, p, &bar, w, h);
        });
        let _ = std::io::stdout().flush();
    }
}

/// The CPU mode: scene 1 at 60 frames a second for a fixed wall-clock window.
fn cpu(w: u16, h: u16, seconds: u64) {
    let mut driver = attach(w, h);
    let began = Instant::now();
    let window = Duration::from_secs(seconds);
    let interval = Duration::from_nanos(1_000_000_000 / 60);
    let mut n: u32 = 0;
    loop {
        let due = began + interval * n;
        let now = Instant::now();
        if now < due {
            std::thread::sleep(due - now);
        }
        if began.elapsed() >= window {
            break;
        }
        driver.frame(|cx| caret(cx, n, w, h));
        let _ = std::io::stdout().flush();
        n += 1;
    }
    eprintln!("frames={n}");
}
