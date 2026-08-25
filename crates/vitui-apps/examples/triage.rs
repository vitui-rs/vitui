//! **Mail triage over two hundred thousand messages, and one component drawn four times.**
//!
//! The application components ticket 12 is for. `list`, option list, menu, multi-select, tabs, radio
//! group and segmented control are **one component and one [`Mode`]** (ADR 0028), and the way to see
//! that is not a paragraph — it is four collections on one screen whose only difference is the value
//! in `CollOpts::mode`:
//!
//! ```text
//! ┌ Triage ──────────────────────────────────────────────────────────────────────┐
//! │ ┌ View ────┐┌ Messages — 66 667 ─────────────────────┐┌ Actions ───────────┐ │
//! │ │ Unread   ││ ! ada        #0   one hit entry per …  ││ Archive            │ │
//! │ │ Flagged  ││   brendan    #3   damage is marked …   ││ Flag               │ │
//! │ │ All      ││ * carmack    #6   crossterm is …       ││ Mark read          │ │
//! │ └──────────┘│ …                                      ││ Move…              │ │
//! │ ┌ Folders ─┐│                                        ││ Delete             │ │
//! │ │ Inbox    ││                                        ││                    │ │
//! │ │ Sent     ││                                        ││                    │ │
//! │ └──────────┘└────────────────────────────────────────┘└────────────────────┘ │
//! │  cursor 41  anchor 38  selected 4 in 2 span(s), 32 B  ·  offset 24            │
//! └──────────────────────────────────────────────────────────────────────────────┘
//!   Mode::Options    Mode::Multi                              Mode::Cursor
//!   Mode::Single (Folders)
//! ```
//!
//! # What it is for, and what it makes visible
//!
//! **The status bar is the point.** A collection's store is three facts — the cursor, the anchor and
//! what is selected — and the third is a sorted, disjoint span list. Every gesture prints all five
//! numbers, so *select-all is one span whatever the length* and *every other gesture adds at most
//! one* are things you watch happen rather than things you are told. `Ctrl+A` over two hundred
//! thousand messages is **1 span and 16 bytes**; alt-clicking your way down alternate rows is one
//! span per click and nothing else changes.
//!
//! **The four modes are one component.** The tab strip, the folder list, the message list and the
//! action menu are four calls to `collection` with four values of `Mode` and four row drawers. The
//! menu is worth pressing `Tab` into: it has a cursor, `Space` does nothing to it, and nothing is
//! ever selected — which is what `Mode::Cursor` *is*.
//!
//! **The pointer gestures are real.** Spec §5 recorded ctrl-click and shift-click as inexpressible
//! above this runtime, because only `Key` carried a modifier byte. `Response::mods` carries them
//! now, and they route through the same `apply` as `Space` and `Shift+↑/↓` — the same six
//! `Gesture`s from both sides.
//!
//! # The keyboard
//!
//! | | |
//! |---|---|
//! | `Tab` / `Shift+Tab` | between the four collections — **each is one tab stop**, however many rows it has |
//! | `↑` `↓` `Home` `End` `PgUp` `PgDn` | move the cursor and select |
//! | `Shift+↑` `Shift+↓` | extend the selection from the anchor |
//! | `Ctrl+↑` `Ctrl+↓` | move the cursor and leave the selection alone |
//! | `Space` | toggle the row under the cursor |
//! | `Ctrl+A` | select every message. One span, at any length |
//! | `Esc` | clear the selection |
//! | a letter | type-ahead over senders, bounded — see the status bar's `search` |
//! | `Enter` | apply the action the menu's cursor is on to the selection |
//! | `q` | quit |
//!
//! # Switching the view is an edit, and the selection goes with it
//!
//! Components ticket 13. The message list stores **positions in the filtered order**, and switching
//! the filter replaces that order without changing any data — so nothing inside the collection can
//! notice, and the frame after would draw a perfectly correct list with the wrong messages
//! selected. The caller stamps a fresh revision on the change, the component compares one `u64`
//! once a frame, and the selection is cleared. Watch `rev` in the status bar move and the selection
//! empty as you `Tab` into **View** and press `↓`.
//!
//! # The pointer
//!
//! | | |
//! |---|---|
//! | click | select that one |
//! | ctrl-click | toggle it |
//! | shift-click | extend from the anchor |
//! | `ctrl`+shift-click | add the range to what is already selected |
//! | wheel | scroll, and the viewport **stays where you put it** |
//!
//! That last one is the wheel gate, from the other side: `collection` asks for a scroll-into-view
//! only when a **key** moved the cursor, so scrolling away from the selection does not fight you.
//!
//! # Two hundred thousand messages and no message store
//!
//! There is no `Vec<Message>`. A row's sender, subject and age are computed from its index, which is
//! the honest way to show the invariant `collection` is built on — *frame cost is proportional to
//! visible cells, never to data volume*. The frame draws the rows the window admits and nothing
//! else, so the screen costs the same at 200 000 as it would at 200.
//!
//! # Three things this application cannot say, and they are the surface's to fix
//!
//! 1. **A segmented control is the store and not the component.** §5 names *tabs* and *segmented
//!    control* among the seven `collection` absorbs, and the **policy** really is one arm of
//!    `apply` — but `collection` virtualises **rows**, so a strip laid out left to right cannot be
//!    drawn through its row loop: the rectangle a row drawer is handed is `(0, i, w, 1)` in the
//!    collection's own content coordinates, and a one-row-tall collection over three entries is a
//!    list of three with one visible. Written that way, the first draft of this file drew **one
//!    tab and clipped the other two**. The View strip below is therefore vertical, which is honest;
//!    a horizontal one shares `Selection` and `apply` and lays itself out.
//! 2. **A collection cannot be given an id.** It mints one from its call site (ADR 0013), so an
//!    application cannot name it before the first draw — which is why the focus is seated from the
//!    `Response` this frame returned rather than before the frame starts. It costs one frame, and
//!    architecture issue 25's rule (`cx.focused().is_none()`) still does the work.
//! 3. **`Ctx::with_id` cannot be used inside a scroll scope**, so ADR 0027's *wrap the row loop* is
//!    written as *wrap the whole component*. That is a finding recorded in components ticket 12
//!    rather than a thing this file works around.
//!
//! # Run it
//!
//! ```text
//! cargo run -p vitui-apps --example triage
//! ```
//!
//! [`Mode`]: vitui_components::collect::Mode

