//! **F9 overlays**, ~35 entries, expressed by `overlay`, layers, placement, scopes and `Trap`.
//!
//! The reduction is R1 and R2 (spec §18): modality is a `bool`, and the eighteen named popup,
//! dialog, drawer, sheet and toast entries are two axes of one component.
//!
//! `select` declares this family and is homed under F6, because §12's finding is that the popup is
//! the *owner's* — [`crate::input::SelectState`] is written only by the owner and
//! [`PopupState`] only by the body, and `&'f mut` is what makes *request the overlay last* a borrow
//! error rather than a comment.
//!
//! # Three kinds on two axes, and the axes are not "modal"
//!
//! [`FAMILY`] is §12's table as a value a test iterates, three rows and three [`Kind`] arms with no
//! fourth on either side. **Modality is one `bool` on the request** —
//! [`OverlayOpts::scrim`](vitui_runtime::overlay::OverlayOpts::scrim) being `Some` — and forces no
//! construction here at all: [`Kind::Dialog`] is a kind because it needs a **host owner, a barrier
//! and a trap**, none of which is modality.
//!
//! | axis | question | who is on the far side |
//! |---|---|---|
//! | A | does it declare anything | [`Kind::Transient`] — a tooltip and a toast are cells and nothing else |
//! | B | is its owner guaranteed to be drawing | [`Kind::Dialog`] — owned by the menu row that opened it, it lives 3 of 8 frames |
//!
//! Axis A is not a preference. An overlay that declares **and** covers its own anchor takes the
//! anchor's hover away, which closes it, which uncovers the anchor, which opens it again:
//! [`crate::popup::FLIPS`] flips in [`crate::popup::FLIP_FRAMES`] frames, for ever.
//!
//! # The rectangle is three parties and the component never hears what it was granted
//!
//! The **anchor** is the component's, in its own coordinates, during its own call. The **size** is
//! the component's too, from a sizing function beside it — [`popup_size`], `CONTEXT.md`'s shape, no
//! draw context. The **placement** is the runtime's
//! [`vitui_runtime::overlay::place`].
//!
//! **The size may not come from the drawn extent.** A popup has no frame before the one it opens
//! on, so its extent there is 0 and stays 0: granted [`SPEC_GRANTED_FROM_EXTENT`] against
//! [`SPEC_GRANTED`], for ever, and the arm is [`crate::input::Sizing::FromTheDrawnExtent`].
//! `CONTEXT.md`'s *one frame old* is survivable for a scroll area and fatal for an overlay.
//!
//! # §9's bar decision moves into the body, and the fixpoint does not arise
//!
//! Because the owner asked for a size and the runtime answered, the gutter is decided **inside the
//! body**, by [`gutter`], in **0 passes** — against [`scroll::MAX_PASSES`] for §9's fixpoint over
//! the same two numbers. A popup owns its own viewport: its horizontal extent *is* the viewport it
//! was granted less the bar, so the two booleans §9 couples have nothing to couple through.
//!
//! Owner-side, the decision is made before the runtime has answered, and on a screen too short
//! [`SPEC_UNREACHABLE`] of [`SHORT_OPTIONS`] rows is unreachable with no bar — because
//! [`vitui_runtime::overlay::place`] **clamps and never resizes**, so an overlay asked for
//! more rows than the screen has hangs off the bottom edge at its stated size and the rows past the
//! edge are drawn, clipped, and reachable by nothing.
//!
//! # One owner is one layer
//!
//! The layer is keyed and censused on the **request**, so a second request under one owner this
//! frame is inert and `Frame::overlays_merged` counts it. A component with two overlays standing
//! must **mint** a second id with `Ctx::with_key` (§4); shared, the two get one slot resized.
//! An `Id` is a hash, so nothing recovers the rooting from the value — every workaround on this map
//! that looks like a hack is that fact.
//!
//! # Dismissal is four things, and blur is qualified by a position
//!
//! [`Dismissal`] is the four, and the one that needs care is [`Dismissal::Blurred`]:
//! `Response::focus_left` is what an outside click already produces, but **`begin` hands out an
//! optimistic focus a frame before the body can speak**, so blur is qualified by a **position** the
//! body reports through one hit entry over the popup's whole rectangle with [`Interest::HOVER`] only
//! — never by a press. The alternative, a catcher layer, is [`Blur::Catcher`]: it covers the screen,
//! declares a click over all of it and **swallows** the click that dismissed it, so the widget the
//! user was aiming at never hears the press.
//!
//! On the way out the owner refocuses **itself** — one id it already has, so no id belonging to
//! anybody else is named. See [`crate::popup::Dismiss`] for the three answers and what the other
//! two cost.
//!
//! # A modal is three mechanisms and the barrier is one of them
//!
//! [`Kind::Dialog`] puts down `Ctx::modal_barrier_here` for the pointer and opens a
//! [`ScopeKind::Trap`] for the keyboard, and **the two are not one verb**: the barrier stops the
//! pointer and only the pointer, and without the trap [`crate::popup::TABS`] `Tab`s walk straight
//! out. The scrim is the third and it is the runtime's operator layer — never a fill. See
//! [`crate::popup::ScrimSpelling`] for the three spellings and what the fill costs.
//!
//! # The sentence `Ctx::overlay` owed is written, and this file is where that is checked
//!
//! Spec §1 states it as an obligation with an owner:
//!
//! > On an overlay body that captures its owner's state by `&mut`, rustc's own `help:` line — *add
//! > explicit lifetime `'f` to the type of `st`* — **compiles**. What then fails is the caller, with
//! > `E0503: cannot use st.open because it was mutably borrowed`, naming the caller's own read one
//! > level away from the mistake and never mentioning the overlay. `Ctx::overlay`'s documentation
//! > owes: *the body answers through the inbox; a `&'f mut` capture compiles and costs you the state
//! > for the rest of the frame.*
//!
//! Components ticket 10 could not write it — it does not touch `crates/vitui-runtime/`, and a note
//! about `Ctx::overlay` written anywhere else is a note nobody hits, because the diagnostic that
//! sends a reader looking arrives at the **caller** and names `E0503` on a field read. Runtime
//! ticket 13's own follow-up wrote it, under the heading *The body answers through the inbox, and a
//! `&'f mut` capture costs you the state*.
//!
//! **A sentence owed on another crate's item is checked by opening that crate's file**, which is
//! [`OWED_SENTENCE`] and [`owed_sentence_is_written`]. A `use` cannot see a doc comment and a
//! `compile_fail` cannot see one either, so the instrument is a scan — the same arrangement
//! [`crate::popup::subjects_declared`] uses one direction over, and the same reason: *the item does
//! not exist* has no expression.

use std::time::Instant;

use vitui_runtime::focus::ScopeKind;
use vitui_runtime::keys::{Chord, Code};
use vitui_runtime::overlay::{OverlayOpts, Placement, place};
use vitui_runtime::sizing::auto_fit;
use vitui_runtime::{
    Button, Buttons, Ctx, Density, Id, Interest, Mods, Mouse, MouseKind, Notch, Rect, Response,
    Role,
};

use crate::collect::CollState;
use crate::ink::{Direct, Ink};
use crate::input::{SelectOpts, SelectState, Sizing, select_into};
use crate::scroll::{self, BarOpts, Orient, Span};

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &["overlay"];

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// the family: three kinds on two axes
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **The three kinds §12's eight entries are, and there is no fourth.**
///
/// The split is not between the eight components. It is [`FAMILY`]'s two columns: *does it declare
/// anything* and *is its owner guaranteed to be drawing*. Everything else about the eight — where it
/// lands, whether it is modal, what it holds — is an argument.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum Kind {
    /// Dropdown, menu, submenu, context menu. **One mechanism.** It declares a blur position, its
    /// owner is drawing whenever it is up, and it is not modal.
    #[default]
    Popup,
    /// Tooltip and toast. **Cells and nothing else** — it declares no region at all, which is Axis
    /// A, and there is no hit entry for a blur to be qualified by.
    Transient,
    /// Dialog and command-palette host. It needs a **host owner** — Axis B — a barrier for the
    /// pointer and a [`ScopeKind::Trap`] for the keyboard.
    Dialog,
}

impl Kind {
    /// All three, in the order the module header argues them.
    pub const ALL: [Kind; 3] = [Kind::Popup, Kind::Transient, Kind::Dialog];

    /// The word a report prints it under.
    pub const fn word(self) -> &'static str {
        match self {
            Kind::Popup => "popup",
            Kind::Transient => "transient",
            Kind::Dialog => "dialog",
        }
    }

    /// **Whether the shell declares a blur position.** [`Kind::Popup`] alone, and it is a narrower
    /// question than Axis A.
    ///
    /// A transient declares nothing at all, which is Axis A. A **dialog** declares plenty — its
    /// buttons — and still has no blur position, for two reasons that point the same way: its
    /// barrier already withholds the pointer from everything outside it, so an entry over its own
    /// rectangle is one nothing can read; and a modal is not dismissed by blur, so there is no
    /// decision for a position to qualify.
    pub const fn has_blur_position(self) -> bool {
        matches!(self, Kind::Popup)
    }
}

/// **One row of §12's family table: a kind, and where it sits on the two axes.**
///
/// A value a test iterates and not a table in a comment, for [`crate::disclose::SPLIT`]'s reason:
/// every obligation stated as a sentence has been broken by someone who had read it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Axes {
    /// Which kind.
    pub kind: Kind,
    /// **Axis A.** Whether it declares an interactive region of its own.
    pub declares: bool,
    /// **Axis B.** Whether its owner is guaranteed to be drawing on every frame it is up.
    pub owner_draws: bool,
    /// The §12 entries this kind is. Their union is the family and nothing appears twice.
    pub members: &'static [&'static str],
}

/// **§12's family, as three rows.** The eight entries, the two axes, and no other structure.
pub const FAMILY: [Axes; 3] = [
    Axes {
        kind: Kind::Popup,
        declares: true,
        owner_draws: true,
        members: &["dropdown", "menu", "submenu", "context_menu"],
    },
    Axes {
        kind: Kind::Transient,
        declares: false,
        owner_draws: true,
        members: &["tooltip", "toast"],
    },
    Axes {
        kind: Kind::Dialog,
        declares: true,
        owner_draws: false,
        members: &["dialog", "command_palette"],
    },
];

