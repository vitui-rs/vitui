//! **`roster` — the three Tier 2 composites, and `Ctrl+G` is the key to press.**
//!
//! The form's application, and the only consumer of
//! [`form`](vitui_components::input::form), [`pagination`](vitui_components::collect::pagination)
//! and [`status_bar`](vitui_components::structure::status_bar) that is not a gate.
//!
//! ```text
//! cargo run -p vitui-apps --example roster            # the directory
//! cargo run -p vitui-apps --example roster -- --probe # one headless frame, and what it cost
//! ```
//!
//! # What it is for, and `Ctrl+G` is the key to press
//!
//! A composition is *shipped components with no new mechanism*, and this one is called
//! **the class that requires the most care** — because *composition without a new mechanism is
//! exactly the claim that turns out to be false when it is false.* What a screen can show that a
//! gate cannot is the **keyboard**, and `Ctrl+G` is where it is visible: it takes the form's `Group`
//! scope away, and three things happen at once.
//!
//! - The status bar's `stops` jumps from **1 to 6**. A `Group` collapses the walk onto its first
//!   entry and the rest stay in the ring, which is *a list is one tab stop* over fields.
//! - `Tab` starts walking the fields instead of leaving the form.
//! - **The arrows stop working entirely.** That is not a preference: a container hears what its
//!   children hand back only through `Ctx::scope`'s after-the-body moment, and `ScopeKind` has three
//!   arms of which the other two are a modal's `Trap` and the code editor's `Isolated`. There is no
//!   *not-a-group* scope, so a form that is not one tab stop is a form with no `nav::cursor` at all.
//!
//! # `Ctrl+D` is where density stops being theme data and starts being two fields
//!
//! *Density is theme data, it changes rectangles, and `block` is where that lands* — one
//! cell of padding at `Compact` and two at `Cosy`, on both edges, which is two rows of interior. Two
//! rows of a one-row entry is **two fields**, and the status bar prints *standing / of* so the
//! number that falls off the bottom is on the screen rather than in a comment.
//!
//! # `Ctrl+F` is where a band's axis argument stops being decoration
//!
//! The status bar is `crate::scroll::sticky`'s construction with the axis argument taken verbatim,
//! and at `Fill::Even` its segments are laid out over the width it can see — so there is nothing to
//! scroll and the shared offset moves nothing. `Ctrl+F` puts it at `Fill::Natural`, where the
//! content is as wide as the text; `Ctrl+←`/`Ctrl+→` then move the offset it shares with the body,
//! and the bar scrolls under its own rectangle rather than over its neighbour's. The clip is the
//! whole of what a band buys, and it is why one of these is a component and four of them are one
//! function.
//!
//! # Keys
//!
//! | key | what |
//! |---|---|
//! | any character | into the focused field — which is what a field is *for* |
//! | `↑` `↓` | the next field, through `nav::cursor` — grouped only |
//! | `Tab` | the ring's key, never the widget's |
//! | `←` `→` | the previous or next page, when the pager holds the keyboard |
//! | a digit | type-ahead over the page numbers, one second's worth |
//! | `Ctrl+G` | take the form's `Group` scope away, and its arrows with it |
//! | `Ctrl+D` | `Compact` against `Cosy` |
//! | `Ctrl+F` | the status bar's fill: even against natural |
//! | `Ctrl+←` `Ctrl+→` | the offset the status bar shares with the body |
//! | `Ctrl+Q` · `Esc` | quit |
//!
//! # `q` is not a quit key here, and that is the third time on this map
//!
//! A focused [`field`](vitui_components::input::field) consumes **every text-bearing key** into its
//! buffer, which is what `keys::text` is for and what `compose` found first. So there is no `q` to
//! bind: typing one is the point. `Esc` survives — a field declines it — and `Ctrl+Q` is bound
//! beside it, which is `ledger`'s established arrangement for a binding a terminal can take away.
//!
//! # The three components are reached through their `_into` spellings
//!
//! `--probe` chooses its [`Ink`] at run time, and a second call site is a second
//! `Location::caller()`, therefore a second id, therefore a screen that looks identical and loses
//! its focus the moment the counters are on. `vitals` and `mixer` say the same thing.

use std::env;

