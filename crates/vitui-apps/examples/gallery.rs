//! **`gallery` — every built component on one screen, and `t` is the key to press.**
//!
//! Components ticket 39, and the implementation of `tickets/002`. Spec §17 (O2), §21.
//!
//! ```text
//! cargo run -p vitui-apps --example gallery              # the screen
//! cargo run -p vitui-apps --example gallery -- --probe   # one headless frame, and what it cost
//! cargo run -p vitui-apps --example gallery -- --matrix  # the nine cells, as counts
//! cargo run -p vitui-apps --example gallery -- --panic   # for `scripts/gallery-panic-gate.sh`
//! ```
//!
//! # The gallery is a gate, not a demo, and this file is the thinnest half of it
//!
//! Everything drawn here is [`vitui_components::gallery`]'s: the panel table, the twenty-eight
//! drawings, the tile grid and the state. **This file iterates that table and mints no panel of its
//! own**, and a source scan in the component crate says so from the other side
//! (`gallery::tests::the_application_draws_the_table_and_mints_no_panel_of_its_own`). An application
//! free to list its own panels is an application that can drift from the freeze in a direction no
//! equality over the table can see — which is the whole of O2.
//!
//! The screen lives in the library crate for a reason that is not tidiness: spec §21 names two
//! defects to be measured **on the assembled gallery** — the sentinel and the palette swap, register
//! rows 7 and 8 — and both are components tickets whose gate is `cargo test`. A screen only an
//! application can reach is a screen no gate can measure.
//!
//! # `t` is not cosmetic, and this screen is where it is cheap enough to be a key
//!
//! `t` re-imports and re-resolves the whole theme on a live frame — fourteen schemes, thirteen
//! roles, ten distinctions, both axes of §16's matrix. That is the test of *degradation is resolved
//! at construction, never branched at the draw* (ADR 0032): if construction quietly meant *at
//! start-up*, this key would stutter. `examples/gallery_numbers.rs` prints what it costs.
//!
//! It is also the most visible possible form of *a memo's key is every input* (spec §10). Six of the
//! twenty-eight keep a memo — the two charts, the sparkline, the wrap index, the flatten index and
//! the preview — and a memo keyed without the theme shows up here as a panel that keeps the old
//! palette while its neighbours change.
//!
//! # `t` cannot be the only spelling, and the reason is O4
//!
//! **A focused `field` consumes every text-bearing key** and a focused `collection` eats one into
//! its type-ahead buffer (spec §3, §5, components 38). This gallery has a `field`, a `form`, three
//! collections and a picker on it, all one `Tab` away — so `t` reaches the theme only while the
//! focus is somewhere that does not read text, and `Ctrl+T` is the spelling that always arrives.
//! `ledger` and `explorer` bind `Ctrl+Q` beside `q` for exactly this; here **there is no `q` to
//! bind beside**, because typing one into the field is the point.
//!
//! # Keys
//!
//! | key | what |
//! |---|---|
//! | `Tab` `BackTab` | the focus walk — the runtime's key and no component's (O4) |
//! | any character | into whatever holds the focus, which is what a `field` is *for* |
//! | `t` `T` | the next or previous of the fourteen schemes — **eaten by a focused field** |
//! | `Ctrl+T` | the next scheme, always |
//! | `Ctrl+G` | the next of §16's three repertoires: ascii, unicode, extended |
//! | `Ctrl+L` | the next of §16's three colour depths — watch the traffic light go monochrome |
//! | `Ctrl+N` `Ctrl+P` | the next or previous page of panels |
//! | `Ctrl+D` | `Compact` against `Cosy` — density is theme data (spec §3) |
//! | `Ctrl+F` | ask the preview pane about the next file |
//! | `Ctrl+O` | the counters on and off, through one `Ink` and one call site |
//! | `Ctrl+Q` · `Esc` | quit |
//!
//! # Nothing here names crossterm, and that is the criterion met rather than excepted
//!
//! The ticket asks for the gallery to join ADR 0001's wrapper list because a gallery uses crossterm
//! "for raw mode, the alternate screen and input". **It needs none of that**: `Driver::attach` enters
//! raw mode and the alternate screen, reads the input, and restores both — the panic hook included —
//! and `deny.toml`'s `{ name = "crossterm", wrappers = ["vitui-engine"] }` is unchanged. The
//! restoration is checked here by `scripts/gallery-panic-gate.sh`, under a real pty, on this binary.

