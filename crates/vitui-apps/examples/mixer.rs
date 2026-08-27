//! **`mixer` — eight faders, a master, and a value that cannot reach its own maximum.**
//!
//! Components ticket 33's application, and the only consumer of
//! [`vitui_components::input::slider`] that is not a gate. Spec §14, §17.
//!
//! ```text
//! cargo run -p vitui-apps --example mixer            # the desk
//! cargo run -p vitui-apps --example mixer -- --probe # one headless frame, and what it cost
//! ```
//!
//! # What it is for, and `x` is the key to press
//!
//! §17 froze `slider` at Tier 3 for an **unmeasured mechanism**, and §14 measured it: a drag is
//! `Response::local` over `Response::rect` and nothing else. That half is what a pointer shows —
//! press a fader anywhere and it *jumps* there rather than starting a delta, drag it and it follows,
//! release it and nothing moves.
//!
//! The half a pointer cannot show is the **step**, and `x` is what shows it. `x` presses `Right`
//! fifty times on the focused fader in one keystroke, and `f` swaps the stepping arm underneath it.
//! On the shipped arm the fader lands on exactly `0.5` and its thumb on the middle cell of the track.
//! On [`defective::float_stepped`] it lands on `0.4999998` and its thumb **one cell short of the
//! middle** — and then `X` (fifty more) takes it to `0.99999934`, where the thumb is in exactly the
//! right place and the value has never reached its own maximum. The status bar prints the value to
//! nine digits beside the thumb's column, because *the two defects hide each other* is only legible
//! when both are on the screen at once.
//!
//! # The band is criterion 4 on the screen
//!
//! The bottom pair is a **range** — a low and a high over one span — drawn as two sliders and driven
//! by [`defective::Range`], the spelling that stores nothing and derives its boundary from the two
//! values it separates. Drag the low thumb rightwards past the midpoint and watch the **high** thumb
//! jump back to meet the pointer, in a gesture that never released. That is why no range slider
//! ships: the fact it needs — *which thumb* — is the fifth cross-frame fact §14's headline is that
//! drag capture does not need.
//!
//! # Keys
//!
//! | key | what |
//! |---|---|
//! | `Tab` | the next fader — the ring's key, never the widget's |
//! | `←` `→` `↑` `↓` | one step. **`↑` is more at both orientations**, which is not `nav::step`'s pairing |
//! | `PageUp` `PageDown` | ten steps |
//! | `Home` `End` | 0 and 1, exactly, on both stepping arms |
//! | `x` · `X` | fifty `Right`s on the focused fader — the drift, in one keystroke |
//! | `f` | swap the stepping arm: the integer grid or the `f32` step |
//! | `r` | reset the desk |
//! | `q` · `Ctrl+Q` · `Esc` | quit |
//!
//! # `q` *is* a quit key here, and it is the first application on this map where that is true
//!
//! `compose` cannot bind `q` because a focused `field` consumes every text-bearing key into its
//! buffer, and `ledger` and `explorer` cannot because a focused `collection` eats it into a
//! type-ahead buffer. A focused `slider` takes **cursor keys and nothing else** — `stepped` returns
//! `None` for every other code, so the key is declined and reaches `Driver::unhandled`. So `q`
//! works, `f`, `x` and `r` work, and `Ctrl+Q` is bound beside `q` only because a terminal can take a
//! bare letter away and cannot take that.
//!
//! # The faders are drawn in a loop, so the caller keys them
//!
//! `Ctx::id` mints from `Location::caller()` and there is one call site inside the loop below, so
//! without `cx.with_key` all eight faders would be **one** widget: the first would be draggable and
//! the other seven inert, with every thumb in the right place and the screen looking perfect. That is
//! ADR 0027's defect, and `crate::media`'s chrome shipped it twice.

use std::env;

use vitui_components::counters::Tally;
use vitui_components::ink::{Direct, Ink};
use vitui_components::input::{SliderOpts, defective, slider_into};
use vitui_components::scroll::Orient;
use vitui_components::structure::{PanelOpts, panel_into};
use vitui_components::text::{Justify, TextOpts, text_into};
use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{Code, Pressed};
use vitui_runtime::layout::rect;
use vitui_runtime::work::Wake;
use vitui_runtime::{Ctx, Id, Mods, Rect, Role, Themes};

/// The eight channels. Names short enough for a two-column-wide fader's label.
const CHANNELS: [&str; 8] = ["kick", "snare", "hat", "bass", "gtr", "keys", "vox", "fx"];

