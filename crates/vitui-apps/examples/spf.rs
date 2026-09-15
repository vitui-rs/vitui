//! **`spf` — superfile, drawn on the component surface.**
//!
//! ```text
//! cargo run -p vitui-apps --example spf
//! ```
//!
//! <https://github.com/yorukot/superfile>
//!
//! # A port, and the tree is the only invented part
//!
//! The shape is superfile's: a sidebar of `Home`, `Pinned` and `Disks` sections, one to three file
//! panels side by side, and a footer of three — processes, metadata, clipboard. The hotkeys are
//! `src/superfile_config/hotkeys.toml`'s own, including the ones that read oddly out of context
//! (`w` closes a panel, `n` opens one, `v` changes the panel mode, `A` selects everything). The
//! bottom border of each panel carries superfile's three border-info items — the sort, the mode
//! and `3/24`.
//!
//! **Nothing here touches a filesystem.** Sizes and dates are hashed out of each entry's own name,
//! so two runs agree and a rename is a deliberate change rather than a different random draw.
//!
//! # Four things the port cannot say, recorded rather than worked around
//!
//! 1. **The theme has twenty glyphs and none of them is a file icon.** superfile is a nerd-font
//!    application: every row carries a codepoint from a private-use range chosen by extension.
//!    The repertoire ladder is ascii · unicode · extended, and a hard-coded private-use glyph
//!    would be outside all three — so a directory here is a trailing `/` and the cursor is
//!    `Glyph::ArrowRight`, both of which degrade with the ladder. This is the one difference a
//!    reader will see first, and it is **refused rather than owed**: a `Glyph` is a lookup with a
//!    spelling at every rung and a private-use codepoint has none, so an icon in the theme would be
//!    blank at two of the three — which is the defect the complete table exists to prevent. A
//!    file's type is content the caller draws.
//! 2. **A panel's border cannot carry the search bar**, and superfile puts one there. It is a row
//!    of the interior here, shown only while `/` is open — a border caption is a string, and a
//!    search bar is a focused `field` with a caret in it.
//! 3. **Every process row spins on one phase.** `SpinState` is an anchor and the clock is
//!    `Ctx::now` — and it means two processes running at once are in step, where
//!    superfile gives each its own model. **The anchor is seeded inside the draw and not where the
//!    process is started**, which is the only place that sentence can be true: `Ctrl+V` is answered
//!    out in the loop, where nothing holds a `Ctx`, so `paste` pushes a stopped spinner and
//!    `App::tick` starts it from the frame's own `now`. Seeding it from `Instant::now()` out
//!    there draws the identical screen and puts every phase in the program outside
//!    `Driver::pin_clock`, which is the entire test regime of this workspace.
//! 4. **`→` and `←` are *open* and *parent* here, and for most of this file's life they could not
//!    be.** `hotkeys.toml` binds `confirm = ['enter', 'right', 'l']` and
//!    `parent_directory = ['h', 'left', 'backspace']`, and `vitui_components::nav::step` read `←`
//!    and `→` as `↑` and `↓` — so a focused `collection` or `table` consumed both to move its
//!    cursor and neither ever reached this application. It took the letters alone, and
//!    `commander` met the same wall in its menu bar and went round it with `Alt+←`/`Alt+→`. A
//!    group declares its axis now: a list is vertical, the two horizontal arrows are declined,
//!    and both applications bind what their originals bind.
//!
//! # What a process is here, and why it needs a deadline
//!
//! `Ctrl+V` starts a copy: a `Process` with a total, a done count and a spinner. It advances one
//! step every 140 ms, and the frame that advances it **asks for the next wake itself** —
//! `Ctx::deadline`, because a paused application must cost zero wakeups and a process
//! that finished must stop asking. That is the whole of the animation contract in an application:
//! one anchor, one clock, one deadline, and no timer thread.
//!
//! # Rename is inline, and the row drawer is where it happens
//!
//! superfile renames in place, and so does this. The row drawer is a closure, so it can hold a
//! `&mut Text` and call [`field`] for the cursor's row — which is worth saying because the
//! *shipped* row signature at four arguments and none of them is an editor. The state a container
//! needs is the caller's to capture, and `CollState::editing` is the component's own word for the
//! same idea.
//!
//! # Keys — `hotkeys.toml`'s, not invented
//!
//! | key | what |
//! |---|---|
//! | `j` `k` · arrows | down · up |
//! | `l` `Enter` | open · `h` `Backspace` the parent. **Not `→`/`←`** — see the fifth note above |
//! | `L` `H` | the next / previous file panel · `Tab` is the ring's walk over every pane · `n` opens one · `w` closes one |
//! | `s` `p` `m` | focus the sidebar · the process bar · the metadata |
//! | `v` | browser mode ⇄ select mode · `A` selects everything |
//! | `J` `K` | extend the selection in select mode · `Ctrl+Space` toggles one row. **Not `Shift+↓`/`Shift+↑`**, which move the collection's own cursor and extend its own marks — routing that into this application's marks is work nobody has asked for, and it is not `collect::owns`'s defect: the key does something |
//! | `Ctrl+C` `Ctrl+X` `Ctrl+V` | copy · cut · paste, which starts a process |
//! | `Ctrl+N` `Ctrl+R` `Ctrl+D` | create · rename in place · delete |
//! | `o` `R` | the sort menu · reverse it |
//! | `.` `F` | dot files · the footer |
//! | `/` | search inside the panel · `?` the help · `q` `Ctrl+Q` quit |

use std::fmt::Write as _;
use std::time::Duration;

use vitui_components::collect::{
    Cell, CollOpts, CollState, Column, Gesture, Mode, Selection, TableOpts, TableState, apply,
    collection, table,
};
use vitui_components::edit::Text;
use vitui_components::frame::{Face, face_role};
use vitui_components::indicate::{MeterOpts, SpinState, meter_with, spinner};
use vitui_components::input::{ButtonOpts, button_with, field};
use vitui_components::order::Rows;
use vitui_components::overlay::{Kind as ShellKind, ShellOpts, overlay_with};
use vitui_components::scroll::{Span, scrollbar};
use vitui_components::structure::{PanelOpts, panel_with};
use vitui_components::text::{FitOpts, Justify, fit_with};
use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{ActionId, Chord, Code, KeyMap, Pressed};
use vitui_runtime::layout::Constraint::{Fixed, Weight};
use vitui_runtime::layout::{Col, Row, Stack, rect};
use vitui_runtime::work::Wake;
use vitui_runtime::{Ctx, Glyph, Id, Interest, OverlayOpts, Rect, Revision, Role, Themes, Z};

// ── the tree ─────────────────────────────────────────────────────────────────────────────────────

/// A deterministic 64-bit hash. FNV-1a; two runs agree, which is the whole requirement.
fn hash(s: &str) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in s.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// One entry of the synthetic tree. Flat: every entry names its parent.
struct Item {
    parent: usize,
    name: String,
    dir: bool,
    size: u64,
    minute: u32,
    /// **Deleted, and still in the vector.** Every index here is a position in one `Vec` and
    /// shifting them would invalidate every panel at once — so a delete marks rather than removes.
    ///
    /// **It is a flag and not a reparent**, which is the second thing this file tried: pointing a
    /// deleted entry at *itself* keeps it out of every listing and turns [`Tree::path`]'s walk up
    /// the parents into an infinite loop the moment anything asks for its path. The application
    /// hung inside `Driver::frame` with a screen that looked like a dialog that would not close.
    dead: bool,
}

impl Item {
    /// Whether superfile's `.` hides it.
    fn hidden(&self) -> bool {
        self.name.starts_with('.')
    }
}

/// The tree, and the two disks the sidebar lists.
struct Tree {
    items: Vec<Item>,
}

impl Tree {
    fn seed() -> Tree {
        let mut tree = Tree {
            items: vec![Item {
                parent: 0,
                name: "/".to_owned(),
                dir: true,
                size: 0,
                minute: 0,
                dead: false,
            }],
        };
        let home = tree.dir(0, "home");
        let user = tree.dir(home, "vitaly");
        for name in [
            "Downloads",
            "Documents",
            "Pictures",
            "Videos",
            "Music",
            "projects",
        ] {
            tree.dir(user, name);
        }
        for name in [".zshrc", ".gitconfig", ".hushlogin"] {
            tree.file(user, name);
        }

        let projects = tree.find("/home/vitaly/projects");
        let vitui = tree.dir(projects, "vitui");
        for name in ["devkit", "notes", "sandbox"] {
            tree.dir(projects, name);
        }
        let crates = tree.dir(vitui, "crates");
        for name in [
            "vitui-engine",
            "vitui-runtime",
            "vitui-components",
            "vitui-apps",
        ] {
            let at = tree.dir(crates, name);
            tree.file(at, "Cargo.toml");
            tree.file(at, "README.md");
        }
        tree.dir(vitui, "docs");
        tree.dir(vitui, "scripts");
        for name in [
            "CLAUDE.md",
            "CONTEXT.md",
            "Cargo.lock",
            "Cargo.toml",
            "README.md",
            "deny.toml",
            ".gitignore",
            ".gitlab-ci.yml",
        ] {
            tree.file(vitui, name);
        }

        let downloads = tree.find("/home/vitaly/Downloads");
        for name in [
            "alacritty-0.15.1.tar.gz",
            "ghostty-1.2.0.dmg",
            "kitty-0.42.2.txz",
            "superfile-1.3.4.zip",
            "wezterm-20260812.tar.xz",
        ] {
            tree.file(downloads, name);
        }

        let documents = tree.find("/home/vitaly/Documents");
        for name in [
            "conform-report.md",
            "invoice-2026-08.pdf",
            "quirks.md",
            "terminals.csv",
        ] {
            tree.file(documents, name);
        }

        let pictures = tree.find("/home/vitaly/Pictures");
        for name in ["ghostty.png", "kitty.png", "screenshot-2026-09-04.png"] {
            tree.file(pictures, name);
        }

        let videos = tree.find("/home/vitaly/Videos");
        tree.file(videos, "demo.mp4");
        let music = tree.find("/home/vitaly/Music");
        for name in ["bruckner-8.flac", "reich-music-for-18.flac"] {
            tree.file(music, name);
        }
        tree
    }

