//! `Slot<T>`: what is offered instead of blocking.
//!
//! The last section is one sentence long — *what is offered instead of blocking is
//! [`WakeHandle`](crate::WakeHandle) and a `Slot<T>` whose only accessor is a non-blocking
//! [`take`](Slot::take)* — and the refusal 7 is what it costs: **there is no `recv`, no `wait`, no
//! `Future` and no completion returned by anything on the app-thread side.** That absence is the
//! second compile-time rung of the unblockable app thread, and it is checkable by reading the API
//! rather than by running anything. `crate::gates::every_blocking_receive_in_the_crate_is_outside_the_app_threads_loop` is the half a
//! reader cannot do.
//!
//! # The shape a background result arrives in
//!
//! ```text
//! worker                                  app thread
//!   compute                                 wait()      -> Wake::Posted
//!   slot.put(result)                        slot.take()  -> Some(result)
//!   wake.post()                             draw with it
//!                                           present()
//! ```
//!
//! Two objects and no third: the result travels in the slot and the *fact that there is one*
//! travels in the wake. They are deliberately not one call. A single "post this value" verb would
//! have to own the value's type, which would put a generic parameter on
//! [`WakeHandle`](crate::WakeHandle) — a handle an application clones into every worker it has,
//! each of which produces a different type. So the wake stays typeless and the slot stays untyped
//! by the engine: an application has as many slots as it has kinds of result, and one handle.
//!
//! # Why this is in the engine at all
//!
//! It is three lines around a `Mutex`, and an application could write it. That is the point of
//! writing it here: the three lines an application would write instead are `Receiver::recv`,
//! `Condvar::wait` or `block_on`, and each of those is a frozen interface. A named, shipped,
//! documented type with **no blocking accessor on it** is what makes the cheap thing the obvious
//! thing — and the lint fragment names `std::sync::mpsc::Receiver::recv` and
//! `std::sync::Mutex::lock` for the application that reaches past it anyway.
//!
//! The lock inside is not a contradiction, and the distinction is the whole of the definition of
//! the offence: **what freezes an interface is waiting for work that has not happened**, not
//! waiting for a `memcpy` that is already running. This lock is held for one move of one value and
//! never across anything that can block, so its worst case is the length of a `mem::replace` on
//! whichever core is holding it. `Receiver::recv` has no worst case at all.

use std::sync::{Mutex, PoisonError};

/// Somewhere a worker leaves a result and the app thread picks it up. **Non-blocking, both ends.**
///
/// `Send + Sync` — an application wraps one in an `Arc` and clones it into as many workers as it
/// likes:
///
/// ```
/// fn assert_send_sync<T: Send + Sync>() {}
/// assert_send_sync::<vitui_engine::Slot<Vec<u8>>>();
/// ```
///
/// ```
/// use std::sync::Arc;
/// use vitui_engine::Slot;
///
/// let slot = Arc::new(Slot::new());
/// let worker = Arc::clone(&slot);
/// std::thread::spawn(move || worker.put(41 + 1)).join().unwrap();
/// assert_eq!(slot.take(), Some(42));
/// // Taken once. There is nothing to wait for and nothing to poll.
/// assert_eq!(slot.take(), None);
/// ```
///
/// # One value, latest wins, and nothing is dropped silently
///
/// [`put`](Slot::put) answers with whatever it displaced. A slot is not a queue — a queue that the
/// app thread drains at frame rate is a queue that grows without bound when the producer is faster,
/// which is the backpressure question arriving on the application's side of the fence — so
/// *latest wins* is the only policy that has no unbounded state in it. But a superseded value that
/// merely vanished would be a `Drop` at a moment nobody chose, on a thread nobody chose, and that is
/// exactly the class of thing this crate does not do to its caller. So it comes back, on the worker's
/// own thread, and the worker decides.
///
/// # There is no `is_some`, and that is not an omission
///
/// A predicate followed by a `take` is two reads with a window between them, and the second one can
/// still answer `None` on the app thread's own next line — the worker is running. Every honest use
/// of a check is `if let Some(v) = slot.take()`, which is one read, so the check is the take.
///
/// # Refusal 7, as a compile outcome
///
/// **No blocking primitive on the app thread and no completion anywhere.** It is *the word
/// `recv` does not appear*, and `crate::gates` holds the literal form of that over the source with a
/// reason per allowance. Here is the half a caller can see: there is nothing on this type to wait
/// on, and no `Future` to await.
///
/// ```compile_fail,E0599
/// let slot: vitui_engine::Slot<u32> = vitui_engine::Slot::new();
/// let _ = slot.recv();
/// ```
///
/// ```compile_fail,E0599
/// let slot: vitui_engine::Slot<u32> = vitui_engine::Slot::new();
/// let _ = slot.wait();
/// ```
///
/// and the twin, naming by path the one accessor there is — which returns an `Option` rather than
/// blocking, and is the whole of what the app thread is offered in place of a receive:
///
/// ```
/// use vitui_engine::Slot;
///
/// let slot: Slot<u32> = Slot::new();
/// assert_eq!(Slot::take(&slot), None);
/// assert_eq!(Slot::put(&slot, 42), None);
/// assert_eq!(Slot::take(&slot), Some(42));
/// ```
pub struct Slot<T> {
    /// **A `Mutex<Option<T>>`, and not an `AtomicPtr`.** The atomic version needs a `Box` per `put`
    /// and one `unsafe` to get the value back out, and the refusal 12 is that there is no `unsafe`
    /// in this crate. The lock is held for a `mem::replace` and the values that cross here are
    /// whole results — one per frame at the very most, against a frame that is 16.6 ms long.
    held: Mutex<Option<T>>,
}

