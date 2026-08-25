//! **The accordion, and the fold set: the two scenes no golden-cell gate can see.**
//!
//! Components ticket 21. Spec §8, §21. Two rows of §21's table stand here — row 11, *an accordion of
//! twelve sections, 0 / 6 / 12 open*, and row 10, *a 200 000-line document with 4 167 folds and an
//! insert above them* — and they are one module because they are one component's. §8's whole claim
//! is that **a section of an accordion and a closed fold are the same machine**; the file is named
//! for the half that draws.
//!
//! | scene | what it decides |
//! |---|---|
//! | an accordion of twelve sections, 0 / 6 / 12 open | closed content is not drawn: **478 hit entries against 70**, on **identical surfaces** |
//! | a 200 000-line document with 4 167 folds and an insert above them | the anchor: **4 166 of 4 167** wrong, reanchored **0 of 4 167** |
//!
//! # Why this scene asserts on the hit index and not on the picture
//!
//! [`Ctx::interact`] appends a hit entry and a ring entry **before** it looks at the rectangle. So a
//! body drawn into an `h = 0` rectangle declares every entry it would have declared open, every one
//! of its writes is rejected by the clip, and **the two surfaces are identical** — 0 cells over 0
//! rows, at 300x80, asserted by [`surfaces_are_identical`]. Every gate this crate owns that reads a
//! *surface* scores the defect clean, and so does every counter that reads what **landed**:
//! [`counters_that_separate_them`] walks §20's nine and returns the ones that move.
//!
//! **It returns three for one spelling of the defect and two for the one beside it**, and the
//! difference is this module's own finding. A body called at `h = 0` that *draws* every row moves
//! `verbs` as well, so a gate on the verb count would catch it; a body that declares every row and
//! paints only the admitted ones ([`Body::DeclaresOnly`]) makes **exactly the drawing calls the
//! correct one makes** — same verbs, same writes, same asked, same surface — and still carries 408
//! entries nobody can reach. Only `regions` and `tab stops` see that one, which is why the scene is
//! written on the hit index and the ring rather than on the cheapest counter that happened to work.
//!
//! That is the same shape [`crate::listing`] found one axis over — *writes flat 1k → 1M* is green on
//! a listing that declares a million hit entries — and it is not a coincidence. Both are the engine
//! reporting a fully clipped verb as zero columns while `Ctx::interact` clips nothing, which is the
//! runtime's own rule read from the wrong end.
//!
//! # The numbers reproduce as a **subtraction**, and the constant is fitted once
//!
//! §8's screen is a prototype this ticket does not own: `crates/proto-c14-app` on branch
//! `prototype/c14-collapsible` drew the accordion under a menu bar, a toolbar of sixteen chips and
//! four rows of status chips, none of which this crate can build — `menu_bar` is components ticket
//! 35's and `field` is ticket 24's. So §8's declaration figures are **this screen's plus one
//! constant pair**, [`CHROME_ENTRIES`] and [`CHROME_STOPS`], and the pair is fixed from **one** row
//! of §8's table and then used to predict the other five.
//!
//! `tests::the_rest_of_8s_declaration_figures_are_this_screen_plus_the_chrome_fitted_from_the_first`
//! is that prediction and it holds to the unit on all five. The stop constant is **3 against 57**
//! for one reason and it is checkable rather than convenient: the prototype's toolbar and status bar
//! each opened a `ScopeKind::Group`, and a group collapses its whole range onto one tab stop
//! ([`Frame::stop_count`]) while leaving every entry in the ring. This screen opens no group, so its
//! ring length and its stop count are equal everywhere — which is why §8's ring column and its stop
//! column need two different constants over the same screen.
//!
//! **The cell columns are not reconciled and are printed side by side instead.** §8's *23 034
//! against 56 298 cells asked for* is a screen with different furniture on it; what reproduces here
//! is the direction and the equality underneath it — `asked` grows, `writes` does not move, and the
//! surfaces are identical.
//!
//! # The fold half reproduces exactly, and it is not a screen at all
//!
//! A fold set is a `Vec<u32>` and a document is a `Vec<Line>`; neither draws, and §8 is explicit
//! that the closed folds are **caller state** for the same forced reason a collection's selection
//! is. So scene 10 is standable today in full: 200 000 lines, a block every twelve, eight lines
//! long, every fourth block closed is **4 167 folds**; ten lines inserted at line 24 leaves **4 166
//! of 4 167** on a line that opens no block, and [`Anchor::Shifted`] leaves **0 of 4 167**. Both
//! numbers are §8's own and both are asserted rather than reported.
//!
//! What is *not* here is `collapsible`. The fold set is the input the component reads; the index it
//! builds, the splice, the tween and the two-state machine are components ticket 22's, and until
//! that ticket lands both scenes are [`Standing::Red`] waiting for their subject — [`standing`],
//! [`owed_message`] and [`assert_stands_up`], inherited from [`crate::listing`] whole because the
//! distinction is the same one: *a scene that fails because it is unimplemented and a scene that
//! fails because the code is wrong are the same failure unless the message separates them.*
//!
//! [`Ctx::interact`]: vitui_runtime::Ctx::interact
//! [`Frame::stop_count`]: vitui_runtime::ctx::Frame::stop_count
//! [`Standing::Red`]: crate::gates::Standing::Red

use std::time::{Duration, Instant};

use vitui_runtime::ctx::Driver;
use vitui_runtime::{Ctx, Density, Id, Interest, Rect, Role, Scrollable};

use crate::counters::{Allocations, Counter, Counters, Tally};
use crate::ink::{Direct, Ink};
use crate::input::{ButtonOpts, button_into};
use crate::obligations::Verdict;
use crate::runner::{Canvas, Diff, Pen};
use crate::text::{ChipOpts, TextOpts, chip_into, text_into};

// ── the screen ───────────────────────────────────────────────────────────────────────────────────

/// The screen's width. **Three hundred**, which is §8's own.
pub const W: u16 = 300;
/// The screen's height. **Eighty**, which is §8's own.
pub const H: u16 = 80;

/// **The viewport §8's accordion actually had, which is not [`H`].**
///
/// The prototype screen spent one row on a menu bar, one on a toolbar and four on a status area, so
/// the accordion was handed **seventy-four**. It matters for exactly one of §8's rows — *twelve
/// open* — where the content is 132 rows and the difference between a 74-row window and an 80-row
/// one is whether an eighth section is reached at all.
pub const PROTO_VIEWPORT: u16 = 74;

/// Twelve sections, which is §8's accordion.
pub const SECTIONS: usize = 12;

/// How tall an open section's body is. **Ten**, chosen by the prototype so that six open sections
/// overflow a 74-row viewport and five do not — the residue a collapse leaves is reachable only when
/// the content is shorter than the rectangle, which is why no list and no table on this map met it.
pub const BODY_ROWS: u16 = 10;

/// How many rows of chips a body carries.
pub const CHIP_ROWS: u16 = 8;
/// How many columns of chips a body carries.
pub const CHIP_COLS: u16 = 4;
/// How many one-row buttons sit under the chips.
///
/// §8's screen had two **fields** here. `field` is components ticket 24's and does not exist, so the
/// two rows are [`crate::input::button`]s: the count depends only on each declaring one hit entry
/// and one ring entry, which both do, and the substitution is on **both** arms of every pair below.
pub const BUTTONS: u16 = 2;

/// **What one open body declares. Thirty-four**, and the number is the whole scene: `8 x 4 + 2`.
pub const PER_BODY: usize = (CHIP_ROWS * CHIP_COLS + BUTTONS) as usize;

/// Rows an open section occupies: its header and its body.
pub const SECTION_ROWS: u16 = 1 + BODY_ROWS;

/// The titles, so that a header is a row of text rather than a rectangle.
pub const TITLES: [&str; SECTIONS] = [
    "General",
    "Appearance",
    "Editor",
    "Keyboard",
    "Terminal",
    "Extensions",
    "Version control",
    "Search",
    "Diagnostics",
    "Telemetry",
    "Experimental",
    "About",
];

