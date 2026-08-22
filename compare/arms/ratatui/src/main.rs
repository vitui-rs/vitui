//! The ratatui arm of the vitui comparative suite.
//!
//! `compare/SCENES.md` defines the scenes and `compare/ARM-CONTRACT.md` defines the program this
//! has to be. Everything here is in service of reaching the *pictures* those files describe; where
//! the scene text left a choice open, the choice made is recorded in `NOTES.md` beside this file
//! rather than buried in a comment.
//!
//! The idiom is ratatui's own: build the whole frame every time and let `Terminal`'s
//! double-buffered diff decide which cells reach the wire. That is the thing being compared.

use std::io::{self, BufWriter, Read, Write};
use std::process::ExitCode;
use std::time::{Duration, Instant};

use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, BorderType, Clear, Widget};
use ratatui::{Frame, Terminal, TerminalOptions, Viewport};
use ratatui_crossterm::CrosstermBackend;

/// The version reported on stderr. `ratatui`'s own version, pinned exactly in `Cargo.toml`; kept
/// as a literal because `env!("CARGO_PKG_VERSION")` would report the arm's version, not its
/// subject's. If the pin in `Cargo.toml` moves, this moves with it.
const RATATUI_VERSION: &str = if cfg!(feature = "scrolling-regions") {
    "0.30.2+scrolling-regions"
} else {
    "0.30.2"
};

/// `NO_COLOR` is *honoured*, and by crossterm rather than by this file: `Colored::fmt` writes
/// nothing when `NO_COLOR` is non-empty (`crossterm-0.29.0/src/style/types/colored.rs`). Verified
/// on the wire — `full-repaint` over 120 frames drops from 12 745 942 bytes to 2 914 822 — and it
/// is honoured with a caveat large enough that `NOTES.md` spells it out: the enclosing
/// `SetForegroundColor` command still writes its `ESC [` and `m`, so each suppressed colour becomes
/// a five-byte `ESC [ ; m`, which is an SGR reset rather than nothing.
const NO_COLOR: &str = "honoured";

/// The arm enters the alternate screen, because `ratatui::init()` does.
const ALT_SCREEN: &str = "yes";

const DEFAULT_WIDTH: u16 = 120;
const DEFAULT_HEIGHT: u16 = 40;
const DEFAULT_FRAMES: u64 = 120;
const DEFAULT_SECONDS: f64 = 10.0;

const CARET_TITLE: &str = "vitui compare — caret";

/// The 64-character lorem string of scene 2. `SCENES.md` fixes its length and not its content;
/// this is the arm's choice and it is load-bearing for the frame-0 byte count only. See `NOTES.md`.
const LOREM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do.";
const _: () = assert!(LOREM.len() == 64);

/// Scene 3's label width. `SCENES.md` says "a 20-character label" and illustrates it with a
/// 19-character example; the prose wins. See `NOTES.md`.
const LABEL_WIDTH: usize = 20;

/// Scene 5's spinner, one step per frame.
const SPINNER: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// Scene 5's four rows of body text. Content unspecified by `SCENES.md`; see `NOTES.md`.
const DIALOG_BODY: [&str; 4] = [
    "Rebuilding the index for 3 tasks.",
    "Scanned 4 812 of 10 000 rows.",
    "Elapsed 00:12, estimate 00:26.",
    "Esc runs it in the background.",
];

// -------------------------------------------------------------------------------------------------
// Scenes
// -------------------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum Scene {
    Caret,
    StatusLine,
    ListScroll,
    FullRepaint,
    ModalOverList,
    Latency,
    Cpu,
}

impl Scene {
    fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "caret" => Self::Caret,
            "status-line" => Self::StatusLine,
            "list-scroll" => Self::ListScroll,
            "full-repaint" => Self::FullRepaint,
            "modal-over-list" => Self::ModalOverList,
            "latency" => Self::Latency,
            "cpu" => Self::Cpu,
            _ => return None,
        })
    }
}

// -------------------------------------------------------------------------------------------------
// Drawing — every picture is a pure function of the frame index
// -------------------------------------------------------------------------------------------------

