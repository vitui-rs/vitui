//! The handoff: one mailbox, a pool of two, and the authoritative size.
//!
//! # One synchronisation primitive in the whole design
//!
//! `Mutex<Shared>` plus two condvars — one for *a packet landed*, one for *the renderer is free* —
//! and `Shared` holds the slot, the free list and the ready flag together. There is not
//! one primitive per concern, and the mutex is not a compromise: hybrid pacing requires a condvar,
//! `std`'s condvar requires a mutex, so the mutex exists whatever the slot looks like. Putting the
//! slot under it costs nothing. The critical section on submit is measured at
//! [`crate::gates::what_the_handoff_costs`], against **47 ns**.
//!
//! The alternatives were rejected on `unsafe`, not on speed, and the list is here so nobody
//! rebuilds one: **a seqlock cannot be written in safe Rust at all**, because its reader reads while
//! the writer writes; a lock-free newest-only slot needs `AtomicPtr` and a manual `Box::from_raw`; a
//! channel of owned frames queues frames we would only drop.
//!
//! **The honest cost of not being lock-free**, stated rather than buried: if the app thread is
//! preempted while holding the lock, the render thread waits a quantum. Nothing cheaper than
//! `unsafe` prevents that and it is accepted.
//!
//! **The lock is never held across a syscall** (the fifth invariant). Everything below hands
//! a `Box<Packet>` in or out and returns; the write happens between [`Mailbox::take`] and
//! [`Mailbox::finish`], with no guard alive.
//!
//! # Why *free* means the renderer has taken the packet, not that it has finished writing
//!
//! This is the decision that fixes the pool at two, so it is worth stating rather than reading off
//! the code. §7 derives the pool size as *one being filled, one in the renderer's hands* — two
//! packets alive at one instant — and that shape exists only if the app may begin filling while the
//! renderer still holds the last one. So [`Mailbox::take`] is what sets `ready`, and the app's
//! composite overlaps the renderer's write.
//!
//! The trade, because it is real: a frame submitted while a 200 ms write is in flight waits for that
//! write to end, so on a slow link the frame that reaches the wire is one composite stale. Signalling
//! at *finish* instead would compose later and paint fresher, at the price of never overlapping the
//! composite with the write at all — which is most of why compositing is on the app thread. The
//! coalescing argument is untouched either way: on a 200 ms frame the app pays for one composite
//! rather than the twenty it would have thrown away, because damage accumulates in the structure
//! and 6.6 ns is what it costs to interrogate.
//!
//! # There is no backpressure and the drop path is unreachable
//!
//! The app thread never waits for a slot. Dropping intermediate frames is implemented as **never
//! composing them**: [`Lease::Busy`] is what `present` gets while the renderer has not taken the
//! last packet, and it returns without compositing. The consequence is stronger than intended —
//! with one producer and the ready gate, **the slot is always empty at submit, so a packet can never
//! be superseded**. [`Mailbox::superseded`] exists so that the counter can prove it stays at zero,
//! and register entry #9 reads it over 10 000 cycles. If it ever moves, the pacing gate has left the
//! app thread and frames are being composed to be thrown away.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Condvar, Mutex};

use crate::packet::Packet;

/// How many packets exist, ever.
///
/// **Two, provably rather than empirically**: one being filled, one in the renderer's hands. See the
/// module documentation for why *free* is the take and not the write, which is what makes the second
/// one necessary and the third one unreachable.
pub(crate) const POOL: usize = 2;

/// What the app thread got when it asked for a packet to fill.
#[derive(Debug)]
pub(crate) enum Lease {
    /// A packet nobody else holds. Fill it, then [`Mailbox::submit`] or [`Mailbox::give_back`].
    Ready(Box<Packet>),
    /// The render thread has not taken the last packet yet, so this frame is not composited at all.
    /// Damage stays in the structure and coalesces into the next one.
    Busy,
    /// The renderer is free and the pool is empty, which the pool size makes unreachable.
    /// [`Mailbox::starved`] is register entry #8's counter.
    Starved,
}

