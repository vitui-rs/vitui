//! A file picker, and the preview pane behind it.
//!
//! [`file_picker`] is the dialog: a listing on the left, a preview on the right, and an answer when
//! the user chooses. [`file_preview_pane`] is the right-hand half on its own, for an application
//! that has its own idea of what a listing looks like.
//!
//! # The preview is a question, not a read
//!
//! A preview pane never opens a file on the drawing thread. It asks — [`asking`] builds the
//! question, keyed by whatever identifies the file — and a worker answers on its own thread. The
//! pane draws what it has: nothing, a spinner, or the answer that landed. That is why an arrow key
//! held down through a directory of large files does not stall a frame.
//!
//! # The keyboard is the list's
//!
//! An open picker focuses the **listing**: the arrows move the cursor, `Enter` answers the file
//! under it, and `Esc` cancels and answers nothing. The preview pane has no keyboard and will not
//! get one — its document is replaced by the next arrow press, so there is nothing there to
//! navigate. It is scrolled with the pointer.
//!
//! # A landing is a shrink
//!
//! When an answer arrives, the pane's content usually changes size, and a pane that let its offset
//! stand would show a window past the end of a shorter document. The offset is clamped on the
//! frame the answer lands, which is the same rule [`crate::scroll`] applies to any content that
//! stops reaching its viewport.

use vitui_runtime::data::{Revision, Versioned};
use vitui_runtime::keys::{Code, Edge, Pressed};
use vitui_runtime::work::{Cancel, Requested, Task};
use vitui_runtime::{Ctx, Glyph, Id, Interest, Rect, Response, Role};

use crate::collect::{CollOpts, CollShape, CollState, Mode, collection_shaped};
use crate::frame::{Face, face_paint};
use crate::ink::{Direct, Ink};
use crate::order::Rows;
use crate::overlay::{Kind, ShellOpts, overlay_into};
use crate::scroll::{self, AreaOpts, AreaState, Hide};

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &["file_picker", "file_preview_pane"];

// ── what an answer is ────────────────────────────────────────────────────────────────────────────

/// **What a decoded preview is, from the pane's side.** Two methods, and both are about the *file*.
///
/// # `shows` is the eight bytes the staleness fix costs above the runtime's own
///
/// [`Task`] already drops a landing whose [`Generation`](vitui_runtime::work::Generation) is not the
/// newest — that is an answer to a question *nobody is asking any more*, and it is the runtime's
/// half. This is the other one: an answer to a question **nobody ever asked**, which arrives with a
/// perfectly current generation because the job that made it was started for the right question and
/// answered a different one:
///
/// > An asynchronous answer carries the identity of the question it answers, and the pane may write
/// > only on an equality against it — `shows() == dir.at(pos).ino`, 8 bytes on the payload.
///
/// **A test on the *answer* cannot see it**, which is why the equality is the pane's and not the
/// worker's: an ordering test asserts that answers arrive in the order they were asked for, and a
/// question that was never asked is not out of order.
///
/// # `extent` is a property of the file and not of the pane
///
/// The precondition — *a declared content size* — is a field of the answer, so while the next file
/// decodes the pane's extent is the previous file's: **4 000 where 800 is right**. That is what
/// makes a preview pane a *positive case* for bars-reserved rather than a new question for it, and
/// it is also the whole of [`Extent`].
pub trait Preview {
    /// **Which file this is the document of.** The identity the question was named with.
    fn shows(&self) -> u64;

    /// **How large the document is**, in content cells: `(columns, rows)`.
    fn extent(&self) -> (u32, u32);
}

/// **A `&` to an answer is an answer**, so a caller holding one behind a reference does not have to
/// name the concrete type twice. [`crate::media::Pixels`]'s arrangement, for its reason.
impl<T: Preview + ?Sized> Preview for &T {
    fn shows(&self) -> u64 {
        (**self).shows()
    }

    fn extent(&self) -> (u32, u32) {
        (**self).extent()
    }
}

/// **What [`PaneState::land`] did with the outbox.** Three arms, and the third is the one at
/// about.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Landed {
    /// Nothing was waiting. The overwhelming majority of frames.
    Nothing,
    /// An answer arrived, matched the question, and is now what the pane shows.
    Shown,
    /// **An answer arrived for a question nobody asked** and was dropped without a cell written.
    Refused,
}

// ── the four axes a pane can be spelled on ───────────────────────────────────────────────────────

/// **Where the offset goes when a new file arrives.** The table, less the row that is not on this
/// axis at all — see [`Extent`] — and less the per-file map, which is [`defective::Mapped`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Reset {
    /// **The rule.** Zeroed when the answer lands: right on arrival, forgets on return. That is the
    /// trade rather than a defect, and the spelling that is right in both directions is refused on
    /// release rather than on behaviour.
    #[default]
    OnTheLanding,
    /// **The defect.** Zeroed when the question is asked, which scrolls the *previous* file — still
    /// on screen — to its top for the length of the decode.
    OnTheRequest,
    /// **The defect.** Left where it was. The new file opens at its last page, because
    /// [`crate::scroll::scroll_area`] clamps against the extent it was handed.
    Never,
}

/// **What content size the pane declares**, and this is where *unclamped* row lives.
///
/// # the five spellings are four on the offset axis, and the fifth is on this one
///
/// An *unclamped* offset costs **the body drawing nothing at all**, and there is no way to
/// leave an offset unclamped in this build: [`crate::scroll::scroll_area`] clamps against the extent
/// it is handed, on **every** frame, and the clamp is free. What a caller can do instead is decline
/// to bound it — declare an extent no offset can be past — and that produces the row exactly,
/// with its cause named.
///
/// **And it costs two things rather than one.** The offset survives the shrink, so at row 3 926 over
/// an 800-row document the body draws nothing; *and* the area's tail is
/// `[extent, offset + viewport)`, which over an unbounded extent is **empty**, so the cells the
/// body cannot write are cells nobody writes. *1 650 writes against 4 166* is a partition
/// failure and not only a blank body — see [`crate::preview::UNCLAMPED_WRITES`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Extent {
    /// **The rule.** The content size of the answer the pane is showing, which is a property of the
    /// file: while the next one decodes the pane's extent is the previous one's, **4 000 where 800
    /// is right**, and that is what makes a preview pane a positive case for bars-reserved.
    #[default]
    OfTheAnswer,
    /// **The defect.** No bound at all, so nothing clamps and there is no tail:
    /// **0 content cells against 2 516**, on a rectangle that is no longer a partition.
    Unbounded,
}

/// **When the pane's revision advances**, which is what every memo over its document keys on.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Bump {
    /// **The rule.** On a landing, and on nothing else.
    #[default]
    OnTheLanding,
    /// **The defect.** Every frame, because [`Edit`](vitui_runtime::data::Edit) bumps on drop and a
    /// landing that did not happen is still a drop. The two screens are cell-identical and the
    /// highlighter re-folds the whole file on every one of them.
    ///
    /// It is spent in [`PaneState::land`], which is where the guard would be taken, so it says
    /// nothing about a pane whose landing is [`Taken::InsideTheDraw`] — that arm is a *second*
    /// defect and the two are not composed.
    EveryFrame,
}

/// **Where the landing is taken.** R18 the unenforced ordering, as an axis.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Taken {
    /// **The rule.** Once, at the top of the view, by [`PaneState::land`], before anything draws.
    #[default]
    AtTheTopOfTheView,
    /// **The defect.** Inside the pane's own draw, so a caller with a status row on each side of the
    /// pane draws the two from two versions of one value — with no thread having done anything at
    /// all, because [`Task::take`] is destructive.
    InsideTheDraw,
}

/// **The four axes as one value**, so that a reviewer's diff between the shipped pane and any
/// refused one is a single line.
///
/// [`crate::disclose::Collapses`]'s arrangement and [`crate::input`]'s, for their reason: the diff a
/// reviewer would have to catch is the diff the register can point at. [`Default`] is what ships.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct PaneShape {
    /// Where the offset goes.
    pub reset: Reset,
    /// Which file's content size is declared.
    pub extent: Extent,
    /// When the revision advances.
    pub bump: Bump,
    /// Where the landing is taken.
    pub taken: Taken,
}

