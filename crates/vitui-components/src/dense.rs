//! **The dense screen: 300×80, 338 interactive regions, and the five re-damage instances ADR 0026
//! prices — each one reachable, each one measured.**
//!
//! Components ticket 09. Spec §2, §21.
//!
//! > | where | cells re-damaged every steady frame |
//! > |---|---|
//! > | the application's own `cx.clear(body)` at the top of every frame | **6 662** |
//! > | `chip` filling its face before drawing its label | **2 648** |
//! > | a chip that does not narrow, whose label runs into its sibling's rectangle | **432** |
//! > | a modal's scrim filled *under* the dialog rather than around it | **229** |
//! > | a `panel` drawing its top border as one run and writing its title over it | **15** |
//!
//! That table is the whole argument for the partition rule, and until this ticket **not one of its
//! five rows was standing on a screen anything ran**. Ticket 06 reproduced the last one at 15 on a
//! panel; the other four were sentences. A sentence cannot be inverted, which is why ticket 10 —
//! *`text`, `chip`, `button`, `panel`, and clearing once* — needs this file before it can prove
//! anything: **a defect that is not on a screen is proved against a review.**
//!
//! # `marked` is unreachable and this measures its input
//!
//! Spec §20's `marked` is [`crate::counters::Reading::Unreachable`] and stays that way —
//! `crates/vitui-engine/src/damage.rs` is `pub(crate)` from top to bottom and `Presented` carries
//! no count of cells. What is knowable from here is the **rule** that produces it: the engine drops
//! a write whose value equals the resident value, so *writes whose value differs from what is
//! already there* is computable at the verb boundary. That is [`crate::runner::Canvas::repaints`],
//! carried across frames by [`Pen::over`] — a fresh [`Pen`] a frame reports every cell as a first
//! paint for ever, which is re-damage with the relation removed. Every figure in this module's
//! ledger is that quantity, and the metric row prints `marked=unreachable` and never `marked=0`.
//!
//! # The screen is a partition of 24 000 cells, and that is what makes the equality possible
//!
//! [`crate::form`]'s screen deliberately leaves a tail nobody writes, so that register row 7's
//! *no cell never* has something to stand on. This one deliberately does the opposite: **every one
//! of the 24 000 cells is written exactly once by exactly one branch**, so the correct build and
//! the naive twin are equal *cell for cell* rather than equal-except-the-tail. §2 states the two
//! halves as one sentence — *the owner of a rectangle writes all of it; a component handed a
//! rectangle writes all of that* — and the two screens are the two halves.
//!
//! It is also why the correct arm needs no `cx.clear` at all. §2's *the correct build clears once,
//! on its first frame and on a resize* is a property of an application **loop** and ticket 10 owns
//! it; what this file can show is the other end of the same fact, that a screen which covers itself
//! has nothing for a clear to fix and pays 0 where the every-frame spelling pays
//! [`CLEARED_EVERY_FRAME`].
//!
//! # Four components, and the screen is now drawn through them
//!
//! `text`, `chip`, `button` and `panel` are components ticket 10's, and they exist:
//! [`crate::text::text`], [`crate::text::chip`], [`crate::input::button`],
//! [`crate::structure::panel`]. For one ticket this screen stood on their **construction** instead
//! — [`crate::text::fit`], [`crate::frame::block`] and [`crate::state::press`], which is exactly
//! what spec §17's freeze says each of them is, `constructions: 1` apiece — and [`crate::scenes`]
//! filed scenes 1, 2 and 28 **red** with that as the failing set rather than green over a stand-in.
//! They are `Evaluated` now. See [`standing`].
//!
//! **Every defective arm is a defective component too**, which is what makes the five instances
//! statements about the library rather than about this file: `Arm::ChipFillsItsFace` reaches
//! [`crate::text::defective::chip_that_fills_its_face`], `Arm::BorderOverTheTitle` reaches
//! [`crate::structure::defective::panel_over_title`], and `Arm::ClearsEveryFrame` reaches
//! [`crate::app::defective::every_frame`]. One function with one boolean between it and the correct
//! one, in each case, so the diff a reviewer would have to catch is the diff a register can point
//! at.
//!
//! **One figure moved when the components landed**, and it is written down rather than absorbed:
//! [`CHIP_FILLED_FACE`] is 1 095 where ticket 09 measured 1 149, because a chip's label wears its
//! own face. See that constant — the 54 is a defect the stand-in had and the component does not.
//!
//! # Density is theme data and it changes rectangles, so 338 is a `Compact` figure
//!
//! Spec §3. At `Compact` a panel's ring is one cell and its interior is 74 rows; at `Cosy` it is
//! two and 72. The region count is `2 + 3 × (1 + rows + rows.div_ceil(2))`, which is **338** at 74
//! rows and 329 at 72 — so every measurement here names its density, and
//! [`crate::runner::play`]'s `Density::default()` (which is `Cosy`) is never what this screen is
//! played through.

use vitui_runtime::{Ctx, Density, Interest, Paint, Role};

use crate::app::Clears;
use crate::ink::Ink;
use crate::input::{ButtonOpts, button_into};
use crate::obligations::Verdict;
use crate::runner::{Canvas, Diff, Fixture, MetricRow, Painter, Pen, Run, compare_at, play_at};
use crate::structure::{PanelOpts, defective::panel_over_title, panel_into};
use crate::text::{ChipOpts, Justify, TextOpts, chip_into, defective as text_defective, text_into};
use vitui_runtime::Rect;
use vitui_runtime::layout::rect;

// ── the screen ───────────────────────────────────────────────────────────────────────────────────

/// The screen's width. §2 prices the dense screen at 300×80 and §21's row states it.
pub const W: u16 = 300;
/// The screen's height.
pub const H: u16 = 80;
/// **The second size the equality runs at.** §2: *the two screens are the same screen: 0 of 24 000
/// cells differ, at 300×80 and at 120×40.*
pub const NARROW: (u16, u16) = (120, 40);
/// How many panels stand side by side. 300 and 120 are both divisible by it, which is what keeps
/// the band a partition at either size.
pub const PANELS: u16 = 3;
/// How wide a chip is.
pub const CHIP: u16 = 12;
/// How wide the column right of the chip is — a button on an even row, a reading on an odd one.
pub const ACTION: u16 = 10;
/// The title every panel carries. **Fifteen columns**, which is the 15-cell instance's own width.
pub const TITLE: &str = " panel systems ";
/// How many cells the screen holds. 300 × 80.
pub const SCREEN: u64 = W as u64 * H as u64;
/// How many cells the narrow screen holds. 120 × 40.
pub const NARROW_SCREEN: u64 = NARROW.0 as u64 * NARROW.1 as u64;

/// **How many widgets each panel is asked for.** Exactly the rows a `Compact` panel has, so nothing
/// falls off the bottom and the screen stands its full region count.
pub const REQUESTED: usize = 74;

/// The dialog's width. Sixty by ten is **600 cells**, and that is the quantity §2's *600 fewer
/// writes* is about: a scrim filled under the dialog writes the dialog's own cells and one drawn
/// around it does not. The size is chosen so that structural figure reproduces by construction; the
/// re-damage figure beside it ([`SCRIM_UNDER`]) is measured and is not.
pub const DIALOG: (u16, u16) = (60, 10);

/// What the header says.
const HEADER: &str = "vitui — components";
/// What the footer says.
const FOOTER: &str = "ready";
/// The dialog's title.
const DIALOG_TITLE: &str = " confirm ";

/// The labels the rows cycle through. **Static, and that is deliberate**: this screen measures a
/// construction, and a label whose length varied with the step would move the write count for a
/// reason that is not the construction.
///
/// **Five of the eight are longer than a `Compact` label column at 120 columns and all eight are
/// shorter than one at 300**, which is a construction choice and not a magnitude: it is what puts
/// §16's one-cell ellipsis rule on the screen at the narrow size, and what makes
/// [`Arm::LabelDoesNotNarrow`] green at the wide size and red at the narrow one.
const LABELS: [&str; 8] = [
    "throughput, req/s",
    "queue depth",
    "retries in flight",
    "an unusually long metric name that will not fit",
    "latency p99, ms",
    "errors",
    "connections, live",
    "backlog",
];

/// What a chip says. **The fourth is wider than a chip**, which is the population
/// [`Arm::ChipDoesNotNarrow`] overruns on: one row in four, six columns each.
const VALUES: [&str; 4] = ["on", "off", "auto", "degraded, retrying"];

/// What a button says.
const ACTIONS: [&str; 4] = ["reset", "pause", "clear", "retry"];

/// What an odd row's reading says.
const UNITS: [&str; 4] = ["req/s", "ms", "%", "MB"];

// ── the ledger ───────────────────────────────────────────────────────────────────────────────────
//
// **Every number this screen is gated or reported on has exactly one home and it is here** — the
// runtime's ledger rule (`crates/vitui-runtime/src/ledger.rs`), inherited through `crate::form`.
//
// Every figure is a **count** over a deterministic screen, so none carries a machine: the labels are
// static, the arithmetic is integer, and a debug build and a release build produce the same numbers.
// The microsecond column of the metric row is a **report** and is gated by nothing (R15).
//
// **Where a figure reproduces §2's it says so, and where it does not both columns are printed and
// the arithmetic behind the difference is stated.** The magnitudes in §2 were taken on C01's
// prototype screen, which this ticket does not own and cannot recover; what a screen owes is that
// every *direction* and every *structural* number reproduces, and that no figure was aimed at.