/// Scene 1. One line of text and a caret that alternates between shown and hidden.
///
/// The caret is the terminal's own cursor: ratatui shows it when the frame asked for a position
/// and hides it when it did not, so "shown on even frames" is `set_cursor_position` on even frames.
fn draw_caret(frame: &mut Frame, n: u64) {
    let area = frame.area();
    frame.render_widget(
        Line::from(CARET_TITLE),
        Rect::new(area.x, area.y, area.width, 1),
    );
    if n.is_multiple_of(2) {
        let caret_col = area.x + CARET_TITLE.chars().count() as u16;
        frame.set_cursor_position(Position::new(caret_col, area.y));
    }
}

/// The status line of scenes 2 and `latency`, laid out so that only the first field moves.
///
/// Frame 0 is exactly `SCENES.md`'s literal:
/// `" frame 0     elapsed 0.00s    cpu 12%    3 tasks"`.
fn status_line(first_field: &str, elapsed: f64, width: u16) -> Line<'static> {
    let mut text = format!(" {first_field:<12}elapsed {elapsed:.2}s    cpu 12%    3 tasks");
    let cells = text.chars().count();
    if cells < width as usize {
        text.push_str(&" ".repeat(width as usize - cells));
    }
    Line::styled(text, Style::new().add_modifier(Modifier::REVERSED))
}

/// Scenes 2 and `latency`: 39 static rows and a full-width reverse-video status line.
fn draw_status_scene(frame: &mut Frame, first_field: &str, elapsed: f64) {
    let area = frame.area();
    let body_rows = area.height.saturating_sub(1);
    for r in 0..body_rows {
        frame.render_widget(
            Line::from(format!("line {r:02}  {LOREM}")),
            Rect::new(area.x, area.y + r, area.width, 1),
        );
    }
    frame.render_widget(
        status_line(first_field, elapsed, area.width),
        Rect::new(area.x, area.y + body_rows, area.width, 1),
    );
}

/// One row of scene 3's ten-thousand-row list: `NNNNN  ` then a 20-character label.
fn list_row(index: usize) -> String {
    let mut label = format!("item-{index:05}");
    while label.chars().count() < LABEL_WIDTH {
        label.push('-');
    }
    format!("{index:05}  {label}")
}

/// Scenes 3 and 5: a window onto the list with one reverse-video row.
///
/// `top` is the list index on the screen's first row; `highlight_row` is the *screen* row that
/// carries the highlight, which is what keeps the highlight on `12 + n` as the window advances.
fn draw_list(frame: &mut Frame, top: usize, highlight_row: u16) {
    let area = frame.area();
    for r in 0..area.height {
        let text = list_row(top + r as usize);
        let line = if r == highlight_row {
            Line::styled(text, Style::new().add_modifier(Modifier::REVERSED))
        } else {
            Line::from(text)
        };
        frame.render_widget(line, Rect::new(area.x, area.y + r, area.width, 1));
    }
}

/// Scene 4's 24-bit ramp. A per-cell colour has no widget in ratatui's library, and writing cells
/// from a `Widget` implementation is ratatui's own answer to that — this is the shape `Clear` has.
struct Ramp {
    shift: u16,
}

impl Widget for Ramp {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for y in 0..area.height {
            for x in 0..area.width {
                // Frame n's column x wears the colour column (x + n) % width wore on frame 0.
                let source = (x + self.shift) % area.width;
                if let Some(cell) = buf.cell_mut((area.x + x, area.y + y)) {
                    cell.set_char('#');
                    cell.set_fg(Color::Rgb(
                        (source.saturating_mul(2) & 0xff) as u8,
                        (y.saturating_mul(6) & 0xff) as u8,
                        128,
                    ));
                }
            }
        }
    }
}

/// Scene 5's dim: every cell's foreground and background mixed halfway toward black.
///
/// ratatui has no layers, so there is nothing to composite and nothing to set an alpha on. The
/// dim is therefore computed here, over the buffer, after the list has been drawn into it and
/// before the dialog is. `SCENES.md` licenses this explicitly: the scene is the picture, not the
/// mechanism.
struct Dim;

impl Widget for Dim {
    fn render(self, area: Rect, buf: &mut Buffer) {
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let Some(cell) = buf.cell_mut((x, y)) else {
                    continue;
                };
                // A reverse-video cell has to be resolved before it can be dimmed: halving the
                // colours of a cell that the terminal will then swap is not halfway to black.
                let mut fg = resolve(cell.fg, true);
                let mut bg = resolve(cell.bg, false);
                if cell.modifier.contains(Modifier::REVERSED) {
                    core::mem::swap(&mut fg, &mut bg);
                    cell.modifier.remove(Modifier::REVERSED);
                }
                cell.set_fg(halve(fg));
                cell.set_bg(halve(bg));
            }
        }
    }
}

