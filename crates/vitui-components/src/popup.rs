//! **The overlay family's screen: 300x80, 312 chips, two `select`s and a menu bar, and the five
//! configurations §12 states as a table.**
//!
//! Components ticket 25. Spec §12, §17 (O5), §21 scene 14. This is the screen the overlay family's
//! scene is a scene *of*, and it is the third of this backlog's scenes tickets — [`crate::dense`]
//! and [`crate::listing`] are the other two, and everything about how a scene is stated, made red
//! and reported is theirs.
//!
//! | configuration | what it decides |
//! |---|---|
//! | nothing open | the base: 317 regions and 316 stops, and **a menu bar is a region that is not a stop** |
//! | one select open | the popup's own declaration: one collection entry and one blur position, **+2 / +1** |
//! | menu + submenu | two overlays, one nested in the other: **+6 / +6** |
//! | modal dialog + scrim | a `Trap` and a barrier: **+2 / +2**, and the walk visits 2 of 318 |
//! | the popup drawn fill-first | the fill that runs first, re-damaging its own ink on every frame |
//!
//! # Three instruments, for [`crate::listing`]'s reason
//!
//! 1. **The frame structures go through a real `Ctx::overlay`** — the regions, the stops, the ring,
//!    the traps and the census. Those are properties of the *frame* and the overlay pass is what
//!    builds them, so nothing here stands in for the mechanism.
//! 2. **The re-damage measurements draw the overlay into the base pass**, which is
//!    [`crate::dense::modal_steady`]'s substitution taken whole and named for the same reason: an
//!    overlay body draws through a `Ctx` whose origin has moved, and a [`Pen`] over one surface would
//!    union two coordinate systems. It is on **both** arms, so it is on the side of neither.
//! 3. **The keyboard goes through posted keys.** Six `Tab`s are six `Driver::post_key` calls and not
//!    six reads of `Frame::tab_walk` — a walkthrough that consulted the frame's own answer would be
//!    testing one expression against itself (`ctx.rs`'s own note, one crate down).
//!
//! # Every region and every stop figure in §12's table reproduces, and it is by construction
//!
//! The base is `312 + 2 + 3 = 317` regions and `312 + 2 + 2 = 316` stops. **The one region that is
//! not a stop is the menu bar itself**, which declares [`Interest::HOVER`] and nothing else so that
//! moving the pointer along an open bar switches menus; its two titles are the stops. That is the
//! same shape §12 gives the popup's blur entry — *one hit entry over the whole rectangle with
//! `Interest::HOVER` only* — arriving in the base pass, and it is why a screen of 317 regions has 316
//! stops rather than 317.
//!
//! The three deltas are then the family's own arithmetic, and each is a different sentence of §12:
//!
//! | | regions | stops | why |
//! |---|---|---|---|
//! | a dropdown | +2 | +1 | one hit entry per collection plus the blur position |
//! | a menu and its submenu | +6 | +6 | a menu row is a target, and there are three of them twice |
//! | a modal | +2 | +2 | two buttons inside a `Trap`, and a scope declares no region |
//!
//! **A menu carries no blur entry and a dropdown does**, which is not an inconsistency: §12 lists
//! four dismissals, and a menu bar's menu takes *the owner ceasing to request* while a dropdown takes
//! *blur qualified by a position*. The entry exists for the dismissal that needs it.
//!
//! # What does not reproduce, and both are findings rather than misses
//!
//! **`allocations` is not 0 and cannot be.** §12's table reads `0` in all five rows. That table was
//! measured on the prototype's crate-private bump arena, which runtime ticket 21 **deleted**:
//! a body is now one `Box` in a queue the frame call owns, so a frame with `n` overlays
//! standing costs `n + 1` allocations and a frame with none costs nothing. The runtime's own gate is
//! the marginal equality — *one more overlay standing is exactly one more allocation a frame* — and
//! it is the shipped number. Bending it back would mean restoring seven `unsafe` blocks to a
//! workspace that has none. See [`overlay_allocs`].
//!
//! **`content layers` is not 2 / 4 / 3.** Every request in the prototype carried `shadow: 96`, a
//! second *operator* layer beside the overlay's own, so its host held two layers an overlay and three
//! for a modal with a scrim. [`OverlayOpts`] has **no shadow field**, so the shipped census is one
//! layer an overlay and a scrim's engine layer is folded into its owner's row rather than counted
//! beside it. [`crate::counters::Counters::content_layers`] already carried the scrim half of this;
//! the shadow is the other half, and it is what makes the column a 2:4:3 rather than a 1:2:1. See
//! [`LAYERS`].
//!
//! # The gate is red, and it is red for one reason
//!
//! `select` and `overlay` are both undeclared, which is [`standing`]'s verdict and components ticket
//! **26**'s job. Everything the screen itself can be asked is measured: the five configurations, the
//! five region and stop rows, the ring and the traps, the walk's named exception, the opening cliff,
//! the three scrim spellings, the fill-first popup, the two axes of the family and the three answers
//! to *where does the keyboard go when a modal closes*. What is missing is the subject, and
//! [`owed_message`] is the sentence that says so rather than reading as a defect in the screen.
//!
//! [`OverlayOpts`]: vitui_runtime::overlay::OverlayOpts
//! [`Pen`]: crate::runner::Pen

use std::time::{Duration, Instant};

use vitui_runtime::keys::{Chord, Code};
use vitui_runtime::overlay::{OverlayOpts, Placement, Z};
use vitui_runtime::{Buttons, Ctx, Density, Id, Interest, Mods, Mouse, MouseKind, Role};

use crate::collect::{CollOpts, collection};
use crate::counters::{Allocations, Counters, Tally};
use crate::ink::{Direct, Ink};
use crate::input::{SelectOpts, SelectState, Sizing, select_into};
use crate::obligations::Verdict;
use crate::order::Rows;
use crate::overlay::{Kind, MARK, PopupState, ShellOpts, overlay, overlay_into, overlay_with};
use crate::runner::{Canvas, Pen};
use crate::text::{ChipOpts, chip_drawn};
use vitui_runtime::Rect;

// ── the screen ───────────────────────────────────────────────────────────────────────────────────

/// The screen's width. §12's own, and [`crate::dense::W`]'s, so that two screens on this backlog are
/// comparable without a conversion.
pub const W: u16 = 300;
/// The screen's height. See [`W`].
pub const H: u16 = 80;

/// **312 chips**, which is §12's own count and what the base row is built out of.
pub const CHIPS: usize = 312;
/// A chip's width. [`crate::dense::CHIP`]'s, for [`W`]'s reason.
pub const CHIP: u16 = 12;
/// Chips a row: `24 x 12 = 288` of [`W`]'s three hundred columns.
pub const PER_ROW: usize = 24;
/// Rows of chips: `24 x 13 = 312` exactly, which is why neither factor is rounded.
pub const CHIP_ROWS: usize = CHIPS / PER_ROW;
/// The row the chip band starts on: the menu bar is row 0 and the two `select`s row 1.
pub const BAND_TOP: u16 = 2;

/// **Two `select`s**, §12's own count. One opens; the other owns the dialog.
pub const SELECTS: usize = 2;
/// A `select`'s width, which is also its popup's.
pub const SELECT: u16 = 20;
/// **Two menu titles on the bar.** Two and not three, because the bar's own hover entry is the third
/// region and 317 is the count §12 states.
pub const MENUS: usize = 2;
/// A menu title's width on the bar.
pub const MENU_TITLE: u16 = 10;

/// **The base screen's hit entries: 317.** `312 chips + 2 selects + 1 bar + 2 titles`.
pub const REGIONS: usize = CHIPS + SELECTS + 1 + MENUS;
/// **The base screen's tab stops: 316.** [`REGIONS`] less the bar, which declares
/// [`Interest::HOVER`] and is therefore not in the ring.
pub const STOPS: usize = CHIPS + SELECTS + MENUS;

/// **The popup's granted size, `(20, 8)`** — §12's own, and the one it contrasts `(20, 0)` against.
pub const POPUP: (u16, u16) = (SELECT, 8);
/// A menu's granted size.
pub const MENU: (u16, u16) = (16, 3);
/// **Rows a menu declares**, each one a target of its own. See the module header for why a menu row
/// is a stop and a dropdown's whole option list is one entry.
pub const MENU_ROWS: usize = 3;
/// **The dialog, `(60, 10)`** — [`crate::dense::DIALOG`] rather than a second rectangle, so that the
/// scrim's six hundred cells have one home in this crate and not two.
pub const DIALOG: (u16, u16) = crate::dense::DIALOG;
/// The dialog's buttons, which are its two tab stops and the trap's whole range.
pub const BUTTONS: usize = 2;

/// **What a dropdown adds: two regions and one stop.** One hit entry for the option list and one
/// over the whole rectangle for the blur position, which asks for the pointer and not the ring.
pub const DROPDOWN_DELTA: (usize, usize) = (2, 1);
/// **What a menu with its submenu adds: four regions and two stops — and §12 says six and six.**
///
/// The subtraction is §5's rule and it is a finding rather than a discrepancy. §12's six is
/// [`MENU_ROWS`] twice, *each row a target of its own*; §5 collapses a menu into a
/// [`Mode`](crate::collect::Mode) of `collection`, and **a collection declares one hit entry however
/// many rows it has**. So each level of the shipped menu is [`DROPDOWN_DELTA`] — one entry for its
/// rows and one for the blur position — and two levels are twice that.
///
/// It is the same subtraction [`PER_ROW_ENTRIES`] prices for a dropdown, arriving on the construction
/// §12's own prototype spent per row. Reproducing the six would mean declaring per row on purpose,
/// which is the defect [`Config::PerRow`] exists to be.
pub const MENU_DELTA: (usize, usize) = (2 * DROPDOWN_DELTA.0, 2 * DROPDOWN_DELTA.1);

/// **§12's remembered menu delta.** Recorded, not reproduced — see [`MENU_DELTA`].
pub const SPEC_MENU_DELTA: (usize, usize) = (MENU_ROWS * 2, MENU_ROWS * 2);
/// **What a modal adds: two regions and two stops.** Its buttons; the `Trap` scope declares neither,
/// which is `Ctx::scope`'s own sentence — *a scope renames nothing and declares no region*.
pub const MODAL_DELTA: (usize, usize) = (BUTTONS, BUTTONS);

/// The eight options the dropdown offers.
pub const OPTIONS: [&str; POPUP.1 as usize] = [
    "name",
    "date modified",
    "date created",
    "size on disk",
    "kind used most",
    "tags and labels",
    "date last opened",
    "date added here",
];

/// The menu bar's titles.
pub const TITLES: [&str; MENUS] = ["  view", "  sort"];
/// A menu's rows.
pub const ROWS: [&str; MENU_ROWS] = [" density", " columns", " grouping"];
/// The dialog's title.
pub const DIALOG_TITLE: &str = " discard changes ";
/// The dialog's two buttons, which are the trap's whole range.
pub const BUTTON_LABELS: [&str; BUTTONS] = ["discard", "keep"];
/// What a `select` says when it is shut.
pub const SELECT_LABEL: &str = "sort: name";
/// What a chip says. Static, for [`crate::listing::ROW`]'s reason: a screen whose content varies
/// reports a number about its content.
pub const CHIP_LABEL: &str = "photo";

/// **The whole screen, and what a correct arm writes on every frame.** `300 x 80`.
pub const SCREEN: u64 = W as u64 * H as u64;

// ── the numbers this screen is held to ───────────────────────────────────────────────────────────

/// **Allocations a frame with `n` overlays standing costs: `n + 1`, and `0` for none.**
///
/// §12's table reads **0** in every row, and that was true of the prototype's crate-private bump
/// arena. Runtime ticket 21 deleted it: a body is one `Box` in a queue the frame call owns,
/// and the queue is the `+ 1`. The `+ 1` is not slack — a stored body is `+ 'f` and safe Rust cannot
/// put a `'f`-bounded value inside the thing borrowed for `'f`, so the queue is a local of
/// `Driver::frame` rather than a field.
///
/// **Asserted as measured rather than bent to fit**: the only edit that would make §12's zero true
/// again is the arena, which is seven `unsafe` blocks in a workspace that has none.
#[must_use]
pub const fn overlay_allocs(bodies: usize) -> usize {
    if bodies == 0 { 0 } else { bodies + 1 }
}

