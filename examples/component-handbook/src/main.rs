//! **The worked example behind `docs/guide/`** — five components written against `vitui-runtime`
//! alone, and the loop that drives them.
//!
//! ```text
//!   wait()          park until something happens — the loop's only blocking call
//!     |
//!   frame(|cx| …)   begin · base pass · overlay pass · end · settle · present
//!     |
//!   unhandled()     the keys nobody took, read AFTER the frame and never before
//! ```
//!
//! # What this file is here to show
//!
//! **The screen clears once.** Not every frame — that is the single largest damage figure an
//! application can produce, and it is invisible because the screen looks right. Not never either:
//! the cells nobody paints keep what was there before. Once, on the first frame and on a resize,
//! which needs a value that survives a frame and therefore belongs to the application.
//!
//! **The keyboard is drained by whoever has the focus, and what nobody takes comes back here.** A
//! component reads `cx.next_key(its own id)` and hands back what it does not own; the runtime's
//! focus ring answers `Tab` itself; and the keys that reached nobody are in `Driver::unhandled`
//! after the frame — which is where an application binds *quit*, because a printable quit key is
//! not always available and a focused text field would eat one.
//!
//! **Nothing here names `vitui-engine`.** The dependency is `vitui-runtime` and nothing else, which
//! is the claim the guide makes: a component is a function over a `Ctx`, and the component library
//! is one consumer of that surface rather than a privileged one.
//!
//! # Run it
//!
//! ```text
//! cd examples/component-handbook && cargo run
//! ```
//!
//! `Tab` moves the focus, `←`/`→` and `+`/`-` drive the stepper, `j`/`k` and the wheel drive the
//! list, the button opens a menu, and `q` or `Ctrl+Q` leaves.

mod ink;
mod menu;
mod meter;
mod picker;
mod pill;
mod stepper;

use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{ActionId, Chord, KeyMap};
use vitui_runtime::layout::{Col, Constraint, Row};
use vitui_runtime::work::Wake;
use vitui_runtime::{Ctx, Role, Themes};

/// **The application's clear, and the whole of its cross-frame drawing state.**
///
/// Two words. It is not a component and it cannot be one: every other rule here is a statement
/// about one rectangle inside one frame, which a function can obey, and this one is a statement
/// about a *sequence* of frames.
#[derive(Default)]
struct Clears {
    seen: Option<(u16, u16)>,
    count: u32,
}

impl Clears {
    /// Clear if this is the first frame or the size has changed. Answers whether it did.
    fn frame(&mut self, cx: &mut Ctx<'_, '_>) -> bool {
        let screen = cx.area();
        let size = (screen.w, screen.h);
        if self.seen == Some(size) {
            return false;
        }
        let body = cx.theme().paint(Role::Body);
        cx.clear(body);
        self.seen = Some(size);
        self.count += 1;
        true
    }
}

/// The menu's entries. `'static`, so the body the frame keeps can borrow them.
const SCHEMES: &[&str] = &["Restore", "Compact", "Roomy", "Monochrome"];

/// The list's rows.
const HOSTS: &[&str] = &[
    "alder", "birch", "cedar", "dogwood", "elm", "fir", "gum", "hazel", "ironwood", "juniper",
    "katsura", "larch", "maple", "nyssa", "oak", "pine", "quince", "rowan", "spruce", "teak",
    "ulmus", "viburnum", "willow", "yew", "zelkova",
];

/// Quit, bound outside the draw because it is read outside the draw.
const QUIT: ActionId = 1;

struct App {
    clears: Clears,
    /// The stepper's contract, rendered once at start-up. Help text is derived from the binding
    /// table rather than typed out beside it, so a chord that is added cannot go unmentioned.
    help: String,
    volume: i32,
    list: picker::PickerState,
    menu: menu::MenuState,
    popup: menu::MenuPopup,
    note: String,
}

