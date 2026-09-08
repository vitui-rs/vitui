//! Async work: the handoff between a background thread and the frame.
//!
//! Six names — [`Slot`] (the engine's, re-exported), [`Drain`], [`Task`], [`Worker`],
//! [`Landing`] and [`Cancel`] — and the whole module is `std::sync`, `std::cell` and the engine's
//! two handoff types. There is no executor here and no future: **the runtime takes no dependency
//! beyond the engine**, so a worker is a *noun* rather than a spawned task on somebody's runtime.
//!
//! # The engine's primitive is right and the obvious noun for it is wrong
//!
//! [`Slot`] keeps the newest **landing** and nothing anywhere keeps the newest **question**. Twenty
//! rows of arrow key down a directory whose files get smaller end with the preview pane on **file
//! 0** while the selection is on **19** — **19 of 20** frames wrong — and it stays wrong for as long
//! as you care to run it, because there is no further post and therefore no further frame. The
//! asymmetry that makes the engine's handoff cheap (*the worker posts, the app thread takes when it
//! gets round to it*) is exactly what removes the correcting mechanism a polling loop would have had
//! by accident.
//!
//! What closes it is **8 bytes**: one [`Generation`] minted on the app thread at request time and
//! carried back beside the answer on a [`Landing`]. `tests::the_bare_slot_shows_the_wrong_file_on_nineteen_of_twenty_frames`
//! is the defect and `tests::the_generation_shows_the_selected_file_on_twenty_of_twenty_frames` is
//! the twin.
//!
//! # It cannot be a `Revision`, and the constraint runs both ways
//!
//! [`crate::data::Memo`]'s hit test is `self.rev == rev`. A [`Revision`](crate::data::Revision)
//! answers *is this different* and has never claimed to answer *is this newer*, so a revision
//! stamped by whichever worker finished last is accepted exactly as readily as one stamped by the
//! newest. The two types do not convert in either direction and there is a compile-outcome pair on
//! [`Generation`] that says so.
//!
//! Running the other way: `data`'s counter is safe to touch from a worker **only** because nothing
//! anywhere compares two revisions with an ordering operator. The first `<` between two revisions
//! turns a worker landing into a silent wrong answer, and `data`'s own `E0369` case is what holds
//! it.
//!
//! # A pending job is not a fifth cross-frame fact
//!
//! The registry that would make it one has **no correct setting**: join [`crate::id`]'s sweep and a
//! pane that misses one frame restarts its decode; skip it and the entry outlives the widget for
//! ever. The grab, the press origin and the focus are *interaction* state, meaningless the moment
//! the widget is not there to interact with. A decode is not — it is **commissioned work whose
//! lifetime is the question's**.
//!
//! So the slot lives in application state, exactly as a `ListState` and a
//! [`Versioned`](crate::data::Versioned) do, and a component holds `&Task<T>`. This module names no
//! [`Id`](crate::Id) anywhere, which is the structural half of the same statement and is asserted by
//! `tests::the_module_names_no_identity_so_there_is_no_path_to_a_registry`.
//!
//! # Where the take happens
//!
//! **A landing is a write to application data**, so it belongs at the top of the view, before
//! anything reads. Taken inside the pane's own draw — which is where a component author puts it,
//! because it is where the value is used — the status line above the pane and the footer below it
//! disagree on **every landing frame**: 1 torn frame per landing against 0.
//!
//! Nothing enforces the ordering, and that is named rather than solved. A type cannot express it
//! here: the runtime does not know the application's tasks, and a `take` that took the
//! [`Frame`](crate::ctx::Frame) in order to check would stop being *nothing on the app thread but a
//! take*. It is the same residue `Ctx::child` carries.
//!
//! # Requirement 11 in the case it did not mention
//!
//! Sixty frames with a job in flight ask for **0** wakeups against **60** for the polling shape,
//! whose streak of sixty fires the runaway detector on a screen doing nothing. The park is the whole
//! mechanism: the app thread blocks on the same condvar [`Wake::Posted`]
//! is, and **a job in flight is not a reason to run a frame**. Twenty posts arriving while the app
//! thread is busy cost **one** wake.
//!
//! Nothing in this module names a deadline, and the plumbing posts only when something *lands* — a
//! job superseded while it ran posts nothing at all.

use std::cell::Cell;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};

/// Where a worker leaves a result for the app thread. **The engine's, re-exported unchanged.**
///
/// Three names here are the engine's and are not the runtime's; shipping a second type
/// with this name across the seam is the review finding `GlyphSet` already carries. `work` wraps it
/// — in [`Task`] — and adds [`Drain`] beside it. It adds nothing *to* it.
pub use vitui_engine::Slot;

/// Why the app thread woke, and the handle that wakes it. **The engine's, re-exported unchanged**,
/// for [`Slot`]'s reason and one more.
///
/// [`Worker::hire`] takes a `WakeHandle` and [`Driver::wait`](crate::ctx::Driver::wait) returns a
/// `Wake`, so **both halves of the handoff are already on this crate's public surface** — a caller
/// who cannot name them can hire no worker and write no loop. Before `Driver::wait` existed neither
/// was reachable and neither had to be: `Driver::attach` dropped the handle it was given and there
/// was nothing above the engine that could park. Who owns the loop is still open, and an application
/// that owns it needs the two nouns the parking is written in.
pub use vitui_engine::{Wake, WakeHandle};

/// Minted on the app thread, once per question, and never reused.
///
/// **Process-global**, like [`Revision::fresh`](crate::data::Revision::fresh) and for a stronger
/// reason: two [`Task`]s' generations are comparable, so a landing left over from a task that has
/// been replaced can never look fresh to the one that replaced it.
static NEXT_GENERATION: AtomicU64 = AtomicU64::new(1);

/// The eight bytes that close the staleness defect: which question an answer is an answer to.
///
/// # It is not a [`Revision`](crate::data::Revision), in either direction
///
/// A revision answers *is this different*; a generation answers *is this newer*. The types do not
/// convert, and a `u64` does not become either of them by accident:
///
/// ```compile_fail,E0308
/// use vitui_runtime::{Revision, work::{Task, Worker}};
///
/// let worker = Worker::queueing();
/// let task: Task<u32> = Task::new(&worker);
/// task.request(1, |_| 7);
/// let _: Revision = task.generation().expect("a question was asked");
/// ```
///
/// ```compile_fail,E0308
/// use vitui_runtime::{Revision, work::Generation};
///
/// let _: Generation = Revision::fresh();
/// ```
///
/// ```compile_fail,E0277
/// use vitui_runtime::{Revision, work::Generation};
///
/// let _ = Generation::from(Revision::fresh());
/// ```
///
/// and the twins, naming both doors by path — the numbers are comparable, the types are not:
///
/// ```
/// use vitui_runtime::data::Revision;
/// use vitui_runtime::work::{Generation, Task, Worker};
///
/// let worker = Worker::queueing();
/// let task: Task<u32> = Task::new(&worker);
/// task.request(1, |_| 7);
/// let g: Generation = task.generation().expect("a question was asked");
/// let r: Revision = Revision::from_raw(Generation::raw(g));
/// assert_eq!(Generation::raw(g), Revision::raw(r));
/// ```
///
/// # It is ordered, and that is what a revision may never be
///
/// Two jobs started on two threads can land in either order, and [`Slot::put`] is last-*arrival*
/// wins. `land` compares the two generations with `>` and keeps the newer, which is the one place
/// in this crate where an ordering between two stamps is taken — and the reason `data`'s `E0369`
/// case is load bearing rather than tidy.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Generation(u64);

