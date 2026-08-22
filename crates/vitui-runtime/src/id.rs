//! Identity: where a name that survives between frames comes from.
//!
//! Spec §5; ADR 0013 (an `Id` may never be persisted). **The source is the call site.**
//! `#[track_caller]` gives the caller's `file:line:col` — stable across frames, unique per call site,
//! and free at run time because it is a `&'static Location`.
//!
//! ```text
//! Id = fnv(parent_id, file_ptr, line, col)          // FNV-1a, and the finalizer was measured off
//! ```
//!
//! ```
//! use vitui_runtime::ctx::{Ctx, Driver};
//! use vitui_runtime::id::Id;
//!
//! // **One function, drawn twice.** Two `frame` closures would be two call sites, and comparing
//! // their ids would compare two widgets rather than one widget across two frames.
//! fn draw(cx: &mut Ctx<'_, '_>) -> Id {
//!     cx.id()
//! }
//!
//! let mut driver = Driver::headless(20, 5).expect("sink");
//! let mut first = None;
//! let mut second = None;
//! driver.frame(|cx| { first = Some(draw(cx)); });
//! driver.frame(|cx| { second = Some(draw(cx)); });
//! assert_eq!(first, second, "the same call site is the same widget, frame after frame");
//! ```
//!
//! # A loop needs a key
//!
//! One call site produces N widgets, so a loop says which one it is at:
//! `cx.with_key(row.id, |cx| …)`. That is the only thing a caller has to remember, and it is the only
//! thing a caller *can* do about identity — there is no registration and nothing to keep.
//!
//! # The finalizer was built on a sound argument and measured backwards
//!
//! A splitmix64 finalizer over the FNV result is the textbook fix for a hash whose low bits are
//! poorly mixed, and here it made things **worse**: **1.000 probes per insert without it against
//! 1.370 with**. The reason is the input. Line and column are small dense integers, and FNV-1a maps
//! them injectively over the range a source file actually occupies, so the unmixed hash is already
//! collision-free where it matters and the finalizer only smears dense keys across buckets they then
//! have to probe out of. **It is off, and the measurement rather than the argument is why.**
//!
//! # The container rule, readable from a signature
//!
//! > **A container that returns a rectangle preserves its children's identity; one that takes a
//! > closure renames them, and scoping has nothing to do with it.**
//!
//! The id stack is pushed in exactly two places — [`Ctx::with_key`](crate::ctx::Ctx::with_key) and
//! [`Ctx::with_id`](crate::ctx::Ctx::with_id) — and **`Ctx::child` pushes nothing**, so a
//! rectangle-returning split leaves its panes siblings at one depth. The id path is the **closure
//! tree, not the draw tree**: a realistic screen is depth 1 at the top and 2 inside a keyed row.
//!
//! Getting it the other way costs focus and scroll on every child of a closure-taking container,
//! because both are keyed on an id that changed for a reason the author never wrote down.
//!
//! **It has exactly one exception and the exception is load-bearing:**
//! [`Ctx::scope`](crate::ctx::Ctx::scope) takes a closure and must **not** rename its children, or the
//! frame a modal opens renames every field of the form it traps. `scroll_scope` is the same.
//!
//! # What consumes an id: five, not six
//!
//! Focus, hit-testing, interest declaration, overlay ownership and scroll association. **A
//! [`Memo`](crate::data::Memo) is keyed by where it is stored and consumes no `Id`** — the id-keyed
//! alternative was built and pays a full fold every time a tab is switched away and back.

use std::panic::Location;

/// A widget's name for this frame and the next.
///
/// # It may never be persisted
///
/// The file pointer is an **address**, so an `Id` differs between two runs of the same binary — and
/// between two builds of it. Nothing may write one to disk, send one over a wire, or compare one
/// against a stored value. That is why there is no `Serialize`, no `Display`, and **no `Ord`**: an
/// ordering would invite a `BTreeMap` keyed on one, which is a stored value with extra steps.
///
/// `Hash` and `Eq` are here because a hash table keyed on one *within a frame* is exactly what
/// [`IdTable`] is.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Id(u64);

/// FNV-1a's offset basis.
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

/// FNV-1a's prime.
const FNV_PRIME: u64 = 0x100_0000_01b3;

/// Fold eight bytes in, FNV-1a.
const fn fold(mut hash: u64, bytes: [u8; 8]) -> u64 {
    let mut i = 0;
    while i < 8 {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
        i += 1;
    }
    hash
}

