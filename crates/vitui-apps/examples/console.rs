//! **`console` — a menu bar, two `select`s, a command palette and four ways out of a popup.**
//!
//! The overlay family's application, and the only consumer of
//! [`vitui_components::input::select`] and [`vitui_components::overlay::overlay`] that is not a gate.
//!
//! ```text
//! cargo run -p vitui-apps --example console
//! ```
//!
//! # What it is for
//!
//! The headline is that **the whole overlay family is three kinds on two axes over one two-phase
//! protocol**, and an application is where that stops being a sentence: everything that opens on this
//! screen — the two dropdowns, the menu, its submenu, the command palette — is one shell and one
//! collection, differing in a [`Kind`] and a [`Placement`].
//!
//! The status bar prints what the overlay gates are *about*, live: how many layers the census is keeping
//! alive, what the open popup was granted, whether its gutter needs a bar, whether its blur position
//! can see the pointer, and how many requests were merged. **Resize the terminal short** and watch
//! the granted height fall and the bar appear — that is the short-screen case, and the row that would
//! be unreachable if the size came from the content instead of the room.
//!
//! # Four ways out, and the third one is the one to try
//!
//! `Esc` and choosing a row are the body's, and they come home through the inbox on the frame after.
//! **Clicking the widget again** stops the owner requesting, and the layer is gone because the census
//! is over the request — nothing dismissed it. And **`Tab` away with the pointer parked on the
//! popup**: it stays, because the blur is qualified by a position. Tab away with the pointer
//! somewhere else and it goes.
//!
//! # The modal is three mechanisms and you can feel all three
//!
//! `Ctrl+P` opens the command palette and `Ctrl+P` closes it. `Tab` cannot leave it — that is the
//! trap. Clicking anything behind it does nothing — that is the barrier, and it is a different verb.
//! And the dimming is the engine's operator layer rather than a fill, which is why the frame it is up
//! costs nothing extra.
//!
//! # `Ctrl+Z` gives the terminal back to the shell, and `fg` takes it again
//!
//! The suspend forward's application, and the gesture is a demonstration of a seam rather
//! than a feature of a console: `Driver::suspend` writes the epilogue and gives the terminal back,
//! `Driver::resume` renegotiates and repaints every cell. **Neither was reachable above the engine
//! until that issue** — `Driver` owns its `Screen` privately, which is the wall issues 23
//! (`wait`) and 30 (`permit_slow`) hit before it.
//!
//! **`Ctrl+Z` is a key here and not a signal.** Raw mode is `cfmakeraw`, which clears `ISIG`, so
//! the terminal never turns it into a `SIGTSTP` — an application that wants the shell's `Ctrl+Z`
//! has to send the signal itself, and *how* is the application's own dependency policy rather than
//! the runtime's. This one shells out to `kill(1)`, which costs nothing and works everywhere; see
//! `hand_over` for why it is `-TSTP` and not `-STOP`. `fg` brings it back and the screen is
//! repainted in full.
//!
//! **There is no `$EDITOR` key beside it, and the absence is the finding.** The issue names two
//! things an application could not do; only stopping is reachable even with both verbs forwarded,
//! because `Screen::suspend` does not stop the **reader**. A `Ctrl+E` arm was written, reviewed and
//! removed: two threads blocked on one tty means the kernel gives each byte to whichever it
//! schedules, so the user loses about half of every keystroke inside the editor and vitui replays
//! the ones it stole after the resume. `SIGTSTP` is what makes the other case work — it stops every
//! thread in the process, the reader included.
//!
//! # `Esc` closes the palette, and this file is why it does
//!
//! **It did not, and finding that here is what opened the question.** A
//! `collection` read `Esc` as `Gesture::Nothing` — *clear the selection* — and consumed it whatever
//! the selection was, so the key died inside the modal and never reached `Driver::unhandled`. The
//! palette had to close on `Ctrl+P` both ways, because a chord is what `nav::step` and `from_key`
//! both refuse by rule and was the only thing that could get out of the trap.
//!
//! It resolved by **narrowing the claim rather than publishing the hook**: a collection owns
//! `Esc` exactly when `apply` would clear something, and declines it otherwise. `select` still closes
//! its own popup through `collect::collection_shaped`'s first refusal, and that hook stays
//! crate-private for the reason its own documentation gives — *a public one would invite an
//! application to spell a keyboard for a collection it did not write.* What changed is that an
//! application no longer needs it to get the behaviour everybody expects.
//!
//! So the palette now closes on `Esc`, in the same `take_unhandled` window as everything else, and
//! `Ctrl+P` still closes it too. **With a row selected the first `Esc` clears the selection and the
//! second closes the palette**, which is the two-stage dismissal every file manager has and is a
//! consequence of the rule rather than a special case in this file.
//!
//! `Esc` with nothing open **does** quit, and that took a fix in the component: a shut `select` used
//! to consume `Esc` too, so an application whose quit key is `Esc` had none — with nothing on screen
//! to say so.
//!
//! # `q` cannot be the quit key here either
//!
//! A focused `collection` consumes every text-bearing key into its type-ahead buffer, and an
//! open popup is a focused collection. `ledger`, `explorer` and `compose` hit the same wall; this one
//! binds `Ctrl+Q` and `Esc`-when-nothing-is-open, and there is no `q` to bind beside them because the
//! popup's own type-ahead is a thing to try.
//!
//! # The application's keys are read after the frame, and there is no third place
//!
//! `Driver::unhandled` is *a window onto the same queue, valid until the next frame begins*, so it is
//! read immediately after this application's own frame and never before it.