/// Where each channel starts, so the desk does not open as a flat line.
const OPENING: [f32; 8] = [0.8, 0.65, 0.4, 0.75, 0.5, 0.35, 0.9, 0.15];

/// How wide one vertical fader's column is, label included.
const FADER_W: u16 = 7;

/// How many rows the master, the band and the status bar take between them. **Nine, and it was
/// eight**: three plus three plus three, and with eight the status band was two rows, so the third
/// `split_at_v` handed the key legend a **zero-row rectangle** and it was never drawn. A component
/// handed an empty rectangle returns without writing a cell — which is correct, and is exactly why
/// the mistake is silent.
const CHROME_H: u16 = 9;

/// How many `Right`s `x` presses. Fifty, because that is where the drift is one cell.
const BURST: u32 = 50;

/// Everything this application knows.
struct App {
    /// The eight channel faders, `0.0..=1.0`. **A fraction and not a decibel** — a component that
    /// owned a unit would be deciding what the number means, and this application's own `-inf..0 dB`
    /// label is where that decision belongs.
    channels: [f32; 8],
    /// The master, drawn horizontally so both orientations are on one screen.
    master: f32,
    /// The band: a low and a high, driven by the spelling that stores nothing.
    band: defective::Range,
    /// Which stepping arm the faders are on. Swapped by `f`.
    float: bool,
    /// Set inside `take_unhandled`, read by the loop.
    exit: bool,
    /// How many `Right`s to deliver to the focused fader on the next frame, from `x`.
    burst: u32,
    /// What the frame that has just drawn reported.
    seen: Seen,
    /// Whether `--probe` asked for the counters.
    counters: bool,
}

/// What the status bar prints, gathered during the draw.
#[derive(Clone, Copy, Default)]
struct Seen {
    /// The value of whichever fader holds the keyboard.
    value: f32,
    /// Which of [`CHANNELS`] it is, or `None` for the master or the band.
    which: Option<usize>,
    /// The cell the thumb landed on, and how many the track has.
    thumb: (u16, u16),
    /// How many regions the frame declared.
    regions: usize,
    /// How many tab stops it declared.
    stops: usize,
    /// What the band's two values are, and which one the last drag moved.
    band_moved: Option<&'static str>,
    /// Cells written. `--probe` only.
    writes: u64,
    /// How many of them were distinct. **The partition equality, and it is a number rather than an
    /// argument** — this application assembles its own rectangles, so §2's rule is its to keep.
    distinct: u64,
    /// How many verbs it took.
    verbs: u64,
}

impl App {
    /// The desk as it opens.
    fn new() -> App {
        App {
            channels: OPENING,
            master: 0.7,
            band: defective::Range {
                low: 0.2,
                high: 0.8,
            },
            float: false,
            exit: false,
            burst: 0,
            seen: Seen::default(),
            counters: false,
        }
    }

    /// The options both stepping arms share.
    fn opts(&self, orient: Orient) -> SliderOpts {
        SliderOpts {
            orient,
            ..SliderOpts::default()
        }
    }

    /// One frame, through whichever ink `--probe` chose.
    ///
    /// **One call site and one type.** `&mut dyn Ink` satisfies `I: Ink` through the crate's own
    /// `impl<T: Ink + ?Sized> Ink for &mut T`, so the counters are a runtime choice rather than a
    /// second copy of the draw — and a second copy is two `Location::caller()`s, therefore two ids,
    /// therefore a screen that looks identical and loses the focus when the counters are toggled.
    fn ui(&mut self, cx: &mut Ctx<'_, '_>) {
        let mut tally = Tally::new();
        let mut direct = Direct;
        let mut sink: &mut dyn Ink = if self.counters {
            &mut tally
        } else {
            &mut direct
        };
        self.draw(&mut sink, cx);
        if self.counters {
            self.seen.writes = tally.writes();
            self.seen.distinct = tally.distinct();
            self.seen.verbs = tally.verbs();
        }
    }