impl Generation {
    /// A generation nothing else has or will have. **App thread only** — it is minted by
    /// [`Task::request`] and by nothing else.
    fn mint() -> Generation {
        Generation(NEXT_GENERATION.fetch_add(1, Ordering::Relaxed))
    }

    /// The number, for a report or a diagnostic overlay.
    ///
    /// There is deliberately no way back: a `Generation` a caller built is a generation no question
    /// was asked under, and every landing carrying one would be accepted for ever or never.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// An answer arriving from a worker: the payload, and the question it answers.
///
/// **Eight bytes over the payload**, which is the whole of what the staleness fix costs on the wire.
/// A landing whose generation is not the newest is an answer to a question nobody is asking, and
/// [`Task::take`] drops it.
#[derive(Debug)]
pub struct Landing<T> {
    generation: Generation,
    value: T,
}

impl<T> Landing<T> {
    /// Pair a payload with the question it answers.
    ///
    /// Public because an application writing its own worker against a bare
    /// [`Slot<Landing<T>>`](Slot) needs it, and narrow because [`Generation`] has no constructor: the
    /// only generations in existence are ones [`Task::request`] minted.
    pub fn new(generation: Generation, value: T) -> Landing<T> {
        Landing { generation, value }
    }

    /// Which question this answers.
    #[must_use]
    pub fn generation(&self) -> Generation {
        self.generation
    }

    /// The payload, moved out.
    pub fn into_value(self) -> T {
        self.value
    }
}

/// A flag a job agrees to poll. **There is no other kind.**
///
/// A `std::thread` cannot be killed, so "cancel" means the job looks. Both ends of that are counted
/// rather than argued: a superseded job of 210 units that polls on every unit does **1 of 210**; the
/// decode an application writes first does not poll at all and does **210 of 210**.
///
/// # It saves nothing while the decode is faster than the user
///
/// On real threads this is a crossover rather than a number. Twenty selections against a decode of
/// about 5 ms: **1 ms apart** costs 5.3–7.2× the necessary work with the flag polled against 20.0×
/// with it ignored; **10 ms and 30 ms apart** cost **20.0× in every column**, cancellation included.
/// At a 30 ms key repeat every job has finished before it is superseded, so there is nothing to
/// cancel and nothing to replace — the twentyfold waste is intrinsic and the only cure is not to
/// ask. What [`Worker`] saves at 1 ms it saves *in the inbox*, before the job exists. The two
/// mechanisms win in opposite conditions and neither subsumes the other.
///
/// `examples/work_numbers.rs` is where the crossover is measured.
#[derive(Debug, Default)]
pub struct Cancel {
    flag: AtomicBool,
}

impl Cancel {
    /// An open bracket.
    const fn new() -> Cancel {
        Cancel {
            flag: AtomicBool::new(false),
        }
    }

    /// Whether the question this job is answering has been superseded.
    ///
    /// `Relaxed`, and it may be: nothing is published through this flag. The value it guards travels
    /// in the [`Slot`], which has a lock around it, and a job that reads the flag one unit late has
    /// done one unit too much rather than something wrong.
    #[must_use]
    pub fn cancelled(&self) -> bool {
        self.flag.load(Ordering::Relaxed)
    }

    /// Close the bracket. Called when a [`Task`] is asked a different question.
    fn cancel(&self) {
        self.flag.store(true, Ordering::Relaxed);
    }
}

/// What [`Task::request`] did with a question.
///
/// Part of the signature rather than an internal counter, because a component branches on it — a
/// spinner starts on [`Requested::Asked`] and keeps spinning on [`Requested::Deduped`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Requested {
    /// A question nobody was asking: a generation was minted and the job went to the worker.
    Asked,
    /// The same question as last time. **Nothing was minted and nothing was posted.**
    Deduped,
}

/// One assignment as it crosses the thread boundary. Answers whether it landed.
type Assignment = Box<dyn FnOnce() -> bool + Send + 'static>;

/// What a worker is holding.
enum Held {
    /// **One slot: the newest supersedes.** The shipped shape.
    Newest(Option<Assignment>),
    /// Every question, in the order it was asked, each taken at most once.
    ///
    /// The deterministic arm — see [`Worker::queueing`]. A `Vec<Option<_>>` rather than a queue
    /// because the caller names a question by the order it was asked in, and that index may not
    /// shift under it.
    Every(Vec<Option<Assignment>>),
}

/// A worker's inbox, shared with every [`Task`] that asks it questions.
struct Inbox {
    held: Mutex<Held>,
    /// What the resident thread parks on. **The app thread never waits here.**
    ready: Condvar,
    closed: AtomicBool,
    asked: AtomicU64,
    started: AtomicU64,
    replaced: AtomicU64,
}

impl Inbox {
    fn new(held: Held) -> Inbox {
        Inbox {
            held: Mutex::new(held),
            ready: Condvar::new(),
            closed: AtomicBool::new(false),
            asked: AtomicU64::new(0),
            started: AtomicU64::new(0),
            replaced: AtomicU64::new(0),
        }
    }

    /// The engine's [`Slot`] gives the reason and it is the same one here: the invariant a poisoned
    /// mutex protects is *some producer left this structure inconsistent*, and there is no
    /// inconsistent state reachable — every verb replaces one `Option` whole.
    fn lock(&self) -> MutexGuard<'_, Held> {
        self.held.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Put a question in. **Never blocks on anything but the move.**
    fn put(&self, assignment: Assignment) {
        self.asked.fetch_add(1, Ordering::Relaxed);
        if self.closed.load(Ordering::Acquire) {
            // The worker is gone. Dropping here rather than storing it keeps the question's captures
            // — which are the application's data — from outliving the thing that would have consumed
            // them. `take` answers `None` for ever, which is what it answers whenever nothing has
            // landed yet, so this is not a new state for a caller to handle.
            return;
        }
        let displaced = {
            let mut held = self.lock();
            match &mut *held {
                Held::Newest(slot) => slot.replace(assignment),
                Held::Every(queue) => {
                    queue.push(Some(assignment));
                    None
                }
            }
        };
        if displaced.is_some() {
            self.replaced.fetch_add(1, Ordering::Relaxed);
        }
        // **Dropped outside the lock.** A superseded question's captures are the application's, so
        // its `Drop` is arbitrary code — and running arbitrary code under a lock the app thread takes
        // is how a hold the length of a `mem::replace` becomes a hold of unbounded length.
        drop(displaced);
        self.ready.notify_one();
    }

