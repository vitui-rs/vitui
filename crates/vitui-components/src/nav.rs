//! **F8 navigation**, ~35 entries, expressed by `collection` + `overlay`, and **no v1 component of
//! its own** — plus the [`cursor`], the helper that decides *what a `Group` moves with*.
//!
//! The reduction is R1 and R3: the `Mode` absorbs tabs, the content switcher, the menu
//! bar, the submenu and the command palette body, and the two axes absorb the popup that carries
//! them. The dock is v2.
//!
//! **An empty module is a claim and it is checked.** [`MEMBERS`] being empty is counted by
//! `inventory::tests::the_module_tree_and_the_families_column_agree`, which runs the join in both
//! directions. The one entry this family owes a ticket rather than a reduction is `command
//! palette` — R3 over
//! Two families with **no mechanism named as new**, which needs a decision rather than an
//! assertion. It is one of the two exemplars that were not built.
//!
//! # `nav::cursor`'s placement is the decision, not its contents
//!
//! Arrows, `Home`/`End`, `PageUp`/`PageDown` and type-ahead are four lines of arithmetic that every
//! list would otherwise write for itself. What the helper decides is **where they live**: in the
//! family module, called by a collection that has opened its own
//! [`Group`](vitui_runtime::focus::ScopeKind::Group) scope — so *a list is one tab stop* is true of
//! every list rather than of the lists whose author remembered. The number is:
//! **266 tab stops become 69** on C02's screen. [`STOPS_UNGROUPED`] and [`STOPS_GROUPED`] are what
//! that measurement is on the screen this crate can actually stand up, and [`fixture`]'s
//! documentation says why the two are not the same screen.
//!
//! **A `Group` keeps everything in the ring and collapses only the walk**, which is what makes the
//! helper coherent: `Tab` reaches the list once, and once inside it the arrows move over entries the
//! ring still knows about. [`stops`] reports all three counts side by side for exactly that reason —
//! a reading that only took the walk would report a list whose contents had vanished.
//!
//! # The deadline is `cx.deadline_for`, and the `id` is not what attributes it
//!
//! Type-ahead needs one deadline or **the buffer expires at the next keypress — the keypress whose
//! meaning depends on it.** Runtime 06 attributes every deadline to its call site under both verbs;
//! what the `id` buys is the *census*, [`WakeLedger::asked_by`](vitui_runtime::anim::WakeLedger::asked_by),
//! the counter that proves which widget asked. A collection has an id to count against, so
//! type-ahead takes the `_for` half.
//!
//! The distinction is measured rather than asserted:
//! `tests::the_id_is_the_census_and_not_the_attribution`
//! runs both verbs on fresh drivers and reads **1 against 0** out of the census, with the same
//! wakeup arriving either way. That is the ticket's own correction to itself, as a number.

use std::time::{Duration, Instant};

use vitui_runtime::ctx::{Ctx, Driver};
use vitui_runtime::focus::ScopeKind;
use vitui_runtime::keys::{Code, Edge, Pressed};
use vitui_runtime::layout::{Col, Constraint::Weight};
use vitui_runtime::{Id, Interest};

use crate::keys::{is_chord, text};

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &[];

// ── the cursor ───────────────────────────────────────────────────────────────────────────────────

/// Where a group's cursor is, how many entries it moves over, and how far a page is.
///
/// **A page is a parameter and not a constant**, because it is the viewport's height in rows and
/// only the caller knows that. A helper that guessed would be laying something out, which is the one
/// thing no layer of this library does for its caller.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cursor {
    /// Which entry the cursor is on.
    pub at: usize,
    /// How many entries there are.
    pub len: usize,
    /// How many entries `PageUp` and `PageDown` move over.
    pub page: usize,
}

impl Cursor {
    /// A cursor at the top of `len` entries, paging by `page`.
    pub const fn new(len: usize, page: usize) -> Cursor {
        Cursor { at: 0, len, page }
    }

    /// The same cursor, moved to `at` and clamped.
    #[must_use]
    pub const fn to(self, at: usize) -> Cursor {
        Cursor {
            at: if self.len == 0 {
                0
            } else if at >= self.len {
                self.len - 1
            } else {
                at
            },
            ..self
        }
    }

    /// The last entry, or `0` when there are none.
    pub const fn last(self) -> usize {
        match self.len {
            0 => 0,
            n => n - 1,
        }
    }
}