/// The slot, the free list and the ready flag, under one lock.
///
/// **The `Box` is the point and not a habit**, which is why the lint is off rather than obeyed: what
/// crosses the critical section is a pointer, so a submit and a take move eight bytes. A `Vec<Packet>`
/// against an `Option<Packet>` would work and would allocate nothing — the buffers inside a `Packet`
/// keep their heap when it moves — but it would memcpy the whole struct twice per frame *inside the
/// lock*, and the 47 ns is a number about that lock.
#[derive(Debug)]
#[allow(clippy::vec_box)]
struct Shared {
    /// The packet the render thread has not taken yet. **Always `None` at submit**, which is the
    /// whole of the backpressure decision.
    slot: Option<Box<Packet>>,
    /// Packets nobody holds.
    free: Vec<Box<Packet>>,
    /// Whether the render thread has taken every packet submitted so far.
    ///
    /// It equals `slot.is_none()` by construction, and it is a named flag anyway because it is what
    /// the *app* thread waits on: the app has no business reading the slot, and *may I compose* is
    /// the question it is actually asking.
    ready: bool,
    /// Set once, by the app thread, to bring the render thread out of its wait for the last time.
    quit: bool,
    /// Set by the render thread on its way out, **including on the way out of a panic**.
    ///
    /// Without it a dead renderer is a silent deadlock rather than a visible failure: it dies
    /// holding the packet, so `ready` never comes back, `wait_until_free` parks for ever on a
    /// condvar nobody will signal, and `quit` cannot arrive because it is set from `Screen::drop`,
    /// which cannot run while the app thread is parked. **A hang is worse than a failure**, and this
    /// is the flag that makes it a failure.
    ///
    /// It does **not** make the renderer look free. Setting `ready` would let the app submit into a
    /// slot nobody empties, which moves register entry #9's counter and destroys the meaning of the
    /// one gate that says frames are never composed to be thrown away. So the app is released from
    /// the wait and every frame after answers `submitted: false`: nothing can be painted, because
    /// the sink went with the thread. `Screen::wait`'s `Wake::Quit` is where it surfaces, and
    /// [`crate::shutdown`] is what gives the terminal back afterwards: `Screen::drop` finds no
    /// renderer to write the epilogue through and the site's own sink catches it, which is the same
    /// arm a panic on any other thread takes.
    gone: bool,
    /// How many packets were dropped on the floor by a submit landing on a full slot. **Zero, for
    /// ever**: register entry #9.
    superseded: u32,
    /// How many leases found the renderer free and the pool empty. Register entry #8.
    starved: u32,
    /// How many packets the render thread has finished writing.
    painted: u64,
    /// How many times the render thread's wait on *a packet landed* has returned — a notification, a
    /// spurious wakeup, anything. **Zero over an idle window**, which is the render thread's half of
    /// register entry #17: the app thread's half is `crate::clock::WakeSource`'s park counters.
    wakeups: u64,
    /// The **address** of the packet most recently returned to the pool, or zero before any has been.
    ///
    /// An index was what this held first, and the review found two ways it was wrong. `finish` pushes
    /// at the tail and `lease` pops from the tail, so the next lease invalidated it; and zero was a
    /// valid index before anything had ever been packed, so the accessor answered with a packet
    /// nobody had filled instead of admitting there was none. Both are the position rule at the top
    /// of `crate::packet`, one level down: *nothing compared across events may be derived from a
    /// position.* An address is the identity the pool actually preserves, because a `Box` does not
    /// move, and zero is not one.
    ///
    /// **Returned, not painted**, and the distinction is forced rather than chosen: the pool reuses
    /// buffers, so a discarded frame packs over the bytes of the painted one before it — in the same
    /// `Box`. There is no last-painted packet to hold on to once the next pack has happened, and a
    /// name that claimed otherwise would be describing a copy that does not exist.
    #[cfg(test)]
    last_returned: usize,
    /// When the packet in the slot was submitted, for register entry #26.
    ///
    /// `cfg(test)`, and not because the number is uninteresting: it is one `Instant::now()` per
    /// submit on a path §7 prices in nanoseconds, and the distribution it feeds is a **report** that
    /// may never be load-bearing for a gate. A release build has nothing to read it.
    #[cfg(test)]
    submitted_at: Option<std::time::Instant>,
    /// App submit to the render thread holding the packet, one entry per take.
    #[cfg(test)]
    latencies: Vec<std::time::Duration>,
}

#[cfg(test)]
impl Shared {
    /// Record the identity of the packet just pushed onto the free list.
    fn remember_returned(&mut self) {
        let packet: &Packet = self.free.last().expect("just pushed");
        self.last_returned = std::ptr::from_ref(packet) as usize;
    }
}

