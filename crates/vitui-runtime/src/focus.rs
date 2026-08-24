//! The focus ring, the three scope kinds, and the rule for a widget that stops drawing.
//!
//! Spec §8; [ADR 0012](../../../docs/adr/) and [ADR 0015](../../../docs/adr/). Two structures live
//! here and they are the two halves of §8:
//!
//! - **the ring** — the tab stops this frame declared, in draw order, each carrying a rectangle in
//!   the enclosing scroll area's content coordinates;
//! - **the scopes** — [`ScopeKind::Group`], [`ScopeKind::Trap`] and [`ScopeKind::Isolated`] as
//!   **frame-local ranges over that ring**, and not a fifth id-keyed cross-frame fact.
//!
//! # What is in the ring, and how far one `Tab` crosses, are separate questions
//!
//! No combination of the pointer's interest bits distinguishes a tab stop from a click target — 76
//! tree rows and 152 table cells on the dense screen are clickable and none is a stop. So
//! [`Interest::FOCUS`](crate::Interest::FOCUS) is a bit of its own and **costs the mouse nothing**:
//! `tracking() == 0`, and the dense screen stays at `Drag`.
//!
//! That answers *what is in the ring* — 312 hit entries become **67** — and leaves the second
//! question, because 67 entries is still 67 `Tab` presses across a menu bar. [`ScopeKind::Group`]
//! collapses the menu bar, the toolbar and the tab strip to one stop each: **43** stops, and
//! 311 / 43 = **7.2×** fewer things a keyboard walkthrough visits.
//!
//! # Three answers, not three degrees
//!
//! - [`Group`](ScopeKind::Group) — one stop for the whole range. What moves *inside* it is the
//!   component's own business, reached through [`Ctx::next_key`](crate::Ctx::next_key) once the
//!   group holds the focus.
//! - [`Trap`](ScopeKind::Trap) — the walk cannot leave, and wraps at the scope's own ends rather
//!   than the ring's. **Its keyboard half is `next_key` itself**: the verb answers nobody but the
//!   focused id, so a trap that has taken the focus has taken the keyboard.
//! - [`Isolated`](ScopeKind::Isolated) — `Tab` is not the ring's; a code editor inserts one.
//!   **It could not have been a `decline`**: by the time a key is declined the ring has already
//!   consumed it and moved the focus, so the decision is taken in the drain, from the **previous**
//!   frame's scopes. Whether the focus sits in an editor is not something this frame's draw changes.
//!
//! # A scope is a range, and the negative case is that it cannot become anything else
//!
//! `start`, `end`, `parent` — built during the draw and cleared by `begin`, so nothing about a scope
//! survives a frame except the ring position it produced. The four id-keyed facts (ADR 0012) are
//! still four: the grab, the press origin, the focus and the click record.
//!
//! # The vanish rule, and the number that made it a structure rather than a loop
//!
//! If the focused id stops drawing, the focus moves to the nearest surviving entry **in the previous
//! frame's ring order** — forward first, then backward, then `None`. Forward first because a row
//! deleted from a list leaves the row that took its place *after* it in reading order, and every
//! other choice walks the focus backwards out of a form when its last field goes.
//!
//! Written the obvious way it walks the previous ring and asks, per candidate, whether that id is
//! still in *this* ring — **and the inner question is a scan**. One row vanishing answers on the
//! first candidate; a filter typed into a search box kills every candidate after the focus at once,
//! and the cost is their number times the ring: **90 002 probes and +32.2 µs on one keystroke, 32%
//! of the budget at 600 rows and quadratic from there.**
//!
//! The fix is a stamped table filled **lazily**, with one pass the first time a frame needs it — so
//! a frame in which nothing vanished pays **zero probes**, and the hostile frame pays the fill plus
//! one probe a candidate. Both halves are reachable, which is what makes the refusal a measurement:
//! see [`Frame::vanish_probes`](crate::ctx::Frame::vanish_probes) and
//! `examples/focus_numbers.rs`.

use vitui_engine::Rect;

use crate::id::Id;