/// Concrete RGB for a `Color`, because "mixed halfway toward black" needs a number to halve.
///
/// `Color::Reset` is the interesting case: the terminal's default has no value the arm can read,
/// so the arm declares one. See `NOTES.md`.
fn resolve(color: Color, is_fg: bool) -> (u8, u8, u8) {
    match color {
        Color::Reset => {
            if is_fg {
                (192, 192, 192)
            } else {
                (0, 0, 0)
            }
        }
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Black => (0, 0, 0),
        Color::Red => (128, 0, 0),
        Color::Green => (0, 128, 0),
        Color::Yellow => (128, 128, 0),
        Color::Blue => (0, 0, 128),
        Color::Magenta => (128, 0, 128),
        Color::Cyan => (0, 128, 128),
        Color::Gray => (192, 192, 192),
        Color::DarkGray => (128, 128, 128),
        Color::LightRed => (255, 0, 0),
        Color::LightGreen => (0, 255, 0),
        Color::LightYellow => (255, 255, 0),
        Color::LightBlue => (0, 0, 255),
        Color::LightMagenta => (255, 0, 255),
        Color::LightCyan => (0, 255, 255),
        Color::White => (255, 255, 255),
        Color::Indexed(i) => indexed(i),
    }
}

/// The xterm 256-colour palette, enough of it to be honest about `Color::Indexed`.
fn indexed(i: u8) -> (u8, u8, u8) {
    const STEPS: [u8; 6] = [0, 95, 135, 175, 215, 255];
    match i {
        0..=15 => resolve(
            match i {
                0 => Color::Black,
                1 => Color::Red,
                2 => Color::Green,
                3 => Color::Yellow,
                4 => Color::Blue,
                5 => Color::Magenta,
                6 => Color::Cyan,
                7 => Color::Gray,
                8 => Color::DarkGray,
                9 => Color::LightRed,
                10 => Color::LightGreen,
                11 => Color::LightYellow,
                12 => Color::LightBlue,
                13 => Color::LightMagenta,
                14 => Color::LightCyan,
                _ => Color::White,
            },
            true,
        ),
        16..=231 => {
            let c = i - 16;
            (
                STEPS[(c / 36) as usize],
                STEPS[(c % 36 / 6) as usize],
                STEPS[(c % 6) as usize],
            )
        }
        232..=255 => {
            let v = 8 + (i - 232) * 10;
            (v, v, v)
        }
    }
}

fn halve((r, g, b): (u8, u8, u8)) -> Color {
    Color::Rgb(r / 2, g / 2, b / 2)
}

/// Scene 5: the dimmed list, then an undimmed dialog over it.
fn draw_modal_over_list(frame: &mut Frame, n: u64) {
    let area = frame.area();
    draw_list(frame, 0, 12);
    frame.render_widget(Dim, area);

    // 40x12, top-left at (40, 14) — the scene fixes both.
    let dialog = Rect::new(area.x + 40, area.y + 14, 40, 12).intersection(area);
    // `Clear` is how a ratatui popup stops being transparent, and here it is also what makes the
    // dialog undimmed: it resets the cells the dim pass just wrote.
    frame.render_widget(Clear, dialog);

    let block = Block::bordered().border_type(BorderType::Plain);
    let inner = block.inner(dialog);
    frame.render_widget(block, dialog);

    let spinner = SPINNER[(n % SPINNER.len() as u64) as usize];
    frame.render_widget(
        Line::from(format!("{spinner} working")),
        Rect::new(inner.x, inner.y, inner.width, 1),
    );
    for (i, body) in DIALOG_BODY.iter().enumerate() {
        let row = inner.y + 1 + i as u16;
        if row >= inner.bottom() {
            break;
        }
        frame.render_widget(Line::from(*body), Rect::new(inner.x, row, inner.width, 1));
    }
}

/// The one function the whole arm turns on: frame `n` of `scene`, and nothing else decides it.
fn draw(scene: Scene, frame: &mut Frame, n: u64) {
    match scene {
        Scene::Caret | Scene::Cpu => draw_caret(frame, n),
        Scene::StatusLine => draw_status_scene(frame, &format!("frame {n}"), n as f64 / 60.0),
        Scene::ListScroll => draw_list(frame, n as usize, 12),
        Scene::FullRepaint => {
            let area = frame.area();
            let shift = (n % area.width.max(1) as u64) as u16;
            frame.render_widget(Ramp { shift }, area);
        }
        Scene::ModalOverList => draw_modal_over_list(frame, n),
        Scene::Latency => draw_status_scene(frame, "frame 0", 0.0),
    }
}