    /// The resident thread's park. `None` means the worker has been dropped.
    fn next(&self) -> Option<Assignment> {
        let mut held = self.lock();
        loop {
            if self.closed.load(Ordering::Acquire) {
                return None;
            }
            if let Held::Newest(slot) = &mut *held
                && let Some(assignment) = slot.take()
            {
                return Some(assignment);
            }
            held = self
                .ready
                .wait(held)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }

    /// Take the question asked `index`-th, for the deterministic arm.
    fn queued(&self, index: usize) -> Option<Assignment> {
        let mut held = self.lock();
        match &mut *held {
            Held::Every(queue) => queue.get_mut(index).and_then(Option::take),
            Held::Newest(_) => None,
        }
    }
}

/// A background thread with a **one-slot inbox**, asked questions rather than handed functions.
///
/// # `spawn` is a verb over a function; a worker is a noun you send questions to
///
/// `std::thread::spawn` costs **9.63–12.74 µs**; putting a question in a resident worker's inbox
/// costs **73–83 ns** — **121–174×**. Over a twenty-key sweep that is 192–255 µs of thread creation,
/// two and a half frame budgets, spent on nineteen answers nobody will look at.
///
/// The inbox is one slot for the same reason the outbox is and the same reason the engine's frame
/// mailbox is: **the newest supersedes, because nobody wants the older one.** Three mailboxes, one
/// rule — and [`Drain`] is the one place the rule is wrong.
///
/// **A question replaced before anyone looks at it costs nothing at all**: no thread, no started
/// job, no cancel flag, not even the decision to stop. Twenty questions asked while the worker is
/// busy leave nineteen replaced and one decode performed, and the decode performed is the one still
/// being asked for.
///
/// ```
/// use vitui_runtime::work::{Requested, Task, Worker};
///
/// // The deterministic arm: no thread, and the caller says when each question is answered.
/// let worker = Worker::queueing();
/// let task: Task<u32> = Task::new(&worker);
///
/// assert_eq!(task.request(7, |_cancel| 41 + 1), Requested::Asked);
/// assert_eq!(task.request(7, |_cancel| 0), Requested::Deduped, "one question, one job");
/// assert_eq!(task.take(), None, "nothing has landed; nothing waits for it either");
///
/// worker.run(0);
/// assert_eq!(task.take(), Some(42));
/// assert_eq!(task.take(), None);
/// ```
pub struct Worker {
    inbox: Arc<Inbox>,
    /// `None` for [`Worker::queueing`], which has no thread to join.
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Worker {
    /// Hire a resident thread, and hand it the way back to the app thread.
    ///
    /// The [`WakeHandle`] is posted **once per landing** and never for a job that was superseded
    /// while it ran, which is what makes sixty frames with a job in flight cost zero wakeups.
    ///
    /// # Panics
    ///
    /// If the operating system will not give us a thread. That is `std::thread::spawn`'s own
    /// contract; the [`Builder`](std::thread::Builder) is here for the name, which is what a reader
    /// of `Activity Monitor` sees.
    #[must_use]
    pub fn hire(wake: WakeHandle) -> Worker {
        let inbox = Arc::new(Inbox::new(Held::Newest(None)));
        let mine = Arc::clone(&inbox);
        let thread = std::thread::Builder::new()
            .name(String::from("vitui-worker"))
            .spawn(move || {
                while let Some(assignment) = mine.next() {
                    let landed = assignment();
                    mine.started.fetch_add(1, Ordering::Relaxed);
                    if landed {
                        wake.post();
                    }
                }
            })
            .expect("the operating system refused a thread");
        Worker {
            inbox,
            thread: Some(thread),
        }
    }

    /// A worker with **no thread**, whose questions are answered when the caller says so.
    ///
    /// **Public API, and the same decision `Clock::Manual` is.** A gate over staleness arithmetic is
    /// a gate over an *order*, and the only thing a real thread contributes to it is that order —
    /// handing it to the caller turns a race into a count. It is what every behavioural gate in this
    /// module runs on, and it is what an application testing a component that asks for work needs
    /// for the same reason.
    ///
    /// It keeps **every** question rather than the newest, because a test of the staleness rule has
    /// to be able to start the jobs the resident worker would have replaced. [`Worker::run`] names
    /// one by the order it was asked in.
    #[must_use]
    pub fn queueing() -> Worker {
        Worker {
            inbox: Arc::new(Inbox::new(Held::Every(Vec::new()))),
            thread: None,
        }
    }

    /// Ask a question that answers nowhere in particular. **Newest supersedes.**
    ///
    /// The verb [`Task::request`] is built on, exposed because a job whose result is not a value —
    /// a write to disk, a log flush — has no landing and needs no [`Task`]. It mints no generation
    /// and deduplicates nothing: an application calling this every frame gets a job every frame.
    pub fn ask(&self, job: impl FnOnce() + Send + 'static) {
        self.inbox.put(Box::new(move || {
            job();
            false
        }));
    }

    /// Answer the question asked `index`-th, **on the calling thread**.
    ///
    /// Answers whether it landed — false for a question that was superseded while it ran, and for
    /// [`Worker::ask`], which lands nowhere.
    ///
    /// # Panics
    ///
    /// On a resident worker, which answers its own questions, and on an index that was never asked
    /// or has already been answered. Both are a test asserting against a schedule that did not
    /// happen, and an instrument that answered `false` to them would report the mechanism broken
    /// when it is the schedule that is.
    pub fn run(&self, index: usize) -> bool {
        assert!(
            self.thread.is_none(),
            "a resident worker answers its own questions"
        );
        let assignment = self
            .inbox
            .queued(index)
            .unwrap_or_else(|| panic!("question {index} was never asked, or has been answered"));
        self.inbox.started.fetch_add(1, Ordering::Relaxed);
        assignment()
    }

    /// How many questions this worker has been asked.
    #[must_use]
    pub fn asked(&self) -> u64 {
        self.inbox.asked.load(Ordering::Relaxed)
    }

    /// How many of them became a running job.
    ///
    /// **The gate's number.** A thousand frames on one selection are a thousand asks and one job;
    /// twenty questions asked while a resident worker is busy are twenty asks and two jobs.
    #[must_use]
    pub fn started(&self) -> u64 {
        self.inbox.started.load(Ordering::Relaxed)
    }

    /// How many were replaced in the inbox before anybody looked at them. **The free ones.**
    #[must_use]
    pub fn replaced(&self) -> u64 {
        self.inbox.replaced.load(Ordering::Relaxed)
    }
}

/// Closing the worker down **joins**, and a job in flight is waited for.
///
/// That is a block, and it is deliberately not the frame loop's: a worker is dropped when an
/// application is shutting down. The alternative — detaching — leaves a thread landing a value into
/// a slot whose owner is gone, and makes a worker that panicked invisible, which is exactly the
/// class of thing this crate does not do to its caller.
impl Drop for Worker {
    fn drop(&mut self) {
        self.inbox.closed.store(true, Ordering::Release);
        self.inbox.ready.notify_all();
        if let Some(thread) = self.thread.take() {
            let _joined = thread.join();
        }
    }
}

impl std::fmt::Debug for Worker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Worker")
            .field("resident", &self.thread.is_some())
            .field("asked", &self.asked())
            .field("started", &self.started())
            .field("replaced", &self.replaced())
            .finish()
    }
}