/// **Layers the census keeps alive, as `(configuration, §12 remembers, measured here)`.**
///
/// §12's table reads 2 / 4 / 3 because every request in the prototype carried `shadow: 96` — a second
/// operator layer beside the overlay's own. [`vitui_runtime::overlay::OverlayOpts`] has no shadow
/// field, so the shipped numbers are 1 / 2 / 1. **The ratio between a dropdown and a menu with its
/// submenu survives**: twice, either way.
pub const LAYERS: [(&str, usize, usize); 4] = [
    ("nothing open", 0, 0),
    ("one select open", 2, 1),
    ("menu + submenu", 4, 2),
    ("modal dialog + scrim", 3, 1),
];

/// **What a popup at `h = 0` declares when its body declares before it looks: 321 regions and 318
/// stops, against 317 and 316.**
///
/// Both `select`s request at `(20, 0)` and each body declares its collection entry and its blur
/// position anyway — [`DROPDOWN_DELTA`] twice. §12's *the size may not come from the drawn extent* is
/// what puts a popup at `h = 0` in the first place: a popup has no frame before the one it opens on,
/// so its extent there is 0 and stays 0.
///
/// **It is small only because a collection declares one hit entry however many rows it has**.
/// [`PER_ROW_ENTRIES`] is what the other spelling costs on the same screen.
pub const CLOSED_DECLARES: (usize, usize) =
    (REGIONS + 2 * DROPDOWN_DELTA.0, STOPS + 2 * DROPDOWN_DELTA.1);

/// **What one open dropdown declaring one hit entry per row costs**: eight entries and eight stops
/// **on top of** the one and the one §5's rule spends for the whole collection.
///
/// *On top of* and not *instead of*, and the reason is that the rows are still drawn by the
/// collection: the defect a caller who wants a per-row hover actually writes is a loop of
/// `cx.interact` beside the component, not a component with its entry removed. So the arm's cells are
/// identical to the shipped arm's and only the counts move —
/// [`Config::expected`] is where the arithmetic is spelled.
pub const PER_ROW_ENTRIES: usize = POPUP.1 as usize;

/// **Flips an overlay that declares *and* covers its anchor takes: 99 in 100 frames.**
///
/// §12's Axis A. A tooltip made of cells and nothing else is fine; one that declares a hit entry over
/// a rectangle covering its own anchor takes the anchor's hover away, which closes it, which uncovers
/// the anchor, which opens it again. One flip a frame for ever, and 99 is [`FLIP_FRAMES`] less the
/// first, which has no predecessor to differ from.
pub const FLIPS: u32 = 99;
/// The frames [`FLIPS`] is measured over.
pub const FLIP_FRAMES: u32 = 100;

/// **Frames a dialog owned by the menu row that opened it lives, out of eight: 3.**
///
/// §12's Axis B — *the census is over the request*. The application believes the dialog open for all
/// eight; the row that owns it exists only while the menu is drawing, so the body runs on exactly the
/// frames its owner requests it on and the layer dies with the request.
pub const DIALOG_LIVES: u32 = 3;
/// The frames [`DIALOG_LIVES`] is measured over.
pub const CENSUS_FRAMES: u32 = 8;

/// **Wheel notches every instrument that prices a dead wheel posts: twenty.**
///
/// [`crate::wheel::CLICKS`] and not a second constant, because it is the same twenty and this crate
/// keeps one home for a number. §12 states it inside its own section — *§7's literal `Copy`-only body
/// moves the offset 0 in 20 wheel clicks*.
pub const WHEEL_CLICKS: u32 = crate::wheel::CLICKS;

/// **`Tab`s pressed against a modal, which is §12's own six.** With a trap none of them leaves; with
/// no trap every one of them does.
pub const TABS: u32 = 6;

/// **Stops a walkthrough visits with the modal up: 2, of 318 declared.**
///
/// §21's third refinement — *name the exception; do not loosen the gate* — and this is the screen it
/// is named on. The gate is the conjunction §21 states: *the walk repeats no id, and reaches every
/// stop unless a trap is standing*, with `Frame::trap_scopes` as the only thing that can answer the
/// second half.
pub const TRAPPED_VISITS: usize = BUTTONS;

/// **What a scrim filled under the dialog re-damages every steady frame: 600.**
///
/// [`crate::dense::SCRIM_UNDER`] and not a second constant, because it is the same six hundred cells
/// and this crate keeps one home for a number. §2 remembers **229**; that file carries the reason the
/// screen says 600 instead, and this screen reproduces it because it stands the same
/// [`crate::dense::DIALOG`] up.
pub const SCRIM_UNDER: u64 = crate::dense::SCRIM_UNDER;

/// **Writes the scrim-under arm costs over the complement: 600.** §12's *600 fewer writes*, and
/// [`crate::dense::SCRIM_EXCESS_WRITES`] for [`SCRIM_UNDER`]'s reason.
pub const SCRIM_EXCESS_WRITES: u64 = crate::dense::SCRIM_EXCESS_WRITES;

/// **What a popup drawn fill-first re-damages every steady frame: 90.**
///
/// §12 remembers **107**. The quantity is *the popup's own ink* — the mark on the chosen row plus
/// every **non-blank** cell of the eight option labels — and it is arithmetic rather than a
/// measurement: the fill writes the pad value into all one hundred and sixty cells and the rows write
/// their glyphs back over it, so a cell that already carried the pad value is changed by neither. A
/// space *inside* a label is such a cell, which is why [`popup_ink`] counts glyphs and not widths:
/// counting widths gives 102 and the screen says 90, and the twelve between them are the spaces in
/// `"date modified"` and its seven neighbours.
///
/// **The relation is what this screen can be held to** — 90 against the text-first arm's **0** — and
/// the relation is the one §12 states. §12's 107 says that its own popup carried 107 cells of ink in
/// 160; [`OPTIONS`] is this popup's data and was not chosen to make a number come out.
pub const FILL_FIRST: u64 = 1 + popup_ink();

/// **The glyph cells in [`OPTIONS`]**, which is the arithmetic behind [`FILL_FIRST`].
///
/// Non-blank cells and not widths, for the reason that constant gives: the engine drops a write whose
/// value equals the resident value, so a space written over a space is not a change and does not
/// re-damage.
#[must_use]
pub const fn popup_ink() -> u64 {
    let mut sum = 0u64;
    let mut i = 0;
    while i < OPTIONS.len() {
        let bytes = OPTIONS[i].as_bytes();
        let mut b = 0;
        while b < bytes.len() {
            if bytes[b] != b' ' {
                sum += 1;
            }
            b += 1;
        }
        i += 1;
    }
    sum
}

// ── the five configurations ──────────────────────────────────────────────────────────────────────

/// **The five rows of §12's table, and the spellings this ticket stands beside them.**
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Config {
    /// Nothing open. The base row.
    Nothing,
    /// One `select` open, its popup drawing its rows text-first.
    OneSelect,
    /// A menu on the bar with a submenu nested in it — two overlays, one owner each.
    MenuAndSubmenu,
    /// A modal dialog with a scrim, a `Trap` and a barrier. The scrim is the engine's **operator
    /// layer**, which is §12's third spelling and the one that costs neither cells nor damage.
    ModalAndScrim,
    /// The same modal **without** its trap. §12's six `Tab`s, as a screen.
    ModalWithoutTrap,
    /// One `select` open, its popup filling its rectangle before it writes its rows.
    FillFirst,
    /// **Both `select`s requested at `h = 0`, with bodies that declare before they look.** §12's
    /// closed-popup case, and the arm that declares.
    ClosedDeclares,
    /// Both `select`s requested at `h = 0`, with bodies that look first. The correct arm.
    ClosedSilent,
    /// One `select` open, its popup declaring **one hit entry per row** instead of one for the
    /// collection. What §5's rule is worth here.
    PerRow,
}

impl Config {
    /// Every configuration, which is what the report loops over.
    pub const ALL: [Config; 9] = [
        Config::Nothing,
        Config::OneSelect,
        Config::MenuAndSubmenu,
        Config::ModalAndScrim,
        Config::ModalWithoutTrap,
        Config::FillFirst,
        Config::ClosedDeclares,
        Config::ClosedSilent,
        Config::PerRow,
    ];

    /// §12's five, in §12's order.
    pub const TABLE: [Config; 5] = [
        Config::Nothing,
        Config::OneSelect,
        Config::MenuAndSubmenu,
        Config::ModalAndScrim,
        Config::FillFirst,
    ];

    /// The word the report prints it under.
    pub const fn word(self) -> &'static str {
        match self {
            Config::Nothing => "nothing open",
            Config::OneSelect => "one select open",
            Config::MenuAndSubmenu => "menu + submenu",
            Config::ModalAndScrim => "modal dialog + scrim",
            Config::ModalWithoutTrap => "modal dialog, no trap",
            Config::FillFirst => "the popup drawn fill-first",
            Config::ClosedDeclares => "both popups at h = 0, declaring",
            Config::ClosedSilent => "both popups at h = 0, silent",
            Config::PerRow => "one select open, one entry a row",
        }
    }

    /// **How many overlay bodies stand**, which is what the frame's allocation total is a function of.
    pub const fn bodies(self) -> usize {
        match self {
            Config::Nothing => 0,
            Config::OneSelect | Config::FillFirst | Config::PerRow => 1,
            Config::MenuAndSubmenu => 2,
            Config::ModalAndScrim | Config::ModalWithoutTrap => 1,
            // Both `select`s request, whether or not their bodies say anything.
            Config::ClosedDeclares | Config::ClosedSilent => 2,
        }
    }

    /// **What the frame's hit index and ring should hold**, as `(regions, stops)`.
    pub const fn expected(self) -> (usize, usize) {
        match self {
            Config::Nothing | Config::ClosedSilent => (REGIONS, STOPS),
            Config::OneSelect | Config::FillFirst => {
                (REGIONS + DROPDOWN_DELTA.0, STOPS + DROPDOWN_DELTA.1)
            }
            Config::MenuAndSubmenu => (REGIONS + MENU_DELTA.0, STOPS + MENU_DELTA.1),
            Config::ModalAndScrim | Config::ModalWithoutTrap => {
                (REGIONS + MODAL_DELTA.0, STOPS + MODAL_DELTA.1)
            }
            Config::ClosedDeclares => CLOSED_DECLARES,
            // The shipped dropdown, plus one entry and one stop a row on top of it.
            Config::PerRow => (
                REGIONS + DROPDOWN_DELTA.0 + PER_ROW_ENTRIES,
                STOPS + DROPDOWN_DELTA.1 + PER_ROW_ENTRIES,
            ),
        }
    }

    /// **Which `select`s this configuration stands open**, written onto the owners' own state.
    ///
    /// The configuration is the scene's, and `open` is the *component's* — so a `Config` cannot
    /// request anything itself. It seats a bool and the component does the rest, which is what makes
    /// the screen a screen of `select` rather than a screen of `cx.overlay`.
    pub const fn seat(self, first: &mut SelectState, second: &mut SelectState) {
        let (a, b) = match self {
            Config::OneSelect | Config::FillFirst | Config::PerRow => (true, false),
            Config::ClosedDeclares | Config::ClosedSilent => (true, true),
            Config::Nothing
            | Config::MenuAndSubmenu
            | Config::ModalAndScrim
            | Config::ModalWithoutTrap => (false, false),
        };
        if a {
            first.open();
        } else {
            first.close();
        }
        if b {
            second.open();
        } else {
            second.close();
        }
    }
}

// ── the ids, which are named so a gate can point at one ──────────────────────────────────────────

/// The menu bar's own hover entry — the region that is not a stop.
pub const BAR: Id = Id::named("popup.bar");
/// The root the two menu titles are keyed off.
pub const TITLE_ROOT: Id = Id::named("popup.title");
/// The root the two `select`s are keyed off.
pub const SELECT_ROOT: Id = Id::named("popup.select");
/// The root the 312 chips are keyed off.
pub const CHIP_ROOT: Id = Id::named("popup.chip");
/// The root a dropdown's option list is keyed off — **one entry for the whole collection**.
///
/// **Keyed and not bare, and that is §12's own sentence arriving inside a body.** Two `select`s
/// standing at once are two overlays, and the two bodies are one function; with a bare `Id::named`
/// the second popup's entries land on ids the first has already claimed, `Ctx::interact` makes the
/// merged claim **inert**, and the frame reports 319 regions where two popups declared four. The
/// first run of [`CLOSED_DECLARES`] read exactly that — 319 / 317 against the 321 / 318 it states —
/// with `overlays_merged` at **0**, because the requests' owners were distinct and it was the
/// declarations inside them that collided. [`Shape::merges`] is the counter that says so.
pub const LIST: Id = Id::named("popup.list");
/// The root a dropdown's blur position is keyed off: one entry over the whole rectangle,
/// `Interest::HOVER` only. Keyed for [`LIST`]'s reason.
pub const BLUR: Id = Id::named("popup.blur");
/// The root a per-row popup's entries are keyed off.
pub const ROW_ROOT: Id = Id::named("popup.row");
/// The menu's own rows.
pub const MENU_ROOT: Id = Id::named("popup.menurow");
/// **The submenu's owner, minted rather than shared**: a component with two overlays
/// standing must mint a second id, and shared they get one slot resized.
pub const SUBMENU_OWNER: Id = Id::named("popup.submenu");
/// The submenu's own rows.
pub const SUBMENU_ROOT: Id = Id::named("popup.subrow");
/// **The key the dialog's shell is minted under, off its own owner.**
///
/// The shell claims the blur position it does not have, the trap scope and nothing else, and its id
/// has to be spelled rather than minted from a call site: `Ctx::id` inside an overlay body mints from
/// the body's own source line, which is one line for every dialog on the screen.
pub const TRAP_KEY: u64 = 1;
/// The dialog's trap scope, which is [`TRAP_KEY`] off [`dialog_owner`].
#[must_use]
pub fn trap_scope() -> Id {
    Id::keyed(dialog_owner(), TRAP_KEY)
}
/// The dialog's buttons.
pub const BUTTON_ROOT: Id = Id::named("popup.button");