// ── the pane's state ─────────────────────────────────────────────────────────────────────────────

/// **Everything a preview pane keeps across frames.** The document, the offset and two counters.
///
/// # There is no map in it, and that is the release argument
///
/// The one offset spelling that is right in both directions is a slot per *file*, and **nothing
/// releases it**: R02's sweep releases what an [`Id`] stopped drawing and a file is not an `Id`. At
/// a million files that is [`crate::preview::MAP_AT_A_MILLION`] bytes against one slot's four. The
/// spelling is kept as [`defective::Mapped`] so the refusal is priced rather than asserted.
///
/// # `T` is the application's and the pane never makes one
///
/// The pane holds the answer, reads [`Preview`] off it, and hands it back to the caller's line
/// drawer. It does not decode, does not spawn, and does not own the [`Task`] the answer comes
/// through.
pub struct PaneState<T> {
    /// The document, and the revision every memo over it keys on. Bumped **only** on a landing.
    shown: Versioned<Option<T>>,
    /// The offset, held where [`crate::scroll::scroll_area`] expects it.
    area: AreaState,
    /// The extent of the answer being shown.
    declared: (u32, u32),
    /// How many answers have landed and been shown.
    landings: u64,
    /// **How many answered a question nobody asked.** [`Preview::shows`]'s counter.
    refused: u64,
    /// Which of the four spellings this pane is.
    shape: PaneShape,
}

impl<T> Default for PaneState<T> {
    fn default() -> PaneState<T> {
        PaneState::new()
    }
}

impl<T> PaneState<T> {
    /// **A pane with nothing shown, spelled the way that ships.**
    #[must_use]
    pub fn new() -> PaneState<T> {
        PaneState::shaped(PaneShape::default())
    }

    /// A pane spelled some other way. Every arm but [`PaneShape::default`] is a defect with a number
    /// beside it in `crate::preview`.
    #[must_use]
    pub fn shaped(shape: PaneShape) -> PaneState<T> {
        PaneState {
            shown: Versioned::new(None),
            area: AreaState::default(),
            declared: (0, 0),
            landings: 0,
            refused: 0,
            shape,
        }
    }

    /// Which document the pane is showing, if one has landed.
    pub fn showing(&self) -> Option<&T> {
        self.shown.as_ref()
    }

    /// **The revision every memo over the document keys on.** It advances on a landing and on
    /// nothing else — see [`Bump`].
    pub fn revision(&self) -> Revision {
        self.shown.revision()
    }

    /// The first visible document row.
    pub fn offset(&self) -> u32 {
        u32::try_from(self.area.offset.1).unwrap_or(0)
    }

    /// Move the offset, the way a `PageDown` would. Clamped by the pane on the next frame, against
    /// the extent it declares.
    pub fn scroll_to(&mut self, row: u32) {
        self.area.offset.1 = i32::try_from(row).unwrap_or(i32::MAX);
    }

    /// How many answers have landed.
    pub fn landings(&self) -> u64 {
        self.landings
    }

    /// **How many answers were dropped on [`Preview::shows`].** A question that was never asked.
    pub fn refused(&self) -> u64 {
        self.refused
    }

    /// Which spelling this pane is.
    pub fn shape(&self) -> PaneShape {
        self.shape
    }
}

impl<T: Preview> PaneState<T> {
    /// **Take the landing, at the top of the view.**
    ///
    /// [`Task::take`] is destructive, so the answer is taken **once**, before anything draws, into a
    /// value every reader on the frame shares. A view that asks for it wherever it happens to want
    /// it hands the *first* consumer the answer and the second `None`, on the same frame, with no
    /// thread having done anything at all — and a view has no `&mut` with which to put back what it
    /// took. That is R18 the unenforced ordering, and it is
    /// [`crate::preview::TORN`]`[0]` frames of 20 against 0.
    ///
    /// It is a verb on the state and not a step inside [`file_preview_pane`] for that reason: a
    /// component that takes its own landing mid-draw is [`Taken::InsideTheDraw`], and a caller with
    /// a status row on each side of the pane straddles it.
    ///
    /// The equality is [`Preview::shows`] against the question actually being asked. An answer that
    /// fails it is dropped **without a cell written** and without the revision advancing.
    pub fn land(&mut self, task: &Task<T>) -> Landed {
        let Some(answer) = task.take() else {
            // **The revision does not move**, and the branch is the whole of `Bump`: written
            // without it, `Versioned::edit`'s guard drops on every frame and every memo over the
            // document misses.
            if self.shape.bump == Bump::EveryFrame {
                let _bumped = self.shown.edit();
            }
            return Landed::Nothing;
        };
        if task.asking() != Some(answer.shows()) {
            // **An answer to a question nobody asked.** The generation was current — the job was
            // started for the right question — and the payload is another file's.
            self.refused += 1;
            return Landed::Refused;
        }
        self.declared = answer.extent();
        if self.shape.reset == Reset::OnTheLanding {
            self.area.offset = (0, 0);
        }
        *self.shown.edit() = Some(answer);
        self.landings += 1;
        Landed::Shown
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for PaneState<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PaneState")
            .field("shown", &self.shown)
            .field("offset", &self.area.offset)
            .field("declared", &self.declared)
            .field("landings", &self.landings)
            .field("refused", &self.refused)
            .finish()
    }
}

// ── the question ─────────────────────────────────────────────────────────────────────────────────

/// **The question the pane asks this frame: what to name it, and how to answer it.**
///
/// # The key is the *file* and every natural way to name it names a position
///
/// That is the memo-key rule arriving through the one door whose trigger is not a gesture. One
/// re-sort — one line of application code, no keystroke — and a position-keyed pane is wrong on
/// **100 of 100 frames** afterwards, at the same number of questions either way, *because the
/// question still matches, so nothing posts, so nothing wakes, so no frame corrects it.*
///
/// # `decode` is built every frame and boxed on almost none of them
///
/// [`Task::request`] is called unconditionally and returns [`Requested::Deduped`] before it looks at
/// the job, so building a closure literal here is a stack write on every frame but the one that
/// asks. The pane therefore allocates nothing on a steady frame, which is what the standing budget
/// requires of anything a component calls.
pub struct Question<F> {
    /// **The file's own identity.** Never its position in a listing.
    pub key: u64,
    /// How to answer it, off the app thread. Called at most once, on the frame the question changes.
    pub decode: F,
}

impl<F> Question<F> {
    /// A question named by a file's identity.
    ///
    /// # The `&Cancel` has to be annotated, and it is a Rust limitation rather than a design one
    ///
    /// A closure's signature is inferred from the **expected type at the expression**, and there is
    /// none here: `Question` is data, so `decode` is type-checked before it ever meets
    /// [`file_preview_pane`]'s higher-ranked bound. Written bare, the compiler infers one specific
    /// lifetime and the call site fails with *implementation of `FnOnce` is not general enough* —
    /// which names neither the closure nor the parameter. Write `move |cancel: &Cancel| …`, which
    /// [`asking`] does for a caller who would rather not.
    ///
    /// # Examples
    ///
    /// ```
    /// use vitui_runtime::work::Cancel;
    /// use vitui_components::files::Question;
    ///
    /// // The annotation is the whole of the difference; `asking` is the same value without it.
    /// let q = Question::new(7, move |_cancel: &Cancel| 7u32);
    /// assert_eq!(q.key, 7);
    /// ```
    pub fn new(key: u64, decode: F) -> Question<F> {
        Question { key, decode }
    }
}

/// **[`Question::new`] with the bound on it**, so a closure written here needs no annotation.
///
/// `T` appears in the `where` clause, which is what gives the closure an expected signature and
/// makes the higher-ranked lifetime infer. It is a free function and not a constructor because the
/// payload type has to be a parameter of the *function* and `Question` does not carry one — a
/// `PhantomData<T>` on the struct would buy the same inference and cost every caller a type it does
/// not otherwise name.
pub fn asking<T, F>(key: u64, decode: F) -> Question<F>
where
    F: FnOnce(&Cancel) -> T + Send + 'static,
{
    Question { key, decode }
}

// ── the pane's options ───────────────────────────────────────────────────────────────────────────