/// One question and the newest answer to it. **Lives in application state; a component holds
/// `&Task<T>`.**
///
/// # The verb is idempotent in the question, and that is not an optimisation
///
/// Immediate mode has no mount and therefore no *on selection change* hook, so a component can only
/// ask **every frame**. [`Task::request`] compares the key against the last question asked and does
/// nothing if they match: a thousand frames on one selection are **1000 asks and 1 job** against
/// 1000 jobs for the shape that spawns per call. The multiplier is the frame rate, which is the
/// worst possible constant for a defect — it grows with how well the rest of the runtime is doing.
///
/// **The dedupe is also what makes `&self` safe.** A component may not require ownership or `&mut`
/// of application data; without the key, a `&self` request verb is a job per frame with no `&mut`
/// anywhere to warn anybody.
///
/// # A worker cannot reach this half
///
/// `Cell` inside and a `!Send` brand on top: the only thing that crosses the thread boundary is the
/// `Arc<Slot<Landing<T>>>` the job holds, and it is private.
///
/// ```compile_fail,E0277
/// use vitui_runtime::work::{Task, Worker};
///
/// let worker = Worker::queueing();
/// let task: Task<u32> = Task::new(&worker);
/// std::thread::spawn(move || {
///     let _ = Task::take(&task);
/// });
/// ```
///
/// and the twin, naming the accessor by path on the thread that owns it:
///
/// ```
/// use vitui_runtime::work::{Task, Worker};
///
/// let worker = Worker::queueing();
/// let task: Task<u32> = Task::new(&worker);
/// assert_eq!(Task::take(&task), None);
/// ```
///
/// The outbox stays private for the same reason `Versioned` has no `DerefMut`: a public slot
/// restores the raw `Option<Landing<T>>` path and deletes the staleness fix.
///
/// ```compile_fail,E0609
/// use vitui_runtime::work::{Task, Worker};
///
/// let worker = Worker::queueing();
/// let task: Task<u32> = Task::new(&worker);
/// let _ = task.outbox;
/// ```
///
/// # There is no registry, and no way to build one from here
///
/// A [`Ctx`](crate::Ctx) hands out no task, because a pending job is not a cross-frame fact:
///
/// ```compile_fail,E0599
/// fn pane(cx: &mut vitui_runtime::Ctx<'_, '_>) {
///     let _: () = cx.task::<u32>();
/// }
/// ```
///
/// and the twin, which is where a task actually lives — in the application, passed down by
/// reference:
///
/// ```
/// use vitui_runtime::work::{Requested, Task, Worker};
///
/// struct App {
///     preview: Task<String>,
/// }
///
/// fn pane(preview: &Task<String>, selected: u64) -> Requested {
///     preview.request(selected, move |_cancel| format!("file {selected}"))
/// }
///
/// let worker = Worker::queueing();
/// let app = App { preview: Task::new(&worker) };
/// assert_eq!(pane(&app.preview, 3), Requested::Asked);
/// worker.run(0);
/// assert_eq!(app.preview.take().as_deref(), Some("file 3"));
/// ```
pub struct Task<T> {
    /// Where the worker leaves its answer. **Private**, and the whole staleness fix depends on it.
    outbox: Arc<Slot<Landing<T>>>,
    /// The worker this task's questions go to.
    inbox: Arc<Inbox>,
    /// The last question asked, which is what makes the verb idempotent.
    key: Cell<Option<u64>>,
    /// The generation minted for it. A landing carrying any other one is stale.
    asked: Cell<Option<Generation>>,
    /// The in-flight job's bracket, closed when a different question arrives.
    cancel: Cell<Option<Arc<Cancel>>>,
    /// **The app thread's half.** `Cell` already costs `Sync`; this is what costs `Send`, and the
    /// engine's `Screen` and `View` are branded the same way for the same reason.
    _app_thread: PhantomData<*const ()>,
}

impl<T> Task<T> {
    /// A task with no question asked, sending its questions to `worker`.
    #[must_use]
    pub fn new(worker: &Worker) -> Task<T> {
        Task {
            outbox: Arc::new(Slot::new()),
            inbox: Arc::clone(&worker.inbox),
            key: Cell::new(None),
            asked: Cell::new(None),
            cancel: Cell::new(None),
            _app_thread: PhantomData,
        }
    }

    /// Take the answer to the question being asked, if it has arrived.
    ///
    /// **Non-blocking, and there is no blocking twin anywhere** — that is the engine's refusal 7 and
    /// this type inherits it. A landing whose generation is not the newest is dropped here, on the
    /// app thread, and answering `None` is correct: nobody is asking that question any more.
    ///
    /// There is no `is_pending` beside this, and the engine's `Slot` gives the reason: a predicate
    /// followed by a take is two reads with a worker running between them, so the check is the take.
    pub fn take(&self) -> Option<T> {
        let landing = self.outbox.take()?;
        if Some(landing.generation) == self.asked.get() {
            Some(landing.value)
        } else {
            None
        }
    }

    /// Which question is being asked, if one is.
    #[must_use]
    pub fn asking(&self) -> Option<u64> {
        self.key.get()
    }

    /// The generation of the question being asked, if one is.
    #[must_use]
    pub fn generation(&self) -> Option<Generation> {
        self.asked.get()
    }

    /// Whether the in-flight job's bracket has been closed. Answers `false` when nothing is in
    /// flight, which is the same answer for the same reason: nothing is being cancelled.
    fn is_cancelled(&self) -> bool {
        let cancel = self.cancel.take();
        let cancelled = cancel.as_ref().is_some_and(|c| c.cancelled());
        self.cancel.set(cancel);
        cancelled
    }
}