/// The first chip, which is where the screen seats the focus.
#[must_use]
pub fn first_chip() -> Id {
    Id::keyed(CHIP_ROOT, 0)
}

/// The last chip, which is the ring neighbour the vanish rule reaches for.
#[must_use]
pub fn last_chip() -> Id {
    Id::keyed(CHIP_ROOT, CHIPS as u64 - 1)
}

/// The first tab stop in draw order — the first menu title.
#[must_use]
pub fn first_stop() -> Id {
    Id::keyed(TITLE_ROOT, 0)
}

/// The `select` that owns the dialog, and refocuses itself on the way out.
#[must_use]
pub fn dialog_owner() -> Id {
    Id::keyed(SELECT_ROOT, 1)
}

/// The dialog's first button, which is where a standing trap pulls the focus.
#[must_use]
pub fn first_button() -> Id {
    Id::keyed(BUTTON_ROOT, 0)
}

// ── the screen, drawn ────────────────────────────────────────────────────────────────────────────

/// **What the screen turned out to be**, returned rather than printed, because a number only a
/// report prints is a number no gate can read.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Shape {
    /// How many regions the base pass declared, counted by the draw.
    pub declared: usize,
    /// How many the runtime's hit index holds, counted by the frame. **Two counts and not one**, for
    /// [`crate::dense::Shape`]'s reason.
    pub regions: usize,
    /// How many tab stops the frame holds.
    pub stops: usize,
    /// How many entries the ring holds, which is not the same question: a `Group` collapses a range
    /// onto one stop and leaves every entry in the ring.
    pub ring: usize,
    /// How many traps are standing. **One, with the modal up, and nothing else can name the
    /// exception the walkthrough gate carries.**
    pub traps: usize,
    /// How many overlay layers the census is keeping alive.
    pub layers: usize,
    /// How many overlay bodies the frame boxed. One allocation each, and the queue is one more.
    pub bodies: u32,
    /// How many overlay requests named an owner that had already asked. **Zero, or two overlays are
    /// sharing one slot**.
    pub merged: u32,
    /// **How many claims landed on an id that had already claimed this frame.** §21's register row 3,
    /// and the counter that catches a defect [`Shape::merged`] cannot see: two overlays with distinct
    /// *owners* whose *bodies* declare the same ids. See [`LIST`], which is the instance this file
    /// shipped and then removed.
    pub merges: u32,
    /// Who holds the focus at the end of the frame.
    pub focused: Option<Id>,
}

/// **Everything §12's screen holds across frames, which is what a popup being the owner's costs a
/// caller.**
///
/// One [`SelectState`] and one [`PopupState`] per widget, and there is nowhere else for either to
/// live: the owner's crosses the base pass by value and the body's crosses the **frame** by
/// `&'f mut`, so both have to be outside the frame call. That is §12's *two structs, one writer
/// each* arriving as a fact about the caller's own storage.
#[derive(Clone, Debug, Default)]
pub struct Held {
    /// The two `select`s' owner state. Written only by [`crate::input::select`].
    pub owners: [SelectState; SELECTS],
    /// Their bodies'. Written only by the body.
    pub bodies: [PopupState; SELECTS],
    /// The menu bar's popup, one per title. Only the first is ever opened by [`Config`].
    pub menus: [PopupState; MENUS],
    /// **The submenu's**, handed to the menu's body wrapped in an `Option` the body takes.
    ///
    /// The wrapper is this ticket's seam finding and it is not decoration: an overlay body is
    /// `FnMut`, so it **cannot move a capture**, and a nested request has to move a `&'f mut` into
    /// the inner closure. A reborrow is no help — it is shorter than `'f` and fails the `+ 'f`
    /// bound. `Option::take` *mutates* the capture instead of moving out of it, which is the one
    /// spelling that compiles, and the state itself stays here across frames.
    pub submenu: PopupState,
}

impl Held {
    /// Nothing open and nothing chosen.
    #[must_use]
    pub fn new() -> Held {
        Held::default()
    }
}

/// **The screen, one frame, in the configuration `config` asks for.**
///
/// Generic over [`Ink`] so a [`Tally`] and a [`Pen`] measure the shipped drawing path rather than a
/// copy of it — [`crate::ink`]'s whole argument. Returns what the **base pass** declared; the frame's
/// own counts are read off `Driver::inspect` afterwards, because an overlay body runs after every
/// context in the base pass has been dropped.
///
/// # The two lifetime annotations spec §1 measured
///
/// `cx: &mut Ctx<'f, '_>` and `held: &'f mut Held`. §1 records that four of its five components carry
/// no lifetime at all and *the fifth opens an overlay*; this is the screen that fifth is on, and the
/// annotation count is two here for the same reason it was two there.
///
/// # The overlay bodies draw through [`Direct`] and not through `ink`
///
/// A body is `FnMut(&mut Ctx<'f, '_>) + 'f`, so a `&mut I` borrowed for the call cannot travel into
/// one. That is not a hole in the seam: a body's `Ctx` is rooted at its own layer, so its writes are
/// in a different coordinate system from the base pass's and a `Tally` that saw both would union two
/// grids — the same collision [`crate::frame`] measures at 124 false double writes. The counters an
/// overlay actually moves are the frame's, and those are read from the frame.
pub fn draw_into<'f, I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'f, '_>,
    held: &'f mut Held,
    config: Config,
) -> usize {
    let theme = cx.theme();
    let body = theme.paint(Role::Body);
    let title = theme.paint(Role::Title);
    let mut declared = 0usize;

    // **The bar is the region that is not a stop.** `Interest::HOVER` and nothing else, so that
    // moving the pointer along an open bar switches menus — §12's blur shape, arriving in the base
    // pass and settling 317 against 316.
    let bar = Rect::new(0, 0, W, 1);
    let _ = cx.interact(BAR, bar, Interest::HOVER);
    declared += 1;
    let mut x = 0i32;
    for (i, label) in TITLES.iter().enumerate() {
        let r = Rect::new(x, 0, MENU_TITLE, 1);
        let resp = cx.interact(
            Id::keyed(TITLE_ROOT, i as u64),
            r,
            Interest::CLICK.with(Interest::FOCUS),
        );
        declared += 1;
        let _ = resp;
        let _ = ink.pad_to(cx, x, 0, label, MENU_TITLE, title);
        x += i32::from(MENU_TITLE);
    }
    let _ = ink.run(cx, x, 0, " ", W - MENU_TITLE * MENUS as u16, title);

    // **The subjects.** `held` is destructured with a slice pattern rather than iterated, because
    // `iter_mut` yields items borrowed for the *reborrow* and `select` needs `&'f mut` — the same
    // `'f` the frame call has. Two calls and not a loop for the same reason `Id::keyed` is spelled
    // out below: `Ctx::id` mints from the call site, so a loop would mint one id for both widgets and
    // `Ctx::interact` would make the second inert.
    let Held {
        owners,
        bodies,
        menus,
        submenu,
    } = held;
    let [first_owner, second_owner] = owners;
    let [first_body, second_body] = bodies;
    let [menu_body_state, _second_menu] = menus;

    config.seat(first_owner, second_owner);
    let opts = SelectOpts {
        placement: Placement::BELOW,
        ..SelectOpts::default()
    };
    let first = Rect::new(0, 1, SELECT, 1);
    let second = Rect::new(i32::from(SELECT) + 2, 1, SELECT, 1);
    // **One function, four spellings, one call each.** The shipped arm and the three §12 refuses are
    // one value apart inside `select`, so what a gate plays is the shipped drawing path.
    match config {
        Config::FillFirst => {
            let _ = crate::input::defective::fill_first(
                ink,
                cx,
                Id::keyed(SELECT_ROOT, 0),
                first,
                first_owner,
                first_body,
                &OPTIONS,
                &opts,
            );
        }
        Config::PerRow => {
            let _ = crate::input::defective::per_row(
                ink,
                cx,
                Id::keyed(SELECT_ROOT, 0),
                first,
                first_owner,
                first_body,
                &OPTIONS,
                &opts,
            );
        }
        Config::ClosedDeclares => {
            let _ = crate::input::defective::declares_at_zero(
                ink,
                cx,
                Id::keyed(SELECT_ROOT, 0),
                first,
                first_owner,
                first_body,
                &OPTIONS,
                &opts,
                Sizing::FromTheDrawnExtent,
            );
        }
        Config::ClosedSilent => {
            let _ = crate::input::defective::sized(
                ink,
                cx,
                Id::keyed(SELECT_ROOT, 0),
                first,
                first_owner,
                first_body,
                &OPTIONS,
                &opts,
                Sizing::FromTheDrawnExtent,
            );
        }
        _ => {
            let _ = select_into(
                ink,
                cx,
                Id::keyed(SELECT_ROOT, 0),
                first,
                first_owner,
                first_body,
                &OPTIONS,
                &opts,
            );
        }
    }
    match config {
        Config::ClosedDeclares => {
            let _ = crate::input::defective::declares_at_zero(
                ink,
                cx,
                Id::keyed(SELECT_ROOT, 1),
                second,
                second_owner,
                second_body,
                &OPTIONS,
                &opts,
                Sizing::FromTheDrawnExtent,
            );
        }
        Config::ClosedSilent => {
            let _ = crate::input::defective::sized(
                ink,
                cx,
                Id::keyed(SELECT_ROOT, 1),
                second,
                second_owner,
                second_body,
                &OPTIONS,
                &opts,
                Sizing::FromTheDrawnExtent,
            );
        }
        _ => {
            let _ = select_into(
                ink,
                cx,
                Id::keyed(SELECT_ROOT, 1),
                second,
                second_owner,
                second_body,
                &OPTIONS,
                &opts,
            );
        }
    }
    declared += SELECTS;
    // **The two gaps and the tail, which is what makes row 1 a partition.** A screen where two cells
    // between the widgets belong to nobody is 23 998 of 24 000, and nothing about it looks wrong.
    let _ = ink.run(cx, i32::from(SELECT), 1, " ", 2, body);
    let filled = 2 * SELECT + 2;
    let _ = ink.run(cx, i32::from(filled), 1, " ", W - filled, body);

    // The chip band. `chip_drawn` and not `chip`, because a gate later in this file has to *name*
    // the last chip — `Ctx::id` mints from the call site and nothing outside the loop can spell one.
    let chip = ChipOpts::default();
    for i in 0..CHIPS {
        let row = BAND_TOP + (i / PER_ROW) as u16;
        let col = (i % PER_ROW) as i32 * i32::from(CHIP);
        let area = Rect::new(col, i32::from(row), CHIP, 1);
        let id = Id::keyed(CHIP_ROOT, i as u64);
        let resp = cx.interact(id, area, chip.interest);
        declared += 1;
        let _ = chip_drawn(ink, cx, area, CHIP_LABEL, &resp, &chip);
        if i % PER_ROW == PER_ROW - 1 {
            let filled = PER_ROW as u16 * CHIP;
            let _ = ink.run(cx, i32::from(filled), i32::from(row), " ", W - filled, body);
        }
    }
    for row in BAND_TOP + CHIP_ROWS as u16..H {
        let _ = ink.run(cx, 0, i32::from(row), " ", W, body);
    }

    // **Nothing holds the focus until an application says so** (architecture issue 25). The screen
    // seats it once, in the form that issue settled — `focused()` asked from inside the draw — and
    // the arm that says *this is not the same question as a dismissal* is [`Dismiss::LetItVanish`],
    // where this clause is present and does not fire.
    if cx.focused().is_none() {
        cx.focus(first_chip());
    }

    match config {
        Config::MenuAndSubmenu => request_menu(cx, menu_body_state, Some(submenu)),
        Config::ModalAndScrim => request_dialog(cx, true),
        Config::ModalWithoutTrap => request_dialog(cx, false),
        _ => {}
    }
    declared
}

