//! Routing: which events one frame consumes, and the one queue every key is drained from.
//!
//! Two things live here and they are the two halves of
//! the same rule:
//!
//! - [`edge_of`] classifies one event, and [`batch_len`] turns that classification into *how many
//!   events this frame takes*. **A frame consumes at most one routing edge.**
//! - `KeyQueue` is the one queue, private to the crate and reached through
//!   [`crate::ctx::Ctx::next_key`]. **There are no per-id inboxes**, and there is no public shape
//!   that could become one.
//!
//! # "One event a frame" is the wrong rule, in both directions
//!
//! It drops throughput on input that costs nothing to fold — an 8 000-key paste becomes 8 000
//! frames — and it does not protect the case that needs protecting, because two edges in one batch
//! are misrouted whether or not the rest of the batch went with them.
//!
//! The input model already says which events may **collapse**: moves, wheel clicks and ordinary keys. What
//! costs a frame is a *routing edge* — an event whose effect the frame **reads**.
//!
//! # Which end of the batch an edge may sit at
//!
//! Decided by **when its effect is read**, and not by what kind of event it is.
//!
//! - An edge resolved **after** the draw [closes](Edge::Closing) a batch: every event before it was
//!   routed against the state the frame actually drew with.
//! - An edge resolved **before** the draw would [open](Edge::Opening) one, and could therefore only
//!   ever be the *first* event a frame consumes.
//!
//! **Every routing edge that ships today is a closing edge.** `Down`/`Up` resolve in `Frame::end`;
//! `Tab` resolved in `begin` would have been the mirror image, and that was overturned
//! exactly that by declaring the focus ring during the draw at 1.000×–1.009× of the frame — its own
//! noise floor. So `[Key(a), Tab]` is one frame and nothing about focus crosses a frame.
//!
//! The rule keeps both cases even though only one of them is reachable, **because the rule is what
//! makes the classification checkable**: getting it backwards still misroutes a batch, and
//! `tests::the_backwards_classification_splits_a_batch_that_should_not_split` is the negative case.
//! A future edge resolved before the draw is expressible; it is not expressible by accident.

use vitui_engine::{Event, Key, KeyCode, KeyKind, MouseKind};

/// Which end of a batch a routing edge may sit at.
///
/// **Not a severity and not a kind of event** — see the module documentation. The variant is
/// decided by where in the frame sequence the edge's effect is read.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Edge {
    /// Resolved **before** the draw, so it may only be the **first** event a frame consumes.
    ///
    /// **Nothing is classified this way today**, and that is the point of keeping the variant: the
    /// rule is what makes the answer checkable, and a case nobody can express is a case nobody can
    /// get wrong on purpose.
    Opening,
    /// Resolved **after** the draw, so it may only be the **last** event a frame consumes.
    Closing,
}

/// Whether this event is a routing edge, and which end of a batch it may sit at.
///
/// **Everything not named here folds**, which is the majority: ordinary keys, key releases, pointer
/// moves and wheel notches all cost a frame nothing.
///
/// ```
/// use vitui_runtime::route::{Edge, edge_of};
/// use vitui_engine::{Button, Buttons, Event, Key, KeyCode, KeyKind, KeyText, Mods, Mouse, MouseKind};
/// use std::time::Instant;
///
/// let key = |code| Event::Key(Key {
///     code,
///     mods: Mods::NONE,
///     kind: KeyKind::Press,
///     text: KeyText::EMPTY,
///     at: Instant::now(),
/// });
/// assert_eq!(edge_of(&key(KeyCode::Char('a'))), None, "an ordinary key folds");
/// assert_eq!(edge_of(&key(KeyCode::Tab)), Some(Edge::Closing), "Tab moves the focus");
///
/// let mouse = |kind| Event::Mouse(Mouse {
///     x: 0, y: 0, kind, buttons: Buttons::NONE, mods: Mods::NONE, at: Instant::now(),
/// });
/// assert_eq!(edge_of(&mouse(MouseKind::Move)), None, "a position is not intent");
/// assert_eq!(edge_of(&mouse(MouseKind::Down(Button::Left))), Some(Edge::Closing));
/// ```
#[must_use]
pub fn edge_of(event: &Event) -> Option<Edge> {
    match event {
        // **The press award reads these in `end`**, so everything earlier in the batch was routed
        // against the index this frame actually drew.
        Event::Mouse(m) => match m.kind {
            MouseKind::Down(_) | MouseKind::Up(_) => Some(Edge::Closing),
            // ADR 0008: an intermediate position is not intent, and a wheel notch changes nothing
            // routing reads.
            MouseKind::Move | MouseKind::Wheel(_) => None,
        },
        // **`Tab` moves the focus, and the focus is what `next_key` routes against.** A release does
        // not move it, so it folds like any other key.
        Event::Key(k) => match (k.code, k.kind) {
            (KeyCode::Tab | KeyCode::BackTab, KeyKind::Press | KeyKind::Repeat) => {
                Some(Edge::Closing)
            }
            _ => None,
        },
        // A resize changes geometry, and **no geometry crosses a frame** — there is no
        // routing state for it to be an edge in. Focus gain and loss are the terminal window's, not
        // a widget's. A bracketed paste is not delivered by this crate at all yet; a terminal
        // without bracketed paste delivers a pasted megabyte as ordinary keys, which is the case
        // `KeyQueue` is measured against.
        Event::Paste(_) | Event::Resize(_, _) | Event::FocusGained | Event::FocusLost => None,
    }
}

