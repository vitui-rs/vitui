//! **The wheel gate: twenty clicks move the offset twenty, and a reveal fires only when asked.**
//!
//! The glossary forbids the unconditional
//! scroll-into-view.
//!
//! > It fires only for a keyboard-driven focus move. A press already proves the widget was on
//! > screen, and an unconditional pull is the list's old bug: it fights the wheel, dragging the
//! > viewport back to the selection every time the user scrolls away from it.
//! > (`crates/vitui-runtime/src/scroll.rs`)
//!
//! **This is the sharpest instance of the argument the whole components map rests on.** The rule
//! above is written down in `CONTEXT.md`, and **four *resolved* tickets broke it anyway** — four
//! prototypes, each written by someone who had read it. That is why this is a module with a posted
//! pointer in it rather than a line in a review checklist:
//!
//! > Every obligation this map has stated as a sentence has been broken by someone who had read it.
//!
//! # What replaced what, and why the gate moved out of [`crate::listing`]
//!
//! The wheel scene was first built inside the collection's own screen, over a
//! hand-written row loop and with **the click's delta handed to the arithmetic `Response::scrolled`
//! would have delivered it to** — because `Driver::post_mouse` takes a `vitui_engine::Mouse`, and a
//! `Mouse` needs a `Buttons` and a `MouseKind`, and *neither of those was in `ENGINE_NAMES` at all*.
//! That barrier is lifted. So this module is the instrument with
//! the two substitutions taken out, and both removals were load-bearing:
//!
//! 1. **The click is posted.** [`Driver::post_mouse`] with a real [`Notch`], routed through the
//!    previous frame's hit index by `Frame::resolve_wheel`, read by the component inside its own
//!    draw. The arithmetic arm could not have told a component that publishes no scrollable region
//!    from one that does.
//! 2. **The subject is the shipped component.** [`collection_into`] and [`scroll_area`], not a
//!    closure written beside the gate. A gate written against a copy of the code tests the copy
//!    (`crate::ink`), and the copy is exactly where the four resolved tickets' defect was *not*.
//!
//! [`Driver::post_mouse`]: vitui_runtime::Driver::post_mouse
//!
//! # Five subjects, run separately, because the blindness is per axis
//!
//! The fourth criterion: *the same gate runs over `scroll_area` and over a virtualised
//! `collection` separately, because the watermark's blindness is per axis — a body dead downward and
//! alive sideways must be distinguishable.*
//!
//! **The third is the original's and it is a different kind of question.** [`Subject::Table`] owns
//! one offset in rows, like a collection, and every number it reports is a collection's — which is
//! the point: a table's *row axis, wheel, keyboard, type-ahead and
//! reveal are all `collection`'s, reached by calling it*, and until this ticket nothing had asked it
//! a wheel question. The arm costs one `match` arm here and three
//! [`crate::collect::defective`] entries, and what it buys is that the claim is compared rather than
//! trusted: the assertions are *`table` equals `collection`, arm for arm*, not three constants
//! written twice.
//!
//! **The fourth and fifth arrived later, and both are the third's shape on
//! another component.** [`Subject::Pane`] reaches its offset through `scroll_area` and
//! [`Subject::Tree`] reaches its through `collection`, so the assertions on both are *equal to the
//! component it calls, arm for arm* rather than constants written twice. Three instances make it a
//! rule: **a component whose spec says *reached by calling it* is compared against the component it
//! calls.** What each cost is one `match` arm; what the tree arm bought beyond the table's is the
//! **press** clause, because a tree adds a second pointer gesture on the chevron column and *a press
//! still selects a row and still refuses to pull the viewport* is therefore a question rather than
//! an inheritance.
//!
//! A [`Subject::Collection`] owns one offset, in rows. A [`Subject::Area`] owns two, in content
//! cells, and its reveal is the **body's** rather than the component's — `scroll_area` applies the
//! delta and never asks for one. So the unconditional arm over an area names the axis it drags back
//! ([`Play::pull`]), and [`Along::Rows`] against [`Along::Columns`] is the pair the criterion is
//! about: a body that asks for content row 0 every frame is dead to a vertical click and **entirely
//! healthy sideways**, and one number for the offset cannot say so.
//!
//! # The three arms, and why a one-directional gate is not a gate
//!
//! [`Reveal`] has three arms because deleting the call passes the loud half:
//!
//! | arm | the wheel | a keyboard reveal |
//! |---|---|---|
//! | [`Reveal::WhenAsked`] | moves [`MOVED`] | arrives |
//! | [`Reveal::EveryFrame`] | moves [`DRAGGED_BACK`] — **the defect** | arrives |
//! | [`Reveal::Never`] | moves [`MOVED`] | **0** — the keyboard is gone |
//!
//! [`Reveal::Never`] is what a one-directional gate green-lights, and it is not a fix.
//!
//! [`Notch`]: vitui_runtime::Notch
//! [`collection_into`]: crate::collect::collection_into
//! [`scroll_area`]: crate::scroll::scroll_area

use vitui_runtime::keys::{Chord, Code};
use vitui_runtime::{
    Button, Buttons, Ctx, Density, Driver, Mods, Mouse, MouseKind, Notch, Rect, Role,
};

use crate::collect::{
    Cell, CollOpts, CollState, Column, Node as TreeNode, TableOpts, TableState, TreeOpts,
    TreeState, collection_into, defective as coll_defective, table_into, tree_into,
};
use crate::files::{PaneOpts, PaneState, Preview, asking, file_preview_pane_with};
use crate::ink::{Direct, Ink};
use crate::keys;
use crate::order::{Order, Rows};
use crate::scroll::{AreaOpts, AreaState, parts, scroll_area};
use vitui_runtime::layout::Constraint;
use vitui_runtime::work::{Cancel, Task, Worker};

// ── the screen ───────────────────────────────────────────────────────────────────────────────────

/// The viewport's width. [`crate::listing::W`]'s forty, because the collection arm is that screen.
pub const W: u16 = crate::listing::W;

/// The viewport's height. [`crate::listing::H`]'s eighty.
pub const H: u16 = crate::listing::H;

/// How many rows the collection holds. A million, which is the wheel scene's volume.
pub const ROWS: u64 = 1_000_000;

/// **The area's content, in cells and on both axes.**
///
/// Wide enough that twenty horizontal clicks are twenty columns of a much larger content — the
/// question is whether the offset moves, and a content the clicks could exhaust would answer a
/// clamp rather than a wheel.
pub const EXTENT: (u32, u32) = (1_000, 1_000_000);

/// **How many wheel clicks the gate plays. Twenty**, which is the gesture.
pub const CLICKS: u32 = 20;

/// **How far twenty clicks move the offset when the reveal is conditional. Twenty.**
///
/// One row a click is `vitui_runtime::scroll::Wheel`'s default and the runtime's whole motion model:
/// a notch is intent and the runtime has no standing to multiply somebody's intent by
/// three. One *column* a click is the same default's other field, which is why the two axes share
/// this number.
pub const MOVED: i32 = CLICKS as i32;

/// **How far twenty clicks move the offset when the reveal fires every frame. Zero.**
///
/// The pinned failing set the gate was red on for nine tickets. See [`wheeled`] for why the number
/// one frame earlier is `1` and not `0`, and why that one is the documented residue rather
/// than a softened defect.
pub const DRAGGED_BACK: i32 = 0;

/// **How far the user has scrolled away from what a reveal will ask for**, for the second direction.
///
/// Two hundred: the selection sits at the top of the content and the viewport is nowhere near it,
/// which is the state the defect's own sentence describes — *it drags the viewport back to the
/// selection every time the user scrolls away from it*.
pub const SCROLLED_AWAY: i32 = 200;

/// **The document [`Subject::Pane`]'s pane is showing**, in content cells.
///
/// [`EXTENT`] and not a second pair, so the two subjects that own two offsets share **one** clamp
/// and the arm-for-arm equality below is about the components rather than about two fixtures.
pub const PANE_EXTENT: (u32, u32) = EXTENT;

/// **The file the pane's question names.** One identity, asked on every frame, so `Task::request`
/// deduplicates and the pane asks exactly once — which is the pane's own contract and not a
/// convenience here.
const PANE_FILE: u64 = 7;

/// **What a decode produces**, and the extent is [`PANE_EXTENT`].
///
/// A free function over an identity and never a closure, which is why the payload is
/// eight bytes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Doc(u64);

impl Preview for Doc {
    fn shows(&self) -> u64 {
        self.0
    }
    fn extent(&self) -> (u32, u32) {
        PANE_EXTENT
    }
}

/// The decode, as the preview pane requires it: a free function over an identity.
fn decode(id: u64, _cancel: &Cancel) -> Doc {
    Doc(id)
}

// ── the vocabulary ───────────────────────────────────────────────────────────────────────────────

/// **When the reveal fires**, and the whole of what the wheel gate is about.
///
/// Three arms and not two, because a **one-directional** gate goes green the moment somebody deletes
/// the call entirely — which loses the keyboard behaviour instead of fixing the pointer one. That is
/// the third criterion, and [`Reveal::Never`] is the arm it is written against.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reveal {
    /// **The rule.** The reveal is requested only when something asked for it — a keyboard cursor
    /// move, a programmatic reveal.
    WhenAsked,
    /// **The defect.** `scroll_into_view` on every frame, which is what four *resolved* tickets
    /// wrote after reading the rule in `CONTEXT.md` forbidding it.
    EveryFrame,
    /// **The way to pass a gate written in one direction**, and it is not a fix: the keyboard cursor
    /// can no longer bring anything into view.
    Never,
}