    /// One frame, top to bottom.
    fn draw<I: Ink>(&mut self, ink: &mut I, cx: &mut Ctx<'_, '_>) {
        let title = if self.float {
            " mixer — the f32 step (press f for the grid) "
        } else {
            " mixer — the integer grid (press f for the f32 step) "
        };
        let block = panel_into(ink, cx, cx.area(), title, &PanelOpts::default());
        let interior = block.interior;
        if interior.h < CHROME_H + 3 || interior.w < FADER_W + 5 {
            text_into(
                ink,
                cx,
                interior,
                "the desk needs a taller terminal",
                &TextOpts {
                    justify: Justify::Middle,
                    role: Role::Warn,
                    ..Default::default()
                },
            );
            return;
        }

        // Four bands, all of them the caller's arithmetic: the components narrow nothing on their
        // behalf, which is `CONTEXT.md`'s identity rule and why every one of them returns a
        // rectangle or a `Response` rather than taking a closure.
        let (desk, rest) = rect::split_at_v(interior, interior.h - CHROME_H);
        let (master_band, rest) = rect::split_at_v(rest, 3);
        let (band, status) = rect::split_at_v(rest, 3);

        let mut focused_value = None;
        let mut thumb = (0u16, 0u16);
        let mut first = None;
        // Read once, before anything borrows the desk's values mutably.
        let float = self.float;

        // ── the eight channel faders, vertical, up is more ───────────────────────────────────────
        //
        // **One call site and eight widgets**, which is what `cx.with_key` buys. See this file's
        // header for what its absence costs and why the screen would not show it.
        let (label_row, faders) = rect::split_at_v(desk, 1);
        let mut drawn = 0u16;
        for (i, name) in CHANNELS.iter().enumerate() {
            let x = desk.x + i32::from(FADER_W) * i as i32;
            if x + i32::from(FADER_W) > desk.right() {
                break;
            }
            drawn += 1;
            text_into(
                ink,
                cx,
                Rect::new(x, label_row.y, FADER_W, 1),
                name,
                &TextOpts {
                    justify: Justify::Middle,
                    role: Role::Title,
                    ..Default::default()
                },
            );
            // **The channel's column is partitioned, not sampled.** A three-cell track inside a
            // seven-cell column leaves two gutters either side, and a first draft drew the track and
            // the dB row and left them alone: four columns a channel that nobody writes, which keep
            // whatever the previous screen put there for ever. Spec §2's rule is a component's, and
            // an application that assembles rectangles owes the same equality over the ones it made
            // up — which is the shape three of this crate's tickets found in their own applications.
            let column = Rect::new(x, faders.y, FADER_W, faders.h);
            let (body, foot) = rect::split_at_v(column, column.h.saturating_sub(1));
            let (left, rest) = rect::split_at_h(body, 2);
            let (track, right) = rect::split_at_h(rest, 3);
            let blank = TextOpts::default();
            text_into(ink, cx, left, "", &blank);
            text_into(ink, cx, right, "", &blank);
            let id = cx.with_key(i as u64, |cx| {
                fader(
                    ink,
                    float,
                    cx,
                    track,
                    &mut self.channels[i],
                    Orient::Vertical,
                )
            });
            if first.is_none() {
                first = Some(id);
            }
            if cx.is_focused(id) {
                focused_value = Some((self.channels[i], Some(i)));
                thumb = (thumb_cell(self.channels[i], track.h), track.h);
            }
            // The dB reading under each fader. **The unit is the application's**, which is the whole
            // reason the component's value is a bare fraction.
            text_into(
                ink,
                cx,
                foot,
                &db(self.channels[i]),
                &TextOpts {
                    justify: Justify::Middle,
                    role: Role::Dim,
                    ..Default::default()
                },
            );
        }
        // The columns the faders did not reach, so the desk band is a partition of itself.
        //
        // **`drawn` and not `CHANNELS.len()`.** A first draft assumed all eight columns fitted, so on
        // a terminal too narrow for the desk the loop broke early *and* the tail was computed as
        // though it had not — leaving the skipped channels' columns written by nobody. The hole only
        // appears below 61 columns, which is why a screenshot at any comfortable size shows nothing.
        let used = drawn * FADER_W;
        if used < desk.w {
            let tail = Rect::new(desk.x + i32::from(used), desk.y, desk.w - used, desk.h);
            text_into(ink, cx, tail, "", &TextOpts::default());
        }

        // ── the master, horizontal ───────────────────────────────────────────────────────────────
        let (master_label, master_track) = rect::split_at_h(master_band, 10);
        text_into(
            ink,
            cx,
            master_label,
            "master",
            &TextOpts {
                role: Role::Title,
                ..Default::default()
            },
        );
        let track = rect::split_at_v(master_track, 1).0;
        let master_id = fader(ink, float, cx, track, &mut self.master, Orient::Horizontal);
        if cx.is_focused(master_id) {
            focused_value = Some((self.master, None));
            thumb = (thumb_cell(self.master, track.w), track.w);
        }
        // The two rows of the master band the track did not take.
        let (_, master_rest) = rect::split_at_v(master_track, 1);
        text_into(ink, cx, master_rest, "", &TextOpts::default());

        // ── the band: criterion 4, on the screen ─────────────────────────────────────────────────
        let (band_label, band_tracks) = rect::split_at_h(band, 10);
        text_into(
            ink,
            cx,
            band_label,
            "band",
            &TextOpts {
                role: Role::Title,
                ..Default::default()
            },
        );
        let (low_row, rest) = rect::split_at_v(band_tracks, 1);
        let (high_row, spare) = rect::split_at_v(rest, 1);
        // **Two sliders over one span, and the boundary is derived rather than stored.** Each thumb
        // is dragged through its own component; what makes this the *refused* spelling is that a
        // press is routed to whichever thumb the midpoint says, and the midpoint moves with them.
        let before = (self.band.low, self.band.high);
        let mut low = self.band.low;
        let mut high = self.band.high;
        let opts = self.opts(Orient::Horizontal);
        let low_resp = slider_into(ink, cx, low_row, &mut low, &opts);
        let high_resp = slider_into(ink, cx, high_row, &mut high, &opts);
        // Whichever one the pointer moved is routed through `Range::drag`, which is the spelling
        // under test: it decides *which* thumb from the midpoint alone.
        if low_resp.changed && low != before.0 {
            self.seen.band_moved = Some(match self.band.drag(low) {
                defective::Which::Low => "low",
                defective::Which::High => "high (nobody touched it)",
            });
        } else if high_resp.changed && high != before.1 {
            self.seen.band_moved = Some(match self.band.drag(high) {
                defective::Which::Low => "low (nobody touched it)",
                defective::Which::High => "high",
            });
        }
        text_into(ink, cx, spare, "", &TextOpts::default());

        // **Seat the keyboard on the first fader while nothing holds it** — architecture issue 25's
        // one line inside the draw. `focused().is_none()` and never `!is_focused(id)`, because the
        // second drags the keyboard back every frame the user has tabbed away and `Tab` then appears
        // to do nothing.
        if cx.focused().is_none()
            && let Some(id) = first
        {
            cx.focus(id);
        }

        // **The burst is delivered here**, after the draw that told us which fader holds the
        // keyboard, and it is the application's own arithmetic rather than fifty posted keys: what
        // `x` is demonstrating is `stepped`, and posting keys would demonstrate the key queue.
        if self.burst > 0 {
            self.apply_burst(
                focused_value.and_then(|(_, which)| which),
                cx.focused(),
                master_id,
            );
        }

        let (value, which) = focused_value.unwrap_or((0.0, None));
        self.seen.value = value;
        self.seen.which = which;
        self.seen.thumb = thumb;
        self.status(ink, cx, status);
    }