use std::process::Command;
use vitui_components::collect::{CollOpts, CollState, collection};
use vitui_components::frame::face_paint;
use vitui_components::input::{SelectOpts, SelectState, select_with};
use vitui_components::order::Rows;
use vitui_components::overlay::{Kind, PopupState, ShellOpts, gutter, overlay_with, popup_size};
use vitui_components::structure::{PanelOpts, panel_with};
use vitui_components::text::{TextOpts, text_with};
use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{Code, Pressed};
use vitui_runtime::layout::rect;
use vitui_runtime::overlay::{OverlayOpts, Placement, Z};
use vitui_runtime::work::Wake;
use vitui_runtime::{Ctx, Id, Interest, Mods, Rect, Role, Themes};

/// What the left `select` chooses between.
static SORTS: [&str; 8] = [
    "name",
    "date modified",
    "date created",
    "size",
    "kind",
    "tags",
    "date last opened",
    "date added",
];

/// What the right `select` chooses between. Deliberately longer than any screen this app is run on,
/// so the popup's own gutter is a thing you can see.
static VIEWS: [&str; 14] = [
    "as icons",
    "as list",
    "as columns",
    "as gallery",
    "grouped by kind",
    "grouped by tag",
    "grouped by date",
    "sorted ascending",
    "sorted descending",
    "show hidden",
    "show extensions",
    "show path bar",
    "show status bar",
    "show preview",
];

/// The command palette's rows.
static COMMANDS: [&str; 9] = [
    "reveal in finder",
    "duplicate",
    "compress",
    "rename",
    "move to trash",
    "get info",
    "open with...",
    "share",
    "quick look",
];

/// **The host the palette's layer is owned by** — Axis B, as one line of an application.
///
/// A dialog's owner must be guaranteed to be drawing: owned by the row that opened it, the census
/// keeps it alive exactly as long as that row is being drawn, which for a menu row is three frames of
/// eight. This id is claimed by the root panel on every frame.
const HOST: Id = Id::named("console.host");