/// **The menu, and the submenu nested inside it.**
///
/// Two levels of one mechanism: each is [`crate::overlay::overlay`]'s shell over §5's collection, so
/// each declares **one** blur position and **one** entry for its rows, whatever the row count. That
/// is [`MENU_DELTA`], and it is not §12's — see that constant for the subtraction.
///
/// `sub` is an `Option` and this function's own reason for existing: an overlay body is `FnMut`, so
/// it cannot move a capture, and the inner request needs to move a `&'f mut PopupState` into the
/// inner closure. `Option::take` mutates the capture rather than moving it.
fn request_menu<'f>(
    cx: &mut Ctx<'f, '_>,
    menu: &'f mut PopupState,
    sub: Option<&'f mut PopupState>,
) {
    let anchor = Rect::new(0, 0, MENU_TITLE, 1);
    let mut sub = sub;
    cx.overlay(
        Id::keyed(TITLE_ROOT, 0),
        anchor,
        OverlayOpts {
            placement: Placement::BELOW,
            ..OverlayOpts::sized(MENU.0, MENU.1)
        },
        move |cx| {
            let area = cx.area();
            let rows = u32::try_from(MENU_ROWS).unwrap_or(0);
            let mut opens_at = 0usize;
            let _ = overlay(cx, area, rows, &mut |cx, interior| {
                let opts = CollOpts::default();
                let _ = collection(
                    cx,
                    interior,
                    &mut menu.list,
                    &opts,
                    Rows::of(MENU_ROWS),
                    &mut |buf, range: std::ops::Range<usize>| {
                        range.into_iter().find(|&i| ROWS[i].starts_with(buf))
                    },
                    &mut |cx, r, i, face| {
                        let paint = crate::frame::face_paint(cx.theme(), face);
                        let mut ink = Direct;
                        let _ = ink.pad_to(cx, r.x, r.y, ROWS[i], r.w, paint);
                    },
                );
                opens_at = menu.list.sel.lead;
            });
            // **A second overlay from one component mints a second id** (§12, §4). Shared, the two
            // get one slot resized and `Shape::merged` is what would say so.
            //
            // `sub.take()` and not `sub`: this closure is `FnMut`, so moving a capture out of it
            // does not compile at all — see [`Held::submenu`].
            if let Some(state) = sub.take() {
                let row = Rect::new(area.x, area.y + opens_at as i32, area.w, 1);
                cx.overlay(
                    SUBMENU_OWNER,
                    row,
                    OverlayOpts {
                        placement: Placement::RIGHT,
                        ..OverlayOpts::sized(MENU.0, MENU.1)
                    },
                    move |cx| {
                        let a = cx.area();
                        let rows = u32::try_from(MENU_ROWS).unwrap_or(0);
                        let _ = overlay(cx, a, rows, &mut |cx, interior| {
                            let opts = CollOpts::default();
                            let _ = collection(
                                cx,
                                interior,
                                &mut state.list,
                                &opts,
                                Rows::of(MENU_ROWS),
                                &mut |buf, range: std::ops::Range<usize>| {
                                    range.into_iter().find(|&i| ROWS[i].starts_with(buf))
                                },
                                &mut |cx, r, i, face| {
                                    let paint = crate::frame::face_paint(cx.theme(), face);
                                    let mut ink = Direct;
                                    let _ = ink.pad_to(cx, r.x, r.y, ROWS[i], r.w, paint);
                                },
                            );
                        });
                    },
                );
            }
        },
    );
}

/// **The modal dialog: [`crate::overlay::overlay`] at [`Kind::Dialog`], with the trap withheld on
/// one arm.**
///
/// The barrier and the trap are two verbs and the shell puts down both; `trapped` is
/// [`crate::overlay::defective::no_trap`], which is the arm §12's six `Tab`s are measured on.
fn request_dialog(cx: &mut Ctx<'_, '_>, trapped: bool) {
    let theme = *cx.theme();
    cx.overlay(
        dialog_owner(),
        Rect::new(i32::from(W) / 2, i32::from(H) / 2, 1, 1),
        OverlayOpts {
            z: Z::MODAL,
            ..OverlayOpts::modal(DIALOG.0, DIALOG.1, &theme)
        },
        move |cx| {
            let area = cx.area();
            let opts = ShellOpts {
                kind: Kind::Dialog,
                ..ShellOpts::default()
            };
            let id = trap_scope();
            if trapped {
                let _ = overlay_into(&mut Direct, cx, id, area, 0, &opts, |ink, cx, r| {
                    dialog_contents(ink, cx, r);
                });
            } else {
                let _ = crate::overlay::defective::no_trap(
                    &mut Direct,
                    cx,
                    id,
                    area,
                    0,
                    &opts,
                    dialog_contents,
                );
            }
        },
    );
}

/// The dialog's cells and its two buttons, with the trap decided one level up.
fn dialog_contents<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, area: Rect) {
    let title = cx.theme().paint(Role::Title);
    let _ = ink.text(cx, area.x, area.y, DIALOG_TITLE, title);
    let opts = ChipOpts::default();
    for (i, label) in BUTTON_LABELS.iter().enumerate() {
        let r = Rect::new(
            area.x + i as i32 * 12,
            area.y + i32::from(area.h) - 1,
            11,
            1,
        );
        let resp = cx.interact(Id::keyed(BUTTON_ROOT, i as u64), r, opts.interest);
        let _ = chip_drawn(ink, cx, r, label, &resp, &opts);
    }
}

/// **The popup's cells, drawn for the [`Pen`] that prices the fill.**
///
/// The shipped drawing order lives inside `select`'s own body; this is the same two orders written
/// where a [`Pen`] can see them, and [`popup_steady`] is what compares them. One function and two
/// call sites, so the re-damage figure is a measurement of an order rather than of a copy.
///
/// [`Pen`]: crate::runner::Pen
pub fn popup_cells_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    chosen: usize,
    fill_first: bool,
) {
    if area.h == 0 || area.w == 0 {
        return;
    }
    let body = cx.theme().paint(Role::Body);
    let tick = cx.theme().glyph(vitui_runtime::Glyph::Tick);
    // **The defect.** A fill over the whole rectangle before a single row is written, so every cell
    // the rows then write is written twice and re-damaged for as long as the popup stands.
    if fill_first {
        for row in 0..area.h {
            let _ = ink.run(cx, area.x, area.y + i32::from(row), " ", area.w, body);
        }
    }
    for (i, label) in OPTIONS.iter().enumerate() {
        if i >= usize::from(area.h) {
            break;
        }
        let y = area.y + i as i32;
        let mark = if i == chosen { tick } else { " " };
        let m = ink.text(cx, area.x, y, mark, body);
        let _ = ink.text(cx, area.x + i32::from(m), y, " ", body);
        if fill_first {
            let _ = ink.text(cx, area.x + i32::from(MARK), y, label, body);
        } else {
            let _ = ink.pad_to(
                cx,
                area.x + i32::from(MARK),
                y,
                label,
                area.w.saturating_sub(MARK),
                body,
            );
        }
    }
}

// ── the frame structures, measured over a real overlay pass ──────────────────────────────────────

/// **One configuration, measured**: its shape, its per-frame cost and §20's nine counters.
#[derive(Clone, Copy, Debug)]
pub struct Measured {
    /// Which configuration.
    pub config: Config,
    /// What the frame turned out to be.
    pub shape: Shape,
    /// **A report and never a gate.** R15's rule, inherited unchanged, and it is taken with a
    /// [`Tally`] in the loop — see [`cost`] for the figure comparable with §12's own.
    pub per_frame: Duration,
    /// The nine, of which this crate can read eight.
    pub counters: Counters,
}

/// **The screen in one configuration over `frames` steady frames.**
///
/// Two warm-up frames before the clock starts, and they are not slack: the first frame *places* the
/// overlay's layer and the second is the first on which the layer is merely kept, which is the
/// difference between the opening cliff and the steady state that [`opening`] is about.
///
/// # Panics
///
/// Panics on zero frames. A per-frame figure over no frames is a division by zero dressed as a
/// measurement — [`crate::listing::volume_over`]'s refusal, and [`Allocations::over`]'s.
pub fn steady(config: Config, frames: u32, allocations: Allocations) -> Measured {
    assert!(frames > 0, "a per-frame figure needs a frame");
    let mut driver = crate::runner::driver_at(W, H, Density::Compact);
    let mut held = Held::new();
    let mut warm = Tally::new();
    for _ in 0..2 {
        driver.frame(|cx| {
            draw_into(&mut warm, cx, &mut held, config);
        });
    }

    let mut shape = Shape::default();
    let mut tally = Tally::new();
    let mut elapsed = Duration::ZERO;
    for _ in 0..frames {
        let mut frame_tally = Tally::new();
        let mut declared = 0usize;
        let started = Instant::now();
        driver.frame(|cx| declared = draw_into(&mut frame_tally, cx, &mut held, config));
        elapsed += started.elapsed();
        {
            let frame = driver.inspect();
            shape = Shape {
                declared,
                regions: frame.hits().len(),
                stops: frame.stop_count(),
                ring: frame.ring().len(),
                traps: frame.trap_scopes().count(),
                layers: 0,
                bodies: frame.overlay_bodies_boxed(),
                merged: frame.overlays_merged(),
                merges: frame.ids().merges(),
                focused: frame.focused(),
            };
        }
        shape.layers = driver.layers_live();
        tally = frame_tally;
    }
    let counters = Counters::of(&driver, &tally, allocations);
    Measured {
        config,
        shape,
        per_frame: elapsed / frames,
        counters,
    }
}

/// **[`steady`]'s shape alone**, over the two warm-up frames and one measured one.
pub fn shape(config: Config) -> Shape {
    steady(config, 1, Allocations::over(1, 0)).shape
}

/// **What the frame costs a component**, drawn through [`Direct`] rather than through a [`Tally`].
///
/// The one number here comparable with §12's own microseconds, and separate from [`steady`]'s for
/// the reason [`crate::listing::volume_cost`] states: a `Tally` keeps a `BTreeSet` of every cell it
/// sees, so a figure taken with one in the loop is a report about the instrument. The two run the
/// identical [`draw_into`], which is [`crate::ink`]'s whole argument.
///
/// A report and never a gate.
///
/// **The minimum of `frames` and not their mean**, which is `vitui-bench`'s own rule and the
/// prototype's: a mean over a run this short is a report about whatever else the machine was doing,
/// and the deltas §12 states are single microseconds. A minimum is the frame with the least
/// interference in it, and a delta between two minima is the only comparison this instrument can
/// actually make.
///
/// # Panics
///
/// [`steady`]'s, unchanged.
pub fn cost(config: Config, frames: u32) -> Duration {
    assert!(frames > 0, "a per-frame figure needs a frame");
    let mut driver = crate::runner::driver_at(W, H, Density::Compact);
    let mut held = Held::new();
    let mut ink = Direct;
    for _ in 0..2 {
        driver.frame(|cx| {
            draw_into(&mut ink, cx, &mut held, config);
        });
    }
    let mut best = Duration::MAX;
    for _ in 0..frames {
        let started = Instant::now();
        driver.frame(|cx| {
            draw_into(&mut ink, cx, &mut held, config);
        });
        best = best.min(started.elapsed());
    }
    best
}

/// **Several configurations at once, round-robin, minimum of `frames`.**
///
/// The deltas §12 states are single microseconds against a base of hundreds, which is inside this
/// instrument's own run-to-run spread — so [`cost`] called once per configuration measures the drift
/// between the calls as often as it measures the configuration, and the modal's delta has been seen
/// come out **negative** that way. Round-robin is `vitui-bench`'s own answer and the only one that
/// makes a difference of two microseconds mean anything: every configuration's frames are drawn in
/// the same interval, so whatever the machine is doing it is doing to all of them.
///
/// Each configuration keeps its own driver, because the layer the census holds is per configuration
/// and sharing one would put an opening cliff in the middle of a steady measurement.
///
/// A report and never a gate.
///
/// # Panics
///
/// [`cost`]'s, and on an empty configuration list.
pub fn costs(configs: &[Config], frames: u32) -> Vec<Duration> {
    assert!(frames > 0, "a per-frame figure needs a frame");
    assert!(
        !configs.is_empty(),
        "a round-robin over nothing measures nothing"
    );
    let mut drivers: Vec<_> = configs
        .iter()
        .map(|_| {
            (
                crate::runner::driver_at(W, H, Density::Compact),
                Held::new(),
            )
        })
        .collect();
    let mut ink = Direct;
    for ((driver, held), config) in drivers.iter_mut().zip(configs) {
        for _ in 0..2 {
            driver.frame(|cx| {
                draw_into(&mut ink, cx, held, *config);
            });
        }
    }
    let mut best = vec![Duration::MAX; configs.len()];
    for _ in 0..frames {
        for (i, ((driver, held), config)) in drivers.iter_mut().zip(configs).enumerate() {
            let started = Instant::now();
            driver.frame(|cx| {
                draw_into(&mut ink, cx, held, *config);
            });
            best[i] = best[i].min(started.elapsed());
        }
    }
    best
}

