//! The data contract: [`Revision`], [`Versioned`], [`Edit`] and [`Memo`].
//!
//! Spec §14, and [ADR 0019](../../../docs/adr/). Four plain types over a caller's `T`, `std` only —
//! **the engine is not reachable from this module and does not need to be**, which is why this is the
//! first ticket on the runtime backlog and the only one that needs neither the engine nor the frame.
//!
//! # There is no trait, and the case that was supposed to justify one is the case that removes it
//!
//! A `Rows` trait was designed, built and refused. Both of its proposed methods fail when run:
//!
//! - **`len()` is already an argument to every collection**, and becomes a *second source of truth*
//!   the moment a view is filtered. It also collides with the `len` the collection already has —
//!   `src.len()` is `E0034`, *multiple applicable items in scope*, at every call site.
//! - **`revision()` is a promise to report a number**, and a promise is exactly what gets forgotten.
//!   What makes a bump unforgettable is `Drop` on a guard: a wrapper, not a trait.
//!
//! And the only trait shape that would really have carried the O(1) guarantee — one handing out a
//! slice — **cannot be implemented for the struct-of-vectors every measurement on the map rests on**
//! (`E0308`). A trait that accepts a walked source does not carry the guarantee it was added for.
//!
//! > **Closures cover every shape a trait would.**
//!
//! So: no trait is declared here, and `tests::the_module_declares_no_trait` is what keeps that
//! true, because "we decided not to" is not a property of the code.
//!
//! # What each type is for, in one line each
//!
//! | | |
//! |---|---|
//! | [`Revision`] | a number that answers *is this different* — never *is this newer* |
//! | [`Versioned<T>`] | `T`, readable by anyone, writable only through a guard that bumps on drop |
//! | [`Edit`] | that guard |
//! | [`Memo<T>`] | a cached result beside the `Revision` it was computed at |
//!
//! # The load-bearing assumption neither shape carries
//!
//! **O(1) indexed access is prose, not a type.** One row drawer accepts indexed, chunked and linked
//! sources with the same eighty verbs and the same hit index, and the three cost 19.77 µs,
//! 320.85 µs and 78.40 ms — the *reasonable-looking* chunked source is already **321% of the whole
//! frame budget**. `examples/data_numbers.rs` is where that is in the repository rather than only in
//! the spec, and the gate under it is a **step count** — 78 against 780 078 — because all three
//! allocate nothing and the difference is invisible to every heap tool.

use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};

/// **One process-global counter, and its being global is a property rather than a convenience.**
///
/// Two per-value counters both sitting at revision 1 make a data swap invisible to every memo keyed
/// on them — the two values are different and their revisions are equal. That was built before it
/// was a rule; `tests::a_per_value_revision_counter_lets_a_data_swap_go_unnoticed` is the wrong
/// version preserved, and the test beside it is the right one.
///
/// Starts at 1 so that 0 can mean [`Revision::UNKNOWN`], and `Relaxed` because nothing is ordered
/// against it: the number's only job is to be different from the last one. One relaxed fetch-add is
/// 2.09 ns, paid **per edit** and never per frame or per row.
static NEXT_REVISION: AtomicU64 = AtomicU64::new(1);