/// How many events of `batch`, from the front, one frame may consume.
///
/// **The batch splitter.** The remainder stays queued for the next frame, and the frame that took a
/// short prefix asks for another one — which is what makes sixteen edges cost sixteen frames with
/// nothing lost.
///
/// ```
/// use vitui_runtime::route::batch_len;
/// # use vitui_engine::{Button, Buttons, Event, Key, KeyCode, KeyKind, KeyText, Mods, Mouse, MouseKind};
/// # use std::time::Instant;
/// # let key = |code| Event::Key(Key { code, mods: Mods::NONE, kind: KeyKind::Press, text: KeyText::EMPTY, at: Instant::now() });
/// // `[Key(a), Tab]` is ONE frame: the key was routed against the focus the frame drew with, and
/// // `Tab` moves it afterwards.
/// assert_eq!(batch_len(&[key(KeyCode::Char('a')), key(KeyCode::Tab)]), 2);
/// // `[Tab, Key(a)]` is two, and neither key is lost.
/// assert_eq!(batch_len(&[key(KeyCode::Tab), key(KeyCode::Char('a'))]), 1);
/// ```
#[must_use]
pub fn batch_len(batch: &[Event]) -> usize {
    batch_len_with(batch, edge_of)
}

/// [`batch_len`] over a classification the caller supplies.
///
/// **This exists for the negative case**, which is the whole reason the two-ended rule is kept: a
/// classification that calls `Tab` an [opening](Edge::Opening) edge splits `[Key(a), Tab]`, and the
/// key then goes to the widget the `Tab` is about to move *to*. Nothing else calls it.
#[must_use]
pub fn batch_len_with(batch: &[Event], classify: impl Fn(&Event) -> Option<Edge>) -> usize {
    let mut n = 0;
    for event in batch {
        match classify(event) {
            // Folds. ADR 0008 measured these at 5.0 ns an event, and a frame takes as many as have
            // arrived.
            None => n += 1,
            // Closes: it is read after the draw, so it is the last thing this frame takes.
            Some(Edge::Closing) => {
                n += 1;
                break;
            }
            // Opens: it is read before the draw, so it may only be the first thing this frame takes.
            // Anything already folded in was routed against the state this edge is about to change.
            Some(Edge::Opening) => {
                if n > 0 {
                    break;
                }
                n += 1;
            }
        }
    }
    n
}