    fn dir(&mut self, parent: usize, name: &str) -> usize {
        self.add(parent, name, true)
    }

    fn file(&mut self, parent: usize, name: &str) -> usize {
        self.add(parent, name, false)
    }

    fn add(&mut self, parent: usize, name: &str, dir: bool) -> usize {
        let h = hash(name);
        self.items.push(Item {
            parent,
            name: name.to_owned(),
            dir,
            size: if dir { 0 } else { plausible(name, h) },
            minute: u32::try_from(h % (360 * 1440)).unwrap_or(0),
            dead: false,
        });
        self.items.len() - 1
    }

    /// The absolute path of an entry.
    fn path(&self, at: usize) -> String {
        let mut parts = Vec::new();
        let mut cur = at;
        // **Bounded, and the bound is the tree itself.** A walk up the parents terminates only if
        // no entry is its own ancestor, and nothing in the type says so — see [`Item::dead`] for
        // the day that stopped being true. A cap degrades into a wrong path; the loop it replaces
        // degrades into a frame that never returns.
        for _ in 0..self.items.len() {
            if cur == 0 {
                break;
            }
            parts.push(self.items[cur].name.as_str());
            cur = self.items[cur].parent;
        }
        if parts.is_empty() {
            return "/".to_owned();
        }
        let mut out = String::new();
        for part in parts.iter().rev() {
            out.push('/');
            out.push_str(part);
        }
        out
    }

    /// The entry at a path, or the root.
    fn find(&self, path: &str) -> usize {
        (0..self.items.len())
            .find(|&i| !self.items[i].dead && self.path(i) == path)
            .unwrap_or(0)
    }

    /// `2026-04-24 19:53`, off an invented calendar of twelve thirty-day months.
    fn stamp(&self, at: usize, out: &mut String) {
        let m = self.items[at].minute;
        let days = m / 1440;
        out.clear();
        let _ = write!(
            out,
            "2026-{:02}-{:02} {:02}:{:02}",
            days / 30 % 12 + 1,
            days % 30 + 1,
            m % 1440 / 60,
            m % 60,
        );
    }
}

/// A size the extension makes believable.
///
/// **Hashing a name into one range gives a `deny.toml` of 8.5 MB**, which is the kind of detail
/// that makes invented data read as invented. The bands are the file manager's own intuition: a
/// manifest is kilobytes, a photograph is megabytes, a video is hundreds of them.
fn plausible(name: &str, h: u64) -> u64 {
    let (lo, span) = match extension(name) {
        "md" | "toml" | "txt" | "rs" | "csv" | "yml" | "gitignore" => (900, 90_000),
        "lock" => (60_000, 400_000),
        "png" | "jpg" => (120_000, 3_000_000),
        "pdf" => (200_000, 6_000_000),
        "mp4" => (120_000_000, 800_000_000),
        "flac" => (20_000_000, 40_000_000),
        "gz" | "xz" | "txz" | "zip" | "dmg" => (5_000_000, 90_000_000),
        _ => (200, 400_000),
    };
    lo + h % span
}

/// `9.4 MB`, `812 kB`, `40 B` — superfile prints a scaled size, not a byte count.
fn human(bytes: u64, out: &mut String) {
    out.clear();
    let _ = if bytes >= 1_000_000_000 {
        write!(out, "{:.1} GB", bytes as f64 / 1e9)
    } else if bytes >= 1_000_000 {
        write!(out, "{:.1} MB", bytes as f64 / 1e6)
    } else if bytes >= 1_000 {
        write!(out, "{:.1} kB", bytes as f64 / 1e3)
    } else {
        write!(out, "{bytes} B")
    };
}

// ── the sidebar ──────────────────────────────────────────────────────────────────────────────────

/// One row of the sidebar: a section heading, or a place to go.
enum Side {
    /// `Home ────────`, `Pinned ──────`, `Disks ───────`.
    Divider(&'static str),
    /// A directory, with the label the sidebar shows.
    Place(&'static str, &'static str),
}

/// The sidebar, in superfile's own three sections.
const SIDEBAR: [Side; 15] = [
    Side::Divider("Home"),
    Side::Place("Home", "/home/vitaly"),
    Side::Place("Downloads", "/home/vitaly/Downloads"),
    Side::Place("Documents", "/home/vitaly/Documents"),
    Side::Place("Pictures", "/home/vitaly/Pictures"),
    Side::Place("Videos", "/home/vitaly/Videos"),
    Side::Place("Music", "/home/vitaly/Music"),
    Side::Divider("Pinned"),
    Side::Place("projects", "/home/vitaly/projects"),
    Side::Place("vitui", "/home/vitaly/projects/vitui"),
    Side::Place("crates", "/home/vitaly/projects/vitui/crates"),
    Side::Divider("Disks"),
    Side::Place("Macintosh HD", "/"),
    Side::Place("home", "/home"),
    Side::Place("backup", "/home/vitaly/Documents"),
];

// ── a file panel ─────────────────────────────────────────────────────────────────────────────────

/// superfile's five sort keys, from `sortmodel`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Sort {
    Name,
    Size,
    Date,
    Type,
}

impl Sort {
    const ALL: [Sort; 4] = [Sort::Name, Sort::Size, Sort::Date, Sort::Type];

    const fn word(self) -> &'static str {
        match self {
            Sort::Name => "Name",
            Sort::Size => "Size",
            Sort::Date => "Date Modified",
            Sort::Type => "Type",
        }
    }
}

/// superfile's two panel modes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PanelMode {
    Browser,
    Select,
}

/// One file panel.
struct FilePanel {
    cwd: usize,
    rows: Vec<usize>,
    st: TableState,
    /// The select-mode selection. Its own store, because `Mode::Cursor` never selects and an arrow
    /// key must not throw away what `Shift+↓` built.
    marks: Selection,
    sort: Sort,
    desc: bool,
    mode: PanelMode,
    /// The panel's own search, superfile's `/`.
    search: String,
    rev: Revision,
    id: Option<Id>,
}

impl FilePanel {
    fn new(cwd: usize) -> FilePanel {
        FilePanel {
            cwd,
            rows: Vec::new(),
            st: TableState::new(),
            marks: Selection::new(),
            sort: Sort::Name,
            desc: false,
            mode: PanelMode::Browser,
            search: String::new(),
            rev: Revision::UNKNOWN,
            id: None,
        }
    }

    /// Rebuild the listing: the dot-file rule, then the search, then the sort.
    fn relist(&mut self, tree: &Tree, dots: bool) {
        let at = self.st.coll.sel.lead;
        let was = self.rows.get(at).copied();
        // **The selection is carried by identity, not by position.** Toggling dot files or
        // re-sorting must not silently drop what select mode built — the row indices move and the
        // entries do not, so the marks are remapped through the entries they name.
        let marked: Vec<usize> = self
            .rows
            .iter()
            .enumerate()
            .filter(|(i, _)| self.marks.contains(*i))
            .map(|(_, at)| *at)
            .collect();
        let keep = self.search.to_lowercase();
        let mut rows: Vec<usize> = (1..tree.items.len())
            .filter(|&i| !tree.items[i].dead && tree.items[i].parent == self.cwd)
            .filter(|&i| dots || !tree.items[i].hidden())
            .filter(|&i| keep.is_empty() || tree.items[i].name.to_lowercase().contains(&keep))
            .collect();
        rows.sort_by(|&a, &b| {
            let (ia, ib) = (&tree.items[a], &tree.items[b]);
            let dir = (ib.dir).cmp(&(ia.dir));
            let key = match self.sort {
                Sort::Name => ia.name.to_lowercase().cmp(&ib.name.to_lowercase()),
                Sort::Size => ib.size.cmp(&ia.size),
                Sort::Date => ib.minute.cmp(&ia.minute),
                Sort::Type => extension(&ia.name)
                    .cmp(extension(&ib.name))
                    .then_with(|| ia.name.cmp(&ib.name)),
            };
            dir.then(if self.desc { key.reverse() } else { key })
        });
        self.rows = rows;
        self.marks.clear();
        for at in marked {
            if let Some(i) = self.rows.iter().position(|x| *x == at) {
                self.marks.toggle(i);
            }
        }
        let last = self.rows.len().saturating_sub(1);
        // **The row that vanished leaves its position behind, not the top of the list.** Deleting
        // the entry above the cursor and being thrown onto `..` is what every file manager is
        // judged on not doing; `mc` keeps the index and clamps it.
        self.st.coll.sel.lead = was
            .and_then(|r| self.rows.iter().position(|x| *x == r))
            .unwrap_or(at)
            .min(last);
        self.rev = Revision::fresh();
    }

    /// The entry under the cursor.
    fn cursor(&self) -> Option<usize> {
        self.rows.get(self.st.coll.sel.lead).copied()
    }

    /// What an operation acts on: the selection in select mode, the cursor otherwise.
    fn targets(&self) -> Vec<usize> {
        if self.mode == PanelMode::Select && !self.marks.is_empty() {
            return self
                .rows
                .iter()
                .enumerate()
                .filter(|(i, _)| self.marks.contains(*i))
                .map(|(_, at)| *at)
                .collect();
        }
        self.cursor().into_iter().collect()
    }
}

/// The part of a name after its last dot, or `""`.
fn extension(name: &str) -> &str {
    name.rsplit_once('.').map_or("", |(_, ext)| ext)
}

/// The three columns superfile shows when a panel is wide enough for them.
const COLS: [Column; 3] = [
    Column::new(0, "Name", Weight(1)),
    Column::new(1, "Size", Fixed(10)),
    Column::new(2, "Date Modified", Fixed(18)),
];

/// The metadata panel's two columns.
const META_COLS: [Column; 2] = [
    Column::new(0, "Key", Fixed(14)),
    Column::new(1, "Value", Weight(1)),
];

// ── the footer ───────────────────────────────────────────────────────────────────────────────────

/// **One ladder frame per this**, for every process bar in the program.
///
/// One constant rather than a literal at the `start` call, because two processes running at once
/// are in step here — note 4 in the header — and a second literal is how that stops being true.
const SPIN_PER: Duration = Duration::from_millis(120);

/// One running operation, which is what `Ctrl+V` starts.
struct Process {
    what: String,
    done: u32,
    total: u32,
    /// **Stopped when the process is pushed**, and started by `App::tick` from the frame's own
    /// `Ctx::now`: an anchor is the application's and a clock is not.
    spin: SpinState,
}