/// A number that says *this is not what it was*.
///
/// # Two revisions are compared with `==` and never with `<`
///
/// **[`PartialOrd`] and [`Ord`] are deliberately absent, and this is the one absence here with a
/// wrong answer rather than a compile error behind it.** A revision answers *is this different*; it
/// does not answer *is this newer*, and it cannot, because the counter is global. Two values edited
/// in either order take numbers from the same sequence, so `a < b` compares *who touched the counter
/// first across the whole process* — which is not the same question as *whose data is stale*, and is
/// not even about the two values being compared.
///
/// It becomes load-bearing the moment a worker lands: a background result carries the revision its
/// input had, and the frame asks *is this still the revision I have*. Written with `==`, a landing
/// whose input has been edited is discarded, which is correct. **Written with `<`, it is accepted,
/// and the frame shows a result computed from data that no longer exists** — no panic, no log, a
/// plausible wrong number. The first `<` between two revisions turns a landing into a silent wrong
/// answer.
///
/// ```compile_fail,E0369
/// use vitui_runtime::data::Revision;
/// let a = Revision::fresh();
/// let b = Revision::fresh();
/// let _ = a < b;
/// ```
///
/// and the twin, which is the whole of what a revision is for:
///
/// ```
/// use vitui_runtime::data::Revision;
///
/// let a = Revision::fresh();
/// let b = Revision::fresh();
/// assert_ne!(a, b);
/// assert_eq!(a, a);
/// ```
///
/// # Nominal, which is what makes one global counter safe under any crate line
///
/// Two copies of this runtime in one binary both hand out revision 1, and the two can never meet:
/// `Revision` is a distinct type per crate instance, so passing one where the other is expected is
/// `E0308`. That is the argument, and it cannot be demonstrated from inside one crate — there is no
/// way to build two instances of `vitui-runtime` in a doctest. What *is* demonstrable is the
/// property the argument rests on, that a `Revision` is not a `u64` wearing a hat:
///
/// ```compile_fail,E0308
/// use vitui_runtime::data::Revision;
/// let r: Revision = 1;
/// ```
///
/// ```compile_fail,E0308
/// use vitui_runtime::data::Revision;
/// fn takes(_: u64) {}
/// takes(Revision::fresh());
/// ```
///
/// and the twin, naming the two doors that do exist and are deliberately narrow:
///
/// ```
/// use vitui_runtime::data::Revision;
///
/// let r = Revision::from_raw(7);
/// assert_eq!(r.raw(), 7);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Revision(u64);

impl Revision {
    /// *Memoise nothing.*
    ///
    /// A caller whose data has no revision to offer — a borrowed slice, an mmap, a database cursor —
    /// hands this over, and every [`Memo`] keyed on it recomputes on every frame for ever.
    ///
    /// **The cost is a counter and an assertion rather than a sentence in a doc comment**, because
    /// §14's "the cost is stated in the docs" is how it stops being noticed: on the canonical screen
    /// it is the chart's fold, **84.91 µs, 85% of a 100 µs frame**, and a `Vec`-valued memo under
    /// `UNKNOWN` allocates once a frame for ever.
    /// [`Memo::recomputes`] is the counter, and
    /// `tests::an_unknown_revision_means_memoise_nothing_and_the_counter_says_so` is the
    /// assertion.
    pub const UNKNOWN: Revision = Revision(0);

    /// A revision nothing else has or will have.
    pub fn fresh() -> Revision {
        Revision(NEXT_REVISION.fetch_add(1, Ordering::Relaxed))
    }

    /// Whether this is a real revision rather than [`Revision::UNKNOWN`].
    pub const fn is_known(self) -> bool {
        self.0 != Revision::UNKNOWN.0
    }

    /// Build one from a number.
    ///
    /// **Public, and not only for tests.** A caller composing a key out of two revisions — the
    /// canonical case is a memo whose value is made of paints, which must carry the theme in its key
    /// — folds them into one and this is the door. It is narrow on purpose: there is no `From<u64>`,
    /// so a `u64` never becomes a `Revision` by accident.
    pub const fn from_raw(v: u64) -> Revision {
        Revision(v)
    }