/// The mailbox: everything that crosses between the app thread and the render thread.
#[derive(Debug)]
pub(crate) struct Mailbox {
    shared: Mutex<Shared>,
    /// A packet landed in the slot, or `quit` was set.
    landed: Condvar,
    /// The render thread took a packet, so the app may compose again.
    free: Condvar,
}

impl Mailbox {
    /// A mailbox holding the whole pool, with the renderer free and nothing in the slot.
    pub(crate) fn new() -> Mailbox {
        Mailbox {
            shared: Mutex::new(Shared {
                slot: None,
                free: (0..POOL).map(|_| Box::new(Packet::new())).collect(),
                ready: true,
                quit: false,
                gone: false,
                superseded: 0,
                starved: 0,
                painted: 0,
                wakeups: 0,
                #[cfg(test)]
                last_returned: 0,
                #[cfg(test)]
                submitted_at: None,
                #[cfg(test)]
                latencies: Vec::new(),
            }),
            landed: Condvar::new(),
            free: Condvar::new(),
        }
    }

    /// Ask for a packet to fill.
    ///
    /// One lock acquisition, and it answers the pacing question and hands over the buffer together:
    /// a caller that asked *is the renderer free* and then *give me a packet* would take the lock
    /// twice to learn one thing, because nothing but this thread can change either answer.
    pub(crate) fn lease(&self) -> Lease {
        let mut shared = self.lock();
        if !shared.ready {
            return Lease::Busy;
        }
        match shared.free.pop() {
            Some(packet) => Lease::Ready(packet),
            None => {
                shared.starved += 1;
                Lease::Starved
            }
        }
    }

    /// Hand a filled packet to the render thread.
    ///
    /// The slot is empty here, always — see the module documentation — and the `superseded` counter
    /// is what proves it rather than an `assert` that would be compiled out of a release build.
    pub(crate) fn submit(&self, packet: Box<Packet>) {
        {
            let mut shared = self.lock();
            if let Some(dropped) = shared.slot.replace(packet) {
                shared.superseded += 1;
                shared.free.push(dropped);
            }
            shared.ready = false;
            #[cfg(test)]
            {
                shared.submitted_at = Some(std::time::Instant::now());
            }
        }
        self.landed.notify_one();
    }

    /// Put a packet back without submitting it: the resize path, and nothing else.
    ///
    /// `ready` is untouched, because a lease never cleared it. **A lease is never invalidated; the
    /// frame it produced may be discarded** (the sixth invariant).
    pub(crate) fn give_back(&self, packet: Box<Packet>) {
        let mut shared = self.lock();
        shared.free.push(packet);
        #[cfg(test)]
        shared.remember_returned();
    }

    /// Block until there is a packet to write, or until the app thread has said to stop.
    ///
    /// `None` means stop. Taking is what signals the app that it may compose again, so it happens
    /// under the same lock as the wait and the notification goes out with no guard alive.
    pub(crate) fn take(&self) -> Option<Box<Packet>> {
        let mut shared = self.lock();
        loop {
            // **Quit first, and the last frame is not flushed** (spec §7): showing state that is
            // already stale buys nothing and costs exit latency. Asserted here as an absence, and
            // as a write count on the wire at
            // `crate::gates::the_last_frame_is_not_flushed_on_quit`.
            if shared.quit {
                return None;
            }
            if let Some(packet) = shared.slot.take() {
                shared.ready = true;
                // Stamped here rather than after the write: what §7 reports is *app submit to the
                // render thread holding the packet*, which is a scheduler latency and ends the
                // moment this thread has it.
                #[cfg(test)]
                if let Some(at) = shared.submitted_at.take() {
                    shared.latencies.push(at.elapsed());
                }
                drop(shared);
                self.free.notify_one();
                return Some(packet);
            }
            shared = self
                .landed
                .wait(shared)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            shared.wakeups += 1;
        }
    }

    /// Whether the app thread has said stop.
    ///
    /// The one thing a test can watch to know that `Screen::drop` has reached its join, which is
    /// what lets a gate about *the last frame is not flushed* be an ordering rather than a sleep.
    #[cfg(test)]
    pub(crate) fn quit_requested(&self) -> bool {
        self.lock().quit
    }