/// What is in the clipboard, and whether it was cut.
struct Clipboard {
    items: Vec<usize>,
    cut: bool,
}

// ── what is open, and what it decided ────────────────────────────────────────────────────────────

/// The one modal that can be open at a time.
enum Modal {
    None,
    Help,
    /// `Ctrl+N`.
    Create(Text),
    /// `Ctrl+D`, over this many entries.
    Delete(usize),
    /// `o` — the sort menu.
    SortMenu(CollState),
}

/// **What a modal decided, written inside a frame and read after it.**
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Act {
    Close,
    /// A click in the sidebar asked to go somewhere.
    Sidebar,
    Create,
    Delete,
    Sort(usize),
}

/// Where the keyboard is. superfile's `s`, `p` and `m` move it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Focus {
    Sidebar,
    Panel,
    Process,
    Metadata,
}

/// The help modal's text: `hotkeys.toml`'s own names beside its own keys.
const HELP: [(&str, &str); 19] = [
    ("j / k / ↓ / ↑", "list down · list up"),
    ("l / enter / →", "confirm — open"),
    ("h / backspace / ←", "parent directory"),
    (
        "L / H",
        "next / previous file panel — `tab` walks every pane",
    ),
    ("n / w", "create · close a file panel"),
    ("s / p / m", "focus sidebar · process bar · metadata"),
    ("v", "change panel mode"),
    ("A", "select all items"),
    ("J / K", "extend the selection — shift+arrows are eaten"),
    ("ctrl+space", "toggle one row in select mode"),
    ("ctrl+c / ctrl+x", "copy · cut"),
    ("ctrl+v", "paste — this starts a process"),
    ("ctrl+d", "delete — tab, then enter: it opens on Cancel"),
    ("ctrl+n / ctrl+r", "create · rename in place"),
    ("o / R", "sort options · reverse sort"),
    (".", "toggle dot files"),
    ("F", "toggle the footer"),
    ("/", "search inside the panel"),
    ("? / q", "this menu · quit"),
];

// ── the keyboard, as a map ───────────────────────────────────────────────────────────────────────

/// **`hotkeys.toml`'s bindings as a [`KeyMap`], and it is a map rather than a match on `k.code` for
/// the same reason.**
///
/// **One keystroke has four wire spellings**, and a terminal speaking the enhanced keyboard
/// protocol picks the last two: `?` arrives as `CSI 47;2;63u` — the base key `/`, the shift bit,
/// and `?` as the text — or as `CSI 47;2u`, which says `/` and shift and nothing about `?` at all.
/// A match on `Code::Char('?')` sees neither, and the same is true of every capital
/// `hotkeys.toml` binds: `J`, `K`, `L`, `H`, `A`, `R`, `F`. Read off `k.code`, half this
/// application's keyboard is dead on a modern terminal and perfect on a legacy one — which is what
/// a headless gate and a hand test in one emulator both are.
///
/// [`Chord::typed`] covers three of the four: it compares the **text the terminal says was
/// produced** and masks the shift bit, because on a character that bit is *how it was made* and not
/// intent. The fourth needs an alternate — `Chord::key('j').shift()` — and it is a second binding
/// rather than a repair, because nothing in the process knows a `J` was produced.
///
/// **Order is precedence** ([`KeyMap::match_first`] takes the first match) and it is load-bearing
/// for every one of the five shifted/unshifted pairs: `J` before `j`, `L` before `l`, `H` before
/// `h`, `?` before `/`. The alternate for the shifted key is a chord on the *unshifted* one, so the
/// unshifted binding would otherwise swallow it.
/// **`?`, on every wire spelling a terminal can send it as, in one place.**
///
/// [`key_map`]'s own `shifted` helper is a closure inside that function and is not reachable from a
/// draw, so the help modal's *close* arm re-spelled the pair by hand — a second place for one of the
/// two arms to go missing, in a file written to record that exact defect. Worse, the hand-written
/// version was a `match` on `k.code`, which misses a spelling [`Chord::typed`] catches: a terminal
/// speaking the enhanced protocol may report the base key `/` **and** the text `?`, and
/// `Code::Char('?')` sees neither half of that.
const HELP_KEYS: [Chord; 2] = [Chord::typed('?'), Chord::key('/').shift()];

/// Whether a keystroke is one of [`HELP_KEYS`].
///
/// `MatchMode::Masked` is what ships and what [`KeyMap`] uses, so the two readings of `?` in this
/// file cannot disagree about the lock bits either.
fn is_help_key(k: &Pressed) -> bool {
    HELP_KEYS
        .iter()
        .any(|c| c.matches(k, vitui_runtime::keys::MatchMode::Masked))
}

fn key_map() -> KeyMap {
    let ctrl = |c: char| Chord::key(c).ctrl();
    let shifted = |shift: char, base: char| [Chord::typed(shift), Chord::key(base).shift()];
    KeyMap::new()
        .bind(&[ctrl('q')], QUIT, "Quit")
        .bind(&[ctrl('c')], COPY, "Copy")
        .bind(&[ctrl('x')], CUT, "Cut")
        .bind(&[ctrl('v')], PASTE, "Paste")
        .bind(&[ctrl('d')], DELETE, "Delete")
        .bind(&[ctrl('n')], CREATE, "Create")
        .bind(&[ctrl('r')], RENAME, "Rename")
        .bind(&[ctrl(' ')], TOGGLE, "Toggle one row")
        // The shifted half of every pair, first.
        .bind(&shifted('J', 'j'), EXTEND_DOWN, "Extend down")
        .bind(&shifted('K', 'k'), EXTEND_UP, "Extend up")
        .bind(&shifted('L', 'l'), NEXT_PANEL, "Next file panel")
        .bind(&shifted('H', 'h'), PREV_PANEL, "Previous file panel")
        .bind(&shifted('A', 'a'), SELECT_ALL, "Select all")
        .bind(&shifted('R', 'r'), REVERSE, "Reverse sort")
        .bind(&shifted('F', 'f'), FOOTER, "Toggle footer")
        .bind(&shifted('?', '/'), SHOW_HELP, "Help")
        // …then the unshifted one.
        .bind(&[Chord::typed('q')], QUIT, "Quit")
        .bind(&[Chord::typed('j'), Chord::new(Code::Down)], DOWN, "Down")
        .bind(&[Chord::typed('k'), Chord::new(Code::Up)], UP, "Up")
        // **The arrows are here, and for most of this file's life they could not be.** A focused
        // `collection` or `table` read `←` and `→` as `↑` and `↓` and consumed both to move its
        // cursor, so `hotkeys.toml`'s own spelling of *open* and *parent* was unreachable and this
        // application took the letters alone. A group declares its axis now: a list is vertical,
        // the horizontal arrows are declined, and they reach this map.
        .bind(
            &[
                Chord::typed('l'),
                Chord::new(Code::Enter),
                Chord::new(Code::Right),
            ],
            OPEN,
            "Open",
        )
        .bind(
            &[
                Chord::typed('h'),
                Chord::new(Code::Backspace),
                Chord::new(Code::Left),
            ],
            PARENT,
            "Parent",
        )
        .bind(&[Chord::typed('n')], NEW_PANEL, "New file panel")
        .bind(&[Chord::typed('w')], CLOSE_PANEL, "Close file panel")
        .bind(&[Chord::typed('s')], FOCUS_SIDEBAR, "Focus sidebar")
        .bind(&[Chord::typed('p')], PROCESS, "Focus processes")
        .bind(&[Chord::typed('m')], METADATA, "Focus metadata")
        .bind(&[Chord::typed('v')], MODE, "Panel mode")
        .bind(&[Chord::typed('o')], SORT, "Sort options")
        .bind(&[Chord::typed('.')], DOTS, "Dot files")
        .bind(&[Chord::typed('/')], SEARCH, "Search")
        .bind(&[Chord::new(Code::Escape)], BACK, "Back to the panel")
}

const QUIT: ActionId = 1;
const COPY: ActionId = 2;
const CUT: ActionId = 3;
const PASTE: ActionId = 4;
const DELETE: ActionId = 5;
const CREATE: ActionId = 6;
const RENAME: ActionId = 7;
const TOGGLE: ActionId = 8;
const EXTEND_DOWN: ActionId = 9;
const EXTEND_UP: ActionId = 10;
const NEXT_PANEL: ActionId = 11;
const PREV_PANEL: ActionId = 12;
const SELECT_ALL: ActionId = 13;
const REVERSE: ActionId = 14;
const FOOTER: ActionId = 15;
const SHOW_HELP: ActionId = 16;
const DOWN: ActionId = 17;
const UP: ActionId = 18;
const OPEN: ActionId = 19;
const PARENT: ActionId = 20;
const NEW_PANEL: ActionId = 21;
const CLOSE_PANEL: ActionId = 22;
const FOCUS_SIDEBAR: ActionId = 23;
const PROCESS: ActionId = 24;
const METADATA: ActionId = 25;
const MODE: ActionId = 26;
const SORT: ActionId = 27;
const DOTS: ActionId = 28;
const SEARCH: ActionId = 29;
const BACK: ActionId = 30;

/// **Every [`ActionId`] above is distinct, and the compiler says so.**
///
/// Hand-numbered ids are the one place in this file where a copy-paste is silent: two actions on one
/// number is a chord that fires the wrong verb, and nothing objects — [`KeyMap`] has no reason to,
/// and a wrong verb draws a perfectly good screen. A `const` block is the cheapest instrument that
/// cannot be forgotten, and it is the shape `latency`'s two word-table length asserts already have.
const _: () = {
    let ids = [
        QUIT,
        COPY,
        CUT,
        PASTE,
        DELETE,
        CREATE,
        RENAME,
        TOGGLE,
        EXTEND_DOWN,
        EXTEND_UP,
        NEXT_PANEL,
        PREV_PANEL,
        SELECT_ALL,
        REVERSE,
        FOOTER,
        SHOW_HELP,
        DOWN,
        UP,
        OPEN,
        PARENT,
        NEW_PANEL,
        CLOSE_PANEL,
        FOCUS_SIDEBAR,
        PROCESS,
        METADATA,
        MODE,
        SORT,
        DOTS,
        SEARCH,
        BACK,
    ];
    let mut i = 0;
    while i < ids.len() {
        let mut j = i + 1;
        while j < ids.len() {
            assert!(ids[i] != ids[j], "two actions share one `ActionId`");
            j += 1;
        }
        i += 1;
    }
};