/// The one key queue: one byte a key, **drained at successively outer levels**.
///
/// # There are no per-id inboxes
///
/// Bubbling, accelerators and *unhandled reaches the application* are this one structure. The
/// literal per-id version costs **1.17×** routing and at least one allocation a frame against zero,
/// and its pointer targets come from last frame's index — the stale answer this design exists to
/// remove.
///
/// # Draining is where the quadratic hides
///
/// `next_key` written the obvious way is `Vec::remove(0)`, which shifts the whole tail on every
/// key: an 8 000-key paste is **11.75 ms against 53.79 µs**. The fix is the `at` cursor below, one
/// `usize`, and the queue keeps its allocation across frames because `begin` rewinds rather than
/// dropping it.
///
/// **It is a slope and not a cliff**, so a 100 µs budget gate walks straight past the quadratic
/// form. `touched` is what the growth-ratio gate reads.
///
/// # A decline ends this level's turn, and it has to
///
/// A decline rewinds the cursor, so without something more the obvious drain loop —
/// `while let Some(k) = take() { if !mine(k) { put_back(k) } }` — hands the same key out for ever.
/// It is not an exotic shape: it is the shape the verb's own documentation describes.
///
/// So `put_back` also **closes the queue to this level**, and only a move of the routing target
/// re-opens it. That is the same sentence as *drained at successively outer levels*, and it is what
/// makes the order hold: a level that declined key *k* may not then take *k+1*, because the level
/// above is about to be offered *k* and would see the two out of order.
#[derive(Debug, Default)]
pub(crate) struct KeyQueue {
    /// Every key this frame was given, in arrival order. **Not truncated as it drains** — see
    /// `undrained`, which is how `end` sees what nobody took.
    keys: Vec<Key>,
    /// How many have been handed out. **The whole of the fix**: taking a key moves this and nothing
    /// else.
    at: usize,
    /// **Whether the level currently holding the queue has handed a key back.**
    ///
    /// Set by `put_back` and cleared by `resume`, which is what a bubbling scope calls when the
    /// routing target moves outward. While it is set, `take` answers `None` however many keys are
    /// left — see the type's documentation for why that is the contract and not a limitation.
    declined: bool,
    /// How many key slots this queue has touched since it was made.
    ///
    /// **A count, because the gate is a ratio and a ratio of timings is a report.** The linear form
    /// touches exactly one slot a key; `Vec::remove(0)` touches the whole tail, so the count grows
    /// quadratically and the ratio at 4× the keys is 16× rather than 4×.
    touched: u64,
}

impl KeyQueue {
    /// A queue with room for a burst, so a steady frame never grows it.
    pub(crate) fn new() -> KeyQueue {
        KeyQueue {
            keys: Vec::with_capacity(32),
            at: 0,
            declined: false,
            touched: 0,
        }
    }

    /// Rewind for a new frame. **Keeps the allocation**, which is why a steady frame allocates zero.
    pub(crate) fn begin(&mut self) {
        self.keys.clear();
        self.at = 0;
        self.declined = false;
    }

    /// Re-open the queue, because the routing target has moved outward.
    ///
    /// **The only thing that undoes a decline**, and a bubbling scope's after-the-body moment is
    /// its only caller.
    pub(crate) fn resume(&mut self) {
        self.declined = false;
    }

    /// Post one key, at the back.
    pub(crate) fn push(&mut self, key: Key) {
        self.keys.push(key);
    }

    /// Take the next key, unless this level has already declined one.
    pub(crate) fn take(&mut self) -> Option<Key> {
        if self.declined {
            return None;
        }
        let key = *self.keys.get(self.at)?;
        self.at += 1;
        self.touched += 1;
        Some(key)
    }

    /// What `take` would answer, without taking it.
    ///
    /// **The focus ring is the only caller**, and it needs this because `Tab` is the ring's key and
    /// not the focused widget's: an ordinary widget must not be able to drain the key that is about
    /// to move the focus off it, and the only way to withhold one key is to look at it first. A
    /// scope that *is* isolated gets it, and that decision is taken from the previous frame's scopes
    /// — see [`crate::focus`].
    pub(crate) fn peek(&self) -> Option<Key> {
        if self.declined {
            return None;
        }
        self.keys.get(self.at).copied()
    }

    /// The routing edge nobody drained, if this batch ended with one.
    ///
    /// **Only ever the last key**, which is not a shortcut but the classification: every routing
    /// edge that ships is a *closing* edge, so a `Tab` is the last thing in its batch.
    pub(crate) fn peek_edge(&self) -> Option<Key> {
        let last = self.keys.len().checked_sub(1)?;
        if self.at > last {
            // Somebody took it. An isolated scope inserting a tab character is the case.
            return None;
        }
        let key = self.keys[last];
        matches!(
            (key.code, key.kind),
            (
                KeyCode::Tab | KeyCode::BackTab,
                KeyKind::Press | KeyKind::Repeat
            )
        )
        .then_some(key)
    }