impl<T: Send + 'static> Task<T> {
    /// Ask for the answer to `key`, computing it with `job` if nobody is asking that already.
    ///
    /// **Call it unconditionally, every frame.** `key` is what the app thread computes to identify
    /// the question — a selected index, a hash of a path — and two calls carrying the same key are
    /// one question. The answer arrives through [`Task::take`], and the [`Cancel`] handed to `job`
    /// is closed the moment a different question is asked.
    ///
    /// # What it costs
    ///
    /// The deduplicated call — which is all but one frame in a thousand — **allocates nothing**:
    /// `job` is boxed on the path that actually asks and nowhere else. Starting a job allocates a
    /// small handful of times, once per distinct question and never per frame.
    pub fn request<F>(&self, key: u64, job: F) -> Requested
    where
        F: FnOnce(&Cancel) -> T + Send + 'static,
    {
        if self.key.get() == Some(key) {
            return Requested::Deduped;
        }

        // The bracket closes on the question being replaced. It may already have finished, in which
        // case this is a store nobody reads — cancellation is a bracket, not a promise.
        if let Some(previous) = self.cancel.take() {
            previous.cancel();
        }

        let generation = Generation::mint();
        self.key.set(Some(key));
        self.asked.set(Some(generation));
        let cancel = Arc::new(Cancel::new());
        self.cancel.set(Some(Arc::clone(&cancel)));

        let outbox = Arc::clone(&self.outbox);
        self.inbox.put(Box::new(move || {
            let value = job(&cancel);
            if cancel.cancelled() {
                // Superseded while it ran. Landing it would cost a wake for a value `take` is about
                // to drop, and the drop happens here — on the worker's thread, which is the thread
                // that made it.
                return false;
            }
            land(&outbox, Landing { generation, value })
        }));
        Requested::Asked
    }
}

/// Leave a landing in the outbox, keeping the newer of the two if one is already there.
///
/// [`Slot::put`] is last-*arrival* wins, which is the right rule for a value with no identity and
/// the wrong one for an answer: two jobs running on two threads can finish in either order, and a
/// stale landing displacing a fresh one that had not been taken yet loses the fresh answer for ever
/// — there is no further post and therefore no further frame. The comparison is `>`, and it is the
/// one place in this crate two stamps are ordered.
///
/// Answers whether anything new is waiting, which is what decides whether a wake is posted.
fn land<T>(outbox: &Slot<Landing<T>>, landing: Landing<T>) -> bool {
    let generation = landing.generation;
    match outbox.put(landing) {
        None => true,
        Some(displaced) if displaced.generation > generation => {
            // A newer answer was already waiting. Put it back and drop ours — on this thread, which
            // is the worker's, which is where the engine's slot puts a displaced value on purpose.
            let _stale = outbox.put(displaced);
            false
        }
        // We displaced something older. It is dropped here for the same reason.
        Some(_older) => true,
    }
}

impl<T> std::fmt::Debug for Task<T> {
    /// Says which question is being asked and never what the answer is: `T` is the application's and
    /// may not be `Debug`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Task")
            .field("asking", &self.key.get())
            .field("generation", &self.asked.get())
            .field("cancelled", &self.is_cancelled())
            .finish()
    }
}

/// Every element, in order. **The one place *newest wins* is data loss.**
///
/// A directory read is not a preview: its payload is a *stream*, and a [`Slot`] holds one value.
///
/// | the same 10 000 rows in 100 batches | |
/// |---|---|
/// | a slot of **deltas** | **1 000 of 10 000** rows ever reach the app thread — silently |
/// | a slot of **the whole** | loses nothing, and copies the prefix again on every batch: **505 000** elements, **50.5×** |
/// | a `Drain` | each row moves once |
///
/// The delta shape is the dangerous one because it is correct-looking: the app thread sees a batch
/// only on the frames it happens to look, and frame coalescing guarantees it will not look on most
/// of them. So the pair ships **documented rather than left to preference** — [`Slot`] for the
/// newest answer to the current question, `Drain` for every element in order. Neither blocks on the
/// app thread and the difference is one method.
///
/// ```
/// use std::sync::Arc;
/// use vitui_runtime::work::Drain;
///
/// let rows: Arc<Drain<u32>> = Arc::new(Drain::new());
/// let producer = Arc::clone(&rows);
/// std::thread::spawn(move || {
///     for batch in 0..10u32 {
///         producer.extend(batch * 10..batch * 10 + 10);
///     }
/// })
/// .join()
/// .unwrap();
///
/// // One look, on one frame, and nothing was lost by not looking on the other ninety-nine.
/// let mut seen = Vec::new();
/// assert_eq!(rows.drain_into(&mut seen), 100);
/// assert_eq!(seen.len(), 100);
/// assert_eq!(rows.drain_into(&mut seen), 0);
/// ```
#[derive(Default)]
pub struct Drain<T> {
    /// A `Vec` and not a `VecDeque`: every read takes the whole of it, so there is no front to pop.
    queued: Mutex<Vec<T>>,
}

impl<T> Drain<T> {
    /// An empty queue.
    ///
    /// `const`, so an application can put one in a `static` without a `OnceLock` around it — the
    /// same door the engine's [`Slot::new`] opens.
    #[must_use]
    pub const fn new() -> Drain<T> {
        Drain {
            queued: Mutex::new(Vec::new()),
        }
    }

    /// Leave one element. `&self`, because every holder of the `Arc` is a producer.
    pub fn push(&self, value: T) {
        self.lock().push(value);
    }

    /// Leave a batch. One lock for the batch rather than one a row, which is the difference between
    /// a directory read and a hundred thousand lock acquisitions.
    pub fn extend<I: IntoIterator<Item = T>>(&self, batch: I) {
        self.lock().extend(batch);
    }

    /// Move everything waiting onto the end of `out`, and answer how many that was.
    ///
    /// **Appends rather than replaces**, so the caller keeps its allocation across frames — the same
    /// reason the key queue rewinds instead of dropping. Each element moves exactly once.
    pub fn drain_into(&self, out: &mut Vec<T>) -> usize {
        let mut queued = self.lock();
        let n = queued.len();
        out.append(&mut queued);
        n
    }