impl Reveal {
    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Reveal::WhenAsked => "only when asked",
            Reveal::EveryFrame => "every frame",
            Reveal::Never => "never",
        }
    }
}

/// **Which shipped component the gate is playing over.** criterion 4.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Subject {
    /// A virtualised [`crate::collect::collection`]: one offset, in rows, and the reveal is the
    /// component's own — it asks when a keyboard gesture moved its cursor.
    Collection,
    /// A [`crate::scroll::scroll_area`]: two offsets, in content cells, and the reveal is the
    /// **body's**. The component applies a delta and never asks for one.
    Area,
    /// A [`crate::collect::table`]: **one offset, in rows, and it is `collection`'s** — the row
    /// axis, the wheel and the reveal are all reached by calling it, which is the sentence the table
    /// opens with.
    ///
    /// **That is why it is a third subject rather than a fourth assertion about the first.**
    /// *`table` is `collection`* is the claim; a gate that plays the wheel over `collection` and
    /// takes the claim on trust is a gate that would stay green if the column split ever grew a
    /// second offset, a second store or a reveal of its own. It was asked directly, and
    /// what it costs is one `match` arm — which is the measure of how much of the sentence is
    /// true.
    Table,
    /// A [`crate::files::file_preview_pane`]: **two offsets, in content cells, and they are
    /// `scroll_area`'s** — the pane calls it, hands it the document's extent and keeps its
    /// [`AreaState`] as a field.
    ///
    /// **It is [`Subject::Table`]'s shape on the other family.** The preview pane
    /// states the pane as *a scroll area over a document that arrives from another thread* and the
    /// component is `scroll::scroll_area_into` with a virtualising body in front of it — so the
    /// wheel, the reveal and both clamps are reached by calling it, and nothing had ever asked. The
    /// assertions are therefore *`file_preview_pane` equals `scroll_area`, arm for arm* rather than
    /// six constants written twice, which is the arrangement and its reason.
    ///
    /// # The extent is a field of the answer, so the run has to land one first
    ///
    /// A pane with nothing showing declares `(0, 0)` and admits **no offset at all** — the clamp is
    /// `extent − viewport` — so a wheel run over an empty pane reports `0` and *a dead wheel and a
    /// meaningless axis are the same number*, which is [`Subject::axes`]' own refusal one step
    /// earlier. The drive loop answers the pane's question and lands it before the first frame; see
    /// [`PANE_EXTENT`], which is [`EXTENT`] so that the two subjects share one clamp.
    Pane,
    /// A [`crate::collect::tree`]: **one offset, in rows, and it is `collection`'s** — the row
    /// axis, the wheel and the reveal are all reached by calling it, which is the sentence a tree
    /// opens with and which nothing had ever asked a wheel question of.
    ///
    /// **It is [`Subject::Table`]'s shape on the third member of the same family.** A tree states
    /// it in more words than a table does — *there is no second selection store, no
    /// second scan cursor, no second [`Mode`], **no second offset** and no second press edge* — so
    /// the assertions here are *`tree` equals `collection`, arm for arm* rather than three
    /// constants written twice. What it costs is one `match` arm and two
    /// [`crate::collect::defective`] entries, which is the measure of how much of the sentence is
    /// true.
    ///
    /// **The index is nested and that is deliberate rather than decorative.** `crate::forest`'s own
    /// two scenes are built by [`crate::forest::index_for`], which puts *every* row at one depth —
    /// so `has_children` is false on all of them, every row is a leaf, and the chevron `tree` draws
    /// on scenes 8 and 9 is a **space** on every one of eighty rows. A wheel is an offset and would
    /// not notice; a degenerate index is still a worse fixture than a nested one for the same
    /// money, and [`crate::forest::nested`] is the builder both this arm and the narrow scene take.
    ///
    /// [`Mode`]: crate::collect::Mode
    Tree,
}

impl Subject {
    /// **Both of them, as a value a test iterates.**
    ///
    /// Criterion 6's population is *every subject this gate runs against*, and written out by hand on
    /// each side of the join a third subject escapes both halves while both stay green. This crate
    /// makes such populations values — `INVENTORY`, `SCENES`, `REGISTER` — and this is the same form
    /// at two rows.
    pub const ALL: [Subject; 5] = [
        Subject::Collection,
        Subject::Area,
        Subject::Table,
        Subject::Pane,
        Subject::Tree,
    ];

    /// The `INVENTORY` id, which is what makes criterion 6 a query rather than a claim.
    pub const fn id(self) -> &'static str {
        match self {
            Subject::Collection => "collection",
            Subject::Area => "scroll_area",
            Subject::Table => "table",
            Subject::Pane => "file_preview_pane",
            Subject::Tree => "tree",
        }
    }

    /// Which axes it owns an offset on. A collection owns rows and nothing else.
    ///
    /// **Read by [`wheeled`] and not only asserted about.** A `Play` naming an axis its subject does
    /// not own reports `0`, and *a dead wheel and a meaningless axis are the same number* — which is
    /// the shape of defect this whole file exists to make unwritable. So the mismatch panics.
    pub const fn axes(self) -> &'static [Along] {
        match self {
            Subject::Collection => &[Along::Rows],
            Subject::Area => &[Along::Rows, Along::Columns],
            // **A table owns a horizontal offset and a wheel notch does not reach it.** `TableState`
            // carries `hoff`, and `collection_shaped` is where the notch is consumed — one axis, in
            // rows. The column offset is the caller's to move, which is `crate::grid`'s scene 7 and
            // is not a wheel question; naming `Along::Columns` here would make `wheeled` report a
            // motionless offset as a dead wheel.
            Subject::Table => &[Along::Rows],
            // **A pane owns both**, because `scroll_area` does and the pane keeps its state: the
            // document is [`PANE_EXTENT`] cells on each axis and `Scrollable::between` publishes
            // the pair. That the answer here is the area's is the arm's whole claim.
            Subject::Pane => &[Along::Rows, Along::Columns],
            // **A tree owns one offset and it is in rows**, for [`Subject::Table`]'s reason and
            // not a second one: `TreeState::coll` *is* a `CollState`, so `collection_shaped` is
            // where the notch is consumed. The sentence is *no second offset*, and naming a
            // second axis here would make `wheeled` report a motionless offset as a dead wheel.
            Subject::Tree => &[Along::Rows],
        }
    }

    /// Whether it owns an offset on `along`.
    pub fn owns(self, along: Along) -> bool {
        self.axes().contains(&along)
    }

    /// **The largest offset this subject's content admits**, per axis, at this screen.
    ///
    /// The clamp the component itself will apply, available before the first frame — so that an
    /// offset handed in from outside is the one the component would have reached rather than an
    /// invalid state whose behaviour is nobody's stated property. `crate::listing`'s version of this
    /// gate clamped its starting offset for the same reason and the move mislaid it.
    pub fn max_offset(self) -> (i32, i32) {
        match self {
            // **The table's clamp is the collection's**, and it is reached through the same
            // function rather than restated: `table` has no second row store, so a second
            // expression here would be a second answer to one question.
            // **A tree's clamp is the collection's**, joined into the same arm for the table's
            // reason: a tree keeps no row store of its own, so a third expression here would be a
            // third answer to one question.
            Subject::Collection | Subject::Table | Subject::Tree => (
                0,
                CollState::max_offset(usize::try_from(ROWS).unwrap_or(usize::MAX), H),
            ),
            // **The pane's clamp is the area's**, reached through the same function for the
            // table's reason: the pane has no offset store of its own, so a second expression here
            // would be a second answer to one question.
            Subject::Area | Subject::Pane => area_max(),
        }
    }
}

/// **One axis**, and it is both *which way the clicks travel* and *which way an unconditional reveal
/// drags the viewport back*.
///
/// One enum for the two because the finding is the pair being **different**: a body dead downward
/// and alive sideways is one build, and naming the two roles with two types would let a gate compare
/// an axis against itself without the type system noticing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Along {
    /// Down the content. [`Notch::Down`].
    Rows,
    /// Across it. [`Notch::Right`], which a tilt wheel or a trackpad produces.
    Columns,
}

impl Along {
    /// The notch a click on this axis carries.
    const fn notch(self) -> Notch {
        match self {
            Along::Rows => Notch::Down,
            Along::Columns => Notch::Right,
        }
    }

    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Along::Rows => "rows",
            Along::Columns => "columns",
        }
    }

    /// This axis of an offset pair.
    pub const fn of(self, offset: (i32, i32)) -> i32 {
        match self {
            Along::Rows => offset.1,
            Along::Columns => offset.0,
        }
    }
}

