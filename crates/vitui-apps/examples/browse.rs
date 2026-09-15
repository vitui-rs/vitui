//! A file browser whose preview pane really decodes on another thread.
//!
//! The preview pane's application. It is the first thing to put
//! a [`file_preview_pane`](vitui_components::files::file_preview_pane) anywhere, and it exists for
//! the reason every file here exists: **the surface's only consumer is an application**, and ten of
//! the ten before it found something their own gates could not see.
//!
//! ```text
//!   the listing            the caller's order, and the caller's to re-sort
//!     |
//!   collection(…)          one hit entry, whatever the volume
//!     |
//!   the cursor's FILE      not its position — one re-sort and the two are different questions
//!     |
//!   Task::request(key, …)  deduplicated on the key, every frame, unconditionally
//!     |
//!   Worker (a real thread) the decode, which never runs here
//!     |
//!   PaneState::land(&task) taken once, at the top of the view
//!     |
//!   file_preview_pane(…)   the extent is the answer's, the offset is reset on the landing
//! ```
//!
//! # What it demonstrates
//!
//! | key | what it shows |
//! |---|---|
//! | `↑` `↓` `PgUp` `PgDn` | move the cursor. Every move is a question; most of them are cancelled |
//! | `s` | **re-sort the listing.** One line of application code and no keystroke inside the pane |
//! | `k` | key the question on the cursor's **position** instead. Then press `s` |
//! | `o` | cycle the four offset spellings, on a long file and a short one |
//! | `x` | declare no extent, so the offset is never clamped. Watch the pane go blank after a shrink |
//! | `b` | advance the revision on every frame. The screen does not change and the fold count does |
//! | `l` | take the landing inside the draw. The two status rows stop agreeing |
//! | `p` | show the selected file as a **photograph** instead: one `Theme::custom` a cell |
//! | `d` | make the decode slow, so the crossover is something you can see |
//! | `v` | the live counters, through a `Tally` |
//! | `q` `Ctrl+Q` | quit |
//!
//! **`s` is the one to watch, and it is the reason this application exists.** Press `k`, then `s`.
//! Nothing moves. The listing is in a different order, the cursor is on a different file, the pane
//! is still showing the file that used to be at that position, and it will go on showing it for
//! ever — because the question still matches, so nothing posts, so nothing wakes, so no frame
//! corrects it. Press `k` again and the same `s` fixes it on the next frame. That is the memo-key
//! rule through the one door whose trigger is not a gesture, and there is no keystroke anywhere
//! that could have found it.
//!
//! **`l` is the tear, and no thread is involved in it.** [`Task::take`] is destructive, so a pane
//! that takes its own landing mid-draw hands the *first* reader the answer and the second the value
//! that was there before. This screen has a status row above the pane and one below it, so the tear
//! is visible: press `l` and hold `↓`, and the two rows disagree on every frame a decode lands on.
//! Nothing throws and the state ends up correct, which is exactly why a torn frame leaves nothing
//! behind.
//!
//! **`x` is the shrink.** Select the long file, press `End`, then select a short one. With the
//! extent declared the offset clamps and the short file opens at its last page or at its top,
//! depending on `o`; with `x` on, nothing clamps, the body has no rows to draw at that offset, and
//! the area has no tail with which to cover them — so the pane is *blank*, and the counters say
//! 21 484 cells written of 24 000.
//!
//! # What it deliberately does not do
//!
//! **It does not own the worker, and neither does the pane.** The [`Worker`] and the [`Task`] are
//! fields of `App`, handed to the component by reference on every frame. That is R02's sweep
//! refused: a slot in the widget registry is released when its widget stops drawing, and ten tab
//! switches would then cost ten spawns and ten decodes instead of one and one.
//!
//! ```text
//! cargo run -p vitui-apps --example browse
//! ```

use std::time::{Duration, Instant};