// ── the opening frame, which is a cliff and is allowed ───────────────────────────────────────────

/// **What an opening costs, and how many times it costs it.**
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Opening {
    /// How many open/close cycles were played.
    pub cycles: u32,
    /// **How many frames placed a layer that was not alive the frame before.** The count that makes
    /// *once per opening* a gate rather than a sentence: it is exactly [`Opening::cycles`], whatever
    /// the microseconds do.
    pub cliffs: u32,
    /// The steady frame with the modal down. **A report.**
    pub steady: Duration,
    /// The cheapest opening frame over the cycles. **A report.**
    pub opening: Duration,
    /// Surfaces the layer lifecycle reallocated over the whole run. **A move is 0 and a resize is 1**,
    /// so a dialog that opens thirty times at one size costs one.
    pub reallocs: u64,
}

impl Opening {
    /// The opening frame against the steady one, as a ratio. **A report**, and the number §12 states
    /// as 40.50 → 138.04.
    pub fn ratio(self) -> f64 {
        self.opening.as_secs_f64() / self.steady.as_secs_f64().max(f64::MIN_POSITIVE)
    }
}

/// **Thirty open/close cycles, and the cliff counted rather than timed.**
///
/// §12: *the scrim's real price is the frame it appears on: 40.50 → 138.04 µs, 25 080 cells, over the
/// whole budget — once per opening.* The microseconds are a report here for R15's reason, and the
/// cells are **unreachable** — `marked` is not on the engine's public surface for anyone
/// ([`crate::counters::Counters::marked`]), so 25 080 is a number this crate cannot ask for and does
/// not print as 0.
///
/// **What is a gate is *once per opening***: [`Opening::cliffs`] counts the frames on which the
/// census went from keeping no layer alive to keeping one, and it is the cycle count exactly.
///
/// # Panics
///
/// Panics on zero cycles, for [`steady`]'s reason.
pub fn opening(cycles: u32) -> Opening {
    assert!(
        cycles > 0,
        "an opening cliff over no openings is not a measurement"
    );
    let mut driver = crate::runner::driver_at(W, H, Density::Compact);
    let mut held = Held::new();
    let mut ink = Direct;
    for _ in 0..2 {
        driver.frame(|cx| {
            draw_into(&mut ink, cx, &mut held, Config::Nothing);
        });
    }

    let mut cliffs = 0u32;
    let mut alive = driver.layers_live();
    let mut best = Duration::MAX;
    let mut steady = Duration::MAX;
    for _ in 0..cycles {
        let started = Instant::now();
        driver.frame(|cx| {
            draw_into(&mut ink, cx, &mut held, Config::ModalAndScrim);
        });
        let took = started.elapsed();
        let now = driver.layers_live();
        if alive == 0 && now > 0 {
            cliffs += 1;
            best = best.min(took);
        }
        alive = now;
        // Two closed frames: the first stops requesting, the second is the frame the census has
        // already taken the layer away on.
        for _ in 0..2 {
            let started = Instant::now();
            driver.frame(|cx| {
                draw_into(&mut ink, cx, &mut held, Config::Nothing);
            });
            steady = steady.min(started.elapsed());
            alive = driver.layers_live();
        }
    }
    Opening {
        cycles,
        cliffs,
        steady,
        opening: best,
        reallocs: driver.surface_reallocs(),
    }
}

// ── the two re-damage instruments ────────────────────────────────────────────────────────────────

/// **Where a scrim's cells go**, which is §12's three spellings and not two.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScrimSpelling {
    /// **Filled under the dialog.** The defect: every cell of the dialog is written twice and
    /// re-damaged on every frame the modal is up.
    Under,
    /// **The four rectangles around the dialog.** §2's complement, and there is no cut-out — a
    /// terminal cell has no alpha channel, so a scrim cannot have a hole in it.
    Around,
    /// **The engine's operator layer**, which is what [`OverlayOpts::modal`] asks for and what
    /// [`Config::ModalAndScrim`] stands. It writes no cell of its own, so it costs neither the
    /// damage nor the writes.
    ///
    /// [`OverlayOpts::modal`]: vitui_runtime::overlay::OverlayOpts::modal
    OperatorLayer,
}

impl ScrimSpelling {
    /// All three, in the order §12 states them.
    pub const ALL: [ScrimSpelling; 3] = [
        ScrimSpelling::Under,
        ScrimSpelling::Around,
        ScrimSpelling::OperatorLayer,
    ];

    /// The word the report prints it under.
    pub const fn word(self) -> &'static str {
        match self {
            ScrimSpelling::Under => "filled under the dialog",
            ScrimSpelling::Around => "the four rectangles around it",
            ScrimSpelling::OperatorLayer => "the engine's operator layer",
        }
    }
}

/// The dialog's rectangle, centred, in the base pass's coordinates.
#[must_use]
pub fn dialog_rect() -> Rect {
    Rect::new(
        (i32::from(W) - i32::from(DIALOG.0)) / 2,
        (i32::from(H) - i32::from(DIALOG.1)) / 2,
        DIALOG.0,
        DIALOG.1,
    )
}

/// The popup's rectangle, below the first `select`, in the base pass's coordinates.
#[must_use]
pub fn popup_rect() -> Rect {
    Rect::new(0, 2, POPUP.0, POPUP.1)
}

/// **The scrim and the dialog, written into whatever `ink` is over.** Returns nothing; the counters
/// are the caller's.
pub fn scrim_into<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, spelling: ScrimSpelling) {
    let theme = cx.theme();
    let scrim = theme.paint(Role::Disabled);
    let body = theme.paint(Role::Body);
    let d = dialog_rect();
    match spelling {
        ScrimSpelling::Under => {
            for y in 0..H {
                let _ = ink.run(cx, 0, i32::from(y), " ", W, scrim);
            }
        }
        ScrimSpelling::Around => {
            for y in 0..H {
                let row = i32::from(y);
                if row < d.y || row >= d.bottom() {
                    let _ = ink.run(cx, 0, row, " ", W, scrim);
                } else {
                    let _ = ink.run(cx, 0, row, " ", d.x as u16, scrim);
                    let _ = ink.run(cx, d.right(), row, " ", W - d.right() as u16, scrim);
                }
            }
        }
        ScrimSpelling::OperatorLayer => {}
    }
    for y in 0..d.h {
        let _ = ink.run(cx, d.x, d.y + i32::from(y), " ", d.w, body);
    }
}

/// **What one scrim spelling re-damages, frame after frame.**
///
/// The base pass is drawn once and the modal every frame, which is [`crate::dense::modal_steady`]'s
/// substitution and named there: a scrim and a dialog stand on their own layer over a base pass that
/// has not changed, and a [`Pen`] over one surface cannot follow a body whose origin has moved. It is
/// on all three arms, so it is on the side of none of them.
///
/// # Panics
///
/// Panics under two frames — re-damage is a relation between two — and on a steady state that is not
/// constant.
///
/// [`Pen`]: crate::runner::Pen
pub fn scrim_steady(spelling: ScrimSpelling, frames: u32) -> crate::dense::Redamage {
    run_frames(frames, move |n, pen, cx, held| {
        if n == 0 {
            draw_into(pen, cx, held, Config::Nothing);
        }
        scrim_into(pen, cx, spelling);
    })
}

/// **What one scrim spelling writes on a steady frame**, which is §12's *600 fewer writes*.
pub fn scrim_writes(spelling: ScrimSpelling) -> u64 {
    let mut driver = crate::runner::driver_at(W, H, Density::Compact);
    let mut tally = Tally::new();
    driver.frame(|cx| scrim_into(&mut tally, cx, spelling));
    let mut measured = Tally::new();
    driver.frame(|cx| scrim_into(&mut measured, cx, spelling));
    measured.writes()
}

/// **What the popup re-damages, frame after frame**, in each of its two drawing orders.
///
/// [`scrim_steady`]'s substitution, unchanged and for the same reason.
///
/// # Panics
///
/// [`scrim_steady`]'s, unchanged.
pub fn popup_steady(fill_first: bool, frames: u32) -> crate::dense::Redamage {
    let area = popup_rect();
    run_frames(frames, move |n, pen, cx, held| {
        if n == 0 {
            draw_into(pen, cx, held, Config::Nothing);
        }
        popup_cells_into(pen, cx, area, 0, fill_first);
    })
}