use std::env;
use std::fmt::Write as _;

use vitui_components::counters::Tally;
use vitui_components::gallery::{self, Gallery, Sink};
use vitui_components::ink::Direct;
use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{Code, Pressed};
use vitui_runtime::work::{Wake, Worker};
use vitui_runtime::{ColorDepth, Density, GlyphSet, Mods};

/// What the loop keeps beside the gallery itself.
struct App {
    gallery: Gallery,
    exit: bool,
    counters: bool,
    cosy: bool,
    theme_moved: bool,
    density_moved: bool,
    seen: Seen,
    line: String,
}

/// What `--probe` and `Ctrl+O` report. Read after the frame, never during it.
#[derive(Clone, Copy, Default, Debug)]
struct Seen {
    writes: u64,
    distinct: u64,
    verbs: u64,
    regions: usize,
    stops: usize,
    merges: u32,
    panels: usize,
}

impl App {
    fn new(worker: Worker) -> App {
        App {
            gallery: Gallery::new(worker),
            exit: false,
            counters: false,
            cosy: true,
            theme_moved: false,
            density_moved: false,
            seen: Seen::default(),
            line: String::new(),
        }
    }

    /// One frame, through whichever ink `Ctrl+O` chose.
    ///
    /// **One call site and one type.** `&mut dyn Ink` satisfies `I: Ink` through the components
    /// crate's blanket impl, and `explorer` found what two call sites cost: two
    /// `Location::caller()`s, so toggling the counters would re-declare every widget under a new id
    /// and lose the focus on a screen that looks identical.
    fn ui<'f>(&'f mut self, cx: &mut vitui_runtime::Ctx<'f, '_>) {
        let mut tally = Tally::new();
        let mut direct = Direct;
        // **Destructured, and the status text is read rather than cloned.** `ui_into` takes `&'f mut
        // self` of the gallery, so a later `self.line` would be `E0502`; a `clone` would compile and
        // allocate on the frame path, which is the one thing §20 forbids outright.
        let App {
            gallery,
            counters,
            seen,
            line,
            ..
        } = self;
        let mut sink: Sink<'_> = if *counters { &mut tally } else { &mut direct };
        gallery.ui_into(&mut sink, cx, line);
        if *counters {
            seen.writes = tally.writes();
            seen.distinct = tally.distinct();
            seen.verbs = tally.verbs();
        }
    }

    /// What the frame declared. **After the frame and never inside it.**
    fn read_frame(&mut self, driver: &Driver) {
        let frame = driver.inspect();
        self.seen.regions = frame.hits().len();
        self.seen.stops = frame.stop_count();
        self.seen.merges = frame.ids().merges();
        self.seen.panels = self.gallery.shown();
    }

    /// The keys nobody in the frame took.
    ///
    /// **Read immediately after this application's own frame and never before it.** The window is
    /// valid until the next frame begins, so a loop that read it first acted on the previous frame's
    /// window — one wake late, which for a single keystroke means never (map decision C25).
    fn take_unhandled(&mut self, keys: &[Pressed], w: u16, h: u16) -> bool {
        let mut moved = false;
        for k in keys {
            let ctrl = k.mods.chord().contains(Mods::CTRL);
            match k.code {
                Code::Escape => self.exit = true,
                Code::Char('q') if ctrl => self.exit = true,
                Code::Char('t') if ctrl => {
                    self.gallery.next_theme();
                    self.theme_moved = true;
                    moved = true;
                }
                Code::Char('t') if !ctrl => {
                    self.gallery.next_theme();
                    self.theme_moved = true;
                    moved = true;
                }
                Code::Char('T') if !ctrl => {
                    self.gallery.prev_theme();
                    self.theme_moved = true;
                    moved = true;
                }
                Code::Char('g') if ctrl => {
                    self.gallery.next_rung();
                    self.theme_moved = true;
                    moved = true;
                }
                Code::Char('l') if ctrl => {
                    self.gallery.next_tier();
                    self.theme_moved = true;
                    moved = true;
                }
                Code::Char('n') if ctrl => {
                    self.gallery.next_page(w, h);
                    moved = true;
                }
                Code::Char('p') if ctrl => {
                    self.gallery.prev_page(w, h);
                    moved = true;
                }
                Code::Char('d') if ctrl => {
                    self.cosy = !self.cosy;
                    self.density_moved = true;
                    moved = true;
                }
                Code::Char('f') if ctrl => {
                    self.gallery.bag.show_next();
                    moved = true;
                }
                Code::Char('o') if ctrl => {
                    self.counters = !self.counters;
                    moved = true;
                }
                _ => {}
            }
        }
        moved
    }

    /// The status line, staged into one reused `String`. **No allocation on the frame path.**
    fn status(&mut self, w: u16, h: u16) -> &str {
        let (page, of) = self.gallery.paging(w, h);
        self.line.clear();
        let _ = write!(
            self.line,
            "{} panels · page {}/{of} · {} · {}/{} · {} regions · {} stops · {} merges",
            self.seen.panels,
            page + 1,
            self.gallery.scheme(),
            gallery::rung_word(self.gallery.rung()),
            gallery::tier_word(self.gallery.tier()),
            self.seen.regions,
            self.seen.stops,
            self.seen.merges,
        );
        &self.line
    }
}

