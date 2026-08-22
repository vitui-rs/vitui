//! The vitui arm of the comparative suite.
//!
//! `../../SCENES.md` is normative and `../../ARM-CONTRACT.md` is what the harness relies on. This
//! file implements the seven modes for `vitui-engine` and nothing else — no runtime, because there
//! is not one yet, and no components, for the same reason. What is being compared is the layer that
//! touches the terminal.
//!
//! # Two things about this arm that the report has to say out loud
//!
//! **It is the home team, and it wrote the scenes.** Every rule in `SCENES.md` exists to stop that
//! being worth anything — a scene is a described picture and an arm may reach it however its
//! framework prefers — but the rules cannot make the *choice of scenes* neutral. Three of the five
//! are scenes this engine was designed against. `full-repaint` is on the list precisely because it
//! is the one row where nothing this engine does can help.
//!
//! **`Clock::Manual`, and it is not a test fixture.** It is public API (spec §14), and it is the
//! mode in which `present` composites, packs, serialises and writes inline on the calling thread
//! before returning. That is the honest arrangement for a byte measurement: the alternative puts
//! serialisation on another thread, and the bytes would then be counted from a process that had not
//! finished producing them. It also makes frame *n* a pure function of *n*, which the contract
//! requires and the harness asserts.

use std::io::Write as _;
use std::time::{Duration, Instant};

use vitui_engine::{
    Clock, Color, Config, Cursor, CursorShape, Engine, LayerId, Mix, Output, Rect, Screen, Style,
};

/// The 64-character lorem row `status-line`'s static body is made of, verbatim from `SCENES.md`.
///
/// **Written out rather than generated, and `SCENES.md` had to be corrected to write it out too.**
/// The first draft of that file gave the string's *length* and not the string, which decides scene
/// 2's frame 0 outright — the ratatui arm chose a different sixty-four characters and the two arms
/// were measuring two pictures. One of the nine under-specifications the first run found.
const LOREM: &str = "lorem ipsum dolor sit amet consectetur adipiscing elit sed do ei";