/// The loop both re-damage measurements share. [`crate::dense`]'s `run_frames`, and the two are
/// deliberately not merged: that one is `pub(self)` in a module whose screen is a different screen,
/// and a shared helper across two screens would make a change to one a change to both.
fn run_frames<F>(frames: u32, mut paint: F) -> crate::dense::Redamage
where
    F: for<'f> FnMut(u32, &mut Pen, &mut Ctx<'f, '_>, &'f mut Held),
{
    assert!(frames >= 2, "re-damage is a relation between two frames");
    let mut driver = crate::runner::driver_at(W, H, Density::Compact);
    let mut held = Held::new();
    let mut canvas = Canvas::new(W, H);
    let mut first = 0u64;
    let mut steady = 0u64;
    let mut per_frame: Option<u64> = None;
    for n in 0..frames {
        let mut pen = Pen::over(canvas);
        driver.frame(|cx| paint(n, &mut pen, cx, &mut held));
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
    crate::dense::Redamage {
        frames,
        first,
        steady,
        per_frame: per_frame.unwrap_or_default(),
    }
}

// ── axis A: an overlay that declares and covers its anchor ───────────────────────────────────────

/// The anchor the tooltip hangs off, in the small screen [`tooltip_flips`] plays on.
const TIP_ANCHOR: Id = Id::named("popup.tip.anchor");
/// The small screen's width.
const TIP_W: u16 = 40;
/// The small screen's height.
const TIP_H: u16 = 8;

/// **§12's Axis A, as a count: an overlay that declares *and* covers its anchor takes its own
/// hover — and [`Kind::Transient`] cannot, at any rectangle.**
///
/// The pointer is parked once and never moves. When a *declaring* tooltip's rectangle covers the
/// anchor its own hit entry wins — overlay entries append after the base pass and win by draw order —
/// so the anchor stops being hovered, the tooltip stops being requested, the anchor is hovered again,
/// and the layer flips on and off for ever.
///
/// `covers` is spelled as *the anchor the placement is measured from*, so the two rectangles differ
/// in one number and nothing else. **`kind` is the third arm and it is the component's**: at
/// [`Kind::Transient`] the shell declares nothing at all, so the flip is not available to it — which
/// is what makes Axis A a fact about the kind rather than a warning about placement. *The rectangle
/// is not the fix.*
///
/// # Panics
///
/// Panics under two frames: a flip is a relation between two.
pub fn tooltip_flips(kind: Kind, covers: bool, frames: u32) -> u32 {
    assert!(frames >= 2, "a flip is a relation between two frames");
    let mut driver = crate::runner::driver_at(TIP_W, TIP_H, Density::Compact);
    driver.post_mouse(Mouse {
        x: 8,
        y: 4,
        kind: MouseKind::Move,
        buttons: Buttons::NONE,
        mods: Mods::NONE,
        at: Instant::now(),
    });

    let mut flips = 0u32;
    let mut up: Option<bool> = None;
    for _ in 0..frames {
        driver.frame(|cx| {
            let anchor = Rect::new(4, 4, 8, 1);
            let resp = cx.interact(TIP_ANCHOR, anchor, Interest::HOVER);
            if resp.hovered {
                // **The one rectangle the two placements differ in.** Placed below a band one row
                // higher, the tooltip lands *on* its anchor; placed below the anchor itself it lands
                // under it.
                let from = if covers {
                    Rect::new(4, 3, 8, 1)
                } else {
                    anchor
                };
                cx.overlay(
                    TIP_ANCHOR,
                    from,
                    OverlayOpts {
                        z: Z::TOOLTIP,
                        placement: Placement::BELOW,
                        ..OverlayOpts::sized(8, 1)
                    },
                    move |cx| {
                        let a = cx.area();
                        let opts = ShellOpts {
                            kind,
                            ..ShellOpts::default()
                        };
                        // **The subject.** `overlay` at `Kind::Transient` declares nothing; at
                        // `Kind::Popup` it declares the blur position, which over a rectangle
                        // covering its own anchor is the flip.
                        let _ = overlay_with(cx, a, 1, &opts, &mut |cx, r| {
                            let p = cx.theme().paint(Role::Body);
                            let mut ink = Direct;
                            let _ = ink.run(cx, r.x, r.y, " ", r.w, p);
                        });
                    },
                );
            }
        });
        let now = driver.layers_live() > 0;
        if up.replace(now).is_some_and(|was| was != now) {
            flips += 1;
        }
    }
    flips
}

// ── axis B: the census is over the request ───────────────────────────────────────────────────────

/// **§12's Axis B, as a count: a dialog owned by the menu row that opened it lives 3 of 8 frames.**
///
/// The application believes the dialog open on every one of `frames`; the menu is open on
/// `menu_open`. The dialog's owner is a *menu row*, which exists only while the menu is drawing, so
/// the census — which is over the **request** — keeps the dialog exactly as long as the row asks for
/// it. Returns `(frames the application believes it open, frames its body actually drew)`.
///
/// The second half is read off the frame's own ring rather than out of a counter the body increments:
/// a body is `+ 'f` and cannot borrow anything outside the frame call, which is the same fact
/// ADR 0034 turned into the `n + 1`.
pub fn census(frames: u32, menu_open: std::ops::Range<u32>) -> (u32, u32) {
    let mut driver = crate::runner::driver_at(W, H, Density::Compact);
    let mut lived = 0u32;
    for n in 0..frames {
        let open = menu_open.contains(&n);
        driver.frame(|cx| {
            let mut ink = Direct;
            let _ = ink.run(cx, 0, 0, " ", W, cx.theme().paint(Role::Body));
            let _ = cx.interact(
                Id::keyed(TITLE_ROOT, 0),
                Rect::new(0, 0, MENU_TITLE, 1),
                Interest::CLICK.with(Interest::FOCUS),
            );
            if open {
                cx.overlay(
                    Id::keyed(TITLE_ROOT, 0),
                    Rect::new(0, 0, MENU_TITLE, 1),
                    OverlayOpts {
                        placement: Placement::BELOW,
                        ..OverlayOpts::sized(MENU.0, MENU.1)
                    },
                    |cx| {
                        let a = cx.area();
                        // The row that owns the dialog. It exists only while this body runs.
                        let row = Rect::new(a.x, a.y, a.w, 1);
                        let _ = cx.interact(
                            Id::keyed(MENU_ROOT, 0),
                            row,
                            Interest::CLICK.with(Interest::FOCUS),
                        );
                        cx.overlay(
                            Id::keyed(MENU_ROOT, 0),
                            row,
                            OverlayOpts::sized(DIALOG.0, 3),
                            |cx| {
                                let d = cx.area();
                                let _ = cx.interact(
                                    Id::keyed(BUTTON_ROOT, 0),
                                    Rect::new(d.x, d.y, 8, 1),
                                    Interest::CLICK.with(Interest::FOCUS),
                                );
                            },
                        );
                    },
                );
            }
        });
        let button = Id::keyed(BUTTON_ROOT, 0);
        if driver.inspect().ring_ids().any(|id| id == button) {
            lived += 1;
        }
    }
    (frames, lived)
}

// ── the ring, the trap and the walkthrough ───────────────────────────────────────────────────────

/// **What a keyboard walkthrough did**, and the named exception beside it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Walk {
    /// How many `Tab`s were pressed.
    pub pressed: u32,
    /// How many distinct widgets they reached.
    pub distinct: usize,
    /// How many of the presses landed **inside the dialog**.
    pub inside: usize,
    /// How many stops the frame declared.
    pub declared: usize,
    /// How many traps were standing. **The only thing that can name the exception**.
    pub traps: usize,
}

/// **Press `Tab` `presses` times against `config`, and report where it went.**
///
/// Driven by posting keys and not by reading [`vitui_runtime::ctx::Frame::tab_walk`]: a walkthrough
/// that consulted the frame's own answer would be testing one expression against itself.
///
/// # Panics
///
/// Panics on zero presses.
pub fn walkthrough(config: Config, presses: u32) -> Walk {
    assert!(presses > 0, "a walkthrough of no presses visits nothing");
    let mut driver = crate::runner::driver_at(W, H, Density::Compact);
    let mut held = Held::new();
    let mut ink = Direct;
    for _ in 0..2 {
        driver.frame(|cx| {
            draw_into(&mut ink, cx, &mut held, config);
        });
    }

    let mut visited: Vec<Id> = Vec::new();
    let mut inside = 0usize;
    for _ in 0..presses {
        driver.post_key(crate::keys::press(Chord::new(Code::Tab)));
        loop {
            driver.frame(|cx| {
                draw_into(&mut ink, cx, &mut held, config);
            });
            if driver.queued() == 0 {
                break;
            }
        }
        if let Some(id) = driver.inspect().focused() {
            visited.push(id);
            if is_dialog_button(id) {
                inside += 1;
            }
        }
    }
    let declared = driver.inspect().stop_count();
    let traps = driver.inspect().trap_scopes().count();
    let mut deduped: Vec<u64> = visited.iter().map(|id| id.raw()).collect();
    deduped.sort_unstable();
    deduped.dedup();
    Walk {
        pressed: presses,
        distinct: deduped.len(),
        inside,
        declared,
        traps,
    }
}

/// Whether an id is one of the dialog's two buttons.
fn is_dialog_button(id: Id) -> bool {
    (0..BUTTONS as u64).any(|i| Id::keyed(BUTTON_ROOT, i) == id)
}

// ── the vanish rule, and the three answers to one question ───────────────────────────────────────

/// **How a closing modal hands the keyboard on.** Three spellings, and they are three programs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dismiss {
    /// §12's rule: **on the way out the owner refocuses itself** — one id it already has, so no id
    /// belonging to anybody else is named.
    OwnerRefocusesItself,
    /// Nothing at all, so the **vanish rule** answers: forward through the previous ring from where
    /// the focus was, then backward. The dialog's stops were appended last, so backward is the last
    /// widget of the base pass — [`last_chip`], which the user was nowhere near.
    LetItVanish,
    /// **What the refused runtime rule would have done** (architecture issue 25): seat the first stop
    /// in draw order. It is [`first_stop`], the menu bar's first title — *whatever draws first*.
    AsIfTheRuntimeSeatedTheFirstStop,
}

impl Dismiss {
    /// All three, in the order the module header argues them.
    pub const ALL: [Dismiss; 3] = [
        Dismiss::OwnerRefocusesItself,
        Dismiss::LetItVanish,
        Dismiss::AsIfTheRuntimeSeatedTheFirstStop,
    ];

    /// The word the report prints it under.
    pub const fn word(self) -> &'static str {
        match self {
            Dismiss::OwnerRefocusesItself => "the owner refocuses itself",
            Dismiss::LetItVanish => "nothing, so the vanish rule answers",
            Dismiss::AsIfTheRuntimeSeatedTheFirstStop => "as if the runtime seated the first stop",
        }
    }
}

/// **Where the keyboard is on the frame after a modal closes, and what the vanish rule cost.**
///
/// Three frames with the modal up — the standing trap pulls the focus into it — and then one frame
/// with it down, spelled the way `how` says. Returns `(who holds the focus, slots the vanish rule
/// touched)`.
///
/// **[`Dismiss::LetItVanish`] is the arm that proves the seating clause is a different question.**
/// [`draw_into`] carries `if cx.focused().is_none() { cx.focus(first_chip()) }` — architecture issue
/// 25's settled form — and on the closing frame the focus is *not* `None`, so the clause does not
/// fire and the answer is [`last_chip`] rather than [`first_chip`]. A dismissal is not a seating, and
/// neither is a substitute for the other.
pub fn closing_focus(how: Dismiss) -> (Option<Id>, u64) {
    let mut driver = crate::runner::driver_at(W, H, Density::Compact);
    let mut held = Held::new();
    let mut ink = Direct;
    for _ in 0..3 {
        driver.frame(|cx| {
            draw_into(&mut ink, cx, &mut held, Config::ModalAndScrim);
        });
    }
    driver.frame(|cx| {
        draw_into(&mut ink, cx, &mut held, Config::Nothing);
        match how {
            Dismiss::OwnerRefocusesItself => cx.focus(dialog_owner()),
            Dismiss::LetItVanish => {}
            Dismiss::AsIfTheRuntimeSeatedTheFirstStop => cx.focus(first_stop()),
        }
    });
    let frame = driver.inspect();
    (frame.focused(), frame.vanish_probes())
}

/// **Who the focus is on while the modal is up.** The standing trap's pull, as a value.
pub fn trapped_focus(config: Config) -> Option<Id> {
    let mut driver = crate::runner::driver_at(W, H, Density::Compact);
    let mut held = Held::new();
    let mut ink = Direct;
    for _ in 0..3 {
        driver.frame(|cx| {
            draw_into(&mut ink, cx, &mut held, config);
        });
    }
    driver.inspect().focused()
}

// ── the subjects, and the scan that says whether they are here ───────────────────────────────────

/// **The two components this scene is a scene of, and both are declared.**
pub const SUBJECTS: [&str; 2] = ["select", "overlay"];

/// Where [`SUBJECTS`] are declared, as `(module file, the declaration)`.
///
/// The homes are the freeze's, joined through [`crate::Family`]: `select`'s first family is
/// `F6Input`, whose module is `input.rs`; `overlay`'s is `F9Overlays`, whose module is `overlay.rs`.
/// A component is `fn(&mut Ctx, Rect, …) -> Response` (spec §1, rule 1), so the thing to look for is
/// a public function of the component's own name in its own family's module.
pub const DECLARATIONS: [(&str, &str); 2] = [
    ("input.rs", "pub fn select<'f>("),
    ("overlay.rs", "pub fn overlay("),
];

/// **The needle this list carried while the scene was red, and why it could never have matched.**
///
/// It read `pub fn select(`, and spec §1 already said it could not: *the fifth component opens an
/// overlay, and `'f` costs it two annotations*. A `select` cannot be written without them — its body
/// captures the caller's [`PopupState`] and its option list, and a body is `+ 'f` — so the lifetime
/// parameter sits between the name and the parenthesis and the scan reads *undeclared* about a
/// component that is right there.
///
/// It is recorded rather than quietly corrected because it is the same trap `crate::collect`'s own
/// note describes one family over — *a generic spelling puts `<F, R>` between the name and the
/// parenthesis* — and because a scene that had gone green with the old needle would have been a scene
/// that went green by deleting the lifetime, which is a different component.
pub const NEEDLE_WHILE_RED: &str = "pub fn select(";

/// **Which of [`SUBJECTS`] this crate actually declares. Today: both.**
///
/// A source scan and not a `use`, for [`crate::dense::subjects_declared`]'s reason: *the item does not
/// exist* has no expression, and a `compile_fail` fence would pass today and pass again the day
/// somebody renames the module. The predicate is `crate::dense::declares` and it is shared rather
/// than copied — one definition of *a line that is not a comment*.
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

/// **Whether the overlay family's screen stands on its subjects, as a verdict rather than a
/// sentence.**
///
/// `Met` over two since **components 26**. Everything below this line is drawn *through*
/// [`crate::input::select`] and [`crate::overlay::overlay`]: the two shut widgets, the open
/// dropdown's shell and list, the menu and its submenu, the modal's barrier and trap, and all four
/// spellings §12 refuses.
pub fn standing() -> Verdict {
    let declared = subjects_declared();
    Verdict::of(
        SUBJECTS.len(),
        SUBJECTS.len() - declared.len(),
        "`select` or `overlay` has stopped being declared where the freeze homes it, so this screen \
         is back to a stand-in label in a rectangle and a body written beside the request. Every \
         other number on it would go on reproducing, which is what makes the scan the gate",
        "components 26",
    )
}

/// **The sentence a scene waiting for its subject fails with**, or `None` once both are declared.
///
/// [`crate::listing::owed_message`]'s arrangement, and it takes the declaration list as an argument
/// for the same reason: the crate is on the other side of it now, and the hostile case is still one
/// call away.
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
         its 317 regions and 316 stops are counted, every delta of §12's table reproduces, the \
         walkthrough's exception is named at 2 of 318, the opening cliff is counted once per \
         opening and the scrim's three spellings are priced — see `crate::popup::tests`. Inverted \
         by `components 26`",
        owed.len(),
        SUBJECTS.len(),
        owed.join(", "),
    ))
}