/// **One run of the gate**, as a value rather than five positional arguments.
///
/// A struct because [`Play::pull`] and [`Play::along`] are the same type and the whole finding is
/// that they can differ — two `Along`s side by side in a call are exactly the pair a reader
/// transposes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Play {
    /// Which shipped component.
    pub subject: Subject,
    /// Which build of the reveal.
    pub reveal: Reveal,
    /// **Which axis an unconditional reveal drags back.** Read only for [`Subject::Area`], whose
    /// reveal is its body's; a collection's reveal is its own and is a row.
    pub pull: Along,
    /// Which axis the clicks travel on.
    pub along: Along,
    /// How many clicks.
    pub clicks: u32,
}

impl Play {
    /// [`CLICKS`] clicks down the rows, with an unconditional reveal pulling on the same axis.
    pub const fn of(subject: Subject, reveal: Reveal) -> Play {
        Play {
            subject,
            reveal,
            pull: Along::Rows,
            along: Along::Rows,
            clicks: CLICKS,
        }
    }

    /// The same run with the clicks travelling on `along`.
    pub const fn along(self, along: Along) -> Play {
        Play { along, ..self }
    }

    /// The same run with an unconditional reveal pulling on `pull`.
    pub const fn pulling(self, pull: Along) -> Play {
        Play { pull, ..self }
    }
}

/// **What the clicks did**, in three numbers rather than one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Wheeled {
    /// The offset after the last click's own frame.
    pub after_last_click: (i32, i32),
    /// **The offset one frame later**, when the reveal that frame asked for has been applied.
    ///
    /// This is the number the gate is written on. See [`wheeled`].
    pub settled: (i32, i32),
    /// How many of the frames asked for a reveal at all.
    pub reveals: u32,
}

impl Wheeled {
    /// The settled offset on one axis.
    pub const fn on(&self, along: Along) -> i32 {
        along.of(self.settled)
    }
}

// ── the instrument ───────────────────────────────────────────────────────────────────────────────

/// **Play [`Play::clicks`] posted wheel clicks over a shipped component and report where the offset
/// ended up.**
///
/// Criterion 2 and criterion 4's instrument, and `crate::gates::REGISTER`'s row 29.
///
/// # The click is a real one, and that is what changed
///
/// Each click is a [`Mouse`] with a [`MouseKind::Wheel`] posted through `Driver::post_mouse`,
/// resolved against the **previous** frame's hit index and read by the component inside its own
/// draw. One `Move` is posted first, because `Frame::chain_target` finds the innermost hit that is
/// both `over` and admits the direction, and nothing is `over` until the pointer has a position.
///
/// # Why `settled` and not `after_last_click`
///
/// A reveal crosses the frame boundary as sixteen bytes and is read on the frame after,
/// so the defective arm is always one click ahead of its own correction: [`Wheeled::after_last_click`]
/// is **1** and not 0. That one click is the runtime's own documented price —
///
/// > The residue is one click, at each end stop and on an area's first frame. It is a documented
/// > property, not a bug to be fixed later.
///
/// — arriving from the other side, and it is reported rather than hidden. The gate is written on
/// [`Wheeled::settled`], which plays one more frame with no click on it, because *twenty wheel
/// clicks move the offset twenty* is a statement about where the screen came to rest.
pub fn wheeled(play: Play) -> Wheeled {
    assert!(
        play.subject.owns(play.along),
        "`{}` owns no offset along the {} — a click on an axis a subject does not have reports 0, \
         and a dead wheel and a meaningless axis are the same number",
        play.subject.id(),
        play.along.word()
    );
    let mut run = Run::new(play.subject, (0, 0));
    // The pointer needs a position before anything is `over`, and the index it is resolved against
    // is the previous frame's — so one frame with no click on it opens the run.
    run.point();
    run.play(play, |_| {});

    let mut after_last_click = (0, 0);
    // **The opening frame counts too.** `Wheeled::reveals` is *how many of the frames asked at all*,
    // and leaving this one out is harmless only while the run starts at `(0, 0)` — the one offset
    // `leaves_no_request` documents every arm as silent on.
    let mut reveals = u32::from(run.asked);
    for _ in 0..play.clicks {
        run.click(play.along);
        run.play(play, |_| {});
        reveals += u32::from(run.asked);
        after_last_click = run.offset;
    }
    // One more frame with no click on it: the last one's reveal has to have somewhere to land, or
    // the arm is measured a frame before its own defect happens.
    run.play(play, |_| {});
    reveals += u32::from(run.asked);
    Wheeled {
        after_last_click,
        settled: run.offset,
        reveals,
    }
}

/// **How far a reveal moves the offset when something really asks for one. The gate's second
/// direction.**
///
/// The subject starts at `from` — the user has scrolled that far away from what the reveal will ask
/// for — and then something legitimate asks: for a [`Subject::Collection`] a posted `Ctrl+Home`,
/// which is a keyboard cursor move and the one gesture `CONTEXT.md` permits; for a
/// [`Subject::Area`] the body's own single request, which is where an area's reveal lives.
///
/// A build whose reveal was **deleted** rather than made conditional answers `(0, 0)` here, and that
/// is why the gate has two halves. *An offset that moves when nothing asked is a failure* on its own
/// is satisfied by a build that can no longer follow the keyboard at all, and
/// It is said in as many words: *a one-directional spelling
/// would go green the moment somebody deleted the call entirely, which loses the keyboard behaviour
/// instead of fixing the pointer one.*
pub fn revealed(subject: Subject, reveal: Reveal, from: (i32, i32)) -> (i32, i32) {
    let play = Play::of(subject, reveal).pulling(Along::Rows);
    let mut run = Run::new(subject, from);
    let start = run.offset;

    // **Frame zero seats the focus, and it is a frame rather than a line.** `Ctx::next_key` answers
    // `frame.route_to`, which is resolved from the *previous* frame's focus — so a widget that takes
    // the focus inside its own draw is deaf for that frame, and a gate that posted the key beside
    // the seating would measure a keyboard nobody is listening to. Runtime architecture issue 25
    // settled the seating; this is its cadence. The area subject has no focus to seat and plays it
    // anyway, so that *nothing has asked yet* is a frame on both arms rather than an absence on one.
    run.play(play, |_| {});

    // **Frame one: something asks, on every arm including [`Reveal::Never`].**
    //
    // The gesture is posted unconditionally and that is the whole of this function's second half
    // working at all. `Reveal::Never` answering `(0, 0)` is the only evidence that deleting the call
    // loses the keyboard — and guarded by `if reveal != Reveal::Never` the zero is produced by the
    // **absent gesture** rather than by the absent reveal, which makes the one assertion the
    // three-arm design exists for vacuous. Measured: with the guard in place, repairing
    // `collect::defective::never_reveals` to behave exactly like the shipped component left all five
    // tests in this module green.
    //
    // A collection asks because its cursor moved — a posted `Ctrl+Home`, routed to the focus frame
    // zero seated. An area asks because the application told its body to. The key is posted *for*
    // the frame that reads it rather than inside one: `post_key` is a `Driver` verb, which is the
    // runtime's own statement that an event is a thing a frame *finds*.
    match subject {
        // **A table's cursor is a collection's**, so the gesture is the same one and reaches it
        // through the same drain loop. That is the third instance of the sentence being checked
        // rather than trusted.
        Subject::Collection | Subject::Table | Subject::Tree => run.press(REVEAL_CHORD),
        // **A pane's reveal is its body's**, exactly as an area's is, because the pane hands its
        // rectangle to `scroll_area` and that component applies a delta and never asks for one.
        // So the gesture is the same one and the arm is `Subject::Area`'s — which is what production
        // 09 is checking rather than trusting.
        Subject::Area | Subject::Pane => run.ask(),
    }
    run.play(play, |_| {});
    // Frame two: the widget that owns the offset applies the delta it was handed. **The application
    // owns the offset** — `take_into_view` answers a delta for exactly that reason.
    run.play(play, |_| {});
    (run.offset.0 - start.0, run.offset.1 - start.1)
}

/// **Whether one frame at `offset`, with nobody's cursor having moved, leaves a request behind.**
///
/// The gate's other direction, as a count of the one structure that crosses the frame boundary
/// (sixteen bytes, an area and a delta, no `Rect`).
///
/// # The offset is a parameter and it is the whole test
///
/// At offset `(0, 0)` every arm leaves nothing, because the selection is on screen and
/// `Area::into_view` answers `(0, 0)` for a rectangle that is already visible. **That is the frame
/// the defect is invisible on**, and a gate written only there would be green on the code four
/// resolved tickets shipped. At any offset the user has actually scrolled to, the unconditional arm
/// asks — every frame, for ever — and the conditional one does not.
pub fn leaves_no_request(play: Play, offset: (i32, i32)) -> bool {
    let mut run = Run::new(play.subject, offset);
    run.play(play, |_| {});
    !run.asked
}