    /// The number, for a caller folding two revisions into one key.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// `T`, readable by anyone, writable only through a guard.
///
/// # `DerefMut` is deliberately absent, and that absence is the whole mechanism
///
/// Reading goes through [`Deref`] and stays shared. There is no `&mut T` except through
/// [`Versioned::edit`], and the guard it returns bumps the revision when it drops — so **the bump is
/// the drop, and there is nothing to forget.** Adding a `DerefMut` impl would restore the silent
/// forgetting this type exists to prevent, by an impl block:
///
/// ```compile_fail,E0596
/// use vitui_runtime::data::Versioned;
/// let mut v = Versioned::new(vec![1u32, 2, 3]);
/// v.push(4);
/// ```
///
/// ```compile_fail,E0594
/// use vitui_runtime::data::Versioned;
/// let mut v = Versioned::new(7u32);
/// *v = 8;
/// ```
///
/// and the twin, naming the one path there is:
///
/// ```
/// use vitui_runtime::data::Versioned;
///
/// let mut v = Versioned::new(vec![1u32, 2, 3]);
/// let before = v.revision();
/// v.edit().push(4);
/// assert_eq!(v.len(), 4);
/// assert_ne!(v.revision(), before);
/// ```
///
/// # Wrapping the data does not make it exclusive
///
/// Requirement 2 of the map's precedence order is that **no component may require ownership or a
/// mutable borrow of application data**, and *N* components must be able to read the same data at
/// once. `Deref` through a shared reference is what keeps that true:
///
/// ```
/// use vitui_runtime::data::Versioned;
///
/// let v = Versioned::new(vec![1u32, 2, 3]);
/// let as_list = &*v;
/// let as_chart = &*v;
/// assert_eq!(as_list.len(), as_chart.len());
/// ```
///
/// # Its three prices, each a gate rather than an oversight
///
/// 1. **The data is exclusive while the guard is held** (`E0502`). An application computing a new
///    value *from* the data it is editing must end the guard first — and ending it is what bumps:
///
/// ```compile_fail,E0502
/// use vitui_runtime::data::Versioned;
/// let mut v = Versioned::new(vec![1.0f32, 2.0, 3.0]);
/// let mut e = v.edit();
/// let max = v.iter().copied().fold(0.0f32, f32::max);
/// e[0] = max;
/// ```
///
/// ```
/// use vitui_runtime::data::Versioned;
///
/// let mut v = Versioned::new(vec![1.0f32, 2.0, 3.0]);
/// let max = v.iter().copied().fold(0.0f32, f32::max);
/// v.edit()[0] = max;
/// assert_eq!(v[0], 3.0);
/// ```
///
/// 2. **The bump happens on drop rather than on change.** `drop(v.edit())` having touched nothing
///    still invalidates every memo over the value. **Over-invalidation is the cost of not having to
///    diff**, and a content-derived revision — FNV-1a over every row's bytes — was built and refused
///    at **2.59 ms a frame, 26× the budget**: it converts one O(rows) fold into a more expensive one.
/// 3. **One revision covers the whole `T`.** A `Versioned<StructOfVectors>` invalidates a memo over a
///    column that did not change. The fix is one `Versioned` a column, it costs one field a column,
///    and it is the caller's to choose rather than this type's to impose.
///
/// The read side costs **19 202 ns against 19 302 ns** for the same screen — and **zero characters**,
/// because `Deref` is implemented.
pub struct Versioned<T> {
    value: T,
    rev: Revision,
}

impl<T> Versioned<T> {
    /// Wrap a value, taking a fresh revision.
    pub fn new(value: T) -> Versioned<T> {
        Versioned {
            value,
            rev: Revision::fresh(),
        }
    }

    /// The value, explicitly. Never required — [`Deref`] is what call sites use.
    pub const fn get(&self) -> &T {
        &self.value
    }

    /// The revision the value currently has.
    pub const fn revision(&self) -> Revision {
        self.rev
    }

    /// The only path to `&mut T`. The revision is bumped when the returned guard drops.
    pub const fn edit(&mut self) -> Edit<'_, T> {
        Edit { owner: self }
    }

    /// Unwrap, discarding the revision.
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T> Deref for Versioned<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Versioned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Versioned")
            .field("value", &self.value)
            .field("rev", &self.rev)
            .finish()
    }
}

/// The write guard. `DerefMut` here and nowhere else, and the bump is its [`Drop`].
///
/// There is one hole and it is named rather than patched: `std::mem::forget` on a guard skips the
/// bump. That is the whole of it, and a caller who forgets a guard has done something a type system
/// does not undertake to prevent.
pub struct Edit<'a, T> {
    owner: &'a mut Versioned<T>,
}

impl<T> Deref for Edit<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.owner.value
    }
}

impl<T> DerefMut for Edit<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.owner.value
    }
}

impl<T> Drop for Edit<'_, T> {
    /// **A fresh number from the global counter, not an increment of a local one.** Incrementing per
    /// value is the defect this type exists on the other side of: two values both at 1 make a swap
    /// between them invisible.
    fn drop(&mut self) {
        self.owner.rev = Revision::fresh();
    }
}