/// What a scope answers about `Tab`.
///
/// **Three answers, not three degrees** — see the module documentation. There is no fourth, and a
/// container that wants only [`Ctx::scope`](crate::Ctx::scope)'s after-the-body moment declares the
/// one that describes what its children are.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScopeKind {
    /// One tab stop for the whole range: the range collapses to its first entry and the rest stay in
    /// the ring, so a component can still move inside it.
    Group,
    /// The walk cannot leave, and wraps at this scope's own ends. A standing trap also pulls the
    /// focus into itself, which is where the focus goes when a modal opens.
    Trap,
    /// The ring never gets the key. The code editor that inserts a tab character.
    Isolated,
}

/// One entry of the focus ring.
///
/// # The rectangle, and why it is not on the hit index
///
/// `rect` is in **the enclosing scroll area's content coordinates**, read at `end` and never across
/// a frame (ADR 0015). Ticket 14's scroll-into-view resolves from the ring that has just drawn —
/// exactly where the press award already is — so the *next* frame is already scrolled, and what
/// crosses the frame boundary is a scroll offset naming no `Rect`.
///
/// **The hit index is a different structure and stays at sixteen bytes with no rectangle.** The two
/// questions are not the same one: containment is decided during the draw in the widget's own
/// coordinates, and no rectangle helps.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Stop {
    /// Whose.
    pub id: Id,
    /// Where, in the enclosing scroll area's content coordinates.
    pub rect: Rect,
    /// The innermost scope open when it was declared, as an index into the frame's scope list.
    pub scope: Option<u32>,
    /// The innermost **scroll area** open when it was declared, as an index into the frame's list of
    /// them — which is what makes [`rect`](Stop::rect) mean something at `end`.
    ///
    /// An index and not an [`Id`], because the id alone would need a scan and the record `end` reads
    /// holds the viewport and the offset beside it. `None` for a stop outside every area, which asks
    /// for nothing because there is nothing to move.
    pub area: Option<u32>,
}

/// A scope, as a **frame-local range over this frame's ring**.
///
/// Not a fifth id-keyed cross-frame fact: it is built during the draw and cleared by the next
/// `begin`. See the module documentation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ScopeRec {
    /// The id the scope was opened under. **A scope renames nothing**, so this is the one id a
    /// scoping container adds: its own.
    pub id: Id,
    /// What it answers about `Tab`.
    pub kind: ScopeKind,
    /// The first ring index inside it.
    pub start: usize,
    /// One past the last ring index inside it.
    pub end: usize,
    /// The enclosing scope, if any.
    pub parent: Option<u32>,
}

/// One slot of the ring's lazily filled stamped table.
#[derive(Clone, Copy, Debug, Default)]
struct Seen {
    id: u64,
    stamp: u32,
}

/// The focus ring and this frame's scopes, plus the previous frame's copy of both.
///
/// **One more swapped buffer**, which is the whole storage cost of the vanish rule: the previous
/// ring keeps its allocation and a steady frame allocates nothing.
#[derive(Debug)]
pub(crate) struct Ring {
    /// This frame's stops, in draw order.
    now: Vec<Stop>,
    /// The previous frame's, kept for the vanish rule.
    prev: Vec<Stop>,
    /// This frame's scopes, in the order they opened.
    scopes: Vec<ScopeRec>,
    /// The previous frame's, kept for the two questions that must be answered **before** this
    /// frame's draw has produced any: whether the focus is isolated, and whether a trap stood.
    prev_scopes: Vec<ScopeRec>,
    /// The innermost scope open right now. Only meaningful during the draw.
    open: Option<u32>,
    /// Where the focused id sat in `now`, recorded as it was pushed.
    ///
    /// **This is what keeps the vanish rule from opening with a scan.** The id that vanished does
    /// not draw this frame, so its position can only come from the frame in which it did.
    at: Option<usize>,
    /// The previous frame's [`Ring::at`].
    prev_at: Option<usize>,
    /// The stamped table over `now`, filled lazily. See the module documentation.
    seen: Vec<Seen>,
    /// Which frame `seen` was filled for; `0` means *not filled*.
    seen_stamp: u32,
    /// This frame's number, for `seen_stamp` to compare against.
    stamp: u32,
    /// How many slots the vanish rule has touched this frame. **A count, because the gate is a
    /// ratio and a ratio of timings is a report.**
    probes: u64,
}

impl Ring {
    /// A ring with room for a dense screen's 67 stops without growing.
    pub(crate) fn new() -> Ring {
        Ring {
            now: Vec::with_capacity(128),
            prev: Vec::with_capacity(128),
            scopes: Vec::with_capacity(16),
            prev_scopes: Vec::with_capacity(16),
            open: None,
            at: None,
            prev_at: None,
            seen: vec![Seen::default(); 256],
            seen_stamp: 0,
            stamp: 1,
            probes: 0,
        }
    }