/// **One headless frame at 300×80, and what it cost.**
///
/// The budget is measured **in the gallery** and not only in isolated harnesses, which is the
/// criterion §20 states and the four single-component screens over it are the reason for.
fn probe() {
    for (w, h) in [(300u16, 80u16), (100, 30), (80, 24)] {
        let (cols, rows, per) = gallery::grid(w, h);
        let shape = gallery::shape(w, h, 3);
        let cells = usize::from(w) * usize::from(h);
        println!("== gallery, one frame at {w}x{h} ==\n");
        let of = gallery::pages(w, h);
        println!(
            "  grid          {cols}x{rows}   {per} panels a page, {of} page{}",
            if of == 1 { "" } else { "s" }
        );
        println!("  panels     {:>8}", shape.panels);
        println!("  writes     {:>8}", shape.writes);
        println!(
            "  distinct   {:>8}   of {cells} cells{}",
            shape.distinct,
            if shape.writes == shape.distinct {
                "   no cell written twice"
            } else {
                "   ** a cell is written twice **"
            }
        );
        println!("  verbs      {:>8}", shape.verbs);
        println!("  regions    {:>8}", shape.regions);
        println!("  stops      {:>8}", shape.stops);
        println!(
            "  merges     {:>8}   {}",
            shape.merges,
            if shape.merges == 0 {
                "every tile is its own widget"
            } else {
                "** tiles share an id **"
            }
        );
        println!(
            "  unwritten  {:>8}   register row 7's subject, reported and not gated (components 40)\n",
            shape.unwritten
        );
    }
}