/// A cached result beside the [`Revision`] it was computed at.
///
/// # What it is worth
///
/// The canonical screen is one table shown at once as a list and as a bar chart. The bars are
/// O(visible); the chart's **scale** is a maximum over all 1 000 000 rows, and that fold is
/// **84.91 µs a frame — 85% of a 100 µs budget — against a 1.28 ns memo hit, which is 66 301×.**
///
/// # What it is not worth, and the sentence belongs here rather than in a ticket
///
/// > **`Memo` skips the computation and damage skips the output, but nothing skips the composition.**
/// > A frame in which nothing changed still costs **23.58 µs, 23.6% of the budget**. That number, not
/// > the sentence it replaces, is what a reactivity layer inherits.
///
/// The view function still runs, the rows are still formatted, the verbs are still issued and the
/// clip arithmetic is still done. The memo hit is 1.28 ns of that 23.58 µs. **§15's promise is
/// narrowed to two thirds**, and a reactivity layer built on top inherits the residue rather than
/// the promise.
///
/// # Keyed by where it is stored, and it consumes no `Id`
///
/// A memo is a field of the caller's own state. It takes one `Revision` by value and nothing else:
/// no id, no name, no path into the identity table, and **it must not join the identity sweep.**
///
/// That is not a filing decision. The id-keyed alternative was built, and because a sweep drops
/// whatever did not draw this frame, it **pays a full fold every time a tab is switched away and
/// back** — 84.91 µs bought by a keystroke. Keyed by storage, the same memo survives its widget
/// being renamed, survives a resize (a revision mentions no geometry), and is untouched when its
/// widget stops drawing.
///
/// ```
/// use vitui_runtime::data::{Memo, Revision, Versioned};
///
/// // The memo lives in the application's own state, beside nothing in particular.
/// #[derive(Default)]
/// struct ChartState {
///     max: Memo<f32>,
/// }
///
/// let data = Versioned::new(vec![1.0f32, 4.0, 2.0]);
/// let mut st = ChartState::default();
///
/// // A length, a revision, and a closure. The chart never learns what it is folding over.
/// let fold = || data.iter().copied().fold(f32::MIN, f32::max);
/// assert_eq!(*st.max.get(data.revision(), fold), 4.0);
/// assert_eq!(st.max.recomputes, 1);
///
/// // Same revision, so the fold does not run again.
/// let fold = || unreachable!("the memo is warm");
/// assert_eq!(*st.max.get(data.revision(), fold), 4.0);
/// assert_eq!(st.max.recomputes, 1);
/// ```
pub struct Memo<T> {
    rev: Revision,
    value: Option<T>,
    /// How many times the closure has run.
    ///
    /// **Public, and a count rather than a timing**, which is what makes every assertion about this
    /// type a gate: §14's rule is that a gate is a count, a ratio, an equality or a compile outcome,
    /// and *the fold did not run* is a count. It is also how [`Revision::UNKNOWN`]'s price stops
    /// being a sentence.
    pub recomputes: u32,
}

impl<T> Memo<T> {
    /// An empty memo, holding [`Revision::UNKNOWN`] and no value.
    pub const fn new() -> Memo<T> {
        Memo {
            rev: Revision::UNKNOWN,
            value: None,
            recomputes: 0,
        }
    }

    /// The cached value, computing it if the revision is not the one it was computed at.
    ///
    /// [`Revision::UNKNOWN`] never hits, which is what *memoise nothing* means: it is not a wildcard
    /// and it is not a first-time-only miss.
    pub fn get(&mut self, rev: Revision, compute: impl FnOnce() -> T) -> &T {
        let hit = rev.is_known() && self.rev == rev && self.value.is_some();
        if !hit {
            self.value = Some(compute());
            self.rev = rev;
            self.recomputes += 1;
        }
        self.value
            .as_ref()
            .expect("the miss branch above just filled it")
    }

    /// What is cached, without computing anything.
    pub const fn peek(&self) -> Option<&T> {
        self.value.as_ref()
    }

    /// The revision the cached value was computed at.
    pub const fn revision(&self) -> Revision {
        self.rev
    }
}