/// [`file_preview_pane`]'s options. rule 3: a `Default` struct, never a required builder.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PaneOpts {
    /// The role the cells past the document's extent are written in.
    pub tail: Role,
    /// What the pane is interested in besides the wheel.
    pub interest: Interest,
    /// How the two bars are drawn.
    pub bar: scroll::ScrollbarOpts,
}

impl Default for PaneOpts {
    fn default() -> PaneOpts {
        PaneOpts {
            tail: Role::Body,
            interest: Interest::CLICK.with(Interest::HOVER),
            bar: scroll::ScrollbarOpts::default(),
        }
    }
}

impl PaneOpts {
    /// What [`crate::scroll::scroll_area`] is asked for.
    ///
    /// **[`Hide::Never`] is not a default carried through; it is the decision.** A pane's extent is a
    /// property of the *file*, so a gutter cut only when an axis overflows appears and disappears as
    /// the selection moves — the furniture jumping while the content does, which is the
    /// consequence rather than as a preference: *that makes a preview pane a positive case for
    /// bars-reserved rather than a new question for it.*
    fn area(self) -> AreaOpts {
        AreaOpts {
            hide: Hide::Never,
            bar: self.bar,
            tail: self.tail,
            interest: self.interest,
            ..AreaOpts::default()
        }
    }
}

// ── the pane ─────────────────────────────────────────────────────────────────────────────────────

/// **The file preview pane**: a scroll area over a document that arrives from another thread.
///
/// **Hostile axes:** `scrolled`, `shrunk`, `wheeled`.
///
/// Two scenes, and the second is *a landing is a shrink*,
/// from another thread for the first time. The offset belongs to neither side, which is where the
/// five spellings of the second come from.
///
/// It asks its question on every frame, declares the extent of the answer it is showing, and calls
/// `line` once per document row that is both inside the viewport and inside the document. Everything
/// else in its rectangle — the two reserved gutters, the corner and the cells past the extent — is
/// [`crate::scroll::scroll_area`]'s, which is what makes the pane a partition of its rectangle
/// without a single line of arithmetic here.
///
/// **The landing is not taken here.** Call [`PaneState::land`] at the top of the view first; see its
/// documentation for the twenty torn frames that is about.
///
/// # Examples
///
/// ```
/// use vitui_components::files::{PaneState, Preview, asking, file_preview_pane};
/// use vitui_runtime::ctx::Driver;
/// use vitui_runtime::work::{Task, Worker};
///
/// struct Doc { id: u64 }
/// impl Preview for Doc {
///     fn shows(&self) -> u64 { self.id }
///     fn extent(&self) -> (u32, u32) { (8, 3) }
/// }
///
/// let worker = Worker::queueing();
/// let task: Task<Doc> = Task::new(&worker);
/// let mut pane = PaneState::new();
/// let mut driver = Driver::headless(20, 5).expect("a sink attaches");
///
/// // Frame one asks; the worker answers; frame two lands it and draws it.
/// for frame in 0..2 {
///     if frame == 1 {
///         worker.run(0);
///     }
///     pane.land(&task);
///     driver.frame(|cx| {
///         let area = cx.area();
///         let _ = file_preview_pane(
///             cx,
///             area,
///             &mut pane,
///             &task,
///             asking(7, |_cancel| Doc { id: 7 }),
///             &mut |cx, row, _doc, i| {
///                 let paint = cx.theme().paint(vitui_runtime::Role::Body);
///                 let _ = cx.text(row.x, row.y, &format!("row {i:03}"), paint);
///             },
///         );
///     });
/// }
/// assert_eq!(pane.showing().map(Preview::shows), Some(7));
/// assert_eq!(pane.landings(), 1);
/// ```
#[track_caller]
pub fn file_preview_pane<T, F>(
    cx: &mut Ctx<'_, '_>,
    rect: Rect,
    st: &mut PaneState<T>,
    task: &Task<T>,
    q: Question<F>,
    line: &mut dyn FnMut(&mut Ctx<'_, '_>, Rect, &T, u32),
) -> Response
where
    T: Preview + Send + 'static,
    F: FnOnce(&Cancel) -> T + Send + 'static,
{
    file_preview_pane_with(cx, rect, st, task, q, &PaneOpts::default(), line)
}

/// [`file_preview_pane`], with the options spelled out.
#[track_caller]
pub fn file_preview_pane_with<T, F>(
    cx: &mut Ctx<'_, '_>,
    rect: Rect,
    st: &mut PaneState<T>,
    task: &Task<T>,
    q: Question<F>,
    opts: &PaneOpts,
    line: &mut dyn FnMut(&mut Ctx<'_, '_>, Rect, &T, u32),
) -> Response
where
    T: Preview + Send + 'static,
    F: FnOnce(&Cancel) -> T + Send + 'static,
{
    let id = cx.id();
    file_preview_pane_into(
        &mut Direct,
        cx,
        id,
        rect,
        st,
        task,
        q,
        opts,
        |_ink, cx, row, doc, i| line(cx, row, doc, i),
    )
}

/// **[`file_preview_pane`], drawing through an [`Ink`] and under an id its caller minted.**
///
/// The entry point a gate takes; [`file_preview_pane`] is this with [`Direct`]. See [`crate::ink`]
/// for why the seam exists rather than a second implementation written against a `Tally`.
///
/// The id is a parameter for [`crate::scroll::scroll_area_into`]'s reason: a pane drawn on behalf of
/// a component that already claimed an id must not mint a second one, and `#[track_caller]` cannot
/// see through that component's own call.
#[expect(
    clippy::too_many_arguments,
    reason = "the component's own six — a context, an id, a rectangle, the state, the task the \
              answer comes through and the question — plus the options, the line drawer and the \
              `Ink` seam's writer. The task and the question cannot be folded together: a \
              job's lifetime is the question's and the question changes every time the cursor \
              moves"
)]
pub fn file_preview_pane_into<I, T, F, L>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    id: Id,
    rect: Rect,
    st: &mut PaneState<T>,
    task: &Task<T>,
    q: Question<F>,
    opts: &PaneOpts,
    mut line: L,
) -> Response
where
    I: Ink,
    T: Preview + Send + 'static,
    F: FnOnce(&Cancel) -> T + Send + 'static,
    L: FnMut(&mut I, &mut Ctx<'_, '_>, Rect, &T, u32),
{
    if rect.is_empty() {
        return Response::inert(id, rect);
    }

    // **The question, asked unconditionally, every frame.** `Task::request` deduplicates on the key
    // before it looks at the job, so the closure literal above is a stack write on every frame but
    // the one that asks — and the whole of the memo-key rule is which value `q.key` is.
    let asked = task.request(q.key, q.decode);
    if asked == Requested::Asked && st.shape.reset == Reset::OnTheRequest {
        // **The defect.** The previous file is still on screen and this scrolls it to its top for
        // the length of the decode.
        st.area.offset = (0, 0);
    }
    if st.shape.taken == Taken::InsideTheDraw {
        // **The defect.** Taking it here rather than at the top of the view is what makes two
        // readers of one value disagree on one frame.
        if let Some(answer) = task.take()
            && task.asking() == Some(answer.shows())
        {
            st.declared = answer.extent();
            if st.shape.reset == Reset::OnTheLanding {
                st.area.offset = (0, 0);
            }
            *st.shown.edit() = Some(answer);
            st.landings += 1;
        }
    }

    // **The extent is a field of the answer.** `OfTheAnswerBefore` is *unclamped* row, and it
    // is on this axis rather than on the offset's: the clamp runs every frame against whatever it is
    // handed, so the only way to a body that draws nothing is to hand it a stale one.
    let extent = match st.shape.extent {
        Extent::OfTheAnswer => st.declared,
        Extent::Unbounded => (st.declared.0, u32::MAX),
    };

    let PaneState { shown, area, .. } = st;
    let doc = shown.as_ref();
    let area_opts = opts.area();
    scroll::scroll_area_into(
        ink,
        cx,
        id,
        rect,
        area,
        &area_opts,
        extent,
        |_ink, _cx, _band| {},
        |ink, cx| {
            let Some(doc) = doc else {
                // Nothing has landed. Every cell of the rectangle is the area's tail, which it
                // writes itself: `extent` is `(0, 0)` and the whole viewport is past it.
                return;
            };
            // **The body virtualises**, which is `CONTEXT.md`'s invariant: frame cost is
            // proportional to visible cells and never to the document's length.
            let window = cx.visible_rows();
            let cols = cx.visible_cols();
            let top = u32::try_from(window.start.max(0)).unwrap_or(0);
            let height = u32::try_from((window.end - window.start).max(0)).unwrap_or(0);
            // **Bounded by the *document* and not by what was declared**, which is the one place the
            // two can differ: `Extent::Unbounded` declares a size no offset is past, and a drawer
            // asked for a row the document has not got would be asked to invent one.
            let held = doc.extent();
            let last = top.saturating_add(height).min(held.1);
            // **The row the drawer is handed stops at the extent**, because the cells past it are
            // the area's tail and writing them here would write them twice.
            let right = cols.end.min(i32::try_from(held.0).unwrap_or(i32::MAX));
            let w = u16::try_from((right - cols.start).max(0)).unwrap_or(u16::MAX);
            for row in top..last {
                let y = i32::try_from(row).unwrap_or(i32::MAX);
                line(ink, cx, Rect::new(cols.start, y, w, 1), doc, row);
            }
        },
    )
}