/// **Give this process's terminal back to the shell, and take it again when continued.**
///
/// The suspend forward's application. `Driver::suspend` writes the epilogue and hands the
/// terminal over, `Driver::resume` renegotiates and repaints every cell, and until that issue
/// neither was reachable from here — `Driver` owns its `Screen` privately, which is the wall issues
/// 23 (`wait`) and 30 (`permit_slow`) hit before it.
///
/// # Why this stops the process, and why there is no `$EDITOR` key beside it
///
/// The issue names two things an application could not do, and only one of them is reachable even
/// with the pair forwarded. **`Screen::suspend` does not stop the reader.** The `vitui-pty` thread
/// is an unconditional `loop { stdin.read(..) }`, and safe Rust cannot cancel a blocking `read`, so
/// it is still parked on this terminal for the whole of a suspension — which is why `Screen::resume`
/// throws away what arrived rather than delivering it, and why the engine's own documentation says
/// **a child that needs the keyboard needs its own standard input, or this process needs to be
/// stopped while it runs.**
///
/// A `$EDITOR` launched from here would be the first case: two readers blocked on one tty, the
/// kernel giving each byte to whichever it schedules, and the user losing roughly half of every
/// keystroke. It was written and removed rather than never tried — the review is in the ticket.
/// **Stopping is the second case**, and it is the one that works, because `SIGTSTP` stops every
/// thread in the process including the reader.
///
/// # `-TSTP` and not `-STOP`, which is the one decision in this function
///
/// How an application stops itself is the *application's* dependency policy: `vitui-runtime` depends
/// on nothing and is not acquiring `libc` to spell one signal. The three options are `libc::raise`,
/// a crate that wraps it, and this — `kill(1)`, which every POSIX system has and which costs no
/// dependency at all.
///
/// `SIGSTOP` cannot be caught or ignored by anything, the shell's job control included, so a process
/// stopped with it is one the shell was never told about: no *Suspended* line and no `fg`. `SIGTSTP`
/// is the signal `Ctrl+Z` would have sent if raw mode had not cleared `ISIG`, so the shell reports
/// it and `fg` brings the application back.
///
/// **The resume is not the line it looks like.** `kill` exits immediately and the signal lands on
/// this process a moment later, so the `resume` below does not run until the shell continues us. The
/// terminal is given back *before* the signal goes out, because the epilogue has to be written while
/// this process still has somewhere to write it.
///
/// A failure is reported rather than swallowed: on a system with no `kill(1)` the suspend would
/// otherwise look like an instantaneous full repaint and say nothing.
fn hand_over(driver: &mut Driver) -> Option<String> {
    driver.suspend();
    let outcome = match Command::new("kill")
        .args(["-TSTP", &std::process::id().to_string()])
        .status()
    {
        Ok(status) if status.success() => None,
        Ok(status) => Some(format!("kill -TSTP exited {status}")),
        Err(why) => Some(format!("could not run kill: {why}")),
    };
    driver.resume();
    outcome
}

/// Everything this application knows.
struct App {
    /// The left `select`'s owner state.
    sort: SelectState,
    /// Its popup's.
    sort_popup: PopupState,
    /// The right `select`'s owner state.
    view: SelectState,
    /// Its popup's.
    view_popup: PopupState,
    /// Whether the command palette is up. The owner's half of a modal.
    palette: bool,
    /// The palette's list.
    commands: CollState,
    /// What the palette last answered, printed until something else happens.
    ran: Option<&'static str>,
    /// **The palette's inbox.** The body writes it and `ui` takes it at the top of the frame after.
    ///
    /// Not decoration: the body is held in the queue until the satisfy pass, so a local the body
    /// wrote cannot be read again in the same frame. Written as a local the diagnostic is `E0503:
    /// cannot use answer because it was mutably borrowed`, pointing at this function's own read and
    /// never mentioning the overlay — the trap, met while writing the application for it.
    pending: Option<&'static str>,
    /// Set inside `take_unhandled`, read by the loop.
    exit: bool,
    /// **Set inside `take_unhandled`, acted on by the loop, and it has to be both.**
    ///
    /// A suspend takes `&mut Driver` and `take_unhandled` has only the keys — which is the same
    /// separation `exit` is on, and for a sharper reason here: a `Driver` reachable from inside the
    /// draw is a `suspend` reachable from inside a frame, and the terminal cannot be given away
    /// while a frame is being composited into it.
    handoff: bool,
    /// What the last handoff went wrong with, printed until something else happens. `None` on the
    /// path that works, which is every path on a machine with a `kill(1)`.
    handoff_failed: Option<String>,
    /// What the status bar last saw, so the bar reports the frame rather than the state.
    seen: Seen,
}