use vitui_components::collect::{CollState, PageOpts, pagination_into};
use vitui_components::counters::Tally;
use vitui_components::edit::Text;
use vitui_components::ink::{Direct, Ink};
use vitui_components::input::{FormOpts, FormState, form_into, form_row_id};
use vitui_components::structure::{Fill, PanelOpts, StatusOpts, panel_into, status_bar_into};
use vitui_components::text::{Justify, TextOpts, text_into};
use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{Code, Pressed};
use vitui_runtime::layout::rect;
use vitui_runtime::work::Wake;
use vitui_runtime::{Ctx, Density, Mods, Role, Themes};

/// The record's fields, in draw order. Six, because two of them fall off the bottom at `Cosy` in a
/// panel this size and the point of `Ctrl+D` is that you can watch it happen.
const LABELS: [&str; 6] = ["name", "email", "role", "team", "location", "pronouns"];

/// How many records the directory holds. A hundred and thirty-seven, so the pager's window is
/// narrower than its content at every terminal width anybody uses.
const RECORDS: usize = 137;

/// How wide one page cell is. Three columns fits `137` with a cell of padding either side.
const PAGE_CELL: u16 = 5;

/// How many rows the pager's strip takes.
const PAGER_H: u16 = 1;

/// How many rows the status bar takes.
const STATUS_H: u16 = 1;

/// **The smallest interior the form draws in.** A caption row and one row a field, plus the pager
/// and the bar — below it the screen says so rather than drawing a form with nothing in it.
const FLOOR_H: u16 = LABELS.len() as u16 + 4;

/// Everything this application knows.
struct App {
    /// The six fields of the record on screen. **The widgets' own values, never application data** —
    /// The widgets' own state, and the reason `form` takes them by `&mut [Text]`.
    fields: [Text; 6],
    /// The form's one cross-frame fact, which is `nav::cursor`'s type-ahead buffer and nothing else.
    form: FormState,
    /// The pager's store. `collection`'s own, at `Mode::Options` — there is no second one.
    pages: CollState,
    /// Whether the form is one tab stop. Toggled by `Ctrl+G`.
    grouped: bool,
    /// Whether the status bar's segments are as wide as they measure. Toggled by `Ctrl+F`.
    natural: bool,
    /// The offset the status bar shares with the body it is a bar of. Moved by `Ctrl+←`/`Ctrl+→`.
    offset: i32,
    /// Which density the theme is built at. Toggled by `Ctrl+D`.
    cosy: bool,
    /// Set inside `take_unhandled`, read by the loop.
    exit: bool,
    /// Whether the density changed and the theme owes a rebuild between frames.
    density_moved: bool,
    /// What the frame that has just drawn reported.
    seen: Seen,
    /// Whether `--probe` asked for the counters.
    counters: bool,
    /// Whether the last frame had room to draw the form at all, rather than the too-small message.
    drew_a_form: bool,
    /// **The bar's four segments, and it is a field rather than a local for a measured reason.**
    ///
    /// `status_bar` takes `&[&str]`, so a screen formats its own segments — and four `String`s built
    /// inside the draw are **four allocations a frame** against a budget of zero, invisible to every
    /// counter because the picture is identical. As a field the capacity survives, `write` clears
    /// rather than reallocates, and the steady frame is free. That is `crate::media::player`'s
    /// chapter list and `rule`'s caption for the fourth time on this map.
    bar: Segments,
}

/// What the status bar prints, gathered during the draw.
#[derive(Clone, Copy, Default)]
struct Seen {
    /// How many fields the form stood, which at `Cosy` is fewer than [`LABELS`] has.
    standing: usize,
    /// How many regions the frame declared. **One for the bar, one for the pager, one a field** —
    /// and never one per segment or one per page, which is what makes this number worth printing.
    regions: usize,
    /// How many tab stops it declared. **2 grouped and `standing + 1` ungrouped**: the form is one
    /// stop or every field, and the pager is one either way.
    stops: usize,
    /// Which page is current, one-based.
    page: usize,
    /// Cells written. `--probe` only.
    writes: u64,
    /// How many of them were distinct. **The partition equality as a number** — this application
    /// assembles its own rectangles, so the partition rule is its to keep, and four of this crate's applications
    /// found the hole in their own application rather than in a component.
    distinct: u64,
    /// How many verbs it took.
    verbs: u64,
}