/// **How many entries [`FAMILY`] covers: eight, over three kinds.** §12's own list.
pub const ENTRIES: usize = 8;

/// The row of [`FAMILY`] a kind is.
///
/// # Panics
///
/// Cannot: [`FAMILY`] carries every [`Kind::ALL`] arm, and
/// `tests::every_kind_has_exactly_one_row_and_the_members_do_not_overlap` keeps that true.
#[must_use]
pub fn axes(kind: Kind) -> Axes {
    FAMILY
        .into_iter()
        .find(|row| row.kind == kind)
        .expect("every `Kind` has a row in `FAMILY`")
}

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// the size: a sizing function, and the extent that cannot be one
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **The columns a popup spends on its own mark**, before the widest option.
///
/// Two: a tick or a space, and the space after it. The chosen row wears
/// [`Glyph::Tick`](vitui_runtime::Glyph::Tick) and every other row wears a space, so the label
/// column does not move when the choice does.
pub const MARK: u16 = 2;

/// **How large a popup over `options` wants to be, inside `within`. The sizing function.**
///
/// `CONTEXT.md`'s shape exactly: the same data the component takes, plus the room it is about to be
/// given, returning integers. **No draw context** — it cannot draw, cannot claim an identity and
/// cannot route, which is the whole of what makes it a function rather than a method on a trait.
///
/// `within` is what caps it, and the cap is the half that matters:
/// [`vitui_runtime::overlay::place`] clamps a position and **never a size**, so a popup that
/// asked for more rows than the screen has hangs off the edge at its stated size and the rows past
/// it are drawn, clipped, and reachable by nothing. Capping here is what turns *off the bottom* into
/// *scrollable*, and [`gutter`] is the other half.
///
/// ```
/// use vitui_components::overlay::popup_size;
///
/// let options = ["name", "date modified", "size"];
/// // Two columns of mark, then the widest option.
/// assert_eq!(popup_size(&options, (40, 20)), (15, 3));
/// // Three rows do not fit in two, so the popup takes two and owes a bar.
/// assert_eq!(popup_size(&options, (40, 2)), (15, 2));
/// // Narrower than the widest option: the labels truncate, the popup does not overflow.
/// assert_eq!(popup_size(&options, (8, 20)), (8, 3));
/// ```
#[must_use]
pub fn popup_size(options: &[&str], within: (u16, u16)) -> (u16, u16) {
    let widest = auto_fit(options.iter().copied());
    let w = widest.saturating_add(MARK).min(within.0);
    let h = u16::try_from(options.len())
        .unwrap_or(u16::MAX)
        .min(within.1);
    (w, h)
}

/// **What [`popup_size`] grants §12's own dropdown: `(20, 8)`.**
pub const SPEC_GRANTED: (u16, u16) = (20, 8);

/// **What a popup sized from its own drawn extent is granted, on every frame, for ever: `(20, 0)`.**
///
/// §12's *the size may not come from the drawn extent*. The popup has no frame before the one it
/// opens on, so the extent it reads is zero; granted zero rows it draws nothing; having drawn
/// nothing its extent is zero again. See [`crate::input::Sizing::FromTheDrawnExtent`].
pub const SPEC_GRANTED_FROM_EXTENT: (u16, u16) = (20, 0);

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// the gutter, decided once, in the body
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **The gutter decision, with the pass count beside it.**
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Gutter {
    /// Whether a vertical bar stands. It costs a **column**.
    pub bar: bool,
    /// **How many passes it took. Zero, always.** §9's fixpoint does not arise here — see [`gutter`].
    pub passes: u8,
}

impl Gutter {
    /// No bar, no passes — what an empty or transient shell decides.
    pub const NONE: Gutter = Gutter {
        bar: false,
        passes: 0,
    };
}

/// **Whether a popup granted `granted` cells needs a bar for `rows` of content — in no passes.**
///
/// One comparison. §9's [`decide`](scroll::decide) iterates because the axis a bar *reports* and the
/// axis it *costs* are perpendicular, so raising the vertical bar can raise the horizontal one and
/// round again — [`scroll::MAX_PASSES`] worst case. **A popup has no second axis to be coupled
/// through**: its content is as wide as the viewport it was granted, because the labels truncate to
/// it, so reserving a column narrows the labels and changes no row count.
///
/// That is the whole of §12's *0 passes against ≤ 3*, and it is why the decision has to be made here
/// rather than owner-side: the owner does not know what it will be granted.
///
/// ```
/// use vitui_components::overlay::gutter;
/// use vitui_components::scroll::{Hide, MAX_PASSES, decide};
///
/// // Four options in three rows: the bar stands, and it took no passes to know.
/// let g = gutter((20, 3), 4);
/// assert!(g.bar);
/// assert_eq!(g.passes, 0);
/// // Everything fits: no bar, still no passes.
/// assert!(!gutter((20, 8), 4).bar);
/// // §9's own loop over the same two numbers, for the comparison the gate makes.
/// assert_eq!(decide((20, 3), (20, 4), Hide::WhenItFits).passes, MAX_PASSES);
/// ```
#[must_use]
pub const fn gutter(granted: (u16, u16), rows: u32) -> Gutter {
    Gutter {
        bar: rows > granted.1 as u32,
        passes: 0,
    }
}

/// **The options §12's short-screen case is measured over: four.**
pub const SHORT_OPTIONS: usize = 4;

/// **Rows the owner-side spelling leaves unreachable on a screen too short: one of
/// [`SHORT_OPTIONS`].**
///
/// The decision is mechanically in the body either way; what differs is **whose number it is made
/// from**. Sized to the room, the body is granted three rows for four options, [`gutter`] says *bar*
/// and every option is reachable. Sized to the content — [`crate::input::Sizing::ToTheContent`] —
/// the body is granted four rows for four options, says *no bar*, and the fourth row is below the
/// bottom edge of a screen that has three.
pub const SPEC_UNREACHABLE: usize = 1;

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// dismissal
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **The four ways a popup goes away, and there is no fifth.**
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Dismissal {
    /// `Esc`, taken by the body inside its own drain loop and answered through the inbox.
    Escape,
    /// A row was chosen. The choice is the dismissal, and it comes home through the inbox.
    Chose,
    /// **The owner stopped requesting.** Nothing dismisses it: the census is over the request, so a
    /// layer nobody asked for this frame is gone. Axis B is this arm arriving by accident.
    OwnerStopped,
    /// The focus left, **qualified by a position**. See [`Blur`].
    Blurred,
}

impl Dismissal {
    /// All four, in the order §12 lists them.
    pub const ALL: [Dismissal; 4] = [
        Dismissal::Escape,
        Dismissal::Chose,
        Dismissal::OwnerStopped,
        Dismissal::Blurred,
    ];

    /// The word a report prints it under.
    pub const fn word(self) -> &'static str {
        match self {
            Dismissal::Escape => "escape",
            Dismissal::Chose => "chose",
            Dismissal::OwnerStopped => "the owner stopped requesting",
            Dismissal::Blurred => "blurred",
        }
    }
}

/// **How [`Dismissal::Blurred`] is qualified.** Three spellings, and two of them are wrong.
///
/// The vocabulary lives here, with [`Dismissal`]; the two refused arms are played by
/// [`crate::input::defective`], because both are things the **owner** does and neither is expressible
/// inside a popup's own layer — a catcher covers the *screen*, and `cx.area()` inside a body is the
/// popup.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum Blur {
    /// **The rule.** One hit entry over the popup's whole rectangle with [`Interest::HOVER`] only,
    /// so the body can say *the pointer was not over me* on the frame the focus left.
    #[default]
    Position,
    /// **The defect.** Qualified by a press. `begin` hands out an optimistic focus a frame before the
    /// body can speak, so on the frame that matters the press the body would qualify on has not
    /// happened and the popup dismisses itself.
    Press,
    /// **The refused alternative.** A full-screen layer that declares a click over everything, so an
    /// outside press is *reported* rather than inferred. It **swallows** that press: the widget the
    /// user was aiming at never hears it, so dismissing costs a second click.
    Catcher,
    /// **The defect an application found, and it is one clause rather than one spelling.**
    ///
    /// The position alone, without asking the body whether the focus went **inside** the popup. A
    /// popup that takes the keyboard hands it to an id minted in its own body, and the owner's
    /// `Response::focus_left` fires on the frame after — indistinguishable, from the owner's side,
    /// from the user tabbing away. So a popup dismisses itself the frame after it takes the keyboard,
    /// with the arrows dead and nothing on screen having gone wrong. See [`PopupState::inside`].
    PositionAlone,
}

impl Blur {
    /// All four, in the order the module header argues them.
    pub const ALL: [Blur; 4] = [
        Blur::Position,
        Blur::Press,
        Blur::Catcher,
        Blur::PositionAlone,
    ];

    /// The word a report prints it under.
    pub const fn word(self) -> &'static str {
        match self {
            Blur::Position => "a position",
            Blur::Press => "a press",
            Blur::Catcher => "a catcher layer",
            Blur::PositionAlone => "a position, ignoring the handover",
        }
    }
}

/// **§12's remembered price of a catcher layer: 386 912 bytes.**
///
/// Recorded, not reproduced: the engine publishes no cell width and nothing above it can ask for
/// one, so what this crate measures is [`CATCHER_CELLS`] against [`BLUR_CELLS`] — the same ratio,
/// in the unit a component can count.
pub const SPEC_CATCHER_BYTES: usize = 386_912;

/// **§12's remembered price of the blur position: 2 592 bytes.** Recorded, not reproduced.
pub const SPEC_BLUR_BYTES: usize = 2_592;

/// **Cells a catcher layer covers on §12's own screen**, which is every cell of it.
pub const CATCHER_CELLS: u64 = crate::popup::SCREEN;