/// What the status bar prints, gathered during the draw.
#[derive(Clone, Copy, Default)]
struct Seen {
    /// Whether a popup is standing **this** frame. The one field that is not one frame behind.
    standing: bool,
    layers: usize,
    merged: u32,
    regions: usize,
    stops: usize,
    traps: usize,
    granted: (u16, u16),
    bar: bool,
    over: bool,
    room: (u16, u16),
    wants: (u16, u16),
}

impl App {
    /// One frame, top to bottom.
    fn ui<'f>(&'f mut self, cx: &mut Ctx<'f, '_>) {
        // **The host claims its id first.** Axis B: a dialog owned by something that is not
        // guaranteed to draw lives exactly as many frames as its owner does.
        let whole = cx.area();
        let _ = cx.interact(HOST, whole, Interest::HOVER);
        let block = panel_with(
            cx,
            whole,
            " console — one shell, one collection, three kinds ",
            "",
            &PanelOpts::default(),
        );
        let interior = block.interior;
        if interior.h < 6 || interior.w < 44 {
            return;
        }
        let room = cx.size();

        let (bar, rest) = rect::split_at_v(interior, 1);
        let (body, status) = rect::split_at_v(rest, rest.h.saturating_sub(3));
        let (left, right) = rect::split_at_h(bar, bar.w / 2);
        let (left, _) = rect::split_at_h(left, 24.min(left.w));
        let (right, _) = rect::split_at_h(right, 24.min(right.w));

        // Destructured, because `select` takes `&'f mut` of two of these and the rest are read
        // afterwards. One line of an application is what *two structs, one writer each* costs.
        let App {
            sort,
            sort_popup,
            view,
            view_popup,
            palette,
            commands,
            ran,
            pending,
            seen,
            handoff_failed,
            ..
        } = self;

        // **The inbox, taken at the top.** The body wrote it on the frame before and this is the only
        // place it is read, which is the same arrangement `select` uses for its own popup.
        if let Some(name) = pending.take() {
            *ran = Some(name);
        }

        // **Read the popups before handing them over, never after.** `select` takes `&'f mut` of
        // one, so the borrow lives for the rest of the frame and any later read is `E0502` at this
        // call site — with nothing in the message about overlays. The granted size is last frame's,
        // which is what it is for.
        let open_granted = if sort.is_open() {
            Some((sort_popup.granted(), sort_popup.over(), SORTS.len()))
        } else if view.is_open() {
            Some((view_popup.granted(), view_popup.over(), VIEWS.len()))
        } else {
            None
        };

        let opts = SelectOpts {
            placement: Placement::BELOW,
            ..SelectOpts::default()
        };
        let sort_resp = select_with(cx, left, sort, sort_popup, &SORTS, &opts);
        let view_resp = select_with(cx, right, view, view_popup, &VIEWS, &opts);

        // **Nothing holds the focus until an application says so**.
        // `focused().is_none()` and never `!is_focused(id)`, which would drag the keyboard back every
        // frame the user had tabbed away.
        if cx.focused().is_none() {
            cx.focus(sort_resp.id);
        }

        // The body: what is chosen, and the four ways out written where a person can read them.
        let lines = [
            format!(
                "sort: {}    view: {}",
                SORTS[sort.chosen().min(SORTS.len() - 1)],
                VIEWS[view.chosen().min(VIEWS.len() - 1)],
            ),
            String::new(),
            "Enter / Space / Down opens the focused select.  Tab moves.".to_string(),
            "In a popup: arrows move, type to seek, Enter chooses, Esc cancels.".to_string(),
            "Click the widget again and nothing dismissed it — the owner stopped asking."
                .to_string(),
            "Tab away with the pointer ON the popup and it stays: the blur wants a position."
                .to_string(),
            String::new(),
            format!(
                "Ctrl+P opens and closes the palette. Tab cannot leave it; clicks behind it do \
                 nothing.{}",
                ran.map(|r| format!("   last: {r}")).unwrap_or_default()
            ),
        ];
        for (i, text) in lines.iter().enumerate() {
            if i32::try_from(i).unwrap_or(i32::MAX) >= i32::from(body.h) {
                break;
            }
            let row = Rect::new(body.x, body.y + i as i32, body.w, 1);
            text_with(
                cx,
                row,
                text,
                &TextOpts {
                    role: if i == 0 { Role::Title } else { Role::Dim },
                    ..Default::default()
                },
            );
        }

        // **The palette: a modal, and modality is one `bool` on the request.** The owner is `HOST`
        // and not the widget that opened it, which is Axis B.
        if *palette {
            let theme = *cx.theme();
            let anchor = Rect::new(i32::from(room.0) / 2, 2, 1, 1);
            // **The header is a row of the popup and the sizing function does not know that.**
            // `popup_size` sizes a *list*; this body spends one row of what it is granted on a title,
            // so the popup has to ask for one more or the last command is off the bottom — which is
            // The short-screen case arriving as an off-by-one in an application. The `rows`
            // handed to the shell counts the same way, so the gutter's decision is about the same
            // rectangle.
            let list = popup_size(&COMMANDS, (44.min(room.0), room.1.saturating_sub(5)));
            let size = (list.0, list.1.saturating_add(1));
            cx.overlay(
                HOST,
                anchor,
                OverlayOpts {
                    z: Z::MODAL,
                    ..OverlayOpts::modal(size.0.max(40), size.1, &theme)
                },
                |cx| {
                    let area = cx.area();
                    let shell = ShellOpts {
                        kind: Kind::Dialog,
                        ..ShellOpts::default()
                    };
                    let rows = u32::try_from(COMMANDS.len() + 1).unwrap_or(0);
                    let _ = overlay_with(cx, area, rows, &shell, &mut |cx, r| {
                        let (head, list) = rect::split_at_v(r, 1);
                        text_with(
                            cx,
                            head,
                            " run a command — Ctrl+P closes, Enter runs ",
                            &TextOpts {
                                role: Role::Title,
                                ..Default::default()
                            },
                        );
                        let opts = CollOpts::default();
                        let resp = collection(
                            cx,
                            list,
                            commands,
                            &opts,
                            Rows::of(COMMANDS.len()),
                            &mut |buf, range: std::ops::Range<usize>| {
                                range.into_iter().find(|&i| COMMANDS[i].starts_with(buf))
                            },
                            &mut |cx, cell, i, face| {
                                let paint = face_paint(cx.theme(), face);
                                cx.text(cell.x + 1, cell.y, COMMANDS[i], paint);
                            },
                        );
                        if resp.clicked {
                            // **Through the inbox**, because there is nowhere else for it to go.
                            *pending = Some(COMMANDS[commands.sel.lead.min(COMMANDS.len() - 1)]);
                        }
                    });
                },
            );
        }

        // **The open flag is read *after* the components and the granted size before them**, and the
        // split is forced rather than chosen: `select` takes `&'f mut` of the popup's state, so
        // `granted()` is unreadable from here down, while `SelectState` is only `&mut` and stays
        // readable. So the size on the bar is **last frame's** — which is exactly what a body reports
        // — and the open flag is this frame's.
        let standing = if sort.is_open() {
            Some(SORTS.len())
        } else if view.is_open() {
            Some(VIEWS.len())
        } else {
            None
        };
        let wants = match standing {
            Some(len) if len == SORTS.len() => popup_size(&SORTS, room),
            Some(_) => popup_size(&VIEWS, room),
            None => (0, 0),
        };
        // **One more frame, and only while the bar is behind.** A popup opened by a keystroke is
        // requested *inside* the same frame the bar was drawn in, so on that frame the body has not
        // run and `granted()` is still `(0, 0)`. Asking for one more frame is what makes the bar
        // catch up; the condition is what stops it being a wake loop — the frame after has a real
        // size and asks for nothing.
        if standing.is_some() && open_granted.is_none() {
            cx.request_frame();
        }
        // **The frame's own counters are not readable from inside it** — `Driver::inspect` and
        // `Driver::layers_live` are the driver's — so the loop gathers those *after* the frame and
        // this fills in the half the state knows. The bar therefore reports the frame before, and
        // says so in as many words.
        *seen = Seen {
            granted: open_granted.map_or((0, 0), |(g, _, _)| g),
            bar: open_granted
                .is_some_and(|(g, _, len)| gutter(g, u32::try_from(len).unwrap_or(0)).bar),
            standing: standing.is_some(),
            over: open_granted.is_some_and(|(_, over, _)| over),
            room,
            wants,
            ..*seen
        };
        let _ = view_resp;
        // **`*seen` and not `self`**: `self.sort_popup` is borrowed for the rest of the frame, so
        // `self` is not readable again from here — E0502, on a method call three lines from a
        // popup nobody was thinking about. The status bar takes the value it needs.
        status_rows(cx, status, *seen, handoff_failed.as_deref());
    }
}