/// **Arrows, `Home`/`End`, `PageUp`/`PageDown`.** The new index, or `None` when the key is not the
/// group's.
///
/// # A chord is not a cursor key either, and it is the same predicate
///
/// `Ctrl+Home` is an accelerator and `Home` is a cursor key, and the difference is
/// [`crate::keys::is_chord`] — the same one line that stops a field eating `Ctrl+S`. Stated once and
/// read twice rather than written twice: without it a list swallows every accelerator exactly as an
/// unguarded field does, and the defect has a different symptom and the same cause.
///
/// **Shift passes**, for [`SIGNIFICANT`](crate::keys::SIGNIFICANT)'s reason: `Shift+Down` is a range
/// selection, which is the group's business and not the application's.
///
/// # A release is not a cursor key, and the terminal sends one
///
/// **The engine pushes kitty flag 31, and bit 2 of that is *report event types*** — so on a terminal
/// that speaks the protocol every arrow arrives twice, once as a press and once as a release, and a
/// helper that read only `code` moved the cursor two rows for one keystroke. It is invisible on a
/// legacy terminal, which reports one edge, and that is why it shipped: the same defect
/// `vitui_runtime::keys::Chord::matches` refuses in its first three lines, arriving here because
/// this helper does not go through a `Chord`.
///
/// [`crate::keys`] already refuses a release in three places; this is the fourth, and it is the one
/// a component reaches for when it has no key map at all.
///
/// # Both directions clamp and neither wraps
///
/// A cursor that wrapped would put `Down` at the bottom of a list back at the top, which is what the
/// ring does at the ends of the *walk* — and a list that did the same makes `Tab` and `Down`
/// indistinguishable to a user who cannot see where either one went. The ring wraps because it has
/// somewhere to wrap to; a group's cursor stops.
pub fn step(k: &Pressed, cur: Cursor) -> Option<usize> {
    if cur.len == 0 || k.kind == Edge::Release || is_chord(k) {
        return None;
    }
    let last = cur.last();
    let moved = match k.code {
        Code::Up | Code::Left => cur.at.saturating_sub(1),
        Code::Down | Code::Right => (cur.at + 1).min(last),
        Code::Home => 0,
        Code::End => last,
        Code::PageUp => cur.at.saturating_sub(cur.page),
        Code::PageDown => (cur.at + cur.page).min(last),
        // `Code` is `#[non_exhaustive]`, so this arm is required rather than tidy.
        _ => return None,
    };
    Some(moved)
}

/// How long a type-ahead buffer stands before it lapses. **One second.**
///
/// The same number as [`KeyMap::TIMEOUT`](vitui_runtime::keys::KeyMap::TIMEOUT) and **not the same
/// decision**, which is why it is a constant here rather than a reference to that one. Both answer
/// *has the user stopped typing*, and a component may not reach into the runtime's constant for a
/// different question — the day one of the two moves, the other must be able to stay.
///
/// A second is also where the two agree with the shipped corpus of muscle memory: vim's `timeoutlen`
/// on one side, and a second on the other in every file manager that has type-ahead.
pub const WINDOW: Duration = Duration::from_millis(1000);

/// **A type-ahead buffer and the one deadline that keeps it alive.**
///
/// The state a group carries across frames, and it is the only state in this module. Kept by the
/// caller rather than by the runtime, on `Pending`'s arrangement one crate down: nothing is keyed by
/// an identity, so nothing needs sweeping.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct TypeAhead {
    buf: String,
    since: Option<Instant>,
}

impl TypeAhead {
    /// An empty buffer.
    pub fn new() -> TypeAhead {
        TypeAhead::default()
    }

    /// What has been typed, or `""`.
    pub fn buffer(&self) -> &str {
        &self.buf
    }

    /// Whether anything is standing.
    pub fn is_standing(&self) -> bool {
        self.since.is_some()
    }

    /// **The one wakeup a standing buffer owes**, and `None` when nothing is standing.
    ///
    /// One deadline per buffer — not one per frame and not one per key — discharged when the buffer
    /// lapses. `Pending::deadline`'s shape, one crate down, for its reason.
    pub fn deadline(&self) -> Option<Instant> {
        self.since.map(|since| since + WINDOW)
    }

    /// Drop the buffer if it has lapsed. Answers whether it did.
    ///
    /// **Called at the top of a frame and not at the arrival of a key**, which is the whole of what
    /// the deadline is for: expiring lazily makes the keypress whose meaning depends on the buffer
    /// the same keypress that clears it.
    pub fn expire(&mut self, now: Instant) -> bool {
        match self.deadline() {
            Some(at) if now >= at => {
                self.clear();
                true
            }
            _ => false,
        }
    }