/// **Interactive regions at 300×80, `Compact`. §21's own upper figure, and it is arithmetic.**
///
/// `2 + 3 × (1 + 74 + 37)`: a header, a footer, three panels each declaring itself, one chip a row,
/// and a button on every second row. Nothing here was tuned to reach it — the panel geometry is
/// `block`'s and the row count is what a `Compact` ring leaves.
pub const REGIONS: usize = 338;
/// Interactive regions at 120×40, `Compact`. `2 + 3 × (1 + 34 + 17)`.
pub const NARROW_REGIONS: usize = 158;
/// Tab stops at 300×80. Every chip and every button takes [`Interest::FOCUS`]; the header, the
/// footer and the three panels do not.
pub const STOPS: usize = 333;
/// Rect the correct build writes at 300×80. **Every cell of the screen, exactly once.**
pub const WRITES: u64 = SCREEN;

/// **Rect re-damaged every steady frame by `cx.clear(body)` at the top of the frame.**
///
/// §2 remembers **6 662** on C01's screen and this one says **9 024**. The quantity is *cells whose
/// steady value is not a space painted [`Role::Body`]* — the clear writes one into every cell and
/// the content writes its own value back — so it is a measure of **how much of a screen is ink**,
/// which is a property of the screen and not of the rule. 9 024 of 24 000 is 37.6% against C01's
/// 27.8%, and the direction, the relation to [`Arm::Correct`]'s 0 and the fact that it is the
/// largest of the five all reproduce.
pub const CLEARED_EVERY_FRAME: u64 = 9_024;
/// **Rect re-damaged every steady frame by a chip that fills its face before drawing its label.**
///
/// §2 remembers **2 648**, R07's original. The quantity is the *label* cells of every chip and not
/// the chip's: the fill writes the face over all twelve, the padding cells it lands on already carry
/// the face and cost nothing, and the label writes itself back over the rest. So it is
/// `Σ label cells whose value differs from a space in the face` over 222 chips, and it is
/// arithmetic — `3 panels × (18 × (2 + 3 + 4 + 11) + 2 + 3)` for the four values a row cycles
/// through, the fourth of which is elided to the chip's own twelve columns.
///
/// # It was 1 149 until components ticket 10, and the 54 that went is a finding
///
/// Ticket 09's stand-in chip painted its label [`Role::Dim`] on a face painted `Role::Face`, so
/// **every** cell of the label differed from the fill and the fourth value contributed all twelve of
/// its columns. `chip` does not: its label wears its face, because a label in a second paint is
/// repainted by the hover award every frame the pointer rests on it — [`crate::text::ChipOpts`]
/// carries the argument and `text::tests::a_hovered_chip_re_damages_its_own_eight_cells_and_not_
/// the_screen` is the eight cells it would otherwise have been ten.
///
/// So the one **space** inside `"degraded, r…"` now carries exactly what the fill wrote, and the
/// engine's equality filter drops it: `18 rows × 1 space × 3 panels = 54`. The instance is smaller
/// and the defect is the same defect; what shrank is the part of it that was being counted twice —
/// once as a fill-then-draw and once as a label wearing a role its face does not.
pub const CHIP_FILLED_FACE: u64 = 1_095;
/// **Rect re-damaged every steady frame by a chip that does not narrow.**
///
/// §2 remembers **432**. The quantity is `overrunning chips × overrun width`: one value in four is
/// eighteen columns in a twelve-column chip, so 54 chips of 222 overrun 6 columns each into the
/// column beside them, and the neighbour writes them back every frame. `54 × 6 = 324`, and both
/// factors are the screen's rather than the rule's.
pub const CHIP_NOT_NARROWED: u64 = 324;
/// **Rect re-damaged every steady frame by a scrim filled under the dialog.**
///
/// §2 remembers **229**, and this screen says **600 — the whole dialog**, which is a stronger
/// statement and not a looser one. The quantity is *the dialog's cells whose value differs from the
/// scrim's*: the scrim writes its own paint into all six hundred and the dialog writes its border,
/// its title, its text and its padding back. Here nothing in the dialog is painted in the scrim's
/// role, so the difference is total; §2's 229 says that on C01's screen 371 of the dialog's 600
/// cells already carried what the scrim wrote. **The number this screen can be held to is the
/// relation** — 600 against [`Arm::ScrimAroundTheDialog`]'s 0 — and the relation is the one §2
/// states.
pub const SCRIM_UNDER: u64 = 600;
/// **Rect re-damaged every steady frame by one panel's border run written over its own title.**
///
/// §2 remembers **15**, and **it reproduces exactly**, because the number *is* the title's width.
/// Ticket 06 measured the same 15 as `writes - distinct` on one panel; this is the same defect
/// counted as re-damage on a screen, and the screen carries [`PANELS`] of them.
pub const BORDER_OVER_TITLE: u64 = 15;
/// [`BORDER_OVER_TITLE`] across the screen's three panels.
pub const BORDER_OVER_TITLE_SCREEN: u64 = BORDER_OVER_TITLE * PANELS as u64;
/// **Writes the scrim-under arm costs over the scrim-around arm.** The dialog's own cell count, and
/// §2's *600 fewer writes*. See [`DIALOG`] for why that reproduces by construction.
///
/// It is equal to [`SCRIM_UNDER`] on this screen and that is a coincidence of one fact rather than
/// two: every cell the scrim writes under the dialog it also re-damages, because none of the
/// dialog's own cells is painted in the scrim's role. The two constants stay separate because they
/// are two questions — *what did the frame write* and *what did the terminal have to redraw* — and
/// on §2's screen they answered 600 and 229.
pub const SCRIM_EXCESS_WRITES: u64 = DIALOG.0 as u64 * DIALOG.1 as u64;

/// **Rect the naive twin re-damages every steady frame, and it is not the sum of its three
/// instances.**
///
/// The twin is a screen clear *and* a fill under every panel interior *and* a face filled under
/// every chip label, and it re-damages exactly what [`CLEARED_EVERY_FRAME`] does. Re-damage is a
/// count of **distinct cells**, so three defects whose cell sets are nested cost the largest of
/// them and not their total — a screen clear already changes every cell the other two change. A
/// report that added the five rows of ADR 0026's table together would be double counting, and this
/// constant is where that is written down.
pub const NAIVE_TWIN: u64 = CLEARED_EVERY_FRAME;

// ── the arms ─────────────────────────────────────────────────────────────────────────────────────

/// **Which of the screens is being drawn.**
///
/// One function draws all of them, so that what differs between two arms is the thing under test and
/// not the layout — [`crate::form`]'s arrangement and [`crate::frame::draw`]'s single boolean before
/// it. Every defective arm is one branch inside [`draw_into`] and each is named for the row of
/// ADR 0026's table it is.
///
/// [`crate::frame::draw`]: crate::frame::block_into
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Arm {
    /// **The partition discipline.** Every cell written exactly once, by exactly one branch.
    Correct,
    /// **The naive twin — the reference the correct build is proved equal to.**
    ///
    /// A screen clear, a fill under every panel interior, and a face filled under every chip label:
    /// three of ADR 0026's five instances at once. It is **kept and never deleted** (§21's rule,
    /// arriving here for the fourth time) and its existence is gated by path —
    /// `tests::the_naive_twin_is_declared_in_this_file_and_named_by_path`.
    Naive,
    /// **`cx.clear(body)` at the top of every frame.** The largest single entry in the table.
    ClearsEveryFrame,
    /// **A chip that fills its face before drawing its label.** R07's original defect.
    ChipFillsItsFace,
    /// **A chip that does not narrow**, whose label runs into its sibling's rectangle.
    ChipDoesNotNarrow,
    /// **A label that does not narrow.**
    ///
    /// Not one of ADR 0026's five — it is the sixth arm, and it is here because it is the only one
    /// of them that is **green at 300×80 and red at 120×40**, which is the whole reason the equality
    /// runs at two sizes. §21's scene 15 states the shape for `chart` at 60×20; this is it for
    /// `text` on the screen `text` is drawn on.
    LabelDoesNotNarrow,
    /// **A panel's top border drawn as one run with its title written over it.** The 15-cell
    /// instance, reached through [`crate::frame::defective::block_over_title`] rather than
    /// re-written, so the two builds stay one function with one boolean between them.
    BorderOverTheTitle,
    /// **A modal whose scrim is drawn as the four rectangles around the dialog.** The correct arm of
    /// the fourth instance.
    ScrimAroundTheDialog,
    /// **A modal whose scrim is filled under the dialog.** The fourth instance.
    ScrimUnderTheDialog,
}

impl Arm {
    /// Whether the arm washes the whole screen before it draws.
    const fn clears_the_screen(self) -> bool {
        matches!(self, Arm::Naive | Arm::ClearsEveryFrame)
    }

    /// Whether the arm washes a panel interior after `block` hands it over.
    const fn fills_the_interior(self) -> bool {
        matches!(self, Arm::Naive)
    }