/// The three status rows. A report of the frame that has just been drawn.
///
/// A free function taking a [`Seen`] by value, because `self` is not readable after a `select` has
/// been handed its popup — see the call site.
fn status_rows(cx: &mut Ctx<'_, '_>, at: Rect, s: Seen, handoff_failed: Option<&str>) {
    {
        let dim = TextOpts {
            role: Role::Dim,
            ..Default::default()
        };
        let (first, rest) = rect::split_at_v(at, 1);
        let (second, third) = rect::split_at_v(rest, 1);
        text_with(
            cx,
            first,
            &format!(
                "last frame: layers {}  merged {}  regions {}  stops {}  traps {}",
                s.layers, s.merged, s.regions, s.stops, s.traps
            ),
            &dim,
        );
        text_with(
            cx,
            second,
            &format!(
                "popup {}  granted {}x{} (last frame)  wants {}x{}  screen {}x{}  bar {}  \
                 pointer over it {}",
                if s.standing { "open " } else { "shut " },
                s.granted.0,
                s.granted.1,
                s.wants.0,
                s.wants.1,
                s.room.0,
                s.room.1,
                if s.bar { "yes" } else { "no" },
                if s.over { "yes" } else { "no" },
            ),
            &TextOpts {
                role: if s.bar { Role::Warn } else { Role::Dim },
                ..Default::default()
            },
        );
        text_with(
            cx,
            third,
            // **The new key leads**, because all three sections of this bar overflow their
            // third and clip — what a person reads is the prefix, and the key worth discovering is
            // the one this application was extended to demonstrate. A failed handoff takes the
            // whole section, because it is the only place this application can say so.
            &match handoff_failed {
                Some(why) => format!("Ctrl+Z could not suspend: {why}"),
                None => "Ctrl+Z to the shell (fg to return) · Ctrl+P palette · Ctrl+Q quits · \
                         Esc quits when nothing is open"
                    .to_string(),
            },
            &dim,
        );
    }
}