/// **Hit entries a correct frame declares with all twelve sections closed. Thirteen.**
///
/// One for the accordion's scrollable region and one for each header. A closed section's body is not
/// called, so it declares nothing at all — which is the rule §8 states in one line: *a body is
/// handed a rectangle and draws inside it.*
pub const CLOSED_ENTRIES: usize = 1 + SECTIONS;

/// **Tab stops a correct frame declares with all twelve sections closed. Twelve.**
///
/// The headers. The accordion's own region asks for [`Interest::NONE`] beside the scroll interest
/// `Ctx::scrollable` folds in, so it is a hit entry and not a stop.
pub const CLOSED_STOPS: usize = SECTIONS;

/// **What the `h = 0` spelling adds. Four hundred and eight**, and it is `12 x 34`.
///
/// The same number on both columns, which is not a coincidence and is asserted as an equality:
/// `Ctx::interact` appends to the hit index and to the ring in one call, before either looks at the
/// rectangle. §8's own pair is `478 - 70` and `423 - 15`, and both are 408.
pub const UNCULLED_EXCESS: usize = SECTIONS * PER_BODY;

/// **The chrome §8's screen carried and this crate cannot build, as ring entries.**
///
/// Fitted from one row of §8's table — *twelve sections closed: 70 hit entries* against this
/// screen's [`CLOSED_ENTRIES`] — and then used to predict the other five. See this module's header.
pub const CHROME_ENTRIES: usize = 57;

/// **The same chrome as tab stops. Three, not fifty-seven**, because the prototype's menu bar,
/// toolbar and status bar each stood inside a `ScopeKind::Group` and a group collapses its whole
/// range onto one stop.
pub const CHROME_STOPS: usize = 3;

/// §8's hit entries with twelve sections closed, the body skipped.
pub const SPEC_CLOSED_ENTRIES: usize = 70;
/// §8's tab stops with twelve sections closed, the body skipped.
pub const SPEC_CLOSED_STOPS: usize = 15;
/// §8's hit entries with twelve sections closed and the body drawn at `h = 0`.
pub const SPEC_ZERO_ENTRIES: usize = 478;
/// §8's tab stops with twelve sections closed and the body drawn at `h = 0`.
pub const SPEC_ZERO_STOPS: usize = 423;
/// §8's hit entries with six sections open.
pub const SPEC_SIX_ENTRIES: usize = 274;
/// §8's tab stops with six sections open.
pub const SPEC_SIX_STOPS: usize = 219;
/// §8's hit entries with twelve sections open.
pub const SPEC_TWELVE_ENTRIES: usize = 303;
/// §8's tab stops with twelve sections open.
pub const SPEC_TWELVE_STOPS: usize = 248;
/// §8's cells asked for with the body skipped.
pub const SPEC_CLOSED_CELLS: u64 = 23_034;
/// §8's cells asked for with the body drawn at `h = 0`.
pub const SPEC_ZERO_CELLS: u64 = 56_298;
/// §8's µs for twelve sections closed. A report.
pub const SPEC_CLOSED_US: f64 = 24.38;
/// §8's µs for twelve sections closed with the body drawn at `h = 0`. A report.
pub const SPEC_ZERO_US: f64 = 49.25;
/// §8's µs for six sections open. A report.
pub const SPEC_SIX_US: f64 = 43.71;
/// §8's µs for twelve sections open. A report.
pub const SPEC_TWELVE_US: f64 = 45.71;

/// **The height the collapsing section is caught at. Two rows of its body left.**
///
/// §8 states the mid-transition pair as *273 ring entries against 247* and states no height beside
/// it. Two is the height at which the difference is 26, which is that pair's own subtraction, and
/// §8's two absolutes reconcile there with the same [`CHROME_ENTRIES`] the steady rows use — which
/// is what makes two the answer rather than a fit. See
/// `tests::the_mid_transition_pair_is_this_screen_plus_the_same_chrome`.
pub const MID_HEIGHT: u16 = 2;

/// How many sections are open on the mid-transition frame, the collapsing one included.
pub const MID_OPEN: usize = 6;

/// **What a body two rows tall declares when it culls. Eight** — two rows of four chips.
pub const MID_CULLED: usize = (MID_HEIGHT * CHIP_COLS) as usize;

/// **What the mid-transition costs, and §8's own subtraction. Twenty-six.**
pub const MID_EXCESS: usize = PER_BODY - MID_CULLED;

/// §8's mid-transition ring entries, the arm that culls.
pub const SPEC_MID_CULLED: usize = 247;
/// §8's mid-transition ring entries, the arm that does not.
pub const SPEC_MID_UNCULLED: usize = 273;

// ── the two spellings, which is one field between them ───────────────────────────────────────────

/// **What a closed section does with its body.**
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Closed {
    /// **The rule.** The body is not a value, it is a closure, and a closure that is not called
    /// costs nothing at all.
    Skip,
    /// **The defect.** Call it with a zero-height rectangle and let the clip reject every write —
    /// the shape a reader writes when *collapsed* is spelled as a height rather than as a branch.
    ZeroRect,
}

impl Closed {
    /// The words a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Closed::Skip => "body skipped",
            Closed::ZeroRect => "body drawn at h = 0",
        }
    }
}

/// **Whether a body draws only the rows its rectangle admits.**
///
/// §8's rule is one line — *a body is handed a rectangle and draws inside it* — and [`Body::Uncut`]
/// is the spelling that ignores it. On a closed section the two flags compound: the body is called
/// at `h = 0` **and** draws every row, which is §8's *neither culling* pair.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Body {
    /// **The rule.** The body stops at the rectangle it was handed.
    Culls,
    /// **The defect.** Every row is drawn and every entry declared, whatever the rectangle says.
    Uncut,
    /// **The same defect with the drawing taken out**, and it is the one worth the third arm.
    ///
    /// Every row declares its region and only the admitted rows paint. The verb count, the write
    /// count, the distinct-cell count and the surface are then **identical** to the correct arm's —
    /// so of §20's nine counters only `regions` and `tab stops` move, which is §8's sentence exactly:
    /// *`interact` pushes a hit entry and a ring entry before it looks at the rectangle.*
    ///
    /// [`Body::Uncut`] is separable by `verbs` as well, and that is a fact about `Uncut` rather than
    /// about the defect — see [`counters_that_separate_them`].
    DeclaresOnly,
}

impl Body {
    /// The words a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Body::Culls => "culls",
            Body::Uncut => "does not cull",
            Body::DeclaresOnly => "declares every row, paints the admitted ones",
        }
    }
}

/// **One frame of the accordion, as a value.**
///
/// Every arm of every pair below is one of these, so a reviewer's diff between a correct build and a
/// defective one is a field rather than a function — components ticket 06's `title_split`, one
/// ticket on.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Screen {
    /// How many of the twelve sections are open.
    pub open: usize,
    /// **The last open section's body height, when it is mid-collapse.** `None` is a steady frame.
    pub collapsing: Option<u16>,
    /// What a closed section does with its body.
    pub closed: Closed,
    /// Whether a body draws inside the rectangle it was handed.
    pub body: Body,
    /// The viewport the accordion is handed. [`H`] here, [`PROTO_VIEWPORT`] for §8's own screen.
    pub viewport: u16,
}

impl Screen {
    /// **The correct build with `open` sections open**, on this crate's full-height screen.
    pub const fn correct(open: usize) -> Screen {
        Screen {
            open,
            collapsing: None,
            closed: Closed::Skip,
            body: Body::Culls,
            viewport: H,
        }
    }

    /// **The `h = 0` spelling of twelve closed sections**, whose body draws every row it would have
    /// drawn open. §8's *body drawn at `h = 0`, neither culling*.
    pub const fn zero_rect() -> Screen {
        Screen {
            open: 0,
            collapsing: None,
            closed: Closed::ZeroRect,
            body: Body::Uncut,
            viewport: H,
        }
    }

    /// **The `h = 0` spelling with the drawing taken out**: every row declares its region and none
    /// of them paints. See [`Body::DeclaresOnly`] — this is the arm the verb count cannot see.
    pub const fn zero_rect_declaring() -> Screen {
        Screen {
            open: 0,
            collapsing: None,
            closed: Closed::ZeroRect,
            body: Body::DeclaresOnly,
            viewport: H,
        }
    }