impl Id {
    /// The root: what a frame's outermost draw is, and the parent of everything declared at depth
    /// zero.
    pub const ROOT: Id = Id(FNV_OFFSET);

    /// Derive an id from a parent and a call site.
    ///
    /// **No finalizer** — see the module comment for the measurement that removed it.
    pub fn at(parent: Id, loc: &'static Location<'static>) -> Id {
        let file = loc.file();
        // The *pointer*, not the contents. Two call sites in one file share the pointer and differ in
        // line and column; two files differ in the pointer. Hashing the bytes would be O(path length)
        // per widget for a distinction the pointer already makes — and it is the reason an `Id` cannot
        // be persisted, which is stated on the type rather than hidden here.
        let file_ptr = file.as_ptr() as usize as u64;
        let mut h = fold(parent.0, parent.0.to_le_bytes());
        h = fold(h, file_ptr.to_le_bytes());
        h = fold(h, u64::from(loc.line()).to_le_bytes());
        h = fold(h, u64::from(loc.column()).to_le_bytes());
        Id(h)
    }

    /// Derive an id from a parent and a caller-supplied key, for a loop.
    pub fn keyed(parent: Id, key: u64) -> Id {
        Id(fold(
            fold(parent.0, parent.0.to_le_bytes()),
            key.to_le_bytes(),
        ))
    }

    /// A `const` caller-level escape: a name the author chose.
    ///
    /// **Free, and available.** *No user-supplied string* narrows to *not mandatory*, not to
    /// forbidden — an author who wants a stable name across a refactor that moves the call site can
    /// have one.
    ///
    /// It participates in duplicate detection like everything else, and **that had to be fixed rather
    /// than noticed**: it escaped detection until a test failed, because it was derived without a
    /// parent and so could not collide with anything by construction.
    ///
    /// ```
    /// use vitui_runtime::id::Id;
    ///
    /// const SAVE: Id = Id::named("toolbar.save");
    /// assert_eq!(SAVE, Id::named("toolbar.save"));
    /// assert_ne!(SAVE, Id::named("toolbar.open"));
    /// ```
    pub const fn named(name: &str) -> Id {
        let bytes = name.as_bytes();
        let mut h = FNV_OFFSET;
        let mut i = 0;
        while i < bytes.len() {
            h ^= bytes[i] as u64;
            h = h.wrapping_mul(FNV_PRIME);
            i += 1;
        }
        Id(h)
    }

    /// An id from a raw value, for a test that needs to name one.
    pub const fn from_raw(v: u64) -> Id {
        Id(v)
    }

    /// The raw value, for a table to index by. **Not for storing** — see the type's documentation.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// One slot of the stamped table.
#[derive(Clone, Copy, Debug, Default)]
struct Slot {
    id: u64,
    /// Which frame claimed it. **Compared against the table's stamp**, which is why the table is
    /// never cleared — and why a `Frame` that was never begun would hang without ticket 08's two
    /// remedies.
    stamp: u32,
}

/// The stamped open-addressed table that detects a duplicate id in about one probe.
///
/// # Chosen with a number, not an argument
///
/// The obvious implementation is `Vec::contains`, and it is **22.5 ns a widget and O(n²)**: growth of
/// **13.20× for 4× the widgets**. This is **1.1 ns** and **3.95×**. Duplicate detection is part of the
/// design rather than a debug aid, so it had to be affordable on every frame at every widget.
///
/// **Stamped rather than cleared**, which is what makes ticket 08's `begin` obligation an obligation:
/// a table whose stamp matches every slot has no free slot, and a naive probe walks the ring for
/// ever.
#[derive(Clone, Debug)]
pub struct IdTable {
    slots: Vec<Slot>,
    stamp: u32,
    live: usize,
    /// How many times the table has grown. **A count, and the gate is that it is zero in a steady
    /// frame**: growing every frame would mean allocating every frame.
    grows: u32,
    /// How many probes the claims this frame took, for the report.
    probes: u64,
    /// How many duplicates were rejected this frame.
    merges: u32,
}

/// The load factor the table grows at.
///
/// Two thirds. Above it an open-addressed table's probe count climbs steeply; below it the table is
/// mostly empty and the cache misses cost more than the probes save.
const LOAD_NUMERATOR: usize = 2;
/// See [`LOAD_NUMERATOR`].
const LOAD_DENOMINATOR: usize = 3;

impl IdTable {
    /// A table with room for a dense screen.
    ///
    /// Five hundred and twelve slots for the three hundred and thirteen interactive regions a dense
    /// 300×80 screen has — under the load factor without a growth on the first frame, which is what
    /// makes `grows == 0` reachable rather than aspirational.
    pub fn new() -> IdTable {
        IdTable {
            slots: vec![Slot::default(); 512],
            // One, not zero: see ticket 08. A fresh table is already past the value an untouched slot
            // carries.
            stamp: 1,
            live: 0,
            grows: 0,
            probes: 0,
            merges: 0,
        }
    }