    /// Add a character and restart the window.
    pub fn push(&mut self, c: char, now: Instant) {
        self.buf.push(c);
        self.since = Some(now);
    }

    /// Forget the buffer, keeping the allocation.
    pub fn clear(&mut self) {
        self.buf.clear();
        self.since = None;
    }
}

/// **Type-ahead over a group's labels, arming the one deadline the buffer needs.**
///
/// Answers the index of the first label the buffer is a prefix of, or `None` when the key is not
/// text or nothing matches. ASCII-case-insensitive, because a user typing at a list is not thinking
/// about Shift — and *ASCII*, deliberately: a full Unicode case fold needs tables, and this crate's
/// dependency list is `vitui-runtime` and nothing else (C6).
///
/// # The `deadline_for` is the point of the function
///
/// Everything else here is a `starts_with`. What cannot be written by the caller is the wakeup: a
/// screen with a standing type-ahead buffer would otherwise block until the *next* key, so the
/// buffer would lapse at the moment its meaning was needed rather than a second after the last one.
/// [`Ctx::deadline_for`] takes `id` so the census can prove which widget asked; see this module's
/// header for why that is not the same claim as attribution.
pub fn seek(
    cx: &mut Ctx<'_, '_>,
    id: Id,
    ahead: &mut TypeAhead,
    k: &Pressed,
    labels: &[&str],
) -> Option<usize> {
    let now = cx.now();
    ahead.expire(now);
    let mut typed = String::new();
    if !text(k, &mut typed) {
        return None;
    }
    for c in typed.chars() {
        ahead.push(c, now);
    }
    // **The one call.** A buffer is standing, so `deadline` is `Some` by construction.
    if let Some(at) = ahead.deadline() {
        cx.deadline_for(id, at);
    }
    matched(ahead.buffer(), labels)
}

/// The first label `buf` is an ASCII-case-insensitive prefix of.
///
/// Split out so that the matching rule can be tested without a frame, and so that [`seek`]'s body is
/// visibly *the deadline plus a `starts_with`*.
pub fn matched(buf: &str, labels: &[&str]) -> Option<usize> {
    if buf.is_empty() {
        return None;
    }
    labels.iter().position(|label| {
        label.len() >= buf.len()
            && label
                .as_bytes()
                .iter()
                .zip(buf.as_bytes())
                .all(|(a, b)| a.eq_ignore_ascii_case(b))
    })
}

/// **The helper: what a `Group` moves with.**
///
/// [`step`] first, then [`seek`] — and the order is load-bearing rather than tidy. A list whose
/// entries begin with `h`, `j`, `k` and `l` must still move on `Home`, and a reading that offered
/// the key to type-ahead first would seek to the entry beginning `Home`… if one existed, and
/// otherwise consume the key and move nothing. `KeyMap::step`-before-`match_first` is the same
/// ordering one crate down, made for the same reason.
///
/// `None` means **not this group's key**, and the caller owes it a `Ctx::decline`.
pub fn cursor(
    cx: &mut Ctx<'_, '_>,
    id: Id,
    cur: Cursor,
    ahead: &mut TypeAhead,
    k: &Pressed,
    labels: &[&str],
) -> Option<usize> {
    if let Some(moved) = step(k, cur) {
        // A cursor key is not type-ahead, and a standing buffer that survived one would make the
        // next letter continue a search the user has visibly abandoned.
        ahead.clear();
        return Some(moved);
    }
    seek(cx, id, ahead, k, labels)
}

/// The reading of type-ahead that is **wrong**, kept runnable.
pub mod defective {
    use super::{Ctx, Id, Pressed, TypeAhead, matched, text};

    /// **[`super::seek`] without the deadline.**
    ///
    /// Identical in every other respect, and identical on every frame a key arrives on. What it
    /// cannot do is expire: no frame runs at the deadline, so the buffer is cleared by the arrival
    /// of the next key — the key whose meaning depends on it — and the census can name nobody who
    /// asked.
    pub fn seek_without_a_deadline(
        cx: &mut Ctx<'_, '_>,
        _id: Id,
        ahead: &mut TypeAhead,
        k: &Pressed,
        labels: &[&str],
    ) -> Option<usize> {
        let now = cx.now();
        ahead.expire(now);
        let mut typed = String::new();
        if !text(k, &mut typed) {
            return None;
        }
        for c in typed.chars() {
            ahead.push(c, now);
        }
        matched(ahead.buffer(), labels)
    }
}

