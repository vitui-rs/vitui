//! **`commander` — GNU Midnight Commander, drawn on the component surface.**
//!
//! ```text
//! cargo run -p vitui-apps --example commander
//! ```
//!
//! <https://github.com/MidnightCommander/mc>
//!
//! # A port, and the data is the only invented part
//!
//! The shape is `mc`'s and is not ours to argue with: a menu bar, two directory panels side by
//! side, a mini-status under each, a hint line, a shell prompt and the ten-button function bar. The
//! labels on that bar are `src/filemanager/filemanager.c`'s own —
//! `Help Menu View Edit Copy RenMov Mkdir Delete PullDn Quit` — rather than ten words that look
//! about right.
//!
//! **The tree is synthetic and nothing here opens a file.** That is deliberate and it is not a
//! shortcut: an application in this directory exists to exercise the component surface, and a real
//! `readdir` would put the interesting failures in `std::fs` instead of in the library under test.
//! Sizes and timestamps are a hash of the entry's own name, so the screen is the same every run.
//!
//! # What the port cannot say, recorded rather than worked around
//!
//! 1. **A status bar's segments are drawn in one `Role`, and a segment is one run.** `mc` paints the
//!    digit and the label of `1Help` differently — two paints **inside** one segment, which is not
//!    what a per-segment role would buy: split into `1` and `Help` the bar would draw a separator
//!    between them and share the width out per part. So the function bar here is flat where `mc`'s
//!    is two-toned, and the field this file used to ask for would not have fixed it. Drawing it by
//!    hand would have hidden the gap and lost the component.
//! 2. **A marked file loses its type colour.** `mc` paints marks yellow *over* the directory white
//!    and the executable green; the face ladder is *disabled > selected > cursor > hovered > base*,
//!    resolved before a cell is written, so a mark replaces the type rather than riding
//!    on it. That is the library's decision working, not a defect — it is recorded because a reader
//!    comparing the two screens will notice.
//!
//! # Two that were on this list and are not: the path is centred and the volume is in the frame
//!
//! `mc` centres a panel's path over its top border and prints the free-space readout under its
//! bottom one, and for most of this file's life the first was pinned to the left and the second sat
//! on the mini-status row. `PanelOpts::justify` and `panel_with`'s second string are what closed
//! it — this application, `counter` and `spf` all wanted the same two, and one alignment governs
//! both borders because none of the three wanted them to disagree.
//!
//! # The prompt is fed from the unhandled window, and that is a finding rather than a trick
//!
//! In `mc` a letter typed anywhere lands on the shell prompt. Here the panel holds the focus, and
//! **a focused collection consumes every text-bearing key into its type-ahead buffer** —
//! the trap `ledger` and `explorer` bind `Ctrl+Q` around. What makes the prompt reachable is that
//! type-ahead is *the caller's search behind a component's budget*: this application's search
//! answers `None` for every buffer, so the component declines the key and it arrives in
//! `Driver::unhandled` beside the arrows and the function keys.
//!
//! The cost is stated rather than hidden: the component still pushes the character into its buffer
//! and still asks for the expiry deadline, so a burst of typing costs one extra wake per keystroke.
//! Nothing is ever drawn from that buffer, because nothing ever matches.
//!
//! **`Space` used to be the one printable key that did not get through, and this is where it was
//! found.** `crate::collect::from_key` answers a bare space with `Gesture::Toggle` in *every*
//! [`Mode`], and `apply` ignores it at [`Mode::Cursor`] — so it was consumed to do nothing and the
//! prompt never saw it, and this file recorded the words running together rather than working
//! around it. It was `owns_escape`'s defect one key over: `Escape` had been taught
//! `Escape` to decline when `apply` would clear nothing and left `Space` and `Ctrl+A` as they
//! were. `crate::collect::owns` is that narrowing said of the whole vocabulary, so the panel
//! declines both here and the prompt takes its spaces.
//!
//! # An overlay body may not capture a local, so what a dialog decides goes in a field
//!
//! `Ctx::overlay`'s body is `FnMut(&mut Ctx<'f, '_>) + 'f`, and `'f` is the frame — so a body can
//! hold a `&'f mut` of an application field and cannot hold a reference to a local of the draw.
//! Every dialog here therefore writes its answer into `App::pending` and the loop reads it after
//! the frame, which is `console`'s inbox one application over. The scrollable viewer keeps its
//! text **inside** the `Modal` for the same reason: a `Vec<String>` built in the draw could not be
//! reached from the body at all.
//!
//! # Keys
//!
//! | key | what |
//! |---|---|
//! | `Tab` | the other panel — the runtime's focus walk, and no key of this application's |
//! | `↑` `↓` `PgUp` `PgDn` `Home` `End` | the cursor, which is `Mode::Cursor`: it never selects |
//! | `Enter` | descend into a directory, or `..`; on a non-empty prompt, run it |
//! | `Insert` | mark the row and step down · `Ctrl+T` inverts · `Esc` drops the marks |
//! | `F1` … `F10` | help, menu, view, edit, copy, move, mkdir, delete, menu, quit |
//! | `←` `→` | inside a pull-down: its neighbour, as `mc` binds it. They were `Alt+←`/`Alt+→` here while a `collection` read the bare arrows as `↑`/`↓` and consumed them |
//! | `Ctrl+R` | re-read: fresh sizes and timestamps for the active directory |
//! | `Ctrl+U` | swap the panels, as `mc` does |
//! | `Ctrl+Q` | quit, because `q` is a character and characters go to the prompt |
//! | printable · `Backspace` | the shell prompt, **`Space` included** — see the note above |
//! | `Esc` | close whatever is open, or clear the prompt |

use std::fmt::Write as _;

use vitui_components::collect::{
    Cell, CollOpts, CollState, Column, Gesture, Mode, Selection, TableOpts, TableState, apply,
    collection, table,
};
use vitui_components::edit::{Caret, Text};
use vitui_components::frame::{Face, face_role};
use vitui_components::input::{ButtonOpts, button_with, field};
use vitui_components::keys::text as typed_text;
use vitui_components::order::Rows;
use vitui_components::overlay::{Kind as ShellKind, ShellOpts, overlay_with};
use vitui_components::scroll::{Span, scrollbar};
use vitui_components::structure::{
    PanelOpts, RuleOpts, StatusOpts, panel_with, rule_with, status_bar_with,
};
use vitui_components::text::{FitOpts, Justify, TextOpts, fit_with, text_with};
use vitui_runtime::ctx::Driver;
use vitui_runtime::focus::ScopeKind;
use vitui_runtime::keys::{Code, Pressed};
use vitui_runtime::layout::Constraint::{Fixed, Weight};
use vitui_runtime::layout::{Col, Row, Stack, rect};
use vitui_runtime::work::Wake;
use vitui_runtime::{Ctx, Glyph, Id, Interest, OverlayOpts, Rect, Revision, Role, Themes, Z};

// ── the tree, which is the only invented thing here ──────────────────────────────────────────────

/// What an entry is. `mc` paints four kinds differently and so does this.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    /// A directory. Sorted before every file, whatever the sort key — `mc`'s own rule.
    Dir,
    /// An ordinary file.
    File,
    /// Executable.
    Exec,
    /// A symbolic link.
    Link,
}

impl Kind {
    /// The role an entry of this kind is drawn in **while nothing else claims the row**.
    ///
    /// A component names a `Role` and never a colour, so `mc`'s cyan directories are
    /// `Role::Title` here and its green executables are `Role::Ok`. Which colour a scheme gives
    /// those is the scheme's business.
    const fn role(self) -> Role {
        match self {
            Kind::Dir => Role::Title,
            Kind::File => Role::Body,
            Kind::Exec => Role::Ok,
            Kind::Link => Role::Warn,
        }
    }

    /// The permission string `mc` prints in the mini-status.
    const fn perms(self) -> &'static str {
        match self {
            Kind::Dir => "drwxr-xr-x",
            Kind::File => "-rw-r--r--",
            Kind::Exec => "-rwxr-xr-x",
            Kind::Link => "lrwxrwxrwx",
        }
    }
}

/// One entry of the synthetic tree.
struct Item {
    /// Its directory. The root is its own parent, which is what stops the walk.
    parent: usize,
    /// The last path segment.
    name: String,
    /// Which of the four it is.
    kind: Kind,
    /// Bytes. A directory reports `SUB-DIR` and carries this anyway, unused.
    size: u64,
    /// Minutes into the invented calendar. See [`Vfs::stamp`].
    minute: u32,
    /// **Deleted, and still in the vector.** Every index here is a position in one `Vec` and
    /// shifting them would invalidate two panels at once — so a delete marks rather than removes.
    ///
    /// **It is a flag and not a reparent**, which is what this file tried first: pointing a deleted
    /// entry at *itself* keeps it out of every listing and turns [`Vfs::path`]'s walk up the
    /// parents into an infinite loop the moment anything asks for its path — which `F5`'s
    /// destination search does, for every entry in the tree. The symptom is a dialog that will not
    /// close, because the application is inside `Driver::frame` and never comes out.
    dead: bool,
}