    /// Consume the edge [`KeyQueue::peek_edge`] answered.
    ///
    /// **Two calls and not one, because the ring may decline it.** A frame with no tab stops moves
    /// no focus, and a `Tab` the ring did not use has to reach the application like any other key
    /// nobody took — the outermost level of the one queue is still the application.
    ///
    /// Popping the last slot leaves everything before it exactly where the application's own reader
    /// expects to find it.
    pub(crate) fn drop_edge(&mut self) {
        let Some(last) = self.keys.len().checked_sub(1) else {
            return;
        };
        self.keys.truncate(last);
        self.at = self.at.min(last);
    }

    /// Hand a key back, **in order**: it is the next key the queue answers, to the next level out.
    ///
    /// The contract is *undo the most recent take*, and it composes: two levels declining on the
    /// way out rewind two slots and the queue reads exactly as it did before either took anything.
    ///
    /// **Declining with nothing taken does nothing at all**, which is the only answer that is not
    /// wrong. Whatever key the caller is holding, the queue already contains every key it could be —
    /// it never removed one — so inserting it would duplicate a keystroke, and the application would
    /// process it twice. It would also shift the tail and could reallocate, which is an allocation
    /// during frame composition.
    pub(crate) fn put_back(&mut self, key: Key) {
        // Closed either way: a level that tried to decline is finished with the queue whether or not
        // it had anything to give back.
        self.declined = true;
        if self.at == 0 {
            return;
        }
        self.at -= 1;
        self.keys[self.at] = key;
        self.touched += 1;
    }

    /// What nobody took. **`end` reads this** — a `Tab` no scope claimed is what moves the focus.
    pub(crate) fn undrained(&self) -> &[Key] {
        &self.keys[self.at.min(self.keys.len())..]
    }

    /// How many keys arrived this frame, drained or not.
    pub(crate) fn len(&self) -> usize {
        self.keys.len()
    }