    /// Whether the arm washes a chip's face before drawing its label.
    const fn fills_the_face(self) -> bool {
        matches!(self, Arm::Naive | Arm::ChipFillsItsFace)
    }

    /// Whether the arm stands a dialog up, and whether its scrim goes under it.
    const fn scrim(self) -> Option<bool> {
        match self {
            Arm::ScrimAroundTheDialog => Some(false),
            Arm::ScrimUnderTheDialog => Some(true),
            _ => None,
        }
    }
}

/// **What the screen turned out to be**, returned rather than printed, because a number only a
/// report prints is a number no gate can read.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Shape {
    /// How many widgets the three panels asked for.
    pub requested: usize,
    /// How many were drawn.
    pub visible: usize,
    /// How many fell off the bottom.
    pub dropped: usize,
    /// How many interactive regions the screen declared, counted by the draw.
    pub declared: usize,
    /// How many the runtime's hit index holds, counted by the frame. **Two counts and not one**:
    /// the first is what the screen believes and the second is what the runtime recorded, and a
    /// widget that collided with another would move the second and not the first.
    pub regions: usize,
    /// How many tab stops the frame holds.
    pub stops: usize,
    /// How many cells the three `block` calls handed over.
    pub handed_over: u64,
}

/// **The screen, one frame, drawn the way `arm` says.**
///
/// Generic over [`Ink`] so that a [`crate::counters::Tally`] and a [`Pen`] measure the shipped
/// drawing path rather than a copy of it — [`crate::ink`]'s whole argument, and the reason there is
/// no second implementation of this function for the gates to run over.
pub fn draw_into<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, arm: Arm, requested: usize) -> Shape {
    let theme = cx.theme();
    let body = theme.paint(Role::Body);
    let scrim = theme.paint(Role::Disabled);
    let screen = cx.area();
    let mut shape = Shape {
        requested: requested * PANELS as usize,
        ..Shape::default()
    };

    // **The screen clear that is a defect.** ADR 0026's largest instance, and it is
    // [`crate::app::defective::every_frame`] rather than a local wash so that the defect belongs to
    // the helper whose correct spelling is [`Clears`]. The correct arm has nothing for it to fix,
    // because the branches below cover all 24 000 cells between them.
    if arm.clears_the_screen() {
        crate::app::defective::every_frame(ink, cx);
    }

    let (header, rest) = rect::split_at_v(screen, 1);
    let (band, footer) = rect::split_at_v(rest, rest.h.saturating_sub(1));
    // **`text`, twice, and each declares its own region.** The chrome is clickable and the labels
    // below are not, which is [`TextOpts::interest`]'s whole reason: a label that took a hit entry
    // would put 222 more regions on this screen and swallow every click that landed on one.
    let _ = cx.with_key(u64::MAX, |cx| {
        text_into(
            ink,
            cx,
            header,
            HEADER,
            &clickable(Justify::Start, Role::Title, Role::Title),
        )
    });
    let _ = cx.with_key(u64::MAX - 1, |cx| {
        text_into(
            ink,
            cx,
            footer,
            FOOTER,
            &clickable(Justify::End, Role::Dim, Role::Dim),
        )
    });
    shape.declared += 2;

    let mut left = band;
    for panel in 0..PANELS {
        let (slot, next) = rect::split_at_h(left, band.w / PANELS);
        left = next;

        let opts = PanelOpts {
            border: if panel == 0 {
                Role::Focus
            } else {
                Role::Border
            },
            ..PanelOpts::default()
        };
        // **`panel`, and it declares its own region.** Ticket 09's stand-in declared the panel
        // *after* its rows, which puts the container in front of its children in a reverse-scanned
        // index; the component fixes the order once.
        let stood = cx.with_key(u64::MAX - 2 - u64::from(panel), |cx| {
            if arm == Arm::BorderOverTheTitle {
                panel_over_title(ink, cx, slot, TITLE, &opts)
            } else {
                panel_into(ink, cx, slot, TITLE, &opts)
            }
        });
        shape.declared += 1;
        let interior = stood.interior;
        shape.handed_over += u64::from(interior.w) * u64::from(interior.h);

        // **A panel that clears what `block` handed it.** The 22 200-cell instance `crate::form`
        // prices on its own screen; here it is one third of the naive twin.
        if arm.fills_the_interior() {
            wash(ink, cx, interior, body);
        }

        let fits = usize::from(interior.h);
        let visible = requested.min(fits);
        shape.visible += visible;
        shape.dropped += requested - visible;

        let declared = cx.with_key(u64::from(panel), |cx| {
            let mut rows = interior;
            let mut declared = 0usize;
            for i in 0..fits {
                let (line, below) = rect::split_at_v(rows, 1);
                rows = below;
                if i < visible {
                    declared += widget_row(ink, cx, arm, line, i);
                } else {
                    // **The tail is written by its owner**, which is the other half of §2's rule and
                    // what makes the equality below an equality over 24 000 cells rather than over
                    // the cells somebody happened to draw.
                    text_into(
                        ink,
                        cx,
                        line,
                        "",
                        &how(Justify::Start, Role::Body, Role::Body),
                    );
                }
            }
            declared
        });
        shape.declared += declared;
    }

    if let Some(under) = arm.scrim() {
        shape.declared += modal(ink, cx, screen, under, scrim);
    }

    shape
}

/// One row: a label, a chip, and either a button or a reading. Returns how many regions it declared.
///
/// **Every cell of the row is written exactly once**, by exactly one of the three, and the three
/// rectangles are a partition of the row because they come out of [`Rect::split_at_h`].
fn widget_row<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, arm: Arm, line: Rect, i: usize) -> usize {
    let (label, tail) = rect::split_at_h(line, line.w.saturating_sub(CHIP + ACTION));
    let (chip, action) = rect::split_at_h(tail, CHIP);

    // ── `text`: the row's label ──────────────────────────────────────────────────────────────────
    let label_opts = how(Justify::Start, Role::Body, Role::Body);
    if arm == Arm::LabelDoesNotNarrow {
        text_defective::text_that_does_not_narrow(
            ink,
            cx,
            label,
            LABELS[i % LABELS.len()],
            &label_opts,
        );
    } else {
        text_into(ink, cx, label, LABELS[i % LABELS.len()], &label_opts);
    }

    // ── `chip` ───────────────────────────────────────────────────────────────────────────────────
    //
    // **One key a widget and the component mints its own id inside it**, which is what keeps the
    // `#[track_caller]` on `chip` honest here: every chip on this screen shares a call site and no
    // two share a key.
    let value = VALUES[i % VALUES.len()];
    let chip_opts = ChipOpts::default();
    cx.with_key((i as u64) * 2, |cx| {
        if arm.fills_the_face() {
            // **R07's order**: the face is filled and the label is drawn into it. The screen is
            // right on every frame and the label's own cells are written twice, every frame.
            text_defective::chip_that_fills_its_face(ink, cx, chip, value, &chip_opts)
        } else if arm == Arm::ChipDoesNotNarrow {
            text_defective::chip_that_does_not_narrow(ink, cx, chip, value, &chip_opts)
        } else {
            chip_into(ink, cx, chip, value, &chip_opts)
        }
    });

    // ── `button` on an even row, `text` on an odd one ────────────────────────────────────────────
    if i.is_multiple_of(2) {
        cx.with_key((i as u64) * 2 + 1, |cx| {
            button_into(
                ink,
                cx,
                action,
                ACTIONS[i % ACTIONS.len()],
                &ButtonOpts::default(),
            )
        });
        2
    } else {
        text_into(
            ink,
            cx,
            action,
            UNITS[i % UNITS.len()],
            &how(Justify::End, Role::Dim, Role::Body),
        );
        1
    }
}

/// The scrim and the dialog. Returns how many regions it declared.
///
/// **The scrim is a complement and never a fill**. Drawn around the dialog it is four
/// rectangles whose union with the dialog is the screen; filled under it, it is one wash and the
/// dialog writes its own cells back on every frame that the dialog is not moving.
fn modal<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    screen: Rect,
    under: bool,
    scrim: Paint,
) -> usize {
    let dialog = dialog_at(screen);
    if under {
        wash(ink, cx, screen, scrim);
    } else {
        let (above, rest) = rect::split_at_v(screen, dialog.y as u16);
        let (middle, below) = rect::split_at_v(rest, dialog.h);
        let (left, right) = rect::split_at_h(middle, dialog.x as u16);
        let (_, right) = rect::split_at_h(right, dialog.w);
        wash(ink, cx, above, scrim);
        wash(ink, cx, below, scrim);
        wash(ink, cx, left, scrim);
        wash(ink, cx, right, scrim);
    }

    // The barrier is the panel's own region: the dialog swallows what the base pass would otherwise
    // hear, and a container declared **before** its children is what makes a reverse-scanned index
    // give the innermost widget.
    let stood = cx.with_key(u64::MAX - 32, |cx| {
        panel_into(
            ink,
            cx,
            dialog,
            DIALOG_TITLE,
            &PanelOpts {
                interest: Interest::CLICK,
                ..PanelOpts::default()
            },
        )
    });
    let interior = stood.interior;
    let mut rows = interior;
    let last = interior.h.saturating_sub(1);
    for r in 0..last {
        let (line, below) = rect::split_at_v(rows, 1);
        rows = below;
        text_into(
            ink,
            cx,
            line,
            LABELS[usize::from(r) % LABELS.len()],
            &how(Justify::Start, Role::Body, Role::Body),
        );
    }
    let (buttons, _) = rect::split_at_v(rows, 1);
    let (ok, cancel) = rect::split_at_h(buttons, buttons.w / 2);
    let mut declared = 0usize;
    for (n, (cells, word)) in [(ok, "confirm"), (cancel, "cancel")]
        .into_iter()
        .enumerate()
    {
        cx.with_key(u64::MAX - 16 - n as u64, |cx| {
            button_into(ink, cx, cells, word, &ButtonOpts::default())
        });
        declared += 1;
    }
    declared + 1
}