// ── the picker ───────────────────────────────────────────────────────────────────────────────────

/// **One row of a picker's listing**: an identity and a name, which is all a picker needs.
///
/// It is deliberately not the application's file record. A picker asks *which file* and shows *what
/// it is called*; everything else about a file — its size, its mode, its mtime — belongs to the
/// caller's listing and to the caller's decode.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Entry<'a> {
    /// **The file's own identity**, which is the key the pane's question is named with.
    pub id: u64,
    /// What the list draws.
    pub name: &'a str,
}

/// **What the picker's owner keeps**: whether it is open, and what was chosen.
///
/// [`crate::input::SelectState`]'s two-struct arrangement, and for the reason: the owner and the
/// body are two writers and folding them together is the thing that cannot be done.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct PickerState {
    /// Whether the picker's overlay is standing.
    open: bool,
    /// Whether the body has held the keyboard since it opened.
    seated: bool,
    /// The identity of the file that was chosen, if one was.
    chosen: Option<u64>,
}

impl PickerState {
    /// A shut picker with nothing chosen.
    #[must_use]
    pub fn new() -> PickerState {
        PickerState::default()
    }

    /// Whether the overlay is standing.
    pub fn is_open(self) -> bool {
        self.open
    }

    /// Open it. Clears the latch, for [`crate::input::SelectState::open`]'s reason: without the
    /// clearing the blur clause fires on the frame after the reopening.
    pub fn open(&mut self) {
        self.open = true;
        self.seated = false;
    }

    /// Shut it.
    pub fn close(&mut self) {
        self.open = false;
        self.seated = false;
    }

    /// Which file was chosen, if one was.
    pub fn chosen(self) -> Option<u64> {
        self.chosen
    }

    /// Seat the choice, the way an application restoring its last session would.
    pub fn choose(&mut self, file: u64) {
        self.chosen = Some(file);
    }
}

/// **What the picker's body keeps**: the list, the pane and the one slot the answer comes back in.
///
/// It is the caller's storage, borrowed for the frame — [`crate::overlay`]'s `ByMutRef` rule, and
/// the reason is measured there: a body holding a `Copy` of its own position writes into a value
/// that dies with the frame.
pub struct PickerBody<T> {
    /// The listing's cursor, selection and type-ahead.
    pub list: CollState,
    /// **The preview pane's document and offset, and it is private.**
    ///
    /// A component that takes its own landing is refused, and what makes the refusal hold is
    /// that there is **one** consumer. A `pub` field here would put a second `PaneState::land` in
    /// reach of the caller, on the one screen where a second reader of the same value is the whole
    /// defect. [`PickerBody::pane`] is the read-only half.
    pane: PaneState<T>,
    /// What the body chose, read once by the owner on the next frame.
    answer: Option<u64>,
    /// **That the body was dismissed with nothing chosen**, read once by the owner on the next
    /// frame beside [`PickerBody::answer`].
    ///
    /// Two slots and not one, and the reason is that this family's two owners hold their choice in
    /// different shapes. `select`'s popup answers the *previously chosen*
    /// index on `Esc`, which works because a `SelectState` always has one; a picker's
    /// [`PickerState::chosen`] is an `Option<u64>` that starts empty, so *answer what was already
    /// there* is `None` on the first open — indistinguishable from *nothing has been decided yet*,
    /// which is the state the owner uses to decide whether to close at all. Cancelling is therefore
    /// its own fact rather than an answer with a special value.
    cancelled: bool,
    /// Whether the focus was inside the body when it last ran.
    inside: bool,
    /// Whether the pointer was over it when it last ran.
    over: bool,
}

impl<T> Default for PickerBody<T> {
    fn default() -> PickerBody<T> {
        PickerBody::new()
    }
}

impl<T> PickerBody<T> {
    /// An empty body.
    #[must_use]
    pub fn new() -> PickerBody<T> {
        PickerBody {
            list: CollState::new(),
            pane: PaneState::new(),
            answer: None,
            cancelled: false,
            inside: false,
            over: false,
        }
    }

    /// Whether the focus was inside the body when it last ran.
    pub fn inside(&self) -> bool {
        self.inside
    }

    /// Whether the pointer was over it when it last ran.
    pub fn over(&self) -> bool {
        self.over
    }

    /// **The preview pane, to read.** There is deliberately no `&mut` twin — see the field.
    pub fn pane(&self) -> &PaneState<T> {
        &self.pane
    }
}

/// [`file_picker`]'s options.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PickerOpts {
    /// How wide the listing is inside the overlay, in columns.
    pub list_w: u16,
    /// How the pane inside it is drawn.
    pub pane: PaneOpts,
    /// What the shut face is interested in.
    pub interest: Interest,
}

impl Default for PickerOpts {
    fn default() -> PickerOpts {
        PickerOpts {
            list_w: 24,
            pane: PaneOpts::default(),
            interest: Interest::CLICK.with(Interest::HOVER).with(Interest::FOCUS),
        }
    }
}