    /// Start a frame: **swap and clear**, so both buffers keep their allocation.
    pub(crate) fn begin(&mut self) {
        std::mem::swap(&mut self.now, &mut self.prev);
        std::mem::swap(&mut self.scopes, &mut self.prev_scopes);
        std::mem::swap(&mut self.at, &mut self.prev_at);
        self.now.clear();
        self.scopes.clear();
        self.at = None;
        self.open = None;
        self.probes = 0;
        self.stamp = self.stamp.wrapping_add(1);
        if self.stamp == 0 {
            // The same wrap the id table takes, and for the same reason: a slot stamped `u32::MAX`
            // would read as this frame's again.
            for slot in &mut self.seen {
                *slot = Seen::default();
            }
            self.stamp = 1;
        }
        self.seen_stamp = 0;
    }

    /// Append a stop. `focused` says whether this is the id that currently holds the focus.
    pub(crate) fn push(&mut self, id: Id, rect: Rect, area: Option<u32>, focused: bool) {
        if focused {
            self.at = Some(self.now.len());
        }
        self.now.push(Stop {
            id,
            rect,
            scope: self.open,
            area,
        });
    }

    /// Open a scope, returning its index so the caller can close it.
    pub(crate) fn open_scope(&mut self, id: Id, kind: ScopeKind) -> u32 {
        let ix = u32::try_from(self.scopes.len()).unwrap_or(u32::MAX);
        self.scopes.push(ScopeRec {
            id,
            kind,
            start: self.now.len(),
            end: self.now.len(),
            parent: self.open,
        });
        self.open = Some(ix);
        ix
    }

    /// Close the scope `open_scope` returned.
    pub(crate) fn close_scope(&mut self, ix: u32) {
        let Some(rec) = self.scopes.get_mut(ix as usize) else {
            return;
        };
        rec.end = self.now.len();
        self.open = rec.parent;
    }

    /// The innermost scope open right now.
    pub(crate) fn open(&self) -> Option<u32> {
        self.open
    }

    /// The scope enclosing `ix`.
    pub(crate) fn parent_of(&self, ix: u32) -> Option<u32> {
        self.scopes.get(ix as usize).and_then(|s| s.parent)
    }

    /// This frame's stops.
    pub(crate) fn stops(&self) -> &[Stop] {
        &self.now
    }

    /// This frame's scopes.
    pub(crate) fn scopes(&self) -> &[ScopeRec] {
        &self.scopes
    }

    /// How many slots the vanish rule has touched this frame. **Zero on a quiet frame.**
    pub(crate) fn probes(&self) -> u64 {
        self.probes
    }

    /// Where an id sits in this frame's ring.
    ///
    /// **The recorded position first**, which is the answer on every frame where the focus simply
    /// drew again; the scan behind it is reached only after a vanish or an explicit
    /// [`Ctx::focus`](crate::Ctx::focus), which are the two moments the recorded position is about
    /// somebody else.
    pub(crate) fn position(&self, id: Id) -> Option<usize> {
        if let Some(i) = self.at
            && self.now.get(i).is_some_and(|s| s.id == id)
        {
            return Some(i);
        }
        self.now.iter().position(|s| s.id == id)
    }

    /// Record where the focus ended this frame, so the vanish rule can start without a scan.
    ///
    /// **The recorded position is written as the ring is built**, which is the answer whenever the
    /// focus was already on that widget when it drew. The three moments it is not — the press award,
    /// a `Tab`, and a [`Ctx::focus`](crate::Ctx::focus) made *after* the widget drew — all land in
    /// `end`, after the ring is complete, and this is the one lookup that closes them.
    ///
    /// It runs **only when the recorded position is about somebody else**, so a steady frame does
    /// nothing at all, and it is one pass rather than the vanish rule's per-candidate question: the
    /// two are different costs and only the second is what 90 002 was.
    pub(crate) fn note(&mut self, focused: Option<Id>) {
        match focused {
            None => self.at = None,
            Some(id) => {
                if !self
                    .at
                    .is_some_and(|i| self.now.get(i).is_some_and(|s| s.id == id))
                {
                    self.at = self.now.iter().position(|s| s.id == id);
                }
            }
        }
    }