/// Where the dialog sits: centred, [`DIALOG`] cells.
pub fn dialog_at(screen: Rect) -> Rect {
    let w = DIALOG.0.min(screen.w);
    let h = DIALOG.1.min(screen.h);
    Rect::new(
        i32::from((screen.w - w) / 2),
        i32::from((screen.h - h) / 2),
        w,
        h,
    )
}

/// The three roles a label is drawn with, as one value. **No region** — see
/// [`TextOpts::interest`].
const fn how(justify: Justify, role: Role, pad: Role) -> TextOpts {
    TextOpts {
        justify,
        role,
        pad,
        interest: None,
    }
}

/// [`how`], for the two pieces of chrome that are clickable. The header and the footer are `text`
/// and they take a hit entry; the 222 labels below them do not, which is the difference between 338
/// regions and 560.
const fn clickable(justify: Justify, role: Role, pad: Role) -> TextOpts {
    TextOpts {
        justify,
        role,
        pad,
        interest: Some(Interest::CLICK),
    }
}

/// Write every cell of `cells` as a space. **The verb every fill-shaped defect on this screen is
/// made of**, and never `Ctx::fill` — see [`crate::ink`]: `fill` returns `()`, so a filled cell is
/// modelled rather than reported and the pair stops being a comparison between two sources.
fn wash<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, cells: Rect, st: Paint) {
    if cells.is_empty() {
        return;
    }
    let x = cells.x;
    for r in 0..cells.h {
        ink.run(cx, x, cells.y + i32::from(r), " ", cells.w, st);
    }
}

// ── the painters ─────────────────────────────────────────────────────────────────────────────────

/// **The screen that ships**, as a [`Painter`].
pub fn correct(pen: &mut Pen, cx: &mut Ctx<'_, '_>, fx: &Fixture) {
    let _ = draw_into(pen, cx, Arm::Correct, fx.len());
}

/// **The naive twin, and it is kept.**
///
/// A screen clear, a fill under every panel interior and a face filled under every chip label. It is
/// the reference the correct build is proved **equal to** — deleting it deletes the argument, which
/// is §21's own rule — and it is what stops the damage count being gamed: a build that merely drew
/// less would score 0 re-damaged cells by drawing nothing, and three separate defects on this map
/// did exactly that.
///
/// ```
/// use vitui_components::dense::{self, Arm};
///
/// // Named by path, so deleting it is a compile error here and not a quiet loss of the reference.
/// let twin: vitui_components::runner::Painter = vitui_components::dense::naive;
/// let redamage = dense::steady(Arm::Naive, 3);
/// assert!(redamage.per_frame > dense::steady(Arm::Correct, 3).per_frame);
/// let _ = twin;
/// ```
pub fn naive(pen: &mut Pen, cx: &mut Ctx<'_, '_>, fx: &Fixture) {
    let _ = draw_into(pen, cx, Arm::Naive, fx.len());
}

/// The screen with `cx.clear(body)` at the top of every frame.
pub fn clears_every_frame(pen: &mut Pen, cx: &mut Ctx<'_, '_>, fx: &Fixture) {
    let _ = draw_into(pen, cx, Arm::ClearsEveryFrame, fx.len());
}

/// The screen whose chips fill their face before drawing their label.
pub fn chip_fills_its_face(pen: &mut Pen, cx: &mut Ctx<'_, '_>, fx: &Fixture) {
    let _ = draw_into(pen, cx, Arm::ChipFillsItsFace, fx.len());
}

/// The screen whose chips do not narrow.
pub fn chip_does_not_narrow(pen: &mut Pen, cx: &mut Ctx<'_, '_>, fx: &Fixture) {
    let _ = draw_into(pen, cx, Arm::ChipDoesNotNarrow, fx.len());
}

/// The screen whose labels do not narrow. **Green at 300×80 and red at 120×40.**
pub fn label_does_not_narrow(pen: &mut Pen, cx: &mut Ctx<'_, '_>, fx: &Fixture) {
    let _ = draw_into(pen, cx, Arm::LabelDoesNotNarrow, fx.len());
}

/// The screen whose panels write their top border over their own title.
pub fn border_over_the_title(pen: &mut Pen, cx: &mut Ctx<'_, '_>, fx: &Fixture) {
    let _ = draw_into(pen, cx, Arm::BorderOverTheTitle, fx.len());
}

/// The screen with a modal up, its scrim drawn as the four rectangles around the dialog.
pub fn scrim_around_the_dialog(pen: &mut Pen, cx: &mut Ctx<'_, '_>, fx: &Fixture) {
    let _ = draw_into(pen, cx, Arm::ScrimAroundTheDialog, fx.len());
}

/// The screen with a modal up, its scrim filled under the dialog.
pub fn scrim_under_the_dialog(pen: &mut Pen, cx: &mut Ctx<'_, '_>, fx: &Fixture) {
    let _ = draw_into(pen, cx, Arm::ScrimUnderTheDialog, fx.len());
}

/// The painter for an arm. **A `match` rather than a method on [`Arm`]**, so that adding an arm
/// without a painter is a non-exhaustive match and not a silently unreachable screen.
pub const fn painter(arm: Arm) -> Painter {
    match arm {
        Arm::Correct => correct,
        Arm::Naive => naive,
        Arm::ClearsEveryFrame => clears_every_frame,
        Arm::ChipFillsItsFace => chip_fills_its_face,
        Arm::ChipDoesNotNarrow => chip_does_not_narrow,
        Arm::LabelDoesNotNarrow => label_does_not_narrow,
        Arm::BorderOverTheTitle => border_over_the_title,
        Arm::ScrimAroundTheDialog => scrim_around_the_dialog,
        Arm::ScrimUnderTheDialog => scrim_under_the_dialog,
    }
}

// ── playing it ───────────────────────────────────────────────────────────────────────────────────

/// **The fixture's content is a count**: one row per widget a panel is asked for.
///
/// The labels are static — a label whose length varied with the step would move the write count for
/// a reason that is not the construction — so what a step carries is *how many widgets are
/// standing*, and [`Fixture::shrunk_to`] is then §21's own spelling of the gesture this screen
/// plays: **less content, in the same rectangle**. The rectangle is [`W`]×[`H`] and
/// [`crate::runner::play`] refuses a step that moves it.
pub fn widgets(n: usize) -> Fixture {
    Fixture::of(W, H, vec![String::from("widget"); n])
}

/// The dense screen, one step, at the size §2 and §21 both state it at.
pub fn screen() -> Vec<Fixture> {
    vec![widgets(REQUESTED)]
}

/// **Play one arm at `Compact`.** Density is theme data and it changes rectangles, so 338 is a
/// `Compact` figure and every measurement here names the density rather than taking
/// `Density::default()`.
pub fn play(arm: Arm, steps: &[Fixture]) -> Run {
    play_at(Density::Compact, painter(arm), steps)
}

/// **The shape the screen turned out to be**, measured by drawing it rather than by repeating the
/// arithmetic beside the drawing code — a `Shape` computed twice would agree with a wrong one.
pub fn shape(arm: Arm, size: (u16, u16), requested: usize) -> Shape {
    let mut driver = crate::runner::driver_at(size.0, size.1, Density::Compact);
    let mut pen = Pen::new(size.0, size.1);
    let mut shape = Shape::default();
    driver.frame(|cx| shape = draw_into(&mut pen, cx, arm, requested));
    let frame = driver.inspect();
    shape.regions = frame.hits().len();
    shape.stops = frame.stop_count();
    shape
}

/// **The metric row §21 reports every scene in**, for one arm at 300×80.
///
/// `us / marked / writes / verbs / regions / stops / allocations`, and `marked` prints
/// `unreachable` — printing `0` there would be §21's first refinement arriving as a column.
pub fn row(arm: Arm, scene: &'static str, allocations: crate::counters::Allocations) -> MetricRow {
    play(arm, &screen()).row(scene, allocations)
}

/// **The correct build against the naive twin, cell for cell, at one size.**
pub fn equality(size: (u16, u16)) -> Diff {
    compare_at(
        Density::Compact,
        correct,
        naive,
        &[widgets(REQUESTED).resized(size.0, size.1)],
    )
}

/// **What one arm re-damages, frame after frame.**
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Redamage {
    /// How many frames were drawn.
    pub frames: u32,
    /// Rect changed on the **first** frame, when the screen appears. Not re-damage.
    pub first: u64,
    /// Rect changed on every frame after the first, summed.
    pub steady: u64,
    /// Rect changed on each steady frame, which is constant or this is not a steady state.
    pub per_frame: u64,
}