impl App {
    /// The directory as it opens.
    fn new() -> App {
        let mut fields = [
            Text::input(),
            Text::input(),
            Text::input(),
            Text::input(),
            Text::input(),
            Text::input(),
        ];
        for (f, seed) in fields.iter_mut().zip([
            "ada lovelace",
            "ada@analytical.engine",
            "engineer",
            "notes",
            "london",
            "she/her",
        ]) {
            f.insert(40, seed);
        }
        App {
            fields,
            form: FormState::new(),
            pages: CollState::new(),
            grouped: true,
            natural: false,
            offset: 0,
            cosy: false,
            exit: false,
            density_moved: false,
            seen: Seen::default(),
            counters: false,
            drew_a_form: false,
            bar: Segments::default(),
        }
    }

    /// One frame, through whichever ink `--probe` chose.
    ///
    /// **One call site and one type.** `&mut dyn Ink` satisfies `I: Ink` through the components
    /// crate's own `impl<T: Ink + ?Sized> Ink for &mut T`, so the counters are a run-time choice
    /// rather than a second copy of the draw — and a second copy is two `Location::caller()`s and
    /// therefore two forms.
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

    /// The screen: a panel with the record in it, the pager under it, and the bar along the bottom.
    fn draw<I: Ink>(&mut self, ink: &mut I, cx: &mut Ctx<'_, '_>) {
        let screen = cx.area();
        self.drew_a_form = screen.h >= FLOOR_H && screen.w >= 24;
        let (body, rest) = rect::split_at_v(screen, screen.h.saturating_sub(STATUS_H));
        if !self.drew_a_form {
            text_into(
                ink,
                cx,
                screen,
                "a roster needs 24 columns and ten rows",
                &TextOpts {
                    justify: Justify::Middle,
                    role: Role::Dim,
                    ..TextOpts::default()
                },
            );
            self.seen = Seen::default();
            return;
        }

        let (panel_at, strip) = rect::split_at_v(body, body.h.saturating_sub(PAGER_H));
        let panel = panel_into(
            ink,
            cx,
            panel_at,
            " record ",
            "",
            &PanelOpts {
                interest: vitui_runtime::Interest::HOVER,
                ..PanelOpts::default()
            },
        );
        let resp = form_into(
            ink,
            cx,
            panel.interior,
            &mut self.form,
            &LABELS,
            &mut self.fields,
            &FormOpts {
                group: self.grouped,
                ..FormOpts::default()
            },
        );
        // **Nothing holds the focus until an application says so** — and the
        // symptom is a program whose documented keys do nothing until `Tab`. `form_row_id` is the
        // only way to name a row from out here, because `Ctx::id` mints from `Location::caller()`.
        if cx.focused().is_none() {
            cx.focus(form_row_id(resp.id, 0));
        }

        pagination_into(
            ink,
            cx,
            strip,
            &mut self.pages,
            RECORDS,
            &PageOpts {
                cell: PAGE_CELL,
                ..PageOpts::default()
            },
        );

        // **The counts the bar prints are the previous frame's ring**, and they have to be: the
        // ring is built *by* the draw and the bar is drawn inside it, so a bar that asked how many
        // fields were standing would be asking before the answer exists. `read_frame` takes them
        // after `Driver::frame` returns, which is one frame of lag on three numbers and the honest
        // arrangement — `--probe` draws two frames for exactly this reason.
        self.seen.page = self.pages.sel.lead + 1;
        let bar = rect::split_at_v(rest, STATUS_H).0;
        let standing = self.seen.standing;
        self.bar.write(
            format_args!(
                " {} {}/{} ",
                if self.grouped { "group" } else { "flat" },
                standing,
                LABELS.len()
            ),
            format_args!(" page {}/{} ", self.seen.page.min(RECORDS), RECORDS),
            format_args!(
                " {} · off {} ",
                if self.natural { "natural" } else { "even" },
                self.offset
            ),
            format_args!(
                " {} · Ctrl+G group · Ctrl+D density · Ctrl+F fill · Ctrl+Q quit ",
                if self.cosy { "cosy" } else { "compact" }
            ),
        );
        status_bar_into(
            ink,
            cx,
            bar,
            &self.bar.parts(),
            (self.offset, 0),
            &StatusOpts {
                fill: if self.natural {
                    Fill::Natural
                } else {
                    Fill::Even
                },
                ..StatusOpts::default()
            },
        );
    }