/// **What a press at `offset` did: whether it selected a row, and whether it pulled the viewport.**
///
/// The rule's second clause, and the one a component is most likely to get wrong while looking
/// right:
///
/// > A press already proves the widget was on screen, and an unconditional pull is the list's old
/// > bug.
///
/// Selecting a row and revealing it are one gesture in every list written by hand, and the row is
/// already on screen — the user just pointed at it.
///
/// # It takes four frames, and every one of them is the runtime's cadence rather than this gate's
///
/// The grab is awarded at `end` from the index that has just drawn, so `Response::pressed` is false
/// on the frame that *processes* the `Down` and true on the one after — and a `collection` selects on
/// the press **edge**, which is therefore the second. The release is a frame of its own for the same
/// reason: a `Down` and an `Up` in one batch open and close the grab before anything draws, so the
/// component would see no press at all and the whole measurement would be of a frame where nothing
/// happened.
///
/// # `selected` is here so that *no pull* is distinguishable from *no gesture*
///
/// A frame where nothing was pressed also pulls nothing. [`Tapped::selected`] is what separates the
/// two, and it is why this returns a value rather than a `bool`.
///
/// # Only [`Subject::Collection`] answers anything about a selection
///
/// A `scroll_area` has no cursor and its reveal is its body's, so a press over one is a press over
/// whatever the body drew. The question belongs to the component that owns a cursor.
pub fn tapped(play: Play, offset: (i32, i32)) -> Tapped {
    let mut run = Run::new(play.subject, offset);
    run.point();
    run.play(play, |_| {});
    let before = run.asked;

    run.press_pointer();
    run.play(play, |_| {});
    let on_the_press = run.asked;
    run.play(play, |_| {});
    let on_the_edge = run.asked;
    let selected = run.selected;

    run.release_pointer();
    run.play(play, |_| {});
    Tapped {
        before,
        asked: on_the_press || on_the_edge || run.asked,
        selected,
        settled: run.offset,
    }
}

/// What [`tapped`] measured.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Tapped {
    /// Whether the frame **before** the press already left a request. The unconditional arm does,
    /// which is why it can never be watched failing the press clause: it has converged by then.
    pub before: bool,
    /// Whether any of the press's three frames left a request behind.
    pub asked: bool,
    /// How many rows are selected afterwards. `0` means the gesture did not land.
    pub selected: usize,
    /// Where the offset ended up.
    pub settled: (i32, i32),
}

// ── one drive loop, and both subjects go through it ──────────────────────────────────────────────

/// The driver, the state and the offset, so that both subjects are played by one loop.
///
/// **A second loop is what the two substitutions were**: the arithmetic click and its
/// hand-written row body were each a copy of something, and both copies were where the defect is
/// not.
struct Run {
    driver: Driver,
    subject: Subject,
    coll: CollState,
    table: TableState,
    area: AreaState,
    /// **The pane's state and the task its answer came through.** Both inert on every other arm.
    ///
    /// The `Task` is held across the frames rather than minted inside one, which is
    /// `Task::request`'s own contract — *call it unconditionally, every frame*, and the verb is
    /// idempotent because the task remembers the key. A task per frame would ask [`PANE_FRAMES`]
    /// questions for one document.
    pane: PaneState<Doc>,
    task: Task<Doc>,
    /// **The tree's state and the flatten index it reads.** Both inert on every other arm.
    ///
    /// The index is built in [`Run::new`] and not inside a frame, which is `crate::forest`'s own
    /// rule and the price for the other spelling: materialising a million rows is
    /// proportional to the data by construction, and doing it in a frame costs 211 frame budgets.
    tree: TreeState,
    index: Order,
    /// **What the pane's body saw**, as `(first visible column, first visible row)`.
    ///
    /// The offset **read out of the coordinate system the component put the body in**, and not off
    /// a getter: `PaneState::offset` answers the vertical axis alone, and the axis this gate is
    /// about is the pair. It is also the stronger reading — a pane that clamped its offset and
    /// scrolled its body to somewhere else would show up here and not in a field.
    seen: (i32, i32),
    offset: (i32, i32),
    /// Whether the frame just played left a reveal request behind.
    asked: bool,
    /// **Whether an area's body has already made its one legitimate request.**
    ///
    /// It lives here and not inside the body closure, and that is the defect this gate exists to
    /// catch arriving in the gate itself: a flag minted inside the frame is `false` again on the
    /// next one, so [`Reveal::WhenAsked`] asked **every frame** and measured
    /// [`Reveal::EveryFrame`] under the correct arm's name. *Only when asked* is a statement across
    /// frames, so the state has to be one too.
    once: bool,
    /// **Whether anything has asked an area's body to reveal yet.**
    ///
    /// A collection's [`Reveal::WhenAsked`] arm has a real gesture behind it — a posted key that moved
    /// the cursor. An area's has to have one too, or *only when asked* would mean *once, whether or
    /// not anybody asked*: the request would land on the run's first frame, which at offset `(0, 0)`
    /// is a no-op and at any other offset is a pull nothing asked for. So the two subjects share a
    /// cadence — [`Run::ask`] is the area's `post_key`.
    armed: bool,
    /// How many rows the collection has selected. Read by the press assertion, so that *no reveal*
    /// is distinguishable from *no gesture*.
    selected: usize,
}

impl Run {
    fn new(subject: Subject, offset: (i32, i32)) -> Run {
        // **Clamped, and zeroed on an axis the subject does not own.** `Run::play` recomputes the
        // offset from the component's own state every frame, so an unclamped or one-axis-too-wide
        // value handed in here would be the *only* reading that had never been through the
        // component — which is exactly the reading `revealed` subtracts from.
        let max = subject.max_offset();
        let offset = (offset.0.clamp(0, max.0), offset.1.clamp(0, max.1));
        let mut coll = CollState::default();
        coll.offset = offset.1;
        let mut table = TableState::new();
        table.coll.offset = offset.1;
        // **The pane's document is landed before the first frame**, because a pane with nothing
        // showing declares `(0, 0)` and admits no offset at all — see [`Subject::Pane`]. The
        // `Worker::queueing` answers on this thread when told to, which is `Worker::queueing`'s own
        // argument: a decode that finishes when the caller says so is a schedule and not a race.
        let worker = Worker::queueing();
        let task: Task<Doc> = Task::new(&worker);
        let mut pane: PaneState<Doc> = PaneState::new();
        if subject == Subject::Pane {
            let _asked = task.request(PANE_FILE, |cancel| decode(PANE_FILE, cancel));
            let _ran = worker.run(0);
            let _landed = pane.land(&task);
            // The offset was clamped to `[0, max]` above, so the conversion cannot fail — and a
            // fallback to zero would silently start the run at the one offset every arm of this
            // gate is documented as silent on.
            pane.scroll_to(u32::try_from(offset.1).expect("clamped to a non-negative offset"));
        }
        let mut tree = TreeState::new();
        tree.coll.offset = offset.1;
        // **Nested, and only on the arm that reads it.** A million-entry `Order` is 8 MB and every
        // other subject would pay for it unread; `Order::built(Vec::new())` is what the others get.
        let index = if subject == Subject::Tree {
            crate::forest::nested(usize::try_from(ROWS).unwrap_or(usize::MAX))
        } else {
            Order::built(Vec::new())
        };
        Run {
            driver: crate::runner::driver_at(W, H, Density::default()),
            subject,
            coll,
            table,
            area: AreaState { offset },
            pane,
            task,
            tree,
            index,
            seen: offset,
            offset,
            asked: false,
            once: false,
            armed: false,
            selected: 0,
        }
    }

    /// Give the pointer a position, so that a hit can be `over`.
    fn point(&mut self) {
        self.driver.post_mouse(at(MouseKind::Move));
    }

    /// Post one key press.
    fn press(&mut self, chord: Chord) {
        self.driver.post_key(keys::press(chord));
    }

    /// **Ask an area's body to reveal.** The area's half of [`Run::press`] — see [`Run::armed`].
    fn ask(&mut self) {
        self.armed = true;
    }

    /// Press over the pointer's position, without releasing. See [`tapped`] for the cadence.
    fn press_pointer(&mut self) {
        self.driver.post_mouse(at(MouseKind::Down(Button::Left)));
    }

    /// Release it, so no grab is left standing.
    fn release_pointer(&mut self) {
        self.driver.post_mouse(at(MouseKind::Up(Button::Left)));
    }

    /// Post one click on `along`.
    fn click(&mut self, along: Along) {
        self.driver.post_mouse(at(MouseKind::Wheel(along.notch())));
    }

    /// One frame, with `before` run inside it ahead of the component.
    fn play(&mut self, play: Play, before: impl FnOnce(&mut Ctx<'_, '_>)) {
        let subject = self.subject;
        let coll = &mut self.coll;
        let table = &mut self.table;
        let area = &mut self.area;
        let pane = &mut self.pane;
        let task = &self.task;
        let tree = &mut self.tree;
        let index = &self.index;
        let seen = &mut self.seen;
        let once = &mut self.once;
        let armed = self.armed;
        self.driver.frame(|cx| {
            before(cx);
            match subject {
                Subject::Collection => collection_frame(cx, coll, play),
                Subject::Area => area_frame(cx, area, play, once, armed),
                Subject::Table => table_frame(cx, table, play),
                Subject::Pane => pane_frame(cx, pane, task, play, once, armed, seen),
                Subject::Tree => tree_frame(cx, tree, index, play),
            }
        });
        self.offset = match subject {
            Subject::Collection => (0, self.coll.offset),
            Subject::Area => self.area.offset,
            Subject::Table => (0, self.table.coll.offset),
            Subject::Pane => self.seen,
            Subject::Tree => (0, self.tree.coll.offset),
        };
        self.asked = self.driver.inspect().into_view().is_some();
        self.selected = match subject {
            Subject::Table => self.table.coll.sel.count(),
            Subject::Tree => self.tree.coll.sel.count(),
            _ => self.coll.sel.count(),
        };
    }
}