/// **The file picker: `collection` + `overlay` + [`file_preview_pane`], and no mechanism that is
/// new.**
///
/// **Hostile axes:** `scrolled`, `shrunk`, `wheeled`.
///
/// All three are [`crate::collect::collection`]'s and [`file_preview_pane`]'s, which is
/// R3's claim read on this list: a composition that introduces no mechanism introduces no axis
/// either.
///
/// **R3's claim is checked rather than asserted**, because it is the class that turns out
/// to be false when it is false: `crate::preview::picker_introduces_no_mechanism` reads this
/// function's own source for a second [`Task`], a second [`CollState`], a second hit entry and a
/// second key loop, and `crate::preview::picker_declares_what_its_parts_declare` compares the frame
/// it produces against the three parts drawn by hand.
///
/// **`'f` is the mechanism and not an annotation** — the sentence about the fifth
/// component, one family over. The body outlives the base pass, so everything it touches is borrowed
/// for the frame: the caller's body state, the listing, and the task the answer comes through.
///
/// `decode` is a **function pointer** rather than a closure, and the reason is the body: a job must
/// be `Send + 'static` and a body is `FnMut`, so the thing that makes a job has to be callable more
/// than once and may capture nothing that dies with the frame. A `fn(u64, &Cancel) -> T` is `Copy`,
/// `Send`, `Sync` and `'static` by construction, which is exactly the set. A decode is a free
/// function over an identity in every application that has one.
///
/// # Examples
///
/// ```
/// use vitui_components::files::{
///     Entry, PickerBody, PickerOpts, PickerState, Preview, file_picker,
/// };
/// use vitui_runtime::Rect;
/// use vitui_runtime::ctx::Driver;
/// use vitui_runtime::work::{Cancel, Task, Worker};
///
/// struct Doc(u64);
/// impl Preview for Doc {
///     fn shows(&self) -> u64 { self.0 }
///     fn extent(&self) -> (u32, u32) { (8, 2) }
/// }
///
/// // A decode is a free function over an identity — never a closure. See above.
/// fn decode(id: u64, _cancel: &Cancel) -> Doc { Doc(id) }
/// fn line(cx: &mut vitui_runtime::ctx::Ctx<'_, '_>, row: Rect, _doc: &Doc, i: u32) {
///     let paint = cx.theme().paint(vitui_runtime::Role::Body);
///     let _ = cx.text(row.x, row.y, &format!("row {i:03}"), paint);
/// }
///
/// let names = ["notes.md", "main.rs"];
/// let files: Vec<Entry<'_>> = names
///     .iter()
///     .enumerate()
///     .map(|(i, n)| Entry { id: i as u64, name: n })
///     .collect();
///
/// let worker = Worker::queueing();
/// let task: Task<Doc> = Task::new(&worker);
/// let mut body: PickerBody<Doc> = PickerBody::new();
/// let mut st = PickerState::new();
/// let opts = PickerOpts::default();
///
/// let mut driver = Driver::headless(60, 12).expect("a sink attaches");
/// driver.frame(|cx| {
///     let resp = file_picker(
///         cx,
///         Rect::new(0, 0, 40, 1),
///         &mut st,
///         &mut body,
///         &files,
///         &task,
///         decode,
///         line,
///         &opts,
///     );
///     assert_eq!(resp.rect.h, 1, "shut, the picker is its own one row");
/// });
/// // Shut: one region, and no overlay has been placed.
/// assert_eq!(driver.inspect().overlays_placed(), 0);
/// ```
#[track_caller]
#[expect(
    clippy::too_many_arguments,
    reason = "the component's own five — a context, a rectangle, the owner's state, the body's \
              state and the listing — plus the task, the decode and the options. The two states \
              cannot be folded together, and the task cannot join them because its lifetime \
              is the question's"
)]
pub fn file_picker<'f, T>(
    cx: &mut Ctx<'f, '_>,
    area: Rect,
    st: &mut PickerState,
    body: &'f mut PickerBody<T>,
    files: &'f [Entry<'f>],
    task: &'f Task<T>,
    decode: fn(u64, &Cancel) -> T,
    line: fn(&mut Ctx<'_, '_>, Rect, &T, u32),
    opts: &PickerOpts,
) -> Response
where
    T: Preview + Send + 'static,
{
    let id = cx.id();
    file_picker_into(
        &mut Direct,
        cx,
        id,
        area,
        st,
        body,
        files,
        task,
        decode,
        line,
        opts,
    )
}

/// **What the picker's own list refuses**, as one value.
///
/// It is [`crate::input::SelectShape`]'s `list` field one component over and for
/// its reason: the picker's body is *the shell, then a collection beside a preview
/// pane*, so the axes a windowed list can be wrong on are `collection`'s vocabulary reached by
/// calling it. One struct rather than three booleans, which is
/// [`crate::collect::TableShape`]'s arrangement and its reason — three enums side by side in a call
/// are the arguments a reader transposes.
///
/// **The pane's four axes are not here.** [`defective`] carries those, they are `PaneState`'s own
/// and they are `file_preview_pane`'s rather than `file_picker`'s — scenes 23 and 24 measure them.
/// This value is the *picker*'s, and what the picker adds to the pane is the list.
///
/// [`crate::input::SelectShape`]: crate::input
/// [`crate::collect::TableShape`]: crate::collect
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct PickerShape {
    /// What the list refuses. [`CollShape::RULE`] is what the shipped picker passes.
    pub(crate) list: CollShape,
    /// Whether the body takes the keyboard from its owner. See [`Keyboard`].
    pub(crate) keyboard: Keyboard,
}

/// **Whether the picker's body has a keyboard at all**, and the refused arm is what shipped.
///
/// Until this was answered, `picker_body` seated no focus and declared no
/// [`crate::collect::Refusal`], so **an open `file_picker` could only be used with a mouse**: no
/// arrows, no `Home`/`End`, no type-ahead and no way to choose a file from the keyboard. It rendered
/// perfectly, which is why nothing caught it — every gate in this module and in [`crate::preview`]
/// drives the picker with a pointer or asserts about the pane.
///
/// It is an axis rather than a deleted branch for [`CollShape`]'s reason: a gate written against
/// *The same screen with one thing changed* separates a keyboard from its absence, where a gate
/// written against a second implementation would test the second implementation.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum Keyboard {
    /// **The rule.** The body takes the keyboard from its owner exactly once and reads `Enter` and
    /// `Esc` through the collection's one drain loop.
    #[default]
    Seated,
    /// **What shipped for a while**: no focus, no refusal, a pointer-only popup.
    Unseated,
}

/// **[`file_picker`]'s shut face, drawn through an [`Ink`] and under an id its caller minted.**
///
/// The entry point a gate takes; [`file_picker`] is this with [`Direct`]. See [`crate::ink`] for why
/// the seam exists rather than a second implementation written against a `Tally`.
///
/// # The popup's body draws through [`Direct`] and not through `ink`
///
/// A body is `FnMut(&mut Ctx<'f, '_>) + 'f`, so a `&mut I` borrowed for this call cannot travel into
/// one — [`crate::input::select_into`]'s note, and its reason: a body's `Ctx` is rooted at its own
/// layer, so a tally that saw both would union two grids. What `ink` sees is the **shut face**,
/// which is what the partition rule is about here.
#[track_caller]
#[expect(
    clippy::too_many_arguments,
    reason = "the component's own eight plus the `Ink` seam's writer and the id its caller minted. \
              See `file_picker` for why none of the eight folds into another"
)]
pub fn file_picker_into<'f, I, T>(
    ink: &mut I,
    cx: &mut Ctx<'f, '_>,
    id: Id,
    area: Rect,
    st: &mut PickerState,
    body: &'f mut PickerBody<T>,
    files: &'f [Entry<'f>],
    task: &'f Task<T>,
    decode: fn(u64, &Cancel) -> T,
    line: fn(&mut Ctx<'_, '_>, Rect, &T, u32),
    opts: &PickerOpts,
) -> Response
where
    I: Ink,
    T: Preview + Send + 'static,
{
    file_picker_shaped(
        ink,
        cx,
        id,
        area,
        st,
        body,
        files,
        task,
        decode,
        line,
        opts,
        PickerShape::default(),
    )
}