    /// **The mid-transition frame**: [`MID_OPEN`] open, the last of them at [`MID_HEIGHT`].
    pub const fn collapsing(body: Body) -> Screen {
        Screen {
            open: MID_OPEN,
            collapsing: Some(MID_HEIGHT),
            closed: Closed::Skip,
            body,
            viewport: H,
        }
    }

    /// The same screen on §8's own [`PROTO_VIEWPORT`].
    pub const fn on_the_prototypes_viewport(self) -> Screen {
        Screen {
            viewport: PROTO_VIEWPORT,
            ..self
        }
    }

    /// The body height section `i` is drawn at.
    pub const fn body_height(&self, i: usize) -> u16 {
        if i >= self.open {
            return 0;
        }
        match self.collapsing {
            Some(h) => {
                if i + 1 == self.open {
                    h
                } else {
                    BODY_ROWS
                }
            }
            None => BODY_ROWS,
        }
    }

    /// **What section `i`'s body declares on this frame**, header excluded.
    ///
    /// Nothing when the closure is not called, [`PER_BODY`] when the body ignores its rectangle, and
    /// [`admitted`] of its height when it obeys it. The prediction the counts below are asserted
    /// against, so that *the entry count is a function of the rectangle* is arithmetic rather than a
    /// sentence.
    pub const fn body_entries(&self, i: usize) -> usize {
        let h = self.body_height(i);
        if h == 0 {
            match self.closed {
                // The closure is not called at all. This is the whole of *closed content is not
                // drawn*.
                Closed::Skip => return 0,
                Closed::ZeroRect => {}
            }
        }
        match self.body {
            Body::Uncut | Body::DeclaresOnly => PER_BODY,
            Body::Culls => admitted(h),
        }
    }

    /// The content's height in rows, which is what the scroll area is clamped against.
    pub const fn content_rows(&self) -> i32 {
        let mut total = 0i32;
        let mut i = 0;
        while i < SECTIONS {
            total += 1 + self.body_height(i) as i32;
            i += 1;
        }
        total
    }

    /// **Which sections the window reaches**, as `0..n`. A section entirely outside it is not drawn
    /// and therefore declares nothing, which is §8's *1.05x rather than 2x*.
    pub const fn sections_drawn(&self) -> usize {
        let mut y = 0i32;
        let mut i = 0;
        let mut drawn = 0;
        while i < SECTIONS {
            let want = 1 + self.body_height(i) as i32;
            if y < self.viewport as i32 {
                drawn += 1;
            }
            y += want;
            i += 1;
        }
        drawn
    }

    /// **What this frame declares in the hit index**, predicted from the construction.
    pub const fn predicted_entries(&self) -> usize {
        let drawn = self.sections_drawn();
        let mut total = 1; // the accordion's own scrollable region
        let mut i = 0;
        while i < drawn {
            total += 1 + self.body_entries(i);
            i += 1;
        }
        total
    }

    /// **What this frame declares in the ring**, which is [`Screen::predicted_entries`] without the
    /// accordion's own region: it asks for the wheel and not for the keyboard.
    pub const fn predicted_stops(&self) -> usize {
        self.predicted_entries() - 1
    }
}

/// **How many entries a body `h` rows tall declares when it obeys its rectangle.**
///
/// Four a chip row for the first [`CHIP_ROWS`], then one a button row. A `const fn` rather than a
/// table, because the shape — *proportional to the rectangle, not to the content* — is the property
/// the whole scene is about.
pub const fn admitted(h: u16) -> usize {
    let chips = if h > CHIP_ROWS { CHIP_ROWS } else { h };
    let below = h.saturating_sub(CHIP_ROWS);
    let buttons = if below > BUTTONS { BUTTONS } else { below };
    (chips * CHIP_COLS + buttons) as usize
}

// ── the draw ─────────────────────────────────────────────────────────────────────────────────────

/// **Draw one frame of the accordion.**
///
/// Generic over [`Ink`] so a [`Tally`], a [`Pen`] and a [`Direct`] measure the same drawing path
/// rather than a copy of it — [`crate::ink`]'s whole argument, and the reason the surface equality
/// and the entry counts below are two questions about **one** function.
///
/// The rectangle is the whole context. The accordion publishes one scrollable region, opens one
/// scroll scope so that a section entirely outside the window is not drawn, stacks the sections, and
/// **clears the residue itself**: the cells between the last section's last row and the bottom of
/// the viewport are inside the accordion's rectangle and no section owns them (§2).
pub fn draw_into<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, s: Screen) {
    let id = Id::named("accordion");
    let view = cx.area();
    let max = (0, (s.content_rows() - i32::from(view.h)).max(0));
    let at = (0, 0);
    let _ = cx.scrollable(id, view, Interest::NONE, Scrollable::between(at, max));

    let body_paint = cx.theme().paint(Role::Body);
    cx.scroll_scope(id, view, at, max, |cx| {
        let window = cx.visible_rows();
        let mut y = 0i32;
        for i in 0..SECTIONS {
            let want = 1 + i32::from(s.body_height(i));
            // **The caller's culling, not the engine's.** A section entirely outside the window is
            // not drawn and therefore declares nothing.
            if y + want > window.start && y < window.end {
                cx.with_key(i as u64, |cx| section(ink, cx, i, y, view.w, s));
            }
            y += want;
        }
        // The residue, and it is the container's. One verb a row, so that `reported == writes`
        // holds and the pair really is two sources compared — `Ctx::fill` returns `()`.
        for row in y.max(window.start)..window.end {
            let _ = ink.run(cx, 0, row, " ", view.w, body_paint);
        }
    });
}

/// One section: a one-row header, and a body that is drawn, or is not.
fn section<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, i: usize, y: i32, w: u16, s: Screen) {
    let head = Rect::new(0, y, w, 1);
    let id = cx.id();
    let _ = cx.interact(id, head, Interest::CLICK.with(Interest::FOCUS));

    // The header is a partition of its row: one cell of chevron, the title in the rest.
    let title = cx.theme().paint(Role::Title);
    let open = i < s.open;
    let _ = ink.run(cx, 0, y, if open { "v" } else { ">" }, 1, title);
    let label = Rect::new(1, y, w.saturating_sub(1), 1);
    let _ = text_into(
        ink,
        cx,
        label,
        TITLES[i],
        &TextOpts {
            role: Role::Title,
            pad: Role::Title,
            ..TextOpts::default()
        },
    );

    let h = s.body_height(i);
    if h == 0 && s.closed == Closed::Skip {
        // **The closure is not called.** This is the whole of *closed content is not drawn*.
        return;
    }
    let mut child = cx.child(Rect::new(0, y + 1, w, h));
    body_into(ink, &mut child, w, h, s.body);
}

/// The body: [`CHIP_ROWS`] x [`CHIP_COLS`] chips and [`BUTTONS`] buttons under them.
fn body_into<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, w: u16, h: u16, body: Body) {
    let cw = (w / CHIP_COLS).max(1);
    let chip_opts = ChipOpts::default();
    for row in 0..CHIP_ROWS {
        if body == Body::Culls && row >= h {
            break;
        }
        for col in 0..CHIP_COLS {
            let cells = Rect::new(i32::from(col) * i32::from(cw), i32::from(row), cw, 1);
            cx.with_key(u64::from(row) * 16 + u64::from(col), |cx| {
                if body == Body::DeclaresOnly && row >= h {
                    declared_and_not_painted(cx, cells, chip_opts.interest);
                } else {
                    let _ = chip_into(ink, cx, cells, "option", &chip_opts);
                }
            });
        }
    }
    let button_opts = ButtonOpts::default();
    for row in 0..BUTTONS {
        let at = CHIP_ROWS + row;
        if body == Body::Culls && at >= h {
            break;
        }
        let cells = Rect::new(0, i32::from(at), w, 1);
        cx.with_key(0x100 + u64::from(row), |cx| {
            if body == Body::DeclaresOnly && at >= h {
                declared_and_not_painted(cx, cells, button_opts.interest);
            } else {
                let _ = button_into(ink, cx, cells, "apply", &button_opts);
            }
        });
    }
}