use vitui_components::collect::{CollOpts, CollState, Mode, collection};
use vitui_components::frame::{Face, face_paint};
use vitui_components::order::Rows;
use vitui_components::structure::{PanelOpts, panel_with};
use vitui_components::text::{FitOpts, Justify, fit_with};
use vitui_runtime::ctx::Driver;
use vitui_runtime::layout::{Col, Constraint::Fixed, Constraint::Weight, Row};
use vitui_runtime::work::Wake;
use vitui_runtime::{Ctx, Id, Interest, Rect, Revision, Role, Themes};

// ── the data, which is a function and not a store ────────────────────────────────────────────────

/// How many messages the mailbox holds. **Two hundred thousand, and none of them exists.**
const MESSAGES: usize = 200_000;

/// The senders, cycled. Deliberately unequal in length, so type-ahead has something to find.
const SENDERS: [&str; 16] = [
    "ada",
    "brendan",
    "carmack",
    "dijkstra",
    "edsger",
    "fran",
    "grace",
    "hopper",
    "ingrid",
    "joan",
    "katherine",
    "linus",
    "margaret",
    "niklaus",
    "ousterhout",
    "petra",
];

/// The subject lines, cycled against a different modulus so the pairs do not repeat.
const SUBJECTS: [&str; 11] = [
    "re: the render thread mirrors what the terminal shows",
    "damage is marked at write time",
    "one hit entry per collection",
    "the engine does not lay anything out",
    "re: re: a memo's key is every input",
    "select-all is one span",
    "the wheel gate is still pinned",
    "crossterm is invisible",
    "a gate is a count, a ratio, an equality",
    "no unsafe above the engine",
    "the scan cursor seeks once",
];

/// Who sent message `i`.
fn sender(i: usize) -> &'static str {
    SENDERS[i % SENDERS.len()]
}

/// What message `i` is about.
fn subject(i: usize) -> &'static str {
    SUBJECTS[(i / SENDERS.len()) % SUBJECTS.len()]
}

/// Whether message `i` is unread. Every third, by construction.
fn unread(i: usize) -> bool {
    i.is_multiple_of(3)
}

/// Whether message `i` is flagged. Every seventh.
fn flagged(i: usize) -> bool {
    i.is_multiple_of(7)
}