// ── the application ──────────────────────────────────────────────────────────────────────────────

/// Everything this program knows.
struct App {
    tree: Tree,
    panels: Vec<FilePanel>,
    /// Which file panel is the focused one.
    panel: usize,
    focus: Focus,
    /// Set when *this application* moved the focus word — `s`, `p`, `m`, `L`, `H`, `n`, `w`, `Esc`.
    /// Cleared by [`App::seat_focus`], which is what stops the seat fighting the ring's own walk.
    seat: bool,
    /// The sidebar's own cursor, and where it was on the previous frame — which is the direction
    /// [`App::skip_divider`] steps in.
    side: CollState,
    side_was: usize,
    side_id: Option<Id>,
    /// The process bar and the metadata panel each own a cursor too, and each is a place `p` and
    /// `m` can put the keyboard — so each has to answer with the id it drew under.
    proc: CollState,
    proc_id: Option<Id>,
    meta: TableState,
    meta_id: Option<Id>,
    processes: Vec<Process>,
    clipboard: Clipboard,
    dots: bool,
    footer: bool,
    /// The `/` search field, open over the focused panel's search row.
    searching: bool,
    search: Text,
    /// The inline rename, and the row it is on.
    renaming: Option<(usize, Text)>,
    modal: Modal,
    pending: Option<Act>,
    /// What the last action said, on the metadata panel's last row.
    flash: String,
    exit: bool,
    /// The bindings, built once. See [`key_map`] for why this is a map and not a match.
    keys: KeyMap,
    /// Scratch buffers, so a frame formats into two allocations rather than into every cell.
    size_buf: String,
    date_buf: String,
}

impl App {
    fn new() -> App {
        let tree = Tree::seed();
        let start = tree.find("/home/vitaly/projects/vitui");
        let mut app = App {
            tree,
            panels: vec![FilePanel::new(start)],
            panel: 0,
            focus: Focus::Panel,
            seat: true,
            side: CollState::new(),
            side_was: 9,
            side_id: None,
            proc: CollState::new(),
            proc_id: None,
            meta: TableState::new(),
            meta_id: None,
            processes: Vec::new(),
            clipboard: Clipboard {
                items: Vec::new(),
                cut: false,
            },
            dots: false,
            footer: true,
            searching: false,
            search: Text::input(),
            renaming: None,
            modal: Modal::None,
            pending: None,
            flash: "superfile on vitui — ? for the hotkeys".to_owned(),
            exit: false,
            keys: key_map(),
            size_buf: String::with_capacity(16),
            date_buf: String::with_capacity(24),
        };
        // The sidebar opens on the pinned `vitui`, which is where the first panel is.
        app.side.sel.lead = 9;
        app.relist_all();
        app
    }

    /// Rebuild every panel's listing.
    fn relist_all(&mut self) {
        let (tree, dots) = (&self.tree, self.dots);
        for panel in &mut self.panels {
            panel.relist(tree, dots);
        }
    }
}

// ── the drawing ──────────────────────────────────────────────────────────────────────────────────