use vitui_components::collect::{CollOpts, CollState, Mode, collection_into};
use vitui_components::counters::Tally;
use vitui_components::files::{
    Bump, Extent, Landed, PaneOpts, PaneShape, PaneState, Preview, Reset, Taken, asking,
    file_preview_pane_into,
};
use vitui_components::frame::{Face, face_paint};
use vitui_components::ink::{Direct, Ink};
use vitui_components::media::{Census, PictureOpts, Pixels, picture_into, sub_rows};
use vitui_components::order::Rows;
use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{Code, Pressed};
use vitui_runtime::work::{Cancel, Task, Wake, Worker};
use vitui_runtime::{Ctx, Rect, Rgb, Role};

// ── the directory ────────────────────────────────────────────────────────────────────────────────

/// How many files the browser lists.
const FILES: usize = 400;

/// One entry, with everything the two panes need.
#[derive(Clone)]
struct File {
    /// **The file's own identity.** What an identity key is keyed on.
    id: u64,
    name: String,
    /// How many lines its document has.
    rows: u32,
    /// Its rank in the by-size order, so that sorting by size really is a sort.
    size: u64,
}

/// The directory, in name order. Three of the four hundred are fixed points of the size order, so a
/// re-sort moves 397 of 400 positions and is a permutation of the data rather than a stated one.
fn directory() -> Vec<File> {
    let fixed = [0usize, 200, 399];
    let movable: Vec<usize> = (0..FILES).filter(|i| !fixed.contains(i)).collect();
    let mut rank = vec![0u64; FILES];
    for (k, &position) in movable.iter().enumerate() {
        rank[movable[(k + 1) % movable.len()]] = position as u64;
    }
    for &f in &fixed {
        rank[f] = f as u64;
    }
    (0..FILES)
        .map(|i| File {
            id: 0x9E37_79B9_0000_0000 ^ i as u64,
            name: format!("f{i:03}.txt"),
            // **A long file every seventeenth**, so that a shrink is one keystroke away wherever
            // the cursor happens to be. `x` and `o` are about the moment the extent gets smaller.
            rows: if i % 17 == 0 {
                4_000
            } else {
                40 + (i as u32 % 7) * 13
            },
            size: rank[i],
        })
        .collect()
}

/// Which order the listing is in. **The application's, and the application's to change.**
#[derive(Clone, Copy, PartialEq, Eq)]
enum Order {
    ByName,
    BySize,
}

// ── what a decode produces ───────────────────────────────────────────────────────────────────────

/// What the pane shows. **Eight bytes of identity on the payload**, which is [`Preview::shows`].
struct Doc {
    id: u64,
    name: String,
    rows: u32,
    /// Whether the caller asked for the file to be drawn as a photograph.
    photograph: bool,
}

impl Preview for Doc {
    fn shows(&self) -> u64 {
        self.id
    }

    fn extent(&self) -> (u32, u32) {
        (if self.photograph { PICTURE_W } else { LINE_W }, self.rows)
    }
}

/// How wide a text document is.
const LINE_W: u32 = 72;

/// How wide a photograph is. Wider than any pane this screen produces, so the horizontal bar has
/// something to say.
const PICTURE_W: u32 = 220;