// -------------------------------------------------------------------------------------------------
// Keys, for the latency scene
// -------------------------------------------------------------------------------------------------

/// The minimal decode the latency scene needs: printable ASCII, `\x1b[A`, `\x03`.
///
/// A plain blocking `Read` on stdin, not crossterm's event stream — that wants a tty, and the
/// contract guarantees a pipe. Returns `None` at EOF.
fn read_byte(stdin: &mut impl Read) -> io::Result<Option<u8>> {
    let mut byte = [0u8; 1];
    Ok(if stdin.read(&mut byte)? == 0 {
        None
    } else {
        Some(byte[0])
    })
}

fn read_key(stdin: &mut impl Read) -> io::Result<Option<String>> {
    let Some(first) = read_byte(stdin)? else {
        return Ok(None);
    };
    let name = match first {
        0x1b => match read_byte(stdin)? {
            Some(b'[') => match read_byte(stdin)? {
                Some(b'A') => "Up".to_string(),
                Some(b'B') => "Down".to_string(),
                Some(b'C') => "Right".to_string(),
                Some(b'D') => "Left".to_string(),
                _ => "Esc".to_string(),
            },
            _ => "Esc".to_string(),
        },
        0x03 => "C-c".to_string(),
        0x09 => "Tab".to_string(),
        b'\r' | b'\n' => "Enter".to_string(),
        0x7f => "Backspace".to_string(),
        b @ 0x20..=0x7e => (b as char).to_string(),
        b @ 0x01..=0x1a => format!("C-{}", (b + 0x60) as char),
        b => format!("0x{b:02x}"),
    };
    Ok(Some(name))
}

// -------------------------------------------------------------------------------------------------
// Wiring
// -------------------------------------------------------------------------------------------------

struct Args {
    scene: Scene,
    frames: u64,
    seconds: f64,
}

fn parse_args() -> Result<Args, String> {
    let mut scene = None;
    let mut frames = DEFAULT_FRAMES;
    let mut seconds = DEFAULT_SECONDS;
    let mut argv = std::env::args().skip(1);
    while let Some(arg) = argv.next() {
        match arg.as_str() {
            "--scene" => {
                let name = argv.next().ok_or("--scene needs a value")?;
                scene = Some(Scene::parse(&name).ok_or(format!("unknown scene `{name}`"))?);
            }
            "--frames" => {
                frames = argv
                    .next()
                    .ok_or("--frames needs a value")?
                    .parse()
                    .map_err(|_| "--frames needs an integer".to_string())?;
            }
            "--seconds" => {
                seconds = argv
                    .next()
                    .ok_or("--seconds needs a value")?
                    .parse()
                    .map_err(|_| "--seconds needs a number".to_string())?;
            }
            other => return Err(format!("unexpected argument `{other}`")),
        }
    }
    Ok(Args {
        scene: scene.ok_or("--scene is required")?,
        frames,
        seconds,
    })
}

/// The terminal size, and there is no other source of it.
///
/// `crossterm::terminal::size()` is *not* that source: it opens `/dev/tty` and so answers with the
/// developer's real window even when stdout is a pipe, which is the silent wrong answer the
/// contract warns about. See `NOTES.md`.
fn viewport_size() -> (u16, u16) {
    let read = |name: &str, default: u16| {
        std::env::var(name)
            .ok()
            .and_then(|v| v.trim().parse::<u16>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(default)
    };
    (
        read("COLUMNS", DEFAULT_WIDTH),
        read("LINES", DEFAULT_HEIGHT),
    )
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(args) => args,
        Err(message) => {
            eprintln!("arm-ratatui: {message}");
            return ExitCode::FAILURE;
        }
    };
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("arm-ratatui: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Args) -> io::Result<()> {
    let (width, height) = viewport_size();

    // The one line on stderr, before the first frame and before anything reaches stdout.
    eprintln!(
        "arm=ratatui version={RATATUI_VERSION} no_color={NO_COLOR} alt_screen={ALT_SCREEN}"
    );

    // `ratatui::init()` enables raw mode and enters the alternate screen. Raw mode is deliberately
    // skipped: crossterm implements it against `/dev/tty`, so calling it here would reconfigure the
    // developer's real terminal while measuring a pipe. The alternate screen is kept, because it is
    // a cost every other arm pays and the harness discards the prologue only when it can see it.
    execute!(io::stdout(), EnterAlternateScreen)?;

    let writer = BufWriter::with_capacity(1 << 16, io::stdout());
    let backend = CrosstermBackend::new(writer);
    // `Viewport::Fixed` is the arrangement that imposes a size: it is the one viewport
    // `Terminal::with_options` and `Terminal::autoresize` never ask the backend to measure. At the
    // origin and at full size it emits exactly what `Viewport::Fullscreen` would.
    let mut terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, width, height)),
        },
    )?;

    let result = match args.scene {
        Scene::Latency => run_latency(&mut terminal),
        Scene::Cpu => run_cpu(&mut terminal, args.seconds),
        scene => run_frames(&mut terminal, scene, args.frames),
    };

    // Drain the arm's own buffer before the epilogue goes out on the raw handle.
    let flushed = terminal.backend_mut().flush();
    drop(terminal);
    execute!(io::stdout(), LeaveAlternateScreen)?;
    io::stdout().flush()?;

    result.and(flushed)
}