/// The tree, flat: every entry names its parent and nothing holds a child list.
///
/// **Flat because a listing is a filter and not a traversal.** A panel needs *the children of one
/// directory in one order*, which is one pass over this vector; a child list would be a second
/// structure to keep correct across a create and a delete, and both of those are keys on the
/// function bar.
struct Vfs {
    items: Vec<Item>,
    /// How many bytes the invented volume holds, and how many are used. The free-space readout.
    volume: (u64, u64),
}

impl Vfs {
    /// The free-space readout `mc` prints in a panel's bottom frame: free of total, and the
    /// percentage used.
    fn free_line(&self) -> String {
        let (total, used) = self.volume;
        let gb = 1024 * 1024 * 1024;
        format!(
            " {}G/{}G ({}%) ",
            (total - used) / gb,
            total / gb,
            used * 100 / total.max(1),
        )
    }

    /// The tree this application opens on.
    fn seed() -> Vfs {
        let mut vfs = Vfs {
            items: vec![Item {
                parent: 0,
                name: "/".to_owned(),
                kind: Kind::Dir,
                size: 0,
                minute: 0,
                dead: false,
            }],
            volume: (58 * 1024 * 1024 * 1024, 46 * 1024 * 1024 * 1024),
        };

        let bin = vfs.dir(0, "bin");
        for name in ["bash", "cargo", "git", "ls", "rustc", "ssh", "tmux", "vim"] {
            vfs.add(bin, name, Kind::Exec);
        }

        let etc = vfs.dir(0, "etc");
        for name in ["fstab", "hostname", "hosts", "passwd", "profile", "shells"] {
            vfs.add(etc, name, Kind::File);
        }
        vfs.add(etc, "localtime", Kind::Link);

        let home = vfs.dir(0, "home");
        let user = vfs.dir(home, "vitaly");
        let projects = vfs.dir(user, "projects");
        let vitui = vfs.dir(projects, "vitui");
        let crates = vfs.dir(vitui, "crates");
        for name in [
            "vitui-engine",
            "vitui-runtime",
            "vitui-components",
            "vitui-apps",
            "vitui-bench",
        ] {
            let at = vfs.dir(crates, name);
            vfs.add(at, "Cargo.toml", Kind::File);
            let src = vfs.dir(at, "src");
            vfs.add(src, "lib.rs", Kind::File);
        }
        let docs = vfs.dir(vitui, "docs");
        let adr = vfs.dir(docs, "adr");
        for n in 1..=12u32 {
            vfs.add(adr, &format!("{n:04}-a-decision.md"), Kind::File);
        }
        for name in [
            "CLAUDE.md",
            "CONTEXT.md",
            "Cargo.lock",
            "Cargo.toml",
            "README.md",
            "deny.toml",
            "rustfmt.toml",
        ] {
            vfs.add(vitui, name, Kind::File);
        }
        vfs.add(vitui, "target", Kind::Link);

        let devkit = vfs.dir(projects, "devkit");
        for name in ["README.md", "compose.yml"] {
            vfs.add(devkit, name, Kind::File);
        }
        vfs.add(devkit, "devkit", Kind::Exec);

        let downloads = vfs.dir(user, "downloads");
        for name in [
            "alacritty-0.15.1.tar.gz",
            "ghostty-1.2.0.dmg",
            "kitty-0.42.2.txz",
            "rust-1.88.0-aarch64-apple-darwin.pkg",
            "wezterm-20260812.zip",
        ] {
            vfs.add(downloads, name, Kind::File);
        }

        let notes = vfs.dir(user, "notes");
        for name in [
            "conform.md",
            "quirks.md",
            "terminals.md",
            "todo.txt",
            "wide-clusters.md",
        ] {
            vfs.add(notes, name, Kind::File);
        }
        vfs.add(user, ".profile", Kind::File);
        vfs.add(user, ".zshrc", Kind::File);

        let tmp = vfs.dir(0, "tmp");
        for n in 0..4u32 {
            vfs.dir(tmp, &format!("cargo-install{n:04}"));
        }

        let usr = vfs.dir(0, "usr");
        let share = vfs.dir(usr, "share");
        for name in ["man", "terminfo", "zsh"] {
            vfs.dir(share, name);
        }
        vfs.add(usr, "local", Kind::Link);

        vfs
    }

    /// Add a directory and answer its index.
    fn dir(&mut self, parent: usize, name: &str) -> usize {
        self.add(parent, name, Kind::Dir)
    }

    /// Add an entry and answer its index.
    ///
    /// The size and the timestamp come out of the name, so the screen is the same every run and a
    /// re-read is a *deliberate* change rather than a different random draw.
    fn add(&mut self, parent: usize, name: &str, kind: Kind) -> usize {
        let h = hash(name);
        self.items.push(Item {
            parent,
            name: name.to_owned(),
            kind,
            size: match kind {
                Kind::Dir => 0,
                Kind::Link => 8 + h % 40,
                _ => 1 + h % 4_000_000,
            },
            minute: u32::try_from(h % (360 * 1440)).unwrap_or(0),
            dead: false,
        });
        self.items.len() - 1
    }

    /// The absolute path of an entry, walked up through the parents.
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

    /// How many entries a directory holds, and how many bytes, all the way down.
    ///
    /// `mc`'s `F3` on a directory, which is the one thing its viewer does that is not a viewer.
    fn weigh(&self, at: usize) -> (usize, u64) {
        let (mut n, mut bytes) = (0, 0);
        let mut stack = vec![at];
        while let Some(dir) = stack.pop() {
            for i in 1..self.items.len() {
                if self.items[i].dead || self.items[i].parent != dir || i == dir {
                    continue;
                }
                n += 1;
                bytes += self.items[i].size;
                if self.items[i].kind == Kind::Dir {
                    stack.push(i);
                }
            }
        }
        (n, bytes)
    }

    /// `Mon DD HH:MM`, off an invented calendar of twelve thirty-day months.
    ///
    /// A real `mtime` would need a clock, and a component may own a time anchor and never a clock
    ///. This is neither: it is a label computed from the entry.
    fn stamp(&self, at: usize, out: &mut String) {
        const MONTHS: [&str; 12] = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        let m = self.items[at].minute;
        let days = m / 1440;
        out.clear();
        let _ = write!(
            out,
            "{} {:2} {:02}:{:02}",
            MONTHS[(days / 30 % 12) as usize],
            days % 30 + 1,
            m % 1440 / 60,
            m % 60,
        );
    }
}

/// A deterministic 64-bit hash of a name. FNV-1a, because the arithmetic is four lines and the
/// point is only that two runs agree.
fn hash(s: &str) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for b in s.as_bytes() {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// The part of a name after its last dot, or `""`. `mc` sorts by it and so does [`Sort::Extension`].
fn extension(name: &str) -> &str {
    name.rsplit_once('.').map_or("", |(_, ext)| ext)
}

// ── the panel ────────────────────────────────────────────────────────────────────────────────────

/// `mc`'s five sort orders.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Sort {
    Name,
    Extension,
    Size,
    Time,
    Unsorted,
}

impl Sort {
    /// The next one, so that one menu entry can walk the list.
    const fn next(self) -> Sort {
        match self {
            Sort::Name => Sort::Extension,
            Sort::Extension => Sort::Size,
            Sort::Size => Sort::Time,
            Sort::Time => Sort::Unsorted,
            Sort::Unsorted => Sort::Name,
        }
    }

    /// What the panel title says it is sorted by.
    const fn word(self) -> &'static str {
        match self {
            Sort::Name => "name",
            Sort::Extension => "extension",
            Sort::Size => "size",
            Sort::Time => "time",
            Sort::Unsorted => "unsorted",
        }
    }
}

/// One row of a panel: the `..` entry, or an entry of the tree.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RowRef {
    /// `..`, which `mc` shows as `UP--DIR` and which is never in the tree.
    Up(usize),
    /// An index into [`Vfs::items`].
    At(usize),
}

impl RowRef {
    /// The entry this row is about — for `..` that is the parent directory itself.
    const fn item(self) -> usize {
        match self {
            RowRef::Up(i) | RowRef::At(i) => i,
        }
    }
}

/// One directory panel.
struct Panel {
    /// The directory it is showing.
    cwd: usize,
    /// Its rows, in the sorted order. Rebuilt by [`Panel::relist`] and never by the draw.
    rows: Vec<RowRef>,
    /// The table's own state: the offset, the cursor and the horizontal offset.
    st: TableState,
    /// **The marks, which are this application's store and not the component's.**
    ///
    /// `Mode::Cursor` never selects, which is exactly `mc`'s panel: arrows move a cursor and leave
    /// the marks alone. So the marks are a [`Selection`] of this application's, moved through the
    /// published [`apply`] at `Mode::Multi` — the same thirteen arms a multi-select goes through,
    /// reached from outside the component.
    marks: Selection,
    /// What it is sorted by.
    sort: Sort,
    /// Whether that sort is reversed. `mc`'s `Reverse sort` menu entry.
    desc: bool,
    /// Stamped by [`Panel::relist`]. *One `u64` compared once a frame*: the component drops
    /// positions it can no longer trust rather than keeping a cursor on a deleted row.
    rev: Revision,
    /// The id the table drew under, learned from its response and used to seat the focus.
    id: Option<Id>,
}