impl App {
    /// **What the frame turned out to be, read after it.**
    ///
    /// `Driver::inspect` and `Driver::layers_live` are the driver's and not the `Ctx`'s, so these five
    /// cannot be gathered inside the draw at all. The status bar is therefore one frame behind on
    /// them, which is the honest arrangement rather than a compromise: a count read *during* the
    /// frame it describes would be a count taken before the overlay pass ran.
    fn observe(&mut self, driver: &Driver) {
        let layers = driver.layers_live();
        let frame = driver.inspect();
        self.seen = Seen {
            layers,
            merged: frame.overlays_merged(),
            regions: frame.hits().len(),
            stops: frame.stop_count(),
            traps: frame.trap_scopes().count(),
            ..self.seen
        };
    }

    /// **The application's own keys, read from what nothing wanted.** See this file's header.
    fn take_unhandled(&mut self, keys: &[Pressed]) {
        for key in keys {
            let chord = key.mods.chord();
            match key.code {
                // **`Ctrl+P` both ways**, which was once the only way out of the palette: a
                // `collection` claimed `Esc` as `Gesture::Nothing` whatever the selection was, and
                // the trap has nobody to hand a declined key to. A chord is what `nav::step` and
                // `from_key` both refuse by rule, so it reaches this window from inside the modal.
                // It is kept because it is a good binding, not because `Esc` still cannot — see
                // this file's header.
                Code::Char('p') if chord.contains(Mods::CTRL) => self.palette = !self.palette,
                Code::Char('q') if chord.contains(Mods::CTRL) => self.exit = true,
                // **The one the runtime could not reach for a long time.** `Ctrl+Z` arrives here as
                // an ordinary key and not as a signal, because raw mode is `cfmakeraw` and that
                // clears `ISIG` — so an application that wants the shell's `Ctrl+Z` has to spell
                // it, and could not.
                Code::Char('z') if chord.contains(Mods::CTRL) => self.handoff = true,
                // **`Esc` closes the palette, and it is the same key that quits.** It reaches here
                // only when nothing took it first — a *shut* `select` declines it, an open popup's
                // own body consumes it as *cancel*, and since `Escape` was narrowed the palette's
                // own `collection` declines it whenever it has no selection to clear. So with a row
                // selected the first press clears the selection and the second lands here, which is
                // the two-stage dismissal and not a missed key.
                //
                // It is safe to quit on only because the component was fixed twice: a shut `select`
                // used to eat `Esc`, which left this application with no quit key at all.
                Code::Escape if self.palette => self.palette = false,
                Code::Escape => self.exit = true,
                _ => {}
            }
        }
    }
}