impl App {
    /// One frame, top to bottom.
    fn ui<'f>(&'f mut self, cx: &mut Ctx<'f, '_>) {
        // **The processes advance here and ask for their own next wake.** One clock — `Ctx::now`
        // — and one deadline, so a screen with nothing running costs zero wakeups.
        self.tick(cx);
        self.skip_divider();
        self.adopt_focus(cx);

        let whole = cx.area();
        let foot_rows = if self.footer { 9.min(whole.h / 3) } else { 0 };
        let [body, foot] = Col::new().split(whole, [Weight(1), Fixed(foot_rows)]);
        let side_w = 22.min(body.w / 3);
        let (side, panels) = rect::split_at_h(body, side_w);

        self.draw_sidebar(cx, side);
        self.draw_panels(cx, panels);
        if self.footer {
            self.draw_footer(cx, foot);
        }
        self.seat_focus(cx);
        self.draw_modal(cx);
    }

    /// Advance every running process, and ask for the frame that will advance it again.
    fn tick(&mut self, cx: &mut Ctx<'_, '_>) {
        if self.processes.is_empty() {
            return;
        }
        let now = cx.now();
        let mut running = false;
        for p in &mut self.processes {
            if p.done < p.total {
                // **The anchor is seeded here and nowhere else**, because this is the only place in
                // the program holding a `Ctx`. A `SpinState` started from `Instant::now()` in the
                // driver loop is a phase the application sampled itself, and the whole point of a caller-owned anchor
                // is that such a phase is invisible to `Driver::pin_clock` — the frame would still
                // draw, and every spinner in it would be outside the one instrument that can hold a
                // clock still. `SpinState::spinning` is what makes the seeding idempotent, so the
                // frame a process is pushed on is also the frame it starts turning on.
                if !p.spin.spinning() {
                    p.spin.start(now, SPIN_PER);
                }
                p.done += 1;
                running = true;
                if p.done >= p.total {
                    p.spin.stop();
                }
            }
        }
        if running {
            // **The application asks, and only while something is moving.** A body that asked
            // every frame would be a runaway in the wake census and would look identical on screen.
            cx.deadline(now + Duration::from_millis(140));
        }
    }

    /// The sidebar: the title, the search row and the three sections.
    fn draw_sidebar(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) {
        let focused = self.focus == Focus::Sidebar;
        let panel = panel_with(
            cx,
            area,
            " superfile ",
            "",
            &PanelOpts {
                border: if focused { Role::Focus } else { Role::Border },
                padded: false,
                ..Default::default()
            },
        );
        if panel.interior.h < 2 {
            return;
        }
        let [head, list] = Col::new().split(panel.interior, [Fixed(1), Weight(1)]);
        fit_with(
            cx,
            head,
            " a file manager, ported ",
            &FitOpts {
                justify: Justify::Start,
                role: Role::Dim,
                pad: Role::Dim,
            },
        );

        let arrow = cx.theme().glyph(Glyph::ArrowRight);
        let line = cx.theme().glyph(Glyph::HLine);
        let here = self.panels[self.panel].cwd;
        let tree = &self.tree;
        let opts = CollOpts {
            mode: Mode::Cursor,
            ..CollOpts::default()
        };
        let resp = collection(
            cx,
            list,
            &mut self.side,
            &opts,
            Rows::of(SIDEBAR.len()),
            // Type-ahead is off for the same reason it is off everywhere here: every letter is a
            // hotkey, and a search that answered would eat it.
            &mut |_buf, _range| None,
            &mut |cx, r, i, face| {
                match &SIDEBAR[i] {
                    // **A divider is a heading and a rule on one row**, which is what superfile's
                    // `SideBarHomeDivider` is: a label, then the line to the panel's edge.
                    Side::Divider(name) => {
                        let cut = u16::try_from(name.len() + 2).unwrap_or(8).min(r.w);
                        let (label, rest) = rect::split_at_h(r, cut);
                        fit_with(
                            cx,
                            label,
                            &format!(" {name} "),
                            &FitOpts {
                                justify: Justify::Start,
                                role: Role::Title,
                                pad: Role::Title,
                            },
                        );
                        cx.fill(rest, line, cx.theme().paint(Role::Border));
                    }
                    Side::Place(label, path) => {
                        let role = match face_role(face) {
                            Role::Body if tree.find(path) == here => Role::Ok,
                            other => other,
                        };
                        let mark = if face.cursor { arrow } else { " " };
                        fit_with(
                            cx,
                            r,
                            &format!(" {mark} {label}"),
                            &FitOpts {
                                justify: Justify::Start,
                                role,
                                pad: role,
                            },
                        );
                    }
                }
            },
        );
        self.side_id = Some(resp.id);
        if resp.clicked {
            // **Through the inbox, like every other decision.** A `cwd` changed inside the draw is
            // a `cwd` the frame that has already run cannot show, and nothing asks for the next
            // one — the panel would follow the click on the *following* keystroke.
            self.pending = Some(Act::Sidebar);
        }
    }

    /// One to three file panels, side by side.
    fn draw_panels(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) {
        let n = self.panels.len();
        let mut lanes = [Rect::default(); 3];
        let spec: Vec<_> = (0..n).map(|_| Weight(1)).collect();
        Row::new().split_into(area, &spec, &mut lanes);
        for (i, &lane) in lanes.iter().enumerate().take(n) {
            // **Every panel is drawn from one call site, so every panel would mint one id.**
            // `Ctx::id` is `Location::caller()`; `with_key` is what makes three panels three
            // widgets rather than one that three panels fight over.
            cx.with_key(i as u64, |cx| self.draw_panel(cx, lane, i));
        }
    }

    /// One file panel: the path, the search row, the table and the footer items.
    fn draw_panel(&mut self, cx: &mut Ctx<'_, '_>, area: Rect, which: usize) {
        let focused = self.focus == Focus::Panel && which == self.panel;
        let path = self.tree.path(self.panels[which].cwd);
        let frame = panel_with(
            cx,
            area,
            &format!(" {path} "),
            &self.info_line(cx, which),
            &PanelOpts {
                border: if focused { Role::Focus } else { Role::Border },
                padded: false,
                ..Default::default()
            },
        );
        if frame.interior.h < 3 {
            return;
        }
        let search_rows = u16::from(self.searching && focused);
        let [bar, list] = Col::new().split(frame.interior, [Fixed(search_rows), Weight(1)]);

        if search_rows == 1 {
            let cut = 3.min(bar.w);
            let (mark, entry) = rect::split_at_h(bar, cut);
            fit_with(
                cx,
                mark,
                " / ",
                &FitOpts {
                    justify: Justify::Start,
                    role: Role::Warn,
                    pad: Role::Warn,
                },
            );
            let resp = field(cx, entry, &mut self.search);
            if !cx.is_focused(resp.id) {
                cx.focus(resp.id);
            }
        }

        let App {
            tree,
            panels,
            renaming,
            size_buf,
            date_buf,
            ..
        } = self;
        let state = &mut panels[which];
        let rows = &state.rows;
        let marks = &state.marks;
        let mode = state.mode;
        let mut renaming = if focused { renaming.as_mut() } else { None };
        let opts = TableOpts {
            header: list.h > 3,
            coll: CollOpts {
                mode: Mode::Cursor,
                ..CollOpts::default()
            },
        };
        let resp = table(
            cx,
            list,
            &mut state.st,
            &opts,
            &COLS,
            Rows::new(rows.len(), state.rev),
            &mut |_buf, _range| None,
            &mut |cx, r, cell: Cell, face: Face| {
                let Some(&at) = rows.get(cell.row) else {
                    return;
                };
                let face = Face {
                    selected: marks.contains(cell.row),
                    ..face
                };
                let role = face_role(face);
                let item = &tree.items[at];
                // **The rename is drawn where the name would be**, which is superfile's own
                // arrangement. The row drawer is a closure, so it can hold the `&mut Text` that
                // The four-argument row signature has no room for.
                if cell.key == 0
                    && let Some((row, buf)) = renaming.as_deref_mut()
                    && *row == cell.row
                {
                    let _ = field(cx, r, buf);
                    return;
                }
                let (text, justify) = match cell.key {
                    0 => {
                        let box_ = match (mode, marks.contains(cell.row)) {
                            (PanelMode::Browser, _) => "",
                            (PanelMode::Select, true) => "[x] ",
                            (PanelMode::Select, false) => "[ ] ",
                        };
                        let slash = if item.dir { "/" } else { "" };
                        (format!(" {box_}{}{slash}", item.name), Justify::Start)
                    }
                    1 if item.dir => ("— ".to_owned(), Justify::End),
                    1 => {
                        human(item.size, size_buf);
                        (format!("{size_buf} "), Justify::End)
                    }
                    _ => {
                        tree.stamp(at, date_buf);
                        (format!("{date_buf} "), Justify::End)
                    }
                };
                let role = match (role, item.dir, item.hidden()) {
                    (Role::Body, true, _) => Role::Title,
                    (Role::Body, false, true) => Role::Disabled,
                    (other, _, _) => other,
                };
                cell_into(cx, r, &text, justify, role);
            },
        );
        state.id = Some(resp.id);
        // **A double click would be `Enter`, and it was not observed to arrive.** The component
        // declares `Interest::CLICK` and returns `cx.scrollable`'s own `Response`, so the field is
        // wired; two clicks 30, 80 and 250 ms apart on one row — all inside the 400 ms threshold —
        // report `clicked` and never `double_clicked`. It is left connected rather than deleted
        // because the day it fires this is the line that wanted it, and the keyboard route is the
        // documented one.
        let open = resp.double_clicked;

        if open {
            self.open();
        }
    }

    /// **superfile's three border-info items**: the sort, the panel mode and the position.
    ///
    /// One string in the bottom border, which is where superfile draws them and where this port
    /// could not draw them for most of its life — they were an interior row and cost the listing a
    /// line. The separator is the theme's own vertical rule rather than a literal, so it degrades
    /// with the repertoire the way every other glyph on the screen does.
    fn info_line(&self, cx: &Ctx<'_, '_>, which: usize) -> String {
        let panel = &self.panels[which];
        let n = panel.rows.len();
        let at = panel.st.coll.sel.lead;
        let arrow = if panel.desc { "desc" } else { "asc" };
        let mode = match panel.mode {
            PanelMode::Browser => "Browser".to_owned(),
            PanelMode::Select => format!("Select ({})", panel.marks.count()),
        };
        let sep = cx.theme().glyph(Glyph::VLine);
        let sort = panel.sort.word();
        let counter = format!("{}/{n}", if n == 0 { 0 } else { at + 1 });
        format!(" {sort} {arrow} {sep} {mode} {sep} {counter} ")
    }

    /// The three footer panels: processes, metadata, clipboard.
    fn draw_footer(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) {
        let [procs, meta, clip] = Row::new().split(area, [Weight(1), Weight(1), Weight(1)]);
        self.draw_processes(cx, procs);
        self.draw_metadata(cx, meta);
        self.draw_clipboard(cx, clip);
    }

    /// The process bar: a spinner, a name, a meter and `3/12`.
    fn draw_processes(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) {
        let focused = self.focus == Focus::Process;
        let frame = panel_with(
            cx,
            area,
            " Processes ",
            "",
            &PanelOpts {
                border: if focused { Role::Focus } else { Role::Border },
                padded: false,
                ..Default::default()
            },
        );
        if frame.interior.is_empty() {
            return;
        }
        if self.processes.is_empty() {
            self.proc_id = None;
            fit_with(
                cx,
                frame.interior,
                " no processes running",
                &FitOpts {
                    justify: Justify::Start,
                    role: Role::Dim,
                    pad: Role::Dim,
                },
            );
            return;
        }
        let processes = &self.processes;
        let opts = CollOpts {
            mode: Mode::Cursor,
            ..CollOpts::default()
        };
        // Two rows a process — the label and the bar — so the list is scrolled by pairs, which is
        // what `Rows::of` counts here.
        let resp = collection(
            cx,
            frame.interior,
            &mut self.proc,
            &opts,
            Rows::of(processes.len() * 2),
            &mut |_buf, _range| None,
            &mut |cx, r, i, face| {
                let p = &processes[i / 2];
                let role = face_role(face);
                if i % 2 == 0 {
                    let cut = 3.min(r.w);
                    let (mark, rest) = rect::split_at_h(r, cut);
                    // **A spinner is an anchor and the clock is `Ctx::now`**: it stops
                    // when the work does, and a stopped spinner asks for no frames.
                    spinner(cx, mark, &p.spin, "");
                    fit_with(
                        cx,
                        rest,
                        &p.what,
                        &FitOpts {
                            justify: Justify::Start,
                            role,
                            pad: role,
                        },
                    );
                } else {
                    let [bar, count] = Row::new().split(r, [Weight(1), Fixed(9)]);
                    let share = f32::from(u16::try_from(p.done).unwrap_or(0))
                        / f32::from(u16::try_from(p.total.max(1)).unwrap_or(1));
                    meter_with(
                        cx,
                        bar,
                        share,
                        &MeterOpts {
                            done: if p.done >= p.total {
                                Role::Ok
                            } else {
                                Role::Warn
                            },
                            rest: Role::Border,
                            ..Default::default()
                        },
                    );
                    fit_with(
                        cx,
                        count,
                        &format!(" {}/{} ", p.done, p.total),
                        &FitOpts {
                            justify: Justify::End,
                            role: Role::Dim,
                            pad: Role::Dim,
                        },
                    );
                }
            },
        );
        self.proc_id = Some(resp.id);
    }

    /// The metadata panel: what the cursor is on, as a two-column table.
    fn draw_metadata(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) {
        let focused = self.focus == Focus::Metadata;
        let frame = panel_with(
            cx,
            area,
            " Metadata ",
            "",
            &PanelOpts {
                border: if focused { Role::Focus } else { Role::Border },
                padded: false,
                ..Default::default()
            },
        );
        if frame.interior.h < 2 {
            return;
        }
        let [rows_area, flash] = Col::new().split(frame.interior, [Weight(1), Fixed(1)]);
        let facts: Vec<(&'static str, String)> = match self.panels[self.panel].cursor() {
            None => vec![("", "nothing under the cursor".to_owned())],
            Some(at) => {
                let item = &self.tree.items[at];
                human(item.size, &mut self.size_buf);
                self.tree.stamp(at, &mut self.date_buf);
                vec![
                    ("Name", item.name.clone()),
                    (
                        "Kind",
                        if item.dir {
                            "directory".to_owned()
                        } else {
                            format!("{} file", extension(&item.name))
                        },
                    ),
                    ("Size", self.size_buf.clone()),
                    ("Modified", self.date_buf.clone()),
                    ("Location", self.tree.path(item.parent)),
                    (
                        "Permissions",
                        if item.dir { "drwxr-xr-x" } else { "-rw-r--r--" }.to_owned(),
                    ),
                ]
            }
        };
        let opts = TableOpts {
            header: false,
            coll: CollOpts {
                mode: Mode::Cursor,
                ..CollOpts::default()
            },
        };
        let facts = &facts;
        let resp = table(
            cx,
            rows_area,
            &mut self.meta,
            &opts,
            &META_COLS,
            Rows::of(facts.len()),
            &mut |_buf, _range| None,
            &mut |cx, r, cell: Cell, face: Face| {
                let Some((key, value)) = facts.get(cell.row) else {
                    return;
                };
                let base = face_role(face);
                let (text, role) = if cell.key == 0 {
                    (
                        format!(" {key}"),
                        if base == Role::Body { Role::Dim } else { base },
                    )
                } else {
                    (value.clone(), base)
                };
                fit_with(
                    cx,
                    r,
                    &text,
                    &FitOpts {
                        justify: Justify::Start,
                        role,
                        pad: role,
                    },
                );
            },
        );
        self.meta_id = Some(resp.id);
        fit_with(
            cx,
            flash,
            &format!(" {}", self.flash),
            &FitOpts {
                justify: Justify::Start,
                role: Role::Ok,
                pad: Role::Ok,
            },
        );
    }

    /// The clipboard panel.
    fn draw_clipboard(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) {
        let title = if self.clipboard.cut {
            " Clipboard (cut) "
        } else {
            " Clipboard "
        };
        let frame = panel_with(
            cx,
            area,
            title,
            "",
            &PanelOpts {
                padded: false,
                ..Default::default()
            },
        );
        if frame.interior.is_empty() {
            return;
        }
        let [list, bar] = Row::new().split(frame.interior, [Weight(1), Fixed(1)]);
        for i in 0..usize::from(list.h) {
            let row = Rect::new(list.x, list.y + i32::try_from(i).unwrap_or(0), list.w, 1);
            let line = match self.clipboard.items.get(i) {
                None if i == 0 && self.clipboard.items.is_empty() => {
                    " no content in clipboard".to_owned()
                }
                None => String::new(),
                Some(&at) => format!(" {}", self.tree.items[at].name),
            };
            let role = if self.clipboard.items.is_empty() {
                Role::Dim
            } else {
                Role::Body
            };
            fit_with(
                cx,
                row,
                &line,
                &FitOpts {
                    justify: Justify::Start,
                    role,
                    pad: role,
                },
            );
        }
        // **A bar is drawn only where there is something to scroll**, and the column is filled
        // either way: a track with a full-height thumb over an empty list says the opposite of
        // what it means, and an unwritten column keeps the previous frame's cells.
        let extent = u32::try_from(self.clipboard.items.len()).unwrap_or(0);
        if extent > u32::from(list.h) {
            scrollbar(
                cx,
                bar,
                Span {
                    viewport: u32::from(list.h),
                    extent,
                    offset: 0,
                },
            );
        } else {
            cx.fill(bar, " ", cx.theme().paint(Role::Body));
        }
    }

    /// **Follow the ring.** `Tab` is the runtime's key and no key of this application's — the ring
    /// resolves the walk in `settle`, after the previous frame drew — so `Tab` is read here, by
    /// asking which of this screen's panes the focus ended on. That makes `Tab` cycle sidebar →
    /// file panels → processes → metadata, which is a superset of superfile's own
    /// `next_file_panel`; `L` and `H` are the two that walk the file panels alone, and
    /// `hotkeys.toml` binds those beside `tab` for exactly this reason.
    ///
    /// **It is read at the top of the draw**, because everything below is what the reader sees.
    fn adopt_focus(&mut self, cx: &Ctx<'_, '_>) {
        // **A standing request of this application's wins over the ring**, and it has to: the seat
        // is asked for *between* frames and granted *during* the next one, so on that frame the
        // focus is still where the ring left it — and adopting it here would read the application's
        // own `n` back as *the user tabbed to the old panel* and undo it.
        if self.seat {
            return;
        }
        let Some(id) = cx.focused() else {
            return;
        };
        if self.side_id == Some(id) {
            self.focus = Focus::Sidebar;
        } else if self.proc_id == Some(id) {
            self.focus = Focus::Process;
        } else if self.meta_id == Some(id) {
            self.focus = Focus::Metadata;
        } else if let Some(at) = self.panels.iter().position(|p| p.id == Some(id)) {
            self.focus = Focus::Panel;
            self.panel = at;
        }
    }

    /// Seat the keyboard where `s`, `p`, `m`, `L` and `H` last put it.
    ///
    /// **`focused().is_none()` and never `!is_focused(id)`** — except when
    /// this application's own focus word has moved, which is what the second arm is: a `Tab`
    /// between panels is superfile's key and not the runtime's walk here.
    fn seat_focus(&mut self, cx: &mut Ctx<'_, '_>) {
        if self.searching || !matches!(self.modal, Modal::None) || self.renaming.is_some() {
            return;
        }
        // **Every pane `s`, `p` and `m` name has to be a place the keyboard can go.** Answering
        // `None` for two of them left the focus where it was — on a file panel — so `p` and `m`
        // moved a word in this application and nothing on the screen, and the panel went on eating
        // the arrows. A pane with nothing in it (`p` with no processes) is the one honest `None`.
        let want = match self.focus {
            Focus::Sidebar => self.side_id,
            Focus::Panel => self.panels[self.panel].id,
            Focus::Process => self.proc_id,
            Focus::Metadata => self.meta_id,
        };
        // **Only when this application asked, or when nobody holds it**.
        // Seating unconditionally would drag the keyboard back on the frame after every `Tab`,
        // which is the anti-pattern `counter` documents — and here it would silently undo the ring.
        let Some(id) = want else {
            // **The request stands until the pane has an id.** A panel opened by `n` has not drawn
            // yet on the frame that created it, so its id is `None` and a seat cleared here would
            // leave the keyboard on the panel the user just left — which is what `n` did until
            // this line: it opened a panel and typed into its neighbour.
            return;
        };
        if (self.seat || cx.focused().is_none()) && !cx.is_focused(id) {
            cx.focus(id);
        }
        self.seat = false;
    }

    /// The help, the create dialog, the delete confirmation and the sort menu.
    fn draw_modal<'f>(&'f mut self, cx: &mut Ctx<'f, '_>) {
        let screen = cx.area();
        let theme = *cx.theme();
        let host = cx.id();
        let App { modal, pending, .. } = self;
        let (size, title) = match &*modal {
            Modal::None => return,
            Modal::Help => ((60.min(screen.w), 22.min(screen.h)), " Help "),
            Modal::Create(_) => ((56.min(screen.w), 6.min(screen.h)), " Create "),
            Modal::Delete(_) => ((52.min(screen.w), 7.min(screen.h)), " Delete "),
            Modal::SortMenu(_) => ((30.min(screen.w), 6.min(screen.h)), " Sort by "),
        };
        let opts = OverlayOpts {
            z: Z::MODAL,
            ..OverlayOpts::modal(size.0, size.1, &theme)
        };
        let anchor = Stack::center(screen, size.0, size.1);
        cx.overlay(host, anchor, opts, move |cx| {
            let area = cx.area();
            let shell = ShellOpts {
                kind: ShellKind::Dialog,
                ..ShellOpts::default()
            };
            let _ = overlay_with(cx, area, u32::from(area.h), &shell, &mut |cx, r| {
                let frame = panel_with(
                    cx,
                    r,
                    title,
                    "",
                    &PanelOpts {
                        border: Role::Focus,
                        title_role: match modal {
                            Modal::Delete(_) => Role::Danger,
                            _ => Role::Title,
                        },
                        padded: false,
                        ..Default::default()
                    },
                );
                let r = frame.interior;
                match modal {
                    Modal::None => {}
                    Modal::Help => draw_help(cx, r, pending),
                    Modal::Create(buf) => draw_create(cx, r, buf),
                    Modal::Delete(n) => draw_delete(cx, r, *n, pending),
                    Modal::SortMenu(st) => draw_sort(cx, r, st, pending),
                }
            });
        });
    }
}