/// **The component, with the refused list spellings threaded in.**
///
/// [`crate::input::select`]'s `select_shaped` one component over, and for its reason: a refused arm
/// has to differ from the shipped build by **one value**, so a reviewer's diff between them is a
/// single line and the register can point at it.
#[track_caller]
#[expect(
    clippy::too_many_arguments,
    reason = "`file_picker_into`'s eleven plus the one value that carries every refused spelling, \
              so the arms are one call apart"
)]
fn file_picker_shaped<'f, I, T>(
    ink: &mut I,
    cx: &mut Ctx<'f, '_>,
    id: Id,
    area: Rect,
    st: &mut PickerState,
    body: &'f mut PickerBody<T>,
    files: &'f [Entry<'f>],
    task: &'f Task<T>,
    decode: fn(u64, &Cancel) -> T,
    line: fn(&mut Ctx<'_, '_>, Rect, &T, u32),
    opts: &PickerOpts,
    shape: PickerShape,
) -> Response
where
    I: Ink,
    T: Preview + Send + 'static,
{
    if area.is_empty() {
        return Response::inert(id, area);
    }

    // **The inbox, taken first**, which is what makes `PickerState` single-writer.
    //
    // **Two slots, and cancelling leaves `chosen` alone** — which is how a dismissal *restores*
    // rather than clears here, where `select`'s popup has to say the previous index out loud to get
    // the same effect.
    let dismissed = std::mem::take(&mut body.cancelled);
    if let Some(chosen) = body.answer.take() {
        st.chosen = Some(chosen);
        st.close();
        // **On the way out the owner refocuses itself**: the keyboard is on an id minted
        // inside a body that will not run again, so left alone the vanish rule picks a neighbour.
        cx.focus(id);
    } else if dismissed {
        st.close();
        cx.focus(id);
    }

    let resp = cx.interact(id, area, opts.interest);
    let faces = crate::state::Faces::default();
    let role = crate::state::press_into(ink, cx, area, &resp, &faces);
    let paint = cx.theme().paint(role);
    let chevron = cx.theme().glyph(if st.open {
        Glyph::ArrowDown
    } else {
        Glyph::ArrowRight
    });
    let label = st
        .chosen
        .and_then(|chosen| files.iter().find(|f| f.id == chosen))
        .map_or("", |f| f.name);
    // **The row is a partition of its width**, and the label goes through
    // `crate::glyphs::elided_row_into` — the one place the pad and the marker meet. Written by hand
    // here it would be `crate::input::select`'s drawing transcribed, which is what it was before
    // an earlier pass's review: `elide` reserves the marker's cell, so padding to the whole width and
    // then writing the marker over the pad's last cell writes that cell twice, on every truncated
    // widget, invisible to every counter but the pair.
    let head = ink.text(cx, area.x, area.y, chevron, paint);
    let _ = ink.text(cx, area.x + i32::from(head), area.y, " ", paint);
    let used = head + 1;
    let _ = crate::glyphs::elided_row_into(
        ink,
        cx,
        area.x + i32::from(used),
        area.y,
        label,
        area.w.saturating_sub(used),
        paint,
    );
    // **And the rows below it, for `crate::input::select`'s reason and by the same three lines** —
    // the face is the rectangle, the hover award covers all of it, and a `Response` has no field a
    // remainder could be named in. An earlier pass; the two overlay owners were the only two rows of
    // the freeze that did not write a rectangle taller than their content.
    crate::text::pad_rows(
        ink,
        cx,
        Rect::new(area.x, area.y + 1, area.w, area.h.saturating_sub(1)),
        paint,
    );

    if resp.clicked {
        cx.focus(id);
        if st.open {
            st.close();
        } else {
            st.open();
        }
    }

    // **The owner's keys, in one drain loop, and only while it is shut** — `crate::input::select`'s
    // arrangement, and its reason: `Ctx::decline` ends the level's turn at the key queue.
    while let Some(k) = cx.next_key(id) {
        if k.kind == Edge::Release {
            continue;
        }
        // **A chord belongs to the application** — `crate::input::select`'s guard, and components
        // an earlier pass found both loops missing it together, which is what *one drawing and one drain
        // loop apart* costs when only the drawing was shared.
        if crate::keys::is_chord(&k) {
            cx.decline(k);
            break;
        }
        match k.code {
            Code::Enter | Code::Char(' ') | Code::Down => st.open(),
            Code::Escape if st.open => st.close(),
            _ => {
                cx.decline(k);
                break;
            }
        }
    }

    if st.open && body.inside {
        st.seated = true;
    }
    if st.open && st.seated && !body.inside && !body.over {
        st.close();
    }

    // **The request is last, and it has to be**: `body`, `files` and `task` all move into the
    // closure here, and every read above is one the borrow checker has already allowed.
    if st.open {
        let room = cx.size();
        let list_w = opts.list_w.min(room.0);
        let pane_opts = opts.pane;
        let size = (room.0, room.1);
        cx.overlay(
            id,
            area,
            vitui_runtime::overlay::OverlayOpts::sized(size.0, size.1),
            move |cx| {
                picker_body(
                    &mut Direct,
                    cx,
                    id,
                    body,
                    files,
                    task,
                    decode,
                    line,
                    list_w,
                    &pane_opts,
                    shape,
                )
            },
        );
    }

    resp
}

/// **The picker's body: the shell, then a collection beside a preview pane.**
///
/// The three pieces, in order: the landing is taken at the top of the body, the
/// listing runs, and the pane asks about whatever the cursor is now on.
#[expect(
    clippy::too_many_arguments,
    reason = "the body's own four plus the two function pointers the owner cannot close over, the \
              list's width and the pane's options. Every one of them is a thing the body may not \
              capture by reference, which is `'f`'s whole cost — beside the `Ink` seam's \
              writer and the one value carrying every refused spelling"
)]
pub(crate) fn picker_body<'f, I: Ink, T>(
    ink: &mut I,
    cx: &mut Ctx<'f, '_>,
    owner: Id,
    body: &mut PickerBody<T>,
    files: &[Entry<'_>],
    task: &Task<T>,
    decode: fn(u64, &Cancel) -> T,
    line: fn(&mut Ctx<'_, '_>, Rect, &T, u32),
    list_w: u16,
    pane_opts: &PaneOpts,
    shape: PickerShape,
) where
    T: Preview + Send + 'static,
{
    // **The landing, at the top of the view.** The body *is* the view here, and the two readers it
    // has to keep in step are the listing's own label and the pane's document.
    let _landed = body.pane.land(task);

    let area = cx.area();
    let rows = u32::try_from(files.len()).unwrap_or(u32::MAX);
    let mut answer = None;
    let mut cancelled = false;
    let shell_id = cx.id();
    // **The shell goes through the seam and not through `Direct`** — `crate::input::popup_body`'s
    // one a later pass change, and its reason: everything a popup drew was invisible to a
    // [`Pen`](crate::runner::Pen), so a scene about the picker's list had no picture to compare.
    // What the seam does not reach is a `Ctx::overlay` body, which is unchanged; a `Pen` reaches
    // this function only when a caller invokes it **in the base pass**.
    let shell = overlay_into(
        ink,
        cx,
        shell_id,
        area,
        rows,
        &ShellOpts {
            kind: Kind::Popup,
            ..ShellOpts::default()
        },
        |_ink: &mut I, _cx: &mut Ctx<'_, '_>, _interior: Rect| {},
    );
    let interior = shell.interior;
    // **What the body reports and the owner reads a frame later.** `Response::local` and not
    // `Response::hovered`: `hovered` is a *previous*-frame guess and reads false on the frame the
    // layer is placed, which is the frame the optimistic focus arrives on — so a popup dismisses
    // itself out from under a pointer standing on it. The finding, one family over.
    body.over = shell.response.local.is_some();

    let list_w = list_w.min(interior.w);
    let list = Rect::new(interior.x, interior.y, list_w, interior.h);
    let pane = Rect::new(
        interior.x + i32::from(list_w),
        interior.y,
        interior.w - list_w,
        interior.h,
    );

    let coll = CollOpts {
        mode: Mode::Single,
        ..CollOpts::default()
    };
    // **First refusal, inside the one drain loop** — `crate::input::popup_body`'s arrangement and
    // its reason: `Ctx::decline` hands a key back *and ends the level's turn at the queue*, so a
    // body that read `Enter` before the collection would leave it nothing and one that read it
    // after would find the queue closed. `Enter` and `Esc` are the body's two; `crate::nav::step`
    // and the type-ahead own the rest.
    //
    // **It is this function's own two decisions and not `select`'s transcribed** (architecture
    // the keyboard question, which refused a second transcription by name). `Enter` answers **the cursor's file
    // id**, which is the same expression a press already answers with, so one meaning has two
    // triggers rather than two meanings one each. `Esc` **cancels** and answers nothing, because a
    // picker's `chosen` is an `Option` that starts empty and *answer what was already there* would
    // be indistinguishable from *nothing decided yet* — see [`PickerBody::cancelled`].
    let seated = shape.keyboard == Keyboard::Seated;
    let mut mine = |k: &Pressed, cursor: usize| {
        if !seated {
            return false;
        }
        // **A chord belongs to the application**, here as much as at the owner, and
        // an earlier pass found both of this family's loops missing the guard together.
        if crate::keys::is_chord(k) {
            return false;
        }
        match k.code {
            Code::Enter => {
                answer = files.get(cursor).map(|f| f.id);
                true
            }
            Code::Escape => {
                cancelled = true;
                true
            }
            _ => false,
        }
    };
    let list_resp = collection_shaped(
        ink,
        cx,
        list,
        &mut body.list,
        &coll,
        Rows::of(files.len()),
        |buf, range: std::ops::Range<usize>| {
            range.into_iter().find(|&i| files[i].name.starts_with(buf))
        },
        |ink: &mut I, cx: &mut Ctx<'_, '_>, r: Rect, i: usize, face: Face| {
            let paint = face_paint(cx.theme(), face);
            let _ = ink.pad_to(cx, r.x, r.y, files[i].name, r.w, paint);
        },
        &mut mine,
        shape.list,
    );
    // **The body takes the keyboard from its owner, exactly once**, and the id it goes to is the
    // **list** — the first decision. The pane is not a candidate: its document
    // is a function of the cursor, so it has nothing of its own to answer, and `body.inside` is
    // already defined as *the list has the focus*, which the owner's blur clause reads to decide
    // whether the popup is still wanted. Seating anywhere else would leave `inside` false while the
    // body held the keyboard and shut the popup under the user on the frame after.
    //
    // `if cx.is_focused(owner)` and never `if !cx.is_focused(list)`: the second drags the keyboard
    // back every frame the user has tabbed away, which is the refused spelling.
    if seated && cx.is_focused(owner) {
        cx.focus(list_resp.id);
    }
    body.inside = cx.is_focused(list_resp.id);
    // **The press edge and not `Response::clicked`**, which is `crate::input::popup_body`'s
    // spelling and its stated reason: a row selects on the *press*, so by the time a click has
    // completed the collection has already moved its cursor and a release-driven choice arrives a
    // frame late.
    //
    // This body read `clicked` until the picker got a keyboard, and it is the **pointer** half of that
    // issue's own sentence — *the family's two owners are one drawing and not one keyboard*. The
    // measurement is O4's sweep: `select` answers `Shift+Click` and `Ctrl+Shift+Click` and this
    // component answered neither, because a drive that ends at the press never reaches a
    // release-driven reader at all. Two spellings of the twenty-seven, found by declaring the rest.
    if list_resp.press_began {
        answer = files.get(body.list.sel.lead).map(|f| f.id);
    }

    // **The pane's question is the cursor's *file*.** Every natural way to name it names the
    // cursor's position, and one re-sort makes the two different questions for ever.
    let cursor = body.list.sel.lead.min(files.len().saturating_sub(1));
    let key = files.get(cursor).map_or(0, |f| f.id);
    let _ = file_preview_pane_with(
        cx,
        pane,
        &mut body.pane,
        task,
        Question::new(key, move |cancel: &Cancel| decode(key, cancel)),
        pane_opts,
        &mut |cx: &mut Ctx<'_, '_>, row: Rect, doc: &T, i: u32| line(cx, row, doc, i),
    );

    body.answer = answer;
    body.cancelled = cancelled;
    // **A body that answers through the inbox owes the frame that delivers it.** The owner reads
    // both slots at the top of the *next* frame and an application parks on `Driver::wait`, so
    // without a wake the choice lands on whatever input happens next — which for the keystroke that
    // made it means **never**. `crate::input::popup_body`'s finding, and this body did not have it:
    // the pointer path was latent because a pointer usually moves again, and `Enter` would have
    // made it certain.
    //
    // Conditional, and that is the whole of why it is not a wake loop: a frame with nothing in
    // either slot asks for nothing, and the frame that delivers empties them.
    if answer.is_some() || cancelled {
        cx.request_frame();
    }
}