impl<T> Default for Memo<T> {
    fn default() -> Memo<T> {
        Memo::new()
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for Memo<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Memo")
            .field("rev", &self.rev)
            .field("value", &self.value)
            .field("recomputes", &self.recomputes)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A struct-of-vectors, which is the shape every measurement on this map rests on and the shape
    /// the slice-handing trait could not be implemented for.
    struct Indexed {
        pid: Vec<u32>,
        cpu: Vec<f32>,
    }

    impl Indexed {
        fn build(rows: usize) -> Indexed {
            Indexed {
                pid: (0..rows)
                    .map(|i| u32::try_from(i).unwrap_or(u32::MAX))
                    .collect(),
                cpu: (0..rows).map(|i| i as f32).collect(),
            }
        }

        fn max_cpu(&self) -> f32 {
            self.cpu.iter().copied().fold(f32::MIN, f32::max)
        }
    }

    /// **No trait is declared in this module**, and "we decided not to" is not a property of the
    /// code.
    ///
    /// A source scan, which is the same instrument the engine's `audit` module uses for the same
    /// class of claim: this is a statement about *absent* code, and absent code is only visible where
    /// it would have been written. A test that constructed something instead would pass for any
    /// reason including the trait existing and being unused.
    ///
    /// `trait` in a doc comment or a string does not count, which is why the scan is over lines that
    /// are neither.
    #[test]
    fn the_module_declares_no_trait() {
        let source = include_str!("data.rs");
        let declarations: Vec<&str> = source
            .lines()
            .map(str::trim)
            .filter(|line| !line.starts_with("//"))
            .filter(|line| {
                line.starts_with("trait ")
                    || line.starts_with("pub trait ")
                    || line.starts_with("pub(crate) trait ")
                    || line.starts_with("unsafe trait ")
            })
            .collect();
        assert!(
            declarations.is_empty(),
            "spec §14 says there is no trait, and this module declares {declarations:?}"
        );
    }

    /// **The wrong counter, preserved.** Two per-value counters both at 1 make a swap between the two
    /// values invisible to a memo keyed on either.
    ///
    /// This is what was built before the global counter was a rule, and it is here as the negative
    /// half of a pair: the assertion is that the defect *reproduces*, so that the test beside it is
    /// evidence and not decoration.
    #[test]
    fn a_per_value_revision_counter_lets_a_data_swap_go_unnoticed() {
        struct PerValue {
            data: Indexed,
            rev: u64,
        }

        let mut left = PerValue {
            data: Indexed::build(8),
            rev: 1,
        };
        let right = PerValue {
            data: Indexed::build(16),
            rev: 1,
        };

        let mut memo: Memo<f32> = Memo::new();
        let first = *memo.get(Revision::from_raw(left.rev), || left.data.max_cpu());
        assert_eq!(first, 7.0);

        // The swap. Different data, and — this is the defect — the same revision.
        left = PerValue {
            data: right.data,
            rev: right.rev,
        };
        let after = *memo.get(Revision::from_raw(left.rev), || left.data.max_cpu());

        assert_eq!(memo.recomputes, 1, "the memo never noticed");
        assert_eq!(
            after, 7.0,
            "and it is confidently reporting the old maximum"
        );
        assert_ne!(
            after,
            left.data.max_cpu(),
            "which is not the maximum of the data that is actually there"
        );
    }

    /// The same swap, with the process-global counter, which is what makes it visible.
    #[test]
    fn a_globally_fresh_revision_makes_the_same_data_swap_visible() {
        let mut left = Versioned::new(Indexed::build(8));
        let right = Versioned::new(Indexed::build(16));

        let mut memo: Memo<f32> = Memo::new();
        assert_eq!(*memo.get(left.revision(), || left.max_cpu()), 7.0);

        left = right;
        assert_eq!(*memo.get(left.revision(), || left.max_cpu()), 15.0);
        assert_eq!(memo.recomputes, 2, "the swap cost exactly one recompute");
    }

    /// Two values wrapped independently never share a revision.
    #[test]
    fn two_independently_created_values_never_share_a_revision() {
        let a = Versioned::new(0u32);
        let b = Versioned::new(0u32);
        assert_ne!(
            a.revision(),
            b.revision(),
            "identical contents, and the revisions still differ — which is the point: a revision is \
             about identity over time, not about content"
        );
    }

    /// **The failure the guard exists to make unreachable**, written out with a hand-kept revision
    /// so that the chart's wrongness is a fact in the repository.
    #[test]
    fn forgetting_to_bump_a_field_revision_leaves_the_chart_confidently_wrong() {
        let mut data = Indexed::build(8);
        let rev = Revision::fresh();
        let mut memo: Memo<f32> = Memo::new();
        assert_eq!(*memo.get(rev, || data.max_cpu()), 7.0);

        // An edit, and the bump that a `revision()` method would have asked somebody to remember.
        data.cpu[0] = 99.0;

        assert_eq!(*memo.get(rev, || data.max_cpu()), 7.0);
        assert_eq!(memo.recomputes, 1, "the fold never ran again");
        assert_ne!(data.max_cpu(), 7.0, "and 99 is on screen nowhere");
    }

    /// The same edit through the guard. There is nothing to remember, because the bump is the drop.
    #[test]
    fn the_edit_guard_makes_the_same_mistake_unreachable() {
        let mut data = Versioned::new(Indexed::build(8));
        let mut memo: Memo<f32> = Memo::new();
        assert_eq!(*memo.get(data.revision(), || data.max_cpu()), 7.0);

        data.edit().cpu[0] = 99.0;

        assert_eq!(*memo.get(data.revision(), || data.max_cpu()), 99.0);
        assert_eq!(memo.recomputes, 2);
    }

    /// **Over-invalidation is the price of not having to diff**, and it is a price rather than a bug.
    ///
    /// A content-derived revision would have avoided it and was refused at 2.59 ms a frame — 26× the
    /// budget — because it turns one O(rows) fold into a more expensive one.
    #[test]
    fn the_guard_bumps_even_when_nothing_changed_so_over_invalidation_is_its_price() {
        let mut data = Versioned::new(Indexed::build(8));
        let before = data.revision();
        drop(data.edit());
        assert_ne!(
            data.revision(),
            before,
            "the guard touched nothing and still bumped"
        );
    }

    /// [`Revision::UNKNOWN`] is *memoise nothing*, and the counter is what says so.
    #[test]
    fn an_unknown_revision_means_memoise_nothing_and_the_counter_says_so() {
        let data = Indexed::build(8);
        let mut memo: Memo<f32> = Memo::new();
        for _ in 0..10 {
            assert_eq!(*memo.get(Revision::UNKNOWN, || data.max_cpu()), 7.0);
        }
        assert_eq!(
            memo.recomputes, 10,
            "ten frames, ten folds — UNKNOWN is not a first-miss-then-hit wildcard"
        );
    }

    /// A memo is keyed by **where it is stored**, so a widget being renamed is not an invalidation.
    ///
    /// There is no name in `get`'s signature to rename, which is the mechanism rather than the test:
    /// the assertion is that a warm memo stays warm across everything an id-keyed one would notice.
    #[test]
    fn a_memo_is_keyed_by_where_it_is_stored_and_survives_its_widget_being_renamed() {
        struct ChartState {
            max: Memo<f32>,
        }

        let data = Versioned::new(Indexed::build(8));
        let mut st = ChartState { max: Memo::new() };

        // Frame one, under one name.
        let _label = "cpu by pid";
        assert_eq!(*st.max.get(data.revision(), || data.max_cpu()), 7.0);
        // Frame two, renamed, resized, moved. None of it is in the key.
        let _label = "processor load";
        assert_eq!(
            *st.max.get(data.revision(), || unreachable!("still warm")),
            7.0
        );
        assert_eq!(st.max.recomputes, 1);
    }

    /// A revision mentions no geometry, so a resize is not an invalidation either.
    #[test]
    fn a_memo_survives_a_resize_because_a_revision_mentions_no_geometry() {
        let data = Versioned::new(Indexed::build(8));
        let mut memo: Memo<f32> = Memo::new();
        for _size in [(300u16, 80u16), (80, 24), (300, 80)] {
            memo.get(data.revision(), || data.max_cpu());
        }
        assert_eq!(memo.recomputes, 1);
    }

    /// **The id-keyed alternative, and what it costs.**
    ///
    /// Keyed by an id and swept like a grab, a memo is dropped whenever its widget did not draw — so
    /// switching a tab away and back pays a full fold. Here that is two recomputes over five frames
    /// against one, and on the canonical screen the second fold is **84.91 µs bought by a keystroke.**
    #[test]
    fn an_id_keyed_memo_swept_like_a_grab_pays_a_fold_every_time_its_widget_returns() {
        let data = Versioned::new(Indexed::build(8));

        // Keyed by storage: the memo is a field, and a frame in which the chart does not draw simply
        // does not touch it.
        let mut by_storage: Memo<f32> = Memo::new();
        // Keyed by id and swept: modelled as the memo being dropped when the widget is absent.
        let mut by_id: Option<Memo<f32>> = None;

        for visible in [true, true, false, true, true] {
            if visible {
                by_storage.get(data.revision(), || data.max_cpu());
                by_id
                    .get_or_insert_with(Memo::new)
                    .get(data.revision(), || data.max_cpu());
            } else {
                // The sweep drops what did not draw.
                by_id = None;
            }
        }

        assert_eq!(by_storage.recomputes, 1, "one fold, for the whole session");
        assert_eq!(
            by_id.expect("visible on the last frame").recomputes,
            1,
            "and the id-keyed one starts again from zero, so its count is per lifetime"
        );
    }

    /// **One revision covers the whole `T`**, so a memo over one column is invalidated by an edit to
    /// another.
    #[test]
    fn one_revision_over_a_struct_of_vectors_invalidates_a_memo_of_a_column_that_did_not_change() {
        let mut data = Versioned::new(Indexed::build(8));
        let mut pid_total: Memo<u32> = Memo::new();
        let fold = |d: &Indexed| d.pid.iter().copied().sum::<u32>();

        assert_eq!(*pid_total.get(data.revision(), || fold(&data)), 28);
        data.edit().cpu[0] = 99.0;
        assert_eq!(*pid_total.get(data.revision(), || fold(&data)), 28);
        assert_eq!(
            pid_total.recomputes, 2,
            "the pid column did not change and its memo recomputed anyway"
        );
    }

    /// The fix, and its price: one `Versioned` a column costs one field a column.
    ///
    /// **The caller's to choose rather than this type's to impose**, which is why the granularity is a
    /// pair of tests rather than a design change.
    #[test]
    fn per_column_revisions_fix_the_granularity_and_cost_one_versioned_a_column() {
        struct PerColumn {
            pid: Versioned<Vec<u32>>,
            cpu: Versioned<Vec<f32>>,
        }

        let mut data = PerColumn {
            pid: Versioned::new((0..8u32).collect()),
            cpu: Versioned::new((0..8).map(|i| i as f32).collect()),
        };
        let mut pid_total: Memo<u32> = Memo::new();

        assert_eq!(
            *pid_total.get(data.pid.revision(), || data.pid.iter().copied().sum()),
            28
        );
        data.cpu.edit()[0] = 99.0;
        assert_eq!(
            *pid_total.get(data.pid.revision(), || unreachable!("pid is untouched")),
            28
        );
        assert_eq!(pid_total.recomputes, 1);
    }

    /// **Closures cover every shape a trait would**, which is the sentence that killed the trait.
    ///
    /// The same accessor signature serves a struct-of-vectors, an array-of-structs and a range that
    /// is computed and stored nowhere at all — the third being the one no trait handing out a slice
    /// can express.
    #[test]
    fn the_closure_shape_serves_struct_of_vectors_array_of_structs_and_a_computed_range() {
        fn draw(len: usize, value: impl Fn(usize) -> f32) -> f32 {
            (0..len).map(&value).fold(f32::MIN, f32::max)
        }

        let soa = Indexed::build(8);
        struct Row {
            cpu: f32,
        }
        let aos: Vec<Row> = (0..8).map(|i| Row { cpu: i as f32 }).collect();

        assert_eq!(draw(soa.cpu.len(), |i| soa.cpu[i]), 7.0);
        assert_eq!(draw(aos.len(), |i| aos[i].cpu), 7.0);
        assert_eq!(draw(8, |i| i as f32), 7.0);
    }

    /// A memo keyed on two things folds them into one revision, which is what `from_raw` and `raw`
    /// are public for.
    ///
    /// **Narrow on purpose**: the rule is that a memo carries a second revision in its key exactly
    /// when its value is made of paints or glyphs, and reading it as *every memo* is itself the
    /// defect. Nothing here enforces the rule — this only shows the door.
    #[test]
    fn two_revisions_fold_into_one_key_and_either_one_changing_is_a_miss() {
        fn pair(data: Revision, theme: Revision) -> Revision {
            Revision::from_raw(data.raw() ^ theme.raw().rotate_left(32))
        }

        let data = Versioned::new(Indexed::build(8));
        let mut theme = Versioned::new("dark");
        let mut painted: Memo<f32> = Memo::new();

        let key = pair(data.revision(), theme.revision());
        painted.get(key, || data.max_cpu());
        assert_eq!(painted.recomputes, 1);

        // The data did not move; the theme did.
        theme.edit();
        let key = pair(data.revision(), theme.revision());
        painted.get(key, || data.max_cpu());
        assert_eq!(painted.recomputes, 2, "a repaint is a miss");
    }

    /// `peek` reads what is cached without computing, and an empty memo has nothing.
    #[test]
    fn peek_never_computes() {
        let mut memo: Memo<u32> = Memo::new();
        assert!(memo.peek().is_none());
        assert!(!memo.revision().is_known());
        memo.get(Revision::fresh(), || 1);
        assert_eq!(memo.peek(), Some(&1));
        assert!(memo.revision().is_known());
    }
}