/// Write one cell, keeping a one-cell gutter on the right of a left-justified one.
///
/// **A cell drawer owes its whole rectangle**, so the gutter is *written* rather than
/// left out: a left-justified name that fills its column otherwise runs into the column beside it,
/// and the ellipsis lands against a digit. A right-justified cell carries its gutter in the text,
/// because its padding is on the left.
fn cell_into(cx: &mut Ctx<'_, '_>, r: Rect, text: &str, justify: Justify, role: Role) {
    let opts = FitOpts {
        justify,
        role,
        pad: role,
    };
    match justify {
        Justify::Start if r.w > 1 => {
            let (body, gap) = rect::split_at_h(r, r.w - 1);
            fit_with(cx, body, text, &opts);
            fit_with(cx, gap, "", &opts);
        }
        _ => {
            fit_with(cx, r, text, &opts);
        }
    }
}

/// The `?` modal.
fn draw_help(cx: &mut Ctx<'_, '_>, r: Rect, pending: &mut Option<Act>) {
    for (i, (key, what)) in HELP.iter().enumerate().take(usize::from(r.h)) {
        let row = Rect::new(r.x, r.y + i32::try_from(i).unwrap_or(0), r.w, 1);
        let [k, w] = Row::new().split(row, [Fixed(20), Weight(1)]);
        fit_with(
            cx,
            k,
            &format!(" {key}"),
            &FitOpts {
                justify: Justify::Start,
                role: Role::Warn,
                pad: Role::Warn,
            },
        );
        fit_with(
            cx,
            w,
            what,
            &FitOpts {
                justify: Justify::Start,
                role: Role::Body,
                pad: Role::Body,
            },
        );
    }
    let id = cx.id();
    let _ = cx.interact(id, r, Interest::FOCUS);
    if cx.focused().is_none() {
        cx.focus(id);
    }
    while let Some(k) = cx.next_key(id) {
        // **`!k.mods.ctrl()` is load-bearing.** Written as `Code::Char('q')` alone, the arm below
        // also catches `Ctrl+Q` — so the application's quit chord closed the help instead of
        // quitting, and the help then swallowed it. It is the shape a sweep found one
        // crate down: *`Esc` and `Space` are keys and not chords*, and a match on the code alone
        // eats every accelerator built on it.
        //
        // **The `?` alternate is read from one home** rather than re-spelled here: see
        // [`HELP_KEYS`].
        let plain_close = matches!(k.code, Code::Escape | Code::Enter | Code::Char('q'));
        if (plain_close && !k.mods.ctrl()) || is_help_key(&k) {
            *pending = Some(Act::Close);
        } else {
            cx.decline(k);
        }
    }
}

/// `Ctrl+N`: one field, and `Enter`/`Esc` read from the unhandled window.
fn draw_create(cx: &mut Ctx<'_, '_>, r: Rect, buf: &mut Text) {
    let [prompt, entry, hint] = Col::new().split(r, [Fixed(1), Fixed(1), Weight(1)]);
    fit_with(
        cx,
        prompt,
        " Name it. A trailing `/` makes a directory.",
        &FitOpts {
            justify: Justify::Start,
            role: Role::Body,
            pad: Role::Body,
        },
    );
    let resp = field(cx, rect::shrink(entry, 1, 0, 1, 0), buf);
    // The dialog has one widget, so the seat is unconditional — `Enter` and `Esc` are the field's
    // to decline and reach `App::take_unhandled`.
    if !cx.is_focused(resp.id) {
        cx.focus(resp.id);
    }
    fit_with(
        cx,
        hint,
        " (enter) Create        (esc) Cancel",
        &FitOpts {
            justify: Justify::Start,
            role: Role::Dim,
            pad: Role::Dim,
        },
    );
}