    /// The engine's `Slot` gives the reason for recovering from a poisoned lock and it is the same
    /// one here: propagating the poison would turn a producer's bug into a frozen interface, which is
    /// the one outcome this module exists to prevent.
    fn lock(&self) -> MutexGuard<'_, Vec<T>> {
        self.queued.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Says how many elements are waiting and never what they are: `T` is the application's and may not
/// be `Debug`.
impl<T> std::fmt::Debug for Drain<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Drain")
            .field("queued", &self.lock().len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The file browser's directory: file *i* costs `n − i` to decode, so the request made first
    /// finishes last. Not a contrived order — it is what an arrow-key sweep produces on any listing
    /// where the files get smaller, which is most of them.
    const FILES: usize = 20;

    // ---------------------------------------------------------------------------------------------
    // The scene: twenty selections down a directory whose files get smaller.
    // ---------------------------------------------------------------------------------------------

    /// **The defect, and it is not a flicker.** A bare slot with no generation shows a file that is
    /// not selected on nineteen of twenty frames, and the twentieth shows file 0 while the selection
    /// sits on file 19.
    #[test]
    fn the_bare_slot_shows_the_wrong_file_on_nineteen_of_twenty_frames() {
        let outbox: Slot<usize> = Slot::new();
        let selection = FILES - 1;

        let mut pane: Option<usize> = None;
        let mut wrong = 0;
        // Twenty jobs, landing in the reverse of the order they were asked: `put` replaces, so the
        // last value to *arrive* wins rather than the last to be *asked for*.
        for file in (0..FILES).rev() {
            outbox.put(file);
            if let Some(landed) = outbox.take() {
                pane = Some(landed);
            }
            if pane != Some(selection) {
                wrong += 1;
            }
        }

        assert_eq!(wrong, FILES - 1, "nineteen of twenty frames wrong");
        assert_eq!(
            pane,
            Some(0),
            "and the pane ends on the file selected first"
        );
    }

    /// **And it stays wrong**, which is the half that makes it a defect rather than a glitch: there
    /// is no further post, so there is no further wake, so there is no further frame. The wrong pane
    /// is the steady state.
    #[test]
    fn the_bare_slot_stays_wrong_for_as_long_as_you_care_to_run_it() {
        let outbox: Slot<usize> = Slot::new();
        for file in (0..FILES).rev() {
            outbox.put(file);
        }
        let mut pane = outbox.take();
        for _ in 0..100 {
            if let Some(landed) = outbox.take() {
                pane = Some(landed);
            }
        }
        assert_eq!(pane, Some(0), "a hundred further frames corrected nothing");
    }

    /// **The twin: eight bytes, and twenty of twenty frames right.** Nineteen landings are dropped,
    /// which is the whole of what the fix costs.
    #[test]
    fn the_generation_shows_the_selected_file_on_twenty_of_twenty_frames() {
        let worker = Worker::queueing();
        let task: Task<usize> = Task::new(&worker);

        // The sweep: every question asked before any of them is answered.
        for selected in 0..FILES {
            assert_eq!(
                task.request(selected as u64, move |_cancel| selected),
                Requested::Asked
            );
        }
        let selection = FILES - 1;

        let mut pane: Option<usize> = None;
        let mut right = 0;
        let mut dropped = 0;
        // The same reverse completion order, chosen by the test rather than raced for.
        for job in (0..FILES).rev() {
            worker.run(job);
            match task.take() {
                Some(landed) => pane = Some(landed),
                None => dropped += 1,
            }
            if pane == Some(selection) {
                right += 1;
            }
        }

        assert_eq!(
            right, FILES,
            "twenty of twenty frames show the selected file"
        );
        assert_eq!(pane, Some(selection));
        assert_eq!(
            dropped,
            FILES - 1,
            "and nineteen stale landings are the cost"
        );
    }

    /// A stale landing may not displace a fresh one that has not been taken yet. Nothing takes
    /// between the landings here, which is the frame an application does not run.
    #[test]
    fn a_stale_landing_does_not_displace_a_fresh_one_waiting_in_the_slot() {
        let worker = Worker::queueing();
        let task: Task<usize> = Task::new(&worker);
        for selected in 0..FILES {
            task.request(selected as u64, move |_cancel| selected);
        }
        for job in (0..FILES).rev() {
            worker.run(job);
        }
        assert_eq!(
            task.take(),
            Some(FILES - 1),
            "the newest answer survived nineteen older ones arriving after it"
        );
    }

    // ---------------------------------------------------------------------------------------------
    // Idempotence.
    // ---------------------------------------------------------------------------------------------

    /// **A thousand frames on one selection produce one job.** The multiplier on getting this wrong
    /// is the frame rate.
    #[test]
    fn a_thousand_frames_on_one_selection_produce_one_job() {
        let worker = Worker::queueing();
        let task: Task<u32> = Task::new(&worker);

        let mut asked = 0;
        for _frame in 0..1_000 {
            if task.request(7, |_cancel| 42) == Requested::Asked {
                asked += 1;
            }
        }
        assert_eq!(asked, 1, "one question");
        assert_eq!(worker.asked(), 1, "and one job, out of a thousand calls");
        assert!(task.generation().is_some(), "one generation minted");
    }

    /// Going back to a question already answered asks it again: the answer was taken and is gone,
    /// and a task that remembered it would be a cache this type does not claim to be.
    #[test]
    fn returning_to_an_earlier_selection_asks_again() {
        let worker = Worker::queueing();
        let task: Task<u32> = Task::new(&worker);
        assert_eq!(task.request(1, |_| 1), Requested::Asked);
        assert_eq!(task.request(2, |_| 2), Requested::Asked);
        assert_eq!(task.request(1, |_| 1), Requested::Asked);
        assert_eq!(worker.asked(), 3);
    }

    // ---------------------------------------------------------------------------------------------
    // The inbox.
    // ---------------------------------------------------------------------------------------------

    /// **Nineteen of twenty questions cost nothing at all**: no thread, no started job, not even the
    /// decision to stop. The one that runs is the one still being asked for.
    #[test]
    fn a_question_replaced_before_anyone_looks_at_it_costs_nothing() {
        let inbox = Inbox::new(Held::Newest(None));
        let ran = Arc::new(AtomicU64::new(0));
        let answered = Arc::new(AtomicU64::new(0));

        for question in 0..FILES as u64 {
            let ran = Arc::clone(&ran);
            let answered = Arc::clone(&answered);
            inbox.put(Box::new(move || {
                ran.fetch_add(1, Ordering::Relaxed);
                answered.store(question, Ordering::Relaxed);
                true
            }));
        }

        let assignment = inbox
            .next()
            .expect("the inbox is open and holds a question");
        assert!(assignment());
        assert_eq!(ran.load(Ordering::Relaxed), 1, "one of twenty ran");
        assert_eq!(
            answered.load(Ordering::Relaxed),
            FILES as u64 - 1,
            "and it is the newest question, not the oldest"
        );
        assert_eq!(inbox.replaced.load(Ordering::Relaxed), FILES as u64 - 1);
    }

    // ---------------------------------------------------------------------------------------------
    // Cancellation.
    // ---------------------------------------------------------------------------------------------

    /// The decode: 210 units, and whether it polls the bracket is the job's own choice.
    const UNITS: u64 = 210;

    /// **A bracket, both ends counted.** A superseded job that polls does 1 unit of 210; the decode
    /// an application writes first does not poll and does 210 of 210.
    #[test]
    fn cancellation_is_one_unit_of_two_hundred_and_ten_when_it_is_polled_and_all_of_it_when_it_is_not()
     {
        let polled = Arc::new(AtomicU64::new(0));
        let ignored = Arc::new(AtomicU64::new(0));

        let worker = Worker::queueing();
        let task: Task<u64> = Task::new(&worker);

        let counted = Arc::clone(&polled);
        task.request(0, move |cancel| {
            let mut done = 0;
            for _unit in 0..UNITS {
                done += 1;
                counted.store(done, Ordering::Relaxed);
                if cancel.cancelled() {
                    break;
                }
            }
            done
        });
        let counted = Arc::clone(&ignored);
        task.request(1, move |_cancel| {
            let mut done = 0;
            for _unit in 0..UNITS {
                done += 1;
                counted.store(done, Ordering::Relaxed);
            }
            done
        });
        // Question 0 was superseded by question 1 before either ran, and the bracket the task now
        // holds is question 1's, which is open.
        assert!(!task.is_cancelled(), "the live bracket is open");

        assert!(!worker.run(0), "a superseded job lands nothing");
        assert_eq!(polled.load(Ordering::Relaxed), 1, "1 unit of 210");

        assert!(worker.run(1), "and the question still being asked lands");
        assert_eq!(ignored.load(Ordering::Relaxed), UNITS, "210 of 210");
        assert_eq!(task.take(), Some(UNITS));
    }

    /// A job that never polls still does not land: the bracket is checked once more when it
    /// returns, so a superseded answer costs a wake and a frame nowhere.
    #[test]
    fn a_superseded_job_that_never_polls_lands_nothing_and_wakes_nobody() {
        let worker = Worker::queueing();
        let task: Task<u32> = Task::new(&worker);
        task.request(0, |_cancel| 1);
        task.request(1, |_cancel| 2);
        assert!(!worker.run(0), "nothing landed, so nothing posts");
        assert_eq!(task.take(), None);
        assert!(worker.run(1));
        assert_eq!(task.take(), Some(2));
    }

    // ---------------------------------------------------------------------------------------------
    // `Slot` and `Drain`.
    // ---------------------------------------------------------------------------------------------

    /// The directory read: 10 000 rows in 100 batches, and the app thread looks on one frame in ten
    /// because frame coalescing guarantees it will not look on most of them.
    const ROWS: usize = 10_000;
    const BATCHES: usize = 100;
    const LOOKS_EVERY: usize = 10;

    /// **A slot of deltas loses 9 000 of 10 000 rows, silently.** The negative case, and it is the
    /// dangerous one because it is correct-looking on a fast machine with a slow producer.
    #[test]
    fn a_slot_of_deltas_loses_nine_thousand_of_ten_thousand_rows() {
        let outbox: Slot<Vec<usize>> = Slot::new();
        let mut received = 0;
        for batch in 0..BATCHES {
            let rows = batch * (ROWS / BATCHES)..(batch + 1) * (ROWS / BATCHES);
            outbox.put(rows.collect());
            if batch % LOOKS_EVERY == LOOKS_EVERY - 1
                && let Some(delta) = outbox.take()
            {
                received += delta.len();
            }
        }
        assert_eq!(received, 1_000);
        assert_eq!(ROWS - received, 9_000, "nine thousand rows never existed");
    }

    /// **`Drain` loses 0 of 10 000 on the same looking schedule.**
    #[test]
    fn a_drain_loses_none_of_ten_thousand_rows_on_the_same_schedule() {
        let queue: Drain<usize> = Drain::new();
        let mut screen = Vec::new();
        for batch in 0..BATCHES {
            let rows = batch * (ROWS / BATCHES)..(batch + 1) * (ROWS / BATCHES);
            queue.extend(rows);
            if batch % LOOKS_EVERY == LOOKS_EVERY - 1 {
                queue.drain_into(&mut screen);
            }
        }
        queue.drain_into(&mut screen);
        assert_eq!(screen.len(), ROWS);
        assert_eq!(ROWS - screen.len(), 0, "nothing lost by not looking");
        assert!(
            screen.iter().copied().eq(0..ROWS),
            "and in the order they were produced"
        );
    }

    /// The third shape — a slot of the *whole* — loses nothing and copies the prefix again on every
    /// batch. **A ratio, because the number belongs to the shapes and not to this machine.**
    #[test]
    fn a_cumulative_slot_copies_fifty_times_what_a_queue_moves() {
        let mut cumulative = 0;
        let mut whole = Vec::new();
        for batch in 0..BATCHES {
            whole.extend(batch * (ROWS / BATCHES)..(batch + 1) * (ROWS / BATCHES));
            cumulative += whole.len();
        }
        assert_eq!(cumulative, 505_000);

        let queue: Drain<usize> = Drain::new();
        let mut moved = 0;
        let mut screen = Vec::new();
        for batch in 0..BATCHES {
            queue.extend(batch * (ROWS / BATCHES)..(batch + 1) * (ROWS / BATCHES));
            moved += queue.drain_into(&mut screen);
        }
        assert_eq!(moved, ROWS, "each row moves once");

        let ratio = cumulative as f64 / moved as f64;
        assert!(
            ratio > 50.0,
            "the cumulative shape copied {ratio:.1}x what the queue moved"
        );
    }

    // ---------------------------------------------------------------------------------------------
    // The wake.
    // ---------------------------------------------------------------------------------------------

    /// **Sixty frames with a job in flight ask for 0 wakeups; the polling shape asks for 60.**
    ///
    /// The count is taken at the two call sites, because the wake ledger is the
    /// and does not exist yet. What it is really asserting is on the other side of the same coin and
    /// is structural: `take` is the whole of the app thread's side, so the parked shape has nothing
    /// to ask for — and `tests::the_module_names_no_deadline` is the half that stops a later edit
    /// from giving it something.
    #[test]
    fn sixty_frames_with_a_job_in_flight_ask_for_no_wakeups() {
        let worker = Worker::queueing();
        let task: Task<u32> = Task::new(&worker);
        task.request(0, |_cancel| 42);

        let mut parked = 0;
        let mut polled = 0;
        for _frame in 0..60 {
            // The parked shape. `take` answers `None` and asks for nothing: a job in flight is not
            // a reason to run a frame, and there is no verb on `Task` that would make it one.
            if task.take().is_some() {
                parked += 1;
            }
            // The shape an application reaches for when its slot has no wake attached: a deadline so
            // it can look again. Sixty deadlines, worst streak sixty, on a screen doing nothing —
            // which is precisely the symptom the runaway detector exists for.
            polled += 1;
        }
        assert_eq!(parked, 0, "sixty frames, nothing landed, nothing asked for");
        assert_eq!(polled, 60);
    }

    /// Twenty landings arriving while the app thread is busy cost **one** wake, and the mechanism is
    /// the engine's flag rather than anything here.
    #[test]
    fn twenty_posts_while_the_app_thread_is_busy_cost_one_wake() {
        let (mut screen, wake) = sink_screen();
        for _landing in 0..20 {
            wake.post();
        }
        assert_eq!(screen.wait(), vitui_engine::Wake::Posted, "one wake");
        // And there is not a second one waiting behind it. The deadline is a safety net as much as
        // an assertion: without it a second `wait` would be the hang this crate exists to prevent.
        screen.request_wake_at(std::time::Instant::now() + std::time::Duration::from_millis(50));
        assert_eq!(
            screen.wait(),
            vitui_engine::Wake::Deadline,
            "the twenty posts were one wake, not twenty"
        );
    }

    /// **A resident worker posts once per landing and never for a job it superseded.**
    ///
    /// The one gate here that runs on a real thread, because the post is the thing being counted and
    /// a deterministic worker does not have one.
    #[test]
    fn a_resident_worker_posts_once_for_a_landing_and_not_at_all_for_a_superseded_job() {
        let (mut screen, wake) = sink_screen();
        let worker = Worker::hire(wake);
        let task: Task<u32> = Task::new(&worker);

        task.request(1, |_cancel| 7);
        assert_eq!(screen.wait(), vitui_engine::Wake::Posted);
        // The landing may be taken on the frame the wake bought, which is this one.
        let mut landed = task.take();
        while landed.is_none() {
            screen
                .request_wake_at(std::time::Instant::now() + std::time::Duration::from_millis(20));
            screen.wait();
            landed = task.take();
        }
        assert_eq!(landed, Some(7));

        // A question superseded before the worker looks at it is not even a job. The worker is held
        // busy first, and *observed* to be busy: putting two questions in while it is between jobs
        // would be measuring a race rather than the inbox.
        let running = Arc::new(AtomicBool::new(false));
        let held = Arc::new(AtomicBool::new(true));
        let (started, block) = (Arc::clone(&running), Arc::clone(&held));
        worker.ask(move || {
            started.store(true, Ordering::Release);
            while block.load(Ordering::Acquire) {
                std::hint::spin_loop();
            }
        });
        while !running.load(Ordering::Acquire) {
            std::hint::spin_loop();
        }

        let before = worker.replaced();
        task.request(2, |_cancel| 2);
        task.request(3, |_cancel| 3);
        held.store(false, Ordering::Release);

        screen.request_wake_at(std::time::Instant::now() + std::time::Duration::from_millis(500));
        assert_eq!(
            screen.wait(),
            vitui_engine::Wake::Posted,
            "the question still being asked landed"
        );
        assert_eq!(
            worker.replaced() - before,
            1,
            "and the one before it never became a job"
        );
    }

    /// A screen over a sink, on the system clock, so that `wait` is the real one.
    fn sink_screen() -> (vitui_engine::Screen, WakeHandle) {
        vitui_engine::Engine::new(vitui_engine::Config {
            size: (80, 24),
            output: vitui_engine::Output::Sink(Box::new(Vec::new())),
            clock: vitui_engine::Clock::System,
            max_frame_rate: f32::INFINITY,
            ..Default::default()
        })
        .attach()
        .expect("attaching to a sink cannot fail")
    }

    // ---------------------------------------------------------------------------------------------
    // The structural claims: absent code is only visible where it would have been written.
    // ---------------------------------------------------------------------------------------------

    /// What ships, without the gates below it. The claims scanned for are about the module, and a
    /// gate is allowed to name the thing it is asserting the absence of.
    fn shipped_source() -> &'static str {
        let source = include_str!("work.rs");
        let gates = source
            .find("\n#[cfg(test)]\n")
            .expect("this module's own test module");
        &source[..gates]
    }