/// The decode. **It never runs on the app thread** — it is handed to `Task::request` and the worker
/// calls it, which is why this function may take as long as it likes.
fn decode(file: &File, photograph: bool, slow: bool, cancel: &Cancel) -> Doc {
    if slow {
        // A decode a person can watch. It polls the bracket, which is the only kind of cancellation
        // there is: a `std::thread` cannot be killed, so "cancel" means the job looks.
        for _ in 0..60 {
            if cancel.cancelled() {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    Doc {
        id: file.id,
        name: file.name.clone(),
        rows: file.rows,
        photograph,
    }
}

/// One row of the photograph, as a [`Pixels`] source. A picture never tells its source where on the
/// screen it landed, so the strip does the offsetting and the frame costs the rectangle.
struct Strip {
    top: u32,
    sub: u32,
}

impl Pixels for Strip {
    fn pixel(&self, x: u16, sub_y: u32) -> Rgb {
        let x = u32::from(x);
        let y = self.top * self.sub + sub_y;
        Rgb::new(
            (x * 3 + y * 2) as u8,
            (y * 5 + x) as u8,
            (200 - (x % 200)) as u8,
        )
    }
}

// ── the application ──────────────────────────────────────────────────────────────────────────────

/// What the frame reported, for the status line.
#[derive(Clone, Copy, Default)]
struct Counters {
    writes: u64,
    distinct: u64,
    verbs: u64,
    customs: u64,
    regions: usize,
    content: u64,
}

struct App {
    files: Vec<File>,
    order: Order,
    list: CollState,
    /// **The pane's state, and it is the application's.** R02's sweep is refused by where this
    /// lives, not by anything the component does.
    pane: PaneState<Doc>,
    /// **The worker and the task, likewise.** A job's lifetime is the question's.
    worker: Worker,
    task: Task<Doc>,
    /// Whether the question is keyed on the cursor's position instead of on its file.
    by_position: bool,
    /// Whether the pane draws the file as a photograph.
    photograph: bool,
    /// Whether the decode takes 300 ms.
    slow: bool,
    /// Whether the counters are live.
    counting: bool,
    counters: Counters,
    /// What the last landing did, for the status line.
    landed: Landed,
    landings: u64,
    refused: u64,
    /// The two status rows, so that a tear is on the screen and in the report.
    top: String,
    bottom: String,
    exit: bool,
    started: Instant,
}

impl App {
    /// **The worker is handed in and not built here**, which is the same sentence as R02's refusal
    /// one level down: a job's lifetime is the question's, and the question outlives every widget
    /// that ever asks it. `main` hires a resident thread against the driver's wake handle; `--probe`
    /// hands over a `Worker::queueing`, so a report is a straight-line program.
    fn new(worker: Worker) -> App {
        let task = Task::new(&worker);
        App {
            files: directory(),
            order: Order::ByName,
            list: CollState::new(),
            pane: PaneState::new(),
            worker,
            task,
            by_position: false,
            photograph: false,
            slow: false,
            counting: false,
            counters: Counters::default(),
            landed: Landed::Nothing,
            landings: 0,
            refused: 0,
            top: String::new(),
            bottom: String::new(),
            exit: false,
            started: Instant::now(),
        }
    }

    /// **The shape the four keys spell.** One value, so that a reader's diff between the shipped
    /// pane and any refused one is one field.
    fn shape(&self) -> PaneShape {
        self.pane.shape()
    }

    /// Re-shape the pane, keeping nothing: a spelling change is a different pane.
    fn respell(&mut self, f: impl FnOnce(&mut PaneShape)) {
        let mut shape = self.shape();
        f(&mut shape);
        self.pane = PaneState::shaped(shape);
    }
}

/// The listing, as positions into the directory. **The order is the caller's**, and its being one
/// line of application code is the whole of what `s` demonstrates.
fn order_of(files: &[File], order: Order) -> Vec<usize> {
    let mut out: Vec<usize> = (0..files.len()).collect();
    match order {
        Order::ByName => out.sort_by(|&a, &b| files[a].name.cmp(&files[b].name)),
        Order::BySize => out.sort_by_key(|&i| files[i].size),
    }
    out
}

/// The height of the chrome above the pane, and below it.
const HEAD: u16 = 3;
const FOOT: u16 = 2;
/// How wide the listing is.
const LIST_W: u16 = 26;

fn ui<I: Ink>(app: &mut App, ink: &mut I, cx: &mut Ctx<'_, '_>) {
    let (w, h) = cx.size();
    if w < 40 || h < 10 {
        let paint = cx.theme().paint(Role::Warn);
        let _ = ink.pad_to(cx, 0, 0, "too small", w, paint);
        return;
    }

    let App {
        files,
        order,
        list,
        pane,
        task,
        by_position,
        photograph,
        slow,
        counting,
        counters,
        landed,
        landings,
        refused,
        top,
        bottom,
        ..
    } = app;
    let listing = order_of(files, *order);
    let cursor = list.sel.lead.min(listing.len() - 1);

    // **The landing, at the top of the view.** Taken once, into a value both status rows share —
    // unless `l` has moved it into the draw, which is what the two rows are here to show.
    *landed = pane.land(task);
    *landings = pane.landings();
    *refused = pane.refused();

    let theme = cx.theme();
    let title = theme.paint(Role::Title);
    let dim = theme.paint(Role::Dim);
    let border = theme.paint(Role::Border);

    let shape = pane.shape();
    *top = header(
        shape,
        pane.showing().map(|d| d.name.as_str()),
        *by_position,
        *landings,
        *refused,
    );
    let _ = ink.pad_to(cx, 0, 0, "vitui browse", w, title);
    let _ = ink.pad_to(cx, 0, 1, top, w, dim);
    let _ = ink.run(cx, 0, 2, "-", w, border);

    let body_h = h.saturating_sub(HEAD + FOOT);
    let list_w = LIST_W.min(w / 2);
    let list_rect = Rect::new(0, i32::from(HEAD), list_w, body_h);
    let rule = Rect::new(i32::from(list_w), i32::from(HEAD), 1, body_h);
    let pane_rect = Rect::new(
        i32::from(list_w) + 1,
        i32::from(HEAD),
        w - list_w - 1,
        body_h,
    );

    let opts = CollOpts {
        mode: Mode::Single,
        ..CollOpts::default()
    };
    let names: Vec<&str> = listing.iter().map(|&i| files[i].name.as_str()).collect();
    let _ = collection_into(
        &mut *ink,
        cx,
        list_rect,
        list,
        &opts,
        Rows::of(listing.len()),
        |buf, range: std::ops::Range<usize>| range.into_iter().find(|&i| names[i].starts_with(buf)),
        |ink: &mut I, cx: &mut Ctx<'_, '_>, r: Rect, i: usize, face: Face| {
            let paint = face_paint(cx.theme(), face);
            let _ = ink.pad_to(cx, r.x, r.y, names[i], r.w, paint);
        },
    );

    for row in 0..rule.h {
        let _ = ink.run(cx, rule.x, rule.y + i32::from(row), "|", 1, border);
    }

    // **The question, and its key is the whole of `k`.** A file's identity survives a re-sort; its
    // position does not.
    let file = files[listing[cursor]].clone();
    let key = if *by_position { cursor as u64 } else { file.id };
    let photograph = *photograph;
    let slow = *slow;
    let pane_opts = PaneOpts::default();
    let id = cx.id();
    let mut content = 0u64;
    let mut customs = 0u64;
    let mut census = Census::default();
    let _ = file_preview_pane_into(
        &mut *ink,
        cx,
        id,
        pane_rect,
        pane,
        task,
        // **`asking` and not `Question::new`**, and the difference is one annotation: a closure's
        // signature is inferred from the expected type at the expression, and a bare struct field
        // supplies none — so `Question::new(key, |cancel| …)` infers one specific lifetime and the
        // call site fails with *implementation of `FnOnce` is not general enough*, naming neither
        // the closure nor the parameter. `asking` carries the bound, so the higher-ranked lifetime
        // infers.
        asking(key, move |cancel| decode(&file, photograph, slow, cancel)),
        &pane_opts,
        |ink: &mut I, cx: &mut Ctx<'_, '_>, row: Rect, doc: &Doc, i: u32| {
            if doc.photograph {
                let strip = Strip {
                    top: i,
                    sub: u32::from(sub_rows(cx.theme().glyphs())),
                };
                let before = census.customs;
                let _ = picture_into(ink, cx, row, &strip, &PictureOpts::default(), &mut census);
                customs += census.customs - before;
                content += u64::from(row.w);
            } else {
                let paint = cx.theme().paint(Role::Body);
                let text = format!("{:>5}  {}  {}", i, doc.name, "-".repeat(40));
                content += u64::from(ink.pad_to(cx, row.x, row.y, &text, row.w, paint));
            }
        },
    );
    counters.content = content;
    counters.customs = customs;

    let _ = ink.run(cx, 0, i32::from(h - FOOT), "-", w, border);
    // **The second reader.** Under the shipped spelling it reads the same value the first did;
    // under `l` the component above has taken the landing in between.
    *bottom = footer(
        pane.showing().map(|d| d.name.as_str()),
        pane.offset(),
        *counting,
        *counters,
    );
    let _ = ink.pad_to(cx, 0, i32::from(h - 1), bottom, w, dim);
}

/// The top status row: what the pane is showing, and what it was asked.
fn header(
    shape: PaneShape,
    showing: Option<&str>,
    by_position: bool,
    landings: u64,
    refused: u64,
) -> String {
    format!(
        "preview {:<12}  key {}  offset {}  extent {}  revision {}  landing {}  |  landed {} refused {}",
        showing.unwrap_or("-"),
        if by_position { "position" } else { "file" },
        match shape.reset {
            Reset::OnTheLanding => "on the landing",
            Reset::OnTheRequest => "on the request",
            Reset::Never => "kept",
        },
        match shape.extent {
            Extent::OfTheAnswer => "of the answer",
            Extent::Unbounded => "unbounded",
        },
        match shape.bump {
            Bump::OnTheLanding => "on the landing",
            Bump::EveryFrame => "every frame",
        },
        match shape.taken {
            Taken::AtTheTopOfTheView => "top of view",
            Taken::InsideTheDraw => "inside the draw",
        },
        landings,
        refused,
    )
}

/// The bottom status row. **The same value, printed a second time**, which is what makes a torn
/// frame visible without a debugger.
fn footer(showing: Option<&str>, offset: u32, counting: bool, c: Counters) -> String {
    let mut out = format!(
        "preview {:<12}  offset {:>5}  ",
        showing.unwrap_or("-"),
        offset
    );
    if counting {
        out.push_str(&format!(
            "writes {} distinct {} verbs {} customs {} regions {} content {}  ",
            c.writes, c.distinct, c.verbs, c.customs, c.regions, c.content,
        ));
    }
    out.push_str(
        "s sort  k key  o offset  x extent  b revision  l landing  p photo  d slow  v counters  q quit",
    );
    out
}

impl App {
    /// Read what the frame that has just drawn reported.
    fn read_frame(&mut self, driver: &Driver, tally: Option<&Tally>) {
        self.counters.regions = driver.inspect().hits().len();
        if let Some(t) = tally {
            self.counters.writes = t.writes();
            self.counters.distinct = t.distinct();
            self.counters.verbs = t.verbs();
        }
    }

    /// **What nothing wanted, read from the frame that has just drawn.** `Driver::unhandled` is a
    /// window onto the same queue, valid until the next frame begins, so an application that reads
    /// it before its own frame acts one wake late — which for a single keystroke means never.
    fn take_unhandled(&mut self, keys: &[Pressed]) {
        for k in keys {
            match k.code {
                // **A printable quit key cannot survive a focused `collection`**, which eats every
                // text-bearing key into its type-ahead buffer. `Ctrl+Q` is bound beside it for the
                // same reason `ledger` and `explorer` bind it.
                Code::Char('q') | Code::Char('Q') => self.exit = true,
                Code::Char('c') if k.mods.ctrl() => self.exit = true,
                Code::Char('s') => {
                    self.order = if self.order == Order::ByName {
                        Order::BySize
                    } else {
                        Order::ByName
                    };
                }
                Code::Char('k') => self.by_position = !self.by_position,
                Code::Char('o') => self.respell(|s| {
                    s.reset = match s.reset {
                        Reset::OnTheLanding => Reset::OnTheRequest,
                        Reset::OnTheRequest => Reset::Never,
                        Reset::Never => Reset::OnTheLanding,
                    };
                }),
                Code::Char('x') => self.respell(|s| {
                    s.extent = match s.extent {
                        Extent::OfTheAnswer => Extent::Unbounded,
                        Extent::Unbounded => Extent::OfTheAnswer,
                    };
                }),
                Code::Char('b') => self.respell(|s| {
                    s.bump = match s.bump {
                        Bump::OnTheLanding => Bump::EveryFrame,
                        Bump::EveryFrame => Bump::OnTheLanding,
                    };
                }),
                Code::Char('l') => self.respell(|s| {
                    s.taken = match s.taken {
                        Taken::AtTheTopOfTheView => Taken::InsideTheDraw,
                        Taken::InsideTheDraw => Taken::AtTheTopOfTheView,
                    };
                }),
                Code::Char('p') => self.photograph = !self.photograph,
                Code::Char('d') => self.slow = !self.slow,
                Code::Char('v') => self.counting = !self.counting,
                _ => {}
            }
        }
    }
}

/// One headless frame, and what it cost. `theatre`'s own door, for its reason: the subject of this
/// screen is a set of counts and a person reading a report should not have to hold a terminal open.
fn probe(app: &mut App) {
    let mut driver = match Driver::headless(120, 40) {
        Ok(driver) => driver,
        Err(why) => {
            eprintln!("vitui could not attach: {why}");
            return;
        }
    };
    // Two frames and one landing, because a cold frame is not a frame and an empty pane is not a
    // pane: the frame structures take their allocation on the first frame that needs one.
    let mut ink = Direct;
    driver.frame(|cx| ui(app, &mut ink, cx));
    let _landed = app.worker.run(0);
    driver.frame(|cx| ui(app, &mut ink, cx));
    let mut tally = Tally::new();
    driver.frame(|cx| ui(app, &mut tally, cx));
    app.read_frame(&driver, Some(&tally));

    println!("vitui browse — one headless frame at 120 x 40\n");
    println!("  {}", app.top);
    println!("  {}", app.bottom);
    println!(
        "\n  writes {}  distinct {}  verbs {}  regions {}  content {}  customs {}",
        app.counters.writes,
        app.counters.distinct,
        app.counters.verbs,
        app.counters.regions,
        app.counters.content,
        app.counters.customs,
    );
    println!(
        "  spawns {}  landings {}  refused {}",
        app.worker.asked(),
        app.landings,
        app.refused,
    );
}

fn main() {
    if std::env::args().any(|a| a == "--probe") {
        let mut app = App::new(Worker::queueing());
        probe(&mut app);
        return;
    }

    // **The picture arm**: one screen, drawn headlessly and written as SVG. A program nobody
    // outside this machine can see is a program nobody believes in.
    //
    // The worker is `queueing` rather than hired, as it is in the probe arm above: the preview
    // pane's answer has to land *before* the picture is taken, and a queueing worker lands where
    // this code says it does.
    if vitui_apps::pictures::wanted() {
        let mut driver = vitui_apps::pictures::driver(120, 36, Default::default());
        let mut app = App::new(Worker::queueing());
        let mut ink = Direct;
        driver.frame(|cx| ui(&mut app, &mut ink, cx));
        let _landed = app.worker.run(0);
        for _ in 0..vitui_apps::pictures::FRAMES {
            driver.frame(|cx| ui(&mut app, &mut ink, cx));
        }
        vitui_apps::pictures::write("browse", &driver);
        return;
    }

    let mut driver = match Driver::attach(Default::default(), Default::default()) {
        Ok(driver) => driver,
        Err(why) => {
            eprintln!("vitui could not attach: {why}");
            return;
        }
    };
    // **The worker is hired against the driver's own wake handle**, so a landing wakes the app
    // thread. Without it the pane would show the answer on whatever frame happened next — which
    // for an application parked on `Driver::wait` means never.
    let mut app = App::new(Worker::hire(driver.wake()));
    app.started = Instant::now();

    loop {
        // **One call site, whichever ink.** `&mut dyn Ink` satisfies `I: Ink` through
        // `vitui_components::ink`'s blanket impl, which exists for exactly this: two calls would be
        // two `Location::caller()`s, `Ctx::id` mints from one, and toggling `v` would silently
        // replace the pane with a differently-identified widget — losing the focus and the landing
        // on a screen that renders perfectly. `explorer` is the caller that found it.
        let mut tally = Tally::new();
        let mut direct = Direct;
        let counting = app.counting;
        let mut ink: &mut dyn Ink = if counting { &mut tally } else { &mut direct };
        driver.frame(|cx| ui(&mut app, &mut ink, cx));
        app.read_frame(&driver, counting.then_some(&tally));

        let unhandled: Vec<Pressed> = driver.unhandled().to_vec();
        app.take_unhandled(&unhandled);
        if app.exit {
            break;
        }
        if !unhandled.is_empty() {
            continue;
        }
        match driver.wait() {
            Wake::Quit => break,
            Wake::Input | Wake::Posted | Wake::Deadline => {}
        }
    }
}