/// A pointer event over the middle of the screen.
fn at(kind: MouseKind) -> Mouse {
    Mouse {
        x: W / 2,
        y: H / 2,
        kind,
        buttons: Buttons::NONE,
        mods: Mods::NONE,
        at: std::time::Instant::now(),
    }
}

/// One frame of the collection arm, **through the shipped component**.
fn collection_frame(cx: &mut Ctx<'_, '_>, st: &mut CollState, play: Play) {
    let area = cx.area();
    let rows = Rows::of(usize::try_from(ROWS).unwrap_or(usize::MAX));
    let body = cx.theme().paint(Role::Body);
    let mut find = |_: &str, _: std::ops::Range<usize>| None;
    let mut row =
        |ink: &mut Direct, cx: &mut Ctx<'_, '_>, r: Rect, _: usize, _: crate::frame::Face| {
            let _ = ink.run(cx, r.x, r.y, "·", r.w, body);
        };
    let opts = CollOpts::default();
    let resp = match play.reveal {
        Reveal::WhenAsked => {
            collection_into(&mut Direct, cx, area, st, &opts, rows, &mut find, &mut row)
        }
        Reveal::EveryFrame => {
            coll_defective::every_frame(&mut Direct, cx, area, st, &opts, rows, &mut find, &mut row)
        }
        Reveal::Never => coll_defective::never_reveals(
            &mut Direct,
            cx,
            area,
            st,
            &opts,
            rows,
            &mut find,
            &mut row,
        ),
    };
    // **Give it the keyboard while nobody has it, and do it from the id the component answered.**
    //
    // Two facts, both of them the runtime's and both of them load-bearing here. Nothing holds the
    // focus until an application says so, and `next_key` answers
    // nobody but the routing target — so a gate that posts a key and seats no focus measures a
    // keyboard nobody is listening to. And the id is **`Response::id` and not a name this module
    // chose**: `collection`'s id is minted from `Location::caller()`, so the only spelling of *that
    // collection* available from outside it is the one it hands back. This is `explorer`'s and
    // `reader`'s own arrangement, and it is the reason the reveal half of the gate takes three
    // frames rather than two — the seating lands a frame before the key it enables.
    if cx.focused().is_none() {
        cx.focus(resp.id);
    }
}

/// **The table arm's columns.** One pinned each side and one elastic between them, at [`W`].
///
/// Three, and the shape rather than the count is what matters: `solve_columns` runs above the row
/// loop and touches no row, so a wheel notch cannot reach it — the arm exists to show that the row
/// axis under a column split is still `collection`'s, not to price the split. `crate::grid` is
/// where the columns are the question, at twelve of them and three hundred cells wide.
///
/// The elastic middle is what makes the row a partition of [`W`] with no column *boundary* falling
/// short of it. A band whose columns are all `Fixed` and sum to less than the viewport used to leave
/// the remainder unwritten — a **second** shrink surface, and `crate::grid::COLUMN_RESIDUE_WAS`'s
/// measurement rather than this gate's. It is answered: the band writes its own
/// slack now, so an all-`Fixed` list is no longer a partition defect, and the elastic middle here is
/// about where the columns *are* rather than about which cells get written.
fn columns() -> [Column; 3] {
    [
        Column::new(0, "id", Constraint::Fixed(6)).pinned_left(6),
        Column::new(1, "name", Constraint::Weight(1)),
        Column::new(2, "cpu", Constraint::Fixed(5)).pinned_right(5),
    ]
}

/// One frame of the table arm, **through the shipped component**.
///
/// The same three arms as [`collection_frame`] and the same focus seating, one component up. The
/// cell drawer writes one run a cell, because what this gate reads is an offset and not a screen —
/// `crate::grid` is where a table's cells are compared against a reference render.
fn table_frame(cx: &mut Ctx<'_, '_>, st: &mut TableState, play: Play) {
    let area = cx.area();
    let rows = Rows::of(usize::try_from(ROWS).unwrap_or(usize::MAX));
    let body = cx.theme().paint(Role::Body);
    let cols = columns();
    let opts = TableOpts::default();
    let mut find = |_: &str, _: std::ops::Range<usize>| None;
    let mut cell =
        |ink: &mut Direct, cx: &mut Ctx<'_, '_>, r: Rect, _: Cell, _: crate::frame::Face| {
            let _ = ink.run(cx, r.x, r.y, "·", r.w, body);
        };
    let resp = match play.reveal {
        Reveal::WhenAsked => table_into(
            &mut Direct,
            cx,
            area,
            st,
            &opts,
            &cols,
            rows,
            &mut find,
            &mut cell,
        ),
        Reveal::EveryFrame => coll_defective::table_every_frame(
            &mut Direct,
            cx,
            area,
            st,
            &opts,
            &cols,
            rows,
            &mut find,
            &mut cell,
        ),
        Reveal::Never => coll_defective::table_never_reveals(
            &mut Direct,
            cx,
            area,
            st,
            &opts,
            &cols,
            rows,
            &mut find,
            &mut cell,
        ),
    };
    // [`collection_frame`]'s seating, for its reason: nothing holds the focus until an application
    // says so, and the id is the one the component answered.
    if cx.focused().is_none() {
        cx.focus(resp.id);
    }
}

/// One frame of the tree arm, **through the shipped component**.
///
/// [`table_frame`]'s shape one component over and the same three arms, with the columns replaced by
/// a flatten index — because that is exactly what the two components add to `collection` and
/// neither addition is on the row axis. The row drawer writes one run a row, for `table_frame`'s
/// reason: what this gate reads is an offset and not a screen, and `crate::forest` is where a
/// tree's cells are compared against a reference render.
fn tree_frame(cx: &mut Ctx<'_, '_>, st: &mut TreeState, index: &Order, play: Play) {
    let area = cx.area();
    let body = cx.theme().paint(Role::Body);
    let opts = TreeOpts::default();
    let mut find = |_: &str, _: std::ops::Range<usize>| None;
    let mut row =
        |ink: &mut Direct, cx: &mut Ctx<'_, '_>, r: Rect, _: TreeNode, _: crate::frame::Face| {
            if r.w > 0 {
                let _ = ink.run(cx, r.x, r.y, "\u{b7}", r.w, body);
            }
        };
    let resp = match play.reveal {
        Reveal::WhenAsked => {
            tree_into(&mut Direct, cx, area, st, &opts, index, &mut find, &mut row)
        }
        Reveal::EveryFrame => coll_defective::tree_every_frame(
            &mut Direct,
            cx,
            area,
            st,
            &opts,
            index,
            &mut find,
            &mut row,
        ),
        Reveal::Never => coll_defective::tree_never_reveals(
            &mut Direct,
            cx,
            area,
            st,
            &opts,
            index,
            &mut find,
            &mut row,
        ),
    };
    // [`collection_frame`]'s seating, for its reason.
    if cx.focused().is_none() {
        cx.focus(resp.id);
    }
}

/// One frame of the area arm, **through the shipped component**.
///
/// The reveal is the body's, which is the whole difference between the two subjects: `scroll_area`
/// applies a delta and never asks for one, so the defect this gate is about is a *caller's* here and
/// a component's one file over. The gate is the same either way, and criterion 4 asks for exactly
/// that.
fn area_frame(cx: &mut Ctx<'_, '_>, st: &mut AreaState, play: Play, once: &mut bool, armed: bool) {
    let rect = cx.area();
    let _ = scroll_area(cx, rect, st, EXTENT, &mut |cx| {
        let paint = cx.theme().paint(Role::Body);
        let rows = cx.visible_rows();
        let cols = cx.visible_cols();
        let _ = cx.text(cols.start, rows.start, "·", paint);
        let ask = match play.reveal {
            // An area's legitimate reveal is one request from its caller, and the caller is this
            // gate: once, on the frame after it asked — see `Run::armed` for why *asked* and not
            // *first*. It is `crate::scroll`'s own criterion 4 with a gesture in front of it.
            Reveal::WhenAsked => armed && !std::mem::replace(once, true),
            Reveal::EveryFrame => true,
            Reveal::Never => false,
        };
        if ask {
            cx.request_into_view(pull_rect(play.pull, &rows, &cols));
        }
    });
}