/// **A widget's region without its paint** — [`chip_into`] and [`button_into`] with the drawing
/// half removed, which is exactly what each of them is minus [`crate::text::chip_drawn`].
///
/// It is the whole of [`Body::DeclaresOnly`], and it is one call rather than a flag inside the two
/// components because a component that could be asked to skip its own drawing is a component with a
/// branch nothing on this map wants.
#[track_caller]
fn declared_and_not_painted(cx: &mut Ctx<'_, '_>, cells: Rect, interest: Interest) {
    let id = cx.id();
    let _ = cx.interact(id, cells, interest);
}

// ── what one frame turned out to be ──────────────────────────────────────────────────────────────

/// **One frame of the accordion, as counts rather than as a picture.**
///
/// A number only a report prints is a number no gate can read — [`crate::dense::Shape`]'s rule.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Shape {
    /// Columns the engine reported written.
    pub writes: u64,
    /// Columns the caller asked for, before clipping. `asked - writes` is what the clip discarded.
    pub asked: u64,
    /// **Distinct cells touched, and it is only a screen figure while every section is closed.**
    ///
    /// [`Tally`] unions the coordinates the verbs were *called* in, and a body is drawn inside a
    /// `Ctx::child`, which moves the origin — so with sections open the twelve bodies land on one
    /// another and the union under-counts. It is the same collision [`crate::dense::modal_steady`]
    /// names for the overlay pass and [`crate::frame`] measures at 124 false double writes.
    ///
    /// **It is exact on the pair this scene is about**: with every section closed no body writes a
    /// cell on either arm — the correct one is never called and the defective one's every write is
    /// rejected by the clip — so there is nothing to collide, and `writes == distinct == 24 000` is
    /// a real partition of a real screen.
    pub distinct: u64,
    /// Drawing calls made.
    pub verbs: u64,
    /// Entries in the runtime's hit index.
    pub entries: usize,
    /// Entries in the focus ring.
    pub ring: usize,
    /// Tab stops — the ring with every `Group` collapsed onto one.
    pub stops: usize,
}

/// **Draw `s` and read the frame.**
///
/// One warm-up frame and one measured one: the five frame structures take their allocation on the
/// first frame that needs one and keep it, which is the same warming `tests/budget.rs` does.
pub fn shape(s: Screen) -> Shape {
    shape_over(s, 1).0
}

/// [`shape`], over `frames` measured frames, with the **per-frame** time beside it.
///
/// **The duration is a report and never a gate** (§21's rule, and R15's before it). It is here at
/// all because §8 states µs beside every one of its declaration figures, and a report that printed
/// the counts without them would be answering a question §8 did not ask.
///
/// # Panics
///
/// Panics on zero frames. A per-frame figure over no frames is a division by zero dressed as a
/// measurement.
pub fn shape_over(s: Screen, frames: u32) -> (Shape, Duration) {
    assert!(frames > 0, "a per-frame figure needs a frame");
    let mut driver = crate::runner::driver_at(W, s.viewport, Density::default());
    driver.frame(|cx| draw_into(&mut Tally::new(), cx, s));

    let mut elapsed = Duration::ZERO;
    let mut out = Shape::default();
    for _ in 0..frames {
        let mut tally = Tally::new();
        let started = Instant::now();
        driver.frame(|cx| draw_into(&mut tally, cx, s));
        elapsed += started.elapsed();
        let frame = driver.inspect();
        out = Shape {
            writes: tally.writes(),
            asked: tally.asked(),
            distinct: tally.distinct(),
            verbs: tally.verbs(),
            entries: frame.hits().len(),
            ring: frame.ring().len(),
            stops: frame.stop_count(),
        };
    }
    (out, elapsed / frames)
}

/// **What the frame costs a component**, drawn through [`Direct`] rather than through a [`Tally`].
///
/// The one number here comparable with §8's own 24.38 / 43.71 / 45.71 µs, and it is a separate
/// function for [`crate::listing::volume_cost`]'s reason: a `Tally` keeps a `BTreeSet` of every cell
/// it sees, so a figure taken with one in the loop is a report about the instrument.
///
/// A report and never a gate.
///
/// # It is a **minimum** over `repeats` and not a mean, and that is not a preference
///
/// The first spelling was a mean over eight frames and it read `1 621 µs` for six open against
/// `555` for twelve open on the same viewport — a frame that does strictly more work coming out
/// three times cheaper. Under `examples/collapsible_numbers.rs` the counting global allocator is
/// installed, which is exactly where a mean stops being a measurement. The prototype's own last
/// commit says the same thing about the same numbers: *every number quoted in the answer is now a
/// minimum over repeats rather than a single sample*.
///
/// # Panics
///
/// Panics on zero repeats. A minimum over nothing is `Duration::MAX` wearing a measurement's
/// clothes.
pub fn cost(s: Screen, repeats: u32) -> Duration {
    assert!(
        repeats > 0,
        "a minimum over no repeats is not a measurement"
    );
    let mut driver = crate::runner::driver_at(W, s.viewport, Density::default());
    let mut ink = Direct;
    for _ in 0..WARM_FRAMES {
        driver.frame(|cx| draw_into(&mut ink, cx, s));
    }
    let mut best = Duration::MAX;
    for _ in 0..repeats {
        let started = Instant::now();
        driver.frame(|cx| draw_into(&mut ink, cx, s));
        best = best.min(started.elapsed());
    }
    best
}

/// How many frames are drawn before a timing starts. **Four**, because the five frame structures
/// take their allocation on the first frame that needs one and keep it.
const WARM_FRAMES: u32 = 4;

/// The surface `s` draws, cell for cell.
fn canvas(s: Screen) -> Canvas {
    let mut driver = crate::runner::driver_at(W, s.viewport, Density::default());
    let mut pen = Pen::new(W, s.viewport);
    driver.frame(|cx| draw_into(&mut pen, cx, s));
    pen.end_frame();
    pen.into_canvas()
}

/// **The correct accordion against a `defective` one, compared cell for cell. 0 cells over 0 rows.**
///
/// The scene's whole point, as a value: the defect is invisible to every instrument in this crate
/// that reads a surface, which is why the assertions are on the hit index and the ring instead.
/// True of both defective spellings — [`Screen::zero_rect`], whose writes are all rejected by the
/// clip, and [`Screen::zero_rect_declaring`], which does not write at all.
pub fn surfaces_are_identical(defective: Screen) -> Diff {
    canvas(Screen::correct(0)).diff(&canvas(defective))
}

/// **The same question for the mid-transition frame**, where the amplitude is 26 rather than 408.
///
/// # The substitution, named rather than hidden
///
/// Six sections are open here, and a body is drawn inside a `Ctx::child`, which moves the origin —
/// so [`Pen`] unions the six bodies onto one another and this canvas is **not** a picture of the
/// screen. It is on **both** arms, so it is on the side of neither, and what the answer means is
/// *the arm that ignores its rectangle puts no cell anywhere the other one does not* — which is the
/// question. [`surfaces_are_identical`] needs no such caveat: with every section closed, no body
/// writes a cell on either arm.
pub fn transition_surfaces_are_identical() -> Diff {
    canvas(Screen::collapsing(Body::Culls)).diff(&canvas(Screen::collapsing(Body::Uncut)))
}

/// §20's nine counters over one arm of the accordion.
///
/// The allocation total is the caller's, for [`crate::runner::Run::counters`]'s reason:
/// `vitui-alloc-probe` is a dev-dependency and a library cannot install a global allocator on a
/// consumer's behalf. There is no default, because *a figure defaulted to zero is a counter that
/// prints `0` when it means nobody counted*.
pub fn counters(s: Screen, allocations: Allocations) -> Counters {
    let mut driver = crate::runner::driver_at(W, s.viewport, Density::default());
    let mut tally = Tally::new();
    driver.frame(|cx| draw_into(&mut tally, cx, s));
    let mut tally = Tally::new();
    driver.frame(|cx| draw_into(&mut tally, cx, s));
    Counters::of(&driver, &tally, allocations)
}