/// `Ctrl+D`.
fn draw_delete(cx: &mut Ctx<'_, '_>, r: Rect, n: usize, pending: &mut Option<Act>) {
    let [what, _gap, buttons] = Col::new().split(r, [Fixed(1), Fixed(1), Weight(1)]);
    fit_with(
        cx,
        what,
        &format!(
            "  Move {n} item{} to the trash?",
            if n == 1 { "" } else { "s" }
        ),
        &FitOpts {
            justify: Justify::Start,
            role: Role::Body,
            pad: Role::Body,
        },
    );
    let [yes, no] = Row::new()
        .spacing(2)
        .margin(1)
        .split(buttons, [Weight(1), Weight(1)]);
    let yes = button_with(cx, yes, "Delete", &ButtonOpts::default());
    let no = button_with(cx, no, "Cancel", &ButtonOpts::default());
    if yes.clicked {
        *pending = Some(Act::Delete);
    }
    if no.clicked {
        *pending = Some(Act::Close);
    }
    // **`Cancel` is where the keyboard starts, and saying so takes naming both.** A `Kind::Dialog`
    // opens a keyboard trap and the trap seats a stop of its own on the first frame, so
    // `focused().is_none()` is already false — and `Enter` would land on whichever button the walk
    // reached first, which for a delete is the wrong default. k9s makes the same decision by
    // requiring `Tab` before `Enter`.
    if ![yes.id, no.id].iter().any(|id| cx.is_focused(*id)) {
        cx.focus(no.id);
    }
    for (id, act) in [(yes.id, Act::Delete), (no.id, Act::Close)] {
        while let Some(k) = cx.next_key(id) {
            match k.code {
                Code::Enter | Code::Char(' ') => *pending = Some(act),
                Code::Escape => *pending = Some(Act::Close),
                _ => cx.decline(k),
            }
        }
    }
}

/// `o` — the sort menu, which is a `collection` at `Mode::Options`: exactly one, never zero.
fn draw_sort(cx: &mut Ctx<'_, '_>, r: Rect, st: &mut CollState, pending: &mut Option<Act>) {
    let opts = CollOpts {
        mode: Mode::Options,
        ..CollOpts::default()
    };
    let resp = collection(
        cx,
        r,
        st,
        &opts,
        Rows::of(Sort::ALL.len()),
        &mut |_buf, _range| None,
        &mut |cx, row, i, face| {
            let role = face_role(face);
            fit_with(
                cx,
                row,
                &format!(" {} ", Sort::ALL[i].word()),
                &FitOpts {
                    justify: Justify::Start,
                    role,
                    pad: role,
                },
            );
        },
    );
    if cx.focused().is_none() {
        cx.focus(resp.id);
    }
    if resp.clicked {
        *pending = Some(Act::Sort(st.sel.lead));
    }
    // **The keyboard is not read here, and it cannot be.** `Ctx::decline` hands a key back *and
    // ends the level's turn at the queue*, so a second `next_key` on the collection's id answers
    // `None` for the rest of the frame. `Enter` and `Esc` arrive in the unhandled window instead —
    // see `App::take_unhandled`. The one route that would work inside the frame is
    // `crate::collect::Refusal`, and it is `pub(crate)`.
}

// ── what happens between two frames ──────────────────────────────────────────────────────────────

impl App {
    /// **The inbox, read after the draw.**
    fn answer(&mut self) -> bool {
        let Some(act) = self.pending.take() else {
            return false;
        };
        // Every one of these ends with a widget that had the keyboard no longer drawing, and the
        // vanish rule then leaves the focus on whatever survived nearest in the ring.
        self.seat = true;
        match act {
            Act::Close => self.modal = Modal::None,
            Act::Sidebar => self.go_to_sidebar(),
            Act::Create => self.create(),
            Act::Delete => self.delete(),
            Act::Sort(i) => {
                self.modal = Modal::None;
                let sort = Sort::ALL[i.min(Sort::ALL.len() - 1)];
                let dots = self.dots;
                let tree = &self.tree;
                let panel = &mut self.panels[self.panel];
                panel.sort = sort;
                panel.relist(tree, dots);
                self.flash = format!("sorted by {}", sort.word());
            }
        }
        true
    }

    /// Follow the sidebar's cursor.
    fn go_to_sidebar(&mut self) {
        let Side::Place(label, path) = &SIDEBAR[self.side.sel.lead.min(SIDEBAR.len() - 1)] else {
            return;
        };
        let at = self.tree.find(path);
        let (dots, which) = (self.dots, self.panel);
        self.panels[which].cwd = at;
        self.panels[which].st.coll.offset = 0;
        self.panels[which].st.coll.sel.lead = 0;
        let tree = &self.tree;
        self.panels[which].relist(tree, dots);
        self.flash = format!("{label} — {}", self.tree.path(at));
    }

    /// `l` / `Enter`: open the entry under the cursor.
    fn open(&mut self) {
        let (dots, which) = (self.dots, self.panel);
        let Some(at) = self.panels[which].cursor() else {
            return;
        };
        if !self.tree.items[at].dir {
            self.flash = format!("{} is not a directory", self.tree.items[at].name);
            return;
        }
        self.panels[which].cwd = at;
        self.panels[which].search.clear();
        self.panels[which].st.coll.offset = 0;
        self.panels[which].st.coll.sel.lead = 0;
        let tree = &self.tree;
        self.panels[which].relist(tree, dots);
    }

    /// `h` / `Backspace`: the parent.
    fn up(&mut self) {
        let (dots, which) = (self.dots, self.panel);
        let leaving = self.panels[which].cwd;
        if leaving == 0 {
            return;
        }
        self.panels[which].cwd = self.tree.items[leaving].parent;
        self.panels[which].search.clear();
        self.panels[which].st.coll.offset = 0;
        let tree = &self.tree;
        self.panels[which].relist(tree, dots);
        if let Some(at) = self.panels[which].rows.iter().position(|x| *x == leaving) {
            self.panels[which].st.coll.sel.lead = at;
        }
    }

    /// `Ctrl+V`: start a copy, which is what the process bar is for.
    ///
    /// **The process is pushed with a stopped spinner and `App::tick` starts it**, which is the
    /// only arrangement left: nothing out here has a `Ctx`, so nothing out here has a
    /// clock to seed an anchor from.
    fn paste(&mut self) {
        if self.clipboard.items.is_empty() {
            self.flash = "clipboard is empty".to_owned();
            return;
        }
        let dest = self.panels[self.panel].cwd;
        let items = std::mem::take(&mut self.clipboard.items);
        let cut = self.clipboard.cut;
        let total = u32::try_from(items.len()).unwrap_or(1).max(1) * 4;
        for at in &items {
            if cut {
                self.tree.items[*at].parent = dest;
            } else {
                let (name, dir) = (self.tree.items[*at].name.clone(), self.tree.items[*at].dir);
                self.tree.add(dest, &name, dir);
            }
        }
        self.processes.push(Process {
            what: format!(
                "{} {} item{}",
                if cut { "moving" } else { "copying" },
                items.len(),
                if items.len() == 1 { "" } else { "s" },
            ),
            done: 0,
            total,
            spin: SpinState::new(),
        });
        self.clipboard.cut = false;
        self.relist_all();
        self.flash = format!("pasted {} item(s)", items.len());
    }

    /// `Ctrl+N`, once the name has been typed.
    fn create(&mut self) {
        let Modal::Create(buf) = &self.modal else {
            return;
        };
        let text = buf.text().trim().to_owned();
        self.modal = Modal::None;
        if text.is_empty() {
            return;
        }
        let dir = text.ends_with('/');
        let name = text.trim_end_matches('/').to_owned();
        let at = self.panels[self.panel].cwd;
        self.tree.add(at, &name, dir);
        self.relist_all();
        self.flash = format!("created {name}");
    }

    /// `Ctrl+D`, confirmed.
    ///
    /// **An entry is marked rather than removed**: every index in this tree is a position in one
    /// vector, and shifting them would invalidate every panel at once. See [`Item::dead`] for why
    /// the flag is a flag and not a clever reparent.
    fn delete(&mut self) {
        let targets = self.panels[self.panel].targets();
        self.modal = Modal::None;
        let mut n = 0;
        for at in targets {
            if at != 0 {
                self.tree.items[at].dead = true;
                n += 1;
            }
        }
        self.clipboard.items.retain(|at| !self.tree.items[*at].dead);
        self.relist_all();
        self.flash = format!("{n} item(s) moved to the trash");
    }

    /// Finish an inline rename.
    fn commit_rename(&mut self) {
        let Some((row, buf)) = self.renaming.take() else {
            return;
        };
        let text = buf.text().trim().to_owned();
        let which = self.panel;
        let Some(&at) = self.panels[which].rows.get(row) else {
            return;
        };
        if text.is_empty() {
            self.flash = "an empty name does nothing".to_owned();
            return;
        }
        self.flash = format!("{} → {text}", self.tree.items[at].name);
        self.tree.items[at].name = text;
        self.relist_all();
    }

    /// **What nothing wanted, read from the frame that has just drawn.**
    fn take_unhandled(&mut self, keys: &[Pressed]) {
        // **The quit chord is answered before anything else claims the keyboard.** Every branch
        // below returns early for whatever is open, and an application whose way out depends on
        // which dialog is up is an application people cannot leave.
        for k in keys {
            if self.keys.match_first(k) == Some(QUIT) && k.mods.ctrl() {
                self.exit = true;
                return;
            }
        }
        // A field holds the focus while any of these is open, so the two ways out arrive here —
        // `Ctx::next_key` answers only the focused id, and a sink beside a focused field is deaf.
        if self.searching {
            for k in keys {
                match k.code {
                    Code::Enter | Code::Escape => {
                        let text = if k.code == Code::Enter {
                            self.search.text().to_owned()
                        } else {
                            String::new()
                        };
                        self.searching = false;
                        self.seat = true;
                        self.search = Text::input();
                        let (dots, which) = (self.dots, self.panel);
                        self.panels[which].search = text;
                        let tree = &self.tree;
                        self.panels[which].relist(tree, dots);
                    }
                    _ => {}
                }
            }
            return;
        }
        if self.renaming.is_some() {
            for k in keys {
                match k.code {
                    Code::Enter => {
                        self.commit_rename();
                        self.seat = true;
                    }
                    Code::Escape => {
                        self.renaming = None;
                        self.seat = true;
                    }
                    _ => {}
                }
            }
            return;
        }
        if let Modal::Create(_) = self.modal {
            for k in keys {
                match k.code {
                    Code::Enter => self.pending = Some(Act::Create),
                    Code::Escape => self.pending = Some(Act::Close),
                    _ => {}
                }
            }
            return;
        }
        if let Modal::SortMenu(st) = &self.modal {
            // See `draw_sort`: the collection declines these and ends the turn.
            let lead = st.sel.lead;
            for k in keys {
                match k.code {
                    Code::Enter => self.pending = Some(Act::Sort(lead)),
                    Code::Escape | Code::Char('o') => self.pending = Some(Act::Close),
                    _ => {}
                }
            }
            return;
        }
        if !matches!(self.modal, Modal::None) {
            return;
        }
        for k in keys {
            if let Some(action) = self.keys.match_first(k) {
                self.act(action);
            }
        }
    }