    /// Fifty `Right`s on whichever fader holds the keyboard.
    ///
    /// Applied through [`vitui_components::input::stepped`] and its refused twin, which is the pair
    /// the burst exists to contrast. Nothing here posts a key: `x` is one keystroke and fifty steps,
    /// and the two are different things to measure.
    fn apply_burst(&mut self, channel: Option<usize>, focused: Option<Id>, master: Id) {
        let n = std::mem::take(&mut self.burst);
        // **The orientation is not read here**, and that is `stepped`'s own note: `Up` is paired with
        // `Right` at both orientations, so there is no orientation branch in the keyboard at all.
        let opts = SliderOpts::default();
        let key = vitui_components::keys::press_with(Code::Right, Mods::NONE);
        let target: &mut f32 = match channel {
            Some(i) => &mut self.channels[i],
            None if focused == Some(master) => &mut self.master,
            None => return,
        };
        for _ in 0..n {
            let next = if self.float {
                defective::float_stepped(&key, *target, &opts)
            } else {
                vitui_components::input::stepped(&key, *target, &opts)
            };
            match next {
                Some(v) => *target = v,
                None => break,
            }
        }
    }

    /// The three status rows. A report of the frame that has just been drawn.
    fn status<I: Ink>(&self, ink: &mut I, cx: &mut Ctx<'_, '_>, at: Rect) {
        let s = self.seen;
        let dim = TextOpts {
            role: Role::Dim,
            ..Default::default()
        };
        let (first, rest) = rect::split_at_v(at, 1);
        let (second, third) = rect::split_at_v(rest, 1);
        let name = match s.which {
            Some(i) => CHANNELS[i],
            None => "master",
        };
        // **Nine digits, beside the cell.** The drift is in the last three and the thumb is what a
        // person sees, so a bar that printed one of them would be showing exactly the half that
        // makes the other invisible.
        text_into(
            ink,
            cx,
            first,
            &format!(
                "{name}  value {:.9}  thumb cell {} of {}  step {}  {}",
                s.value,
                s.thumb.0,
                s.thumb.1.saturating_sub(1),
                if self.float { "f32 (refused)" } else { "grid" },
                if s.value == 1.0 {
                    "at maximum"
                } else if s.value > 0.999 {
                    "NOT at maximum"
                } else {
                    ""
                },
            ),
            &TextOpts {
                role: if self.float { Role::Warn } else { Role::Dim },
                ..Default::default()
            },
        );
        text_into(
            ink,
            cx,
            second,
            &format!(
                "band low {:.3} high {:.3}{}",
                self.band.low,
                self.band.high,
                match s.band_moved {
                    Some(w) => format!("   last drag moved: {w}"),
                    None => String::new(),
                },
            ),
            &TextOpts {
                role: match s.band_moved {
                    Some(w) if w.contains("nobody") => Role::Danger,
                    _ => Role::Dim,
                },
                ..Default::default()
            },
        );
        text_into(
            ink,
            cx,
            third,
            "Tab fader · arrows step (up is more) · x fifty rights · f swap the step · r reset · q quit",
            &dim,
        );
    }