    /// Read what the frame that has just drawn declared. Called by the loop, never inside the draw.
    fn read_frame(&mut self, driver: &Driver) {
        let frame = driver.inspect();
        self.seen.regions = frame.hits().len();
        self.seen.stops = frame.stop_count();
        // A field is one hit entry and so are the pager and the panel and the bar, so the fields
        // standing are what is left after the three that are not fields.
        self.seen.standing = self.seen.regions.saturating_sub(3);
    }

    /// The keys the frame handed back. **Read immediately after this application's own frame.**
    fn take_unhandled(&mut self, keys: &[Pressed]) -> bool {
        let mut moved = false;
        for k in keys {
            let ctrl = k.mods.chord().contains(Mods::CTRL);
            match k.code {
                Code::Escape => self.exit = true,
                Code::Char('q') if ctrl => self.exit = true,
                Code::Char('g') if ctrl => {
                    self.grouped = !self.grouped;
                    moved = true;
                }
                Code::Char('f') if ctrl => {
                    self.natural = !self.natural;
                    moved = true;
                }
                Code::Char('d') if ctrl => {
                    self.cosy = !self.cosy;
                    self.density_moved = true;
                    moved = true;
                }
                Code::Left if ctrl => {
                    self.offset = (self.offset - 4).max(0);
                    moved = true;
                }
                Code::Right if ctrl => {
                    self.offset += 4;
                    moved = true;
                }
                _ => {}
            }
        }
        moved
    }
}

/// **The bar's four segments, formatted into a buffer this application owns.**
///
/// See [`App::bar`] for why it is a field and not a local.
#[derive(Default)]
struct Segments {
    parts: [String; 4],
}

impl Segments {
    /// Format the four segments in place, reusing whatever capacity is already here.
    fn write(
        &mut self,
        a: std::fmt::Arguments<'_>,
        b: std::fmt::Arguments<'_>,
        c: std::fmt::Arguments<'_>,
        d: std::fmt::Arguments<'_>,
    ) {
        use std::fmt::Write;
        for (slot, args) in self.parts.iter_mut().zip([a, b, c, d]) {
            slot.clear();
            let _ = slot.write_fmt(args);
        }
    }

    /// The four, borrowed.
    fn parts(&self) -> [&str; 4] {
        [
            self.parts[0].as_str(),
            self.parts[1].as_str(),
            self.parts[2].as_str(),
            self.parts[3].as_str(),
        ]
    }
}