/// `modal-over-list`'s spinner, one glyph a frame. Braille, because it is the glyph set a spinner
/// actually uses and because it is two bytes a frame on the wire in UTF-8 rather than one.
const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// How many rows the list scenes pretend to have. Never iterated — the whole point of the culling
/// query is that this number does not reach the frame — but stated, because `list-scroll` is a
/// window onto ten thousand rows and an arm that quietly held forty would be measuring a different
/// scene.
const LIST_ROWS: i32 = 10_000;

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
                eprintln!("arm-vitui: unknown argument {other}");
                std::process::exit(2);
            }
        }
    }

    // `COLUMNS` and `LINES`, and there is no other source. The contract sets both on every run
    // precisely so that no arm has to ask a pipe how wide it is.
    let w: u16 = env_u16("COLUMNS", 120);
    let h: u16 = env_u16("LINES", 40);

    // The one line on stderr, before the first frame. `no_color=honoured` is a claim, and it is
    // checkable: `NO_COLOR` is precedence level 3 in `Capabilities` (see `caps.rs`), above
    // detection and under the application's own `Overrides`, and the harness runs the declared tier
    // to see the byte count move.
    eprintln!(
        "arm={} version={} no_color=honoured alt_screen=yes",
        "vitui",
        env!("CARGO_PKG_VERSION"),
    );

    match scene.as_str() {
        "caret" => picture(w, h, frames, caret),
        "status-line" => picture(w, h, frames, status_line),
        "list-scroll" => picture(w, h, frames, list_scroll),
        "full-repaint" => picture(w, h, frames, full_repaint),
        "modal-over-list" => picture(w, h, frames, modal_over_list),
        "latency" => latency(w, h),
        "cpu" => cpu(w, h, seconds),
        "" => {
            eprintln!("arm-vitui: --scene is required");
            std::process::exit(2);
        }
        other => {
            eprintln!("arm-vitui: no scene {other}");
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

/// A screen writing straight into a `BufWriter<Stdout>`, with the inline clock.
///
/// The buffer matters and is not a thumb on the scale: what is being measured is bytes, and a
/// `BufWriter` changes how many `write` syscalls carry them and not how many there are. It is here
/// because the alternative is a syscall per serialiser flush, which would make the CPU column a
/// measurement of `write(2)`.
fn attach(w: u16, h: u16) -> Screen {
    let sink = std::io::BufWriter::with_capacity(1 << 16, std::io::stdout());
    let (screen, _wake) = Engine::new(Config {
        // Inline: composite, pack, serialise and write before `present` returns. See the module
        // comment — this is the mode a byte measurement has to be taken in.
        clock: Clock::Manual,
        output: Output::Sink(Box::new(sink)),
        size: (w, h),
        ..Default::default()
    })
    .attach()
    .expect("attaching to a sink cannot fail");
    screen
}

/// The five picture scenes, which are all the same loop: build the layers once, then draw frame `n`
/// and present, `frames` times.
///
/// The layers are built once and reused, and that is this engine's idiom rather than a shortcut — a
/// layer is a rectangle and a z, and rebuilding the stack every frame would be paying for a scene
/// tree the engine deliberately does not have.
fn picture(
    w: u16,
    h: u16,
    frames: u32,
    mut draw: impl FnMut(&mut Screen, &mut Stack, u32, u16, u16),
) {
    let mut screen = attach(w, h);
    let mut stack = build(&mut screen, w, h);
    for n in 0..frames {
        draw(&mut screen, &mut stack, n, w, h);
        let presented = screen.present();
        // `Clock::Manual` is not paced, so this cannot be false. Asserted rather than assumed
        // because a silently coalesced frame is a byte count for a scene that was not drawn.
        assert!(presented.submitted, "frame {n} was not submitted");
    }
    // The epilogue — leaving the alternate screen, restoring auto-wrap and the caret — is `Drop`'s,
    // and it is in the byte count on purpose. Every arm that entered the alternate screen pays to
    // leave it, and the contract's `alt_screen` field is what lets the report say which did.
    drop(screen);
    let _ = std::io::stdout().flush();
}

/// Which layers a scene has. Three ids, because `modal-over-list` is the only scene with more than
/// one and a tuple is cheaper to read than an enum with one populated arm.
#[derive(Clone, Copy)]
struct Stack {
    base: LayerId,
    dim: Option<LayerId>,
    dialog: Option<LayerId>,
}

fn build(screen: &mut Screen, w: u16, h: u16) -> Stack {
    let full = Rect::new(0, 0, w, h);
    let base = screen.layers().add_content(0, full, true);
    Stack {
        base,
        dim: None,
        dialog: None,
    }
}

/// Scene 1. Row 0 and a caret, and the caret is the only thing that differs between frames.
fn caret(screen: &mut Screen, stack: &mut Stack, n: u32, _w: u16, _h: u16) {
    const TITLE: &str = "vitui compare — caret";
    if n == 0 {
        let mut view = screen.layers().view(stack.base).expect("the base layer");
        view.text(0, 0, TITLE, Style::new());
    }
    // `set_cursor` rather than a cell, and this is the engine's own decision showing up in a
    // comparison: ADR 0005 puts the caret on the engine because a runtime that computes its column
    // itself is five columns wrong on one family emoji. Here it costs the shortest cursor encoding
    // and a show/hide, and no cell is touched at all.
    //
    // `width_of` rather than `chars().count()`: the title has an em dash in it, which is one
    // grapheme, one column and three bytes, and only one of those three numbers is the caret's
    // column.
    let col = vitui_engine::width_of(TITLE);
    screen.set_cursor(if n % 2 == 0 {
        Some(Cursor {
            x: col,
            y: 0,
            shape: CursorShape::Block,
        })
    } else {
        None
    });
}

/// Scene 2. Thirty-nine static rows and a status line whose first two fields move.
fn status_line(screen: &mut Screen, stack: &mut Stack, n: u32, w: u16, h: u16) {
    let status_row = i32::from(h) - 1;
    let mut view = screen.layers().view(stack.base).expect("the base layer");
    if n == 0 {
        for row in 0..status_row {
            let line = format!("line {row:02}  {LOREM}");
            view.text(0, row, &line, Style::new());
        }
    }
    // The whole status line every frame, reverse video, including the two fields that never change.
    // **That is the arm being ordinary rather than clever**: a caller who wanted to write only the
    // digits could, and the scene does not ask for it. What the equality filter does with the
    // unchanged tail is the engine's business and is exactly what this row measures.
    //
    // **The two field widths are normative and they are worth about ten times this row.** `frame` in
    // six columns and `elapsed` in six means `frame 119` is as wide as `frame 0` and nothing after
    // either of them moves; an arm that lets the tail shift pays roughly 500 bytes a frame against
    // 46, and the difference would have read as a difference in its renderer rather than in its
    // `format!`. `SCENES.md` did not say so until the first run found it.
    let elapsed = format!("{:.2}s", f64::from(n) / 60.0);
    let bar = format!(" frame {n:<6}elapsed {elapsed:<6}   cpu 12%    3 tasks");
    let padded = format!("{bar:<width$}", width = usize::from(w));
    view.text(0, status_row, &padded, Style::new().reverse());
}

/// Scene 3. A window onto ten thousand rows that advances one row a frame.
fn list_scroll(screen: &mut Screen, stack: &mut Stack, n: u32, w: u16, _h: u16) {
    let top = i32::try_from(n).expect("120 frames fit in an i32");
    let mut view = screen.layers().view(stack.base).expect("the base layer");
    // **The culling query, and it is the reason this arm can claim the data-volume invariant.**
    // `scrolled` offsets the view and `visible_rows` says which rows of the offset space the clip
    // can still reach; ten thousand rows never enter the loop. See ADR 0002 — the engine does not
    // iterate application data, and this is what "the caller does the culling" looks like at a call
    // site.
    let mut list = view.scrolled(0, -top);
    let rows = list.visible_rows();
    for row in rows {
        if !(0..LIST_ROWS).contains(&row) {
            continue;
        }
        // The highlight follows list index `12 + n`, which is screen row 12 for every frame.
        let style = if row == top + 12 {
            Style::new().reverse()
        } else {
            Style::new()
        };
        let line = format!("{row:05}  item-{row:05}---------");
        let padded = format!("{line:<width$}", width = usize::from(w));
        list.text(0, row, &padded, style);
    }
}

/// Scene 4. Every one of the cells differs from the frame before it.
fn full_repaint(screen: &mut Screen, stack: &mut Stack, n: u32, w: u16, h: u16) {
    let shift = i32::try_from(n).expect("120 frames fit in an i32");
    let mut view = screen.layers().view(stack.base).expect("the base layer");
    for y in 0..i32::from(h) {
        for x in 0..i32::from(w) {
            let source = (x + shift).rem_euclid(i32::from(w));
            let r = u8::try_from((source * 2).clamp(0, 255)).expect("clamped");
            let g = u8::try_from((y * 6).clamp(0, 255)).expect("clamped");
            view.set(x, y, "#", Style::new().fg(Color::rgb(r, g, 128)));
        }
    }
}

/// Scene 5. Scene 3's frame 0, dimmed, with an undimmed dialog over it.
fn modal_over_list(screen: &mut Screen, stack: &mut Stack, n: u32, w: u16, h: u16) {
    let dialog_rect = Rect::new(
        (i32::from(w) - 40) / 2,
        (i32::from(h) - 12) / 2,
        40,
        12,
    );
    if n == 0 {
        list_scroll(screen, stack, 0, w, h);
    }
    let (dim, dialog) = match (stack.dim, stack.dialog) {
        (Some(a), Some(b)) => (a, b),
        // **Once, on frame 0, and the `else` arm here is why `Stack` is `&mut`.** The first draft
        // took it by value, so this arm ran on every frame and added two layers a frame — a scene
        // with 240 layers by the end of it, and a byte count for a picture nobody described. It
        // failed nothing: the picture was right, because the added layers were identical to the
        // ones already there. A layer stack that grows silently is the shape of defect this
        // backlog has met five times, and it found it here in a measurement program.
        _ => {
            // **Two layers and one primitive.** The dim is an *operator* layer — one `Mix` toward
            // black over the whole screen — and the dialog is a content layer above it, so it is
            // untouched by an operator that is below it. This is spec §6's "blending is one
            // primitive" at a call site: a shadow, a modal dim and a fade are the same mechanism,
            // and the caller never pre-computes a colour.
            //
            // It is also the row where the notcurses arm reads `cannot express`, and the difference
            // is exactly this: the alternative is for the caller to compute dimmed colours for
            // three thousand cells and write them, which is not compositing however identical the
            // picture is.
            let dim = screen
                .layers()
                .add_operator(1, Rect::new(0, 0, w, h), Mix::darken(Mix::FULL / 2));
            let dialog = screen.layers().add_content(2, dialog_rect, true);
            let mut view = screen.layers().view(dialog).expect("the dialog layer");
            let inner = usize::from(dialog_rect.w) - 2;
            view.text(0, 0, &format!("┌{}┐", "─".repeat(inner)), Style::new());
            for row in 1..i32::from(dialog_rect.h) - 1 {
                view.text(0, row, &format!("│{}│", " ".repeat(inner)), Style::new());
            }
            view.text(
                0,
                i32::from(dialog_rect.h) - 1,
                &format!("└{}┘", "─".repeat(inner)),
                Style::new(),
            );
            for (i, line) in [
                "Copying 4 of 12 files.",
                "This dialog is a layer, and the",
                "list behind it is another one.",
                "Nothing below has been rewritten.",
            ]
            .iter()
            .enumerate()
            {
                view.text(
                    2,
                    3 + i32::try_from(i).expect("four lines"),
                    line,
                    Style::new(),
                );
            }
            stack.dim = Some(dim);
            stack.dialog = Some(dialog);
            (dim, dialog)
        }
    };
    let _ = dim;
    // The spinner, and nothing else. One cluster a frame over a 4 800-cell screen with an operator
    // layer over three quarters of it — which is §7's "a status bar across three dialogs" shape,
    // and it is the reason the operator's reach has cost this backlog five defects.
    let mut view = screen.layers().view(dialog).expect("the dialog layer");
    let glyph = SPINNER[usize::try_from(n).expect("120 frames") % SPINNER.len()];
    view.text(2, 1, &format!("{glyph} working"), Style::new().bold());
}

/// The latency mode: scene 2's screen, then one status-line field per keystroke, flushed.
///
/// stdin is read a byte at a time on this thread rather than through the engine's input thread, and
/// that is the contract's doing rather than a shortcut: the input thread needs a tty and the harness
/// gives every arm a pipe. What is measured is unaffected — the interval from the byte arriving to
/// the first output byte after it — and this way no arm is measured through a facility another arm
/// does not have.
fn latency(w: u16, h: u16) {
    use std::io::Read as _;

    let mut screen = attach(w, h);
    let mut stack = build(&mut screen, w, h);
    status_line(&mut screen, &mut stack, 0, w, h);
    assert!(screen.present().submitted, "the initial screen");
    let _ = std::io::stdout().flush();

    let status_row = i32::from(h) - 1;
    let mut stdin = std::io::stdin();
    let mut byte = [0u8; 1];
    let mut pending: Vec<u8> = Vec::new();
    while let Ok(1) = stdin.read(&mut byte) {
        pending.push(byte[0]);
        // A minimal decoder, and deliberately minimal: an escape sequence arrives in pieces and the
        // quantity being measured is the reaction to a *complete* keystroke. Anything still
        // incomplete waits for the next byte rather than being answered with a guess — which is
        // ADR 0007's rule, and it is the right rule for a benchmark too: an arm that answered a
        // half-arrived escape would be advertising a latency for a keystroke that had not happened.
        let name = match pending.as_slice() {
            [0x03] => Some("C-c".to_string()),
            [0x1b] => None,
            [0x1b, b'['] => None,
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
        let padded = format!("{bar:<width$}", width = usize::from(w));
        let mut view = screen.layers().view(stack.base).expect("the base layer");
        view.text(0, status_row, &padded, Style::new().reverse());
        assert!(screen.present().submitted, "a keystroke's frame");
        let _ = std::io::stdout().flush();
    }
}

/// The CPU mode: scene 1 at 60 frames a second for a fixed wall-clock window.
///
/// A deadline computed from the start rather than a `sleep` of a sixtieth each time round, because
/// the second form drifts by however long the frame took and would end up comparing arms on how
/// long they slept.
fn cpu(w: u16, h: u16, seconds: u64) {
    let mut screen = attach(w, h);
    let mut stack = build(&mut screen, w, h);
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
        caret(&mut screen, &mut stack, n, w, h);
        let _ = screen.present();
        let _ = std::io::stdout().flush();
        n += 1;
    }
    eprintln!("frames={n}");
}