    /// Take whatever is in the slot without waiting: the deterministic mode's renderer, which is
    /// the app thread itself and has just put it there.
    pub(crate) fn take_now(&self) -> Option<Box<Packet>> {
        let mut shared = self.lock();
        let packet = shared.slot.take()?;
        shared.ready = true;
        Some(packet)
    }

    /// Give a written packet back to the pool.
    pub(crate) fn finish(&self, packet: Box<Packet>) {
        let mut shared = self.lock();
        shared.free.push(packet);
        shared.painted += 1;
        #[cfg(test)]
        shared.remember_returned();
    }

    /// The packet the last frame packed, for register entry #10's equality.
    ///
    /// A closure rather than a reference, because the packet lives behind the lock and a `&Packet`
    /// handed out of here would outlive the guard. `None` when no frame has packed one yet, and
    /// `None` when the one that did is still in the renderer's hands — which on the threaded path is
    /// any frame the caller has not waited for. Pin `Clock::Manual`, where `present` has finished
    /// writing before it returns.
    #[cfg(test)]
    pub(crate) fn with_last_packed<R>(&self, f: impl FnOnce(&Packet) -> R) -> Option<R> {
        let shared = self.lock();
        let packet = shared
            .free
            .iter()
            .find(|p| std::ptr::from_ref::<Packet>(&**p) as usize == shared.last_returned)?;
        Some(f(packet))
    }

