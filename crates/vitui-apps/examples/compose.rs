//! **`compose` — a title, a body, and the caret pair a megabyte deep.**
//!
//! The text field's application, and the only consumer of [`vitui_components::input::field`] that
//! is not a gate.
//!
//! ```text
//! cargo run -p vitui-apps --example compose            # a short note
//! cargo run -p vitui-apps --example compose -- --mega  # a pasted megabyte, 24 000 lines
//! ```
//!
//! # What it is for
//!
//! The headline is that **`input` and `textarea` are one component**, and an application is where
//! that stops being a sentence: the two widgets below are one function called twice over two states
//! whose only difference is [`WrapKind`]. The title takes `WrapKind::Ruler` — hard breaks every `w`
//! columns, no word breaks, no newlines — and the body takes `WrapKind::Words`. Nothing else about
//! them differs, and `Tab` moves between them because each is a tab stop and the focus ring is
//! built from the draw.
//!
//! The status bar prints what the four field gates are *about*, live: the caret's `(byte, column)` pair,
//! the visual row it is on out of how many, the revision, the width the wrap index was built at, and
//! the undo ring's two bounds. **Watch the width figure while resizing the terminal** — that is the
//! fourth gate, `the index's recorded width equals the width being drawn`, and it is the one a memo
//! keyed on the revision alone gets wrong on a resize while drawing 625 rows where 875 are needed.
//!
//! # Keys
//!
//! | key | what |
//! |---|---|
//! | anything printable | types, through `keys::text` — a chord types nothing |
//! | `←` `→` `↑` `↓` `Home` `End` | the caret, in **cluster** steps and **visual** rows |
//! | `Shift` + any of those | extends the anchored selection |
//! | `Backspace` `Delete` | one cluster, or the selection |
//! | `Ctrl+Z` | undo, restoring the **pair** |
//! | `Ctrl+A` | select all |
//! | `Tab` | the next tab stop — the ring's key, never the widget's |
//! | `Ctrl+Q` · `Esc` | quit |
//!
//! # `q` cannot be the quit key here, and that is the component working
//!
//! A focused `field` consumes every text-bearing key into its buffer — that is what it is for — so
//! an application whose quit key is a printable character has no quit key while a field holds the
//! focus. `ledger` and `explorer` hit the same wall from the other side (a focused `collection`
//! eats it into its type-ahead buffer) and bind `Ctrl+Q` beside `q`; here there is no `q` to bind
//! beside, because typing one is the whole point.
//!
//! # The application's keys are read after the frame, and there is no third place
//!
//! `Ctx::decline` hands a key back **and ends the level's turn at the queue**, so a key an
//! application owns cannot be read inside the draw once a component has declined one. It is read
//! from `Driver::unhandled`, which is *a window onto the same queue, valid until the next frame
//! begins* — so it is read immediately after this application's own frame and never before it.

use std::env;

use vitui_components::edit::{Text, WrapKind};
use vitui_components::input::field;
use vitui_components::structure::{PanelOpts, panel_with};
use vitui_components::text::{Justify, TextOpts, text_with};
use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{Code, Pressed};
use vitui_runtime::layout::rect;
use vitui_runtime::work::Wake;
use vitui_runtime::{Ctx, Mods, Rect, Role, Themes};

/// The note the `--mega` arm pastes: 24 000 lines, a little over a megabyte.
const MEGA_LINES: usize = 24_000;

/// What the short arm starts with. Deliberately not ASCII — every field gate runs on clusters that
/// are not one code point, and so should the thing a person looks at.
const OPENING: &str = "A cafe\u{301} note.\n\nThe caret moves in cluster steps, so \u{1f469}\u{200d}\u{1f469}\u{200d}\u{1f467} is one Left and one \
                       Right — not four, and not seven.\nWide glyphs count two columns: \u{4e2d}\u{6587}\u{3002}\n\nTab moves to the title \
                       above. Ctrl+Z undoes. Ctrl+Q quits.\n";

/// Everything this application knows.
struct App {
    /// The title. One row, [`WrapKind::Ruler`].
    title: Text,
    /// The body. [`WrapKind::Words`], and it holds whatever was pasted into it.
    body: Text,
    /// Set inside `take_unhandled`, read by the loop.
    exit: bool,
    /// What the status bar last saw, so the bar is a report of the frame and not of the state.
    seen: Seen,
}

/// What the status bar prints, gathered during the draw.
#[derive(Clone, Copy, Default)]
struct Seen {
    byte: usize,
    col: u16,
    row: usize,
    rows: usize,
    revision: u64,
    built_at: u16,
    selection: usize,
    entries: usize,
    ring_bytes: usize,
    truncated: bool,
    recomputes: u64,
}