/// **§16's nine cells, as counts.** Three repertoires against three colour depths, on this screen.
fn matrix() {
    println!("== the nine cells, at 100x30 ==\n");
    println!(
        "  {:<10} {:<12} {:>8} {:>7} {:>9} {:>7} {:>10} {:>7}",
        "rung", "colour", "writes", "verbs", "clusters", "paints", "roles gone", "d. lost"
    );
    for cell in gallery::matrix(100, 30) {
        println!(
            "  {:<10} {:<12} {:>8} {:>7} {:>9} {:>7} {:>10} {:>7}",
            gallery::rung_word(cell.rung),
            gallery::tier_word(cell.tier),
            cell.writes,
            cell.verbs,
            cell.clusters,
            cell.paints,
            cell.roles_collapsed,
            cell.distinctions_lost,
        );
    }
    println!("\n== the traffic light, over the fourteen shipped schemes ==\n");
    println!(
        "  {:<12} {:>10} {:>14} {:>18}",
        "colour", "declares", "on the wire", "distinct paints"
    );
    for tier in [
        ColorDepth::None,
        ColorDepth::Ansi16,
        ColorDepth::Indexed256,
        ColorDepth::TrueColor,
    ] {
        let t = gallery::traffic_light(tier);
        println!(
            "  {:<12} {:>7}/14 {:>11}/14 {:>15}/14",
            gallery::tier_word(tier),
            t.declares,
            t.on_the_wire,
            t.distinct_paints
        );
    }
    println!(
        "\n  The criterion says all three quantise to bright white at sixteen colours. Measured on\n  \
         the shipped palette they do so in **8 of 14** schemes — six keep the three distinct — and\n  \
         in **14 of 14** at `none`, so the sentence is the right claim about the wrong rung.\n\n  \
         The third column reads 14/14 at every depth, and the reason is not that a paint is a\n  \
         coarse comparison: **`Theme::resolve` returns the same paint for all thirteen roles at\n  \
         every depth**, because quantisation is the engine's and happens before the mirror. A\n  \
         component is handed the palette's own colour whatever the terminal can show, which is\n  \
         ADR 0018 working rather than a hole — and it is why the colour axis of the nine cells\n  \
         above is read off the theme rather than off the screen.\n"
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--probe") {
        probe();
        return;
    }
    if args.iter().any(|a| a == "--matrix") {
        matrix();
        return;
    }

    let mut driver = match Driver::attach(Default::default(), Default::default()) {
        Ok(driver) => driver,
        Err(why) => {
            eprintln!("vitui could not attach: {why}");
            return;
        }
    };
    // **The worker is hired from the driver's own wake handle**, so a preview that finishes decoding
    // wakes the loop rather than waiting for a keystroke.
    let mut app = App::new(Worker::hire(driver.wake()));
    // The terminal's own answer, told once at start-up. The registry detects nothing itself
    // (ADR 0007) and starts at `ColorDepth::None`, which is the conservative answer and not a
    // placeholder.
    let tier = driver.env().caps().colors;
    app.gallery.set_tier(tier);
    driver.set_theme(*app.gallery.theme());

    // **The panic path is the engine's**, and it is checked on this binary under a real pty by
    // `scripts/gallery-panic-gate.sh`. The flag exists for that gate and for nothing else: a
    // restoration nobody has watched happen is a restoration nobody can trust.
    if args.iter().any(|a| a == "--panic") {
        driver.frame(|cx| app.ui(cx));
        panic!("gallery: the panic gate's own panic");
    }

    loop {
        let (w, h) = driver.size();
        driver.frame(|cx| app.ui(cx));
        app.read_frame(&driver);
        let unhandled: Vec<Pressed> = driver.unhandled().to_vec();
        let moved = app.take_unhandled(&unhandled, w, h);
        if app.exit {
            break;
        }
        if app.density_moved {
            app.density_moved = false;
            app.gallery.set_density(if app.cosy {
                Density::Cosy
            } else {
                Density::Compact
            });
            app.theme_moved = true;
        }
        if app.theme_moved {
            // **The swap happens between frames and cannot do better**: a component may not write to
            // the frame's environment mid-frame, so the key records a choice and the loop applies it.
            app.theme_moved = false;
            driver.set_theme(*app.gallery.theme());
        }
        // The status line is printed by the gallery itself; this is what a caller would log.
        let _ = app.status(w, h);
        if moved || !unhandled.is_empty() {
            continue;
        }
        match driver.wait() {
            Wake::Quit => break,
            Wake::Input | Wake::Posted | Wake::Deadline => {}
        }
    }
    let _ = (GlyphSet::default(), ColorDepth::None);
}