/// One frame of the pane arm, **through the shipped component**.
///
/// [`area_frame`]'s shape, one family over, and the reveal is the **body's** for the same reason:
/// `file_preview_pane` hands its rectangle to `scroll_area`, which applies a delta and never asks
/// for one. So this arm needs no `crate::files::defective` entry at all — the three arms are three
/// bodies, which is the measure of how much of *a scroll area over a document* is true.
///
/// The offset is read **inside the body**, out of the coordinate system the component put it in.
/// See this module's `Run` for why the reading is inside the body rather than off a getter.
fn pane_frame(
    cx: &mut Ctx<'_, '_>,
    st: &mut PaneState<Doc>,
    task: &Task<Doc>,
    play: Play,
    once: &mut bool,
    armed: bool,
    seen: &mut (i32, i32),
) {
    let rect = cx.area();
    let opts = PaneOpts::default();
    // **The line drawer is called once per visible document row**, and the reveal is asked on the
    // first of them: a body asking on every row would ask seventy-four times a frame, which is one
    // request the frame keeps and seventy-three the reader has to reason about.
    let mut first = true;
    let _ = file_preview_pane_with(
        cx,
        rect,
        st,
        task,
        asking(PANE_FILE, |cancel| decode(PANE_FILE, cancel)),
        &opts,
        &mut |cx, row, _doc: &Doc, _i| {
            let paint = cx.theme().paint(Role::Body);
            let _ = cx.text(row.x, row.y, "·", paint);
            if !std::mem::take(&mut first) {
                return;
            }
            let rows = cx.visible_rows();
            let cols = cx.visible_cols();
            *seen = (cols.start, rows.start);
            let ask = match play.reveal {
                // [`area_frame`]'s three arms and its `Run::armed` cadence, unchanged.
                Reveal::WhenAsked => armed && !std::mem::replace(once, true),
                Reveal::EveryFrame => true,
                Reveal::Never => false,
            };
            if ask {
                cx.request_into_view(pull_rect(play.pull, &rows, &cols));
            }
        },
    );
}

/// **How many frames a pane run plays.** [`CLICKS`] plus the opening frame and the settling one,
/// which is [`wheeled`]'s cadence and not a second one.
pub const PANE_FRAMES: u32 = CLICKS + 2;

/// **The rectangle an unconditional reveal asks for, on one axis only.**
///
/// This is criterion 4's *a body dead downward and alive sideways must be distinguishable*, as
/// arithmetic. `Area::into_view` answers per axis and returns `0` for an axis the rectangle already
/// sits inside — so a request at content row 0 that is **horizontally inside the window** pulls the
/// rows and leaves the columns alone, and the transpose does the opposite.
///
/// A rectangle spanning the whole content on the axis that must not move would not do: `into_view`
/// aligns an entry longer than the window to its *start*, so a full-width strip pulls the columns
/// back to zero as hard as a one-cell one.
fn pull_rect(pull: Along, rows: &std::ops::Range<i32>, cols: &std::ops::Range<i32>) -> Rect {
    match pull {
        Along::Rows => Rect::new(cols.start, 0, 1, 1),
        Along::Columns => Rect::new(0, rows.start, 1, 1),
    }
}

/// The largest offset the area's content admits, at this screen. What the clicks are clamped by.
pub fn area_max() -> (i32, i32) {
    parts(Rect::new(0, 0, W, H), EXTENT, &AreaOpts::default()).max_offset(EXTENT)
}