    /// What the frame that has just drawn declared. `--probe` reads it; the desk does not.
    fn read_frame(&mut self, driver: &Driver) {
        if !self.counters {
            return;
        }
        self.seen.regions = driver.inspect().hits().len();
        self.seen.stops = driver.inspect().stop_count();
    }

    /// **The application's own keys, read from what nothing wanted.** See this file's header.
    fn take_unhandled(&mut self, keys: &[Pressed]) -> bool {
        let mut moved = false;
        for k in keys {
            match k.code {
                Code::Escape => self.exit = true,
                Code::Char('q') if k.mods.chord().contains(Mods::CTRL) => self.exit = true,
                Code::Char('q') => self.exit = true,
                Code::Char('f') => {
                    self.float = !self.float;
                    moved = true;
                }
                Code::Char('r') => {
                    self.channels = OPENING;
                    self.master = 0.7;
                    self.band = defective::Range {
                        low: 0.2,
                        high: 0.8,
                    };
                    self.seen.band_moved = None;
                    moved = true;
                }
                Code::Char('x' | 'X') => {
                    self.burst = BURST;
                    moved = true;
                }
                _ => {}
            }
        }
        moved
    }
}

/// One slider, on whichever stepping arm `f` last chose.
///
/// **One function with one `bool` between the shipped build and the refused one**, which is this
/// crate's own arrangement for a spelling it keeps runnable: a reviewer's diff between the two is a
/// field rather than a second body.
///
/// **A free function and not a method**, because a method taking `&self` cannot also be handed
/// `&mut self.channels[i]` — `E0502`, and the fix is the ordinary one rather than a `clone`: the two
/// things it needs from the application are a `bool` and an `Orient`.
fn fader<I: Ink>(
    ink: &mut I,
    float: bool,
    cx: &mut Ctx<'_, '_>,
    at: Rect,
    value: &mut f32,
    orient: Orient,
) -> Id {
    let opts = SliderOpts {
        orient,
        ..SliderOpts::default()
    };
    if float {
        defective::float_stepped_into(ink, cx, at, value, &opts).id
    } else {
        slider_into(ink, cx, at, value, &opts).id
    }
}

/// A fraction as a level a mixing desk would print. **The application's unit, not the component's.**
fn db(v: f32) -> String {
    if v <= 0.0 {
        return "-inf".to_string();
    }
    format!("{:+.0}", 20.0 * v.log10())
}

/// Which cell of a track of `length` a value's thumb sits on.
///
/// The same arithmetic the component draws with, restated here because the status bar wants the
/// number and nothing on `Response` carries it. **A reason to publish it rather than a reason to copy
/// it** — filed in this ticket's Answer as the one thing the surface is missing.
fn thumb_cell(value: f32, length: u16) -> u16 {
    let span = i32::from(length).max(1) - 1;
    let span = span.max(1);
    let at = (value.clamp(0.0, 1.0) * span as f32).round() as u16;
    at.min(length.saturating_sub(1))
}