/// **Draw `arm` for `frames` frames into one surface and count what changes.**
///
/// The surface is carried across frames by [`Pen::over`], which is the whole mechanism: a fresh
/// [`Pen`] a frame reports every cell as a first paint for ever, because re-damage is a relation
/// between two frames and there would be nothing to relate it to.
///
/// # Panics
///
/// Panics on fewer than two frames, and panics when a steady frame's count is not the frame
/// before's — a screen that is not moving and does not settle is a defect this measurement cannot
/// summarise.
pub fn steady(arm: Arm, frames: u32) -> Redamage {
    run_frames(frames, |_, pen, cx| {
        let _ = draw_into(pen, cx, arm, REQUESTED);
    })
}

/// **The application loop §2's clearing rule is about, drawn for `frames` frames.**
///
/// Components ticket 10, criterion 5. [`Clears`] at the top of every frame and the correct screen
/// under it: the first frame clears, and every frame after it re-damages **0** — against
/// [`CLEARED_EVERY_FRAME`]'s 9 024 for the same call written without the state.
///
/// The resize half is [`crate::app`]'s, because [`crate::runner::play`] refuses a step that moves
/// the rectangle and this loop holds one driver: a resize needs two, and that is
/// `app::tests::one_clear_a_size_and_never_a_third`.
pub fn steady_clearing(frames: u32) -> Redamage {
    let mut clears = Clears::new();
    run_frames(frames, move |_, pen, cx| {
        clears.frame_into(pen, cx);
        let _ = draw_into(pen, cx, Arm::Correct, REQUESTED);
    })
}

/// **The modal's re-damage, with the base pass drawn once.**
///
/// # The substitution, named rather than hidden
///
/// A scrim and a dialog stand on their **own layer** over a base pass that has not changed
/// (`Ctx::overlay`, runtime 13). This instrument has one pass and one surface: a `Ctx::child` moves
/// the origin, so an overlay body's writes would land in a different coordinate system from the base
/// pass's and [`Pen`] would union the two — the same collision `crate::frame` measures at 124 false
/// double writes. So the first frame draws the screen and the modal, and every frame after it draws
/// the modal alone, which is what a layer over an unchanged base costs.
///
/// The substitution is on **both** arms, so it is not on the side of either: what separates them is
/// where the scrim goes.
pub fn modal_steady(under: bool, frames: u32) -> Redamage {
    run_frames(frames, move |n, pen, cx| {
        if n == 0 {
            let _ = draw_into(pen, cx, Arm::Correct, REQUESTED);
        }
        let scrim = cx.theme().paint(Role::Disabled);
        let _ = modal(pen, cx, cx.area(), under, scrim);
    })
}

/// The loop both measurements share: one surface, one driver, `end_frame` where `Driver::frame`
/// applies a deferred award, and the steady count asserted constant.
fn run_frames(frames: u32, mut paint: impl FnMut(u32, &mut Pen, &mut Ctx<'_, '_>)) -> Redamage {
    assert!(frames >= 2, "re-damage is a relation between two frames");
    let mut driver = crate::runner::driver_at(W, H, Density::Compact);
    let mut canvas = Canvas::new(W, H);
    let mut first = 0u64;
    let mut steady = 0u64;
    let mut per_frame: Option<u64> = None;
    for n in 0..frames {
        let mut pen = Pen::over(canvas);
        driver.frame(|cx| paint(n, &mut pen, cx));
        pen.end_frame();
        canvas = pen.into_canvas();
        let changed = canvas.take_repaints();
        if n == 0 {
            first = changed;
        } else {
            steady += changed;
            let seen = *per_frame.get_or_insert(changed);
            assert_eq!(
                seen, changed,
                "frame {n} changed {changed} cells and the frame before it {seen}. A screen that is \
                 not moving is a steady state, and a steady state that is not constant is a defect \
                 this measurement cannot summarise"
            );
        }
    }
    Redamage {
        frames,
        first,
        steady,
        per_frame: per_frame.unwrap_or_default(),
    }
}

// ── the four subjects, and the scan that says whether they are here ──────────────────────────────

/// **The four components this screen is a screen of.** Spec §17's freeze gives each of them
/// `constructions: 1`, and **components ticket 10 declared all four**.
pub const SUBJECTS: [&str; 4] = ["text", "chip", "button", "panel"];

/// Where each of [`SUBJECTS`] is declared, as `(module file, the declaration)`.
///
/// The home is the freeze's, joined through [`crate::Family`]: `text` and `chip` are F1 and are
/// homed in `text.rs`, `button` is F6 and `panel` is F2. A component is
/// `fn(&mut Ctx, Rect, …) -> Response` (spec §1, rule 1) — with `Rect` in `Rect`'s place, which is
/// [`vitui_runtime::layout::rect`]'s argument — so the thing to look for is a public function of the component's
/// own name in its own family's module.
///
/// **`pub`, because ticket 10's own criterion 2 reads it**: *every one of them routes its writing
/// through `fit` and `block`, and none contains a fill-then-draw order* is a statement about these
/// four bodies, and a second list of them would be a second thing to keep in step.
pub const DECLARATIONS: [(&str, &str); 4] = [
    ("text.rs", "pub fn text("),
    ("text.rs", "pub fn chip("),
    ("input.rs", "pub fn button("),
    ("structure.rs", "pub fn panel("),
];

/// **Which of [`SUBJECTS`] this crate actually declares. Today: all four.**
///
/// A source scan and not a `use`, because *the item does not exist* has no expression: a
/// `compile_fail` fence would pass today and pass again the day somebody renames the module, which
/// is the diagnostic collision this crate's twins are built around. What a scan can do is fire in
/// both directions, and `tests::the_subject_scan_finds_a_declaration_when_there_is_one` is that
/// half.
pub fn subjects_declared() -> Vec<&'static str> {
    let mut out = Vec::new();
    for (subject, (file, declaration)) in SUBJECTS.into_iter().zip(DECLARATIONS) {
        let path = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).join(file);
        let source = std::fs::read_to_string(&path).unwrap_or_default();
        if declares(&source, declaration) {
            out.push(subject);
        }
    }
    out
}

/// Whether `source` carries `needle` on a line that is not a comment. One predicate for the scan and
/// for both of its negative halves, which is what makes *fires in both directions* mean something:
/// a hostile fixture run through a second copy of the logic proves the copy and not the gate.
///
/// **`pub(crate)`, because ticket 10's criterion 2 scans the same four bodies for what must *not* be
/// in them.** One predicate, one definition of *a line that is not a comment*.
pub(crate) fn declares(source: &str, needle: &str) -> bool {
    source
        .lines()
        .map(str::trim)
        .any(|line| !line.starts_with("//") && line.contains(needle))
}

/// **Whether the screen stands on its subjects, as a verdict rather than as a sentence.**
///
/// [`Verdict::of`] refuses vacuity in its constructor, which is what makes this the right shape: the
/// population is the four subjects, not the scene list, so *no component exists* was `Unmet` over
/// four rather than `Met` over nothing — and now that all four are declared it is `Met` over four
/// rather than `Met` over an empty list.
///
/// # This is the distinction criterion 7 was about, and it is kept in both directions
///
/// A scene that fails because it is unimplemented and a scene that fails because the code is wrong
/// are the same failure unless the message separates them. [`owed_message`] is the message and it
/// is still live: it names the missing subjects, the file each is declared in, and the ticket. What
/// changed is which side of it the crate is on, not whether the sentence exists —
/// `tests::the_waiting_message_still_says_which_failure_it_is` hands it an empty declaration list
/// and reads what comes out.
pub fn standing() -> Verdict {
    let declared = subjects_declared();
    Verdict::of(
        SUBJECTS.len(),
        SUBJECTS.len() - declared.len(),
        "the dense screen's four components are not declared in this crate, so what stands on the \
         screen is their construction — `fit`, `block` and `press` — and not the components \
         themselves. The screen, its 338 regions, its equality against the naive twin and all five \
         re-damage instances are measured and green; what is missing is the subject",
        "components 10",
    )
}

/// **The sentence a scene waiting for its subject fails with**, or `None` when every subject is
/// declared.
///
/// Separated from [`assert_stands_up`] because the message is the mechanism and the panic is only
/// how it is delivered: with all four subjects now declared, a `#[should_panic]` test can no longer
/// reach it, and a message no test can read is a message that rots. This takes the declaration list
/// as an argument, so the hostile case is one call away for ever.
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
         components are undeclared — {}. This is not a defect in the screen. The screen is drawn, \
         its regions are counted, its equality against the naive twin holds at both sizes and every \
         one of ADR 0026's five re-damage instances is measured — see `crate::dense::tests`. \
         Inverted by `components 10`",
        owed.len(),
        SUBJECTS.len(),
        owed.join(", "),
    ))
}