/// The five picture scenes: `N` frames, no sleeping, no clock.
fn run_frames(
    terminal: &mut Terminal<CrosstermBackend<impl Write>>,
    scene: Scene,
    frames: u64,
) -> io::Result<()> {
    for n in 0..frames {
        terminal.draw(|frame| draw(scene, frame, n))?;
    }
    Ok(())
}

/// `latency`: the initial screen, then one status-line update per keystroke, flushed immediately.
fn run_latency(terminal: &mut Terminal<CrosstermBackend<impl Write>>) -> io::Result<()> {
    terminal.draw(|frame| draw(Scene::Latency, frame, 0))?;
    let mut stdin = io::stdin().lock();
    while let Some(key) = read_key(&mut stdin)? {
        // `Terminal::draw` ends in `Backend::flush`, which flushes the writer. Nothing is held back.
        terminal.draw(|frame| draw_status_scene(frame, &format!("key {key}"), 0.0))?;
    }
    Ok(())
}

/// `cpu`: scene `caret` at 60 Hz for a fixed span of wall time. The one scene where sleeping is
/// the measurement rather than a corruption of it.
fn run_cpu(terminal: &mut Terminal<CrosstermBackend<impl Write>>, seconds: f64) -> io::Result<()> {
    const PERIOD: Duration = Duration::from_nanos(16_666_667);
    let start = Instant::now();
    let total = Duration::from_secs_f64(seconds.max(0.0));
    let mut n = 0u64;
    while start.elapsed() < total {
        terminal.draw(|frame| draw(Scene::Cpu, frame, n))?;
        n += 1;
        // Deadline computed from `start`, so a slow frame does not push the schedule out.
        let deadline = PERIOD * u32::try_from(n).unwrap_or(u32::MAX);
        if let Some(nap) = deadline.checked_sub(start.elapsed()) {
            std::thread::sleep(nap);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_line_frame_zero_is_the_literal_from_scenes_md() {
        let line = status_line("frame 0", 0.0, 120);
        let text = line.to_string();
        assert_eq!(
            &text[..48],
            " frame 0     elapsed 0.00s    cpu 12%    3 tasks"
        );
        assert_eq!(text.chars().count(), 120);
    }

    #[test]
    fn status_line_fields_after_the_first_never_move() {
        // The first field owns columns 0..13; `elapsed` starts at column 13 on every frame, which
        // is what keeps frames 1.. to a single short run of changed cells.
        let zero = status_line("frame 0", 0.0, 120);
        let last = status_line("frame 119", 119.0 / 60.0, 120);
        assert_eq!(&zero.to_string()[13..26], "elapsed 0.00s");
        assert_eq!(&last.to_string()[13..26], "elapsed 1.98s");
    }

    #[test]
    fn list_row_is_seven_plus_twenty_cells() {
        assert_eq!(list_row(12), "00012  item-00012----------");
        assert_eq!(list_row(12).chars().count(), 7 + LABEL_WIDTH);
    }

    #[test]
    fn spinner_is_a_pure_function_of_the_frame() {
        assert_eq!(SPINNER[0], '⠋');
        assert_eq!(SPINNER[(120u64 % SPINNER.len() as u64) as usize], '⠋');
    }
}