    /// One action, whichever chord produced it.
    fn act(&mut self, action: ActionId) {
        let (dots, which) = (self.dots, self.panel);
        match action {
            QUIT => self.exit = true,
            COPY | CUT => {
                self.clipboard.items = self.panels[which].targets();
                self.clipboard.cut = action == CUT;
                self.flash = format!(
                    "{} {} item(s)",
                    if action == CUT { "cut" } else { "copied" },
                    self.clipboard.items.len()
                );
            }
            PASTE => self.paste(),
            DELETE => {
                let n = self.panels[which].targets().len();
                if n == 0 {
                    self.flash = "nothing to delete".to_owned();
                } else {
                    self.modal = Modal::Delete(n);
                }
            }
            CREATE => self.modal = Modal::Create(Text::input()),
            RENAME => {
                let row = self.panels[which].st.coll.sel.lead;
                if let Some(at) = self.panels[which].cursor() {
                    let mut buf = Text::input();
                    buf.edit(40, &self.tree.items[at].name);
                    self.renaming = Some((row, buf));
                }
            }
            // **`Ctrl+Space` toggles one row.** superfile has no such binding — it selects with
            // `J`/`K` and `A` — but a select mode with no way to pick one row out of a run is a
            // select mode nobody can use, and `space` itself is eaten by the table.
            TOGGLE if self.panels[which].mode == PanelMode::Select => {
                let panel = &mut self.panels[which];
                let (lead, len) = (panel.st.coll.sel.lead, panel.rows.len());
                if lead < len {
                    apply(Mode::Multi, &mut panel.marks, len, Gesture::Toggle(lead));
                }
            }
            DOWN => self.step(1),
            UP => self.step(-1),
            // **`J`/`K` and not `Shift+↓`/`Shift+↑`.** `hotkeys.toml` binds both;
            // `crate::collect::from_key` answers a shifted arrow with `Gesture::Extend` in **every**
            // mode and `apply` ignores it at `Mode::Cursor`, so the arrow moves the cursor, is
            // consumed, and never reaches this window. The letters are declined and do arrive.
            EXTEND_DOWN => self.extend(1),
            EXTEND_UP => self.extend(-1),
            OPEN => match self.focus {
                Focus::Sidebar => self.go_to_sidebar(),
                _ => self.open(),
            },
            PARENT => self.up(),
            // **`L` and `H`, and `Tab` is the ring's.** `hotkeys.toml` binds
            // `next_file_panel = ['tab', 'L']`, and the `tab` half is one this surface cannot have:
            // the focus ring consumes it in `settle` and never offers it to an application. What it
            // does instead is walk every pane on the screen, which `App::adopt_focus` reads.
            NEXT_PANEL => {
                self.focus = Focus::Panel;
                self.panel = (self.panel + 1) % self.panels.len();
                self.seat = true;
            }
            PREV_PANEL => {
                self.focus = Focus::Panel;
                self.panel = (self.panel + self.panels.len() - 1) % self.panels.len();
                self.seat = true;
            }
            NEW_PANEL if self.panels.len() < 3 => {
                let cwd = self.panels[which].cwd;
                let mut panel = FilePanel::new(cwd);
                panel.relist(&self.tree, dots);
                self.panels.push(panel);
                self.panel = self.panels.len() - 1;
                self.focus = Focus::Panel;
                self.seat = true;
                self.flash = format!("{} file panels", self.panels.len());
            }
            CLOSE_PANEL if self.panels.len() > 1 => {
                self.panels.remove(which);
                self.panel = which.min(self.panels.len() - 1);
                self.seat = true;
                self.flash = format!("{} file panels", self.panels.len());
            }
            FOCUS_SIDEBAR => (self.focus, self.seat) = (Focus::Sidebar, true),
            PROCESS => (self.focus, self.seat) = (Focus::Process, true),
            METADATA => (self.focus, self.seat) = (Focus::Metadata, true),
            MODE => {
                let panel = &mut self.panels[which];
                panel.mode = match panel.mode {
                    PanelMode::Browser => PanelMode::Select,
                    PanelMode::Select => {
                        panel.marks.clear();
                        PanelMode::Browser
                    }
                };
                self.flash = format!("{:?} mode", self.panels[which].mode);
            }
            SELECT_ALL => {
                let panel = &mut self.panels[which];
                if panel.mode == PanelMode::Select {
                    let len = panel.rows.len();
                    apply(Mode::Multi, &mut panel.marks, len, Gesture::All);
                    self.flash = format!("{len} selected");
                }
            }
            REVERSE => {
                self.panels[which].desc = !self.panels[which].desc;
                let tree = &self.tree;
                self.panels[which].relist(tree, dots);
                self.flash = "reversed".to_owned();
            }
            SORT => self.modal = Modal::SortMenu(CollState::new()),
            DOTS => {
                self.dots = !self.dots;
                self.relist_all();
                self.flash = format!("dot files {}", if self.dots { "shown" } else { "hidden" });
            }
            FOOTER => self.footer = !self.footer,
            SEARCH => {
                self.search = Text::input();
                self.search.edit(40, &self.panels[which].search);
                self.searching = true;
            }
            SHOW_HELP => self.modal = Modal::Help,
            BACK => (self.focus, self.seat) = (Focus::Panel, true),
            _ => {}
        }
    }

    /// Move the cursor of whatever holds the focus.
    fn step(&mut self, by: i32) {
        let (lead, len) = match self.focus {
            Focus::Sidebar => (&mut self.side.sel.lead, SIDEBAR.len()),
            Focus::Process => (&mut self.proc.sel.lead, self.processes.len() * 2),
            Focus::Metadata => (&mut self.meta.coll.sel.lead, 6),
            Focus::Panel => {
                let panel = &mut self.panels[self.panel];
                let len = panel.rows.len();
                (&mut panel.st.coll.sel.lead, len)
            }
        };
        if len == 0 {
            return;
        }
        let to = i64::try_from(*lead).unwrap_or(0) + i64::from(by);
        *lead = usize::try_from(to.clamp(0, i64::try_from(len - 1).unwrap_or(0))).unwrap_or(0);
    }

    /// **A sidebar divider is not a place, and the cursor may have been moved by either party.**
    ///
    /// `j` is declined by the collection and steps the cursor here; `↓` is the collection's own key
    /// and steps it there. Nudging past a heading in `step` alone would fix one of those and leave
    /// the other resting on `Pinned` — so it is done once, at the top of the frame, in the
    /// direction the cursor was travelling.
    fn skip_divider(&mut self) {
        let mut dir: i64 = if self.side.sel.lead >= self.side_was {
            1
        } else {
            -1
        };
        let last = i64::try_from(SIDEBAR.len() - 1).unwrap_or(0);
        for _ in 0..SIDEBAR.len() {
            if !matches!(SIDEBAR.get(self.side.sel.lead), Some(Side::Divider(_))) {
                break;
            }
            let to = i64::try_from(self.side.sel.lead).unwrap_or(0) + dir;
            if to < 0 || to > last {
                // The list ended under a heading, so turn round rather than rest on one.
                dir = -dir;
                continue;
            }
            self.side.sel.lead = usize::try_from(to).unwrap_or(0);
        }
        self.side_was = self.side.sel.lead;
    }

    /// `Shift+↓` / `Shift+↑`: move and extend, which only select mode has.
    fn extend(&mut self, by: i32) {
        self.step(by);
        let which = self.panel;
        let panel = &mut self.panels[which];
        if self.focus != Focus::Panel || panel.mode != PanelMode::Select {
            return;
        }
        let (lead, len) = (panel.st.coll.sel.lead, panel.rows.len());
        if panel.marks.anchor.is_none() {
            panel.marks.anchor = Some(lead);
        }
        apply(Mode::Multi, &mut panel.marks, len, Gesture::Extend(lead));
    }
}

fn main() {
    let mut app = App::new();

    // **The picture arm**: one screen, drawn headlessly and written as SVG. A
    // program nobody outside this machine can see is a program nobody believes in.
    if vitui_apps::pictures::wanted() {
        let mut driver = vitui_apps::pictures::driver(120, 36, *Themes::standard().theme());
        for _ in 0..vitui_apps::pictures::FRAMES {
            driver.frame(|cx| app.ui(cx));
        }
        vitui_apps::pictures::write("spf", &driver);
        return;
    }

    let mut driver = match Driver::attach(Default::default(), *Themes::standard().theme()) {
        Ok(driver) => driver,
        Err(why) => {
            eprintln!("vitui could not attach: {why}");
            return;
        }
    };

    driver.frame(|cx| app.ui(cx));
    let _ = app.answer();

    loop {
        driver.frame(|cx| app.ui(cx));
        // **After the draw**: a modal's decision was written inside the frame and cannot be acted
        // on from there.
        let acted = app.answer();
        // **The window onto the frame that has just drawn**, never the one before it: `unhandled`
        // is valid until the next frame begins, and reading it first acts one wake late.
        let unhandled: Vec<Pressed> = driver.unhandled().to_vec();
        app.take_unhandled(&unhandled);
        if app.exit {
            break;
        }
        // **A frame is owed whenever the inbox moved**, and not only when a key arrived: the
        // decision is acted on *after* the draw, so the screen showing a dialog that has just been
        // answered is one frame stale — and with nothing else pending, `wait` would park on it.
        //
        // **A focus move owes a frame too, and this loop deliberately no longer counts it.** The
        // ring resolves its walk in `settle`, *after* the draw, so the frame that consumed a `Tab`
        // painted the old focus ring with an empty `unhandled` — and this file used to compare
        // `Driver::inspect().focused()` across frames because nothing else asked. The award's own wake
        // put the ask where the decision is made: `Frame::resolve_award` calls
        // `wants_another_frame`, which is `deadline(now)`, so `wait` returns at once rather than
        // parking on a screen a keystroke behind. Comparing here as well would be a second spelling
        // of one rule, in the place least able to keep it true.
        if acted || !unhandled.is_empty() {
            continue;
        }
        match driver.wait() {
            Wake::Quit => break,
            Wake::Input | Wake::Posted | Wake::Deadline => {}
        }
    }
}