/// **Fail with the subjects that are missing, the files they belong in, and the ticket.**
///
/// # Panics
///
/// Panics if either subject stops being declared where the freeze homes it. Components ticket 26
/// inverted it; it stays inverted only while both files still carry their declaration.
pub fn assert_stands_up(scene: &str) {
    if let Some(message) = owed_message(&subjects_declared(), scene) {
        panic!("{message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The base row: 317 regions and 316 stops, and the difference has a name.**
    ///
    /// §12's first row, and the whole reason the deltas below land on §12's own numbers. The bar is
    /// in the hit index and not in the ring, which is what a region that is not a stop *is*.
    #[test]
    fn the_base_screen_is_317_regions_and_316_stops_and_the_bar_is_the_difference() {
        let shape = shape(Config::Nothing);
        assert_eq!(
            shape.declared, REGIONS,
            "what the draw believes it declared"
        );
        assert_eq!(shape.regions, REGIONS, "and what the hit index holds");
        assert_eq!(shape.stops, STOPS);
        assert_eq!(
            shape.regions - shape.stops,
            1,
            "one region of the base pass is not a tab stop"
        );

        let mut driver = crate::runner::driver_at(W, H, Density::Compact);
        let mut held = Held::new();
        let mut ink = Direct;
        driver.frame(|cx| {
            draw_into(&mut ink, cx, &mut held, Config::Nothing);
        });
        let frame = driver.inspect();
        assert!(
            !frame.ring_ids().any(|id| id == BAR),
            "the bar declares `Interest::HOVER` and is not in the ring"
        );
        assert!(
            frame.ring_ids().any(|id| id == first_stop()),
            "and its titles are"
        );
    }

    /// **Every one of §12's five rows reproduces, and so do the four beside them.**
    ///
    /// The regions and the stops are not a report: they are a property of what the frame declared,
    /// which is what makes them the two columns of §12's table this crate can be held to.
    #[test]
    fn every_configuration_declares_what_the_specs_table_says() {
        for config in Config::ALL {
            let shape = shape(config);
            // **The merge counts come first, and the order is the finding.** With `LIST` and `BLUR`
            // unkeyed the two popups' bodies claim one another's ids: `merges` reads 2 and the
            // regions read 319 where 321 is right. Asserted the other way round, the diagnostic
            // names the symptom — *two fewer regions than expected* — and a reader goes looking at
            // the popup's declaration list instead of at its ids.
            assert_eq!(
                shape.merges,
                0,
                "{}: a claim landed on an id that had already claimed. Two overlay bodies are one \
                 function, so the ids inside one must be keyed — see `LIST`",
                config.word()
            );
            assert_eq!(
                shape.merged,
                0,
                "{}: two overlay requests named one owner, which is one slot resized",
                config.word()
            );
            assert_eq!(
                (shape.regions, shape.stops),
                config.expected(),
                "{}",
                config.word()
            );
            assert_eq!(
                shape.ring,
                shape.stops,
                "{}: nothing here opens a `Group`, so every ring entry is a stop",
                config.word()
            );
        }
    }

    /// **The three deltas, each a different sentence of §12.**
    #[test]
    fn the_three_deltas_are_the_familys_own_arithmetic() {
        let base = shape(Config::Nothing);
        let delta = |c: Config| {
            let s = shape(c);
            (s.regions - base.regions, s.stops - base.stops)
        };
        assert_eq!(delta(Config::OneSelect), DROPDOWN_DELTA);
        assert_eq!(delta(Config::MenuAndSubmenu), MENU_DELTA);
        assert_eq!(delta(Config::ModalAndScrim), MODAL_DELTA);
        // The fill-first spelling changes the drawing order and nothing the frame can see, which is
        // exactly why it needed a second instrument.
        assert_eq!(delta(Config::FillFirst), DROPDOWN_DELTA);
    }

    /// **A popup at `h = 0`: 321 / 318 when its body declares before it looks, 317 / 316 when it
    /// looks first.**
    ///
    /// §12's *the size may not come from the drawn extent* is what puts a popup at `(20, 0)`, and
    /// this is what it costs. Both `select`s request, so it is [`DROPDOWN_DELTA`] twice.
    #[test]
    fn a_popup_at_h_zero_declares_321_against_317_when_its_body_declares_before_it_looks() {
        let declaring = shape(Config::ClosedDeclares);
        let silent = shape(Config::ClosedSilent);
        assert_eq!((declaring.regions, declaring.stops), CLOSED_DECLARES);
        assert_eq!((declaring.regions, declaring.stops), (321, 318));
        assert_eq!((silent.regions, silent.stops), (REGIONS, STOPS));
        assert_eq!(
            (declaring.layers, silent.layers),
            (2, 2),
            "both arms request, and a request at h = 0 is still a layer"
        );
    }

    /// **What §5's one-entry-per-collection is worth here**, which is why 321 is *small*.
    ///
    /// §12 says the closed case is small *only because a collection declares one hit entry however
    /// many rows it has*. The other spelling is [`Config::PerRow`], and the difference is the popup's
    /// height.
    #[test]
    fn one_hit_entry_per_collection_is_what_keeps_the_open_case_small() {
        let one = shape(Config::OneSelect);
        let per_row = shape(Config::PerRow);
        // **On top of, not instead of**: the rows are drawn by the collection either way, so what a
        // per-row hover costs is exactly the loop beside it.
        assert_eq!(per_row.regions - one.regions, PER_ROW_ENTRIES);
        assert_eq!(per_row.stops - one.stops, PER_ROW_ENTRIES);
        assert_eq!(
            (per_row.regions, per_row.stops),
            (327, 325),
            "eight entries and eight stops on top of the one and the one the rule spends"
        );
    }

    /// **The census keeps one layer an overlay, and §12's column counts two.**
    ///
    /// The finding, as a table: every request in the prototype carried a shadow, so its host held
    /// twice what the shipped one does — and the ratio between a dropdown and a menu with its
    /// submenu survives the difference.
    #[test]
    fn the_census_keeps_one_layer_an_overlay_where_section_twelve_counts_two() {
        let measured = [
            shape(Config::Nothing).layers,
            shape(Config::OneSelect).layers,
            shape(Config::MenuAndSubmenu).layers,
            shape(Config::ModalAndScrim).layers,
        ];
        for (row, got) in LAYERS.iter().zip(measured) {
            assert_eq!(row.2, got, "{}", row.0);
        }
        assert_eq!(measured, [0, 1, 2, 1]);
        assert_eq!(
            LAYERS.map(|r| r.1),
            [0, 2, 4, 3],
            "and this is what §12 remembers, kept beside the measurement rather than instead of it"
        );
        assert_eq!(
            measured[2],
            measured[1] * 2,
            "a menu with its submenu is twice a dropdown, which is the half of §12's column that \
             survives the shadow going"
        );
    }

    /// **A frame boxes one body an overlay**, which is what the allocation total is a function of.
    ///
    /// The total itself needs the counting allocator and lives in `tests/popup.rs`; this is the count
    /// the total is `+ 1` on.
    #[test]
    fn a_frame_boxes_one_body_an_overlay_and_none_for_a_frame_with_no_overlay() {
        for config in Config::ALL {
            let shape = shape(config);
            assert_eq!(shape.bodies as usize, config.bodies(), "{}", config.word());
            assert_eq!(
                overlay_allocs(shape.bodies as usize),
                if config.bodies() == 0 {
                    0
                } else {
                    config.bodies() + 1
                },
                "{}",
                config.word()
            );
        }
    }

    /// **The base pass writes every cell of the screen exactly once.** §2, as a partition.
    #[test]
    fn the_base_pass_is_a_partition_of_the_screen() {
        let measured = steady(Config::Nothing, 2, Allocations::over(2, 0));
        let writes = measured
            .counters
            .writes
            .measured()
            .expect("writes is reachable");
        let distinct = measured
            .counters
            .distinct
            .measured()
            .expect("distinct is reachable");
        assert_eq!(writes, SCREEN);
        assert_eq!(distinct, SCREEN);
        assert_eq!(writes, distinct, "no cell is written twice");
    }

    /// **The scrim, in three spellings: 600 and 0 and 0.**
    ///
    /// §12: *filled as cells under the dialog, 229 damaged cells for a dialog that is not moving;
    /// drawn as the four rectangles around it, 0 damaged cells and 600 fewer writes; drawn as the
    /// engine's operator layer, neither.*
    ///
    /// **600 and not 229**, and the reason is [`crate::dense::SCRIM_UNDER`]'s: nothing in this dialog
    /// is painted in the scrim's role, so the difference is total, and §2's 229 says that on C01's
    /// screen 371 of the dialog's 600 cells already carried what the scrim wrote. It is the same six
    /// hundred cells and the same constant, because this screen stands the same dialog up.
    #[test]
    fn a_scrim_filled_under_the_dialog_re_damages_six_hundred_and_neither_complement_re_damages_any()
     {
        assert_eq!(scrim_steady(ScrimSpelling::Under, 4).per_frame, SCRIM_UNDER);
        assert_eq!(scrim_steady(ScrimSpelling::Under, 4).per_frame, 600);
        assert_eq!(scrim_steady(ScrimSpelling::Around, 4).per_frame, 0);
        assert_eq!(scrim_steady(ScrimSpelling::OperatorLayer, 4).per_frame, 0);
        assert_eq!(
            SCRIM_UNDER,
            crate::dense::SCRIM_UNDER,
            "one home for the number, and this screen stands the same dialog up"
        );
    }

    /// **600 fewer writes for the complement, and 23 400 fewer again for the operator layer.**
    #[test]
    fn a_scrim_under_the_dialog_costs_six_hundred_writes_more_than_the_complement() {
        let under = scrim_writes(ScrimSpelling::Under);
        let around = scrim_writes(ScrimSpelling::Around);
        let operator = scrim_writes(ScrimSpelling::OperatorLayer);
        assert_eq!(under - around, SCRIM_EXCESS_WRITES);
        assert_eq!(under - around, 600);
        assert_eq!(
            around, SCREEN,
            "the complement plus the dialog is the screen"
        );
        assert_eq!(
            operator,
            u64::from(DIALOG.0) * u64::from(DIALOG.1),
            "the operator layer writes the dialog and nothing else"
        );
    }

    /// **A popup that fills before it writes re-damages its own ink on every frame: 90 against 0.**
    ///
    /// §12 remembers 107 marked cells. `marked` is unreachable from this crate
    /// ([`crate::counters::Counters::marked`]), so what is measured is the re-damage the engine's own
    /// equality filter would price — and the quantity is arithmetic, not a reading: the ink in
    /// [`OPTIONS`] plus the mark on the chosen row.
    #[test]
    fn a_popup_that_fills_before_it_writes_re_damages_its_own_ink_every_frame() {
        assert_eq!(popup_steady(true, 4).per_frame, FILL_FIRST);
        assert_eq!(popup_steady(true, 4).per_frame, 90);
        assert_eq!(popup_steady(false, 4).per_frame, 0);
        assert_eq!(
            FILL_FIRST,
            1 + popup_ink(),
            "the mark on the chosen row, plus every non-blank cell of the eight labels"
        );
        // Counting widths instead of glyphs gives 102, and the twelve between them are the spaces
        // *inside* the labels — cells the fill writes and does not change.
        let widths: u64 = OPTIONS.iter().map(|s| s.len() as u64).sum();
        assert_eq!(widths + 1 - FILL_FIRST, 12);
    }

    /// **The opening frame is a cliff, and *once per opening* is a count rather than a timing.**
    ///
    /// §21's rule: a gate is a count, a ratio, an equality or a compile outcome, and a timing is a
    /// report. So the microseconds are printed by `examples/popup_numbers.rs` and gated by nothing,
    /// and what is asserted here is that the census went from keeping no layer to keeping one exactly
    /// as many times as the modal opened — and that thirty openings at one size cost **no**
    /// reallocation, because a move is 0 and a resize is 1.
    #[test]
    fn the_opening_frame_happens_once_per_opening_and_costs_no_reallocation() {
        let opened = opening(10);
        assert_eq!(opened.cliffs, opened.cycles);
        assert_eq!(opened.reallocs, 0, "the dialog's surface is allocated once");
        assert!(
            opened.opening > Duration::ZERO && opened.steady > Duration::ZERO,
            "both figures are real readings; only their ratio is a report"
        );
    }

    /// **§12's Axis A: an overlay that declares *and* covers its anchor flips 99 times in 100
    /// frames, and a [`Kind::Transient`] cannot flip at either rectangle.**
    ///
    /// Four readings over one function, which is what makes Axis A a fact about the kind rather than
    /// a warning about placement: the declaring arm flips where it covers and settles where it does
    /// not, and the transient settles either way. **The rectangle is not the fix.**
    #[test]
    fn an_overlay_that_declares_and_covers_its_anchor_takes_its_own_hover() {
        assert_eq!(tooltip_flips(Kind::Popup, true, FLIP_FRAMES), FLIPS);
        assert_eq!(tooltip_flips(Kind::Popup, true, FLIP_FRAMES), 99);
        assert_eq!(
            tooltip_flips(Kind::Popup, false, FLIP_FRAMES),
            1,
            "beside its anchor it turns on once and stays on"
        );
        for covers in [true, false] {
            assert_eq!(
                tooltip_flips(Kind::Transient, covers, FLIP_FRAMES),
                1,
                "a transient declares nothing, so it cannot take its own anchor's hover — \
                 covers = {covers}"
            );
        }
    }

    /// **§12's Axis B: a dialog owned by the menu row that opened it lives 3 of 8 frames.**
    ///
    /// The application believes it open on all eight. The census is over the **request**.
    #[test]
    fn a_dialog_owned_by_a_menu_row_lives_three_of_eight_frames() {
        let (believed, lived) = census(CENSUS_FRAMES, 2..5);
        assert_eq!((believed, lived), (CENSUS_FRAMES, DIALOG_LIVES));
        assert_eq!((believed, lived), (8, 3));
    }

    /// **A standing trap pulls the focus in, and six `Tab`s do not take it out.**
    ///
    /// §12's *a modal additionally needs a `Trap`, and without one six `Tab`s leave it.*
    ///
    /// # The untrapped arm is stronger than §12's sentence, and it is what the shipped runtime does
    ///
    /// §12 presumes the focus starts inside the dialog and walks out. On this runtime a standing trap
    /// is also what *pulls* the focus in, so the modal without one never receives the keyboard at
    /// all: the focus stays on the widget the screen seated it on and six `Tab`s walk the base pass,
    /// **0 of 6 inside the dialog**. The observable is the same and the mechanism is one step
    /// earlier, so it is recorded rather than dressed up as six departures.
    #[test]
    fn a_standing_trap_pulls_the_focus_in_and_six_tabs_do_not_take_it_out() {
        assert_eq!(
            trapped_focus(Config::ModalAndScrim),
            Some(first_button()),
            "a standing trap pulls the focus into itself"
        );
        assert_eq!(
            trapped_focus(Config::ModalWithoutTrap),
            Some(first_chip()),
            "and without one the modal never gets it"
        );

        let trapped = walkthrough(Config::ModalAndScrim, TABS);
        assert_eq!(trapped.inside, TABS as usize, "every press stayed inside");
        assert_eq!(trapped.distinct, TRAPPED_VISITS, "and reached two widgets");
        assert_eq!(trapped.traps, 1);

        let loose = walkthrough(Config::ModalWithoutTrap, TABS);
        assert_eq!(loose.inside, 0, "and without a trap, not one of the six");
        assert_eq!(loose.distinct, TABS as usize, "the walk repeated no id");
        assert_eq!(loose.traps, 0);
    }

    /// **§21's third refinement, as a screen: the walk reaches every stop unless a trap is standing.**
    ///
    /// > *R08's* a walkthrough visits every tab stop exactly once *fails on exactly one panel of
    /// > twelve, and correctly: §12's dialog is open and a `Trap` is what a modal is.*
    ///
    /// The gate is the conjunction and the exception is **named** rather than the gate loosened —
    /// `Frame::trap_scopes` is the only thing that can answer it, and this is a second instrument
    /// beside [`walkthrough`]'s posted keys rather than a restatement of it.
    #[test]
    fn the_walk_reaches_every_stop_unless_a_trap_is_standing() {
        let reach = |config: Config| {
            let mut driver = crate::runner::driver_at(W, H, Density::Compact);
            let mut held = Held::new();
            let mut ink = Direct;
            for _ in 0..3 {
                driver.frame(|cx| {
                    draw_into(&mut ink, cx, &mut held, config);
                });
            }
            let frame = driver.inspect();
            (
                frame.tab_walk().count(),
                frame.stop_count(),
                frame.trap_scopes().count(),
            )
        };

        let (walked, stops, traps) = reach(Config::Nothing);
        assert_eq!(traps, 0);
        assert_eq!(walked, stops, "no trap, so the walk reaches every stop");
        assert_eq!(stops, STOPS);

        let (walked, stops, traps) = reach(Config::ModalAndScrim);
        assert_eq!(traps, 1, "and the exception is named rather than assumed");
        assert_eq!(stops, STOPS + MODAL_DELTA.1);
        assert_eq!((walked, stops), (TRAPPED_VISITS, 318));
    }

    /// **Where the keyboard goes when a modal closes: three spellings, three programs.**
    ///
    /// §12 settles the first — *on the way out the owner refocuses itself, so no id belonging to
    /// anybody else is named*. The second is what the vanish rule answers when nobody says anything,
    /// and it is the **last widget of the base pass**, because the dialog's stops were appended last
    /// and the rule walks forward before it walks back. The third is what the refused runtime rule
    /// would have done (architecture issue 25) — *whatever draws first*.
    ///
    /// **The vanish arm is also what proves the seating clause is a different question.**
    /// [`draw_into`] carries `if cx.focused().is_none() { cx.focus(first_chip()) }`, and the answer is
    /// [`last_chip`] rather than [`first_chip`]: on the closing frame the focus is not `None`, so the
    /// clause does not fire. A dismissal is not a seating.
    #[test]
    fn where_the_keyboard_goes_when_a_modal_closes_is_three_different_programs() {
        let landed: Vec<Option<Id>> = Dismiss::ALL.iter().map(|d| closing_focus(*d).0).collect();
        assert_eq!(landed[0], Some(dialog_owner()), "§12's rule");
        assert_eq!(landed[1], Some(last_chip()), "the vanish rule's neighbour");
        assert_eq!(landed[2], Some(first_stop()), "whatever draws first");

        let mut raw: Vec<u64> = landed.iter().flatten().map(|id| id.raw()).collect();
        raw.sort_unstable();
        raw.dedup();
        assert_eq!(raw.len(), 3, "three spellings, three answers");

        assert_ne!(
            landed[1],
            Some(first_chip()),
            "the screen's seating clause is present and does not fire: a dismissal is not a seating"
        );

        // The vanish rule pays for itself only on the frame something vanished.
        assert_eq!(closing_focus(Dismiss::OwnerRefocusesItself).1, 0);
        assert!(
            closing_focus(Dismiss::LetItVanish).1 > 0,
            "the frame the dialog went away on is the one that walks the table"
        );
    }

    /// **The screen stands on its subjects, and the scan is what says so.**
    ///
    /// The inversion of components 25's `the_screen_is_red_because_select_and_overlay_are_not_
    /// declared`, and the direction is the whole point: it was `Unmet { over: 2, failing: 2 }` and
    /// nothing on the screen was wrong. What changed is that every cell of it now comes out of
    /// [`crate::input::select`] and [`crate::overlay::overlay`].
    #[test]
    fn the_screen_stands_on_its_subjects() {
        assert_eq!(
            subjects_declared(),
            SUBJECTS.to_vec(),
            "a subject has stopped being declared where the freeze homes it. That re-reds scene 14 \
             and a register row, and it is a deliberate edit in three files"
        );
        let verdict = standing();
        assert!(verdict.met());
        match verdict {
            Verdict::Met { over } => assert_eq!(over, 2, "two subjects, both declared"),
            Verdict::Unmet { failing, .. } => {
                unreachable!("{failing} undeclared, which the assertion above caught")
            }
        }
        assert_stands_up("a select, a menu, a modal and a scrim");
    }

    /// **The needle the red scene carried could not have matched, and spec §1 said so first.**
    ///
    /// [`NEEDLE_WHILE_RED`] is `pub fn select(`; the component is `pub fn select<'f>(`, because its
    /// body captures the caller's `PopupState` and its option list and a body is `+ 'f`. §1 records
    /// the cost in as many words — *the fifth opens an overlay, and `'f` costs it two annotations* —
    /// so the scan was written against a spelling the spec had already ruled out.
    ///
    /// The gate is both directions: the shipped source does **not** carry the old needle, and it does
    /// carry the new one. A scene that had gone green on the old needle would have gone green by
    /// deleting the lifetime, which is a different component.
    #[test]
    fn the_needle_the_red_scene_carried_could_not_have_matched_a_component_that_opens_an_overlay() {
        let path = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src/input.rs"));
        let source = std::fs::read_to_string(&path).expect("the crate can read its own source");
        assert!(
            !crate::dense::declares(&source, NEEDLE_WHILE_RED),
            "`select` is declared without a lifetime parameter, so its popup cannot be capturing \
             the caller's state and §12's `&'f mut` is gone"
        );
        assert!(crate::dense::declares(&source, DECLARATIONS[0].1));
        // And the two needles are one character apart, so the difference really is the annotation.
        assert!(
            DECLARATIONS[0]
                .1
                .starts_with(&NEEDLE_WHILE_RED[..NEEDLE_WHILE_RED.len() - 1])
        );
    }

    /// **The waiting message says which failure it is**, which is criterion 7 inherited whole.
    #[test]
    fn the_waiting_message_separates_unimplemented_from_wrong() {
        let message = owed_message(&[], "the overlay family").expect("no subject is declared");
        assert!(
            message.contains("waiting for its subject rather than failing"),
            "{message}"
        );
        assert!(
            message.contains("This is not a defect in the screen"),
            "{message}"
        );
        assert!(message.contains("components 26"), "{message}");
        assert!(message.contains("src/input.rs"), "{message}");
        assert!(message.contains("src/overlay.rs"), "{message}");
        assert!(message.contains("pub fn select<'f>("), "{message}");
        assert!(message.contains("2 of 318"), "{message}");

        // The other direction: with both declared there is no message at all.
        assert_eq!(owed_message(&SUBJECTS, "the overlay family"), None);
        // And one of two is still a failure that names the one that is missing.
        let half = owed_message(&["select"], "the overlay family").expect("one is still owed");
        assert!(half.contains("1 of 2"), "{half}");
        assert!(half.contains("`overlay`"), "{half}");
        assert!(!half.contains("`select` ("), "{half}");
    }

    /// **The subject scan finds a declaration when there is one**, which is what makes it a scan
    /// rather than a `false`.
    #[test]
    fn the_subject_scan_finds_a_declaration_when_there_is_one() {
        for (_, declaration) in DECLARATIONS {
            assert!(crate::dense::declares(
                &format!("// a comment\n{declaration}cx: &mut Ctx) {{}}\n"),
                declaration
            ));
            assert!(
                !crate::dense::declares(
                    &format!("// {declaration}…) is components 26's\n"),
                    declaration
                ),
                "a mention in a comment is not a declaration"
            );
        }
        for (file, _) in DECLARATIONS {
            let path =
                std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).join(file);
            assert!(path.is_file(), "{}", path.display());
        }
    }

    /// **The freeze homes both subjects where [`DECLARATIONS`] looks for them.**
    #[test]
    fn the_freeze_homes_select_and_overlay_in_the_files_the_scan_opens() {
        for (subject, (file, _)) in SUBJECTS.into_iter().zip(DECLARATIONS) {
            let component = crate::INVENTORY
                .iter()
                .find(|c| c.id == subject)
                .unwrap_or_else(|| panic!("`{subject}` is in the freeze"));
            let home = component.families[0];
            assert!(home.members().contains(&subject));
            assert_eq!(
                format!("{}.rs", home.module().expect("every family has a module")),
                file
            );
        }

        // **`select` declares two of the four axes and `overlay` none, and scene 14 claims neither**
        // — its `covers` is empty, so O5 is unmoved by this ticket. That is O5's own distinction and
        // not an omission: this screen plays no wheel and no scroll, and what does play a real notch
        // over the shipped component is a *gate* (`crate::overlay::wheeled`). **Building a component
        // cannot move O5; only a scene can.**
        let select = crate::INVENTORY
            .iter()
            .find(|c| c.id == "select")
            .expect("in the freeze");
        assert!(select.declares(crate::Axis::Scrolled));
        assert!(select.declares(crate::Axis::Wheeled));
        assert!(!select.declares(crate::Axis::Shrunk));
        assert!(!select.declares(crate::Axis::Narrow));
        let overlay = crate::INVENTORY
            .iter()
            .find(|c| c.id == "overlay")
            .expect("in the freeze");
        assert!(
            crate::Axis::ALL.iter().all(|a| !overlay.declares(*a)),
            "`overlay` declares no hostile axis, so it can only ever *stand* on a scene"
        );
    }
}