    /// How many key slots this queue has touched since it was made. The growth-ratio gate's number.
    pub(crate) fn touched(&self) -> u64 {
        self.touched
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use vitui_engine::{Button, Buttons, KeyText, Mods, Mouse};

    fn key(code: KeyCode) -> Event {
        Event::Key(Key {
            code,
            mods: Mods::NONE,
            kind: KeyKind::Press,
            text: KeyText::EMPTY,
            at: Instant::now(),
        })
    }

    fn mouse(kind: MouseKind) -> Event {
        Event::Mouse(Mouse {
            x: 0,
            y: 0,
            kind,
            buttons: Buttons::NONE,
            mods: Mods::NONE,
            at: Instant::now(),
        })
    }

    /// **Every routing edge that ships is a closing edge.** The rule has two cases; one of them has
    /// no members, and that is an assertion rather than an omission.
    #[test]
    fn every_routing_edge_that_ships_is_a_closing_edge() {
        let every = [
            key(KeyCode::Tab),
            key(KeyCode::BackTab),
            mouse(MouseKind::Down(Button::Left)),
            mouse(MouseKind::Up(Button::Left)),
            mouse(MouseKind::Down(Button::Right)),
        ];
        for event in &every {
            assert_eq!(
                edge_of(event),
                Some(Edge::Closing),
                "{event:?} is an edge and every edge closes"
            );
        }
    }

    /// Moves, wheel notches and ordinary keys fold: none of them is an edge, so a frame takes as
    /// many as have arrived.
    #[test]
    fn moves_wheel_clicks_and_ordinary_keys_fold() {
        let folds = [
            key(KeyCode::Char('a')),
            key(KeyCode::Enter),
            key(KeyCode::Left),
            key(KeyCode::F(5)),
            mouse(MouseKind::Move),
            mouse(MouseKind::Wheel(vitui_engine::Wheel::Down)),
        ];
        for event in &folds {
            assert_eq!(edge_of(event), None, "{event:?} carries no routing edge");
        }
        assert_eq!(batch_len(&folds), folds.len(), "and they are one frame");
    }

    /// A `Tab` **release** does not move the focus, so it is not an edge — the classification is
    /// about the effect, not about the key cap.
    #[test]
    fn a_tab_release_is_not_an_edge() {
        let release = Event::Key(Key {
            code: KeyCode::Tab,
            mods: Mods::NONE,
            kind: KeyKind::Release,
            text: KeyText::EMPTY,
            at: Instant::now(),
        });
        assert_eq!(edge_of(&release), None);
    }

    /// `[Key(a), Tab]` is **one** frame, and `[Tab, Key(a)]` is two with neither key lost.
    #[test]
    fn a_key_before_a_tab_is_one_frame_and_after_it_is_two() {
        let batch = [key(KeyCode::Char('a')), key(KeyCode::Tab)];
        assert_eq!(
            batch_len(&batch),
            2,
            "the key was routed against this focus"
        );

        let batch = [key(KeyCode::Tab), key(KeyCode::Char('a'))];
        assert_eq!(batch_len(&batch), 1);
        assert_eq!(
            batch_len(&batch[1..]),
            1,
            "and the second frame takes the key"
        );
    }

    /// **The negative case, and the reason [`Edge::Opening`] is kept.**
    ///
    /// Classify `Tab` as opening and `[Key(a), Tab]` splits: the key is routed this frame, the
    /// `Tab` opens the next one — which reads correct and is not. The key went to the widget the
    /// `Tab` was about to move *away from* only by luck of ordering; reverse the batch and it goes
    /// to the one it is about to move *to*.
    #[test]
    fn the_backwards_classification_splits_a_batch_that_should_not_split() {
        let backwards = |event: &Event| match event {
            Event::Key(k) if matches!(k.code, KeyCode::Tab | KeyCode::BackTab) => {
                Some(Edge::Opening)
            }
            other => edge_of(other),
        };
        let batch = [key(KeyCode::Char('a')), key(KeyCode::Tab)];
        assert_eq!(
            batch_len_with(&batch, backwards),
            1,
            "the backwards classification splits what the shipped one keeps whole"
        );
        assert_eq!(batch_len(&batch), 2, "and this is the shipped answer");
    }

    /// An opening edge at the front is taken, and the fold continues behind it — which is the half
    /// of the rule nothing exercises in production.
    #[test]
    fn an_opening_edge_may_be_first_and_only_first() {
        let opening = |_: &Event| Some(Edge::Opening);
        let batch = [key(KeyCode::Char('a')), key(KeyCode::Char('b'))];
        assert_eq!(batch_len_with(&batch, opening), 1);
        assert_eq!(batch_len_with(&batch[1..], opening), 1);
    }

    /// Sixteen edges cost sixteen frames, one each, with nothing folded in and nothing dropped.
    #[test]
    fn sixteen_edges_are_sixteen_batches() {
        let mut batch = Vec::new();
        for _ in 0..8 {
            batch.push(mouse(MouseKind::Down(Button::Left)));
            batch.push(mouse(MouseKind::Up(Button::Left)));
        }
        let mut frames = 0;
        let mut rest = &batch[..];
        while !rest.is_empty() {
            let n = batch_len(rest);
            assert_eq!(n, 1, "an edge takes a frame to itself");
            rest = &rest[n..];
            frames += 1;
        }
        assert_eq!(frames, 16);
    }

    /// An empty batch is an empty frame, which is what makes the drain loop terminate.
    #[test]
    fn an_empty_batch_takes_nothing() {
        assert_eq!(batch_len(&[]), 0);
    }

    fn plain(n: u64) -> Key {
        Key {
            code: KeyCode::Char(
                char::from_u32(u32::try_from(n % 26).unwrap_or(0) + 97).unwrap_or('a'),
            ),
            mods: Mods::NONE,
            kind: KeyKind::Press,
            text: KeyText::EMPTY,
            at: Instant::now(),
        }
    }

    /// **The growth-ratio gate, and it is a ratio because the number belongs to the algorithm.**
    ///
    /// Four times the keys must cost about four times the work. The obvious `Vec::remove(0)` form is
    /// written out beside it and costs sixteen — a slope, not a cliff, which is exactly why the
    /// 100 µs budget gate does not catch it.
    #[test]
    fn draining_an_eight_thousand_key_paste_is_linear() {
        let touched_for = |n: u64| {
            let mut q = KeyQueue::new();
            q.begin();
            for i in 0..n {
                q.push(plain(i));
            }
            while q.take().is_some() {}
            q.touched()
        };
        let small = touched_for(2_000);
        let large = touched_for(8_000);
        let ratio = large as f64 / small as f64;
        assert!(
            ratio <= 5.0,
            "the drain touched {ratio:.2}x the slots for 4x the keys, which is quadratic territory"
        );
        assert_eq!(large, 8_000, "and it is exactly one slot a key");

        // What the obvious version would have been, counted the same way.
        let quadratic_for = |n: u64| {
            let mut keys: Vec<Key> = (0..n).map(plain).collect();
            let mut touched = 0u64;
            while !keys.is_empty() {
                // `Vec::remove(0)` shifts the tail: that is the work, and it is what the count has
                // to include for the ratio to mean anything.
                touched += keys.len() as u64;
                keys.remove(0);
            }
            touched
        };
        let quadratic = quadratic_for(8_000) as f64 / quadratic_for(2_000) as f64;
        assert!(
            quadratic > 10.0,
            "the quadratic twin measured {quadratic:.2}x, so the gate is not detecting anything"
        );
    }

    /// `put_back` hands a key back in order: it is what the **next level out** takes.
    #[test]
    fn a_declined_key_is_the_next_one_the_level_out_takes() {
        let mut q = KeyQueue::new();
        q.begin();
        q.push(plain(0));
        q.push(plain(1));
        let first = q.take().expect("a key");
        q.put_back(first);
        assert!(
            q.take().is_none(),
            "the level that declined is finished with the queue"
        );
        q.resume();
        assert_eq!(q.take().map(|k| k.code), Some(first.code));
        assert_eq!(q.take().map(|k| k.code), Some(plain(1).code));
        assert!(q.take().is_none());
    }

    /// **The obvious drain loop terminates.**
    ///
    /// `while let Some(k) = take() { put_back(k) }` is the shape `Ctx::decline`'s own documentation
    /// describes, and with a bare cursor rewind it hands the same key out for ever. Nothing else in
    /// this module would have caught that: every other test breaks out of the loop by hand.
    #[test]
    fn the_obvious_drain_loop_terminates() {
        let mut q = KeyQueue::new();
        q.begin();
        q.push(plain(0));
        let mut rounds = 0;
        while let Some(k) = q.take() {
            rounds += 1;
            assert!(rounds < 100, "the drain loop did not terminate");
            q.put_back(k);
        }
        assert_eq!(rounds, 1, "one offer, one decline, and the queue closes");
        assert_eq!(
            q.undrained().len(),
            1,
            "and the key is still there for the level out"
        );
    }

    /// Nested declines restore the order rather than reversing it, which is what bubbling out
    /// through two levels does.
    #[test]
    fn two_declines_restore_the_order() {
        let mut q = KeyQueue::new();
        q.begin();
        q.push(plain(0));
        q.push(plain(1));
        let a = q.take().expect("a");
        let b = q.take().expect("b");
        q.put_back(b);
        q.put_back(a);
        assert_eq!(q.undrained().len(), 2, "both are back");
        q.resume();
        assert_eq!(q.take().map(|k| k.code), Some(a.code));
        assert_eq!(q.take().map(|k| k.code), Some(b.code));
    }

    /// **Declining with nothing taken does nothing, and above all does not duplicate.**
    ///
    /// The queue never removed a key, so it already holds every key the caller could be handing
    /// back. Inserting one would make the application process a keystroke twice — and would shift
    /// the tail, which is an allocation during frame composition.
    #[test]
    fn declining_with_nothing_taken_does_not_duplicate() {
        let mut q = KeyQueue::new();
        q.begin();
        q.push(plain(1));
        q.put_back(plain(1));
        assert_eq!(q.undrained().len(), 1, "one key in, one key still queued");
        assert_eq!(q.len(), 1);

        // And the same for a second decline after a real one.
        let mut q = KeyQueue::new();
        q.begin();
        q.push(plain(1));
        let k = q.take().expect("a key");
        q.put_back(k);
        q.put_back(k);
        assert_eq!(q.undrained().len(), 1);
        assert_eq!(q.len(), 1);
    }

    /// What nobody took is still there at `end`, which is how a `Tab` reaches the focus ring.
    #[test]
    fn what_nobody_took_is_still_there() {
        let mut q = KeyQueue::new();
        q.begin();
        q.push(plain(0));
        q.push(plain(1));
        assert_eq!(q.undrained().len(), 2);
        q.take();
        assert_eq!(q.undrained().len(), 1);
        q.take();
        assert!(q.undrained().is_empty());
    }
}