impl App {
    /// One frame, top to bottom. There is no update step and no retained tree: the screen is a pure
    /// function of these fields, re-derived every time.
    ///
    /// `&'f mut self` and not `&mut self`: `menu` hands the overlay body a borrow that has to
    /// outlive the frame call, so the state it reaches has to be borrowed for `'f` too. The
    /// destructure on the first line is what keeps that from swallowing the whole of `self`.
    fn ui<'f>(&'f mut self, cx: &mut Ctx<'f, '_>) {
        let App {
            clears,
            help,
            volume,
            list,
            menu: menu_state,
            popup,
            note,
        } = self;

        clears.frame(cx);

        let screen = cx.area();
        let [header, body, footer] = Col::new().split(
            screen,
            [
                Constraint::Fixed(1),
                Constraint::Weight(1),
                Constraint::Fixed(1),
            ],
        );

        let title = cx.theme().paint(Role::Title);
        cx.text(header.x, header.y, " vitui component handbook", title);

        let [left, right] = Row::new()
            .spacing(2)
            .margin(1)
            .split(body, [Constraint::Weight(1), Constraint::Weight(1)]);

        let [menu_row, pill_row, step_row, meter_row, rest] = Col::new().spacing(1).split(
            left,
            [
                Constraint::Fixed(1),
                Constraint::Fixed(1),
                Constraint::Fixed(1),
                Constraint::Fixed(3),
                Constraint::Weight(1),
            ],
        );

        // The overlay owner. Its answer arrives on the frame after the click — see `menu`.
        menu::menu(cx, menu_row, "Density", menu_state, popup, SCHEMES);

        let liked = pill::pill(cx, pill_row, "A pill that reacts");
        if liked.clicked {
            note.clear();
            note.push_str("the pill was clicked");
        }

        let stepped = stepper::stepper(cx, step_row, volume);
        if stepped.changed {
            note.clear();
            note.push_str("volume moved");
        }

        meter::meter(cx, meter_row, *volume as f32 / 100.0);

        // The rectangle nobody else claimed is still somebody's to paint.
        let dim = cx.theme().paint(Role::Dim);
        cx.fill(rest, " ", dim);

        let picked = picker::picker(cx, right, HOSTS, list);
        if let Some(row) = picked.chosen {
            note.clear();
            note.push_str(HOSTS[row]);
        }

        // **Seat the focus on the first frame.** `focused().is_none()` and never
        // `!is_focused(id)`: the second takes the keyboard back every frame the user has tabbed
        // away, so `Tab` appears to do nothing.
        if cx.focused().is_none() {
            cx.focus(stepped.id);
        }

        let body_paint = cx.theme().paint(Role::Body);
        cx.fill(footer, " ", body_paint);
        cx.text(footer.x, footer.y, note, body_paint);
        // The focused component's own contract, where a user looks for it.
        let hint: &str = if stepped.focused {
            help
        } else {
            "Tab move focus   q quit"
        };
        let hint_x = footer.x + i32::from(footer.w) - (hint.len() as i32 + 1);
        cx.text(hint_x, footer.y, hint, dim);
    }
}

/// **One headless frame, drawn through a counter, with the numbers printed.**
///
/// `cargo run -- --probe`. It is the same call sites as the real screen, with the third spelling of
/// each component instead of the first — which is the whole reason the third spelling exists. A
/// number nobody can produce on demand is a number nobody checks.
fn probe() {
    let mut driver = match Driver::headless(80, 24) {
        Ok(driver) => driver,
        Err(why) => {
            eprintln!("vitui could not attach a sink: {why}");
            return;
        }
    };
    let mut tally = ink::Tally::default();
    let mut volume = 40;
    let mut list = picker::PickerState::default();

    driver.frame(|cx| {
        let screen = cx.area();
        let [left, right] =
            Row::new().split(screen, [Constraint::Weight(1), Constraint::Weight(1)]);
        let [a, b, c, rest] = Col::new().split(
            left,
            [
                Constraint::Fixed(1),
                Constraint::Fixed(1),
                Constraint::Fixed(3),
                Constraint::Weight(1),
            ],
        );
        pill::pill_into(&mut tally, cx, a, "A pill that reacts", &Default::default());
        stepper::stepper_into(&mut tally, cx, b, &mut volume, &Default::default());
        meter::meter_into(&mut tally, cx, c, 0.4, &Default::default());
        picker::picker_into(&mut tally, cx, right, HOSTS, &mut list, &Default::default());
        let dim = cx.theme().paint(Role::Dim);
        cx.fill(rest, " ", dim);
    });

    let frame = driver.inspect();
    println!("verbs            {}", tally.verbs);
    println!("cells written    {}", tally.cells);
    println!("distinct cells   {}", tally.distinct());
    println!("written twice    {}", tally.excess());
    println!("regions declared {}", frame.hits().len());
    println!("tab stops        {}", frame.stop_count());
}