/// **Cells the blur position covers**, which is the popup and nothing else.
pub const BLUR_CELLS: u64 = crate::popup::POPUP.0 as u64 * crate::popup::POPUP.1 as u64;

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// the popup's own state, which is the body's and has one writer
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **What the body owns and the owner never writes.**
///
/// §12's second half of *two structs, one writer each*.
/// [`crate::input::SelectState`] is the owner's, written only by the owner from its own
/// input and the inbox; this is the body's, borrowed `&'f mut` and written only by the body.
///
/// # The `&'f mut` is the mechanism and not an annotation
///
/// A body is `FnMut(&mut Ctx<'f, '_>) + 'f`, so a `&'f mut` of this satisfies the bound and lives
/// exactly as long as the queue holds the body — **the rest of the frame**. Two things follow, and
/// both are load-bearing:
///
/// * **Requesting the overlay anywhere but last is a borrow error.** Not a comment and not a review
///   habit. [`crate::input::WhyThePopupIsRequestedLast`] is the pair that keeps it one.
/// * **The choice cannot come back by assignment.** The owner cannot read this until the frame
///   after, so the body writes the answer and the owner takes it — which is the inbox, and which is
///   why nothing here has two writers.
///
/// # Why it is not `Copy`
///
/// It holds a [`CollState`], because §5's `Mode` absorbs the listbox the popup *is* and a second row
/// store would be a second place the offset lives. A body that captured a `Copy` offset instead
/// writes into a value that dies with the frame: [`crate::popup::WHEEL_CLICKS`] notches move it
/// **0**, and the screen is identical while it happens. See
/// [`crate::input::defective::a_copy_of_the_offset`].
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct PopupState {
    /// The list: the offset, the cursor, the selection and the type-ahead buffer. §5's, not a second
    /// one.
    pub list: CollState,
    /// **The inbox.** What the body decided, for the owner to take on the frame after. `None` once
    /// taken, so a choice is delivered once.
    answer: Option<usize>,
    /// **What the body was granted, last time it ran.** Reported, never asked for: a popup sized
    /// from this is [`SPEC_GRANTED_FROM_EXTENT`] for ever.
    granted: (u16, u16),
    /// Whether the pointer was over the popup on the frame the body last ran. The **position**
    /// [`Blur::Position`] qualifies on.
    over: bool,
    /// **Whether the focus was inside the popup on the frame the body last ran.**
    ///
    /// The third thing only the body can know, and the owner needs it because
    /// **`Response::focus_left` on the *owner* stops being the signal the moment the popup takes the
    /// keyboard**: the owner no longer holds the focus, so it has none to lose. What a blur *is*, from
    /// the owner's side, is [`SelectState::seated`](crate::input::SelectState::seated) `&& !inside &&
    /// !over` — and the **latch is the owner's**, because the owner is the only thing that knows an
    /// opening has begun.
    inside: bool,
}

impl PopupState {
    /// A popup at the top of its list with nothing chosen.
    #[must_use]
    pub fn new() -> PopupState {
        PopupState::default()
    }

    /// **Take what the body decided.** The owner's half of the inbox, and it empties.
    pub const fn take(&mut self) -> Option<usize> {
        self.answer.take()
    }

    /// **What the body was granted last time it ran.** For a report and for a gate; sizing a popup
    /// from it is the defect [`SPEC_GRANTED_FROM_EXTENT`] names.
    #[must_use]
    pub const fn granted(&self) -> (u16, u16) {
        self.granted
    }

    /// **Whether the pointer was over the popup on the frame the body last ran.**
    #[must_use]
    pub const fn over(&self) -> bool {
        self.over
    }

    /// **Whether the focus was inside the popup on the frame the body last ran.**
    #[must_use]
    pub const fn inside(&self) -> bool {
        self.inside
    }

    /// The body's half of the inbox. `pub(crate)` because only a body in this crate may write it,
    /// and a public setter is a second writer.
    pub(crate) const fn answer(&mut self, at: usize) {
        self.answer = Some(at);
    }

    /// The body reporting what it got, where the pointer was, and whether it holds the keyboard.
    pub(crate) const fn saw(&mut self, granted: (u16, u16), over: bool, inside: bool) {
        self.granted = granted;
        self.over = over;
        self.inside = inside;
    }
}

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// the component
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// [`overlay`]'s options. Spec §1's rule 3: a `Default` struct, never a required builder.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ShellOpts {
    /// Which of the three kinds. [`Kind::Popup`] by default.
    pub kind: Kind,
    /// The role the bar's groove is drawn in.
    pub track: Role,
    /// The role the bar's thumb is drawn in.
    pub thumb: Role,
}

impl Default for ShellOpts {
    fn default() -> ShellOpts {
        ShellOpts {
            kind: Kind::default(),
            track: Role::Border,
            thumb: Role::Face,
        }
    }
}

/// **What [`overlay`] answers: what happened to the blur position, what it handed over, and what it
/// decided about the gutter.**
///
/// Three fields because §1's rule 4 and §2's *the cells it does not write are named in its return
/// value* are two obligations and a shell owes both.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Shell {
    /// What happened to the blur position. Rule 4. **Inert for [`Kind::Transient`]**, which declares
    /// nothing at all — Axis A, as a return value.
    pub response: Response,
    /// **The interior it handed over and did not write**: the rectangle less the bar's column.
    pub interior: Rect,
    /// What it decided about the bar, and in how many passes.
    pub gutter: Gutter,
}

/// **The shell an overlay body draws inside: the blur position, the reserved bar, the barrier and
/// the trap — and not one cell of the interior.**
///
/// **Hostile axes:** none.
///
/// It writes the blur position, the reserved bar, the barrier and the trap, and **not one cell of the
/// interior** — so every axis on this list belongs to the body it wraps rather than to the shell.
///
/// Called **from inside the body**, on the body's own rectangle, which for a popup is `cx.area()`.
/// The three kinds differ in exactly what this function does and in nothing else:
///
/// | [`Kind`] | declares | barrier | trap | bar |
/// |---|---|---|---|---|
/// | [`Kind::Popup`] | the blur position, [`Interest::HOVER`] only | no | no | reserved when [`gutter`] says so |
/// | [`Kind::Transient`] | **nothing** | no | no | no |
/// | [`Kind::Dialog`] | the blur position | **yes**, first statement | **yes** | reserved when [`gutter`] says so |
///
/// `rows` is the content the interior is over, in rows, and it is an **argument** — there is nowhere
/// here to put a closure that could measure one, which is [`crate::scroll::Hide::WhenItFits`]'s rule
/// arriving one family over.
///
/// # The interior is handed over unwritten, and the order is why
///
/// Nothing here fills the interior. A popup that fills its rectangle before it writes its rows
/// re-damages every cell of its own ink on **every steady frame** — [`crate::popup::FILL_FIRST`] of
/// them — and a row is a partition of its width anyway, which is what
/// [`crate::ink::Ink::pad_to`] is for.
///
/// The blur position is declared **before** the body draws, because the press award is a reverse
/// scan of the hit index: a region declared after the widgets it contains is *in front of* them, and
/// every click inside the popup would land on the blur entry instead.
///
/// # The barrier and the trap are two verbs, and a modal needs both
///
/// The barrier stops the pointer and only the pointer. Without the trap beside it
/// [`crate::popup::TABS`] `Tab`s walk straight out of the modal, and that is gated rather than
/// assumed. Neither is reachable from [`ShellOpts`]: a constructor that could put down a barrier
/// would have to run during the draw.
///
/// ```
/// use vitui_components::overlay::{Kind, ShellOpts, overlay, overlay_with};
/// use vitui_runtime::ctx::Driver;
/// use vitui_runtime::{Rect, Role};
///
/// let mut driver = Driver::headless(20, 8).expect("a sink attaches");
/// driver.frame(|cx| {
///     // Eight rows of room over twelve rows of content: the bar stands and costs a column.
///     let shell = overlay(cx, cx.area(), 12, &mut |cx, r| {
///         assert_eq!((r.w, r.h), (19, 8));
///         let paint = cx.theme().paint(Role::Body);
///         cx.text(r.x, r.y, "first", paint);
///     });
///     assert!(shell.gutter.bar);
///     assert_eq!(shell.gutter.passes, 0);
///     assert_eq!(shell.interior, Rect::new(0, 0, 19, 8));
/// });
/// // A transient declares nothing: no hit entry, no tab stop, nothing to take its anchor's hover.
/// driver.frame(|cx| {
///     let opts = ShellOpts { kind: Kind::Transient, ..Default::default() };
///     let shell = overlay_with(cx, cx.area(), 1, &opts, &mut |_cx, _r| {});
///     assert_eq!(shell.interior, Rect::new(0, 0, 20, 8));
/// });
/// assert_eq!(driver.inspect().hits().len(), 0);
/// ```
#[track_caller]
pub fn overlay(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    rows: u32,
    body: &mut dyn FnMut(&mut Ctx<'_, '_>, Rect),
) -> Shell {
    overlay_with(cx, area, rows, &ShellOpts::default(), body)
}

/// [`overlay`], with the options spelled out.
#[track_caller]
pub fn overlay_with(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    rows: u32,
    opts: &ShellOpts,
    body: &mut dyn FnMut(&mut Ctx<'_, '_>, Rect),
) -> Shell {
    let id = cx.id();
    overlay_into(&mut Direct, cx, id, area, rows, opts, |_ink, cx, r| {
        body(cx, r);
    })
}

/// **[`overlay`], drawing through an [`Ink`] and under an id its caller minted.**
///
/// The entry point a gate takes; [`overlay`] is this with [`Direct`]. See [`crate::ink`] for why the
/// seam exists rather than a second implementation written against a `Tally`.
///
/// The id is a parameter for [`crate::scroll::scroll_area_into`]'s reason: a shell drawn on behalf of
/// a component that already claimed an id must not mint a second one, and `#[track_caller]` cannot
/// see through that component's own call.
pub fn overlay_into<I, B>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    id: Id,
    area: Rect,
    rows: u32,
    opts: &ShellOpts,
    body: B,
) -> Shell
where
    I: Ink,
    B: FnMut(&mut I, &mut Ctx<'_, '_>, Rect),
{
    draw_with(ink, cx, id, area, rows, opts, body, Shape::default())
}