/// **Which of §20's nine counters tell the correct closed accordion from `defective`.**
///
/// [`crate::listing::counters_that_separate_them`]'s question at the other end. There the answer is
/// the empty list and the equality against a reference render is the only detector; here it depends
/// on **which** spelling of the defect is asked about, and the difference is the finding:
///
/// | `defective` | counters that move |
/// |---|---|
/// | [`Screen::zero_rect`] | `verbs`, `regions`, `tab stops` |
/// | [`Screen::zero_rect_declaring`] | `regions`, `tab stops` |
///
/// **No counter that reads a cell appears in either row**, which is §8's *no golden-cell gate can
/// see it*. `verbs` appearing in the first is a fact about that spelling and not about the defect:
/// a body that declares every row and paints the admitted ones makes exactly the drawing calls the
/// correct one makes, so the verb count goes blind while the hit index still carries 408 entries
/// nobody can reach. A gate written on `verbs` would be green on it.
///
/// A counter that is [`Reading::Unreachable`] on both arms contributes nothing rather than counting
/// as agreement — `marked`, and it will stay that way.
///
/// [`Reading::Unreachable`]: crate::counters::Reading::Unreachable
pub fn counters_that_separate_them(
    defective: Screen,
    correct_allocations: Allocations,
    defective_allocations: Allocations,
) -> Vec<Counter> {
    let a = counters(Screen::correct(0), correct_allocations);
    let b = counters(defective, defective_allocations);
    Counter::ALL
        .into_iter()
        .filter(|c| {
            let (left, right) = (a.get(*c).measured(), b.get(*c).measured());
            left.is_some() != right.is_some() || (left.is_some() && left != right)
        })
        .collect()
}

// ── scene 10: the document, the fold set and the anchor ──────────────────────────────────────────

/// How many lines the document holds. §8's own.
pub const DOC_LINES: usize = 200_000;
/// A block opens every this many lines.
pub const BLOCK_EVERY: usize = 12;
/// How many lines a block holds.
pub const BLOCK_LEN: usize = 8;
/// Every this many **blocks** is closed — counted over blocks and not over lines, so the document
/// carries both kinds.
pub const CLOSE_EVERY: usize = 4;
/// **How many folds that comes to. Four thousand one hundred and sixty-seven**, §8's own number, and
/// [`Folds::close_every`] is asked for it rather than told it.
pub const FOLDS: usize = 4_167;
/// Where the edit lands.
pub const INSERT_AT: usize = 24;
/// How many lines the edit inserts.
pub const INSERT_LINES: usize = 10;
/// **How many folds sit on a line that opens no block when nothing reanchors them. 4 166 of 4 167.**
pub const MISANCHORED: usize = 4_166;
/// §8's reanchor cost, in microseconds. A **report**: see [`reanchor_cost`].
pub const SPEC_REANCHOR_US: f64 = 1.04;
/// §8's cost for the document edit the reanchor sits beside, in microseconds. A report.
pub const SPEC_EDIT_US: f64 = 72.83;

/// What a line is, which is what decides where a fold may start.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Line {
    /// Opens a block: the next `n` lines belong to it.
    Opens(u32),
    /// Plain text.
    Plain,
}

/// **The document.** Nothing here is drawn: two hundred thousand lines cost a `Vec` of two-word
/// values and not two hundred thousand `String`s, which is §7's rule arriving at §8's scene.
#[derive(Clone, Debug)]
pub struct Doc {
    lines: Vec<Line>,
}

impl Doc {
    /// `n` lines, a block every `every` lines, each `len` lines long.
    pub fn build(n: usize, every: usize, len: usize) -> Doc {
        let mut lines = vec![Line::Plain; n];
        let mut i = 0usize;
        while i + len + 1 < n {
            lines[i] = Line::Opens(len as u32);
            i += every;
        }
        Doc { lines }
    }

    /// §8's own document: [`DOC_LINES`] lines, a block every [`BLOCK_EVERY`], [`BLOCK_LEN`] long.
    pub fn spec() -> Doc {
        Doc::build(DOC_LINES, BLOCK_EVERY, BLOCK_LEN)
    }

    /// How many lines it holds.
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Whether it holds none.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Whether line `line` opens a block, and how long it is.
    pub fn block_len(&self, line: usize) -> Option<u32> {
        match self.lines.get(line) {
            Some(Line::Opens(n)) => Some(*n),
            _ => None,
        }
    }

    /// How many blocks the document opens.
    pub fn blocks(&self) -> usize {
        self.lines
            .iter()
            .filter(|l| matches!(l, Line::Opens(_)))
            .count()
    }

    /// **The edit that moves every fold below it.** Inserts `k` plain lines at `at`.
    pub fn insert(&mut self, at: usize, k: usize) {
        let at = at.min(self.lines.len());
        self.lines
            .splice(at..at, std::iter::repeat_n(Line::Plain, k));
    }
}

/// **How a closed fold's position survives an edit above it.**
///
/// §8's own sentence: *a fold set is keyed on a line number, not on a node id, and every edit above
/// it moves it.* A tree node needs none of this, which is the one place folding is not the tree
/// case.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Anchor {
    /// **The defect.** Absolute line numbers, left alone — what a reader writes when the tree case
    /// is the only one they have seen, because a node id needs no maintenance.
    Raw,
    /// **The rule.** Shifted by the edit: a `partition_point` and a walk, `O(folds at or after it)`.
    Shifted,
}

impl Anchor {
    /// The words a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Anchor::Raw => "left alone",
            Anchor::Shifted => "reanchored",
        }
    }
}

/// **The closed folds, sorted by start line. Caller state**, for the same forced reason a
/// collection's selection is: it is one of the index's two inputs and only the party holding the
/// document can materialise the index (§8).
#[derive(Clone, Debug, Default)]
pub struct Folds {
    closed: Vec<u32>,
}

impl Folds {
    /// Fold every `every`-th **block** of `doc`.
    pub fn close_every(doc: &Doc, every: usize) -> Folds {
        let mut closed = Vec::new();
        let mut block = 0usize;
        for line in 0..doc.len() {
            if doc.block_len(line).is_some() {
                if block.is_multiple_of(every) {
                    closed.push(line as u32);
                }
                block += 1;
            }
        }
        Folds { closed }
    }

    /// How many folds are closed.
    pub fn len(&self) -> usize {
        self.closed.len()
    }

    /// Whether none is.
    pub fn is_empty(&self) -> bool {
        self.closed.is_empty()
    }

    /// The lines the closed folds sit on.
    pub fn lines(&self) -> &[u32] {
        &self.closed
    }

    /// **The reconciliation the tree case never needed.** An insert of `k` lines at `at` moves every
    /// fold anchored at or after it — and [`Anchor::Raw`] is the spelling that does not.
    pub fn reanchor(&mut self, at: usize, k: usize, anchor: Anchor) {
        if anchor == Anchor::Raw {
            return;
        }
        let first = self.closed.partition_point(|&s| (s as usize) < at);
        for start in &mut self.closed[first..] {
            *start += k as u32;
        }
    }

    /// **How many closed folds no longer sit on a line that opens a block.**
    ///
    /// A count, and it is zero or it is a silent defect: nothing throws, the screen stays plausible,
    /// and every one of these hides the wrong lines.
    pub fn misanchored(&self, doc: &Doc) -> usize {
        self.closed
            .iter()
            .filter(|&&s| doc.block_len(s as usize).is_none())
            .count()
    }
}

/// **Play the fold scene on one arm and report `(misanchored, folds)`.**
///
/// Ten lines at line [`INSERT_AT`] of §8's document. [`Anchor::Raw`] answers [`MISANCHORED`] of
/// [`FOLDS`] and [`Anchor::Shifted`] answers 0 of [`FOLDS`], and both are §8's own numbers.
pub fn misanchored_after_the_edit(anchor: Anchor) -> (usize, usize) {
    let mut doc = Doc::spec();
    let mut folds = Folds::close_every(&doc, CLOSE_EVERY);
    let before = folds.len();
    doc.insert(INSERT_AT, INSERT_LINES);
    folds.reanchor(INSERT_AT, INSERT_LINES, anchor);
    (folds.misanchored(&doc), before)
}