    /// The innermost scope of a given kind enclosing a ring index.
    fn enclosing(&self, ix: usize, kind: ScopeKind) -> Option<u32> {
        let mut at = self.now.get(ix)?.scope;
        while let Some(s) = at {
            let rec = self.scopes.get(s as usize)?;
            if rec.kind == kind {
                return Some(s);
            }
            at = rec.parent;
        }
        None
    }

    /// Whether a ring index is a tab stop, which is *not* the same as being in the ring.
    ///
    /// Everything inside a [`Group`](ScopeKind::Group) collapses onto the group's first entry.
    fn is_stop(&self, ix: usize) -> bool {
        match self.enclosing(ix, ScopeKind::Group) {
            None => true,
            Some(g) => self.scopes[g as usize].start == ix,
        }
    }

    /// The tab stop a ring index belongs to.
    fn stop_of(&self, ix: usize) -> usize {
        match self.enclosing(ix, ScopeKind::Group) {
            None => ix,
            Some(g) => self.scopes[g as usize].start,
        }
    }

    /// The stops of this frame's ring, in ring order.
    pub(crate) fn stop_indices(&self) -> impl Iterator<Item = usize> + '_ {
        (0..self.now.len()).filter(|&ix| self.is_stop(ix))
    }

    /// The half-open ring range a walk from `from` may visit.
    ///
    /// The whole ring, unless the walk starts inside a [`Trap`](ScopeKind::Trap) — in which case it
    /// is that trap's range, and the wrap happens at its ends.
    fn bounds(&self, from: Option<usize>) -> (usize, usize) {
        let whole = (0, self.now.len());
        let Some(ix) = from else { return whole };
        match self.enclosing(ix, ScopeKind::Trap) {
            None => whole,
            Some(t) => {
                let rec = self.scopes[t as usize];
                (rec.start, rec.end)
            }
        }
    }

    /// Where one `Tab` — or one `Shift-Tab` — lands.
    ///
    /// **Private to the crate, deliberately.** A component that could call it would move the focus
    /// during the draw, over a ring that is half built: the stale-ring defect this ticket removed,
    /// reintroduced from above. The only focus verb is [`Ctx::focus`](crate::Ctx::focus), which
    /// names a widget rather than a direction.
    pub(crate) fn advance(&self, from: Option<usize>, forward: bool) -> Option<Id> {
        let (lo, hi) = self.bounds(from);
        let len = hi.checked_sub(lo).filter(|n| *n > 0)?;
        // Nothing focused starts one off the end, so the first step lands on the first stop — or on
        // the last one, walking backwards.
        let start = match from.filter(|ix| (lo..hi).contains(ix)) {
            Some(ix) => self.stop_of(ix),
            None if forward => hi - 1,
            None => lo,
        };
        // Clamped, because a trap nested inside a group has a collapsed stop before its own range.
        let offset = start.saturating_sub(lo).min(len - 1);
        for step in 1..=len {
            let ix = if forward {
                lo + (offset + step) % len
            } else {
                lo + (offset + len - step % len) % len
            };
            if self.is_stop(ix) {
                return Some(self.now[ix].id);
            }
        }
        // One stop in the range: `Tab` stays where it is rather than answering nothing.
        self.is_stop(start).then(|| self.now[start].id)
    }

    /// The order a keyboard walkthrough visits, starting after the current focus.
    ///
    /// A gate reads this; the walkthrough itself presses `Tab`, because a walk that consults the
    /// same function the frame uses would be testing one expression against itself.
    pub(crate) fn walk(&self) -> impl Iterator<Item = Id> + '_ {
        let (lo, hi) = self.bounds(self.at);
        let len = hi.saturating_sub(lo);
        let start = match self.at.filter(|ix| (lo..hi).contains(ix)) {
            Some(ix) => self
                .stop_of(ix)
                .saturating_sub(lo)
                .min(len.saturating_sub(1)),
            None => len.saturating_sub(1),
        };
        // An empty range yields an empty iterator, so the modulo below never divides by zero.
        (1..=len).filter_map(move |step| {
            let ix = lo + (start + step) % len;
            self.is_stop(ix).then(|| self.now[ix].id)
        })
    }

    /// The ids of the traps standing this frame.
    ///
    /// **An iterator and not a `Vec`**: three components tickets read this to name the exception
    /// their keyboard-walkthrough gate carries, and a `collect` here would be one allocation a frame
    /// against a budget of zero.
    pub(crate) fn trap_ids(&self) -> impl Iterator<Item = Id> + '_ {
        self.scopes
            .iter()
            .filter(|s| s.kind == ScopeKind::Trap)
            .map(|s| s.id)
    }

    /// The first stop of the innermost trap standing this frame, and the trap's own id.
    ///
    /// **A standing trap pulls the focus into itself**, which is also the answer to *where does the
    /// focus go when a modal opens*.
    pub(crate) fn trap_pull(&self, focused: Option<Id>) -> Option<Id> {
        // Innermost is last: a nested trap opens after the one enclosing it.
        let (ix, rec) = self
            .scopes
            .iter()
            .enumerate()
            .rev()
            .find(|(_, s)| s.kind == ScopeKind::Trap)?;
        // Already inside it, by id rather than by position: the focused widget may be the trap's
        // own scope id.
        if let Some(id) = focused
            && (rec.id == id
                || self.now[rec.start..rec.end].iter().any(|s| s.id == id)
                || self.nested_in(ix, id))
        {
            return None;
        }
        (rec.start..rec.end)
            .find(|&i| self.is_stop(i))
            .map(|i| self.now[i].id)
    }

    /// Whether `id` names a scope nested inside scope `ix`.
    fn nested_in(&self, ix: usize, id: Id) -> bool {
        self.scopes.iter().any(|s| {
            if s.id != id {
                return false;
            }
            let mut at = s.parent;
            while let Some(p) = at {
                if p as usize == ix {
                    return true;
                }
                at = self.scopes[p as usize].parent;
            }
            false
        })
    }

    /// Whether the focus sat inside an [`Isolated`](ScopeKind::Isolated) scope **last** frame.
    ///
    /// Answered from the previous frame's scopes because the drain happens before this frame's draw
    /// has declared any — and legitimately so: whether the focus sits in an editor is not something
    /// this frame's draw changes.
    pub(crate) fn was_isolated(&self, focused: Option<Id>) -> bool {
        let Some(id) = focused else { return false };
        self.prev_scopes.iter().any(|s| {
            s.kind == ScopeKind::Isolated
                && (s.id == id || self.prev[s.start..s.end].iter().any(|e| e.id == id))
        })
    }

    /// Whether a key may be delivered to `id`, given the trap that stood **last** frame.
    ///
    /// The second of the trap's two keyboard mechanisms: `trap_pull` covers every frame after the
    /// one a trap opens on, and this covers an explicit [`Ctx::focus`](crate::Ctx::focus) that would
    /// otherwise hand the keyboard back to a widget behind the modal.
    ///
    /// **Neither reaches the frame the trap opens, and the reason is structural**: widgets behind a
    /// modal draw *before* it, so they pull their keys before the scope that would have stopped them
    /// exists, and the previous frame has no trap in it to ask about.
    pub(crate) fn prev_delivers_to(&self, id: Id) -> bool {
        let Some((ix, rec)) = self
            .prev_scopes
            .iter()
            .enumerate()
            .rev()
            .find(|(_, s)| s.kind == ScopeKind::Trap)
        else {
            return true;
        };
        rec.id == id
            || self.prev[rec.start..rec.end].iter().any(|s| s.id == id)
            || self.prev_scopes.iter().any(|s| {
                if s.id != id {
                    return false;
                }
                let mut at = s.parent;
                while let Some(p) = at {
                    if p as usize == ix {
                        return true;
                    }
                    at = self.prev_scopes[p as usize].parent;
                }
                false
            })
    }

    /// **The vanish rule.** Where the focus goes when the id holding it stops drawing.
    ///
    /// Forward first, then backward, then `None` — see the module documentation for why forward
    /// first, and for the number that made the membership question a table rather than a scan.
    ///
    /// `None` here is *absence*, and it is the answer only when nothing in the previous ring
    /// survived. **The award's `None` is intent and never reaches this function**, because a press
    /// that landed on nothing leaves the focus `None` and there is no id to have vanished.
    pub(crate) fn vanished(&mut self, gone: Id) -> Option<Id> {
        // The position comes from the frame in which it drew, recorded as it was pushed — the id is
        // not in this frame's ring, and looking it up in the previous one would open the rule with
        // the scan it exists to avoid.
        let at = self
            .prev_at
            .filter(|&i| self.prev.get(i).is_some_and(|s| s.id == gone))?;
        for i in at + 1..self.prev.len() {
            let id = self.prev[i].id;
            if self.holds(id) {
                return Some(id);
            }
        }
        for i in (0..at).rev() {
            let id = self.prev[i].id;
            if self.holds(id) {
                return Some(id);
            }
        }
        None
    }

    /// Whether an id is in **this** frame's ring, through the lazily filled table.
    fn holds(&mut self, id: Id) -> bool {
        self.fill();
        let len = self.seen.len();
        let mut at = usize::try_from(id.raw() % len as u64).unwrap_or(0);
        for _ in 0..len {
            self.probes += 1;
            let slot = self.seen[at];
            if slot.stamp != self.stamp {
                return false;
            }
            if slot.id == id.raw() {
                return true;
            }
            at = (at + 1) % len;
        }
        false
    }

    /// Fill the table, **once, and only if something asks**.
    ///
    /// This is what makes a frame in which nothing vanished cost zero: the fill is the first thing
    /// the vanish rule does and the only thing that walks this frame's ring.
    fn fill(&mut self) {
        if self.seen_stamp == self.stamp {
            return;
        }
        // **A quarter full, where the id table runs at two thirds**, and the difference is what the
        // two tables are asked: the id table answers *claimed* for ids it mostly contains, and this
        // one answers *gone* for candidates it mostly does not. An unsuccessful lookup in an
        // open-addressed table costs 1/(1-α)² against a successful one's 1/(1-α), so the load factor
        // that is right for the first question is the wrong one here — and this table is a screenful
        // of stops, where a quarter-full is kilobytes.
        let want = (self.now.len() * 4 + 1).next_power_of_two().max(256);
        if self.seen.len() < want {
            self.seen = vec![Seen::default(); want];
        }
        let len = self.seen.len();
        for i in 0..self.now.len() {
            let raw = self.now[i].id.raw();
            let mut at = usize::try_from(raw % len as u64).unwrap_or(0);
            for _ in 0..len {
                self.probes += 1;
                if self.seen[at].stamp != self.stamp {
                    self.seen[at] = Seen {
                        id: raw,
                        stamp: self.stamp,
                    };
                    break;
                }
                if self.seen[at].id == raw {
                    break;
                }
                at = (at + 1) % len;
            }
        }
        self.seen_stamp = self.stamp;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ring_of(n: u64) -> Ring {
        let mut r = Ring::new();
        r.begin();
        for i in 0..n {
            r.push(
                Id::from_raw(i + 1),
                Rect::new(0, i as i32, 4, 1),
                None,
                false,
            );
        }
        r
    }

    /// A ring with no scopes is one stop an entry, in draw order.
    #[test]
    fn every_entry_is_a_stop_when_nothing_groups_them() {
        let r = ring_of(5);
        assert_eq!(r.stop_indices().count(), 5);
        assert_eq!(r.advance(Some(0), true), Some(Id::from_raw(2)));
        assert_eq!(r.advance(Some(4), true), Some(Id::from_raw(1)), "it wraps");
        assert_eq!(r.advance(Some(0), false), Some(Id::from_raw(5)));
        assert_eq!(r.advance(None, true), Some(Id::from_raw(1)));
        assert_eq!(r.advance(None, false), Some(Id::from_raw(5)));
    }

    /// **A group is one stop for its range, and the rest stay in the ring.**
    #[test]
    fn a_group_collapses_its_range_to_one_stop() {
        let mut r = Ring::new();
        r.begin();
        r.push(Id::from_raw(1), Rect::default(), None, false);
        let g = r.open_scope(Id::named("menu bar"), ScopeKind::Group);
        for i in 2..=8u64 {
            r.push(Id::from_raw(i), Rect::default(), None, false);
        }
        r.close_scope(g);
        r.push(Id::from_raw(9), Rect::default(), None, false);

        assert_eq!(r.stops().len(), 9, "every entry is still in the ring");
        assert_eq!(r.stop_indices().count(), 3, "and three of them are stops");
        assert_eq!(r.advance(Some(0), true), Some(Id::from_raw(2)));
        assert_eq!(
            r.advance(Some(4), true),
            Some(Id::from_raw(9)),
            "one Tab crosses the whole bar"
        );
        assert_eq!(
            r.advance(Some(8), false),
            Some(Id::from_raw(2)),
            "and back onto its first entry"
        );
    }

    /// **A trap wraps at its own ends**, and one `Tab` never leaves it.
    #[test]
    fn a_trap_wraps_at_its_own_ends() {
        let mut r = Ring::new();
        r.begin();
        r.push(Id::from_raw(1), Rect::default(), None, false);
        let t = r.open_scope(Id::named("modal"), ScopeKind::Trap);
        for i in 2..=4u64 {
            r.push(Id::from_raw(i), Rect::default(), None, false);
        }
        r.close_scope(t);
        r.push(Id::from_raw(5), Rect::default(), None, false);

        // Inside, it walks as any range does.
        assert_eq!(r.advance(Some(1), true), Some(Id::from_raw(3)));
        // **The last entry of the trap wraps to the trap's first**, not to the ring's.
        assert_eq!(r.advance(Some(3), true), Some(Id::from_raw(2)));
        assert_eq!(r.advance(Some(1), false), Some(Id::from_raw(4)));
        // And nothing outside it is ever reachable: five presses from inside stay inside.
        let mut at = Some(1usize);
        for _ in 0..5 {
            let id = r.advance(at, true).expect("a stop inside the trap");
            assert!(
                [2, 3, 4].contains(&id.raw()),
                "the walk left the trap and landed on {}",
                id.raw()
            );
            at = r.stops().iter().position(|s| s.id == id);
        }
    }

    /// The vanish rule walks forward first, then backward.
    #[test]
    fn forward_first_then_backward_then_none() {
        // Frame one: five stops, the third focused.
        let mut r = Ring::new();
        r.begin();
        for i in 0..5u64 {
            r.push(Id::from_raw(i + 1), Rect::default(), None, i == 2);
        }
        // Frame two: the third and the fourth are gone.
        r.begin();
        for i in [0u64, 1, 4] {
            r.push(Id::from_raw(i + 1), Rect::default(), None, false);
        }
        assert_eq!(
            r.vanished(Id::from_raw(3)),
            Some(Id::from_raw(5)),
            "forward first, over the fourth which also went"
        );

        // Frame three: only what is before it survives.
        r.begin();
        for i in 0..5u64 {
            r.push(Id::from_raw(i + 1), Rect::default(), None, i == 2);
        }
        r.begin();
        for i in [0u64, 1] {
            r.push(Id::from_raw(i + 1), Rect::default(), None, false);
        }
        assert_eq!(
            r.vanished(Id::from_raw(3)),
            Some(Id::from_raw(2)),
            "then backward, to the nearest"
        );

        // Frame four: nothing survives.
        r.begin();
        for i in 0..5u64 {
            r.push(Id::from_raw(i + 1), Rect::default(), None, i == 2);
        }
        r.begin();
        assert_eq!(r.vanished(Id::from_raw(3)), None, "and then absence");
    }

    /// **A quiet frame pays zero probes**, which is what makes the lazy fill worth having.
    #[test]
    fn a_quiet_frame_pays_no_probes() {
        let mut r = ring_of(600);
        assert_eq!(r.probes(), 0);
        r.begin();
        for i in 0..600u64 {
            r.push(Id::from_raw(i + 1), Rect::default(), None, false);
        }
        assert_eq!(r.probes(), 0, "nothing asked, so nothing was filled");
    }

    /// The stamped table answers membership in about one probe, and the fill happens once.
    #[test]
    fn the_table_is_filled_once_and_answers_in_about_one_probe() {
        let mut r = Ring::new();
        r.begin();
        for i in 0..300u64 {
            r.push(Id::from_raw(i + 1), Rect::default(), None, i == 299);
        }
        r.begin();
        for i in 0..300u64 {
            r.push(Id::from_raw(i + 1), Rect::default(), None, i == 299);
        }
        assert_eq!(r.probes(), 0, "it drew again, so nothing asked");

        // Now a frame it does not draw in.
        r.begin();
        for i in 0..299u64 {
            r.push(Id::from_raw(i + 1), Rect::default(), None, false);
        }
        assert_eq!(r.vanished(Id::from_raw(300)), Some(Id::from_raw(299)));
        let once = r.probes();
        assert!(
            (299..=307).contains(&once),
            "the fill is one pass over 299 entries plus about one probe for the candidate, not {once}"
        );
    }
}