/// **The two halves of a modal a shell can be missing**, as one value.
///
/// Not on [`ShellOpts`], because neither is a thing a caller may ask to go without: a dialog with no
/// trap is one six `Tab`s leave and a dialog with no barrier is one a press reaches straight through.
/// Threaded here so the shipped build and either refused one are one call apart.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Shape {
    /// Whether the keyboard's half is put down.
    trap: bool,
    /// Whether the pointer's half is put down.
    barrier: bool,
}

impl Default for Shape {
    fn default() -> Shape {
        Shape {
            trap: true,
            barrier: true,
        }
    }
}

/// The shell, with both refused spellings reachable from one place.
#[expect(
    clippy::too_many_arguments,
    reason = "the shipped signature plus the one value carrying the two refused spellings, so the \
              arms are one call apart"
)]
fn draw_with<I, B>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    id: Id,
    area: Rect,
    rows: u32,
    opts: &ShellOpts,
    mut body: B,
    shape: Shape,
) -> Shell
where
    I: Ink,
    B: FnMut(&mut I, &mut Ctx<'_, '_>, Rect),
{
    if area.is_empty() {
        return Shell {
            response: Response::inert(id, area),
            interior: area,
            gutter: Gutter::NONE,
        };
    }

    // **The barrier is the first statement, and it is the pointer's half of a modal.** The base pass
    // has finished by the time a body runs, so `hits.len()` here is exactly the boundary between
    // *under the modal* and *in it*, and nothing depends on the owner having drawn last.
    if opts.kind == Kind::Dialog && shape.barrier {
        cx.modal_barrier_here();
    }

    // **Axis A, as one branch.** A transient declares nothing at all — no hit entry, no stop, and
    // therefore nothing that could take its own anchor's hover away. `Response::inert` is what says
    // so in the return value rather than in a comment. A dialog declares nothing *here* either, and
    // `Kind::has_blur_position` is where that narrower question is answered.
    let response = if opts.kind.has_blur_position() {
        // **The blur position**: one entry over the whole rectangle, `HOVER` and nothing else, so
        // the body reports a *position* rather than waiting for a press that has not happened. See
        // `Blur` for the two spellings this is not.
        cx.interact(id, area, Interest::HOVER)
    } else {
        Response::inert(id, area)
    };

    // **The gutter, decided from what the body was granted, in no passes.**
    let gutter = gutter((area.w, area.h), rows);
    let interior = if gutter.bar {
        Rect::new(area.x, area.y, area.w.saturating_sub(1), area.h)
    } else {
        area
    };

    if gutter.bar {
        let track = Rect::new(area.x + i32::from(interior.w), area.y, 1, area.h);
        let _ = scroll::bar_into(
            ink,
            cx,
            track,
            Span {
                viewport: u32::from(area.h),
                extent: rows,
                offset: 0,
            },
            &BarOpts {
                orient: Orient::Vertical,
                track: opts.track,
                thumb: opts.thumb,
            },
        );
    }

    // **The trap is the keyboard's half, and it is a scope rather than a flag.** A standing trap
    // pulls the focus into itself, which is where the focus goes when a modal opens, and the walk
    // wraps at its own ends rather than leaving.
    if opts.kind == Kind::Dialog && shape.trap {
        cx.scope(id, ScopeKind::Trap, |cx| body(ink, cx, interior));
    } else {
        body(ink, cx, interior);
    }

    Shell {
        response,
        interior,
        gutter,
    }
}

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// the instruments this ticket's own gates run on
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// The width every instrument below plays at, and the width of the widget the popup drops from.
pub const RIG_W: u16 = 20;
/// A screen with room for a popup: the anchor's row and eight below it.
pub const RIG_H: u16 = 12;
/// **A screen too short for its own popup**, which is what §12's short-screen case is.
pub const SHORT_H: u16 = 3;
/// The four options [`SHORT_OPTIONS`] counts.
pub const SHORT: [&str; SHORT_OPTIONS] = ["alpha", "beta", "gamma", "delta"];

/// The owner every instrument below spells, so that a failure can name it.
pub const OWNER: Id = Id::named("overlay.rig.select");
/// The second owner, for the two-overlays instrument.
pub const SECOND: Id = Id::named("overlay.rig.second");

/// The widget's own rectangle, which is also the popup's anchor.
#[must_use]
pub fn anchor() -> Rect {
    Rect::new(0, 0, RIG_W, 1)
}

/// **What a `select`'s popup is granted over two frames, under one [`Sizing`], on a screen `h` tall.**
///
/// Two frames and not one, because [`Sizing::FromTheDrawnExtent`] is a *fixpoint*: the first frame is
/// granted zero because there is no extent yet, and the second is granted zero because the first drew
/// nothing. One frame cannot tell that apart from an ordinary cold start.
#[must_use]
pub fn granted(sizing: Sizing, h: u16) -> (u16, u16) {
    let mut driver = crate::runner::driver_at(RIG_W, h, Density::Compact);
    let mut st = SelectState::new();
    st.open();
    let mut popup = PopupState::new();
    let opts = SelectOpts::default();
    for _ in 0..2 {
        driver.frame(|cx| {
            let _ = crate::input::defective::sized(
                &mut Direct,
                cx,
                OWNER,
                anchor(),
                &mut st,
                &mut popup,
                &SHORT,
                &opts,
                sizing,
            );
        });
    }
    popup.granted()
}

/// **How many of [`SHORT`]'s rows a keyboard can put on screen, under one [`Sizing`].**
///
/// Arithmetic over the three shipped functions that produce the answer —
/// [`popup_size`], [`vitui_runtime::overlay::place`] and
/// [`CollState::max_offset`] — and not a readback, because there is no readback to take: no cell of a
/// composited surface is readable from outside the engine (ADR 0023), so *the row was drawn where the
/// screen is not* has no observable form at this layer. What **is** observable is the rectangle the
/// row was drawn in, and the two numbers this returns are that rectangle's consequences.
///
/// The behavioural halves of the same fact are beside it and both differ between the arms:
/// [`granted`] reports the height, and [`gutter`] reports whether a bar stands.
#[must_use]
pub fn reachable(sizing: Sizing) -> (usize, usize) {
    let screen = Rect::new(0, 0, RIG_W, SHORT_H);
    let size = match sizing {
        Sizing::ToTheRoom => popup_size(&SHORT, (RIG_W, SHORT_H)),
        Sizing::ToTheContent => (
            popup_size(&SHORT, (RIG_W, u16::MAX)).0,
            u16::try_from(SHORT_OPTIONS).unwrap_or(u16::MAX),
        ),
        // The fixpoint: nothing is ever granted, so nothing is ever reachable.
        Sizing::FromTheDrawnExtent => (popup_size(&SHORT, (RIG_W, u16::MAX)).0, 0),
    };
    // `select` widens a popup to at least its anchor and caps it at the room, and both are `RIG_W`
    // here — so the width is fixed and the height is the whole of what the three arms differ in.
    let size = (RIG_W, size.1);
    let rect = place(anchor(), size, screen, Placement::BELOW);
    // How many of the popup's own rows are inside the screen. `place` clamps a position and never a
    // size, so this is where a popup taller than its screen loses rows.
    let visible =
        usize::try_from((rect.bottom().min(screen.bottom()) - rect.y.max(screen.y)).max(0))
            .unwrap_or(0);
    // The collection clamps its offset against the rectangle it was **granted**, which on the arm
    // that asked for more rows than the screen has is a rectangle partly off it.
    if visible == 0 {
        // Nothing is on screen, so nothing is reachable however far the offset can move — which is
        // the fixpoint arm, and it is worse than the short-screen one rather than better.
        return (0, SHORT_OPTIONS);
    }
    let max_offset = usize::try_from(CollState::max_offset(SHORT_OPTIONS, size.1)).unwrap_or(0);
    ((visible + max_offset).min(SHORT_OPTIONS), SHORT_OPTIONS)
}

/// **What two overlays from one component cost, minted against shared.**
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Sharing {
    /// Requests whose owner had already asked this frame. **Zero, or two overlays share one slot.**
    pub merged: u32,
    /// Layers the census is keeping alive.
    pub layers: usize,
    /// Surfaces the lifecycle reallocated. **A move is 0 and a resize is 1.**
    pub reallocs: u64,
    /// Regions the frame declared.
    pub regions: usize,
}

/// **Two `select`s open at once, under one owner id or under two.**
///
/// §12's *one owner is one layer*. Shared, the second request is inert — the census is keyed on the
/// owner — so one popup is simply absent, one slot is resized between the two sizes every frame, and
/// nothing on the screen says which of the two you are looking at.
#[must_use]
pub fn two_overlays(shared: bool) -> Sharing {
    let mut driver = crate::runner::driver_at(RIG_W * 3, RIG_H, Density::Compact);
    let mut first = SelectState::new();
    let mut second = SelectState::new();
    first.open();
    second.open();
    let mut a = PopupState::new();
    let mut b = PopupState::new();
    let opts = SelectOpts::default();
    let wide: [&str; 2] = ["a much longer option", "b"];
    for _ in 0..3 {
        driver.frame(|cx| {
            let _ = select_into(
                &mut Direct,
                cx,
                OWNER,
                anchor(),
                &mut first,
                &mut a,
                &SHORT,
                &opts,
            );
            let _ = select_into(
                &mut Direct,
                cx,
                if shared { OWNER } else { SECOND },
                Rect::new(i32::from(RIG_W) + 2, 0, RIG_W, 1),
                &mut second,
                &mut b,
                &wide,
                &opts,
            );
        });
    }
    let frame = driver.inspect();
    Sharing {
        merged: frame.overlays_merged(),
        layers: 0,
        reallocs: 0,
        regions: frame.hits().len(),
    }
    .with(driver.layers_live(), driver.surface_reallocs())
}