/// **An ASCII-case-insensitive prefix test that allocates nothing.**
///
/// `str::to_lowercase` would allocate once per row scanned, and the type-ahead budget is four
/// thousand rows — which would make one keystroke four thousand allocations against a frame budget
/// of zero. `crate::nav::matched` does the same thing over a `&[&str]`; this one is over a function.
fn starts_with_ci(hay: &str, needle: &str) -> bool {
    hay.len() >= needle.len()
        && hay
            .as_bytes()
            .iter()
            .zip(needle.as_bytes())
            .all(|(a, b)| a.eq_ignore_ascii_case(b))
}

// ── the four collections ─────────────────────────────────────────────────────────────────────────

/// **The inner panels' options: a border and no padding ring.**
///
/// Density is theme data and it changes rectangles (spec §3), so a padded panel spends two of its
/// rows on the ring. That is right for a dialog and wrong for a list five rows tall — and getting it
/// wrong shows up as a collection that draws one of its three entries, which is a *clip* and not a
/// missing row. Stated once here rather than four times below.
const INNER: PanelOpts = PanelOpts {
    border: Role::Border,
    title_role: Role::Title,
    pad: Role::Body,
    bordered: true,
    padded: false,
    interest: Interest::HOVER,
};

/// The tab strip's entries.
const TABS: [&str; 3] = ["Unread", "Flagged", "All"];

/// The folder list's entries.
const FOLDERS: [&str; 6] = ["Inbox", "Sent", "Drafts", "Archive", "Spam", "Trash"];

/// The action menu's entries. **A menu selects nothing**, which is what makes it a menu.
const ACTIONS: [&str; 5] = ["Archive", "Flag", "Mark read", "Move…", "Delete"];

/// Everything this application knows.
///
/// **Four `CollState`s and no message store.** Each is offset, selection, type-ahead buffer and the
/// editing slot — and nothing keyed by a row index, which is the rule the type is the check of.
struct App {
    tabs: CollState,
    folders: CollState,
    messages: CollState,
    actions: CollState,
    /// What the last `Enter` did, for the status bar.
    last: String,
    /// **The revision of the message order**, moved when the view filter changes.
    ///
    /// Components ticket 13, and it is the whole of §10 in one field: the message list stores
    /// *positions* in the filtered order, and switching the filter replaces that order without
    /// changing any data. Nothing inside the collection can notice — the frame after would draw a
    /// perfectly correct list with the wrong rows selected — so the caller stamps a fresh revision
    /// and the component compares it once a frame and applies `Policy::Clear`, which is the honest
    /// default for an edit nobody explained.
    order_rev: Revision,
    /// Which filter the revision above was stamped for, so the stamp happens on the change and not
    /// every frame. A revision that moves every frame is a `Clear` every frame.
    stamped_for: usize,
    exit: bool,
}

impl App {
    fn new() -> App {
        let mut app = App {
            tabs: CollState::new(),
            folders: CollState::new(),
            messages: CollState::new(),
            actions: CollState::new(),
            last: String::from("nothing yet"),
            order_rev: Revision::fresh(),
            stamped_for: 2,
            exit: false,
        };
        // Options can never be empty, so it starts on one. A `Mode::Options` collection whose store
        // is empty is a radio group with no button pressed, which is a state it has no gesture for.
        app.tabs.sel.select_only(2);
        app.folders.sel.select_only(0);
        app
    }

    /// How many messages the current tab shows. The tab strip is the filter.
    fn visible_messages(&self) -> usize {
        match self.tabs.sel.lead {
            0 => MESSAGES.div_ceil(3),
            1 => MESSAGES.div_ceil(7),
            _ => MESSAGES,
        }
    }