impl<T> Slot<T> {
    /// An empty slot.
    ///
    /// `const`, so that an application can put one in a `static` and not have a `OnceLock` around
    /// it as well.
    pub const fn new() -> Slot<T> {
        Slot {
            held: Mutex::new(None),
        }
    }

    /// Leave a value for the app thread, and take back whatever this displaced.
    ///
    /// `&self`, because every holder of the `Arc` is a producer. The value moves; nothing is cloned
    /// and nothing is allocated.
    ///
    /// **This does not wake anybody.** Posting is [`WakeHandle::post`](crate::WakeHandle::post), and
    /// it is a separate call so that a worker producing ten intermediate results can put all ten and
    /// post once.
    pub fn put(&self, value: T) -> Option<T> {
        self.lock().replace(value)
    }

    /// Take the value if there is one, and answer at once if there is not.
    ///
    /// The **only** accessor. There is no blocking twin of this function anywhere, which is the
    /// property the refusal 7 is about.
    pub fn take(&self) -> Option<T> {
        self.lock().take()
    }

    /// A producer that panicked mid-`put` cannot take the app thread down with it.
    ///
    /// The invariant a poisoned `Mutex` protects is *some producer left this structure
    /// inconsistent*, and there is no inconsistent state reachable here: the guard holds one
    /// `Option<T>`, both verbs replace it whole, and a panic between them is a panic between two
    /// moves. Propagating the poison would turn a worker's bug into a frozen interface, which is the
    /// one outcome this whole module exists to prevent.
    fn lock(&self) -> std::sync::MutexGuard<'_, Option<T>> {
        self.held.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl<T> Default for Slot<T> {
    fn default() -> Slot<T> {
        Slot::new()
    }
}

/// Says whether there is something waiting and never what it is: `T` is the application's and may
/// not be `Debug`.
impl<T> std::fmt::Debug for Slot<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Through [`Slot::lock`], not `Mutex::lock` directly. A worker that panicked mid-`put`
        // poisons this, and `is_ok_and` would then report an **empty** slot while `take` still hands
        // the value over — a `Debug` line that contradicts the accessor beside it, in exactly the
        // situation somebody is printing it to understand.
        let filled = self.lock().is_some();
        f.debug_struct("Slot").field("filled", &filled).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_slot_is_send_and_sync_so_a_worker_can_hold_one() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Slot<Vec<u8>>>();
    }

    #[test]
    fn an_empty_slot_answers_nothing_rather_than_waiting() {
        let slot: Slot<u32> = Slot::new();
        assert_eq!(slot.take(), None);
    }

    #[test]
    fn what_a_worker_put_is_what_the_app_thread_takes_once() {
        let slot: Slot<u32> = Slot::new();
        assert_eq!(slot.put(7), None);
        assert_eq!(slot.take(), Some(7));
        assert_eq!(slot.take(), None);
    }

    #[test]
    fn a_second_put_supersedes_the_first_and_hands_it_back_rather_than_dropping_it() {
        let slot: Slot<u32> = Slot::new();
        assert_eq!(slot.put(1), None);
        assert_eq!(slot.put(2), Some(1));
        assert_eq!(slot.take(), Some(2));
    }

    #[test]
    fn a_worker_puts_across_a_thread_boundary_and_the_app_thread_takes_it() {
        let slot: std::sync::Arc<Slot<String>> = std::sync::Arc::new(Slot::new());
        let worker = std::sync::Arc::clone(&slot);
        std::thread::spawn(move || {
            worker.put(String::from("the aggregate, computed elsewhere"));
        })
        .join()
        .expect("the worker returned");
        assert_eq!(
            slot.take().as_deref(),
            Some("the aggregate, computed elsewhere")
        );
    }
}