impl Panel {
    fn new(cwd: usize) -> Panel {
        Panel {
            cwd,
            rows: Vec::new(),
            st: TableState::new(),
            marks: Selection::new(),
            sort: Sort::Name,
            desc: false,
            rev: Revision::UNKNOWN,
            id: None,
        }
    }

    /// Rebuild the row list, keeping the cursor on the entry it was on where that entry survives.
    ///
    /// **The cursor is re-found by identity and not by position**, which is the difference between
    /// a re-sort that keeps your place and one that throws you at row zero. The marks are carried
    /// the same way: old position → entry → new position.
    fn relist(&mut self, vfs: &Vfs) {
        let at = self.st.coll.sel.lead;
        let was = self.rows.get(at).copied();
        let marked: Vec<RowRef> = self
            .rows
            .iter()
            .enumerate()
            .filter(|(i, _)| self.marks.contains(*i))
            .map(|(_, r)| *r)
            .collect();

        self.rows.clear();
        if self.cwd != 0 {
            self.rows.push(RowRef::Up(vfs.items[self.cwd].parent));
        }
        let mut kids: Vec<usize> = (1..vfs.items.len())
            .filter(|&i| !vfs.items[i].dead && vfs.items[i].parent == self.cwd)
            .collect();

        // **Directories first, whatever the key.** `mc`'s own rule, and the reason the comparison
        // is a tuple rather than a `sort_by_key` on one field.
        kids.sort_by(|&a, &b| {
            let (ia, ib) = (&vfs.items[a], &vfs.items[b]);
            let dir = (ib.kind == Kind::Dir).cmp(&(ia.kind == Kind::Dir));
            let key = match self.sort {
                Sort::Name => ia.name.cmp(&ib.name),
                Sort::Extension => extension(&ia.name)
                    .cmp(extension(&ib.name))
                    .then_with(|| ia.name.cmp(&ib.name)),
                Sort::Size => ib.size.cmp(&ia.size),
                Sort::Time => ib.minute.cmp(&ia.minute),
                Sort::Unsorted => a.cmp(&b),
            };
            let key = if self.desc { key.reverse() } else { key };
            dir.then(key)
        });
        self.rows.extend(kids.into_iter().map(RowRef::At));

        self.marks.clear();
        for row in marked {
            if let Some(at) = self.rows.iter().position(|x| *x == row) {
                self.marks.toggle(at);
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

    /// The row the cursor is on.
    fn cursor(&self) -> Option<RowRef> {
        self.rows.get(self.st.coll.sel.lead).copied()
    }

    /// The entries the next file operation acts on: the marks, or the cursor when nothing is
    /// marked. **`mc`'s rule**, and the reason `F8` on an unmarked panel still deletes something.
    fn targets(&self) -> Vec<usize> {
        let mut out: Vec<usize> = self
            .rows
            .iter()
            .enumerate()
            .filter(|(i, r)| self.marks.contains(*i) && matches!(r, RowRef::At(_)))
            .map(|(_, r)| r.item())
            .collect();
        if out.is_empty()
            && let Some(RowRef::At(i)) = self.cursor()
        {
            out.push(i);
        }
        out
    }

    /// How many marked entries there are, and how many bytes they hold.
    fn marked_bytes(&self, vfs: &Vfs) -> (usize, u64) {
        let mut bytes = 0;
        let mut n = 0;
        for (i, row) in self.rows.iter().enumerate() {
            if let (true, RowRef::At(at)) = (self.marks.contains(i), row) {
                n += 1;
                bytes += vfs.items[*at].size;
            }
        }
        (n, bytes)
    }
}

// ── what is open over the panels ─────────────────────────────────────────────────────────────────

/// The one thing that can be open at a time. `mc` is modal in exactly this way.
///
/// **The viewer keeps its own text.** A `Vec<String>` built in the draw could not be reached from
/// an overlay body at all — see this file's header — so it lives here, where it is a field of the
/// application and outlives the frame.
enum Modal {
    /// Nothing. The panels have the keyboard.
    None,
    /// `F1`.
    Help,
    /// `F3` — the internal viewer.
    View {
        name: String,
        body: Vec<String>,
        top: usize,
    },
    /// `F4` — the internal editor. A [`Text::textarea`] over the same body.
    Edit { name: String, buf: Text },
    /// `F5`, `F6` and `F7`: one field and two buttons, differing in the verb.
    Ask { verb: Verb, to: Text, n: usize },
    /// `F8`.
    Delete { n: usize },
    /// `F9`, or a click on the menu bar.
    Menu { which: usize, st: CollState },
}

/// The three questions that are one dialog.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Verb {
    Copy,
    Move,
    Mkdir,
}

impl Verb {
    /// The dialog's own title, in `mc`'s words.
    const fn title(self) -> &'static str {
        match self {
            Verb::Copy => " Copy ",
            Verb::Move => " Move ",
            Verb::Mkdir => " Create a new directory ",
        }
    }

    /// The line above the field.
    const fn prompt(self) -> &'static str {
        match self {
            Verb::Copy => "Copy to:",
            Verb::Move => "Move to:",
            Verb::Mkdir => "Enter directory name:",
        }
    }
}

/// **What a dialog decided, written by an overlay body and read after the frame.**
///
/// An overlay body cannot answer its caller — it is a `FnMut` the frame owns — so this is the
/// inbox, which is `console`'s arrangement one application over.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Act {
    /// Close and do nothing.
    Close,
    /// The `OK` button of the copy/move/mkdir dialog.
    Commit,
    /// The `Delete` button.
    Delete,
    /// An entry of a pull-down was chosen.
    Menu(usize, usize),
    /// A click on the menu bar asked for a pull-down.
    ///
    /// **A modal opened from inside the draw does not appear until the frame after.** The overlay
    /// pass runs, the body draws, and the cells do not reach the screen — so a click on `File`
    /// looked like a click on nothing until the next keystroke. Every other decision in this file
    /// already goes through the inbox; this one was the exception and it is not any more.
    OpenMenu(usize),
    /// `Left`/`Right` inside a pull-down: open the neighbouring one.
    Sibling(usize),
    /// The viewer was scrolled by this many rows.
    Scroll(i32),
    /// A function key pressed inside a dialog, or on the function bar.
    Function(u8),
}

/// The five pull-downs of the menu bar, each with its entries.
const MENUS: [(&str, &[&str]); 5] = [
    (
        " Left ",
        &["Sort order...", "Reverse sort", "Re-read", "Swap panels"],
    ),
    (
        " File ",
        &[
            "View            F3",
            "Edit            F4",
            "Copy            F5",
            "Rename or Move  F6",
            "Make directory  F7",
            "Delete          F8",
            "Quit          C-q",
        ],
    ),
    (
        " Command ",
        &[
            "Directory tree",
            "Find file",
            "Swap panels",
            "Compare directories",
        ],
    ),
    (
        " Options ",
        &[
            "Configuration...",
            "Layout...",
            "Panel options...",
            "Appearance...",
        ],
    ),
    (
        " Right ",
        &["Sort order...", "Reverse sort", "Re-read", "Swap panels"],
    ),
];

/// The ten labels of the function bar, `filemanager.c`'s own.
const FKEYS: [&str; 10] = [
    "1Help", "2Menu", "3View", "4Edit", "5Copy", "6RenMov", "7Mkdir", "8Delete", "9PullDn",
    "10Quit",
];

/// The hints `mc` rotates along the bottom. Advanced when a command is run.
const HINTS: [&str; 6] = [
    "Tab moves between the panels; the focused frame is the active one.",
    "Insert marks and steps down. Ctrl+T inverts; Esc drops them.",
    "The prompt takes every printable key the panel declines — Space is not one.",
    "F5 and F6 act on the marks, or on the cursor when nothing is marked.",
    "Ctrl+U swaps the panels; Ctrl+R re-reads the active directory.",
    "Ctrl+Q quits. `q` is a character, and characters go to the prompt.",
];

/// The help overlay's text: `mc`'s own summary, shortened to what this port has.
const HELP: [&str; 17] = [
    " Midnight Commander, on the vitui component surface",
    "",
    " Tab              the other panel",
    " Enter            descend, or `..`",
    " Insert           mark and step down",
    " Ctrl+T           invert the marks",
    " Esc              clear the prompt, then the marks",
    " F3 / F4          view / edit the cursor entry",
    " F5 / F6 / F7     copy / move / make a directory",
    " F8               delete — tab, then enter: it opens on Cancel",
    " F9               the menu bar",
    " ← / →           the neighbouring pull-down",
    " Ctrl+R           re-read the active directory",
    " Ctrl+U           swap the panels",
    " Ctrl+Q           quit",
    "",
    " Esc closes this.",
];

/// The sentences the invented file bodies are made of.
const WORDS: [&str; 12] = [
    "the frame is flat in n, and the edit is linear with a stated constant",
    "damage is marked at write time, never derived by diffing",
    "a cell holds an interned grapheme-cluster handle, not a char",
    "the engine does not lay anything out; callers bring rectangles",
    "budgets are per class, and an over-budget screen is recorded beside the number",
    "a component names a Role, never a colour",
    "bars are reserved, never overlaid",
    "the clip stack is the call stack and the id path is the closure tree",
    "a frame consumes at most one routing edge",
    "the alternate screen is entered exactly once",
    "a gate that cannot fail is worse than no gate at all",
    "a figure quoted from a spec is usually a prototype's screen",
];