    /// One frame, top to bottom.
    fn ui(&mut self, cx: &mut Ctx<'_, '_>) {
        let outer = panel_with(cx, cx.area(), " Triage ", &PanelOpts::default());
        let [body, status_row] = Col::new().split(outer.interior, [Weight(1), Fixed(1)]);
        let [left, messages, actions] = Row::new().split(body, [Fixed(14), Weight(1), Fixed(18)]);
        // **Five rows for three entries**, because these panels are unpadded: two for the border
        // and one a row. A padded panel would spend the density ring on a list this short and show
        // one of the three — which is the same clipping the horizontal strip hit, arriving from the
        // other direction.
        let [view, folders] = Col::new().split(left, [Fixed(5), Weight(1)]);

        self.view_list(cx, view);
        // **The filter changed, so the order the message list holds positions in is a different
        // order.** One `u64`, stamped here and compared once a frame inside the component.
        if self.tabs.sel.lead != self.stamped_for {
            self.stamped_for = self.tabs.sel.lead;
            self.order_rev = Revision::fresh();
        }
        self.folder_list(cx, folders);
        let msg_id = self.message_list(cx, messages);
        self.action_menu(cx, actions);
        self.status(cx, status_row);

        // **Seat the keyboard on the message list while nobody holds it.** Architecture issue 25's
        // rule, with the one wrinkle a component that mints its own id adds: the id is only knowable
        // after the draw, so this is the last statement rather than the first. Guarded on
        // `focused().is_none()` and **not** on `!is_focused(msg_id)`, which would drag the keyboard
        // back every frame the user has tabbed away.
        if cx.focused().is_none() {
            cx.focus(msg_id);
        }
    }

    /// **`Mode::Options`: the view filter, and exactly one is on.**
    ///
    /// The only difference from the folder list beneath it is **one arm of `apply`** — `Options`
    /// refuses to reach zero and `Single` clears on the second press. Every other line of the two
    /// functions is the row drawer, which is ADR 0028's claim written out as two calls that differ
    /// in one field.
    ///
    /// It is **vertical**, and that is the finding in this file's header rather than a design
    /// choice: a horizontal segmented control shares `Selection` and `apply` and lays itself out,
    /// because `collection` virtualises rows.
    fn view_list(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) {
        let panel = panel_with(cx, area, " View ", &INNER);
        let opts = CollOpts {
            mode: Mode::Options,
            ..Default::default()
        };
        let _ = collection(
            cx,
            panel.interior,
            &mut self.tabs,
            &opts,
            Rows::of(TABS.len()),
            &mut |buf: &str, range: std::ops::Range<usize>| {
                range.into_iter().find(|&i| starts_with_ci(TABS[i], buf))
            },
            &mut |cx: &mut Ctx<'_, '_>, r: Rect, i: usize, face: Face| {
                let paint = face_paint(cx.theme(), face);
                let n = cx.text(r.x, r.y, TABS[i], paint);
                cx.fill(
                    Rect::new(
                        r.x + i32::from(n.cells),
                        r.y,
                        r.w.saturating_sub(n.cells),
                        1,
                    ),
                    " ",
                    paint,
                );
            },
        );
    }

    /// **`Mode::Single`: a folder list, and clicking the current one clears it.**
    fn folder_list(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) {
        let panel = panel_with(cx, area, " Folders ", &INNER);
        let opts = CollOpts {
            mode: Mode::Single,
            ..Default::default()
        };
        let _ = collection(
            cx,
            panel.interior,
            &mut self.folders,
            &opts,
            Rows::of(FOLDERS.len()),
            &mut |buf: &str, range: std::ops::Range<usize>| {
                range.into_iter().find(|&i| starts_with_ci(FOLDERS[i], buf))
            },
            &mut |cx: &mut Ctx<'_, '_>, r: Rect, i: usize, face: Face| {
                let paint = face_paint(cx.theme(), face);
                let n = cx.text(r.x, r.y, FOLDERS[i], paint);
                cx.fill(
                    Rect::new(
                        r.x + i32::from(n.cells),
                        r.y,
                        r.w.saturating_sub(n.cells),
                        1,
                    ),
                    " ",
                    paint,
                );
            },
        );
    }