impl App {
    /// One frame, top to bottom.
    fn ui(&mut self, cx: &mut Ctx<'_, '_>) {
        let block = panel_with(
            cx,
            cx.area(),
            " compose — a field is one component and one flag ",
            "",
            &PanelOpts::default(),
        );
        let interior = block.interior;
        if interior.h < 5 {
            return;
        }

        // Three bands: a one-row title, the body, and a one-row status bar. The rectangles are the
        // caller's and the components narrow nothing on their behalf — `CONTEXT.md`'s identity
        // rule, which is why `field` returns a `Response` and takes no closure.
        let (head, rest) = rect::split_at_v(interior, 1);
        let (body_rect, status) = rect::split_at_v(rest, rest.h.saturating_sub(2));
        let (label, entry) = rect::split_at_h(head, 8);
        text_with(
            cx,
            label,
            "title:",
            &TextOpts {
                role: Role::Dim,
                ..Default::default()
            },
        );

        // **One function, called twice, over two states.** The headline as two lines of an
        // application. The `Ruler` state is above and the `Words` state below, and neither call
        // says which is which.
        let head_resp = field(cx, entry, &mut self.title);
        let body_resp = field(cx, body_rect, &mut self.body);

        // **Seat the keyboard on the body while nothing holds it**, which is architecture issue
        // 25's one line inside the draw. `focused().is_none()` and never `!is_focused(id)` — the
        // second drags the keyboard back every frame the user has tabbed away, so `Tab` would
        // appear to do nothing.
        if cx.focused().is_none() {
            cx.focus(body_resp.id);
        }

        // The status bar reports whichever field holds the keyboard, because the pair it prints is
        // the one the four gates are about and there is only one caret on a screen.
        let which = if cx.is_focused(head_resp.id) {
            &mut self.title
        } else {
            &mut self.body
        };
        let w = if cx.is_focused(head_resp.id) {
            entry.w
        } else {
            body_rect.w
        };
        let caret = which.caret();
        self.seen = Seen {
            byte: caret.byte(),
            col: caret.col(),
            row: which.index(w).row_of(caret.byte()),
            rows: which.index(w).rows(),
            revision: which.revision(),
            built_at: which.index(w).built_at(),
            selection: which.selection().len(),
            entries: which.ring().len(),
            ring_bytes: which.ring().bytes(),
            truncated: which.ring().truncated(),
            recomputes: which.recomputes(),
        };
        self.status(cx, status);
    }

    /// The two status rows. A report of the frame that has just been drawn.
    fn status(&self, cx: &mut Ctx<'_, '_>, at: Rect) {
        let s = self.seen;
        let dim = TextOpts {
            role: Role::Dim,
            ..Default::default()
        };
        let (first, second) = rect::split_at_v(at, 1);
        text_with(
            cx,
            first,
            &format!(
                "caret (byte {}, col {})  row {} of {}  rev {}  index built at {}  selection {} B  \
                 recomputes {}",
                s.byte,
                s.col,
                s.row + 1,
                s.rows,
                s.revision,
                s.built_at,
                s.selection,
                s.recomputes,
            ),
            &dim,
        );
        text_with(
            cx,
            second,
            &format!(
                "undo {} entries / {} B{}   Tab switches · Ctrl+Z undo · Ctrl+A all · Ctrl+Q quit",
                s.entries,
                s.ring_bytes,
                if s.truncated { " (truncated)" } else { "" },
            ),
            &TextOpts {
                justify: Justify::Start,
                role: if s.truncated { Role::Warn } else { Role::Dim },
                ..Default::default()
            },
        );
    }

    /// **The application's own keys, read from what nothing wanted.** See this file's header.
    ///
    /// `Ctrl` specifically, and not *any modifier*: `Mods::chord()` only strips the **locks**, so a
    /// held `Shift` is in it — and `Shift+Q` is a capital `Q`, which is a character somebody typing
    /// produces. Reachable whenever nothing holds the focus, notably the short-terminal path above
    /// where no field is drawn at all. `vitui_components::keys::is_chord` is the crate's own answer
    /// to *is this an accelerator*, and it reads ctrl and alt.
    fn take_unhandled(&mut self, keys: &[Pressed]) {
        for key in keys {
            match key.code {
                Code::Escape => self.exit = true,
                Code::Char('q') if key.mods.chord().contains(Mods::CTRL) => self.exit = true,
                _ => {}
            }
        }
    }
}

fn main() {
    let mega = env::args().any(|a| a == "--mega");
    let body = if mega {
        let mut out = String::with_capacity(MEGA_LINES * 44);
        for i in 0..MEGA_LINES {
            out.push_str("the quick brown fox jumps over the lazy dog ");
            out.push_str(&i.to_string());
            out.push('\n');
        }
        out
    } else {
        OPENING.to_string()
    };

    let mut app = App {
        title: Text::of(
            if mega { "a pasted megabyte" } else { "a note" }.to_string(),
            WrapKind::Ruler,
        ),
        body: Text::of(body, WrapKind::Words),
        exit: false,
        seen: Seen::default(),
    };

    let mut driver = match Driver::attach(Default::default(), *Themes::standard().theme()) {
        Ok(driver) => driver,
        Err(why) => {
            eprintln!("vitui could not attach: {why}");
            return;
        }
    };

    loop {
        driver.frame(|cx| app.ui(cx));
        // **Read immediately after this application's own frame and never before it.** The window
        // is valid until the next frame begins, so a loop that read it before its own frame acted
        // on the previous frame's window — one wake late, which for a single keystroke means never.
        let unhandled: Vec<Pressed> = driver.unhandled().to_vec();
        app.take_unhandled(&unhandled);
        if app.exit {
            break;
        }
        match driver.wait() {
            Wake::Quit => break,
            Wake::Input | Wake::Posted | Wake::Deadline => {}
        }
    }
}