fn main() {
    let mut app = App {
        sort: SelectState::new(),
        sort_popup: PopupState::new(),
        view: SelectState::at(1),
        view_popup: PopupState::new(),
        palette: false,
        commands: CollState::new(),
        ran: None,
        pending: None,
        exit: false,
        handoff: false,
        handoff_failed: None,
        seen: Seen::default(),
    };

    // **The picture arm**: one screen, drawn headlessly and written as SVG. A
    // program nobody outside this machine can see is a program nobody believes in.
    if vitui_apps::pictures::wanted() {
        let mut driver = vitui_apps::pictures::driver(100, 30, *Themes::standard().theme());
        for _ in 0..vitui_apps::pictures::FRAMES {
            driver.frame(|cx| app.ui(cx));
        }
        vitui_apps::pictures::write("console", &driver);
        return;
    }

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

    loop {
        driver.frame(|cx| app.ui(cx));
        app.observe(&driver);
        // **Read immediately after this application's own frame and never before it.** The window is
        // valid until the next frame begins, so a loop that read it first acted on the previous
        // frame's window — one wake late, which for a single keystroke means never.
        let unhandled: Vec<Pressed> = driver.unhandled().to_vec();
        app.take_unhandled(&unhandled);
        if app.exit {
            break;
        }
        // **After the frame and before the park.** The screen the user is leaving behind is the one
        // that has just been painted, and `resume` owes a full repaint that the `continue` below
        // spends — so the terminal comes back carrying this application rather than the editor's
        // last screen.
        if std::mem::take(&mut app.handoff) {
            app.handoff_failed = hand_over(&mut driver);
            continue;
        }
        // **A key this application acted on owes a frame**, and `continue` rather than `wait` is how
        // every loop in this crate spends it: `take_unhandled` runs *after* the draw, so `Ctrl+P`
        // toggles a flag the screen has already been painted without. Parking here leaves the palette
        // invisible until the next keystroke — the same shape as reading the window one frame early,
        // and it is why `ledger` carries this line too.
        if !unhandled.is_empty() {
            continue;
        }
        match driver.wait() {
            Wake::Quit => break,
            Wake::Input | Wake::Posted | Wake::Deadline => {}
        }
    }
}