/// **What the reanchor costs, beside what the document edit it sits next to costs.**
///
/// `(reanchor, edit)`, each a minimum over `repeats`. §8 states 1.04 µs against 72.83, and the
/// prototype's own last commit is *the fold anchor's cost was being reported as the document edit it
/// sits beside* — which is why the two are returned as a pair rather than as one number.
///
/// A **report**, never a gate: both are one-shot operations over a 200 000-element `Vec`, and a gate
/// on either would be a report about this machine's allocator.
///
/// # Panics
///
/// Panics on zero repeats. A minimum over nothing is `Duration::MAX` wearing a measurement's
/// clothes.
pub fn reanchor_cost(repeats: u32) -> (Duration, Duration) {
    assert!(
        repeats > 0,
        "a minimum over no repeats is not a measurement"
    );
    let mut reanchor = Duration::MAX;
    let mut edit = Duration::MAX;
    for _ in 0..repeats {
        let mut doc = Doc::spec();
        let mut folds = Folds::close_every(&doc, CLOSE_EVERY);
        let started = Instant::now();
        doc.insert(INSERT_AT, INSERT_LINES);
        edit = edit.min(started.elapsed());
        let started = Instant::now();
        folds.reanchor(INSERT_AT, INSERT_LINES, Anchor::Shifted);
        reanchor = reanchor.min(started.elapsed());
        std::hint::black_box(&folds);
        std::hint::black_box(&doc);
    }
    (reanchor, edit)
}

// ── the subject, and the scan that says whether it is here ───────────────────────────────────────

/// **The component both scenes are scenes of, and it is not declared yet.**
///
/// One subject and not two, which is [`Verdict::of`]'s vacuity refusal doing its job on a population
/// of one: *no component exists* is `Unmet` over one rather than `Met` over nothing.
pub const SUBJECTS: [&str; 1] = ["collapsible"];

/// Where [`SUBJECTS`] is declared, as `(module file, the declaration)`.
///
/// The home is the freeze's, joined through [`crate::Family`]: `collapsible`'s family is F4
/// disclosure, whose module is `disclose.rs`.
pub const DECLARATIONS: [(&str, &str); 1] = [("disclose.rs", "pub fn collapsible(")];

/// **Which of [`SUBJECTS`] this crate actually declares. Today: none.**
///
/// A source scan and not a `use`, for [`crate::dense::subjects_declared`]'s reason: *the item does
/// not exist* has no expression, and a `compile_fail` fence would pass today and pass again the day
/// somebody renames the module. The predicate is `crate::dense::declares` and it is shared rather
/// than copied — one definition of *a line that is not a comment*, which is the only thing that
/// makes *fires in both directions* mean anything.
pub fn subjects_declared() -> Vec<&'static str> {
    let mut out = Vec::new();
    for (subject, (file, declaration)) in SUBJECTS.into_iter().zip(DECLARATIONS) {
        let path = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).join(file);
        let source = std::fs::read_to_string(&path).unwrap_or_default();
        if crate::dense::declares(&source, declaration) {
            out.push(subject);
        }
    }
    out
}

/// **Whether the two scenes stand on their subject, as a verdict rather than as a sentence.**
///
/// `Unmet` over one, inverted by **components 22**. Everything the screens themselves can be asked
/// is measured and green — the surface equality, the 408 on both columns at once, the
/// mid-transition 26 and the fold anchor's 4 166 against 0 — and what is missing is the subject.
pub fn standing() -> Verdict {
    let declared = subjects_declared();
    Verdict::of(
        SUBJECTS.len(),
        SUBJECTS.len() - declared.len(),
        "`collapsible` is not declared in this crate, so what stands on the accordion is a stand-in \
         stack of headers and bodies and not the component. The screen, its 13 hit entries closed \
         against 421 at `h = 0`, the 408 on the hit index and the ring at once, the two surfaces \
         proved identical cell for cell, the mid-transition 26 and the fold anchor's 4 166 of \
         4 167 against 0 are measured and green; what is missing is the subject",
        "components 22",
    )
}

/// **The sentence a scene waiting for its subject fails with**, or `None` once it is declared.
///
/// Separated from [`assert_stands_up`] because the message is the mechanism and the panic is only
/// how it is delivered, and it takes the declaration list as an argument for the same reason: the
/// day the crate is on the other side of it, the hostile case is still one call away.
///
/// # This is components 09's criterion 7, and it is what makes a red scene readable
///
/// A scene that fails because it is unimplemented and a scene that fails because the code is wrong
/// are **the same failure** unless the message separates them. This one names the subject, the file
/// it belongs in, the declaration to look for, and the ticket — and says in as many words that it is
/// not a defect in the screen.
pub fn owed_message(declared: &[&str], scene: &str) -> Option<String> {
    if declared.len() == SUBJECTS.len() {
        return None;
    }
    let owed: Vec<String> = SUBJECTS
        .into_iter()
        .zip(DECLARATIONS)
        .filter(|(id, _)| !declared.contains(id))
        .map(|(id, (file, declaration))| format!("`{id}` (`src/{file}`: `{declaration}…)`)"))
        .collect();
    Some(format!(
        "{scene} is not standing, and it is waiting for its subject rather than failing: {} of {} \
         components are undeclared — {}. This is not a defect in the screen. The accordion is \
         drawn, its hit entries and its ring are counted, the two arms are proved identical cell \
         for cell, the `h = 0` spelling declares 408 more on both columns at once, a body two rows \
         tall declares 26 fewer than one that ignores its rectangle, and the fold set lands 4 166 \
         of 4 167 folds on a line that opens no block and 0 when it is reanchored — see \
         `crate::accordion::tests`. Inverted by `components 22`",
        owed.len(),
        SUBJECTS.len(),
        owed.join(", "),
    ))
}

/// **Fail with the subject that is missing, the file it belongs in, and the ticket.**
///
/// # Panics
///
/// Panics while [`SUBJECTS`] is undeclared, which is **today**. Components ticket 22 inverts it.
pub fn assert_stands_up(scene: &str) {
    if let Some(message) = owed_message(&subjects_declared(), scene) {
        panic!("{message}");
    }
}