impl Sharing {
    /// The two counters that live on the driver rather than on the frame.
    const fn with(self, layers: usize, reallocs: u64) -> Sharing {
        Sharing {
            layers,
            reallocs,
            ..self
        }
    }
}

/// **What one blur spelling did to a popup the pointer was standing on, and to the press meant for
/// somebody else.**
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Blurred {
    /// Whether the popup was still standing after the focus moved away with the pointer over it.
    pub survived: bool,
    /// Whether the widget the outside press was aimed at heard it.
    pub press_landed: bool,
    /// Cells the spelling's layers cover, which is what a catcher costs.
    pub cells: u64,
}

/// **One blur spelling, played twice: once with the focus moving away while the pointer is over the
/// popup, once with a press aimed at a widget on the other side of the screen.**
///
/// The two halves are two different questions and each spelling gets exactly one of them wrong.
/// [`Blur::Position`] survives the first and lets the second through. [`Blur::Press`] **dismisses
/// itself** on the first, because `begin`'s optimistic focus arrives a frame before the body can
/// report anything and there is no press on that frame to read. [`Blur::Catcher`] survives the first
/// and **swallows** the second: its layer covers the screen and wins the reverse scan.
#[must_use]
pub fn blurs(how: Blur) -> Blurred {
    let opts = SelectOpts::default();
    let elsewhere = Id::named("overlay.rig.elsewhere");
    let target = Rect::new(0, i32::from(RIG_H) - 1, RIG_W, 1);

    // ── the first half: the focus leaves while the pointer is over the popup ─────────────────────
    let mut driver = crate::runner::driver_at(RIG_W, RIG_H, Density::Compact);
    let mut st = SelectState::new();
    st.open();
    let mut popup = PopupState::new();
    driver.post_mouse(pointer(4, 3, MouseKind::Move));
    for n in 0..5 {
        driver.frame(|cx| {
            if n == 0 {
                cx.focus(OWNER);
            }
            // **A keyboard move and not a click**, so that what leaves the focus is not also the
            // thing a press-qualified arm could have read.
            if n == 3 {
                cx.focus(elsewhere);
            }
            let _ = cx.interact(elsewhere, target, Interest::CLICK.with(Interest::FOCUS));
            let _ = crate::input::defective::blurred(
                &mut Direct,
                cx,
                OWNER,
                anchor(),
                &mut st,
                &mut popup,
                &SHORT,
                &opts,
                how,
            );
        });
    }
    let survived = st.is_open();

    // ── the second half: a press aimed at a widget the popup does not cover ──────────────────────
    let mut driver = crate::runner::driver_at(RIG_W, RIG_H, Density::Compact);
    let mut st = SelectState::new();
    st.open();
    let mut popup = PopupState::new();
    let mut landed = false;
    for n in 0..7 {
        // The cadence `crate::wheel` established: post, then a frame. A press is three frames,
        // because the award is made at `end` and resolved against the previous frame's index.
        match n {
            2 => driver.post_mouse(pointer(4, RIG_H - 1, MouseKind::Move)),
            3 => driver.post_mouse(pointer(4, RIG_H - 1, MouseKind::Down(Button::Left))),
            5 => driver.post_mouse(pointer(4, RIG_H - 1, MouseKind::Up(Button::Left))),
            _ => {}
        }
        driver.frame(|cx| {
            let resp = cx.interact(elsewhere, target, Interest::CLICK.with(Interest::FOCUS));
            landed |= resp.clicked;
            let _ = crate::input::defective::blurred(
                &mut Direct,
                cx,
                OWNER,
                anchor(),
                &mut st,
                &mut popup,
                &SHORT,
                &opts,
                how,
            );
        });
    }
    Blurred {
        survived,
        press_landed: landed,
        cells: match how {
            // The screen, plus the popup's own rectangle over it.
            Blur::Catcher => {
                u64::from(RIG_W) * u64::from(RIG_H)
                    + u64::from(RIG_W) * u64::try_from(SHORT_OPTIONS).unwrap_or(0)
            }
            Blur::Position | Blur::Press | Blur::PositionAlone => {
                u64::from(RIG_W) * u64::try_from(SHORT_OPTIONS).unwrap_or(0)
            }
        },
    }
}

/// One mouse event at a cell, with nothing held and no modifiers.
fn pointer(x: u16, y: u16, kind: MouseKind) -> Mouse {
    Mouse {
        x,
        y,
        kind,
        buttons: Buttons::NONE,
        mods: Mods::NONE,
        at: Instant::now(),
    }
}

/// **Whether a press reaches the base pass through a modal, with the barrier and without it.**
///
/// The pointer's half of a modal, on its own. The keyboard's half is
/// [`crate::popup::walkthrough`], and **the two are not one verb**: the arm below without a barrier
/// still has its trap, and the arm in `walkthrough` without a trap still has its barrier.
#[must_use]
pub fn barrier_withholds_the_pointer(present: bool) -> bool {
    let under = Id::named("overlay.rig.under");
    let shell = Id::named("overlay.rig.shell");
    let mut driver = crate::runner::driver_at(RIG_W, RIG_H, Density::Compact);
    let mut reached = false;
    for n in 0..7 {
        match n {
            2 => driver.post_mouse(pointer(2, RIG_H - 1, MouseKind::Move)),
            3 => driver.post_mouse(pointer(2, RIG_H - 1, MouseKind::Down(Button::Left))),
            5 => driver.post_mouse(pointer(2, RIG_H - 1, MouseKind::Up(Button::Left))),
            _ => {}
        }
        driver.frame(|cx| {
            let resp = cx.interact(
                under,
                Rect::new(0, i32::from(RIG_H) - 1, RIG_W, 1),
                Interest::CLICK.with(Interest::FOCUS),
            );
            reached |= resp.clicked;
            cx.overlay(
                Id::named("overlay.rig.modal"),
                Rect::new(0, 0, 1, 1),
                OverlayOpts::sized(RIG_W, 4),
                move |cx| {
                    let area = cx.area();
                    let opts = ShellOpts {
                        kind: Kind::Dialog,
                        ..ShellOpts::default()
                    };
                    let draw = |_ink: &mut Direct, cx: &mut Ctx<'_, '_>, r: Rect| {
                        let _ = cx.interact(
                            Id::named("overlay.rig.button"),
                            Rect::new(r.x, r.y, 6, 1),
                            Interest::CLICK.with(Interest::FOCUS),
                        );
                    };
                    if present {
                        let _ = overlay_into(&mut Direct, cx, shell, area, 0, &opts, draw);
                    } else {
                        let _ = defective::no_barrier(&mut Direct, cx, shell, area, 0, &opts, draw);
                    }
                },
            );
        });
    }
    reached
}

/// **What a keyboard walk through an open popup did.**
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Keyboard {
    /// Whether the popup was standing on the frame after it opened. **The one an open popup that
    /// hands over its keyboard gets wrong**, if the owner reads `focus_left` without asking the body
    /// whether the focus went inside it.
    pub survived_the_handover: bool,
    /// How far the list's cursor moved under the arrows. **Zero if the popup never took the
    /// keyboard**, because the owner's own drain loop eats `Down` as *open me*.
    pub cursor: usize,
    /// What the owner ended up with.
    pub chosen: usize,
    /// Whether the popup is shut at the end.
    pub shut: bool,
    /// Whether the focus is the owner's own id at the end. §12's *the owner refocuses itself*.
    pub focus_is_the_owners: bool,
}

/// **Open a popup with `Enter`, move its cursor with `Down`, and leave it the way `finish` says.**
///
/// The walk §12 describes and the four gates it needs, in one run:
///
/// * the popup **survives the handover** — the frame after it takes the keyboard, which is the frame
///   `Response::focus_left` fires on the owner;
/// * the arrows reach **the list** and not the owner, which is what taking the keyboard is *for*;
/// * `Enter` comes home through the inbox as a choice and `Esc` as a cancellation;
/// * and on the way out the focus is **the owner's own id**, because the id it was on was minted
///   inside a body that will not run again.
#[must_use]
pub fn keyboard(finish: Dismissal) -> Keyboard {
    keyboard_with(finish, Blur::Position)
}

/// [`keyboard`], with the blur spelling stated — which is how the defect an application found is
/// expressible at all. [`Blur::PositionAlone`] does not survive the handover.
#[must_use]
pub fn keyboard_with(finish: Dismissal, blur: Blur) -> Keyboard {
    let mut driver = crate::runner::driver_at(RIG_W, RIG_H, Density::Compact);
    let mut st = SelectState::at(0);
    let mut popup = PopupState::new();
    let opts = SelectOpts::default();
    let mut survived = false;
    for n in 0..10 {
        match n {
            2 => driver.post_key(crate::keys::press(Chord::new(Code::Enter))),
            5 => driver.post_key(crate::keys::press(Chord::new(Code::Down))),
            7 => driver.post_key(crate::keys::press(Chord::new(match finish {
                Dismissal::Chose => Code::Enter,
                _ => Code::Escape,
            }))),
            _ => {}
        }
        driver.frame(|cx| {
            if cx.focused().is_none() {
                cx.focus(OWNER);
            }
            let _ = crate::input::defective::blurred(
                &mut Direct,
                cx,
                OWNER,
                anchor(),
                &mut st,
                &mut popup,
                &SHORT,
                &opts,
                blur,
            );
            // **A second tab stop, and it is what makes `focus_is_the_owners` an assertion.** With
            // one widget on the screen the vanish rule picks the owner anyway, so the gate passed
            // with `select`'s own `cx.focus(id)` **missing** — the defect only shows on a screen that
            // has somewhere else to go. Found by running the application, which has two.
            let _ = cx.interact(
                SECOND,
                Rect::new(0, i32::from(RIG_H) - 1, RIG_W, 1),
                Interest::CLICK.with(Interest::FOCUS),
            );
        });
        // The frame after the handover, which is the frame the owner's `focus_left` fires on.
        if n == 4 {
            survived = st.is_open();
        }
    }
    Keyboard {
        survived_the_handover: survived,
        cursor: popup.list.sel.lead,
        chosen: st.chosen(),
        shut: !st.is_open(),
        focus_is_the_owners: driver.inspect().focused() == Some(OWNER),
    }
}