// ── the auto traits, as a pair ───────────────────────────────────────────────────────────────────

/// **G10b: what `Cell` and `RefCell` remove is `Sync`, and a `Task` is the only thing here that is
/// neither.**
///
/// # G10 named the wrong auto trait, and the correction is itself half wrong
///
/// The corrected statement is *`Task` and `Worker` are `!Sync`; `Cell`, `RefCell`, `Worker` **are**
/// `Send`*, and the half about `Cell` is the finding: the property the design needs is **`!Sync`**,
/// because the app thread owns the staleness arithmetic and a job may never hold a *reference* to
/// the task that spawned it. `Send` is the wrong question in both directions — `Cell` and `RefCell`
/// are `Send`, and what makes a [`Task`] `!Send` is an `Arc<dyn Spawner>` held for an unrelated
/// reason, with no `Send` bound on `Spawner` at all. Add one and the case flips with nothing about
/// the design having moved.
///
/// **The other half does not reproduce, and `crate::gates`'s row 31 already says so**: a
/// [`Worker`](vitui_runtime::work::Worker) is an `Arc<Inbox>` and is `Sync` as well as `Send`. That
/// is asserted here rather than worked around, and it costs the design nothing — what a component
/// may not share is its own `Task`, which is where the whole staleness arithmetic lives. Row 31
/// carries the `Task` half and this type carries the half the correction added:
/// the `Cell`/`RefCell` sentence, spelled as a compiling case.
///
/// # The pair, and why both halves are needed
///
/// A lone `compile_fail` also passes when the item has been renamed — `E0433` instead of `E0277`,
/// and the mechanism cannot tell those apart — so each hostile case has a compiling twin naming the
/// same items by path. **Protects:** [`vitui_runtime::work::Task`],
/// [`vitui_runtime::work::Worker`].
///
/// A `Task` is not `Sync`, which is the property a pane's state rests on:
///
/// ```compile_fail,E0277
/// use vitui_runtime::work::{Task, Worker};
///
/// fn needs_sync<T: Sync>(_: &T) {}
/// let worker = Worker::queueing();
/// let task: Task<u32> = Task::new(&worker);
/// needs_sync(&task);
/// ```
///
/// and it is not `Send` either, for a reason that is not the one the design rests on:
///
/// ```compile_fail,E0277
/// use vitui_runtime::work::{Task, Worker};
///
/// fn needs_send<T: Send>(_: T) {}
/// let worker = Worker::queueing();
/// let task: Task<u32> = Task::new(&worker);
/// needs_send(task);
/// ```
///
/// **G10b**, the compiling twin, naming all four by path and asserting what is *not* removed:
///
/// ```
/// use std::cell::{Cell, RefCell};
/// use vitui_runtime::work::{Task, Worker};
///
/// fn needs_send<T: Send>(_: &T) {}
/// fn needs_sync<T: Sync>(_: &T) {}
///
/// // What `Cell` and `RefCell` remove is `Sync`, and they are what the
/// // staleness arithmetic is written in.
/// needs_send(&Cell::new(0u64));
/// needs_send(&RefCell::new(0u64));
/// assert_eq!(Cell::new(7u64).get(), 7);
/// assert_eq!(*RefCell::new(7u64).borrow(), 7);
///
/// // A worker crosses to the thread it hires — and it is `Sync` too, which the original denied.
/// let worker = Worker::queueing();
/// needs_send(&worker);
/// needs_sync(&worker);
///
/// // And the type that is neither exists and is spelled the way the hostile halves spell it.
/// let task: Task<u32> = Task::new(&worker);
/// assert_eq!(task.asking(), None);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct WhyTheAutoTraitIsSync;

// ── the refused spellings ────────────────────────────────────────────────────────────────────────

/// **What is refused, kept runnable so that each refusal is a number rather than a sentence.**
///
/// Every arm here is the shipped pane with one field of [`PaneShape`] changed, except the per-file
/// map — which is a *structure* the shipped pane has nowhere to put, and that is its refusal.
pub mod defective {
    use super::{Bump, Extent, Landed, PaneShape, PaneState, Preview, Reset, Taken};
    use std::collections::HashMap;
    use vitui_runtime::work::Task;

    /// A pane whose offset is left where it was. **The new file opens at its last page.**
    #[must_use]
    pub fn clamped_and_kept<T>() -> PaneState<T> {
        PaneState::shaped(PaneShape {
            reset: Reset::Never,
            ..PaneShape::default()
        })
    }

    /// A pane whose offset is zeroed at the request. **Scrolls the previous file, still on screen,
    /// to its top** for the length of the decode.
    #[must_use]
    pub fn reset_on_the_request<T>() -> PaneState<T> {
        PaneState::shaped(PaneShape {
            reset: Reset::OnTheRequest,
            ..PaneShape::default()
        })
    }

    /// A pane that declares no bound at all. **The body draws nothing at all**, which is
    /// *unclamped* row with its cause named.
    ///
    /// **Two fields, because the row is two clauses**: *the offset is unclamped* is *left where it
    /// was* **and** *nothing clamped it*. [`clamped_and_kept`] is this arm with the second removed
    /// and the first kept, so the pair is the isolation — one field between them, and the field is
    /// the extent.
    #[must_use]
    pub fn unclamped<T>() -> PaneState<T> {
        PaneState::shaped(PaneShape {
            reset: Reset::Never,
            extent: Extent::Unbounded,
            ..PaneShape::default()
        })
    }

    /// A pane whose revision advances on every frame. **119 computations against 20 landings**, on
    /// two screens that are cell-identical.
    #[must_use]
    pub fn bumping_every_frame<T>() -> PaneState<T> {
        PaneState::shaped(PaneShape {
            bump: Bump::EveryFrame,
            ..PaneShape::default()
        })
    }