    /// Start a frame: bump the stamp, so every slot is now foreign.
    pub(crate) fn begin(&mut self) {
        self.live = 0;
        self.probes = 0;
        self.merges = 0;
        self.stamp = self.stamp.wrapping_add(1);
        if self.stamp == 0 {
            // At the wrap the table is zeroed, because a slot stamped `u32::MAX` would otherwise read
            // as this frame's again. At sixty frames a second that is two years and three months of
            // uptime — the kind of number that arrives as a bug report rather than a test.
            for slot in &mut self.slots {
                *slot = Slot::default();
            }
            self.stamp = 1;
        }
    }

    /// Claim an id for this frame.
    ///
    /// `None` means **the id is already claimed** — a merge. The policy is that **the first claimant
    /// wins and the second is inert**, which is why ticket 13's overlay id is handed over rather than
    /// derived: a widget that wanted to own something and lost has to be told, and the way it is told
    /// is that its `Response` does nothing.
    ///
    /// The probe is bounded by the table's length, so the answer is a value rather than a hang however
    /// wrong the stamp is.
    pub(crate) fn claim(&mut self, id: Id) -> Option<usize> {
        if self.live * LOAD_DENOMINATOR >= self.slots.len() * LOAD_NUMERATOR {
            self.grow();
        }
        let len = self.slots.len();
        if len == 0 {
            return None;
        }
        let mut at = usize::try_from(id.raw() % len as u64).unwrap_or(0);
        for _ in 0..len {
            self.probes += 1;
            let slot = self.slots[at];
            if slot.stamp != self.stamp {
                self.slots[at] = Slot {
                    id: id.raw(),
                    stamp: self.stamp,
                };
                self.live += 1;
                return Some(at);
            }
            if slot.id == id.raw() {
                self.merges += 1;
                return None;
            }
            at = (at + 1) % len;
        }
        None
    }

    /// Double the table, re-inserting what this frame has claimed.
    ///
    /// Only this frame's entries move: an older stamp is already foreign, so there is nothing to
    /// carry over and the rehash is over `live` rather than over the table.
    fn grow(&mut self) {
        let doubled = self.slots.len() * 2;
        let old = std::mem::replace(&mut self.slots, vec![Slot::default(); doubled]);
        self.grows += 1;
        let len = self.slots.len();
        for slot in old {
            if slot.stamp != self.stamp {
                continue;
            }
            let mut at = usize::try_from(slot.id % len as u64).unwrap_or(0);
            for _ in 0..len {
                if self.slots[at].stamp != self.stamp {
                    self.slots[at] = slot;
                    break;
                }
                at = (at + 1) % len;
            }
        }
    }

    /// How many ids are claimed this frame.
    pub fn live(&self) -> usize {
        self.live
    }

    /// **How many times the table has grown.** The gate is that this is zero in a steady frame.
    pub fn grows(&self) -> u32 {
        self.grows
    }

    /// How many probes this frame's claims took, for the report.
    pub fn probes(&self) -> u64 {
        self.probes
    }

    /// How many duplicate claims were rejected this frame.
    pub fn merges(&self) -> u32 {
        self.merges
    }

    /// The current stamp.
    pub fn stamp(&self) -> u32 {
        self.stamp
    }

    /// How many slots there are.
    pub fn capacity(&self) -> usize {
        self.slots.len()
    }
}

impl Default for IdTable {
    fn default() -> IdTable {
        IdTable::new()
    }
}

/// The id stack: the closure tree, one entry per [`Ctx::with_key`](crate::ctx::Ctx::with_key) or
/// [`Ctx::with_id`](crate::ctx::Ctx::with_id).
///
/// **`Ctx::child` pushes nothing**, which is the container rule made mechanical rather than
/// documented.
#[derive(Clone, Debug)]
pub(crate) struct IdStack {
    of: Vec<Id>,
}

impl IdStack {
    pub(crate) fn new() -> IdStack {
        IdStack {
            of: Vec::with_capacity(8),
        }
    }

    /// The current parent.
    pub(crate) fn current(&self) -> Id {
        self.of.last().copied().unwrap_or(Id::ROOT)
    }