/// **Whether a popup dismissed one way can be opened again.**
///
/// A sequence no other instrument here plays, because every one of them opens a *fresh* popup: the
/// state the body reports survives the dismissal, so a popup shut by a blur comes back with *the
/// focus is not inside me* still on record. Returns `(open on the frame after reopening, open two
/// frames after)`.
#[must_use]
pub fn reopens(after: Dismissal) -> (bool, bool) {
    let mut driver = crate::runner::driver_at(RIG_W, RIG_H, Density::Compact);
    let mut st = SelectState::new();
    let mut popup = PopupState::new();
    let opts = SelectOpts::default();
    let elsewhere = Id::named("overlay.rig.elsewhere");
    let target = Rect::new(0, i32::from(RIG_H) - 1, RIG_W, 1);
    let mut after_one = false;
    let mut after_two = false;
    for n in 0..12 {
        if n == 2 {
            driver.post_key(crate::keys::press(Chord::new(Code::Enter)));
        }
        // The dismissal, once the popup is up and holding the keyboard.
        if n == 6 {
            match after {
                // A blur: somebody else takes the keyboard, and the pointer is nowhere near.
                Dismissal::Blurred => {}
                _ => driver.post_key(crate::keys::press(Chord::new(Code::Escape))),
            }
        }
        // And the reopening, by the owner's own verb rather than by a key — the focus is elsewhere
        // after a blur, so `Enter` would not reach it.
        if n == 9 {
            st.open();
        }
        driver.frame(|cx| {
            if cx.focused().is_none() {
                cx.focus(OWNER);
            }
            if n == 6 && after == Dismissal::Blurred {
                cx.focus(elsewhere);
            }
            let _ = cx.interact(elsewhere, target, Interest::CLICK.with(Interest::FOCUS));
            let _ = select_into(
                &mut Direct,
                cx,
                OWNER,
                anchor(),
                &mut st,
                &mut popup,
                &SHORT,
                &opts,
            );
        });
        if n == 10 {
            after_one = st.is_open();
        }
        if n == 11 {
            after_two = st.is_open();
        }
    }
    (after_one, after_two)
}

/// A list longer than any screen this rig has, so the popup's own list really can scroll.
pub const LONG: usize = 32;

/// **How far [`crate::popup::WHEEL_CLICKS`] notches move a popup's offset, under one `Holds`.**
///
/// `by_mut_ref` false is §7's literal `Copy`-only body one family over: the body writes into a value
/// that dies with the frame, the owner hands it the same number again, and the screen is identical
/// while it happens.
#[must_use]
pub fn wheeled(by_mut_ref: bool) -> i32 {
    let labels: Vec<String> = (0..LONG).map(|i| format!("option {i}")).collect();
    let options: Vec<&str> = labels.iter().map(String::as_str).collect();
    let mut driver = crate::runner::driver_at(RIG_W, RIG_H, Density::Compact);
    let mut st = SelectState::new();
    st.open();
    let mut popup = PopupState::new();
    let opts = SelectOpts::default();
    driver.post_mouse(pointer(4, 3, MouseKind::Move));
    for n in 0..=crate::popup::WHEEL_CLICKS + 1 {
        // Two frames before the first notch: the layer has to be in the index the notch is routed
        // against, and that index is the previous frame's.
        if n > 1 {
            driver.post_mouse(pointer(4, 3, MouseKind::Wheel(Notch::Down)));
        }
        driver.frame(|cx| {
            let _ = if by_mut_ref {
                select_into(
                    &mut Direct,
                    cx,
                    OWNER,
                    anchor(),
                    &mut st,
                    &mut popup,
                    &options,
                    &opts,
                )
            } else {
                crate::input::defective::a_copy_of_the_offset(
                    &mut Direct,
                    cx,
                    OWNER,
                    anchor(),
                    &mut st,
                    &mut popup,
                    &options,
                    &opts,
                )
            };
        });
    }
    popup.list.offset
}

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// the sentence owed on another crate's item
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **The file the owed sentence had to be written in**, repo-relative.
pub const OWED_IN: &str = "crates/vitui-runtime/src/ctx.rs";

/// **The sentence spec §1 says `Ctx::overlay`'s documentation owes**, as three phrases a scan looks
/// for.
///
/// Four and not one because the obligation is four claims — *the body answers through the inbox*,
/// *a `&'f mut` capture compiles*, *it costs you the state for the rest of the frame*, and **the
/// code of the diagnostic a reader will actually meet** — and a note that carries three of them is
/// a note that has lost the half a reader needs.
///
/// # The fourth arrived with components ticket 36, and it is the one a reader searches for
///
/// O1 states the obligation as *the sentence exists verbatim, **with the `E0503` trap named***, and
/// the two halves are not the same claim. The first three are the mechanism; `E0503` is the string
/// a reader pastes into a search engine at the moment they meet it, because **the diagnostic never
/// mentions the overlay** — it arrives at the *caller*, one level away, naming the next ordinary
/// read of a local whose only unusual property is that a closure three lines up captured it. A note
/// that explains the mechanism perfectly and never prints the code is a note nobody finds.
///
/// **The needle carries its context and is not the bare code**, and that is the difference between
/// a scan and a coincidence. `OWED_IN` is seven thousand lines and `e0503` is five characters: a
/// borrowck code named anywhere else in that file — another item's `# Errors` section, a
/// `compile_fail` note — would satisfy a bare needle the day `Ctx::overlay`'s own paragraph lost
/// the code, and the gate would go on reporting four of four. The other three phrases are sentences
/// that can only be in the note they guard, and this one is spelled to be the same kind of thing.
///
/// They are matched against [`flattened`] and not against the file, because a doc comment is
/// line-wrapped by `rustfmt` at a column nobody chose: in the shipped file *for the rest of the*
/// ends one line and *frame* begins the next, so a literal needle is a gate that fails on a reflow
/// and passes again when somebody reflows it back.
pub const OWED_SENTENCE: [&str; 4] = [
    "the body answers through the inbox",
    "capture compiles",
    "costs the caller that state for the rest of the frame",
    "one level away, as e0503",
];

/// **A source file as one line of prose**: comment markers, emphasis, backticks and line breaks
/// removed, whitespace collapsed, lowercased.
///
/// What makes a scan for a *sentence* survive the formatter. It is deliberately lossy — it is looking
/// for a claim, not for a spelling.
#[must_use]
pub fn flattened(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut space = true;
    for ch in source.chars() {
        match ch {
            '`' | '*' | '/' | '_' => {}
            c if c.is_whitespace() => {
                if !space {
                    out.push(' ');
                    space = true;
                }
            }
            c => {
                out.extend(c.to_lowercase());
                space = false;
            }
        }
    }
    out
}

/// **Whether the sentence is written where it is owed.**
///
/// A scan and not a `use`, because a doc comment is not an item: nothing a compiler can be asked
/// about changes when it is deleted, and a `compile_fail` cannot see one either. It reads the
/// runtime's own file, which makes this a row on this crate's register whose subject is another
/// crate — `crate::popup::subjects_declared`'s arrangement pointed the other way.
#[must_use]
pub fn owed_sentence_is_written() -> Vec<&'static str> {
    let path =
        std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join(OWED_IN);
    owed_sentence_in(&std::fs::read_to_string(&path).unwrap_or_default())
}

/// **Which of [`OWED_SENTENCE`]'s phrases a source carries.** The half of
/// [`owed_sentence_is_written`] that is not a file read.
///
/// Split out by components ticket 36, and not for tidiness: the shipped runtime carries all four,
/// so the gate above cannot be watched reporting a partial answer, and *a note that carries three
/// of the four* is the failure the fourth phrase was added to catch.
#[must_use]
pub fn owed_sentence_in(source: &str) -> Vec<&'static str> {
    let flat = flattened(source);
    OWED_SENTENCE
        .into_iter()
        .filter(|phrase| flat.contains(&flattened(phrase)))
        .collect()
}

/// **The two halves of a modal, each missing once.**
///
/// `pub` for [`crate::frame::defective`]'s reason: an instrument crate's fixtures are part of the
/// instrument, and a gate validated only against a correct build reports zero for the same reason a
/// broken one would.
pub mod defective {
    use super::{Ctx, Id, Ink, Rect, Shape, Shell, ShellOpts, draw_with};