/// The invented body of a file, for the viewer and the editor.
fn body_of(vfs: &Vfs, at: usize) -> Vec<String> {
    let item = &vfs.items[at];
    let lines = 12 + usize::try_from(hash(&item.name) % 60).unwrap_or(0);
    let mut out = Vec::with_capacity(lines + 3);
    out.push(format!("# {}", vfs.path(at)));
    out.push(format!("# {} bytes, {}", item.size, item.kind.perms()));
    out.push(String::new());
    for n in 0..lines {
        let h = hash(&format!("{}:{n}", item.name));
        let pick = usize::try_from(h % WORDS.len() as u64).unwrap_or(0);
        out.push(format!("{:>4} | {}", n + 1, WORDS[pick]));
    }
    out
}

// ── the application ──────────────────────────────────────────────────────────────────────────────

/// The three columns of a panel. `mc`'s default listing.
const COLS: [Column; 3] = [
    Column::new(0, "Name", Weight(1)),
    Column::new(1, "Size", Fixed(10)),
    Column::new(2, "Modify time", Fixed(13)),
];

/// Everything this program knows.
struct App {
    vfs: Vfs,
    panels: [Panel; 2],
    /// Which panel the keyboard is on. Read off the focus after every draw rather than kept as the
    /// truth, so that `Tab` — the runtime's walk and no key of this application's — moves it free.
    active: usize,
    /// What is open over the panels.
    modal: Modal,
    /// The dialogs' inbox. Written inside an overlay body, read after the frame.
    pending: Option<Act>,
    /// The shell prompt's text. Not a `field`: see this file's header.
    prompt: String,
    /// The last thing the prompt ran, shown where `mc` shows its output.
    ran: String,
    /// Which hint is on the hint row.
    hint: usize,
    /// What the last action said.
    said: String,
    /// **Put the keyboard back on the active panel.** Set whenever a dialog closes: the focused
    /// widget stopped drawing, so the runtime's vanish rule moves the focus to *the nearest
    /// surviving entry in the previous frame's ring order* — which is the panel next to the one the
    /// user was on. Without this, `F5` on the left panel came back with the right one active.
    seat: bool,
    /// Set inside the draw, read by `main`.
    exit: bool,
    /// A scratch buffer for the timestamp column, so a frame formats into one allocation.
    stamp: String,
}

impl App {
    fn new() -> App {
        let vfs = Vfs::seed();
        let find = |path: &str| {
            (0..vfs.items.len())
                .find(|&i| vfs.path(i) == path)
                .unwrap_or(0)
        };
        let (projects, downloads) = (
            find("/home/vitaly/projects"),
            find("/home/vitaly/downloads"),
        );
        let mut app = App {
            vfs,
            panels: [Panel::new(projects), Panel::new(downloads)],
            active: 0,
            modal: Modal::None,
            pending: None,
            prompt: String::new(),
            ran: String::new(),
            hint: 0,
            said: String::new(),
            seat: true,
            exit: false,
            stamp: String::with_capacity(16),
        };
        app.relist_both();
        app
    }