    /// **`Mode::Multi`: two hundred thousand messages, and the whole gesture vocabulary.**
    fn message_list(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) -> Id {
        let shown = self.visible_messages();
        let title = format!(" Messages — {shown} ");
        let panel = panel_with(cx, area, &title, &INNER);
        let opts = CollOpts {
            mode: Mode::Multi,
            ..Default::default()
        };
        // **The tab's filter is a *view* over the same indices**, which is what makes this a
        // function rather than a rebuilt list. Components ticket 13's order/index is the general
        // form of it; here the filter is arithmetic and needs none.
        let at = |n: usize| match self.tabs.sel.lead {
            0 => n * 3,
            1 => n * 7,
            _ => n,
        };
        let resp = collection(
            cx,
            panel.interior,
            &mut self.messages,
            &opts,
            Rows::new(shown, self.order_rev),
            &mut |buf: &str, range: std::ops::Range<usize>| {
                range
                    .into_iter()
                    .find(|&n| starts_with_ci(sender(at(n)), buf))
            },
            &mut |cx: &mut Ctx<'_, '_>, r: Rect, n: usize, face: Face| {
                let i = at(n);
                let paint = face_paint(cx.theme(), face);
                // **Staged and blitted rather than formatted into a `String`.** One verb, no
                // allocation — which is what the standing budget *zero allocations during frame
                // composition* asks of a row drawn eighty times a frame.
                let mark = match (unread(i), flagged(i)) {
                    (_, true) => '!',
                    (true, _) => '*',
                    _ => ' ',
                };
                let _ = cx.stage(format_args!(
                    "{mark} {:<11} #{i:<6} {}",
                    sender(i),
                    subject(i)
                ));
                let n = cx.blit(r.x, r.y, paint);
                cx.fill(
                    Rect::new(
                        r.x + i32::from(n.cells),
                        r.y,
                        r.w.saturating_sub(n.cells),
                        1,
                    ),
                    " ",
                    paint,
                );
            },
        );
        // `Enter` on the message list applies the menu's action, which is what makes the menu a
        // menu: it holds a cursor and the *other* widget holds the selection.
        if resp.changed {
            self.last = format!(
                "{} on {} message(s)",
                ACTIONS[self.actions.sel.lead.min(ACTIONS.len() - 1)],
                self.messages.sel.count()
            );
        }
        resp.id
    }

    /// **`Mode::Cursor`: a menu, and nothing in it is ever selected.**
    ///
    /// Press `Tab` into it and try `Space`: the cursor moves and the store stays empty, because
    /// `apply`'s `(Mode::Cursor, _) => {}` arm is the second of the thirteen.
    fn action_menu(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) {
        let panel = panel_with(cx, area, " Actions ", &INNER);
        let opts = CollOpts {
            mode: Mode::Cursor,
            ..Default::default()
        };
        let _ = collection(
            cx,
            panel.interior,
            &mut self.actions,
            &opts,
            Rows::of(ACTIONS.len()),
            &mut |buf: &str, range: std::ops::Range<usize>| {
                range.into_iter().find(|&i| starts_with_ci(ACTIONS[i], buf))
            },
            &mut |cx: &mut Ctx<'_, '_>, r: Rect, i: usize, face: Face| {
                let paint = face_paint(cx.theme(), face);
                let n = cx.text(r.x, r.y, ACTIONS[i], paint);
                cx.fill(
                    Rect::new(
                        r.x + i32::from(n.cells),
                        r.y,
                        r.w.saturating_sub(n.cells),
                        1,
                    ),
                    " ",
                    paint,
                );
            },
        );
    }

    /// **The store, printed.** The cursor, the anchor, the spans and what they cost.
    fn status(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) {
        let sel = &self.messages.sel;
        let line = format!(
            " cursor {}  anchor {}  selected {} in {} span(s), {} B  ·  offset {}  ·  \
             search {:?}  ·  rev {}  ·  last: {}  ·  q to quit ",
            sel.lead,
            sel.anchor.map_or(-1, |a| a as i64),
            sel.count(),
            sel.span_count(),
            sel.bytes(),
            self.messages.offset,
            self.messages.ahead.buffer(),
            self.order_rev.raw(),
            self.last,
        );
        fit_with(
            cx,
            area,
            &line,
            &FitOpts {
                justify: Justify::Start,
                role: Role::Dim,
                pad: Role::Dim,
            },
        );
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

    // The first frame is drawn before the first park, for `counter`'s reason: parking with nothing
    // pending is indefinite by design and a screen that appears on the first keystroke is a bug.
    driver.frame(|cx| app.ui(cx));

    while !app.exit {
        match driver.wait() {
            Wake::Quit => break,
            Wake::Input | Wake::Posted | Wake::Deadline => {}
        }
        // `q` is the one key no collection owns, so it is declined all the way back out and read
        // here from what nothing wanted. A collection with a type-ahead buffer standing would have
        // eaten it, which is why the check is on the *unhandled* queue rather than inside a draw.
        if driver
            .unhandled()
            .iter()
            .any(|k| k.code == vitui_runtime::keys::Code::Char('q'))
        {
            app.exit = true;
        }
        driver.frame(|cx| app.ui(cx));
    }
}
