//! ratatui's counter-app tutorial, written against the vitui component surface.
//!
//! <https://ratatui.rs/tutorials/counter-app/basic-app/>
//!
//! **A port and not an invention, deliberately.** Its shape is not ours to argue with, so whatever
//! it cannot express here is a fact about this surface rather than a taste — and two things it
//! cannot express are recorded at the bottom of this comment instead of being quietly redesigned
//! away.
//!
//! ```text
//!   wait()          park until something happens — the loop's only blocking call
//!     |
//!   frame(|cx| …)   begin · base pass · overlay pass · end · settle · present
//!     |
//!   exit?           the application's own flag, set inside the draw
//! ```
//!
//! # What it demonstrates
//!
//! **The state is three fields and the library holds none of it.** There is no widget object, no
//! scene tree and nothing to register: `ui` runs top to bottom every frame, and a component is a
//! function that takes the rectangle it is to draw in.
//!
//! **The keyboard arrives through one queue.** `cx.key_map` declares the bindings for the frame,
//! `cx.next_key(id)` drains what this frame's batch carried, and a key nobody wants goes back with
//! `cx.decline`. A frame consumes at most one routing edge (ADR 0016), so a burst of keystrokes is
//! several frames rather than one — which is why holding an arrow key counts up smoothly instead of
//! jumping.
//!
//! **Nothing here names `vitui-engine`.** `Config` is unnameable above the runtime
//! (`vitui_runtime::line::ENGINE_NAMES`), and `Default::default()` names no type — inference reaches
//! what naming cannot, which is the same loophole `vitui_components::cells`'s `rect_in!` is built
//! on. The cost is real and worth knowing: an application that wants `max_frame_rate` or
//! `overrun_report` cannot have them from here, because `..Default::default()` in a struct literal
//! needs the struct's name.
//!
//! # Two things the port cannot say, and they are the surface's to fix
//!
//! 1. **The title is left-anchored.** ratatui centres it. `PanelOpts` has no title alignment.
//! 2. **The instructions are on an interior row, not in the bottom border.** ratatui puts them
//!    there with `Title::position(Bottom)`. There is no bottom title here.
//!
//! Both are missing fields on `PanelOpts` rather than anything structural, and neither is worked
//! around below — the point of a port is to show the gap, not to hide it.
//!
//! # Run it
//!
//! ```text
//! cargo run -p vitui-apps --example counter
//! ```

use vitui_components::cells::Cells;
use vitui_components::structure::{PanelOpts, panel_with};
use vitui_components::text::{Justify, TextOpts, text_with};
use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{ActionId, Chord, Code, KeyMap};
use vitui_runtime::work::Wake;
use vitui_runtime::{Ctx, Interest, Role, Themes};

/// Everything this application knows. Two fields and a flag.
struct App {
    counter: u8,
    exit: bool,
    /// Whether the keyboard has been given to anybody yet.
    ///
    /// **The runtime focuses nothing on its own, and nothing said so.** `Ctx::next_key` answers
    /// only the focused id — *nothing focused, nothing routed*, in its own doctest — and
    /// `frame.focused` starts as `None`. So an application that never calls [`Ctx::focus`] is deaf
    /// to the keyboard until a click awards the focus to something, which is exactly how this was
    /// found: `->` did nothing until the terminal was clicked.
    ///
    /// It is a flag rather than `if !cx.is_focused(sink) { cx.focus(sink) }`, and the difference
    /// only shows up in an application bigger than this one: the conditional form takes the focus
    /// **back** every frame the user has tabbed away, so `Tab` would appear to do nothing. *Give
    /// the keyboard to the widget it should start on* is a statement about the first frame, and a
    /// statement about a sequence of frames needs a value the application keeps —
    /// `vitui_components::app::Clears` is the same shape for a different property.
    focused: bool,
}

const DEC: ActionId = 1;
const INC: ActionId = 2;
const QUIT: ActionId = 3;

/// The bindings, built once. **Chords are stored inline** rather than behind a `&'static [Chord]`,
/// which is why two spellings of one action sit in one slice.
fn key_map() -> KeyMap {
    KeyMap::new()
        .bind(&[Chord::new(Code::Left), Chord::key('h')], DEC, "Decrement")
        .bind(
            &[Chord::new(Code::Right), Chord::key('l')],
            INC,
            "Increment",
        )
        .bind(&[Chord::key('q')], QUIT, "Quit")
}

impl App {
    /// One frame, top to bottom.
    fn ui(&mut self, cx: &mut Ctx<'_, '_>, map: &KeyMap) {
        cx.key_map(map);

        // The block. `panel_with` returns the interior it handed over **and did not write** — spec
        // §2's partition rule, which is why there is no second call to clear the inside.
        let block = panel_with(
            cx,
            Cells::of(cx),
            " Counter App Tutorial ",
            &PanelOpts {
                padded: false,
                ..Default::default()
            },
        );

        let (value_row, instructions) = block
            .interior
            .split_at_v(block.interior.h().saturating_sub(1));
        text_with(
            cx,
            value_row,
            &format!("Value: {}", self.counter),
            &TextOpts {
                justify: Justify::Middle,
                ..Default::default()
            },
        );
        text_with(
            cx,
            instructions,
            " Decrement <Left> Increment <Right> Quit <Q> ",
            &TextOpts {
                justify: Justify::Middle,
                role: Role::Dim,
                ..Default::default()
            },
        );

        // **The keyboard sink.** A tab stop over the whole screen, which is what makes this the
        // widget the key queue is drained by. A real application with several focusable widgets
        // gives each its own id and each drains its own; here there is one.
        let sink = cx.id();
        let _ = cx.interact(sink, cx.area(), Interest::FOCUS);
        // **Give it the keyboard on the first frame.** See `App::focused`: without this the
        // application is deaf until a click awards the focus to something.
        if !self.focused {
            cx.focus(sink);
            self.focused = true;
        }
        while let Some(key) = cx.next_key(sink) {
            match cx.action(&key) {
                Some(DEC) => self.counter = self.counter.saturating_sub(1),
                Some(INC) => self.counter = self.counter.saturating_add(1),
                Some(QUIT) => self.exit = true,
                // Not mine. Back on the queue, so a binding this frame did not claim is still
                // there for whoever does.
                _ => cx.decline(key),
            }
        }
    }
}

fn main() {
    let map = key_map();
    let mut app = App {
        counter: 0,
        exit: false,
        focused: false,
    };

    // **`Default::default()` and not `Config { .. }`** — see this file's header. The theme is the
    // shipped registry's first scheme; a registry nobody has told about the terminal claims no
    // colour distinction at all, which is the conservative answer rather than a placeholder.
    let mut driver = match Driver::attach(Default::default(), *Themes::standard().theme()) {
        Ok(driver) => driver,
        Err(why) => {
            eprintln!("vitui could not attach: {why}");
            return;
        }
    };

    // The first frame is drawn before the first park, because parking with nothing pending is
    // indefinite by design and a screen that appears on the first keystroke is a bug.
    driver.frame(|cx| app.ui(cx, &map));

    while !app.exit {
        match driver.wait() {
            Wake::Quit => break,
            Wake::Input | Wake::Posted | Wake::Deadline => {}
        }
        driver.frame(|cx| app.ui(cx, &map));
    }
}