// ── the tab-stop count ───────────────────────────────────────────────────────────────────────────

/// The three readings of a frame's focus ring, which are not interchangeable.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Stops {
    /// Every entry declared. **A `Group` does not remove anything from here** — that is what lets a
    /// collection move its own cursor over entries `Tab` will never stop on.
    pub ring: usize,
    /// The entries the walk can land on.
    pub stops: usize,
    /// The walk itself, which visits each of those once.
    pub walk: usize,
}

/// How many panels [`fixture`] stands up. Twelve, which is the accordion and the gallery.
pub const PANELS: usize = 12;
/// The smallest collection on [`fixture`]'s screen.
pub const ROWS_MIN: usize = 4;
/// How many focusables a panel has that are not rows: a header and a footer.
pub const CHROME: usize = 2;

/// **The screen the tab-stop count is taken on, and it is this crate's rather than C02's.**
///
/// Twelve panels; panel *i* holds a header, a collection of `ROWS_MIN + i` rows, and a footer.
/// `group` decides whether the collection opens its own
/// [`Group`](vitui_runtime::focus::ScopeKind::Group) scope, which is the entire difference the
/// measurement is about.
///
/// # Why it is not the screen that was measured
///
/// **266 tab stops become 69** is C02's twelve-panel gallery. When this fixture was written not
/// one of the components on it — `table`, `tree`, `select`, `form`, `pagination` — was declared
/// here; all five are now, and the fixture stays anyway, because what it measures is the
/// **identity** and not the screen. There is no way to stand C02's screen up from this crate and no
/// arrangement of a fixture that honestly produces those two numbers: `266 = c·r + s` with
/// `69 = c + s` forces `c·(r − 1) = 197`, and 197 is prime — so the only uniform screen that yields
/// the pair is **one** collection of 198 rows beside 68 loose focusables, which is not a gallery and
/// not anything else. The prototype's screen was heterogeneous and is not recoverable from the
/// numbers it left.
///
/// So the fixture is stated rather than fitted, and `examples/nav_numbers.rs` prints both columns.
/// **What reproduces is the identity** — a collection is one tab stop, whatever it holds — and that
/// is the claim the sentence makes; the magnitude is a property of a screen this ticket does not
/// own. A fixture aimed at a remembered number is a fixture that has stopped measuring anything.
pub fn fixture(cx: &mut Ctx<'_, '_>, group: bool) {
    let band = cx.area();
    let mut panels = [band; PANELS];
    let n = Col::new().split_into(band, &[Weight(1); PANELS], &mut panels);
    for (i, panel) in panels[..n].iter().enumerate() {
        cx.with_key(i as u64, |cx| {
            let head = cx.id();
            let _ = cx.interact(head, *panel, Interest::FOCUS);
            let rows = ROWS_MIN + i;
            let body = |cx: &mut Ctx<'_, '_>| {
                for r in 0..rows {
                    cx.with_key(r as u64, |cx| {
                        let id = cx.id();
                        let _ = cx.interact(id, *panel, Interest::FOCUS);
                    });
                }
            };
            match group {
                // The scope's id is the panel's own, so twelve scopes are twelve identities —
                // `scope` roots no identity of its own (spec §4), and one shared name would make
                // the twelve collide.
                true => {
                    let sid = cx.id();
                    cx.scope(sid, ScopeKind::Group, body);
                }
                false => body(cx),
            }
            cx.with_key(u64::MAX, |cx| {
                let foot = cx.id();
                let _ = cx.interact(foot, *panel, Interest::FOCUS);
            });
        });
    }
}

/// Play [`fixture`] and read the three counts off the frame.
pub fn stops(group: bool) -> Stops {
    let mut driver = Driver::headless(120, 40).expect("a sink cannot fail to attach");
    driver.frame(|cx| fixture(cx, group));
    let frame = driver.inspect();
    Stops {
        ring: frame.ring().len(),
        stops: frame.stop_ids().count(),
        walk: frame.tab_walk().count(),
    }
}

// ── the ledger ───────────────────────────────────────────────────────────────────────────────────
//
// **Every number this module is gated or reported on has one home and it is here** — the runtime's
// ledger rule (`crates/vitui-runtime/src/ledger.rs`). Every figure is a count over a deterministic
// frame, so none carries a machine.