/// **One headless frame at 100×30, and what it cost.**
fn probe(app: &mut App) {
    app.counters = true;
    let (w, h) = (100u16, 30u16);
    let mut driver = Driver::headless(w, h).expect("a headless sink cannot fail to attach");
    // Two frames: the counts the bar prints are the previous frame's ring, so one frame reports a
    // form standing zero fields on a screen that drew six.
    driver.frame(|cx| app.ui(cx));
    app.read_frame(&driver);
    driver.frame(|cx| app.ui(cx));
    app.read_frame(&driver);

    println!("== roster, one frame at {w}x{h} ==\n");
    let cells = u64::from(w) * u64::from(h);
    println!("  writes       {:>8}", app.seen.writes);
    println!(
        "  distinct     {:>8}   of {cells} cells{}",
        app.seen.distinct,
        if app.seen.distinct == cells && app.seen.writes == cells {
            ""
        } else {
            "   ** not a partition **"
        }
    );
    println!("  verbs        {:>8}", app.seen.verbs);
    println!(
        "  regions      {:>8}   the panel, the pager, the bar and one a standing field",
        app.seen.regions
    );
    println!(
        "  stops        {:>8}   the form's one — because a `Group` is one tab stop (spec §3) — \
         and the pager's",
        app.seen.stops
    );
    println!(
        "  standing     {:>8}   of {} fields",
        app.seen.standing,
        LABELS.len()
    );

    println!("\n== the two arms of `Ctrl+G` ==\n");
    println!(
        "  {:<10} {:>7} {:>7} {:>10}",
        "arm", "stops", "regions", "standing"
    );
    for grouped in [true, false] {
        let mut probe = App::new();
        probe.counters = true;
        probe.grouped = grouped;
        let mut driver = Driver::headless(w, h).expect("a sink attaches");
        driver.frame(|cx| probe.ui(cx));
        probe.read_frame(&driver);
        driver.frame(|cx| probe.ui(cx));
        probe.read_frame(&driver);
        println!(
            "  {:<10} {:>7} {:>7} {:>10}",
            if grouped { "group" } else { "flat" },
            probe.seen.stops,
            probe.seen.regions,
            probe.seen.standing
        );
    }

    println!("\n== `Ctrl+D`, on a panel with room for six at one density ==\n");
    println!(
        "  {:<10} {:>7} {:>10} {:>9}",
        "density", "writes", "standing", "dropped"
    );
    for (word, density) in [("compact", Density::Compact), ("cosy", Density::Cosy)] {
        let mut probe = App::new();
        probe.counters = true;
        // A short screen on purpose: the two densities differ by two rows of interior, so a panel
        // with room to spare stands six either way and says nothing.
        let mut driver = vitui_components::runner::driver_at(60, FLOOR_H + 2, density);
        driver.frame(|cx| probe.ui(cx));
        probe.read_frame(&driver);
        driver.frame(|cx| probe.ui(cx));
        probe.read_frame(&driver);
        println!(
            "  {:<10} {:>7} {:>10} {:>9}",
            word,
            probe.seen.writes,
            probe.seen.standing,
            LABELS.len() - probe.seen.standing
        );
    }

    // **And the partition, swept.** An application that assembles its own rectangles owes the partition rule's
    // equality over the ones it made up, and four of this crate's tickets found the hole in their
    // own application rather than in a component.
    let mut swept = 0usize;
    let mut torn: Vec<(u16, u16)> = Vec::new();
    for w in [24u16, 30, 40, 60, 80, 100, 140, 200] {
        for h in 1u16..=40 {
            swept += 1;
            let Ok(mut driver) = Driver::headless(w, h) else {
                continue;
            };
            let mut probe = App::new();
            probe.counters = true;
            driver.frame(|cx| probe.ui(cx));
            probe.read_frame(&driver);
            driver.frame(|cx| probe.ui(cx));
            let cells = u64::from(w) * u64::from(h);
            if probe.seen.writes != probe.seen.distinct || probe.seen.distinct != cells {
                torn.push((w, h));
            }
        }
    }
    println!(
        "\n  partition    {:>8}   sizes swept, {} of them not a partition{}",
        swept,
        torn.len(),
        if torn.is_empty() {
            String::new()
        } else {
            format!(": {torn:?}")
        }
    );
}

fn main() {
    let mut app = App::new();
    if env::args().any(|a| a == "--probe") {
        probe(&mut app);
        return;
    }

    let mut themes = Themes::standard();
    let mut driver = match Driver::attach(Default::default(), *themes.theme()) {
        Ok(driver) => driver,
        Err(why) => {
            eprintln!("vitui could not attach: {why}");
            return;
        }
    };

    loop {
        driver.frame(|cx| app.ui(cx));
        app.read_frame(&driver);
        // **Read immediately after this application's own frame and never before it.** The window is
        // valid until the next frame begins, so a loop that read it before its own frame acted on
        // the previous frame's window — one wake late, which for a single keystroke means never.
        let unhandled: Vec<Pressed> = driver.unhandled().to_vec();
        let moved = app.take_unhandled(&unhandled);
        if app.exit {
            break;
        }
        if app.density_moved {
            // The swap happens between frames and it cannot do better: a component may not write to
            // the frame's environment mid-frame, so the key records a choice and the loop applies it.
            app.density_moved = false;
            themes.set_density(if app.cosy {
                Density::Cosy
            } else {
                Density::Compact
            });
            driver.set_theme(*themes.theme());
        }
        if moved || !unhandled.is_empty() {
            continue;
        }
        match driver.wait() {
            Wake::Quit => break,
            Wake::Input | Wake::Posted | Wake::Deadline => {}
        }
    }
}