fn main() {
    if std::env::args().any(|a| a == "--probe") {
        probe();
        return;
    }

    // The map is built once and lives outside the frame, because the keys it answers are read
    // outside the frame.
    let quit = KeyMap::new().bind(&[Chord::key('q'), Chord::key('q').ctrl()], QUIT, "Quit");

    let mut app = App {
        clears: Clears::default(),
        help: stepper::help(),
        volume: 40,
        list: picker::PickerState::default(),
        menu: menu::MenuState::default(),
        popup: menu::MenuPopup::default(),
        note: String::from("ready"),
    };

    let mut driver = match Driver::attach(Default::default(), *Themes::standard().theme()) {
        Ok(driver) => driver,
        Err(why) => {
            eprintln!("vitui could not attach: {why}");
            return;
        }
    };

    // The first frame is drawn before the first park: parking with nothing pending is indefinite by
    // design, and a screen that appears on the first keystroke is a bug.
    driver.frame(|cx| app.ui(cx));

    loop {
        match driver.wait() {
            Wake::Quit => break,
            Wake::Input | Wake::Posted | Wake::Deadline => {}
        }
        driver.frame(|cx| app.ui(cx));

        // **After the frame, never before.** `unhandled` is a window onto the same queue, valid
        // until the next frame begins.
        let leaving = driver
            .unhandled()
            .iter()
            .any(|k| quit.match_first(k) == Some(QUIT));
        if leaving {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use vitui_runtime::keys::{Chord, Code, Edge, Pressed, Text};
    use vitui_runtime::{Ctx, Interest, Rect, Response};

    use super::*;

    /// A key as the terminal would have delivered it. A chord is a *pattern*; this is a press.
    fn press(c: Chord) -> Pressed {
        Pressed {
            code: c.code,
            mods: c.mods,
            kind: Edge::Press,
            text: Text::EMPTY,
            at: Instant::now(),
        }
    }

    fn headless(w: u16, h: u16) -> Driver {
        Driver::headless(w, h).expect("a sink attaches")
    }

    /// **The partition rule, as a number.** Every cell of the rectangle once, and none of it twice.
    ///
    /// Both halves matter and neither implies the other: `distinct == area` alone is satisfied by a
    /// component that paints the whole rectangle three times, and `excess == 0` alone is satisfied
    /// by one that paints half of it.
    #[test]
    fn a_meter_writes_every_cell_of_its_rectangle_exactly_once() {
        let mut driver = headless(40, 6);
        let mut tally = ink::Tally::default();
        let area = Rect::new(3, 1, 30, 3);
        driver.frame(|cx| {
            meter::meter_into(&mut tally, cx, area, 0.37, &Default::default());
        });
        assert_eq!(tally.distinct(), usize::from(area.w) * usize::from(area.h));
        assert_eq!(tally.excess(), 0, "a cell was painted twice");
    }

    /// **A key consumed to do nothing is the defect this asserts against.**
    ///
    /// At the ceiling the stepper declines `→`, so it reaches `Driver::unhandled` and a container
    /// above could still bind it. Below the ceiling it takes it and the value moves.
    #[test]
    fn a_stepper_declines_the_key_it_cannot_act_on() {
        let opts = stepper::StepperOpts::default();

        // **One call site, called from both frames.** Two `stepper_with(…)` lines would be two
        // widgets with two ids — `Ctx::id` is `Location::caller()` — and the focus planted on the
        // first would be cleared at the end of the second by the vanish rule, which reads as *the
        // key was never routed*. This helper is deliberately **not** `#[track_caller]`: the id is
        // minted at its own line, which is the same line both times.
        fn draw(cx: &mut Ctx<'_, '_>, value: &mut i32, opts: &stepper::StepperOpts) -> Response {
            stepper::stepper_with(cx, Rect::new(0, 0, 20, 1), value, opts)
        }

        for (start, expect_taken) in [(opts.max, false), (opts.min, true)] {
            let mut driver = headless(20, 3);
            let mut value = start;
            // The first frame is what makes the widget exist; the keyboard is answered only for the
            // focused id, so the focus has to be somewhere before a key is worth posting.
            // `Driver::plant` is the test door for that — an application calls `Ctx::focus`.
            let mut id = None;
            driver.frame(|cx| id = Some(draw(cx, &mut value, &opts).id));
            driver.plant(None, id, None);
            driver.post_key(press(Chord::new(Code::Right)));
            driver.frame(|cx| {
                draw(cx, &mut value, &opts);
            });

            let taken = driver.unhandled().is_empty();
            assert_eq!(taken, expect_taken, "starting at {start}");
            assert_eq!(value != start, expect_taken, "starting at {start}");
        }
    }

    /// **The frame's cost is proportional to what is visible, never to what exists.**
    ///
    /// Ten rows and a hundred thousand draw the same screen at the same price. If this ever fails,
    /// something in the component started asking the data a question it cannot answer in `O(1)`.
    #[test]
    fn a_picker_costs_the_same_frame_over_ten_rows_and_over_a_hundred_thousand() {
        fn cost(items: &[&str]) -> (u32, u32) {
            let mut driver = headless(30, 12);
            let mut tally = ink::Tally::default();
            let mut st = picker::PickerState::default();
            driver.frame(|cx| {
                picker::picker_into(
                    &mut tally,
                    cx,
                    Rect::new(0, 0, 30, 12),
                    items,
                    &mut st,
                    &Default::default(),
                );
            });
            (tally.verbs, tally.cells)
        }

        // **Both fill the viewport.** Ten rows in a twelve-row window is a different *screen* —
        // two of its rows are the empty tail, which is one verb each instead of two — and a gate
        // that compared those two would be measuring the tail rather than the data volume.
        let hundred: Vec<&str> = (0..100).map(|_| "host").collect();
        let many: Vec<&str> = (0..100_000).map(|_| "host").collect();
        assert_eq!(cost(&hundred), cost(&many));
    }

    /// **Two call sites are two widgets, and one call site is one.**
    ///
    /// `Ctx::id` is `Location::caller()`, and `#[track_caller]` carries it through the component's
    /// three spellings. The loop is the trap: without `Ctx::with_key` both rows below would be one
    /// widget, sharing a hover, a press and a focus, and the screen would look perfect.
    #[test]
    fn a_loop_needs_a_key_to_mint_two_ids() {
        let mut driver = headless(20, 4);
        let mut same = Vec::new();
        let mut keyed = Vec::new();
        driver.frame(|cx: &mut Ctx<'_, '_>| {
            for i in 0..2 {
                let id = cx.id();
                same.push(id);
                let _ = cx.interact(id, Rect::new(0, i, 20, 1), Interest::NONE);
            }
            for i in 0..2u64 {
                keyed.push(cx.with_key(i, |cx| cx.id()));
            }
        });
        assert_eq!(same[0], same[1], "one source line mints one id");
        assert_ne!(keyed[0], keyed[1], "a key is what separates them");
    }
}