    /// How deep. Zero at the top of a frame.
    pub(crate) fn depth(&self) -> usize {
        self.of.len()
    }

    pub(crate) fn push(&mut self, id: Id) {
        self.of.push(id);
    }

    pub(crate) fn pop(&mut self) {
        self.of.pop();
    }

    pub(crate) fn clear(&mut self) {
        self.of.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two call sites differ; the same call site does not.
    #[test]
    fn an_id_is_the_call_site() {
        #[track_caller]
        fn here() -> Id {
            Id::at(Id::ROOT, Location::caller())
        }
        let a = here();
        let b = here();
        assert_ne!(a, b, "two call sites, two ids");

        // Two ids on **one line**, differing only by column. `Location::caller()` outside a
        // `#[track_caller]` function reports its own position, so this is two call sites.
        let (c, d) = (
            Id::at(Id::ROOT, Location::caller()),
            Id::at(Id::ROOT, Location::caller()),
        );
        assert_ne!(c, d, "one line, two columns, two widgets");
    }

    /// **A `#[track_caller]` wrapper merges the widgets inside its own body**, which this test
    /// reproduces rather than asserts against.
    ///
    /// It was written by accident: the first version of the test above put two `Location::caller()`
    /// calls inside a `#[track_caller]` helper and expected two ids. They were one, because the
    /// attribute makes every call in the body report *the caller's* location — which is exactly the
    /// hazard `Ctx::id`'s documentation describes, and the reason it is documented at all.
    #[test]
    fn a_track_caller_wrapper_merges_the_widgets_in_its_body() {
        #[track_caller]
        fn two_widgets() -> (Id, Id) {
            (
                Id::at(Id::ROOT, Location::caller()),
                Id::at(Id::ROOT, Location::caller()),
            )
        }
        let (a, b) = two_widgets();
        assert_eq!(
            a, b,
            "both report the caller's location, so they are one widget and the second is inert"
        );
        // And the table says so, which is how it was noticed in the first place.
        let mut table = IdTable::new();
        assert!(table.claim(a).is_some());
        assert_eq!(table.claim(b), None);
        assert_eq!(table.merges(), 1, "duplicate detection is what named this");
    }

    /// A parent changes every child.
    #[test]
    fn a_parent_changes_every_child() {
        #[track_caller]
        fn under(parent: Id) -> Id {
            Id::at(parent, Location::caller())
        }
        let a = under(Id::ROOT);
        let b = under(Id::keyed(Id::ROOT, 7));
        assert_ne!(a, b, "the same call site under two parents is two widgets");
    }

    /// A key distinguishes the rows of a loop.
    #[test]
    fn a_key_distinguishes_the_rows_of_a_loop() {
        let mut seen = std::collections::HashSet::new();
        for row in 0..1_000u64 {
            assert!(
                seen.insert(Id::keyed(Id::ROOT, row)),
                "row {row} collided with an earlier one"
            );
        }
    }

    /// `named` is stable, `const`, and participates like everything else.
    #[test]
    fn named_is_stable_and_const() {
        const A: Id = Id::named("toolbar.save");
        assert_eq!(A, Id::named("toolbar.save"));
        assert_ne!(A, Id::named("toolbar.open"));
        // **And it is claimable**, which is the part that had escaped duplicate detection: a `named`
        // id derived without a parent could not collide with anything by construction, so nothing
        // ever rejected a second use of the same name.
        let mut table = IdTable::new();
        assert!(table.claim(A).is_some());
        assert_eq!(
            table.claim(A),
            None,
            "a repeated name is a merge like any other"
        );
        assert_eq!(table.merges(), 1);
    }

    /// **Zero collisions across an enumerated corpus of 2 880 000 ids.**
    ///
    /// Enumerated rather than sampled: four parents, three file pointers, four hundred lines and six
    /// hundred columns is every call site a large source file has, under every parent a realistic id
    /// path produces. Checked rather than asserted — the ticket's own phrase — because a hash's
    /// collision behaviour is a property of its inputs and these are the inputs.
    #[test]
    fn zero_collisions_across_the_enumerated_corpus() {
        let parents = [
            Id::ROOT,
            Id::keyed(Id::ROOT, 1),
            Id::keyed(Id::ROOT, 2),
            Id::named("scope"),
        ];
        // Three plausible file-pointer values, spaced as separate allocations would be.
        let files: [u64; 3] = [0x1_0000, 0x1_4000, 0x2_8000];
        let mut seen = std::collections::HashSet::with_capacity(3_000_000);
        let mut count = 0usize;
        for parent in parents {
            for file in files {
                for line in 1..=400u64 {
                    for col in 1..=600u64 {
                        let mut h = fold(parent.raw(), parent.raw().to_le_bytes());
                        h = fold(h, file.to_le_bytes());
                        h = fold(h, line.to_le_bytes());
                        h = fold(h, col.to_le_bytes());
                        assert!(
                            seen.insert(h),
                            "collision at parent {parent:?} file {file:#x} line {line} col {col}"
                        );
                        count += 1;
                    }
                }
            }
        }
        assert_eq!(count, 2_880_000, "the corpus is 2 880 000 ids");
    }

    /// **The collision that actually happens is a merge, at probability one**, and the policy is that
    /// the first claimant wins.
    #[test]
    fn a_merge_gives_the_first_claimant_the_id() {
        let mut table = IdTable::new();
        let id = Id::named("shared");
        let first = table.claim(id);
        let second = table.claim(id);
        assert!(first.is_some(), "the first claimant wins");
        assert_eq!(second, None, "and the second is inert");
        assert_eq!(table.live(), 1, "one widget, not two");
        assert_eq!(table.merges(), 1);
    }

    /// **The table does not grow in a steady frame**, which is a count and the reason the initial
    /// capacity is what it is.
    #[test]
    fn the_table_does_not_grow_in_a_steady_frame() {
        let mut table = IdTable::new();
        for frame in 0..10u32 {
            table.begin();
            for i in 0..313u64 {
                table.claim(Id::keyed(Id::ROOT, i));
            }
            assert_eq!(
                table.grows(),
                0,
                "the table grew on frame {frame} for a dense screen's widgets"
            );
        }
        assert_eq!(table.live(), 313);
    }

    /// **Growth is sub-quadratic**: probes for 4× the widgets rise by well under 6×.
    ///
    /// The gate is a ratio because the number belongs to the load factor and the hash, not to a
    /// machine. `Vec::contains` would be 13.20× here; the report prints both.
    #[test]
    fn growth_is_sub_quadratic() {
        let probes_for = |n: u64| {
            let mut table = IdTable::new();
            table.begin();
            for i in 0..n {
                table.claim(Id::keyed(Id::ROOT, i));
            }
            table.probes()
        };
        let small = probes_for(200);
        let large = probes_for(800);
        let ratio = large as f64 / small as f64;
        assert!(
            ratio <= 6.0,
            "probes grew {ratio:.2}x for 4x the widgets, which is quadratic territory"
        );
        // And each insert is about one probe, which is what the finalizer made worse.
        assert!(
            small as f64 / 200.0 < 1.5,
            "{:.3} probes an insert at 200 widgets",
            small as f64 / 200.0
        );
    }

    /// An `Id` has no ordering and no serialisation, which is a compile outcome about absent impls.
    #[test]
    fn an_id_has_no_ordering() {
        // A helper that only accepts `Ord`, applied to something that has it — the presence half.
        const fn needs_ord<T: Ord>() {}
        needs_ord::<u64>();
        // `Id` is deliberately absent from that list; the compile outcome is the doctest on the type.
        // What is checkable here is that the raw value is reachable and the type is not comparable
        // through any inherent method.
        assert_eq!(Id::from_raw(7).raw(), 7);
    }
}

/// **`Id` implements no serialisation and no ordering**, and the compile outcome says so.
///
/// ```compile_fail,E0277
/// use vitui_runtime::id::Id;
/// let mut ids = vec![Id::named("b"), Id::named("a")];
/// ids.sort();
/// ```
///
/// ```compile_fail,E0277
/// use std::collections::BTreeMap;
/// use vitui_runtime::id::Id;
/// let mut m: BTreeMap<Id, u32> = BTreeMap::new();
/// m.insert(Id::named("a"), 1);
/// ```
///
/// and the twin, which is the one collection an `Id` belongs in — keyed within a frame, thrown away
/// with it:
///
/// ```
/// use std::collections::HashMap;
/// use vitui_runtime::id::Id;
///
/// let mut m: HashMap<Id, u32> = HashMap::new();
/// m.insert(Id::named("a"), 1);
/// assert_eq!(m.get(&Id::named("a")), Some(&1));
/// ```
///
/// **The file pointer is an address**, so an `Id` differs between two runs of the same binary. An
/// ordering would invite a `BTreeMap` keyed on one, and a `BTreeMap` that outlives a frame is a
/// stored value with extra steps — which is the thing ADR 0013 forbids.
#[cfg(doc)]
pub struct WhyAnIdIsNotOrdered;