    /// Every line of it that is code rather than prose. A doc comment explaining *why* something is
    /// absent has to be able to name it.
    fn shipped_code() -> impl Iterator<Item = (usize, &'static str)> {
        shipped_source()
            .lines()
            .enumerate()
            .map(|(n, line)| (n + 1, line.trim_start()))
            .filter(|(_, code)| !code.starts_with("//"))
    }

    /// **This module names no identity, so there is no path to a task registry.**
    ///
    /// The same instrument `data`'s *no trait* claim uses, and for the same reason: this is a
    /// statement about code that is *not* here, and a test that constructed something instead would
    /// pass for any reason including the registry existing and being unused.
    #[test]
    fn the_module_names_no_identity_so_there_is_no_path_to_a_registry() {
        for (n, code) in shipped_code() {
            assert!(
                !code.contains("IdTable") && !code.contains("crate::id"),
                "line {n} reaches for identity: {code}"
            );
        }
    }

    /// **And it names no deadline.** A job in flight is not a reason to run a frame, and the way that
    /// stops being true is somebody adding a `request_wake_at` here to look again.
    #[test]
    fn the_module_names_no_deadline() {
        for (n, code) in shipped_code() {
            assert!(
                !code.contains("request_wake_at"),
                "line {n} asks for a deadline: {code}"
            );
        }
    }