/// **Tab stops on [`fixture`]'s screen with no `Group` scope. 138.**
///
/// Twelve panels of `2 + (4 + i)` focusables, `i` in `0..12`: `12·2 + Σ(4..=15) = 24 + 114`.
pub const STOPS_UNGROUPED: usize = 138;
/// **Tab stops on the same screen with each collection in its own `Group`. 36.**
///
/// Header, collection, footer — three per panel, twelve panels. *A list is one tab stop.*
pub const STOPS_GROUPED: usize = 36;
/// **Ring entries, which are the same either way. 138.**
///
/// The `Group` collapses the walk and removes nothing, which is what lets a collection move its own
/// cursor over the rows `Tab` no longer stops on. A reading that took only the walk would report a
/// screen whose contents had disappeared.
pub const RING_ENTRIES: usize = 138;

/// **The figure, on C02's screen: 266 tab stops.** Recorded, not reproduced — see [`fixture`].
pub const SPEC_UNGROUPED: usize = 266;
/// **The figure, on C02's screen: 69 tab stops.** Recorded, not reproduced — see [`fixture`].
pub const SPEC_GROUPED: usize = 69;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keys::{press, press_at};
    use vitui_runtime::keys::Chord;

    const LABELS: [&str; 6] = ["ant", "abbot", "bee", "Cormorant", "dove", "eagle"];

    fn key(code: Code) -> Pressed {
        press(Chord::new(code))
    }

    /// The same key on its release edge, which is the second half of one keystroke at kitty flag 2.
    fn released(code: Code) -> Pressed {
        let mut k = press(Chord::new(code));
        k.kind = Edge::Release;
        k
    }

    /// **One keystroke is one step, and the terminal reports two edges.**
    ///
    /// The engine pushes kitty flag 31, whose bit 2 is *report event types*, so a real terminal
    /// sends `Down` twice — once pressed, once released. This helper read only `code`, so a
    /// `collection` on Ghostty moved its cursor **two rows for one press**, and the same program on
    /// a legacy terminal moved one. Watched in both directions: the press still steps, and the
    /// release does not.
    #[test]
    fn a_release_is_not_a_cursor_key_and_a_press_still_is() {
        let cur = Cursor {
            at: 0,
            len: LABELS.len(),
            page: 4,
        };
        for code in [
            Code::Down,
            Code::Up,
            Code::Left,
            Code::Right,
            Code::Home,
            Code::End,
            Code::PageUp,
            Code::PageDown,
        ] {
            assert!(
                step(&key(code), cur).is_some(),
                "{code:?} pressed is this group's key",
            );
            assert_eq!(
                step(&released(code), cur),
                None,
                "{code:?} released is the other half of the same keystroke and must not step again",
            );
        }
    }

    // ── criterion 4: arrows, Home/End, PageUp/PageDown ───────────────────────────────────────────

    /// **Acceptance criterion 4, the half that needs no frame: every cursor key, at both ends.**
    ///
    /// Both ends of the range for each key, because a clamp that is right in the middle and wrong at
    /// the edges is the only kind anybody writes.
    #[test]
    fn the_cursor_keys_move_and_clamp_at_both_ends() {
        let cur = Cursor::new(10, 4);
        for (code, from, to, what) in [
            (Code::Down, 0, 1, "down"),
            (Code::Down, 9, 9, "down clamps at the last"),
            (Code::Up, 5, 4, "up"),
            (Code::Up, 0, 0, "up clamps at the first"),
            (Code::Right, 0, 1, "right is down"),
            (Code::Left, 3, 2, "left is up"),
            (Code::Home, 7, 0, "home"),
            (Code::End, 2, 9, "end"),
            (Code::PageDown, 0, 4, "a page down"),
            (Code::PageDown, 7, 9, "a page down clamps"),
            (Code::PageUp, 9, 5, "a page up"),
            (Code::PageUp, 2, 0, "a page up clamps"),
        ] {
            assert_eq!(
                step(&key(code), cur.to(from)),
                Some(to),
                "{what}: {code:?} from {from}"
            );
        }
    }

    /// **An empty group moves nothing**, and answers `None` so the key is declined rather than
    /// consumed by a list with nothing in it.
    #[test]
    fn an_empty_group_declines_every_cursor_key() {
        let empty = Cursor::new(0, 4);
        for code in [Code::Down, Code::Up, Code::Home, Code::End, Code::PageDown] {
            assert_eq!(step(&key(code), empty), None);
        }
        assert_eq!(empty.last(), 0);
    }

    /// **A chord is not a cursor key, and it is the same predicate a field declines with.**
    ///
    /// `Ctrl+Home` is the application's *go to the top of the document*; `Home` is the list's. A
    /// group that read `code` alone would eat every accelerator on a named key exactly as an
    /// unguarded field eats every accelerator on a letter.
    #[test]
    fn a_chord_on_a_named_key_is_not_the_groups() {
        let cur = Cursor::new(10, 4).to(5);
        assert_eq!(step(&key(Code::Home), cur), Some(0));
        assert_eq!(
            step(&press(Chord::new(Code::Home).ctrl()), cur),
            None,
            "Ctrl+Home is the application's"
        );
        assert_eq!(step(&press(Chord::new(Code::End).alt()), cur), None);
        // Shift passes, because a range selection is the group's business.
        assert_eq!(step(&press(Chord::new(Code::Down).shift()), cur), Some(6));
    }

    /// **Neither end wraps**, which is what tells `Down` apart from `Tab`.
    #[test]
    fn a_groups_cursor_stops_where_the_ring_would_wrap() {
        let cur = Cursor::new(3, 2);
        assert_eq!(step(&key(Code::Down), cur.to(2)), Some(2));
        assert_eq!(step(&key(Code::Up), cur.to(0)), Some(0));
    }

    /// A letter is not a cursor key, so it falls through to type-ahead.
    #[test]
    fn a_letter_is_not_a_cursor_key() {
        assert_eq!(step(&press(Chord::key('a')), Cursor::new(6, 3)), None);
    }

    // ── the matching rule ────────────────────────────────────────────────────────────────────────

    /// Prefix matching, first match wins, ASCII case folded.
    #[test]
    fn type_ahead_matches_the_first_label_it_is_a_prefix_of() {
        assert_eq!(matched("a", &LABELS), Some(0), "ant, in order");
        assert_eq!(matched("ab", &LABELS), Some(1), "and now abbot");
        assert_eq!(matched("cor", &LABELS), Some(3), "folded to ASCII case");
        assert_eq!(matched("z", &LABELS), None);
        assert_eq!(
            matched("", &LABELS),
            None,
            "an empty buffer matches nothing"
        );
        assert_eq!(
            matched("antler", &LABELS),
            None,
            "a buffer longer than the label is not a prefix of it"
        );
    }

    // ── criterion 5: the deadline ────────────────────────────────────────────────────────────────

    /// **Acceptance criterion 5: the buffer survives to the next keypress rather than expiring at
    /// it.**
    ///
    /// Three frames on a pinned clock, which is what makes this a gate rather than a stopwatch:
    /// `a` at *t*, `b` at *t + 400 ms*, and the buffer is `"ab"` — so the second key **refined** a
    /// search rather than starting one. The deadline is armed at *t + 1 s* and the census names the
    /// collection that asked.
    ///
    /// And the other direction: the same two keys a second and a half apart, where the buffer has
    /// lapsed and `b` starts afresh. Without both halves the gate passes on a buffer that never
    /// expires at all.
    #[test]
    fn the_buffer_survives_to_the_next_keypress_rather_than_expiring_at_it() {
        let id = Id::named("collection");
        let mut driver = Driver::headless(60, 8).expect("a sink cannot fail to attach");
        let mut ahead = TypeAhead::new();
        let t0 = Instant::now();
        driver.pin_clock(t0);

        let mut hit = None;
        driver.frame(|cx| hit = seek(cx, id, &mut ahead, &press_at(Chord::key('a'), t0), &LABELS));
        assert_eq!(hit, Some(0), "`a` seeks to ant");
        assert_eq!(ahead.buffer(), "a");
        assert_eq!(
            ahead.deadline(),
            Some(t0 + WINDOW),
            "one deadline, a second out"
        );
        assert_eq!(
            driver.inspect().wakes().asked_by(id),
            1,
            "and the census names the collection that asked"
        );

        // Well inside the window.
        driver.advance(Duration::from_millis(400));
        driver.frame(|cx| hit = seek(cx, id, &mut ahead, &press(Chord::key('b')), &LABELS));
        assert_eq!(
            ahead.buffer(),
            "ab",
            "the buffer survived, so the second key refined the search"
        );
        assert_eq!(hit, Some(1), "which is abbot and not bee");
        assert_eq!(
            driver.inspect().wakes().asked_by(id),
            2,
            "one deadline a key"
        );

        // The other direction: past the window, and `b` starts afresh.
        driver.advance(WINDOW + Duration::from_millis(500));
        driver.frame(|cx| hit = seek(cx, id, &mut ahead, &press(Chord::key('b')), &LABELS));
        assert_eq!(ahead.buffer(), "b", "the lapsed buffer was dropped");
        assert_eq!(hit, Some(2), "so this is bee");
    }

    /// **The defect the deadline exists against, watched happening.**
    ///
    /// `defective::seek_without_a_deadline` is identical on every frame a key arrives on — the same
    /// buffer, the same match — so no assertion about *what was typed* can tell the two apart. What
    /// separates them is that **no frame runs at the deadline**: the census is empty, so nothing
    /// woke the screen at the moment the buffer lapsed, and the buffer is cleared by the arrival of
    /// the next key instead of by time passing.
    ///
    /// Both arms are driven through the same sequence and asserted to agree on the buffer and to
    /// disagree on the census, which is the only place the difference is visible.
    #[test]
    fn without_the_deadline_nothing_wakes_the_screen_when_the_buffer_lapses() {
        let id = Id::named("collection");
        let t0 = Instant::now();

        let drive = |armed: bool| -> (String, u64, usize) {
            let mut driver = Driver::headless(60, 8).expect("a sink cannot fail to attach");
            let mut ahead = TypeAhead::new();
            driver.pin_clock(t0);
            for c in ['a', 'b'] {
                driver.frame(|cx| {
                    let k = press(Chord::key(c));
                    let _ = match armed {
                        true => seek(cx, id, &mut ahead, &k, &LABELS),
                        false => {
                            defective::seek_without_a_deadline(cx, id, &mut ahead, &k, &LABELS)
                        }
                    };
                });
                driver.advance(Duration::from_millis(400));
            }
            let wakes = driver.inspect().wakes();
            (
                ahead.buffer().to_string(),
                wakes.asked_by(id),
                wakes.census_len(),
            )
        };

        let (armed_buf, armed_census, armed_len) = drive(true);
        let (bare_buf, bare_census, bare_len) = drive(false);

        assert_eq!(
            armed_buf, bare_buf,
            "the two arms agree about what was typed, which is why a buffer assertion cannot see \
             the defect"
        );
        assert_eq!(armed_census, 2, "two keys, two deadlines");
        assert_eq!(armed_len, 1, "and one widget in the census");
        assert_eq!(
            (bare_census, bare_len),
            (0, 0),
            "nothing asked for a frame, so the screen sleeps through the moment the buffer lapses"
        );
    }

    /// **The `id` is the census and not the attribution** — the ticket's own correction, as a
    /// number.
    ///
    /// Runtime 06 argues that a deadline is attributed to its **call site**, which is true of
    /// `deadline` and `deadline_for` alike; it never deletes the second. What the `id` buys is
    /// `WakeLedger::asked_by`, and the two verbs differ on that and on nothing else: **1 against 0**
    /// in the census, with a wakeup arriving either way.
    #[test]
    fn the_id_is_the_census_and_not_the_attribution() {
        let id = Id::named("collection");
        let at = Instant::now() + WINDOW;

        let mut with = Driver::headless(20, 3).expect("sink");
        with.frame(|cx| cx.deadline_for(id, at));
        let ledger = with.inspect().wakes();
        assert_eq!(ledger.asked_by(id), 1);
        assert_eq!(ledger.census_len(), 1);
        assert_eq!(
            ledger.line_count(),
            1,
            "and one line asked, which is the attribution"
        );

        let mut without = Driver::headless(20, 3).expect("sink");
        without.frame(|cx| cx.deadline(at));
        let ledger = without.inspect().wakes();
        assert_eq!(ledger.asked_by(id), 0, "no census entry");
        assert_eq!(ledger.census_len(), 0);
        assert_eq!(
            ledger.line_count(),
            1,
            "the same one line asked, which is what makes attribution the wrong reason to choose \
             between the two verbs"
        );
    }

    /// **`cursor` runs `step` before `seek`**, so a list of `h`-words still moves on `Home`.
    #[test]
    fn a_cursor_key_is_never_offered_to_type_ahead() {
        let id = Id::named("collection");
        let homely = ["home", "honey", "hound"];
        let mut driver = Driver::headless(60, 8).expect("sink");
        let mut ahead = TypeAhead::new();
        let cur = Cursor::new(3, 2).to(2);

        let mut moved = None;
        driver.frame(|cx| {
            moved = cursor(cx, id, cur, &mut ahead, &key(Code::Home), &homely);
        });
        assert_eq!(moved, Some(0), "Home moved the cursor");
        assert_eq!(
            ahead.buffer(),
            "",
            "and nothing was typed, so the labels beginning with `Home` are irrelevant"
        );
        assert_eq!(
            driver.inspect().wakes().asked_by(id),
            0,
            "a cursor key owes no deadline"
        );

        // And a letter does reach type-ahead.
        driver.frame(|cx| {
            moved = cursor(cx, id, cur, &mut ahead, &press(Chord::key('h')), &homely);
        });
        assert_eq!(moved, Some(0));
        assert_eq!(ahead.buffer(), "h");
    }

    /// **A cursor key clears a standing buffer**, or the next letter continues a search the user has
    /// visibly abandoned.
    #[test]
    fn a_cursor_key_clears_a_standing_buffer() {
        let id = Id::named("collection");
        let mut driver = Driver::headless(60, 8).expect("sink");
        let mut ahead = TypeAhead::new();
        let cur = Cursor::new(6, 3);
        driver.frame(|cx| {
            let _ = cursor(cx, id, cur, &mut ahead, &press(Chord::key('a')), &LABELS);
        });
        assert_eq!(ahead.buffer(), "a");
        driver.frame(|cx| {
            let _ = cursor(cx, id, cur, &mut ahead, &key(Code::Down), &LABELS);
        });
        assert_eq!(ahead.buffer(), "");
        assert!(!ahead.is_standing());
        assert_eq!(ahead.deadline(), None, "and the wakeup is discharged");
    }

    // ── criterion 6: the tab-stop count ──────────────────────────────────────────────────────────

    /// **Acceptance criterion 6: a collection is one tab stop.**
    ///
    /// [`STOPS_UNGROUPED`] against [`STOPS_GROUPED`] on the screen this crate can stand up — spec
    /// **266 against 69** is C02's gallery and [`fixture`] says why it is not recoverable. What
    /// the gate asserts is the identity the sentence makes, in a form that is a property of the
    /// mechanism rather than of the screen: **the walk with groups is exactly three per panel** —
    /// header, collection, footer — whatever any collection holds.
    ///
    /// And the ring is **unchanged**, which is the half a walk-only reading cannot see: a `Group`
    /// collapses the walk and removes nothing, so the collection can still move its own cursor.
    #[test]
    fn a_collection_is_one_tab_stop_and_the_ring_still_holds_its_rows() {
        let loose = stops(false);
        let grouped = stops(true);

        assert_eq!(loose.stops, STOPS_UNGROUPED);
        assert_eq!(loose.walk, STOPS_UNGROUPED, "with no group, walk == stops");
        assert_eq!(grouped.stops, STOPS_GROUPED);
        assert_eq!(grouped.walk, STOPS_GROUPED);

        // The mechanism, stated so the number is a consequence rather than a fixture's property.
        assert_eq!(
            grouped.walk,
            PANELS * (CHROME + 1),
            "a panel is a header, one collection and a footer, whatever the collection holds"
        );
        let rows: usize = (0..PANELS).map(|i| ROWS_MIN + i).sum();
        assert_eq!(loose.stops, PANELS * CHROME + rows);

        // **The ring is the same either way**, which is what makes the collapse a walk and not a
        // deletion.
        assert_eq!(loose.ring, RING_ENTRIES);
        assert_eq!(
            grouped.ring, RING_ENTRIES,
            "a Group removed nothing from the ring"
        );
        assert!(
            grouped.walk < grouped.ring,
            "and the walk is strictly shorter than what the collection can reach: {} of {}",
            grouped.walk,
            grouped.ring
        );
    }

    /// **The walk repeats no id either way**, so the collapse is not hiding a duplicate.
    #[test]
    fn the_collapsed_walk_repeats_no_id() {
        for group in [false, true] {
            let mut driver = Driver::headless(120, 40).expect("sink");
            driver.frame(|cx| fixture(cx, group));
            let mut walk: Vec<Id> = driver.inspect().tab_walk().collect();
            let before = walk.len();
            walk.sort_by_key(|id| id.raw());
            walk.dedup();
            assert_eq!(
                walk.len(),
                before,
                "the walk repeats an id at group={group}"
            );
        }
    }
}