/// **Fail with the subjects that are missing, the files they belong in, and the ticket.**
///
/// # Panics
///
/// Panics while any of [`SUBJECTS`] is undeclared. **It no longer does** — components 10 declared
/// all four — and the call is kept rather than deleted because it is what a later ticket that moves
/// a component out of its family module will hit, with a sentence that names the file rather than a
/// scene that has quietly stopped being about anything.
pub fn assert_stands_up(scene: &str) {
    if let Some(message) = owed_message(&subjects_declared(), scene) {
        panic!("{message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::counters::Allocations;
    use crate::runner::{Painter, compare_at};

    /// Every arm, so a sweep does not have to spell them.
    const ARMS: [Arm; 9] = [
        Arm::Correct,
        Arm::Naive,
        Arm::ClearsEveryFrame,
        Arm::ChipFillsItsFace,
        Arm::ChipDoesNotNarrow,
        Arm::LabelDoesNotNarrow,
        Arm::BorderOverTheTitle,
        Arm::ScrimAroundTheDialog,
        Arm::ScrimUnderTheDialog,
    ];

    /// **Criterion 2: 338 interactive regions at 300×80, and the metric row.**
    ///
    /// Two counts and not one — what the draw believes it declared and what the runtime's hit index
    /// actually holds. They agree, and a widget that collided with another would move the second
    /// without moving the first, which is ADR 0027's free detector: 110 of 338 widgets were inert on
    /// the first screen written for components ticket 01 and the screen rendered pixel for pixel
    /// correctly.
    #[test]
    fn the_dense_screen_stands_three_hundred_and_thirty_eight_regions_at_three_hundred_by_eighty() {
        let shape = shape(Arm::Correct, (W, H), REQUESTED);
        assert_eq!(shape.declared, REGIONS, "the screen's own count");
        assert_eq!(shape.regions, REGIONS, "the runtime's hit index");
        assert_eq!(
            shape.stops, STOPS,
            "a chip and a button are tab stops; a panel is not"
        );
        assert_eq!(shape.requested, REQUESTED * PANELS as usize);
        assert_eq!(
            shape.dropped, 0,
            "nothing falls off the bottom at `Compact`"
        );
        assert_eq!(shape.visible, shape.requested);

        // And the arithmetic behind the number, so a change in the geometry reads as a change in
        // the geometry: three panels, one region each, a chip a row and a button every second row.
        let rows = shape.visible / PANELS as usize;
        assert_eq!(rows, 74);
        assert_eq!(2 + PANELS as usize * (1 + rows + rows.div_ceil(2)), REGIONS);
    }

    /// **Criterion 2's other half: the metric row, with `marked=unreachable`.**
    #[test]
    fn the_dense_screen_reports_the_metric_row_and_never_prints_marked_zero() {
        let row = row(
            Arm::Correct,
            "the dense screen, 300x80",
            Allocations::over(1, 0),
        );
        let line = row.line();
        assert!(line.contains("marked=unreachable"), "{line}");
        assert!(!line.contains("marked=0"), "{line}");
        for column in [
            "us",
            "marked=",
            "writes=",
            "verbs=",
            "regions=",
            "stops=",
            "allocations=",
        ] {
            assert!(
                line.contains(column),
                "the row is missing `{column}`: {line}"
            );
        }
        let counters = row.counters();
        use crate::counters::Counter;
        assert_eq!(counters.get(Counter::Writes).get(Counter::Writes), WRITES);
        assert_eq!(
            counters.get(Counter::Regions).get(Counter::Regions),
            REGIONS as u64
        );
        assert_eq!(
            counters.get(Counter::TabStops).get(Counter::TabStops),
            STOPS as u64
        );
        assert_eq!(counters.reachable(), 8, "eight of §20's nine");
    }

    /// **Every cell of the screen written exactly once** — §2's two equalities, both directions, on
    /// the screen the rule is stated about.
    #[test]
    fn the_correct_screen_is_a_partition_of_twenty_four_thousand_cells() {
        for (size, cells) in [((W, H), SCREEN), (NARROW, NARROW_SCREEN)] {
            let run = play(Arm::Correct, &[widgets(REQUESTED).resized(size.0, size.1)]);
            let tally = run.tally();
            assert_eq!(
                tally.reported(),
                tally.writes(),
                "{size:?}: a write the engine did not report"
            );
            assert_eq!(
                tally.asked(),
                tally.reported(),
                "{size:?}: a verb left its own rectangle and the clip discarded it"
            );
            assert_eq!(
                tally.writes(),
                tally.distinct(),
                "{size:?}: {} cells written twice",
                tally.writes() - tally.distinct()
            );
            assert_eq!(
                tally.distinct(),
                cells,
                "{size:?}: the screen does not cover itself, and the equality below would then be \
                 an equality about the cells somebody happened to draw"
            );
        }
    }

    /// **Criterion 4: the same screen drawn naive and correct, 0 cells apart, at both sizes.**
    ///
    /// §2's own sentence — *the two screens are the same screen: 0 of 24 000 cells differ, at 300×80
    /// and at 120×40* — and the second half of the ticket's own argument: a build that merely drew
    /// less would score 0 re-damaged cells by drawing nothing, so the equality is what makes the
    /// damage count mean something.
    #[test]
    fn the_same_screen_drawn_naive_and_correct_is_zero_cells_apart_at_both_sizes() {
        for (size, cells) in [((W, H), SCREEN), (NARROW, NARROW_SCREEN)] {
            let diff = equality(size);
            assert_eq!(diff.over, size);
            assert!(
                diff.clean(),
                "{size:?}: {diff}. The naive twin is the reference this build is proved equal to"
            );
            // And neither arm is clean by drawing nothing.
            for arm in [Arm::Correct, Arm::Naive] {
                let run = play(arm, &[widgets(REQUESTED).resized(size.0, size.1)]);
                assert_eq!(run.canvas().written() as u64, cells, "{size:?} {arm:?}");
            }
        }
    }

    /// **Criterion 3: the naive twin is declared in this file, and it is named by path.**
    ///
    /// §21's rule — *it is the reference the correct build is proved equal to, and deleting it
    /// deletes the argument* — as a gate rather than as a comment. The doctest on [`naive`] holds
    /// the public path; this holds the declaration, which is how a kept fixture actually goes away:
    /// somebody makes it private "because only the test uses it" and the path stops existing while
    /// the function still does.
    #[test]
    fn the_naive_twin_is_declared_in_this_file_and_named_by_path() {
        let path = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src/dense.rs"));
        let source = std::fs::read_to_string(&path).expect("this file");
        let declaration = ["pub fn ", "naive("].concat();
        let by_path = ["vitui_components::dense::", "naive"].concat();
        assert!(
            declares(&source, &declaration),
            "the naive twin is gone from `src/dense.rs`, and the equality above is now an equality \
             between the correct build and itself"
        );
        assert!(
            source.contains(&by_path),
            "no doctest names the twin by path, so making it private would not fail anything"
        );
        // The other direction, through the same predicate.
        assert!(!declares(&format!("// {declaration}"), &declaration));
    }

    /// **Criterion 6, and the point of the whole ticket: all five instances, each measured.**
    ///
    /// ADR 0026's table, standing on a screen. The correct arm re-damages **0** on every steady
    /// frame and each defective arm re-damages its own row, so ticket 10 can be proved against a
    /// screen rather than against a review.
    ///
    /// Every one of the five is invisible to the counters that are not this one: the defective build
    /// is not slower to write, it touches at least as many cells, and the screen it produces is
    /// correct on every frame.
    #[test]
    fn all_five_re_damage_instances_are_measured_on_this_screen() {
        assert_eq!(
            steady(Arm::Correct, 4).per_frame,
            0,
            "a screen that is not moving re-damages nothing, and that is the number every row below \
             is a difference from"
        );
        for (arm, expected) in [
            (Arm::ClearsEveryFrame, CLEARED_EVERY_FRAME),
            (Arm::ChipFillsItsFace, CHIP_FILLED_FACE),
            (Arm::ChipDoesNotNarrow, CHIP_NOT_NARROWED),
            (Arm::BorderOverTheTitle, BORDER_OVER_TITLE_SCREEN),
        ] {
            let seen = steady(arm, 4);
            assert_eq!(
                seen.per_frame, expected,
                "{arm:?}: the instance's magnitude moved. It is a count over a deterministic \
                 screen, so it moved because the construction did"
            );
            assert_eq!(seen.steady, expected * u64::from(seen.frames - 1));
            assert_eq!(
                seen.first, SCREEN,
                "{arm:?}: the first frame paints the screen, which is not re-damage"
            );
        }

        // The fourth instance, whose pair is the two ways of drawing one scrim.
        let under = modal_steady(true, 4);
        let around = modal_steady(false, 4);
        assert_eq!(under.per_frame, SCRIM_UNDER);
        assert_eq!(
            around.per_frame, 0,
            "a scrim drawn as the four rectangles around the dialog re-damages nothing"
        );

        // **The twin is not the sum of its three instances.** Re-damage counts distinct cells, and a
        // screen clear already changes every cell the other two change, so the three nest.
        let twin = steady(Arm::Naive, 4);
        assert_eq!(twin.per_frame, NAIVE_TWIN);
        assert_eq!(
            twin.per_frame,
            steady(Arm::ClearsEveryFrame, 4).per_frame,
            "the twin costs the largest of its three instances and not their total, which is what \
             stops a report adding ADR 0026's five rows together"
        );
        assert!(twin.per_frame < CLEARED_EVERY_FRAME + CHIP_FILLED_FACE);

        // **The 15-cell instance, per panel**, which is the unit ADR 0026 states it in and the unit
        // components ticket 06 reproduced it in.
        assert_eq!(
            BORDER_OVER_TITLE_SCREEN / u64::from(PANELS),
            BORDER_OVER_TITLE
        );
        assert_eq!(
            u64::from(vitui_runtime::layout::text::width(TITLE)),
            BORDER_OVER_TITLE,
            "the number *is* the title's width, which is why it reproduces and the others do not"
        );
    }

    /// **The defective arms look healthier by every counter that is not this one.**
    ///
    /// *A gate nobody has watched fail is not a gate*, and this is the other direction of it: a
    /// defect nobody has watched **pass** the existing gates is a defect somebody will argue is
    /// already covered. Three of the four axes on this map were faster and two marked less.
    #[test]
    fn every_defective_arm_covers_at_least_as_many_cells_as_the_correct_one() {
        let correct = play(Arm::Correct, &screen());
        for arm in ARMS {
            if arm == Arm::Correct {
                continue;
            }
            let run = play(arm, &screen());
            assert!(
                run.tally().distinct() >= correct.tally().distinct(),
                "{arm:?}: the defective screen touches fewer cells than the correct one, so *no \
                 cell never* would have caught it and the pair is not the only detector"
            );
        }
        // And the one that is cheaper in verbs, which is the shape ticket 06 found: a border run
        // written across its own title is **one verb fewer** a panel.
        let broken = play(Arm::BorderOverTheTitle, &screen());
        assert!(
            broken.tally().verbs() < correct.tally().verbs(),
            "the 15-cell instance is supposed to be the cheap-looking one"
        );
    }

    /// **Criterion 4's reason for two sizes: a defect green at 300×80 and red at 120×40.**
    ///
    /// A label that is not truncated fits its column at 300 and does not at 120, so the wide
    /// equality is clean and the narrow one is not. §13's overlap is the same shape one component
    /// over, and §21 records it as *green at 300×80 and red at 60×20*.
    ///
    /// # The equality sees one half of this defect and the re-damage count sees the other
    ///
    /// What the narrow equality actually catches is the **missing ellipsis** — one wrong cell a
    /// truncated row, plus the long label's tail. The *overrun itself* is invisible to it, because
    /// the widget the label ran into draws over those cells afterwards and the final surface is
    /// right. That half is [`CHIP_NOT_NARROWED`]'s: a cell written twice a frame with two different
    /// values is re-damage and not a wrong picture. **Neither instrument sees both halves**, which
    /// is why ADR 0026's table and §21's equality are two gates and not one.
    #[test]
    fn a_label_that_does_not_narrow_is_green_at_three_hundred_and_red_at_a_hundred_and_twenty() {
        let wide = compare_at(
            Density::Compact,
            correct,
            label_does_not_narrow as Painter,
            &screen(),
        );
        assert!(
            wide.clean(),
            "green at 300x80, which is the half that makes the narrow size necessary: {wide}"
        );

        let narrow = compare_at(
            Density::Compact,
            correct,
            label_does_not_narrow as Painter,
            &[widgets(REQUESTED).resized(NARROW.0, NARROW.1)],
        );
        assert!(
            !narrow.clean(),
            "red at 120x40, or the two sizes are measuring one thing twice"
        );
        assert!(
            narrow.rows > 0 && narrow.cells > 0,
            "{narrow}: a defect that is red must be red about something"
        );
    }

    /// **Criterion 5: content shrinking inside a rectangle that does not move.**
    ///
    /// §21 refuses to bank *the surface after a shrink equals a freshly built one* written against a
    /// terminal resize, because a fresh rectangle has nowhere for the residue to survive — that
    /// spelling tests the resize path and not the defect. So the shrink here is
    /// [`Fixture::shrunk_to`], the rectangle is 300×80 on every step, and
    /// [`crate::runner::play`] refuses a step that moves it.
    #[test]
    fn the_narrow_screen_shrinks_its_content_inside_a_rectangle_that_does_not_move() {
        let full = widgets(REQUESTED);
        let shrunk = full.shrunk_to(11);
        assert_eq!(full.size(), shrunk.size(), "the rectangle does not move");

        // The residue: a surface that has drawn the full screen and then the shrunk one is the same
        // surface as one that has only ever drawn the shrunk one.
        let after = play(Arm::Correct, &[full.clone(), shrunk.clone()]);
        let fresh = play(Arm::Correct, std::slice::from_ref(&shrunk));
        let diff = after.canvas().diff(fresh.canvas());
        assert!(
            diff.clean(),
            "{diff}: eleven widgets a panel left sixty-three rows of the previous screen standing"
        );

        // And it really did shrink: fewer regions, and the tail is still written by somebody.
        let dense = shape(Arm::Correct, (W, H), REQUESTED);
        let sparse = shape(Arm::Correct, (W, H), 11);
        assert_eq!(dense.regions, REGIONS);
        assert_eq!(
            sparse.regions,
            2 + PANELS as usize * (1 + 11 + 11usize.div_ceil(2))
        );
        assert!(sparse.regions < dense.regions);
        assert_eq!(
            after.tally().distinct(),
            SCREEN,
            "the tail is nobody's stale rows"
        );

        // A step that moves the rectangle is refused, which is where the resize spelling would have
        // to go and is why it cannot arrive here by accident.
        let moved = std::panic::catch_unwind(|| {
            play(Arm::Correct, &[full.clone(), full.resized(W - 1, H)])
        });
        assert!(moved.is_err(), "a step that resizes is refused");
    }

    /// **The narrow size is a different construction and not a clipped one.**
    ///
    /// Criterion 1's *narrow* half: at 120 columns a label column is fourteen cells and every label
    /// but three is truncated, so `fit`'s one-cell ellipsis rule is what is standing between the
    /// label and its neighbour. The rule is `crate::text`'s and it is asserted there; what this adds
    /// is that the screen actually reaches the width where it fires.
    #[test]
    fn at_a_hundred_and_twenty_columns_the_label_column_truncates_and_the_ellipsis_is_one_cell() {
        use vitui_runtime::layout::text::width;
        let interior_w = NARROW.0 / PANELS - 2 - 2;
        let label_w = interior_w - CHIP - ACTION;
        assert_eq!(label_w, 14, "a `Compact` panel at 120 columns");
        let truncated = LABELS.iter().filter(|l| width(l) > label_w).count();
        assert_eq!(
            truncated, 5,
            "five of the eight labels truncate at 120 columns and three do not, which is what makes \
             the narrow size a question about `fit` rather than a question about clipping"
        );
        // Wide, the same labels fit, which is what makes the two sizes two questions.
        let wide_label_w = W / PANELS - 4 - CHIP - ACTION;
        assert!(LABELS.iter().all(|l| width(l) <= wide_label_w));

        let shape = shape(Arm::Correct, NARROW, REQUESTED);
        assert_eq!(shape.regions, NARROW_REGIONS);
        assert!(shape.dropped > 0, "the panels are asked for more than fits");
    }

    /// **Components ticket 10: the screen stands on its four subjects, and the verdict says so.**
    ///
    /// The exact set, in both directions: four subjects, all four declared, and the verdict is `Met`
    /// over four rather than `Met` over nothing — which is [`Verdict::of`]'s vacuity refusal doing
    /// the one job it was written for on the day the population stopped being empty.
    ///
    /// **This test is the inversion of `the_dense_screen_is_red_because_its_four_components_are_not_
    /// declared`**, which was pinned red by ticket 09 and named by `crate::gates::REGISTER`'s row 61
    /// and by `crate::scenes`'s three standings. Changing it back is a deliberate edit in all three
    /// places.
    #[test]
    fn the_dense_screen_stands_on_its_four_declared_components() {
        assert_eq!(
            subjects_declared(),
            SUBJECTS.to_vec(),
            "a component of the dense screen has stopped being declared where the freeze homes it. \
             That is not a defect in the screen: `crate::dense::DECLARATIONS` names the file and \
             the signature each of the four is looked for at"
        );
        let verdict = standing();
        assert!(verdict.met());
        match verdict {
            Verdict::Met { over } => assert_eq!(over, 4),
            Verdict::Unmet { over, failing, .. } => {
                unreachable!("{failing} of {over} undeclared, which the assertion above caught")
            }
        }
        // And the scene does not panic any more, which is the whole ticket in one call.
        assert_stands_up("the dense screen, 300x80, 338 regions");
    }

    /// **Criterion 5: the application clears once, and a steady frame re-damages 0 rather than
    /// 9 024.**
    ///
    /// §2's own rule — *the correct build clears once, on its first frame and on a resize* — on the
    /// screen the 9 024 was measured on. The first frame paints all 24 000 cells and every frame
    /// after it changes **nothing**, which is the same number [`Arm::Correct`] posts without
    /// clearing at all; the clear costs nothing on a screen that covers itself, and it is what makes
    /// one that does not the same screen.
    ///
    /// The resize half needs two drivers and lives in `crate::app::tests::one_clear_a_size_and_
    /// never_a_third`, because [`crate::runner::play`] refuses a step that moves the rectangle.
    #[test]
    fn clearing_once_re_damages_nothing_and_clearing_every_frame_costs_nine_thousand_cells() {
        let once = steady_clearing(4);
        assert_eq!(
            once.per_frame, 0,
            "a screen that clears once and then covers itself re-damages nothing"
        );
        assert_eq!(once.first, SCREEN, "the first frame paints the screen");
        assert_eq!(once.steady, 0);

        // The same call written without the state, which is ADR 0026's largest row.
        let every = steady(Arm::ClearsEveryFrame, 4);
        assert_eq!(every.per_frame, CLEARED_EVERY_FRAME);
        assert_eq!(
            every.per_frame,
            once.per_frame + CLEARED_EVERY_FRAME,
            "the difference between once and every frame *is* the instance"
        );

        // And the clear is a `Clears`, watched clearing exactly once over those frames.
        let mut clears = Clears::new();
        let mut driver = crate::runner::driver_at(W, H, Density::Compact);
        for _ in 0..4 {
            driver.frame(|cx| {
                clears.frame_into(&mut crate::counters::Tally::new(), cx);
            });
        }
        assert_eq!(clears.cleared(), 1, "four frames, one clear");
    }

    /// **Criterion 2: every primitive routes its writing through `fit` or `block`, and not one of
    /// them names a fill.**
    ///
    /// Two halves, and neither is the other.
    ///
    /// The **lexical** half is this one: `Ctx::fill` returns `()`, so a cell it writes is modelled
    /// rather than reported and both sides of `writes == reported` become this crate's own
    /// arithmetic — [`crate::ink`]'s header, and the reason the trait has `run` and not `fill`. A
    /// primitive that reached for it would take its own writing out of the instrument's sight, which
    /// no counter can then object to. So the scan is over the four bodies and everything they reach.
    ///
    /// The **behavioural** half — *no fill-then-draw order* — is `writes == distinct`, and it is in
    /// each component's own module, swept over widths and watched firing on the defective arms. A
    /// scan cannot make that statement: `crate::text::defective::chip_that_fills_its_face` fills
    /// with the same verb the correct chip pads with.
    #[test]
    fn no_primitive_names_a_fill_and_every_one_reaches_a_partition_helper() {
        // The bodies of every `fn` in the files a primitive's call chain runs through.
        let mut bodies: Vec<(String, String)> = Vec::new();
        for file in [
            "text.rs",
            "input.rs",
            "structure.rs",
            "frame.rs",
            "state.rs",
        ] {
            let source = source_of(file);
            let mut lines = source.lines().peekable();
            while let Some(line) = lines.next() {
                let trimmed = line.trim();
                let Some(rest) = trimmed
                    .strip_prefix("pub fn ")
                    .or_else(|| trimmed.strip_prefix("fn "))
                    .or_else(|| trimmed.strip_prefix("pub(crate) fn "))
                else {
                    continue;
                };
                let name = rest
                    .split(['(', '<'])
                    .next()
                    .expect("a split always yields one")
                    .to_string();
                let indent = line.len() - line.trim_start().len();
                let close = format!("{}}}", " ".repeat(indent));
                let mut body = String::new();
                for next in lines.by_ref() {
                    if next == close {
                        break;
                    }
                    body.push_str(next);
                    body.push('\n');
                }
                bodies.push((name, body));
            }
        }
        assert!(
            bodies.len() > 20,
            "the body scan found {} functions, which is not this crate",
            bodies.len()
        );
        let body_of = |want: &str| -> String {
            bodies
                .iter()
                .filter(|(name, _)| name == want)
                .map(|(_, body)| body.clone())
                .collect::<Vec<_>>()
                .concat()
        };

        // Walk from each primitive through the crate functions it names, and stop at a partition
        // helper. Four hops is more than any of them needs; a chain that needed more would be a
        // primitive that is not one.
        for (subject, entry) in [
            ("text", "text_into"),
            ("chip", "chip_drawn"),
            ("button", "button_drawn"),
            ("panel", "panel_into"),
        ] {
            let mut frontier = vec![entry.to_string()];
            let mut seen: Vec<String> = Vec::new();
            let mut reached = None;
            for _ in 0..4 {
                let mut next = Vec::new();
                for name in frontier.drain(..) {
                    let body = body_of(&name);
                    assert!(
                        !body.contains("cx.fill(") && !body.contains(".fill(cx"),
                        "`{subject}` reaches `{name}`, which writes through `Ctx::fill` — a cell \
                         the pair cannot see. See `crate::ink`"
                    );
                    if body.contains("fit_into(") || body.contains("block_into(") {
                        reached = Some(name.clone());
                    }
                    for (candidate, _) in &bodies {
                        if body.contains(&format!("{candidate}(")) && !seen.contains(candidate) {
                            seen.push(candidate.clone());
                            next.push(candidate.clone());
                        }
                    }
                }
                if reached.is_some() {
                    break;
                }
                frontier = next;
            }
            let reached = reached.unwrap_or_else(|| {
                panic!(
                    "`{subject}` reaches neither `fit` nor `block`, so its writing is its own and \
                     spec §3's two partition primitives are decoration"
                )
            });
            assert!(
                ["text_into", "fit_into", "face_and_label", "panel_into"].contains(&&*reached),
                "`{subject}` reaches a partition helper at `{reached}`, which is not one of the \
                 places this crate expects one"
            );
        }
    }

    /// The source of one file of this crate, for the two scans above.
    fn source_of(file: &str) -> String {
        let path = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).join(file);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("src/{file}: {e}"))
    }

    /// **The scan fires in both directions**, which is what stops it reporting *nothing is declared*
    /// because it has quietly stopped scanning.
    #[test]
    fn the_subject_scan_finds_a_declaration_when_there_is_one() {
        for (_, declaration) in DECLARATIONS {
            assert!(declares(
                &format!("{declaration}cx: &mut Ctx, area: Rect) -> Response {{"),
                declaration
            ));
            assert!(!declares(&format!("/// {declaration}…)"), declaration));
        }
        // Every declaration names a module that exists, so a rename cannot leave the scan pointing
        // at nothing and reading as *undeclared*.
        for (file, _) in DECLARATIONS {
            let path =
                std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).join(file);
            assert!(
                path.exists(),
                "`src/{file}` is where a subject will be declared"
            );
        }
    }

    /// **The failure names the ticket and does not read as a defect** — ticket 09's criterion 7,
    /// kept alive after the condition that produced it was inverted.
    ///
    /// A `#[should_panic]` over [`assert_stands_up`] cannot reach this any more, because all four
    /// subjects are declared and the call returns. A message no test can read is a message that
    /// rots, so the declaration list is an argument to [`owed_message`] and the hostile case is one
    /// call: **none declared, and the sentence still separates *unimplemented* from *wrong*.**
    #[test]
    fn the_waiting_message_still_says_which_failure_it_is() {
        let message = owed_message(&[], "the dense screen, 300x80, 338 regions")
            .expect("no subject declared is a scene that is not standing");
        assert!(message.contains("components 10"), "{message}");
        assert!(
            message.contains("waiting for its subject rather than failing"),
            "a failure that could be read as a defect in the screen: {message}"
        );
        // Every missing subject named, with the file it belongs in.
        for (id, (file, _)) in SUBJECTS.into_iter().zip(DECLARATIONS) {
            assert!(message.contains(id), "{id} is not named: {message}");
            assert!(message.contains(file), "src/{file} is not named: {message}");
        }
        // Three of four is not four of four, so the count is the failing set and not a constant.
        let partial = owed_message(&["text", "chip", "button"], "scene 1").expect("one is missing");
        assert!(partial.contains("1 of 4"), "{partial}");
        assert!(partial.contains("`panel`"), "{partial}");
        // And the met direction answers nothing at all.
        assert_eq!(owed_message(&SUBJECTS, "scene 1"), None);
        assert_eq!(owed_message(&subjects_declared(), "scene 1"), None);
    }

    /// **No two widgets on this screen share an id** — ADR 0027's free detector, on a screen with
    /// 338 of them.
    #[test]
    fn the_dense_screen_declares_no_colliding_ids() {
        use crate::counters::Counter;
        for arm in ARMS {
            let run = play(arm, &screen());
            let counters = run.counters(Allocations::over(1, 0));
            assert_eq!(
                counters.get(Counter::Merges).get(Counter::Merges),
                0,
                "{arm:?}: a widget claimed an id another widget already held, and the screen still \
                 renders correctly"
            );
        }
    }

    /// **A scrim filled under the dialog writes the dialog's own cells and one drawn around it does
    /// not** — §2's *600 fewer writes*, which is the dialog's cell count and reproduces by
    /// construction.
    #[test]
    fn a_scrim_under_the_dialog_costs_the_dialogs_own_six_hundred_writes() {
        let under = play(Arm::ScrimUnderTheDialog, &screen());
        let around = play(Arm::ScrimAroundTheDialog, &screen());
        assert_eq!(
            under.tally().writes() - around.tally().writes(),
            SCRIM_EXCESS_WRITES
        );
        assert_eq!(
            {
                let d = dialog_at(Rect::new(0, 0, W, H));
                u64::from(d.w) * u64::from(d.h)
            },
            SCRIM_EXCESS_WRITES,
            "the figure §2 states *is* the dialog"
        );
        // And the two screens are the same screen, which is what makes the difference a cost rather
        // than a change.
        let diff = compare_at(
            Density::Compact,
            scrim_around_the_dialog as Painter,
            scrim_under_the_dialog as Painter,
            &screen(),
        );
        assert!(diff.clean(), "{diff}");
    }
}