/// **One headless frame, and what it cost.** See the module header's `--probe`.
///
/// Two frames and not one: the frame structures take their allocation on the first that needs one,
/// so a cold frame is not a frame.
fn probe(app: &mut App) {
    const W: u16 = 100;
    const H: u16 = 30;
    let mut driver = match Driver::headless(W, H) {
        Ok(driver) => driver,
        Err(why) => {
            eprintln!("a headless sink could not attach: {why}");
            return;
        }
    };
    app.counters = true;
    driver.frame(|cx| app.ui(cx));
    driver.frame(|cx| app.ui(cx));
    app.read_frame(&driver);
    println!("mixer, one frame at {W}x{H}:");
    println!(
        "  regions      {:>8}   the panel, eight faders, the master and the band's two. Every one \
         of them is a slider or the panel: a label declares nothing",
        app.seen.regions
    );
    println!(
        "  tab stops    {:>8}   `Interest::FOCUS` is in the default, and it costs no tracking",
        app.seen.stops
    );
    let cells = u64::from(W) * u64::from(H);
    println!("  writes       {:>8}   of {cells}", app.seen.writes);
    println!(
        "  distinct     {:>8}   equal to `writes` and to every cell of the panel's interior: an \
         application that assembles its own rectangles owes section 2's partition too, and a first \
         draft left four columns a channel to whatever was there before",
        app.seen.distinct
    );
    println!("  verbs        {:>8}", app.seen.verbs);

    // The two arms of the step, fifty presses each, side by side — which is what `x` shows on the
    // desk and what §14's arithmetic finding is.
    let opts = SliderOpts::default();
    let key = vitui_components::keys::press_with(Code::Right, Mods::NONE);
    let mut grid = 0.0f32;
    let mut float = 0.0f32;
    for _ in 0..BURST {
        grid = vitui_components::input::stepped(&key, grid, &opts).expect("a cursor key");
        float = defective::float_stepped(&key, float, &opts).expect("a cursor key");
    }
    println!(
        "  grid  x50    {grid:>8.7}   cell {} of 299",
        thumb_cell(grid, 300)
    );
    println!(
        "  f32   x50    {float:>8.7}   cell {} of 299 — one short of the middle",
        thumb_cell(float, 300)
    );
    for _ in 0..BURST {
        grid = vitui_components::input::stepped(&key, grid, &opts).expect("a cursor key");
        float = defective::float_stepped(&key, float, &opts).expect("a cursor key");
    }
    println!("  grid  x100   {grid:>8.7}   at maximum");
    println!(
        "  f32   x100   {float:>8.7}   cell {} of 299 — the right cell, and never the maximum",
        thumb_cell(float, 300)
    );

    // And the band, driven the way a held drag drives it.
    let mut band = defective::Range {
        low: 0.2,
        high: 0.8,
    };
    let first = band.drag(0.3);
    let second = band.drag(0.6);
    println!(
        "  band drag    {:>8}   one gesture: {first:?} then {second:?}, leaving ({:.1}, {:.1}) — \
         the high thumb moved 0.2 and nobody touched it",
        "0.3->0.6", band.low, band.high
    );
}

fn main() {
    let mut app = App::new();
    if env::args().any(|a| a == "--probe") {
        probe(&mut app);
        return;
    }

    let mut driver = match Driver::attach(Default::default(), *Themes::standard().theme()) {
        Ok(driver) => driver,
        Err(why) => {
            eprintln!("vitui could not attach: {why}");
            return;
        }
    };

    loop {
        driver.frame(|cx| app.ui(cx));
        // **Read immediately after this application's own frame and never before it.** The window is
        // valid until the next frame begins, so a loop that read it before its own frame acted on
        // the previous frame's window — one wake late, which for a single keystroke means never.
        let unhandled: Vec<Pressed> = driver.unhandled().to_vec();
        let moved = app.take_unhandled(&unhandled);
        if app.exit {
            break;
        }
        // Something arrived, so the next frame is drawn without parking: the state the keys changed
        // is only on the screen once it has been drawn again.
        if moved || !unhandled.is_empty() {
            continue;
        }
        match driver.wait() {
            Wake::Quit => break,
            Wake::Input | Wake::Posted | Wake::Deadline => {}
        }
    }
}