/// A `Chord` this gate posts, for the report to print.
///
/// `Ctrl+Home` and not `Home`: `crate::nav::step` reads a bare `Home` too, and the chord is the one
/// is measured beside `Ctrl+A` — *move the cursor, leave the selection alone*, which is the
/// gesture a reveal is legitimately downstream of.
pub const REVEAL_CHORD: Chord = Chord::new(Code::Home).ctrl();

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::{Axis, INVENTORY};

    /// **Criterion 2 and 3: twenty posted wheel clicks move the offset twenty, and the gate fires
    /// both ways, over the shipped `collection`.**
    #[test]
    fn twenty_posted_clicks_move_a_collections_offset_twenty_and_an_unconditional_reveal_takes_it_back()
     {
        let free = wheeled(Play::of(Subject::Collection, Reveal::WhenAsked));
        assert_eq!(free.settled, (0, MOVED), "twenty clicks, twenty rows");
        assert_eq!(free.after_last_click, (0, MOVED));
        assert_eq!(free.reveals, 0, "nothing asked, so nothing was requested");

        let dragged = wheeled(Play::of(Subject::Collection, Reveal::EveryFrame));
        assert_eq!(
            dragged.settled,
            (0, DRAGGED_BACK),
            "the pinned failing set: an unconditional `scroll_into_view` and the pointer is dead"
        );
        assert_eq!(
            dragged.reveals, CLICKS,
            "twenty frames asked and the twenty-first did not need to: by then the viewport was \
             already back where the cursor is, which is what the defect converges to"
        );
        assert_eq!(
            dragged.after_last_click,
            (0, 1),
            "one click ahead of its own correction, which is ADR 0015's documented residue seen \
             from the other side rather than a softened defect"
        );

        // **The second direction.** An offset that moves when nothing asked is a failure, and a
        // build that can no longer follow the keyboard is not a fix.
        let at_rest = Play::of(Subject::Collection, Reveal::WhenAsked);
        assert!(leaves_no_request(at_rest, (0, MOVED)));
        assert!(
            !leaves_no_request(
                Play::of(Subject::Collection, Reveal::EveryFrame),
                (0, MOVED)
            ),
            "twenty rows down, the unconditional arm asks to be dragged back on this very frame"
        );
        assert!(
            leaves_no_request(Play::of(Subject::Collection, Reveal::EveryFrame), (0, 0)),
            "and at offset 0 it asks for nothing, which is the frame this defect is invisible on \
             and the reason the offset is a parameter"
        );

        assert_eq!(
            revealed(Subject::Collection, Reveal::WhenAsked, (0, SCROLLED_AWAY)),
            (0, -SCROLLED_AWAY),
            "the keyboard behaviour survives: a cursor moved to the top of the content brings the \
             viewport with it"
        );
        assert_eq!(
            revealed(Subject::Collection, Reveal::Never, (0, SCROLLED_AWAY)),
            (0, 0),
            "and deleting the call loses it, which is why a one-directional gate is not a gate"
        );
        assert_eq!(
            revealed(Subject::Collection, Reveal::EveryFrame, (0, SCROLLED_AWAY)),
            (0, -SCROLLED_AWAY),
            "the defective arm reveals too, which is the point: what separates it from the rule is \
             *when*, and no single frame can tell them apart"
        );

        // Deleting the call also passes the wheel half, which is the whole argument for pinning
        // both directions rather than the loud one.
        assert_eq!(
            wheeled(Play::of(Subject::Collection, Reveal::Never)).settled,
            (0, MOVED)
        );
    }

    /// **Criterion 4: the same gate over `scroll_area`, and the blindness is per axis.**
    ///
    /// The pair is the finding: a body that asks for content row 0 on every frame is dead to a
    /// vertical click and **entirely healthy sideways**, and the transpose is the mirror image. One
    /// number for *the offset* cannot say either, which is why this gate takes an axis twice.
    #[test]
    fn a_body_dead_downward_is_alive_sideways_and_one_offset_cannot_say_so() {
        let down = Play::of(Subject::Area, Reveal::WhenAsked);
        let across = down.along(Along::Columns);

        assert_eq!(
            wheeled(down).on(Along::Rows),
            MOVED,
            "twenty clicks down, twenty content rows"
        );
        assert_eq!(
            wheeled(across).on(Along::Columns),
            MOVED,
            "and twenty across, twenty content columns"
        );

        // The defect, pulling on the rows: dead downward, alive sideways.
        let pull_rows = Play::of(Subject::Area, Reveal::EveryFrame).pulling(Along::Rows);
        assert_eq!(
            wheeled(pull_rows).on(Along::Rows),
            DRAGGED_BACK,
            "a body that asks for content row 0 every frame is dead to a vertical click"
        );
        assert_eq!(
            wheeled(pull_rows.along(Along::Columns)).on(Along::Columns),
            MOVED,
            "**and entirely healthy sideways** — the axis the request never named"
        );

        // The transpose, which is what makes the line above a measurement rather than a coincidence.
        let pull_cols = pull_rows.pulling(Along::Columns);
        assert_eq!(
            wheeled(pull_cols.along(Along::Columns)).on(Along::Columns),
            DRAGGED_BACK
        );
        assert_eq!(wheeled(pull_cols).on(Along::Rows), MOVED);

        // And deleting the call passes both halves while losing the reveal, on both axes.
        for along in [Along::Rows, Along::Columns] {
            let gone = Play::of(Subject::Area, Reveal::Never).along(along);
            assert_eq!(wheeled(gone).on(along), MOVED, "{}", along.word());
        }
        assert_eq!(
            revealed(Subject::Area, Reveal::Never, (0, SCROLLED_AWAY)),
            (0, 0),
            "a body with no request left is a body the keyboard cannot reach"
        );
        assert_eq!(
            revealed(Subject::Area, Reveal::WhenAsked, (0, SCROLLED_AWAY)),
            (0, -SCROLLED_AWAY),
            "and one with a request is brought back, once"
        );

        // **The area's half of the second direction**, and it is only assertable because
        // `Reveal::WhenAsked` waits to be asked rather than firing on the run's first frame. A body
        // nobody has asked leaves nothing behind at an offset the user scrolled to; the
        // unconditional one asks there on every frame, for ever.
        assert!(leaves_no_request(down, (0, SCROLLED_AWAY)));
        assert!(!leaves_no_request(pull_rows, (0, SCROLLED_AWAY)));
        assert!(
            leaves_no_request(pull_rows, (0, 0)),
            "and at offset 0 it asks for nothing, which is the frame the defect is invisible on"
        );
    }

    /// **A press does not pull, and that is the rule's own second clause.**
    ///
    /// > A press already proves the widget was on screen, and an unconditional pull is the list's
    /// > old bug.
    ///
    /// The whole of `CONTEXT.md`'s rule is *only for a keyboard-driven focus move*, and the clause
    /// that names the pointer is the one a component is most likely to get wrong while looking
    /// right: selecting a row and revealing it are one gesture in every list anybody has written by
    /// hand, and the row is already on screen — the user just pointed at it.
    ///
    /// It is asserted over the shipped component at a **non-zero offset**, which is the only place
    /// it can be seen. At offset 0 the pressed row and every other visible row are already inside
    /// the viewport, so `Area::into_view` answers `(0, 0)` and a pull and no pull are the same
    /// frame.
    #[test]
    fn a_press_selects_a_row_and_does_not_pull_the_viewport_to_it() {
        let play = Play::of(Subject::Collection, Reveal::WhenAsked);
        let scrolled = (0, SCROLLED_AWAY);
        let tap = tapped(play, scrolled);

        assert!(!tap.before, "nothing had happened yet");
        assert!(
            !tap.asked,
            "a press already proves the row was on screen, so a reveal on one fights the wheel for \
             the rest of the session"
        );
        assert_eq!(tap.settled, scrolled, "and the viewport did not move");
        assert_eq!(
            tap.selected, 1,
            "the press did select, so this is not a measurement of a frame where nothing happened"
        );

        // **And the arm that pulls unconditionally is watched pulling at the same offset**, so the
        // assertion above is a comparison and not an inspection.
        //
        // It cannot be watched pulling *on the press*, and the reason is the defect rather than an
        // escape from it: by the frame the press edge lands on, the unconditional arm has already
        // dragged the viewport back to the cursor, so `Area::into_view` answers `(0, 0)` and the
        // loudest arm is silent for the quietest reason. What it is watched doing instead is asking
        // on the frame **before** any gesture — which the conditional arm at the same offset does
        // not.
        let broken = tapped(Play::of(Subject::Collection, Reveal::EveryFrame), scrolled);
        assert!(
            broken.before,
            "the unconditional arm asks before any gesture has happened"
        );
        assert_eq!(
            broken.settled,
            (0, 0),
            "and it has dragged the viewport all the way back, which is why its press frames are \
             quiet"
        );
        assert!(leaves_no_request(play, scrolled));
    }

    /// **The same gate over the shipped `table`, and the sentence is checked
    /// rather than trusted.**
    ///
    /// > There is no second selection store, no second scan cursor and no second `Mode`. The row
    /// > axis, the wheel, the keyboard, the type-ahead, the reveal, the tail below the content and
    /// > the revision check are all `collection`'s, reached by calling it.
    ///
    /// That is `crate::collect::table`'s own claim, and until this ticket nothing had asked it a
    /// wheel question. The numbers are `collection`'s to the click, in both directions — which is
    /// the finding: **a column split above a row axis costs the row axis nothing**, and a build
    /// where it did would show up here as one of these three arms disagreeing with its twin one
    /// component down.
    #[test]
    fn twenty_posted_clicks_move_a_tables_offset_twenty_and_the_numbers_are_the_collections() {
        let free = wheeled(Play::of(Subject::Table, Reveal::WhenAsked));
        assert_eq!(free.settled, (0, MOVED), "twenty clicks, twenty rows");
        assert_eq!(free.after_last_click, (0, MOVED));
        assert_eq!(free.reveals, 0, "nothing asked, so nothing was requested");

        let dragged = wheeled(Play::of(Subject::Table, Reveal::EveryFrame));
        assert_eq!(
            dragged.settled,
            (0, DRAGGED_BACK),
            "an unconditional `scroll_into_view` and the pointer is dead over a table too"
        );
        assert_eq!(dragged.reveals, CLICKS);
        assert_eq!(
            dragged.after_last_click,
            (0, 1),
            "ADR 0015's residue, unchanged by the column split"
        );

        // **The second direction**, and it is what stops the arm below being a fix.
        assert_eq!(
            revealed(Subject::Table, Reveal::WhenAsked, (0, SCROLLED_AWAY)),
            (0, -SCROLLED_AWAY),
            "a cursor moved to the top of the content brings the viewport with it"
        );
        assert_eq!(
            revealed(Subject::Table, Reveal::Never, (0, SCROLLED_AWAY)),
            (0, 0),
            "and deleting the call loses it"
        );
        assert_eq!(
            wheeled(Play::of(Subject::Table, Reveal::Never)).settled,
            (0, MOVED),
            "while passing the wheel half, which is why a one-directional gate is not a gate"
        );

        assert!(leaves_no_request(
            Play::of(Subject::Table, Reveal::WhenAsked),
            (0, MOVED)
        ));
        assert!(
            !leaves_no_request(Play::of(Subject::Table, Reveal::EveryFrame), (0, MOVED)),
            "twenty rows down, the unconditional arm asks to be dragged back on this very frame"
        );

        // **The three arms agree with `collection`'s, arm for arm.** Written as a comparison rather
        // than as three repeated constants, because the claim is *the table's row axis is the
        // collection's* and a constant repeated on both sides cannot say whether the two components
        // reached it — see `crate::obligations`'s two keyboard lists, which is this shape one
        // obligation over.
        for reveal in [Reveal::WhenAsked, Reveal::EveryFrame, Reveal::Never] {
            let table = wheeled(Play::of(Subject::Table, reveal));
            let collection = wheeled(Play::of(Subject::Collection, reveal));
            assert_eq!(
                (table.settled, table.after_last_click, table.reveals),
                (
                    collection.settled,
                    collection.after_last_click,
                    collection.reveals
                ),
                "`table` and `collection` disagree about the wheel on the `{}` arm, so the row \
                 axis is not the one component reached by the other",
                reveal.word()
            );
            assert_eq!(
                revealed(Subject::Table, reveal, (0, SCROLLED_AWAY)),
                revealed(Subject::Collection, reveal, (0, SCROLLED_AWAY)),
                "and they disagree about the keyboard on the `{}` arm",
                reveal.word()
            );
        }
    }

    /// **The same gate over the shipped `tree`, and the sentence is checked rather
    /// than trusted.**
    ///
    /// > There is **no second selection store**, **no second scan cursor** — the row's `Face`
    /// > arrives from `collection`'s own lockstep `Scan` — **no second `Mode`**, **no second
    /// > offset** and **no second press edge**. The row axis, the wheel, the keyboard, the
    /// > type-ahead, the reveal, the tail below the content and the revision check are all
    /// > `collection`'s, reached by calling it. What this function adds is two verbs a row and a
    /// > one-slot request.
    ///
    /// That is `crate::collect::tree`'s own claim — **stated in more words than the table states the
    /// table's** — and until this ticket nothing had asked it a wheel question either. The numbers
    /// are `collection`'s to the click, which is the finding: **two verbs a row and a one-slot fold
    /// request cost the row axis nothing**, and a build where they did would show up here as one of
    /// these arms disagreeing with its twin one component down.
    ///
    /// # It is the arrangement and the third instance of it
    ///
    /// `table`/`collection` was the first and `file_preview_pane`/`scroll_area` the second, so the
    /// shape is now a rule: **a component whose spec says *reached by calling it* is compared
    /// against the component it calls, arm for arm, rather than against three constants written
    /// twice.** What this one cost is one `match` arm in [`Run::play`], one [`Subject`] variant and
    /// **two** `crate::collect::defective` entries — where the pane needed none, because its reveal
    /// is its body's and a tree's is `collection`'s and therefore reachable through a field.
    #[test]
    fn twenty_posted_clicks_move_a_trees_offset_twenty_and_the_numbers_are_the_collections() {
        let free = wheeled(Play::of(Subject::Tree, Reveal::WhenAsked));
        assert_eq!(free.settled, (0, MOVED), "twenty clicks, twenty rows");
        assert_eq!(free.after_last_click, (0, MOVED));
        assert_eq!(free.reveals, 0, "nothing asked, so nothing was requested");

        let dragged = wheeled(Play::of(Subject::Tree, Reveal::EveryFrame));
        assert_eq!(
            dragged.settled,
            (0, DRAGGED_BACK),
            "an unconditional `scroll_into_view` and the pointer is dead over a tree too"
        );
        assert_eq!(dragged.reveals, CLICKS);
        assert_eq!(
            dragged.after_last_click,
            (0, 1),
            "ADR 0015's residue, unchanged by the flatten index"
        );

        // **The second direction**, and it is what stops the arm below being a fix.
        assert_eq!(
            revealed(Subject::Tree, Reveal::WhenAsked, (0, SCROLLED_AWAY)),
            (0, -SCROLLED_AWAY),
            "a cursor moved to the top of the content brings the viewport with it"
        );
        assert_eq!(
            revealed(Subject::Tree, Reveal::Never, (0, SCROLLED_AWAY)),
            (0, 0),
            "and deleting the call loses it"
        );
        assert_eq!(
            wheeled(Play::of(Subject::Tree, Reveal::Never)).settled,
            (0, MOVED),
            "while passing the wheel half, which is why a one-directional gate is not a gate"
        );

        assert!(leaves_no_request(
            Play::of(Subject::Tree, Reveal::WhenAsked),
            (0, MOVED)
        ));
        assert!(
            !leaves_no_request(Play::of(Subject::Tree, Reveal::EveryFrame), (0, MOVED)),
            "twenty rows down, the unconditional arm asks to be dragged back on this very frame"
        );

        // **The three arms agree with `collection`'s, arm for arm**, and the press clause with it:
        // a tree adds a *second* pointer gesture on the chevron column (the fold-by-press), so
        // *does a press still select a row and still refuse to pull the viewport* is a question the
        // table arm did not have to ask.
        for reveal in [Reveal::WhenAsked, Reveal::EveryFrame, Reveal::Never] {
            let tree = wheeled(Play::of(Subject::Tree, reveal));
            let collection = wheeled(Play::of(Subject::Collection, reveal));
            assert_eq!(
                (tree.settled, tree.after_last_click, tree.reveals),
                (
                    collection.settled,
                    collection.after_last_click,
                    collection.reveals
                ),
                "`tree` and `collection` disagree about the wheel on the `{}` arm, so the row axis \
                 is not the one component reached by the other",
                reveal.word()
            );
            assert_eq!(
                revealed(Subject::Tree, reveal, (0, SCROLLED_AWAY)),
                revealed(Subject::Collection, reveal, (0, SCROLLED_AWAY)),
                "and they disagree about the keyboard on the `{}` arm",
                reveal.word()
            );
            assert_eq!(
                tapped(Play::of(Subject::Tree, reveal), (0, SCROLLED_AWAY)),
                tapped(Play::of(Subject::Collection, reveal), (0, SCROLLED_AWAY)),
                "and they disagree about the press on the `{}` arm — a press over the middle of \
                 the screen is nowhere near the chevron column, so a tree's extra gesture must \
                 leave this one exactly as it was",
                reveal.word()
            );
        }
    }

    /// **The same gate over the shipped `file_preview_pane`, and the sentence is
    /// checked rather than trusted.**
    ///
    /// > The file preview pane: **a scroll area over a document that arrives from another thread**.
    /// > … Everything else in its rectangle — the two reserved gutters, the corner and the cells
    /// > past the extent — is `crate::scroll::scroll_area`'s, which is what makes the pane a
    /// > partition of its rectangle without a single line of arithmetic here.
    ///
    /// That is `crate::files::file_preview_pane`'s own claim, and until this ticket nothing had
    /// asked it a wheel question. The numbers are the area's to the click, **on both axes** — which
    /// is the finding: a decode arriving from another thread costs the offset nothing, and a build
    /// where it did would show up here as one of these arms disagreeing with its twin one component
    /// down.
    ///
    /// # Written as a comparison and not as constants repeated
    ///
    /// The arrangement on the pair `table`/`collection`, and its reason: *the claim is
    /// `the pane's offset is the area's` and a constant repeated on both sides cannot say whether
    /// the two components reached it.* What it cost is one `match` arm in [`Run::play`], one
    /// [`Subject`] variant and **no** `crate::files::defective` entry at all — the three reveal arms
    /// are three bodies, because the pane's reveal is its body's.
    #[test]
    fn twenty_posted_clicks_move_a_panes_offset_twenty_and_the_numbers_are_the_areas() {
        // **The document has to have landed**, or the pane declares `(0, 0)`, admits no offset and
        // reports a dead wheel for a fixture's reason. `Run::new` lands it; this is the assertion
        // that says so, and without it every number below would be `0` and agree with itself.
        let opened = wheeled(Play::of(Subject::Pane, Reveal::WhenAsked));
        assert_ne!(
            opened.settled,
            (0, 0),
            "the pane is showing nothing, so its extent is `(0, 0)` and no offset is admissible — \
             a dead wheel and a meaningless axis are the same number"
        );

        for along in [Along::Rows, Along::Columns] {
            let free = Play::of(Subject::Pane, Reveal::WhenAsked).along(along);
            assert_eq!(
                wheeled(free).on(along),
                MOVED,
                "twenty clicks along the {}, twenty content cells",
                along.word()
            );
        }

        // **The blindness is per axis here too**, which is `Area::into_view`'s own property
        // reached through a second component: a body asking for content row 0 every frame is dead
        // downward and entirely healthy sideways.
        let pull_rows = Play::of(Subject::Pane, Reveal::EveryFrame).pulling(Along::Rows);
        assert_eq!(wheeled(pull_rows).on(Along::Rows), DRAGGED_BACK);
        assert_eq!(
            wheeled(pull_rows.along(Along::Columns)).on(Along::Columns),
            MOVED
        );

        // **The three arms agree with `scroll_area`'s, arm for arm and axis for axis.**
        for reveal in [Reveal::WhenAsked, Reveal::EveryFrame, Reveal::Never] {
            for along in [Along::Rows, Along::Columns] {
                let play = |subject| Play::of(subject, reveal).along(along).pulling(along);
                let pane = wheeled(play(Subject::Pane));
                let area = wheeled(play(Subject::Area));
                assert_eq!(
                    (pane.settled, pane.after_last_click, pane.reveals),
                    (area.settled, area.after_last_click, area.reveals),
                    "`file_preview_pane` and `scroll_area` disagree about the wheel on the `{}` \
                     arm along the {}, so the pane's offset is not the one component reached by \
                     the other",
                    reveal.word(),
                    along.word()
                );
            }
            assert_eq!(
                revealed(Subject::Pane, reveal, (0, SCROLLED_AWAY)),
                revealed(Subject::Area, reveal, (0, SCROLLED_AWAY)),
                "and they disagree about the reveal on the `{}` arm",
                reveal.word()
            );
        }

        // **The second direction**, and it is what stops the arm below being a fix.
        assert_eq!(
            revealed(Subject::Pane, Reveal::Never, (0, SCROLLED_AWAY)),
            (0, 0),
            "a body with no request left is a body nothing can reach"
        );
        assert_eq!(
            wheeled(Play::of(Subject::Pane, Reveal::Never)).settled,
            (0, MOVED),
            "while passing the wheel half, which is why a one-directional gate is not a gate"
        );
        assert!(leaves_no_request(
            Play::of(Subject::Pane, Reveal::WhenAsked),
            (0, SCROLLED_AWAY)
        ));
        assert!(
            !leaves_no_request(
                Play::of(Subject::Pane, Reveal::EveryFrame),
                (0, SCROLLED_AWAY)
            ),
            "two hundred rows down, the unconditional arm asks to be dragged back on this very \
             frame"
        );
        assert!(
            leaves_no_request(Play::of(Subject::Pane, Reveal::EveryFrame), (0, 0)),
            "and at the origin it asks for nothing, which is the frame this defect is invisible on"
        );
        assert_eq!(PANE_FRAMES, CLICKS + 2);
    }

    /// **Criterion 6: every subject this gate runs against declares the axis.**
    ///
    /// `INVENTORY`'s `owns_offset` column is `Axis::Wheeled`, so the two are one question — and
    /// asserting it is what stops the gate being run over a component the freeze says owns no
    /// offset, which is a gate measuring nothing while reporting a number.
    #[test]
    fn every_subject_of_this_gate_declares_the_wheeled_axis() {
        for subject in Subject::ALL {
            let row = INVENTORY
                .iter()
                .find(|c| c.id == subject.id())
                .unwrap_or_else(|| panic!("`{}` is not in the freeze", subject.id()));
            assert!(
                row.declares(Axis::Wheeled),
                "`{}` is a subject of the wheel gate and the freeze says it owns no offset",
                subject.id()
            );
            assert!(
                row.built,
                "`{}` is played through the shipped component, so the freeze has to say it is \
                 built",
                subject.id()
            );
        }
        assert_eq!(
            Subject::Collection.axes().len(),
            1,
            "a collection owns rows and nothing else"
        );
        assert_eq!(Subject::Area.axes().len(), 2);
        // **The pane's axes are the area's, and this pair really is two declarations**:
        // `Subject::axes` writes a literal for each arm, so an edit to one of them fails here.
        assert_eq!(
            Subject::Pane.axes(),
            Subject::Area.axes(),
            "the pane's axes are the area's, because the pane's offset is the area's"
        );

        // **There is no assertion that the two clamps agree, and its absence is a finding.**
        // `Subject::max_offset` answers them from **one** `Subject::Area | Subject::Pane` arm, so
        // an equality between the two is one expression compared with itself — the trap
        // `CLAUDE.md` names as *a gate that cannot fail*. What makes the clamps the same clamp is
        // asserted where it can fail instead: `twenty_posted_clicks_move_a_panes_offset_twenty_        // and_the_numbers_are_the_areas` compares the offsets **two runs of the shipped
        // components** came to rest at, on both axes and all three reveal arms. A review of this
        // ticket's first draft found the equality here and it was deleted rather than repaired.
    }

    /// The area's content is larger than the clicks can exhaust, on both axes.
    ///
    /// Otherwise `MOVED` would be a clamp reported as a wheel, which is the shape of a gate that
    /// passes on a build with no wheel at all.
    #[test]
    fn the_clicks_cannot_exhaust_the_content() {
        let max = area_max();
        assert!(max.0 > MOVED, "columns: {} against {MOVED}", max.0);
        assert!(max.1 > MOVED, "rows: {} against {MOVED}", max.1);
        assert!(
            i32::try_from(ROWS).expect("fits") - i32::from(H) > MOVED,
            "and the collection's rows likewise"
        );
    }
}