    /// **A dialog with a barrier and no trap.** The pointer is stopped and the keyboard is not:
    /// [`crate::popup::TABS`] `Tab`s walk straight out, and a test asserts they do not on the arm
    /// beside it.
    pub fn no_trap<I, B>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        id: Id,
        area: Rect,
        rows: u32,
        opts: &ShellOpts,
        body: B,
    ) -> Shell
    where
        I: Ink,
        B: FnMut(&mut I, &mut Ctx<'_, '_>, Rect),
    {
        draw_with(
            ink,
            cx,
            id,
            area,
            rows,
            opts,
            body,
            Shape {
                trap: false,
                barrier: true,
            },
        )
    }

    /// **A dialog with a trap and no barrier.** The keyboard cannot leave and the pointer was never
    /// stopped, so a press lands on whatever is under the modal — **the two are not one verb**.
    pub fn no_barrier<I, B>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        id: Id,
        area: Rect,
        rows: u32,
        opts: &ShellOpts,
        body: B,
    ) -> Shell
    where
        I: Ink,
        B: FnMut(&mut I, &mut Ctx<'_, '_>, Rect),
    {
        draw_with(
            ink,
            cx,
            id,
            area,
            rows,
            opts,
            body,
            Shape {
                trap: true,
                barrier: false,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::counters::Tally;
    use crate::scroll::{Hide, MAX_PASSES, decide};
    use vitui_runtime::ctx::Driver;

    /// **§12's family is three kinds over eight entries, and the two axes separate exactly what §12
    /// says they separate.**
    #[test]
    fn the_family_is_three_kinds_on_two_axes_and_eight_entries() {
        assert_eq!(FAMILY.len(), Kind::ALL.len());
        for kind in Kind::ALL {
            assert_eq!(axes(kind).kind, kind, "every kind has its own row");
        }
        let mut seen: Vec<&str> = FAMILY
            .iter()
            .flat_map(|r| r.members.iter().copied())
            .collect();
        assert_eq!(seen.len(), ENTRIES, "§12's eight entries");
        seen.sort_unstable();
        let before = seen.len();
        seen.dedup();
        assert_eq!(seen.len(), before, "no entry belongs to two kinds");

        // Axis A separates the transient and nothing else; Axis B separates the dialog and nothing
        // else. Two axes, three kinds, and the diagonal is what makes them two axes rather than one.
        let silent: Vec<Kind> = FAMILY
            .iter()
            .filter(|r| !r.declares)
            .map(|r| r.kind)
            .collect();
        assert_eq!(silent, vec![Kind::Transient]);
        let hosted: Vec<Kind> = FAMILY
            .iter()
            .filter(|r| !r.owner_draws)
            .map(|r| r.kind)
            .collect();
        assert_eq!(hosted, vec![Kind::Dialog]);

        // And the narrower question is not Axis A: a dialog declares, and still has no blur position.
        assert!(Kind::Popup.has_blur_position());
        assert!(!Kind::Dialog.has_blur_position());
        assert!(!Kind::Transient.has_blur_position());
        assert!(axes(Kind::Dialog).declares);
    }

    /// **A transient declares nothing, and a popup declares one entry that is not a tab stop.**
    ///
    /// Axis A as two counts on one screen. The transient's zero is what makes
    /// [`crate::popup::tooltip_flips`] zero at *any* rectangle: the rectangle is not the fix.
    #[test]
    fn a_transient_declares_nothing_and_a_popups_blur_position_is_not_a_stop() {
        for kind in Kind::ALL {
            let mut driver = Driver::headless(RIG_W, RIG_H).expect("a sink attaches");
            driver.frame(|cx| {
                let opts = ShellOpts {
                    kind,
                    ..ShellOpts::default()
                };
                let shell = overlay_with(cx, cx.area(), 1, &opts, &mut |_cx, _r| {});
                assert!(!shell.response.focused);
            });
            let frame = driver.inspect();
            let expected = usize::from(kind.has_blur_position());
            assert_eq!(frame.hits().len(), expected, "{}", kind.word());
            assert_eq!(
                frame.stop_count(),
                0,
                "{} declares no tab stop",
                kind.word()
            );
        }
    }

    /// **The shell hands its interior over unwritten, and the parts tile the rectangle exactly.**
    ///
    /// §2's second half. With a bar the interior plus the bar's column is the rectangle; without one
    /// the interior *is* the rectangle. Swept over every size a popup can be, because a partition
    /// that holds at one size is an arithmetic coincidence.
    #[test]
    fn the_interior_and_the_bar_tile_the_rectangle_at_every_size() {
        for w in 1u16..=17 {
            for h in 1u16..=17 {
                for rows in [0u32, 1, u32::from(h), u32::from(h) + 1, 4_000] {
                    let mut driver = Driver::headless(w, h).expect("a sink attaches");
                    let mut handed = Rect::new(0, 0, 0, 0);
                    let mut wrote = 0u64;
                    driver.frame(|cx| {
                        let mut tally = Tally::new();
                        let id = cx.id();
                        let shell = overlay_into(
                            &mut tally,
                            cx,
                            id,
                            cx.area(),
                            rows,
                            &ShellOpts::default(),
                            |_, _, _| {},
                        );
                        handed = shell.interior;
                        wrote = tally.writes();
                        assert_eq!(shell.gutter.bar, rows > u32::from(h), "{w}x{h} over {rows}");
                    });
                    let bar = u64::from(u16::from(rows > u32::from(h))) * u64::from(h);
                    assert_eq!(
                        u64::from(handed.w) * u64::from(handed.h) + bar,
                        u64::from(w) * u64::from(h),
                        "{w}x{h} over {rows}: the interior and the bar are not the rectangle"
                    );
                    assert_eq!(
                        wrote, bar,
                        "{w}x{h} over {rows}: the shell wrote the interior"
                    );
                }
            }
        }
    }

    /// **`select`'s shut face is a partition of its rectangle, at every width, truncated or not.**
    ///
    /// §2, the crate's central rule, over the component this ticket built — and it is here because the
    /// diff had a defect it would have caught: the label was padded to the whole width and then the
    /// ellipsis was written **over the pad's last cell**, so one cell of every truncated `select` was
    /// written twice. Nothing on the screen shows it and no other counter moves.
    ///
    /// Swept over widths that straddle the label, because a partition that holds where nothing
    /// truncates is a partition that has not been asked the question.
    ///
    /// **And over heights, which is components 40's half.** *No cell never* was red for nine tickets
    /// and this component was one of the two rows of the freeze that did not meet it: handed a
    /// rectangle taller than its face it wrote **one row of it**, 576 cells of a 48x13 tile left to
    /// whatever was already there. The other is `crate::files::file_picker` — the two overlay owners,
    /// which is the family §12 gives its own rules, and not a coincidence: their body is in another
    /// layer, so a tall rectangle looks to each of them like somebody else's problem.
    ///
    /// **The remainder could not be named**, which is what makes writing it the only answer rather
    /// than the chosen one: §2's third clause is *the cells it does not write are named in its return
    /// value*, and `select_into` returns the runtime's `Response`, which has no field for a
    /// rectangle. `crate::disclose`'s `Disclosure::used` is what naming it looks like where the
    /// return type is this crate's own.
    #[test]
    fn a_shut_selects_face_is_a_partition_of_its_rectangle_at_every_width() {
        for w in 3u16..=24 {
            for h in 1u16..=4 {
                let mut driver = Driver::headless(w, 4).expect("a sink attaches");
                let mut st = SelectState::at(1);
                let mut popup = PopupState::new();
                let opts = SelectOpts::default();
                let mut tally = Tally::new();
                driver.frame(|cx| {
                    let _ = select_into(
                        &mut tally,
                        cx,
                        OWNER,
                        Rect::new(0, 0, w, h),
                        &mut st,
                        &mut popup,
                        &SHORT,
                        &opts,
                    );
                });
                assert_eq!(
                    tally.writes(),
                    u64::from(w) * u64::from(h),
                    "the shut face wrote {} cells into a {w}x{h} rectangle",
                    tally.writes()
                );
                assert_eq!(
                    tally.distinct(),
                    u64::from(w) * u64::from(h),
                    "{w}x{h}: a cell was written twice, which is what padding past the ellipsis does"
                );
            }
        }
    }

    /// **The size comes from a sizing function, and the drawn extent is a fixpoint at zero.**
    ///
    /// §12's *the size may not come from the drawn extent*, as three rows of one table. The
    /// difference between the first two is a **cap**; the difference between either and the third is
    /// that the third never gets off the ground.
    #[test]
    fn the_size_comes_from_a_sizing_function_and_the_drawn_extent_is_a_fixpoint() {
        // Room enough for every option: the two honest arms agree, which is the point — the cap only
        // bites where it has to.
        assert_eq!(granted(Sizing::ToTheRoom, RIG_H).1 as usize, SHORT_OPTIONS);
        assert_eq!(
            granted(Sizing::ToTheContent, RIG_H).1 as usize,
            SHORT_OPTIONS
        );
        // A screen too short: the cap is the whole difference.
        assert_eq!(granted(Sizing::ToTheRoom, SHORT_H), (RIG_W, SHORT_H));
        assert_eq!(
            granted(Sizing::ToTheContent, SHORT_H),
            (RIG_W, u16::try_from(SHORT_OPTIONS).expect("four"))
        );
        // And the fixpoint, on both screens, over two frames — which is what makes it a fixpoint
        // rather than a cold start.
        for h in [RIG_H, SHORT_H] {
            assert_eq!(granted(Sizing::FromTheDrawnExtent, h).1, 0);
        }
        // §12's own pair, in the widths this rig plays at.
        assert_eq!(SPEC_GRANTED.0, RIG_W);
        assert_eq!(SPEC_GRANTED_FROM_EXTENT.1, 0);

        // The sizing function takes no draw context, and this is what that buys: it is callable with
        // nothing running at all.
        assert_eq!(popup_size(&SHORT, (RIG_W, SHORT_H)), (7, SHORT_H));
    }

    /// **The gutter is decided in no passes, and §9's loop over the same two numbers takes three.**
    #[test]
    fn the_gutter_is_decided_in_no_passes_and_section_nines_fixpoint_takes_three() {
        let granted = (RIG_W, SHORT_H);
        let rows = u32::try_from(SHORT_OPTIONS).expect("four");
        let mine = gutter(granted, rows);
        assert!(mine.bar);
        assert_eq!(mine.passes, 0);
        // §9's, over a content whose width is the viewport's — the coupling that makes it iterate.
        let theirs = decide(granted, (u32::from(RIG_W), rows), Hide::WhenItFits);
        assert_eq!(theirs.passes, MAX_PASSES);
        assert!(theirs.passes > mine.passes, "0 against <= 3");
        // Nothing to scroll: still no passes, and no bar.
        assert_eq!(gutter((RIG_W, RIG_H), rows), Gutter::NONE);
    }

    /// **A screen too short leaves no row unreachable, and sizing to the content leaves one.**
    ///
    /// §12's *1 of 4*. The reason it is one and not four is
    /// [`vitui_runtime::overlay::place`]: it clamps a position and never a size, so the popup
    /// is flush with the near edge and hangs off the far one.
    #[test]
    fn a_short_screen_leaves_no_row_unreachable_and_the_owner_side_spelling_leaves_one() {
        assert_eq!(reachable(Sizing::ToTheRoom), (SHORT_OPTIONS, SHORT_OPTIONS));
        let (reached, of) = reachable(Sizing::ToTheContent);
        assert_eq!(of - reached, SPEC_UNREACHABLE);
        assert_eq!((reached, of), (3, 4));
        // The fixpoint arm is worse than the short-screen one rather than better, which a gate that
        // only compared the two honest arms could not have said.
        assert_eq!(reachable(Sizing::FromTheDrawnExtent), (0, SHORT_OPTIONS));
        // And the behavioural halves of the same fact both move.
        assert!(gutter(granted(Sizing::ToTheRoom, SHORT_H), 4).bar);
        assert!(!gutter(granted(Sizing::ToTheContent, SHORT_H), 4).bar);
    }

    /// **One owner is one layer: two overlays from one component must mint a second id.**
    #[test]
    fn one_owner_is_one_layer_and_the_second_overlay_mints_its_own_id() {
        let minted = two_overlays(false);
        let shared = two_overlays(true);
        assert_eq!(minted.merged, 0, "two owners, two layers, nothing merged");
        assert_eq!(minted.layers, 2);
        assert_eq!(shared.merged, 1, "the second request is inert");
        assert_eq!(shared.layers, 1, "one slot for two popups");
        // The screen says nothing: one popup is simply absent, and the regions are the count that
        // notices. **Three and not two**, and the extra one is §12's *an `Id` is a hash* arriving as
        // an arithmetic fact: a `select`'s own id **is** its overlay's owner, so a component cannot
        // share a layer without sharing its own identity — the second widget's hit entry is merged
        // away too. There is no spelling that shares the one and not the other, which is why a second
        // overlay from one component has to *mint* rather than reuse.
        assert_eq!(
            minted.regions - shared.regions,
            crate::popup::DROPDOWN_DELTA.0 + 1,
            "one blur position, one collection, and one whole widget"
        );
        assert_eq!((minted.regions, shared.regions), (6, 3));
    }

    /// **Blur is qualified by a position, and each of the other two spellings loses something else.**
    #[test]
    fn blur_is_qualified_by_a_position_and_the_other_two_spellings_each_lose_one_thing() {
        let position = blurs(Blur::Position);
        let press = blurs(Blur::Press);
        let catcher = blurs(Blur::Catcher);
        // The fourth arm's own failure is on the keyboard axis and not this one — see
        // `an_open_popup_holds_the_keyboard_and_gives_it_back`. Here it behaves like the rule, which
        // is what makes the two instruments two rather than one.
        assert_eq!(blurs(Blur::PositionAlone), position);

        assert!(
            position.survived && position.press_landed,
            "the rule keeps the popup and lets the press through: {position:?}"
        );
        assert!(
            !press.survived,
            "a press-qualified blur dismisses a popup the pointer is standing on: {press:?}"
        );
        assert!(press.press_landed, "and it is the only thing it gets wrong");
        assert!(
            catcher.survived,
            "a catcher does not change what a blur means"
        );
        assert!(
            !catcher.press_landed,
            "and it swallows the press it exists to report: {catcher:?}"
        );

        // What the catcher costs, in the unit this crate can count. §12's bytes are recorded beside
        // it and not reproduced — see `SPEC_CATCHER_BYTES`.
        assert!(catcher.cells > position.cells);
        assert_eq!(
            catcher.cells - position.cells,
            u64::from(RIG_W) * u64::from(RIG_H)
        );
        const { assert!(SPEC_CATCHER_BYTES > SPEC_BLUR_BYTES) };
        assert_eq!(CATCHER_CELLS, crate::popup::SCREEN);
        const { assert!(CATCHER_CELLS > BLUR_CELLS) };

        // Four ways out and no fifth, each with a word a report can print.
        assert_eq!(Dismissal::ALL.len(), 4);
        let mut words: Vec<&str> = Dismissal::ALL.iter().map(|d| d.word()).collect();
        words.sort_unstable();
        words.dedup();
        assert_eq!(words.len(), 4);
        assert_eq!(Blur::ALL.len(), 4);
    }

    /// **An open popup holds the keyboard, and gives it back to its owner on the way out.**
    ///
    /// The gate this ticket's application earned, and the four things it checks are four sentences of
    /// §12 that no other instrument here reaches:
    ///
    /// * the popup **survives the handover** — the frame after it takes the keyboard;
    /// * the arrows reach **the list** and not the owner, whose own drain loop reads `Down` as
    ///   *open me*, so a popup that never took the keyboard has a cursor that never moves;
    /// * `Enter` comes home through the inbox as a choice and `Esc` as a cancellation;
    /// * and the focus ends on **the owner's own id**, because the id it was on was minted inside a
    ///   body that will not run again — left alone, the vanish rule picks a ring neighbour.
    ///
    /// [`Blur::PositionAlone`] is the arm that fails the first two, and it is the defect
    /// `vitui-apps`'s `console` showed: arrows dead, and a popup that vanishes on the next wake with
    /// nothing on screen having gone wrong.
    #[test]
    fn an_open_popup_holds_the_keyboard_and_gives_it_back() {
        let chose = keyboard(Dismissal::Chose);
        assert!(chose.survived_the_handover, "{chose:?}");
        assert_eq!(chose.cursor, 1, "one `Down` reached the list: {chose:?}");
        assert_eq!(chose.chosen, 1, "and `Enter` came home through the inbox");
        assert!(chose.shut && chose.focus_is_the_owners, "{chose:?}");

        let cancelled = keyboard(Dismissal::Escape);
        assert!(cancelled.survived_the_handover, "{cancelled:?}");
        assert_eq!(cancelled.cursor, 1, "the cursor still moved");
        assert_eq!(
            cancelled.chosen, 0,
            "and `Esc` chose nothing: {cancelled:?}"
        );
        assert!(
            cancelled.shut && cancelled.focus_is_the_owners,
            "{cancelled:?}"
        );

        // **The other direction, and it is the whole reason `PopupState::inside` exists.** Without
        // the handover clause the popup is gone the frame after it takes the keyboard, so the arrow
        // that follows lands on nothing at all.
        let forgot = keyboard_with(Dismissal::Chose, Blur::PositionAlone);
        assert!(!forgot.survived_the_handover, "{forgot:?}");
        assert_eq!(forgot.cursor, 0, "the arrows are dead: {forgot:?}");
        assert_eq!(forgot.chosen, 0);
        // And the three that keep the clause are identical to each other on this axis, which is what
        // says the clause and not the qualification is what this gate is about.
        for how in [Blur::Position, Blur::Press, Blur::Catcher] {
            assert_eq!(
                keyboard_with(Dismissal::Chose, how),
                chose,
                "{}",
                how.word()
            );
        }
    }

    /// **A popup dismissed either way can be opened again, and the latch is why.**
    ///
    /// A sequence no other instrument here plays, because every one of them opens a *fresh* popup:
    /// what the body reports survives the dismissal, so a popup shut **by a blur** comes back with
    /// *the focus is not inside me* still on record. The latch lives on the owner and
    /// [`SelectState::open`](crate::input::SelectState::open) clears it, which is what makes the
    /// clearing reach an application that opens the popup by its own verb rather than by a keystroke.
    ///
    /// Found by reading the diff and not by running anything, which is the half of a review that a
    /// gate cannot do for you: every arm of every other instrument here is a first opening.
    #[test]
    fn a_popup_dismissed_either_way_can_be_opened_again() {
        for how in [Dismissal::Blurred, Dismissal::Escape] {
            assert_eq!(
                reopens(how),
                (true, true),
                "a popup dismissed by {} cannot be reopened: the body's last report is still on \
                 record and the blur clause fires before the body has had a frame",
                how.word()
            );
        }
    }

    /// **The barrier stops the pointer and only the pointer.**
    ///
    /// One half each way: this asserts the pointer, and
    /// `crate::popup::tests::the_walk_reaches_every_stop_unless_a_trap_is_standing` asserts the
    /// keyboard. Each arm here has its trap and each arm there has its barrier, which is what makes
    /// *two verbs* a measurement rather than a sentence.
    #[test]
    fn the_barrier_stops_the_pointer_and_only_the_pointer() {
        assert!(
            !barrier_withholds_the_pointer(true),
            "a press reached the base pass through a modal barrier"
        );
        assert!(
            barrier_withholds_the_pointer(false),
            "with the barrier gone the press has to land, or this gate is measuring nothing"
        );
    }

    /// **A `Copy` of the offset moves nothing in twenty notches, and `&'f mut` moves twenty.**
    #[test]
    fn a_copy_only_body_moves_the_offset_nothing_in_twenty_notches() {
        let clicks = i32::try_from(crate::popup::WHEEL_CLICKS).expect("twenty");
        assert_eq!(wheeled(true), clicks, "one row a notch");
        assert_eq!(wheeled(false), 0, "§7's literal `Copy`-only body");
    }

    /// **The sentence `Ctx::overlay` owed is written where it is owed.**
    ///
    /// Spec §1 states it as an obligation with an owner and components ticket 10 could not discharge
    /// it. It is discharged, on the runtime's own item, and this is the scan that says so — a `use`
    /// cannot see a doc comment and neither can a `compile_fail`.
    #[test]
    fn the_sentence_ctx_overlay_owed_is_written_where_it_is_owed() {
        assert_eq!(
            owed_sentence_is_written(),
            OWED_SENTENCE.to_vec(),
            "`Ctx::overlay`'s documentation no longer carries the sentence spec §1 says it owes, in \
             `{OWED_IN}`. The diagnostic a reader meets is `E0503` at their own call site, naming \
             their own read and never mentioning the overlay, so there is nowhere else for this note \
             to be"
        );
        // The file is opened rather than assumed, so a moved item fails here rather than silently.
        let path =
            std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join(OWED_IN);
        assert!(path.is_file(), "{}", path.display());
        // **Watched reporting a partial answer**, which the shipped file cannot be made to do. The
        // three-phrase note is exactly what this scan looked like before components ticket 36
        // widened it, and the phrase it now misses is the one a reader searches for.
        let three = "the body answers through the inbox, a `&'f mut` capture compiles, and it \
                     costs the caller that state for the rest of the frame";
        assert_eq!(owed_sentence_in(three).len(), OWED_SENTENCE.len() - 1);
        assert!(!owed_sentence_in(three).contains(&"one level away, as e0503"));
        assert!(owed_sentence_in("").is_empty());
    }
}