/// **A headless driver at `s`'s own size**, for a caller that wants to drive the frames itself.
///
/// The allocation total is measured this way rather than from inside the crate, for
/// [`counters`]'s reason: `vitui-alloc-probe` is a dev-dependency, a library cannot install a
/// global allocator on a consumer's behalf, and the driver's own attachment allocates — so the probe
/// has to be able to start counting *after* it. `examples/collapsible_numbers.rs` is the caller.
pub fn driver_for(s: Screen) -> Driver {
    crate::runner::driver_at(W, s.viewport, Density::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Criterion 2 and 3: the accordion at 0 / 6 / 12 open, and the `h = 0` spelling beside it.**
    ///
    /// Thirteen hit entries and twelve tab stops with every section closed; four hundred and eight
    /// more of each when the body is called at `h = 0` and draws every row anyway. **The excess is
    /// the same number on both columns**, and that is asserted rather than noticed: `Ctx::interact`
    /// appends to the hit index and to the ring in one call, before either looks at the rectangle,
    /// so a defect that adds one adds the other. §8's own pair is `478 - 70` and `423 - 15`.
    #[test]
    fn a_closed_body_is_not_called_and_the_h_zero_spelling_declares_four_hundred_and_eight_more() {
        let correct = shape(Screen::correct(0));
        let defective = shape(Screen::zero_rect());

        assert_eq!(
            correct.entries, CLOSED_ENTRIES,
            "one region and twelve headers"
        );
        assert_eq!(correct.stops, CLOSED_STOPS, "the headers, and nothing else");
        assert_eq!(defective.entries, CLOSED_ENTRIES + UNCULLED_EXCESS);
        assert_eq!(defective.stops, CLOSED_STOPS + UNCULLED_EXCESS);
        assert_eq!(UNCULLED_EXCESS, SECTIONS * PER_BODY);
        assert_eq!(UNCULLED_EXCESS, 408);

        // **One subtraction, twice.** A gate on either column alone would be satisfied by a defect
        // that declared a hit entry and no ring entry, which is a different defect.
        assert_eq!(
            defective.entries - correct.entries,
            defective.stops - correct.stops,
            "`Ctx::interact` appends to both indexes in one call, so the two excesses are one"
        );

        // The prediction the construction makes, so the numbers are arithmetic and not a recording.
        for screen in [Screen::correct(0), Screen::zero_rect()] {
            let measured = shape(screen);
            assert_eq!(measured.entries, screen.predicted_entries());
            assert_eq!(measured.stops, screen.predicted_stops());
        }
    }

    /// **The two surfaces are identical, and no counter that reads a cell can see the defect.**
    ///
    /// The scene's whole argument, as two assertions. The first is the equality §8 states in the
    /// same sentence as the numbers — *the two surfaces are identical* — and the second is that
    /// argument as a list: of §20's nine counters, exactly `regions` and `tab stops` move.
    #[test]
    fn the_two_surfaces_are_identical_and_no_counter_that_reads_a_cell_can_see_it() {
        surfaces_are_identical(Screen::zero_rect())
            .assert_clean("the accordion, closed against h = 0");
        surfaces_are_identical(Screen::zero_rect_declaring())
            .assert_clean("the accordion, closed against a body that only declares");

        let correct = shape(Screen::correct(0));
        let defective = shape(Screen::zero_rect());
        assert_eq!(correct.writes, defective.writes, "the write count is blind");
        assert_eq!(correct.distinct, defective.distinct, "and so is the union");
        assert_eq!(
            correct.writes, correct.distinct,
            "a closed accordion is a partition of its screen, so both figures are the screen"
        );
        assert_eq!(correct.writes, u64::from(W) * u64::from(H));

        // **What the clip discarded**, which is the one cell figure that does move: ten rows a
        // section, three hundred columns wide, twelve times.
        assert_eq!(correct.asked, correct.writes, "nothing was clipped");
        assert_eq!(
            defective.asked - defective.writes,
            u64::from(W) * u64::from(BODY_ROWS) * SECTIONS as u64
        );

        // **Three counters separate the `h = 0` spelling and only two separate the one beside it**,
        // and the difference is why this scene asserts on the hit index. A body that declares every
        // row and paints the admitted ones makes exactly the drawing calls the correct one makes, so
        // `verbs` goes blind while 408 entries nobody can reach are still in the index. **No
        // counter that reads a cell is in either list.**
        let none = Allocations::over(1, 0);
        assert_eq!(
            counters_that_separate_them(Screen::zero_rect(), none, none),
            vec![Counter::Verbs, Counter::Regions, Counter::TabStops],
            "a golden-cell gate, a write count and a distinct-cell count are all green on the arm \
             that declares 408 entries nobody can reach"
        );
        assert_eq!(
            counters_that_separate_them(Screen::zero_rect_declaring(), none, none),
            vec![Counter::Regions, Counter::TabStops],
            "a gate written on `verbs` would be green on this one, and it declares the same 408"
        );
        let quiet = shape(Screen::zero_rect_declaring());
        assert_eq!(
            quiet.verbs, correct.verbs,
            "the same drawing calls, exactly"
        );
        assert_eq!(quiet.writes, correct.writes);
        assert_eq!(
            quiet.asked, correct.asked,
            "and nothing even asked to be clipped"
        );
        assert_eq!(quiet.entries, defective.entries, "and the same hit index");
        assert_eq!(quiet.stops, defective.stops);
    }

    /// **The ring and the tab-stop count are equal here, and that is why §8 needs two constants.**
    ///
    /// A `ScopeKind::Group` collapses its whole range onto one stop while leaving every entry in the
    /// ring. This screen opens no group, so the two columns agree everywhere — which is what makes
    /// [`CHROME_ENTRIES`] and [`CHROME_STOPS`] a fact about the prototype's chrome rather than a
    /// second free parameter.
    #[test]
    fn this_screen_opens_no_group_so_its_ring_and_its_stop_count_are_the_same_number() {
        for screen in [
            Screen::correct(0),
            Screen::correct(6),
            Screen::correct(12),
            Screen::zero_rect(),
            Screen::collapsing(Body::Culls),
            Screen::collapsing(Body::Uncut),
        ] {
            let measured = shape(screen);
            assert_eq!(
                measured.ring, measured.stops,
                "a group standing here would make §8's two columns two measurements"
            );
        }
    }

    /// **§8's remaining five declaration figures are this screen plus one constant pair, fitted
    /// from the first.**
    ///
    /// The chrome is fixed by *one* row — *twelve sections closed: 70 hit entries and 15 tab stops*
    /// — and then predicts the other five to the unit. That is what makes it a reconciliation and
    /// not a fit: five predictions out of one constant pair, and the fifth is
    /// [`the_mid_transition_pair_is_this_screen_plus_the_same_chrome`], which is a different table
    /// of §8 again.
    ///
    /// The twelve-open row is the one that needs [`PROTO_VIEWPORT`], and it is the reason that
    /// constant exists: the content is 132 rows, and whether an eighth section is reached at all is
    /// the difference between a 74-row window and an 80-row one.
    #[test]
    fn the_rest_of_8s_declaration_figures_are_this_screen_plus_the_chrome_fitted_from_the_first() {
        let closed = shape(Screen::correct(0));
        assert_eq!(
            (
                SPEC_CLOSED_ENTRIES - closed.entries,
                SPEC_CLOSED_STOPS - closed.stops
            ),
            (CHROME_ENTRIES, CHROME_STOPS),
            "the pair is fitted here and nowhere else"
        );

        for (screen, entries, stops) in [
            (Screen::zero_rect(), SPEC_ZERO_ENTRIES, SPEC_ZERO_STOPS),
            (Screen::correct(6), SPEC_SIX_ENTRIES, SPEC_SIX_STOPS),
            (
                Screen::correct(12).on_the_prototypes_viewport(),
                SPEC_TWELVE_ENTRIES,
                SPEC_TWELVE_STOPS,
            ),
        ] {
            let measured = shape(screen);
            assert_eq!(
                measured.entries + CHROME_ENTRIES,
                entries,
                "§8's hit entries for {screen:?}"
            );
            assert_eq!(
                measured.stops + CHROME_STOPS,
                stops,
                "§8's tab stops for {screen:?}"
            );
        }
    }

    /// **A body that does not cull declares a count flat in its rectangle's height.**
    ///
    /// Criterion 4's mechanism, and the mid-transition case is one point on it. The correct arm is
    /// proportional to the rectangle — four an admitted chip row, then one a button row — and the
    /// defective one is constant at [`PER_BODY`] whatever it was handed, which is the same failure
    /// the `h = 0` pair is at full amplitude.
    #[test]
    fn a_body_that_does_not_cull_declares_a_count_flat_in_the_rectangle_it_was_handed() {
        let mut culled = Vec::new();
        // One section open at `h`, eleven closed. The twelve headers are on the ring either way, so
        // what moves between the two arms is the body and nothing else.
        for h in 1..=BODY_ROWS {
            let mut screen = Screen::correct(1);
            screen.collapsing = Some(h);
            let obedient = shape(screen);
            screen.body = Body::Uncut;
            let rude = shape(screen);

            assert_eq!(obedient.stops, SECTIONS + admitted(h));
            assert_eq!(
                rude.stops,
                SECTIONS + PER_BODY,
                "flat in the height, at h = {h}"
            );
            culled.push(obedient.stops - SECTIONS);
        }
        assert_eq!(culled, vec![4, 8, 12, 16, 20, 24, 28, 32, 33, 34]);

        // **`h = 0` is the one point where the relation stops being about culling**, because a
        // closed section's body is not called at all — which is the 408 pair and not this one. A
        // loop that ran from zero would read the correct arm and the defective one as equal there
        // and score the whole relation weaker than it is.
        let mut closed = Screen::correct(1);
        closed.collapsing = Some(0);
        assert_eq!(shape(closed).stops, SECTIONS);
        closed.body = Body::Uncut;
        assert_eq!(
            shape(closed).stops,
            SECTIONS,
            "`Closed::Skip` gets there first"
        );
    }

    /// **The mid-transition pair is this screen plus the same chrome, at the height where §8's own
    /// subtraction is 26.**
    ///
    /// §8 states *273 ring entries against 247* and no height beside it. Two rows of body left is
    /// the height at which the difference is 26, and — the part that makes two an answer rather
    /// than a fit — **both absolutes** land on §8's, with the [`CHROME_ENTRIES`] fitted from a
    /// different row of a different table.
    #[test]
    fn the_mid_transition_pair_is_this_screen_plus_the_same_chrome() {
        let obedient = shape(Screen::collapsing(Body::Culls));
        let rude = shape(Screen::collapsing(Body::Uncut));

        assert_eq!(rude.ring - obedient.ring, MID_EXCESS);
        assert_eq!(MID_EXCESS, SPEC_MID_UNCULLED - SPEC_MID_CULLED);
        assert_eq!(MID_EXCESS, 26);
        assert_eq!(obedient.ring + CHROME_ENTRIES, SPEC_MID_CULLED);
        assert_eq!(rude.ring + CHROME_ENTRIES, SPEC_MID_UNCULLED);

        // And the surface is no help here either, at a twenty-sixth of the amplitude.
        transition_surfaces_are_identical().assert_clean("halfway down a collapse");
    }

    /// **Twelve open sections declare fewer than twice six, because a section off screen declares
    /// nothing.**
    ///
    /// §8 states this as a cost ratio — *1.05x six open rather than 2x* — and a timing is a report
    /// (R15). The count is the mechanism the timing is a symptom of, and it is a relation rather
    /// than an equality because the number belongs to the viewport: eight sections are reached at
    /// [`H`] and seven at [`PROTO_VIEWPORT`], which is one section and exactly `1 + 34` entries.
    #[test]
    fn twelve_open_sections_declare_fewer_than_twice_six_because_one_off_screen_declares_nothing() {
        let six = shape(Screen::correct(6));
        let twelve = shape(Screen::correct(12));
        let twelve_narrow = shape(Screen::correct(12).on_the_prototypes_viewport());

        assert_eq!(Screen::correct(6).sections_drawn(), SECTIONS, "72 rows fit");
        assert_eq!(Screen::correct(12).sections_drawn(), 8);
        assert_eq!(
            Screen::correct(12)
                .on_the_prototypes_viewport()
                .sections_drawn(),
            7
        );
        assert!(
            twelve.entries < six.entries * 2,
            "{} against {}",
            twelve.entries,
            six.entries
        );
        assert_eq!(
            twelve.entries - twelve_narrow.entries,
            1 + PER_BODY,
            "six rows of viewport is one more section, and a section is a header and its body"
        );
        assert_eq!(Screen::correct(12).content_rows(), 132);
    }

    // ── scene 10 ─────────────────────────────────────────────────────────────────────────────────

    /// **The fold set is built rather than typed**: 4 167 folds out of a 200 000-line document.
    ///
    /// §21's row states the number and this asks the document for it, so a change to any of the
    /// three parameters is a failing test rather than a scene quietly measuring something else.
    #[test]
    fn the_document_opens_sixteen_thousand_blocks_and_a_quarter_of_them_are_closed() {
        let doc = Doc::spec();
        assert_eq!(doc.len(), DOC_LINES);
        assert_eq!(doc.blocks(), 16_666);
        let folds = Folds::close_every(&doc, CLOSE_EVERY);
        assert_eq!(folds.len(), FOLDS, "§21's own 4 167");
        assert_eq!(folds.len(), doc.blocks().div_ceil(CLOSE_EVERY));
        assert_eq!(
            folds.misanchored(&doc),
            0,
            "every fold opens a block to start with, or the scene starts from its own defect"
        );
    }

    /// **Ten lines at line 24, and 4 166 of 4 167 folds sit on a line that opens no block.**
    ///
    /// The scene with no throw and a plausible screen. Nothing panics, nothing is out of bounds, and
    /// every one of the 4 166 hides the wrong lines — which is why the gate is a **count** and there
    /// is no picture to compare.
    ///
    /// Both directions, because a gate written only on [`Anchor::Shifted`] is satisfied by a fold
    /// set that is empty.
    #[test]
    fn four_thousand_one_hundred_and_sixty_six_folds_of_four_thousand_one_hundred_and_sixty_seven_land_wrong()
     {
        let (raw, folds) = misanchored_after_the_edit(Anchor::Raw);
        assert_eq!(folds, FOLDS);
        assert_eq!(raw, MISANCHORED, "§8's own 4 166 of 4 167");
        assert_eq!(
            folds - raw,
            1,
            "the one above the edit is the one that survives"
        );

        let (shifted, again) = misanchored_after_the_edit(Anchor::Shifted);
        assert_eq!(
            again, FOLDS,
            "the reanchor moves folds and does not lose them"
        );
        assert_eq!(shifted, 0, "§8's own 0 of 4 167");
    }

    /// **The folds that survive are exactly the ones above the edit**, which is the mechanism rather
    /// than the instance.
    ///
    /// A gate on 4 166 alone is satisfied by any edit that moves any number of folds. This one says
    /// *which*: `partition_point` splits the set at the edit, everything below it moves, and what
    /// survives untouched is what was above.
    #[test]
    fn the_folds_that_survive_are_exactly_the_ones_above_the_edit() {
        let doc = Doc::spec();
        let folds = Folds::close_every(&doc, CLOSE_EVERY);
        let above = folds
            .lines()
            .iter()
            .filter(|&&s| (s as usize) < INSERT_AT)
            .count();
        assert_eq!(above, 1, "the fold at line 0, and no other");
        assert_eq!(folds.len() - above, MISANCHORED);
    }

    // ── the subject ──────────────────────────────────────────────────────────────────────────────

    /// **Both scenes are red, and they are red because `collapsible` is not declared.**
    ///
    /// The loud half. `Verdict::of` refuses vacuity in its constructor, so this is `Unmet` over one
    /// rather than `Met` over nothing, and the panic names the subject and the ticket.
    #[test]
    #[should_panic(expected = "components 22")]
    fn the_accordion_is_red_because_collapsible_is_not_declared() {
        assert_eq!(
            subjects_declared(),
            Vec::<&str>::new(),
            "`collapsible` is declared, so this ticket's scenes are standing and the register, \
             `crate::scenes` and this module all owe a deliberate edit"
        );
        standing().assert_met("the accordion of twelve sections");
    }

    /// **The waiting message says which failure it is**, which is components 09's criterion 7.
    ///
    /// A scene that fails because it is unimplemented and a scene that fails because the code is
    /// wrong are the same failure unless the message separates them. Read over a declaration list
    /// rather than over the crate, so the hostile case is one call away on both sides of the day
    /// `collapsible` lands.
    #[test]
    fn the_waiting_message_separates_unimplemented_from_wrong() {
        let message = owed_message(&[], "the accordion of twelve sections")
            .expect("`collapsible` is undeclared, so the scene is not standing");
        assert!(message.contains("waiting for its subject rather than failing"));
        assert!(message.contains("not a defect in the screen"));
        assert!(message.contains("collapsible"));
        assert!(message.contains("src/disclose.rs"));
        assert!(message.contains("pub fn collapsible("));
        assert!(message.contains("components 22"));
        assert!(message.contains("408"));
        assert!(message.contains("4 166 of 4 167"));

        // And the other direction: once the subject is declared there is no message at all.
        assert!(owed_message(&["collapsible"], "the accordion").is_none());
    }

    /// **The scan finds a declaration when there is one**, so *nothing is declared* is a reading and
    /// not a scanner that has quietly stopped scanning.
    #[test]
    fn the_subject_scan_finds_a_declaration_when_there_is_one() {
        let (_, declaration) = DECLARATIONS[0];
        assert!(crate::dense::declares(
            &format!("{declaration}cx: &mut Ctx) {{}}"),
            declaration
        ));
        assert!(
            !crate::dense::declares(&format!("// {declaration}…) one day"), declaration),
            "a mention in a comment is not a declaration"
        );
    }

    /// **`assert_stands_up` is the panic a later gate calls**, and it is watched panicking.
    #[test]
    #[should_panic(expected = "waiting for its subject")]
    fn a_scene_with_no_subject_fails_loudly_rather_than_passing() {
        assert_stands_up("a 200 000-line document with 4 167 folds");
    }
}