    /// One frame, top to bottom.
    ///
    /// `&'f mut self` because an overlay body is `+ 'f`: a dialog holds `&'f mut` of this
    /// application's own fields, and a local of this function would not live long enough.
    fn ui<'f>(&'f mut self, cx: &mut Ctx<'f, '_>) {
        // **A standing seat of this application's wins**, because the answer below is about where
        // the *ring* left the focus and a dialog that has just closed left it somewhere nobody
        // asked for.
        //
        // **The active panel is read off the focus, and read *first*.** `Tab` is the ring's key and
        // no key of this application's; the ring resolves it in `settle`, after the previous frame
        // had drawn — so the answer is already here and everything below can use it. Read at the
        // *end* of the draw instead, the prompt and the frame roles would be a keystroke behind,
        // which is exactly how this was found. The ids are the previous frame's, which is all a
        // question about the focus needs.
        if !self.seat {
            if self.panels[1].id.is_some_and(|id| cx.is_focused(id)) {
                self.active = 1;
            } else if self.panels[0].id.is_some_and(|id| cx.is_focused(id)) {
                self.active = 0;
            }
        }

        let [menu, body, hint, prompt, fkeys] = Col::new().split(
            cx.area(),
            [Fixed(1), Weight(1), Fixed(1), Fixed(1), Fixed(1)],
        );
        let anchors = self.draw_menu_bar(cx, menu);
        let [left, right] = Row::new().split(body, [Weight(1), Weight(1)]);
        // **Two panels are two widgets, and `Ctx::id` is `Location::caller()`** — so both calls to
        // `draw_panel` mint *one* id, and the second panel would take the first one's focus, its
        // cursor and its hover. `with_key` is what tells them apart: the trap `CLAUDE.md` records
        // as `#[track_caller]` forwarding into every `#[track_caller]` function and through
        // nothing else. Found by running this application, not by reading it.
        cx.with_key(0, |cx| self.draw_panel(cx, left, 0));
        cx.with_key(1, |cx| self.draw_panel(cx, right, 1));
        self.draw_hint(cx, hint);
        self.draw_prompt(cx, prompt);
        self.draw_fkeys(cx, fkeys);

        // **The focus is seated only when nobody holds it**:
        // `focused().is_none()` rather than `!is_focused(id)`, which would drag the keyboard back
        // every frame the user had tabbed away — and `Tab` between the two panels is exactly that
        // walk. A table mints its own id, so the seat is the last statement of the draw.
        let want = self.panels[self.active].id;
        let modal_open = !matches!(self.modal, Modal::None);
        if !modal_open
            && (self.seat || cx.focused().is_none())
            && let Some(id) = want
        {
            cx.focus(id);
            self.seat = false;
        }

        self.draw_modal(cx, anchors);
    }

    /// The menu bar, and the rectangle each title sits in — which is where its pull-down anchors.
    fn draw_menu_bar(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) -> [Rect; 5] {
        let width = |i: usize| Fixed(u16::try_from(MENUS[i].0.len()).unwrap_or(8));
        let lanes = Row::new().split(
            area,
            [width(0), width(1), width(2), width(3), width(4), Weight(1)],
        );
        let open = match self.modal {
            Modal::Menu { which, .. } => Some(which),
            _ => None,
        };
        let mut anchors = [Rect::default(); 5];
        let mut clicked = None;
        for (i, (title, _)) in MENUS.iter().enumerate() {
            anchors[i] = lanes[i];
            // **Five titles from one call site are one widget.** `Ctx::interact_here` mints
            // `Location::caller()`, so without a key the five lanes share an id, the last one drawn
            // owns the hit rectangle, and clicking `File` does nothing at all. The same trap the two
            // panels met, one loop up — and the pointer is where it is invisible, because the
            // *hover* looks right on whichever lane happens to be last.
            let resp = cx.with_key(i as u64, |cx| {
                cx.interact_here(lanes[i], Interest::CLICK.with(Interest::HOVER))
            });
            let role = match (open == Some(i), resp.hovered) {
                (true, _) => Role::Selection,
                (false, true) => Role::FaceHover,
                (false, false) => Role::Face,
            };
            fit_with(
                cx,
                lanes[i],
                title,
                &FitOpts {
                    justify: Justify::Middle,
                    role,
                    pad: role,
                },
            );
            // **The press and not the click**, which is what a menu bar does everywhere and what
            // `Response::press_began` is for : the edge, true on the frame
            // the button went down, against the level `pressed` reports for the whole grab.
            if resp.press_began {
                clicked = Some(i);
            }
        }
        // The bar's own tail, so that the row is a partition and not a row with holes in it.
        fit_with(
            cx,
            lanes[5],
            "",
            &FitOpts {
                justify: Justify::Start,
                role: Role::Face,
                pad: Role::Face,
            },
        );
        if let Some(i) = clicked {
            self.pending = Some(Act::OpenMenu(i));
        }
        anchors
    }

    /// One panel: the frame, the table, the divider and the mini-status.
    fn draw_panel(&mut self, cx: &mut Ctx<'_, '_>, area: Rect, which: usize) {
        let active = which == self.active;
        let arrow = cx.theme().glyph(if self.panels[which].desc {
            Glyph::ArrowUp
        } else {
            Glyph::ArrowDown
        });
        let title = format!(
            " {} {arrow}{} ",
            self.vfs.path(self.panels[which].cwd),
            self.panels[which].sort.word(),
        );
        // **The free-space readout is the bottom border's**, which is where `mc` puts it and where
        // this port could not put it for most of its life. It is the volume and never the marked
        // total: a mark is about the cursor's panel and belongs on the mini-status row beside the
        // entry it counts.
        let free = self.vfs.free_line();
        let panel = panel_with(
            cx,
            area,
            &title,
            &free,
            &PanelOpts {
                // **This is where focus lands.** A focused panel is drawn in `Role::Focus` before
                // its cells are written, never restyled after — which is how `mc` marks the active
                // panel and how a ring is drawn.
                border: if active { Role::Focus } else { Role::Border },
                padded: false,
                // **`mc` centres both of them**, and one field carries the pair: a title pushed to
                // the middle over a free-space readout left at the edge is a screen nobody drew.
                justify: Justify::Middle,
                ..Default::default()
            },
        );
        if panel.interior.h < 3 {
            return;
        }
        let [list, divider, mini] =
            Col::new().split(panel.interior, [Weight(1), Fixed(1), Fixed(1)]);

        // Everything the cell drawer reads, taken out before the closure: `table` borrows the
        // panel's own state for the whole draw.
        let vfs = &self.vfs;
        let stamp = &mut self.stamp;
        let state = &mut self.panels[which];
        let rows = &state.rows;
        let marks = &state.marks;
        let opts = TableOpts {
            header: true,
            coll: CollOpts {
                // **A panel cursor never selects**, which is `Mode::Cursor` exactly. The marks are
                // this application's store — see `Panel::marks`.
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
            // **The type-ahead never matches, and that is what feeds the prompt.** See the header:
            // the component pushes the character, asks this, gets `None`, declines the key.
            &mut |_buf, _range| None,
            &mut |cx, r, cell: Cell, face: Face| {
                let Some(row) = rows.get(cell.row) else {
                    return;
                };
                let face = Face {
                    selected: marks.contains(cell.row),
                    ..face
                };
                let base = face_role(face);
                let item = &vfs.items[row.item()];
                let role = if base == Role::Body {
                    match row {
                        RowRef::Up(_) => Kind::Dir.role(),
                        RowRef::At(_) => item.kind.role(),
                    }
                } else {
                    base
                };
                let (text, justify) = match (cell.key, row) {
                    (0, RowRef::Up(_)) => ("/..".to_owned(), Justify::Start),
                    (0, RowRef::At(_)) => (item.name.clone(), Justify::Start),
                    // **The trailing space is the column separator**, and it is here rather than
                    // in the width: a right-justified cell puts its padding on the *left*, so a
                    // `Fixed(10)` size column and a date column beside it touch. `mc` reserves the
                    // gap in the same place — the size ends one cell short of its own edge.
                    (1, RowRef::Up(_)) => ("UP--DIR ".to_owned(), Justify::End),
                    (1, RowRef::At(_)) if item.kind == Kind::Dir => {
                        ("SUB-DIR ".to_owned(), Justify::End)
                    }
                    (1, RowRef::At(_)) => (format!("{} ", item.size), Justify::End),
                    _ => {
                        vfs.stamp(row.item(), stamp);
                        (stamp.clone(), Justify::Start)
                    }
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
        let descend = resp.double_clicked;

        rule_with(
            cx,
            divider,
            "",
            &RuleOpts {
                role: Role::Border,
                ..Default::default()
            },
        );
        self.draw_mini(cx, mini, which);
        if descend {
            self.descend(which);
        }
    }

    /// The mini-status: what the cursor is on, and what the volume has left.
    fn draw_mini(&mut self, cx: &mut Ctx<'_, '_>, area: Rect, which: usize) {
        let (marked, bytes) = self.panels[which].marked_bytes(&self.vfs);
        let cursor = self.panels[which].cursor();
        let left = match cursor {
            None => "  (empty)".to_owned(),
            Some(RowRef::Up(_)) => " drwxr-xr-x   UP--DIR             /..".to_owned(),
            Some(RowRef::At(at)) => {
                self.vfs.stamp(at, &mut self.stamp);
                let item = &self.vfs.items[at];
                let size = if item.kind == Kind::Dir {
                    "SUB-DIR".to_owned()
                } else {
                    item.size.to_string()
                };
                format!(
                    " {} {size:>9} {} {}",
                    item.kind.perms(),
                    self.stamp,
                    item.name,
                )
            }
        };
        // **The marked total and nothing else.** The volume moved to the bottom border, so this row
        // is the cursor's entry across the whole width until something is marked.
        let right = match marked {
            0 => String::new(),
            _ => format!(" {marked} marked, {bytes} bytes "),
        };
        let cut = u16::try_from(right.len()).unwrap_or(0).min(area.w);
        let (l, r) = rect::split_at_h(area, area.w.saturating_sub(cut));
        fit_with(
            cx,
            l,
            &left,
            &FitOpts {
                justify: Justify::Start,
                role: Role::Dim,
                pad: Role::Dim,
            },
        );
        fit_with(
            cx,
            r,
            &right,
            &FitOpts {
                justify: Justify::End,
                role: Role::Warn,
                pad: Role::Warn,
            },
        );
    }

    /// The hint row: `mc`'s rotating tip, or whatever the last action said.
    fn draw_hint(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) {
        let (line, role) = if self.said.is_empty() {
            (
                format!(" Hint: {}", HINTS[self.hint % HINTS.len()]),
                Role::Dim,
            )
        } else {
            (format!(" {}", self.said), Role::Ok)
        };
        fit_with(
            cx,
            area,
            &line,
            &FitOpts {
                justify: Justify::Start,
                role,
                pad: role,
            },
        );
    }

    /// The shell prompt, with the caret drawn where a shell would put it.
    fn draw_prompt(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) {
        let cwd = self.vfs.path(self.panels[self.active].cwd);
        let short = cwd
            .strip_prefix("/home/vitaly")
            .map_or_else(|| cwd.clone(), |rest| format!("~{rest}"));
        let head = format!(" vitaly@vitui:{short}$ ");
        let cut = u16::try_from(head.chars().count())
            .unwrap_or(area.w)
            .min(area.w);
        let (l, r) = rect::split_at_h(area, cut);
        text_with(
            cx,
            l,
            &head,
            &TextOpts {
                role: Role::Ok,
                pad: Role::Ok,
                ..Default::default()
            },
        );
        let shown = if self.ran.is_empty() {
            self.prompt.clone()
        } else {
            format!("{}   [{} — not a real shell]", self.prompt, self.ran)
        };
        fit_with(
            cx,
            r,
            &shown,
            &FitOpts {
                justify: Justify::Start,
                role: Role::Body,
                pad: Role::Body,
            },
        );
        // **The caret is the runtime's here and no component's**, because the prompt is not a
        // `field`: it is fed from the unhandled window, so no widget is holding it.
        if matches!(self.modal, Modal::None) {
            let at = r.x + i32::try_from(self.prompt.chars().count()).unwrap_or(0);
            if at < r.right() {
                cx.caret(at, r.y);
            }
        }
    }

    /// The ten-button function bar.
    fn draw_fkeys(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) {
        let resp = status_bar_with(
            cx,
            area,
            &FKEYS,
            (0, 0),
            &StatusOpts {
                role: Role::Face,
                pad: Role::Face,
                sep: Role::Border,
                justify: Justify::Middle,
                ..Default::default()
            },
        );
        // **A status bar declares one region for all ten segments**, so which one was
        // clicked is arithmetic on `Response::local` rather than ten hit entries.
        if let (true, Some((x, _))) = (resp.clicked, resp.local) {
            let per = i32::from(area.w) / i32::try_from(FKEYS.len()).unwrap_or(10);
            let n = (x / per.max(1)).clamp(0, 9);
            self.pending = Some(Act::Function(u8::try_from(n + 1).unwrap_or(1)));
        }
    }

    /// Whatever is open over the panels: one overlay, one body, one match.
    ///
    /// **One `cx.overlay` and not seven.** The body holds `&'f mut self.modal`, so seven bodies
    /// would be seven mutable borrows of one field; the anchor and the size are read out of the
    /// modal *before* the body takes it, which is the only ordering the borrow checker allows.
    fn draw_modal<'f>(&'f mut self, cx: &mut Ctx<'f, '_>, anchors: [Rect; 5]) {
        let screen = cx.area();
        let theme = *cx.theme();
        let host = cx.id();
        let App { modal, pending, .. } = self;
        let big = (
            screen.w.saturating_mul(4) / 5,
            screen.h.saturating_mul(4) / 5,
        );
        let (anchor, opts) = match &*modal {
            Modal::None => return,
            Modal::Help => (
                Stack::center(screen, 58.min(screen.w), 17.min(screen.h)),
                OverlayOpts::modal(58.min(screen.w), 17.min(screen.h), &theme),
            ),
            Modal::View { .. } | Modal::Edit { .. } => (
                Stack::center(screen, big.0, big.1),
                OverlayOpts::modal(big.0, big.1, &theme),
            ),
            Modal::Ask { .. } => (
                Stack::center(screen, 56.min(screen.w), 9.min(screen.h)),
                OverlayOpts::modal(56.min(screen.w), 9.min(screen.h), &theme),
            ),
            Modal::Delete { .. } => (
                Stack::center(screen, 52.min(screen.w), 7.min(screen.h)),
                OverlayOpts::modal(52.min(screen.w), 7.min(screen.h), &theme),
            ),
            Modal::Menu { which, .. } => {
                let entries = MENUS[*which].1;
                let w = entries
                    .iter()
                    .map(|e| u16::try_from(e.len() + 4).unwrap_or(24))
                    .max()
                    .unwrap_or(24)
                    .min(screen.w);
                let h = u16::try_from(entries.len() + 2).unwrap_or(6);
                (
                    anchors[*which],
                    OverlayOpts {
                        z: Z::MENU,
                        ..OverlayOpts::sized(w, h)
                    },
                )
            }
        };
        cx.overlay(host, anchor, opts, move |cx| {
            let area = cx.area();
            let shell = ShellOpts {
                kind: match modal {
                    Modal::Menu { .. } => ShellKind::Popup,
                    _ => ShellKind::Dialog,
                },
                ..ShellOpts::default()
            };
            // **The shell is handed the height it has**, so no bar is reserved. A reserved bar's
            // thumb is a function of the extent and the offset it is handed is a literal zero
            //  — the viewer wants one that moves, and draws its own below.
            let rows = u32::from(area.h);
            let _ = overlay_with(cx, area, rows, &shell, &mut |cx, r| {
                // **The frame is a `panel`, and the overlay does not draw one.** A shell reserves a
                // gutter and manages a body; a border and a title are `structure`'s, which is what
                // makes a dialog here look like `mc`'s and a pull-down look like its menu.
                let title = match modal {
                    Modal::None => String::new(),
                    Modal::Help => " Help ".to_owned(),
                    Modal::View { name, .. } => format!(" View: {name} "),
                    Modal::Edit { name, .. } => format!(" Edit: {name} "),
                    Modal::Ask { verb, .. } => verb.title().to_owned(),
                    Modal::Delete { .. } => " Delete ".to_owned(),
                    Modal::Menu { which, .. } => MENUS[*which].0.to_owned(),
                };
                let frame = panel_with(
                    cx,
                    r,
                    &title,
                    "",
                    &PanelOpts {
                        border: Role::Focus,
                        title_role: match modal {
                            Modal::Delete { .. } => Role::Danger,
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
                    Modal::View { body, top, .. } => draw_view(cx, r, body, *top, pending),
                    Modal::Edit { buf, .. } => draw_edit(cx, r, buf),
                    Modal::Ask { verb, to, n } => draw_ask(cx, r, *verb, to, *n, pending),
                    Modal::Delete { n } => draw_delete(cx, r, *n, pending),
                    Modal::Menu { which, st } => draw_menu(cx, r, *which, st, pending),
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

/// `F1`.
fn draw_help(cx: &mut Ctx<'_, '_>, r: Rect, pending: &mut Option<Act>) {
    for (i, line) in HELP.iter().enumerate() {
        let y = r.y + i32::try_from(i).unwrap_or(0);
        if y >= r.bottom() {
            break;
        }
        let role = if line.starts_with(' ') || line.is_empty() {
            Role::Body
        } else {
            Role::Title
        };
        fit_with(
            cx,
            Rect::new(r.x, y, r.w, 1),
            line,
            &FitOpts {
                justify: Justify::Start,
                role,
                pad: role,
            },
        );
    }
    let id = cx.id();
    let _ = cx.interact(id, r, Interest::FOCUS);
    if cx.focused().is_none() {
        cx.focus(id);
    }
    while let Some(k) = cx.next_key(id) {
        match k.code {
            Code::Escape | Code::Enter | Code::F(1) => *pending = Some(Act::Close),
            _ => cx.decline(k),
        }
    }
}

/// `F3` — the internal viewer.
fn draw_view(
    cx: &mut Ctx<'_, '_>,
    r: Rect,
    body: &[String],
    top: usize,
    pending: &mut Option<Act>,
) {
    let [text, bar] = Row::new().split(r, [Weight(1), Fixed(1)]);
    let [text, foot] = Col::new().split(text, [Weight(1), Fixed(1)]);
    for i in 0..usize::from(text.h) {
        let row = Rect::new(text.x, text.y + i32::try_from(i).unwrap_or(0), text.w, 1);
        let line = body.get(top + i).map_or("", String::as_str);
        fit_with(
            cx,
            row,
            line,
            &FitOpts {
                justify: Justify::Start,
                role: Role::Body,
                pad: Role::Body,
            },
        );
    }
    fit_with(
        cx,
        foot,
        &format!(
            " line {} of {}   arrows and PgUp/PgDn scroll   Esc closes ",
            (top + 1).min(body.len()),
            body.len(),
        ),
        &FitOpts {
            justify: Justify::Start,
            role: Role::Dim,
            pad: Role::Dim,
        },
    );
    // **A scrollbar of this application's, and not the shell's reserved one.** A reserved bar is
    // handed a literal zero for its offset , so its thumb cannot move; this one is
    // given the viewer's own span and therefore reports where the reader is.
    scrollbar(
        cx,
        bar,
        Span {
            viewport: u32::from(text.h),
            extent: u32::try_from(body.len()).unwrap_or(0),
            offset: u32::try_from(top).unwrap_or(0),
        },
    );
    let id = cx.id();
    let resp = cx.interact(id, r, Interest::FOCUS.with(Interest::SCROLL));
    if cx.focused().is_none() {
        cx.focus(id);
    }
    // **The notch is added, never negated, and never multiplied.** `crate::collect` establishes
    // the sign — `offset + resp.scrolled.1` — and `crate::input`'s field carries the warning in as
    // many words: *a field that negated it would scroll the wrong way on a screen where every
    // other scrollable widget scrolls the right one.* One notch is one row; how many notches a
    // physical detent produces is the terminal's answer and the engine has already applied it.
    let mut step = resp.scrolled.1;
    while let Some(k) = cx.next_key(id) {
        match k.code {
            Code::Escape | Code::F(3) => *pending = Some(Act::Close),
            Code::Down => step += 1,
            Code::Up => step -= 1,
            Code::PageDown => step += i32::from(text.h),
            Code::PageUp => step -= i32::from(text.h),
            Code::Home => step = i32::MIN / 2,
            Code::End => step = i32::MAX / 2,
            _ => cx.decline(k),
        }
    }
    if step != 0 && pending.is_none() {
        *pending = Some(Act::Scroll(step));
    }
}

/// `F4` — the internal editor.
fn draw_edit(cx: &mut Ctx<'_, '_>, r: Rect, buf: &mut Text) {
    let [edit, foot] = Col::new().split(r, [Weight(1), Fixed(1)]);
    // **One `field`, one flag.** An input and a textarea are the same component under two break
    // rules, and the rule is `WrapKind` on the state rather than an argument at the call.
    let resp = field(cx, edit, buf);
    if cx.focused().is_none() {
        cx.focus(resp.id);
    }
    fit_with(
        cx,
        foot,
        " Esc closes   Ctrl+Z undoes   Ctrl+A selects all   nothing is written anywhere ",
        &FitOpts {
            justify: Justify::Start,
            role: Role::Dim,
            pad: Role::Dim,
        },
    );
}

/// **Drain a dialog's buttons**, each answering with the act it stands for.
///
/// **A button reads no key** — it is a rectangle, a face and a response — so `Enter` on a *focused*
/// button is the caller's to drain, which is what makes `Tab` then `Enter` work. The field's own
/// `Enter` and `Esc` are declined and arrive in `Driver::unhandled`; a sink beside a focused field
/// would never be routed to, because `next_key` answers only the focused id.
///
/// **It is a function because two dialogs here wrote it identically and a third would have written
/// it again.** The arms are three — `Enter`, `Space`, `Escape` — and a copy that gets one of them
/// wrong is a dialog answering differently from its neighbour for no reason a reader can see.
fn drain_buttons(cx: &mut Ctx<'_, '_>, buttons: &[(Id, Act)], pending: &mut Option<Act>) {
    for (id, act) in buttons {
        while let Some(k) = cx.next_key(*id) {
            match k.code {
                Code::Enter | Code::Char(' ') => *pending = Some(*act),
                Code::Escape => *pending = Some(Act::Close),
                _ => cx.decline(k),
            }
        }
    }
}

/// `F5`, `F6`, `F7`.
fn draw_ask(
    cx: &mut Ctx<'_, '_>,
    r: Rect,
    verb: Verb,
    to: &mut Text,
    n: usize,
    pending: &mut Option<Act>,
) {
    let [what, prompt, entry, _gap, buttons] =
        Col::new().split(r, [Fixed(1), Fixed(1), Fixed(1), Fixed(1), Weight(1)]);
    let subject = match (verb, n) {
        (Verb::Mkdir, _) => " in the active panel".to_owned(),
        (_, 1) => " 1 entry".to_owned(),
        (_, n) => format!(" {n} entries"),
    };
    for (rect_, line, role) in [
        (what, subject.as_str(), Role::Dim),
        (prompt, verb.prompt(), Role::Body),
    ] {
        fit_with(
            cx,
            rect_,
            line,
            &FitOpts {
                justify: Justify::Start,
                role,
                pad: role,
            },
        );
    }
    let f = field(cx, rect::shrink(entry, 1, 0, 1, 0), to);
    let [ok, cancel] = Row::new()
        .spacing(2)
        .margin(1)
        .split(buttons, [Weight(1), Weight(1)]);
    let ok = button_with(cx, ok, "OK", &ButtonOpts::default());
    let cancel = button_with(cx, cancel, "Cancel", &ButtonOpts::default());
    if ok.clicked {
        *pending = Some(Act::Commit);
    }
    if cancel.clicked {
        *pending = Some(Act::Close);
    }
    // **The field is the default, and it is seated by name rather than by `focused().is_none()`.**
    // A `Kind::Dialog` opens a keyboard trap and the trap seats *something* — so the usual
    // condition is already false on the first frame, and the widget that happened to be first would
    // own the keyboard. Naming all three is what makes `Tab` still work.
    if ![f.id, ok.id, cancel.id].iter().any(|id| cx.is_focused(*id)) {
        cx.focus(f.id);
    }
    drain_buttons(
        cx,
        &[(ok.id, Act::Commit), (cancel.id, Act::Close)],
        pending,
    );
}

/// `F8`.
fn draw_delete(cx: &mut Ctx<'_, '_>, r: Rect, n: usize, pending: &mut Option<Act>) {
    let [what, _gap, buttons] = Col::new().split(r, [Fixed(1), Fixed(1), Weight(1)]);
    fit_with(
        cx,
        what,
        &format!("  Delete {n} entr{}?", if n == 1 { "y" } else { "ies" }),
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
    // **`Cancel` is where the keyboard starts, and saying so takes naming both.** The dialog's trap
    // seats a stop of its own on the first frame, so `focused().is_none()` is already false and
    // `Enter` would fall on whichever button the walk reached first — which for a delete is the
    // wrong default. k9s makes the same decision by requiring `Tab` before `Enter`.
    if ![yes.id, no.id].iter().any(|id| cx.is_focused(*id)) {
        cx.focus(no.id);
    }
    drain_buttons(cx, &[(yes.id, Act::Delete), (no.id, Act::Close)], pending);
}

/// `F9`, or a click on the menu bar.
fn draw_menu(
    cx: &mut Ctx<'_, '_>,
    r: Rect,
    which: usize,
    st: &mut CollState,
    pending: &mut Option<Act>,
) {
    let entries = MENUS[which].1;
    let opts = CollOpts {
        mode: Mode::Cursor,
        ..CollOpts::default()
    };
    // **The pull-down's own id, and the scope is what buys it a keyboard.** A container draws
    // *before* its children, so a pull API gives capture and not bubbling; the one moment an
    // ancestor can ask *after* its children is after its body, and `Ctx::scope` is the verb that
    // has one. A scope the focus drew inside becomes the routing target when it closes and the
    // decline that shut the queue is spent, so `next_key` below is answered.
    let menu = cx.id();
    let resp = cx.scope(menu, ScopeKind::Group, |cx| {
        collection(
            cx,
            r,
            st,
            &opts,
            Rows::of(entries.len()),
            &mut |_buf, _range| None,
            &mut |cx, row, i, face| {
                let role = face_role(face);
                fit_with(
                    cx,
                    row,
                    &format!(" {} ", entries[i]),
                    &FitOpts {
                        justify: Justify::Start,
                        role,
                        pad: role,
                    },
                );
            },
        )
    });
    if cx.focused().is_none() {
        cx.focus(resp.id);
    }
    if resp.clicked {
        *pending = Some(Act::Menu(which, st.sel.lead));
    }
    // **`mc`'s own bindings, and this surface could not have two of them.** `←` and `→` were
    // `vitui_components::nav::step`'s — a `collection` consumed both to move its cursor — so this
    // file bound `Alt+←`/`Alt+→` and recorded that the bare arrows were unreachable. A group
    // declares its axis now, a vertical one declines the horizontal arrows, and they arrive here.
    //
    // **`Enter` and `Esc` were reachable the whole time and this file did not know it.** They came
    // through `Driver::unhandled` a frame later, on the belief that a container over a focused
    // `collection` had no in-frame route at all — which compiled, drew correctly, and answered
    // every pull-down one keystroke behind.
    while let Some(k) = cx.next_key(menu) {
        match k.code {
            Code::Enter => *pending = Some(Act::Menu(which, st.sel.lead)),
            Code::Escape | Code::F(9) => *pending = Some(Act::Close),
            Code::Left => *pending = Some(Act::Sibling((which + MENUS.len() - 1) % MENUS.len())),
            Code::Right => *pending = Some(Act::Sibling((which + 1) % MENUS.len())),
            // Anything else is nobody's here — `Ctrl+Q` above all, which must work from inside
            // whatever is open.
            _ => cx.decline(k),
        }
    }
}

// ── what happens between two frames ──────────────────────────────────────────────────────────────

impl App {
    /// **The inbox, read after the draw.** A dialog cannot answer its caller, so this is where
    /// every dialog's decision lands.
    fn answer(&mut self) -> bool {
        let Some(act) = self.pending.take() else {
            return false;
        };
        match act {
            Act::Close => self.close(),
            Act::Commit => self.commit_ask(),
            Act::Delete => self.commit_delete(),
            Act::Menu(which, entry) => self.run_menu(which, entry),
            Act::OpenMenu(which) => self.open_menu(which),
            Act::Sibling(which) => self.open_menu(which),
            Act::Scroll(by) => {
                if let Modal::View { body, top, .. } = &mut self.modal {
                    let last = i64::try_from(body.len().saturating_sub(1)).unwrap_or(0);
                    let to = i64::from(by) + i64::try_from(*top).unwrap_or(0);
                    *top = usize::try_from(to.clamp(0, last)).unwrap_or(0);
                }
            }
            Act::Function(n) => {
                self.said.clear();
                self.function_key(n);
            }
        }
        true
    }

    /// Close whatever is open and give the keyboard back to the active panel.
    fn close(&mut self) {
        self.modal = Modal::None;
        self.seat = true;
    }

    /// Open one of the five pull-downs.
    fn open_menu(&mut self, which: usize) {
        self.modal = Modal::Menu {
            which,
            st: CollState::new(),
        };
    }

    /// Descend into the cursor's directory, or follow `..`.
    fn descend(&mut self, which: usize) {
        let Some(row) = self.panels[which].cursor() else {
            return;
        };
        let to = match row {
            RowRef::Up(parent) => parent,
            RowRef::At(at) if self.vfs.items[at].kind == Kind::Dir => at,
            RowRef::At(_) => return,
        };
        let leaving = self.panels[which].cwd;
        {
            let panel = &mut self.panels[which];
            panel.cwd = to;
            panel.marks.clear();
            panel.st.coll.offset = 0;
        }
        self.panels[which].relist(&self.vfs);
        // Coming up out of a directory, land on the one you left — the thing a file manager is
        // judged on.
        let panel = &mut self.panels[which];
        if let Some(at) = panel.rows.iter().position(|r| *r == RowRef::At(leaving)) {
            panel.st.coll.sel.lead = at;
        }
        self.said.clear();
    }

    /// One of the ten function keys.
    fn function_key(&mut self, n: u8) {
        let which = self.active;
        match n {
            1 => self.modal = Modal::Help,
            2 | 9 => self.open_menu(if which == 0 { 0 } else { 4 }),
            3 | 4 => {
                let Some(RowRef::At(at)) = self.panels[which].cursor() else {
                    return;
                };
                if self.vfs.items[at].kind == Kind::Dir {
                    // `mc`'s `F3` on a directory measures it rather than opening it, which is the
                    // one thing its viewer does that is not a viewer.
                    let (n, bytes) = self.vfs.weigh(at);
                    self.said = format!(
                        "{}: {n} entr{}, {bytes} bytes",
                        self.vfs.items[at].name,
                        if n == 1 { "y" } else { "ies" },
                    );
                    return;
                }
                let name = self.vfs.items[at].name.clone();
                let body = body_of(&self.vfs, at);
                self.modal = if n == 3 {
                    Modal::View { name, body, top: 0 }
                } else {
                    let mut buf = Text::textarea();
                    buf.edit(60, &body.join("\n"));
                    buf.set_pos(Caret::HOME);
                    Modal::Edit { name, buf }
                };
            }
            5 | 6 => {
                let targets = self.panels[which].targets().len();
                if targets == 0 {
                    self.said = "nothing to act on".to_owned();
                    return;
                }
                let mut to = Text::input();
                to.edit(60, &self.vfs.path(self.panels[1 - which].cwd));
                self.modal = Modal::Ask {
                    verb: if n == 5 { Verb::Copy } else { Verb::Move },
                    to,
                    n: targets,
                };
            }
            7 => {
                self.modal = Modal::Ask {
                    verb: Verb::Mkdir,
                    to: Text::input(),
                    n: 0,
                };
            }
            8 => {
                let targets = self.panels[which].targets().len();
                if targets == 0 {
                    self.said = "nothing to delete".to_owned();
                } else {
                    self.modal = Modal::Delete { n: targets };
                }
            }
            10 => self.exit = true,
            _ => {}
        }
    }

    /// Carry out the copy, the move or the mkdir the dialog was asking about.
    fn commit_ask(&mut self) {
        let Modal::Ask { verb, to, .. } = &self.modal else {
            return;
        };
        let verb = *verb;
        let text = to.text().trim().to_owned();
        let which = self.active;
        self.close();
        if text.is_empty() {
            self.said = "an empty name does nothing".to_owned();
            return;
        }
        match verb {
            Verb::Mkdir => {
                let cwd = self.panels[which].cwd;
                self.vfs.dir(cwd, &text);
                self.said = format!("created {text}");
            }
            Verb::Copy | Verb::Move => {
                let Some(dest) = (0..self.vfs.items.len()).find(|&i| {
                    !self.vfs.items[i].dead
                        && self.vfs.items[i].kind == Kind::Dir
                        && self.vfs.path(i) == text
                }) else {
                    self.said = format!("no such directory: {text}");
                    return;
                };
                let targets = self.panels[which].targets();
                for at in &targets {
                    if verb == Verb::Move {
                        self.vfs.items[*at].parent = dest;
                    } else {
                        let name = self.vfs.items[*at].name.clone();
                        let kind = self.vfs.items[*at].kind;
                        self.vfs.add(dest, &name, kind);
                    }
                }
                self.said = format!(
                    "{} {} entr{} to {text}",
                    if verb == Verb::Move {
                        "moved"
                    } else {
                        "copied"
                    },
                    targets.len(),
                    if targets.len() == 1 { "y" } else { "ies" },
                );
            }
        }
        self.relist_both();
    }

    /// Carry out the delete.
    ///
    /// **An entry is marked rather than removed**, because every other index in the tree is a
    /// position in one vector and shifting them would invalidate two panels at once. See
    /// [`Item::dead`] for why the mark is a flag and not a clever reparent.
    fn commit_delete(&mut self) {
        let targets = self.panels[self.active].targets();
        let mut n = 0;
        for at in targets {
            if at != 0 {
                self.vfs.items[at].dead = true;
                n += 1;
            }
        }
        self.said = format!("deleted {n} entr{}", if n == 1 { "y" } else { "ies" });
        self.close();
        self.relist_both();
    }

    /// Run one entry of one pull-down.
    fn run_menu(&mut self, which: usize, entry: usize) {
        self.close();
        let panel = usize::from(which == 4);
        match (which, entry) {
            (0 | 4, 0) => {
                self.panels[panel].sort = self.panels[panel].sort.next();
                self.panels[panel].relist(&self.vfs);
                self.said = format!("sorted by {}", self.panels[panel].sort.word());
            }
            (0 | 4, 1) => {
                self.panels[panel].desc = !self.panels[panel].desc;
                self.panels[panel].relist(&self.vfs);
                self.said = "reversed".to_owned();
            }
            (0 | 4, 2) => self.reread(),
            (0 | 4, 3) | (2, 2) => self.swap(),
            (1, 0) => self.function_key(3),
            (1, 1) => self.function_key(4),
            (1, 2) => self.function_key(5),
            (1, 3) => self.function_key(6),
            (1, 4) => self.function_key(7),
            (1, 5) => self.function_key(8),
            (1, 6) => self.exit = true,
            _ => {
                self.said = format!(
                    "{} is not built in this port",
                    MENUS[which].1[entry].trim_end()
                );
            }
        }
    }

    /// `Ctrl+R`: fresh sizes and timestamps for the active directory.
    fn reread(&mut self) {
        let which = self.active;
        let cwd = self.panels[which].cwd;
        let salt = self.vfs.items.len() as u64;
        for i in 1..self.vfs.items.len() {
            if self.vfs.items[i].parent != cwd {
                continue;
            }
            let h = hash(&self.vfs.items[i].name).wrapping_mul(salt | 1);
            if self.vfs.items[i].kind != Kind::Dir {
                self.vfs.items[i].size = 1 + h % 4_000_000;
            }
            self.vfs.items[i].minute = u32::try_from(h % (360 * 1440)).unwrap_or(0);
        }
        self.panels[which].relist(&self.vfs);
        self.said = "re-read".to_owned();
        self.hint = self.hint.wrapping_add(1);
    }

    /// `Ctrl+U`: swap the panels, which is what `mc` binds it to.
    fn swap(&mut self) {
        self.panels.swap(0, 1);
        self.active = 1 - self.active;
        self.said = "panels swapped".to_owned();
    }

    /// Rebuild both listings — after anything that changed the tree.
    fn relist_both(&mut self) {
        let vfs = &self.vfs;
        for panel in &mut self.panels {
            panel.relist(vfs);
        }
    }

    /// **What nothing wanted, read from the frame that has just drawn.**
    ///
    /// The panel declines every printable key — its search never matches — so this is where the
    /// prompt is fed from, and it is where `Enter`, `Insert`, the function keys and `Ctrl+Q`
    /// arrive, because a table owns none of those either.
    fn take_unhandled(&mut self, keys: &[Pressed]) {
        // **The quit chord is answered before anything else claims the keyboard.** Every branch
        // below returns early for whatever is open, and an application whose way out depends on
        // which dialog is up is an application people cannot leave.
        for k in keys {
            if matches!(k.code, Code::Char('q' | 'Q')) && k.mods.ctrl() {
                self.exit = true;
                return;
            }
        }
        // **A dialog whose focus is on a `field` is answered here, and that is forced.**
        // `Ctx::next_key` answers *only the focused id*, so the keyboard sink a dialog puts beside
        // its field is never routed to: what the field declines goes back on the queue and lands
        // in this window. The dialogs whose focus is on a *button* — `Delete`, `Help`, the viewer
        // — read their own keys inside the frame and are skipped here, which is why this is a
        // match on the modal rather than one early return.
        match &self.modal {
            Modal::None => {}
            Modal::Ask { .. } => {
                for k in keys {
                    match k.code {
                        Code::Enter => self.pending = Some(Act::Commit),
                        Code::Escape => self.pending = Some(Act::Close),
                        _ => {}
                    }
                }
                return;
            }
            Modal::Edit { .. } => {
                for k in keys {
                    if k.code == Code::Escape {
                        self.pending = Some(Act::Close);
                    }
                }
                return;
            }
            // **The pull-down reads its own keys inside the frame** and is skipped here, which is
            // the shape the two button dialogs already had. It was the exception in this file
            // until `draw_menu` opened a scope.
            Modal::Menu { .. } => return,
            Modal::Help | Modal::View { .. } | Modal::Delete { .. } => return,
        }
        for k in keys {
            let which = self.active;
            match k.code {
                Code::F(n) => {
                    self.said.clear();
                    self.function_key(n);
                }
                Code::Char('q' | 'Q') if k.mods.ctrl() => self.exit = true,
                Code::Char('r' | 'R') if k.mods.ctrl() => self.reread(),
                Code::Char('u' | 'U') if k.mods.ctrl() => self.swap(),
                Code::Enter if !self.prompt.is_empty() => {
                    self.ran = std::mem::take(&mut self.prompt);
                    self.hint = self.hint.wrapping_add(1);
                    self.said.clear();
                }
                Code::Enter => {
                    self.said.clear();
                    self.descend(which);
                }
                // **`Insert` and nothing on the character row**, which is `mc`'s own keymap:
                // `Mark = insert` and `Select/Unselect/SelectInvert = KP_Add/KP_Subtract/
                // KP_Multiply`. The keypad codes need application-keypad mode to arrive at all, so
                // `+`, `-` and `*` are left to the prompt here and the invert is a chord.
                Code::Insert => {
                    let lead = self.panels[which].st.coll.sel.lead;
                    let len = self.panels[which].rows.len();
                    if matches!(self.panels[which].rows.get(lead), Some(RowRef::At(_))) {
                        // **The published `apply`, at the mode a multi-select uses.** The store is
                        // this application's; the thirteen arms are the component's.
                        apply(
                            Mode::Multi,
                            &mut self.panels[which].marks,
                            len,
                            Gesture::Toggle(lead),
                        );
                    }
                    self.panels[which].st.coll.sel.lead = (lead + 1).min(len.saturating_sub(1));
                }
                Code::Char('t' | 'T') if k.mods.ctrl() => {
                    let len = self.panels[which].rows.len();
                    let panel = &mut self.panels[which];
                    for i in 0..len {
                        if matches!(panel.rows[i], RowRef::At(_)) {
                            panel.marks.toggle(i);
                        }
                    }
                    let n = self.panels[which].marks.count();
                    self.said = format!("{n} marked");
                }
                Code::Backspace => {
                    self.prompt.pop();
                }
                // **Two stages, and the order is the library's own.** `crate::collect::owns_escape`
                // is *the component owns `Esc` exactly when it would clear something*; the same
                // reading one level up makes the first `Esc` empty the prompt and the second drop
                // the marks, which is what a file manager does.
                Code::Escape if !self.prompt.is_empty() => {
                    self.prompt.clear();
                    self.said.clear();
                    self.ran.clear();
                }
                Code::Escape => {
                    let n = self.panels[which].marks.count();
                    self.panels[which].marks.clear();
                    self.said.clear();
                    self.ran.clear();
                    if n > 0 {
                        self.said = format!("{n} mark{} dropped", if n == 1 { "" } else { "s" });
                    }
                }
                _ => {
                    // **Every remaining printable key goes to the prompt**, which is `mc`'s rule.
                    // `keys::text` is the same reading the components use, so a chord is not text
                    // and a modifier alone is not a character.
                    let mut buf = String::new();
                    if typed_text(k, &mut buf) {
                        self.prompt.push_str(&buf);
                        self.said.clear();
                        self.ran.clear();
                    }
                }
            }
        }
    }
}

fn main() {
    let mut app = App::new();

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
        // **After the draw**: a dialog's decision was written inside the frame and cannot be acted
        // on from there.
        let acted = app.answer();
        // **The window onto the frame that has just drawn**, never the one before it: `unhandled`
        // is valid until the next frame begins, and reading it first acts one wake late — which
        // for a single keystroke means never.
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