    /// The generation is eight bytes and the landing is the payload plus those eight, which is what
    /// *what crosses the thread boundary over the payload* means.
    #[test]
    fn the_generation_is_eight_bytes_over_the_payload() {
        assert_eq!(std::mem::size_of::<Generation>(), 8);
        assert_eq!(
            std::mem::size_of::<Landing<[u8; 64]>>(),
            64 + std::mem::size_of::<Generation>()
        );
    }

    /// What crosses the boundary is `Send`; what stays does not.
    #[test]
    fn the_workers_half_is_send_and_the_app_threads_half_is_not() {
        fn assert_send<T: Send>() {}
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send::<Worker>();
        assert_send_sync::<Cancel>();
        assert_send_sync::<Drain<Vec<u8>>>();
        assert_send_sync::<Slot<Landing<Vec<u8>>>>();
        // `Task<T>` is absent from this list on purpose; the compile-outcome pair on the type is
        // where that is asserted, because a `Send` bound that is *not* satisfied cannot be written
        // as a passing call.
    }

    /// A task whose worker has been dropped answers `None` rather than hanging or growing.
    #[test]
    fn a_task_outliving_its_worker_answers_nothing() {
        let worker = Worker::queueing();
        let task: Task<u32> = Task::new(&worker);
        drop(worker);
        assert_eq!(task.request(1, |_cancel| 7), Requested::Asked);
        assert_eq!(task.take(), None);
    }
}