    /// A pane that takes its own landing mid-draw. **20 torn frames of 20**, with no thread having
    /// done anything.
    #[must_use]
    pub fn taken_inside_the_draw<T>() -> PaneState<T> {
        PaneState::shaped(PaneShape {
            taken: Taken::InsideTheDraw,
            ..PaneShape::default()
        })
    }

    /// **A picker whose list asks to be brought into view on every frame.** The `wheeled` axis on
    /// this component, and the arm `CONTEXT.md` forbids by name.
    ///
    /// `Reveal::EveryFrame` on the collection inside the popup. The wheel is then dead: a notch
    /// moves the offset and the pull puts it back before anything draws, on a screen that is
    /// identical while it happens. [`a_list_that_never_reveals`] is the other arm, and it is not a
    /// fix — see `crate::dropped::wheeled`, which reports all three.
    #[track_caller]
    #[expect(
        clippy::too_many_arguments,
        reason = "[`a_stale_list_tail`]'s eleven, unchanged"
    )]
    pub fn a_list_revealing_every_frame<'f, I, T>(
        ink: &mut I,
        cx: &mut vitui_runtime::Ctx<'f, '_>,
        id: vitui_runtime::Id,
        area: vitui_runtime::Rect,
        st: &mut super::PickerState,
        body: &'f mut super::PickerBody<T>,
        files: &'f [super::Entry<'f>],
        task: &'f Task<T>,
        decode: fn(u64, &vitui_runtime::work::Cancel) -> T,
        line: fn(&mut vitui_runtime::Ctx<'_, '_>, vitui_runtime::Rect, &T, u32),
        opts: &super::PickerOpts,
    ) -> vitui_runtime::Response
    where
        I: crate::ink::Ink,
        T: Preview + Send + 'static,
    {
        super::file_picker_shaped(
            ink,
            cx,
            id,
            area,
            st,
            body,
            files,
            task,
            decode,
            line,
            opts,
            super::PickerShape {
                list: crate::collect::CollShape {
                    reveal: crate::collect::Reveal::EveryFrame,
                    ..crate::collect::CollShape::RULE
                },
                ..super::PickerShape::default()
            },
        )
    }

    /// **A picker whose list never asks to be brought into view.** The way to pass a wheel gate
    /// written in one direction, and it is not a fix: the keyboard cursor can no longer bring
    /// anything into view.
    ///
    /// [`a_list_revealing_every_frame`]'s third arm, and it exists for `crate::wheel`'s reason —
    /// a gate written on the wheel alone goes green the moment somebody deletes the call.
    #[track_caller]
    #[expect(
        clippy::too_many_arguments,
        reason = "[`a_stale_list_tail`]'s eleven, unchanged"
    )]
    pub fn a_list_that_never_reveals<'f, I, T>(
        ink: &mut I,
        cx: &mut vitui_runtime::Ctx<'f, '_>,
        id: vitui_runtime::Id,
        area: vitui_runtime::Rect,
        st: &mut super::PickerState,
        body: &'f mut super::PickerBody<T>,
        files: &'f [super::Entry<'f>],
        task: &'f Task<T>,
        decode: fn(u64, &vitui_runtime::work::Cancel) -> T,
        line: fn(&mut vitui_runtime::Ctx<'_, '_>, vitui_runtime::Rect, &T, u32),
        opts: &super::PickerOpts,
    ) -> vitui_runtime::Response
    where
        I: crate::ink::Ink,
        T: Preview + Send + 'static,
    {
        super::file_picker_shaped(
            ink,
            cx,
            id,
            area,
            st,
            body,
            files,
            task,
            decode,
            line,
            opts,
            super::PickerShape {
                list: crate::collect::CollShape {
                    reveal: crate::collect::Reveal::Never,
                    ..crate::collect::CollShape::RULE
                },
                ..super::PickerShape::default()
            },
        )
    }

    /// **A picker whose popup has no keyboard at all**, which is what
    /// shipped for eight tickets.
    ///
    /// `Keyboard::Unseated`: the body seats no focus and declares no
    /// `collect::Refusal`, so an open picker answers a pointer and nothing else.
    /// No arrows, no `Home`/`End`, no type-ahead, and no way to choose a file from the keyboard.
    ///
    /// **It renders perfectly**, which is the whole reason nothing caught it: every other gate here
    /// and in [`crate::preview`] drives the picker with a pointer or asserts about the pane, and a
    /// keyboard that is never pressed leaves no mark on a canvas. So the instrument that separates
    /// the two arms is not a screen — see
    /// `crate::dropped::tests::an_open_picker_answers_the_keyboard_and_the_arm_that_shipped_answered_none_of_it`.
    #[track_caller]
    #[expect(
        clippy::too_many_arguments,
        reason = "[`a_stale_list_tail`]'s eleven, unchanged"
    )]
    pub fn a_popup_with_no_keyboard<'f, I, T>(
        ink: &mut I,
        cx: &mut vitui_runtime::Ctx<'f, '_>,
        id: vitui_runtime::Id,
        area: vitui_runtime::Rect,
        st: &mut super::PickerState,
        body: &'f mut super::PickerBody<T>,
        files: &'f [super::Entry<'f>],
        task: &'f Task<T>,
        decode: fn(u64, &vitui_runtime::work::Cancel) -> T,
        line: fn(&mut vitui_runtime::Ctx<'_, '_>, vitui_runtime::Rect, &T, u32),
        opts: &super::PickerOpts,
    ) -> vitui_runtime::Response
    where
        I: crate::ink::Ink,
        T: Preview + Send + 'static,
    {
        super::file_picker_shaped(
            ink,
            cx,
            id,
            area,
            st,
            body,
            files,
            task,
            decode,
            line,
            opts,
            super::PickerShape {
                keyboard: super::Keyboard::Unseated,
                ..super::PickerShape::default()
            },
        )
    }

    /// **The fifth offset spelling: a slot per file.**
    ///
    /// Right in both directions, and refused because **nothing releases it**. It is a wrapper rather
    /// than a field of [`PaneState`] precisely because the shipped pane has nowhere to put one: a
    /// map keyed on a file is a map R02's sweep cannot reach, since the sweep releases what an `Id`
    /// stopped drawing and a file is not an `Id`.
    ///
    /// See [`crate::preview::map_bytes`] for what it costs at a million entries and
    /// [`crate::preview::measured_map_bytes`] for what the shipped `HashMap` really takes, which is
    /// worse.
    pub struct Mapped<T> {
        /// The pane, spelled to leave the offset alone: this wrapper puts it where the map says.
        pub pane: PaneState<T>,
        /// The offset each file was last left at.
        remembered: HashMap<u64, u32>,
    }

    impl<T> Default for Mapped<T> {
        fn default() -> Mapped<T> {
            Mapped::new()
        }
    }

    impl<T> Mapped<T> {
        /// An empty map over a pane that does not reset.
        #[must_use]
        pub fn new() -> Mapped<T> {
            Mapped {
                pane: PaneState::shaped(PaneShape {
                    reset: Reset::Never,
                    ..PaneShape::default()
                }),
                remembered: HashMap::new(),
            }
        }

        /// How many files the map is holding an offset for. **Never fewer than it held before**,
        /// which is the refusal.
        pub fn entries(&self) -> usize {
            self.remembered.len()
        }
    }

    impl<T: Preview> Mapped<T> {
        /// File the offset the pane is about to leave, before the question changes.
        ///
        /// Called by the caller and not by the pane, because there is no frame on which the pane
        /// knows a question is *about to* change: [`Task::request`] tells it afterwards.
        pub fn filing(&mut self, leaving: u64) {
            self.remembered.insert(leaving, self.pane.offset());
        }

        /// Take the landing, and put the offset where the map says.
        pub fn land(&mut self, task: &Task<T>) -> Landed {
            let landed = self.pane.land(task);
            if landed == Landed::Shown
                && let Some(doc) = self.pane.showing()
            {
                let row = self.remembered.get(&doc.shows()).copied().unwrap_or(0);
                self.pane.scroll_to(row);
            }
            landed
        }
    }
}