    /// Block until the render thread has taken whatever was submitted.
    ///
    /// **Ticket 19 did not make this public, and the reason is worth the paragraph.** `wait() -> Wake`
    /// does multiplex the renderer going free with an input event, a deadline and a post — but a
    /// thread cannot park on two condvars, so it parks on `crate::clock::WakeSource`'s and the render
    /// thread signals *that* one from its `take`. This condvar stayed where it was: it is what the
    /// gates use to order a frame against the renderer having taken it, which is a question about the
    /// handoff and not about a wake.
    ///
    /// So the second condvar of the design is now blocked on only by tests. That is a smaller claim
    /// than it sounds — the *notification* is in the frame path either way, and what changed is which
    /// waiter it releases — but it is a claim, and it is written here rather than left for a reader to
    /// notice.
    pub(crate) fn wait_until_free(&self) {
        let mut shared = self.lock();
        while !shared.ready && !shared.quit && !shared.gone {
            shared = self
                .free
                .wait(shared)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
    }

    /// The render thread is not coming back, whether it returned or unwound.
    ///
    /// Called from a guard's `Drop` on that thread, so a panic takes this path too. Both condvars
    /// are signalled: the app may be parked on *free*, and nothing else will ever release it.
    pub(crate) fn renderer_gone(&self) {
        self.lock().gone = true;
        self.free.notify_all();
        self.landed.notify_all();
    }

    /// Whether the render thread has left.
    ///
    /// **`present` answers `submitted: false` from the frame after, not from this one.** `lease` gates
    /// on `ready`, and this flag does not lower it — deliberately, because raising `ready` would let
    /// the app submit into a slot nobody empties and move register entry #9's counter. So a thread
    /// that dies while `ready` is true leaves exactly one frame that leases, composites, packs and
    /// submits into a slot that will never be emptied, and reports `submitted: true` for bytes that
    /// cannot reach the wire. One wasted composite; the frame after finds `ready` false and every
    /// frame after that too.
    ///
    /// `cfg(test)`, and the reason is a scope boundary rather than a shortcut: a dead renderer is
    /// **indistinguishable from a permanently busy one** through the public surface, because
    /// `Presented` has no field for it and the engine offers no completion anywhere (the refusal
    /// 7). What this ticket owed was that the app thread does not *hang*; telling the application
    /// its renderer is gone is a `Wake::Quit` or a shutdown, and inventing a third
    /// spelling here would be a public API this backlog has not decided. The half of the same
    /// question is [`crate::clock::WakeSource::renderer_gone`], which **cancels** an owed frame
    /// rather than releasing it — releasing it spins at the frame gap against a sink that is gone —
    /// so the app thread parks and any `post` or `quit` gets it out.
    #[cfg(test)]
    pub(crate) fn renderer_is_gone(&self) -> bool {
        self.lock().gone
    }

    /// Tell the render thread to stop after the packet it is holding, if any.
    ///
    /// **The first thing `Screen::drop` does**, and it has to be: what follows it is the join, and a
    /// render thread parked on a condvar nobody signalled is a process that never exits. It is also
    /// what `crate::gates::the_last_frame_is_not_flushed_on_quit` watches through
    /// [`quit_requested`](Mailbox::quit_requested), because *quit arrived before the renderer could
    /// take the next packet* is an ordering and not a duration.
    pub(crate) fn quit(&self) {
        self.lock().quit = true;
        self.landed.notify_one();
        self.free.notify_one();
    }

    /// Undo [`quit`](Mailbox::quit), for a session that is coming back.
    ///
    /// **The one caller is [`Screen::resume`](crate::Screen::resume)**, and it is the reason `quit`
    /// is a `bool` rather than a one-way atomic: a suspended session joined its render thread the
    /// way `Screen::drop` does, and the mailbox it left behind says *nobody is coming for a packet
    /// ever again*. A fresh render thread on a mailbox in that state exits before it takes a frame.
    ///
    /// # The packet in the slot is returned to the pool here, and not counted as superseded
    ///
    /// *The last frame is not flushed on quit*, so a suspend that raced a submit leaves
    /// a packet nobody wrote. It goes back on the free list directly rather than through the
    /// supersede path, because [`superseded`](Mailbox::superseded) is register entry #9's counter
    /// and its property is **zero, for ever** — a frame that was composed to be thrown away. This
    /// one was composed to be *written*, and the terminal left before it could be. Counting it
    /// there would put a number in the one place that may not have one, for an event that is not
    /// what the entry is about.
    ///
    /// Discarding it is right rather than merely convenient: the next thing the terminal is told is
    /// [`Screen::resume`]'s repaint of every cell, so a frame from before the alt screen was left
    /// is a frame with nothing left to say.
    pub(crate) fn reopen(&self) {
        let mut shared = self.lock();
        if let Some(stale) = shared.slot.take() {
            shared.free.push(stale);
            #[cfg(test)]
            shared.remember_returned();
        }
        shared.ready = true;
        shared.quit = false;
        shared.gone = false;
    }

    /// Register entry #9's counter: how many packets a submit dropped on the floor.
    ///
    /// `cfg(test)` because nothing in a release build reads it: the counters are gate instruments,
    /// and the *path* they count is what has to exist in release — a superseded packet goes back to
    /// the pool rather than being leaked, whether or not anybody is counting.
    #[cfg(test)]
    pub(crate) fn superseded(&self) -> u32 {
        self.lock().superseded
    }

    /// Register entry #8's counter: how many leases found the pool empty.
    #[cfg(test)]
    pub(crate) fn starved(&self) -> u32 {
        self.lock().starved
    }

    /// How many packets the renderer has finished writing.
    #[cfg(test)]
    pub(crate) fn painted(&self) -> u64 {
        self.lock().painted
    }

    /// How many times the render thread's wait has returned. Register entry #17's other half.
    #[cfg(test)]
    pub(crate) fn wakeups(&self) -> u64 {
        self.lock().wakeups
    }

    /// Every app-submit-to-render-holding-it delay recorded so far. Register entry #26.
    #[cfg(test)]
    pub(crate) fn latencies(&self) -> Vec<std::time::Duration> {
        self.lock().latencies.clone()
    }

    /// Throw away what the birth frames recorded, so a report is about the loop that follows.
    #[cfg(test)]
    pub(crate) fn reset_latencies(&self) {
        let mut shared = self.lock();
        shared.latencies.clear();
        shared.submitted_at = None;
    }

    /// A poisoned mailbox is a thread that panicked inside a critical section that runs no caller's
    /// code, so the state behind the lock is intact whoever died holding it. Recovering the guard
    /// here is what keeps a panic on one thread from turning into a second, unrelated panic on the
    /// other two — and [`crate::shutdown::Site::restore`] recovers its own for the same reason, one
    /// level up.
    fn lock(&self) -> std::sync::MutexGuard<'_, Shared> {
        self.shared
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// The authoritative size of the terminal: one packed `AtomicU32`.
///
/// **Written by the input thread, sampled by the app thread at frame start and re-checked at
/// submit** (the sixth invariant). Writing a 300x80 frame into a terminal that is now 120x40
/// wraps and scrolls, which is worse than a missing frame — so a frame whose size moved under it is
/// discarded and a full repaint is scheduled, and [`crate::Presented::discarded_for_resize`] is what
/// says so. Cost: one load per frame, one discarded composite per resize.
///
/// **A leased surface never changes size under a drawing caller** — it does not know the terminal
/// exists. That is why this is sampled rather than applied: the surfaces resize when the application
/// reacts to the resize *event*, which is [`crate::Screen::resize`]'s business and the original's to
/// deliver.
///
/// # Who writes it
///
/// The input thread, which owns the read direction and is the only thread that learns about
/// a resize — a `SIGWINCH` arrives there, as an `Event::Resize`. Until then [`TerminalSize::set`]
/// has one caller and it is a gate: the mechanism this ticket owes is the sampling, the re-check and
/// the discard, and those are testable without the signal that will drive them.
///
/// **The app thread reads this and never writes it**, which `Screen::resize` did until the review
/// found the case: a second resize can land while the application is still draining the first event,
/// and an app thread writing back what that event said would put the older size over the newer one —
/// erasing the evidence the discard exists to act on. The surfaces' size is `Screen::size`; the
/// terminal's is not the app thread's to say.
///
/// What this pair does **not** catch is a resize observed *between* two frames, because the sample is
/// the frame's own first statement: a terminal that resized while the application was idle is already
/// the sampled size. See [`crate::Presented::discarded_for_resize`], which names ticket 20 and why the
/// answer needs the resize event to exist first.
#[derive(Debug)]
pub(crate) struct TerminalSize(AtomicU32);

impl TerminalSize {
    /// The size `attach` resolved, before any thread exists.
    pub(crate) fn new((w, h): (u16, u16)) -> TerminalSize {
        TerminalSize(AtomicU32::new(pack_size(w, h)))
    }

    /// One load. `Relaxed` is not enough and `Acquire` is not needed: nothing else is published
    /// alongside this word, so what matters is only that two loads of it in one frame see a
    /// consistent pair — which the packing is what guarantees, not the ordering.
    pub(crate) fn get(&self) -> (u16, u16) {
        let packed = self.0.load(Ordering::Relaxed);
        ((packed >> 16) as u16, (packed & 0xffff) as u16)
    }

    /// One store. **Both halves in one word**, which is the whole reason for the packing: two
    /// atomics could be read as a width from before a resize and a height from after it, and that
    /// pair describes a terminal that never existed.
    ///
    /// **Only the input thread writes this, and the app thread must never be what does**: a resize
    /// belongs to the thread that observed it, and an app thread writing it back from an event it
    /// was still draining would put an older size over a newer one. See `Screen::resize`, which is
    /// where that mistake was, and `crate::input::thread`, which is the one production caller.
    pub(crate) fn set(&self, (w, h): (u16, u16)) {
        self.0.store(pack_size(w, h), Ordering::Relaxed);
    }
}

fn pack_size(w: u16, h: u16) -> u32 {
    (u32::from(w) << 16) | u32::from(h)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leased(mailbox: &Mailbox) -> Box<Packet> {
        match mailbox.lease() {
            Lease::Ready(packet) => packet,
            other => panic!("the pool was full and the renderer free: {other:?}"),
        }
    }

    #[test]
    fn a_fresh_mailbox_is_free_and_holds_the_whole_pool() {
        let mailbox = Mailbox::new();
        assert!(matches!(mailbox.lease(), Lease::Ready(_)));
        assert_eq!(mailbox.painted(), 0);
        assert_eq!(mailbox.superseded(), 0);
        assert_eq!(mailbox.starved(), 0);
    }

    #[test]
    fn a_submitted_packet_leaves_the_renderer_busy_until_it_is_taken() {
        let mailbox = Mailbox::new();
        mailbox.submit(leased(&mailbox));
        assert!(
            matches!(mailbox.lease(), Lease::Busy),
            "the app composited a frame the renderer had not asked for"
        );
        let packet = mailbox.take().expect("a packet landed");
        assert!(
            matches!(mailbox.lease(), Lease::Ready(_)),
            "taking the packet is what frees the renderer"
        );
        mailbox.finish(packet);
        assert_eq!(mailbox.painted(), 1);
    }

    /// The pool is two because the app fills one while the renderer holds the other, and this is
    /// that instant: the renderer has taken a packet and not given it back, and a lease still
    /// answers with a buffer.
    #[test]
    fn the_app_can_fill_a_packet_while_the_renderer_holds_one() {
        let mailbox = Mailbox::new();
        mailbox.submit(leased(&mailbox));
        let in_flight = mailbox.take().expect("a packet landed");
        let filling = leased(&mailbox);
        mailbox.submit(filling);
        mailbox.finish(in_flight);
        assert_eq!(mailbox.starved(), 0, "two packets were enough");
        assert_eq!(mailbox.superseded(), 0);
    }

    /// A third packet in flight is what the pool size makes unreachable, and the counter is how the
    /// unreachable path is proved rather than asserted.
    #[test]
    fn a_lease_with_the_whole_pool_out_starves_rather_than_allocating_a_third() {
        let mailbox = Mailbox::new();
        let first = leased(&mailbox);
        let second = leased(&mailbox);
        assert!(matches!(mailbox.lease(), Lease::Starved));
        assert_eq!(mailbox.starved(), 1);
        mailbox.give_back(first);
        mailbox.give_back(second);
        assert!(matches!(mailbox.lease(), Lease::Ready(_)));
    }

    #[test]
    fn a_submit_onto_a_full_slot_counts_itself() {
        let mailbox = Mailbox::new();
        let first = leased(&mailbox);
        let second = leased(&mailbox);
        mailbox.submit(first);
        // Only reachable by ignoring the lease's answer, which is what the counter is for: nothing
        // in `present` can take this path, and the gate is that the count stays at zero over
        // 10 000 real cycles.
        mailbox.submit(second);
        assert_eq!(mailbox.superseded(), 1);
        assert_eq!(
            mailbox.take().map(|_| ()),
            Some(()),
            "the newest packet is the one in the slot"
        );
    }

    #[test]
    fn giving_a_packet_back_leaves_the_renderer_free() {
        let mailbox = Mailbox::new();
        let packet = leased(&mailbox);
        mailbox.give_back(packet);
        assert!(matches!(mailbox.lease(), Lease::Ready(_)));
        assert_eq!(mailbox.painted(), 0, "a discarded frame was never painted");
    }

    #[test]
    fn take_answers_none_once_quit_is_set_and_the_slot_is_empty() {
        let mailbox = Mailbox::new();
        mailbox.quit();
        assert!(mailbox.take().is_none());
    }

    /// **The last frame is not flushed on quit**. A packet already in the slot is left
    /// there: showing state that is already stale buys nothing and costs exit latency. The same
    /// property as a write count on the wire is
    /// `crate::gates::the_last_frame_is_not_flushed_on_quit`.
    #[test]
    fn a_packet_already_in_the_slot_is_not_written_after_quit() {
        let mailbox = Mailbox::new();
        mailbox.submit(leased(&mailbox));
        mailbox.quit();
        assert!(mailbox.take().is_none());
        assert_eq!(mailbox.painted(), 0);
    }

    #[test]
    fn waiting_for_a_free_renderer_returns_at_once_when_it_already_is() {
        let mailbox = Mailbox::new();
        mailbox.wait_until_free();
    }

    /// The wait is what a real render thread releases, and this is the only test here that has one.
    #[test]
    fn the_wait_returns_when_a_render_thread_takes_the_packet() {
        let mailbox = std::sync::Arc::new(Mailbox::new());
        mailbox.submit(leased(&mailbox));
        let renderer = std::sync::Arc::clone(&mailbox);
        let thread = std::thread::spawn(move || {
            let packet = renderer.take().expect("a packet landed");
            renderer.finish(packet);
        });
        mailbox.wait_until_free();
        thread.join().expect("the render thread does not panic");
        assert_eq!(mailbox.painted(), 1);
        assert_eq!(mailbox.superseded(), 0);
    }

    #[test]
    fn a_packed_size_survives_the_round_trip() {
        let size = TerminalSize::new((300, 80));
        assert_eq!(size.get(), (300, 80));
        size.set((120, 40));
        assert_eq!(size.get(), (120, 40));
        size.set((u16::MAX, u16::MAX));
        assert_eq!(size.get(), (u16::MAX, u16::MAX));
        size.set((0, 0));
        assert_eq!(size.get(), (0, 0));
    }
}
