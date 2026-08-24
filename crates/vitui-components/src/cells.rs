//! **The rectangle this crate can name**, and the resolution of runtime architecture issue 22 from
//! the components side.
//!
//! # The crux, stated exactly
//!
//! Spec §2's rule is *the cells a helper does not write are **named in its return value***, and spec
//! §3 spells both halves of it out as signatures: `text::fit` returns the remainder, `frame::block`
//! *"draws its frame, **returns the rectangle it did not write**"*. Neither sentence is writable as
//! Rust in this package. `Rect` is `vitui_engine::Rect`; it is named by **27 of the runtime's public
//! declarations and re-exported by none of them** (`crates/vitui-runtime/src/line.rs`,
//! `EngineName { name: "Rect", reachable_as: None }`), and constraint C6 says this crate's
//! `[dependencies]` table is `vitui-runtime` and nothing else. So `-> Rect` here is
//! `error[E0412]: cannot find type` before anything runs, and there is no second spelling: Rust has
//! no `typeof`, an `impl Trait` return is opaque to its own crate, and no public runtime trait
//! carries `Rect` as an associated type.
//!
//! Ticket 04 hit the same wall one instrument earlier and stopped: [`crate::runner::Pen`] has no
//! `fill` because `Ctx::fill` takes a `Rect`. [`crate::counters::Tally::filled`] hit it and took
//! four scalars, and said so in its own header. This module is the third sighting, and the first one
//! that is on the *component-facing* surface rather than on an instrument — which is where issue 22
//! predicted it would become load-bearing: *"what is blocked is the first component, which is the
//! next crate to be written."*
//!
//! # Four candidate answers, and why this is the one
//!
//! 1. **Re-export `Rect` from the runtime.** Issue 22's first bullet, and it is a *runtime* decision
//!    taken on a runtime map; this ticket may not take it, and the constraint it would move is
//!    checked by a test in another crate.
//! 2. **Depend on `vitui-engine` for the vocabulary types.** Issue 22's third bullet. It reopens C6,
//!    which `deny.toml` enforces, and it makes *the runtime is replaceable on the same engine* a
//!    claim about two crates rather than one.
//! 3. **Hand the interior to a closure** — `block(cx, opts, |cx| …)` — so nothing is ever named.
//!    It matches ADR 0012's shape and it is the candidate this ticket was written expecting.
//!    **It is refused, on two measurements**; see [`crate::frame`]'s header, which carries both.
//! 4. **This crate names its own rectangle.** What ships.
//!
//! # What `Cells` is
//!
//! A rectangle of cells **in the coordinates of the [`Ctx`] it came from**, which is the same
//! coordinate system [`Ctx::area`] hands back and the same one [`crate::counters::Tally`] unions its
//! spans in. It is four scalars — the shape `Tally::filled` already takes — with the arithmetic
//! attached, and the two verbs that need a real `Rect` build one **by inference** at the call site:
//! `layout::rect::shrink(cx.area(), …)` mentions no type name, which is the loophole issue 22
//! already documented (*"inference reaches what naming cannot"*).
//!
//! **The name is not `Rect` and that is deliberate.** A second `Rect` in the workspace would make
//! every mismatch read `expected `Rect`, found `Rect``, which is the worst diagnostic Rust
//! produces. The name that ships is the one ADR 0026's own sentence uses — *the **cells** it does
//! not write are named in its return value* — so a signature returning `Cells` reads as the rule.
//!
//! # The arithmetic is a second implementation, and it is gated against the first
//!
//! Every operator here duplicates one in [`vitui_runtime::layout::rect`], down to the saturating
//! behaviour: `shrink` moves the origin by `l` and `t` **even when the size collapses**, because
//! that is what the runtime's does and *"two intersections that disagree by one cell is exactly the
//! class of defect this module exists to avoid."* The duplication is not a preference — the
//! runtime's operators take and return `Rect`, so this crate cannot wrap them in anything with a
//! signature. What it can do is *compare*, and `tests::the_algebra_agrees_with_the_runtimes` sweeps
//! every operator over a corpus and asserts field-for-field equality with the runtime's answer,
//! reached by inference inside the test body. That is the engine's `reference.rs` arrangement one
//! layer up: a second implementation is allowed when an equality holds it to the first.

use vitui_runtime::{Ctx, Id, Interest, Paint, Response, Role};

/// **A rectangle of cells, in the coordinates of the [`Ctx`] it came from.**
///
/// The crate's rectangle vocabulary; see this module's header for why it exists and why it is not
/// called `Rect`.
///
/// # It belongs to one context and carries no proof of it
///
/// A `Cells` is origin-relative to a particular `Ctx`, exactly as [`Ctx::area`]'s result is, and
/// nothing in the type records which one. Two sibling `Ctx::child` contexts hand out the same local
/// `(x, y)` for two different screen cells — [`crate::counters::Tally`] states the same caveat for
/// the same reason, and it is the same rectangle spec §2's partition rule is about.
///
/// Coordinates are `u16` rather than `i32`: a `Cells` is always **inside** the rectangle a component
/// was handed, which is what the partition rule is a statement about, and a negative origin would be
/// a component drawing outside its own rectangle — already the failure the counter is measuring.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Hash)]
pub struct Cells {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

/// Build the `Rect` a runtime verb wants, **without naming its type**.
///
/// The whole bridge, in one macro, and a macro rather than a function because a function would need
/// a return type and there is no type name to write. `$cx`'s own area is the base — [`Ctx::area`] is
/// documented to be origin-relative, so its `x` and `y` are zero — and the shrink puts the sub-
/// rectangle where the `Cells` says, clamped to the context by the saturating arithmetic the runtime
/// already does.
macro_rules! rect_in {
    ($cx:expr, $cells:expr) => {{
        let cells = $cells;
        let area = $cx.area();
        vitui_runtime::layout::rect::shrink(
            area,
            cells.x(),
            cells.y(),
            area.w.saturating_sub(cells.x().saturating_add(cells.w())),
            area.h.saturating_sub(cells.y().saturating_add(cells.h())),
        )
    }};
}

impl Cells {
    /// A rectangle at `(x, y)`, `w` by `h`.
    pub const fn at(x: u16, y: u16, w: u16, h: u16) -> Cells {
        Cells { x, y, w, h }
    }

    /// **The whole of a context**, which is where every partition on this map starts.
    ///
    /// The one constructor a component calls: a component is handed a `Ctx` and its own rectangle is
    /// `Cells::of(cx)`.
    pub fn of(cx: &Ctx<'_, '_>) -> Cells {
        let (w, h) = cx.size();
        Cells { x: 0, y: 0, w, h }
    }

    /// Its left edge.
    pub const fn x(self) -> u16 {
        self.x
    }

    /// Its top edge.
    pub const fn y(self) -> u16 {
        self.y
    }

    /// Its width.
    pub const fn w(self) -> u16 {
        self.w
    }

    /// Its height.
    pub const fn h(self) -> u16 {
        self.h
    }

    /// One past its right edge.
    pub const fn right(self) -> u16 {
        self.x.saturating_add(self.w)
    }

    /// One past its bottom edge.
    pub const fn bottom(self) -> u16 {
        self.y.saturating_add(self.h)
    }

    /// **How many cells it holds** — the right-hand side of §2's second equality, `distinct cells
    /// touched == area.w * area.h`.
    ///
    /// `u64` because a 300×80 screen is 24 000 and the counters it is compared against are `u64`;
    /// widening at the source rather than at every comparison.
    pub const fn count(self) -> u64 {
        self.w as u64 * self.h as u64
    }

    /// Whether it holds no cells at all.
    ///
    /// A zero-width or zero-height rectangle is a **legitimate** answer and not an error: `block` on
    /// a two-row rectangle returns one, and the caller writing nothing into it is the partition
    /// holding rather than failing.
    pub const fn is_empty(self) -> bool {
        self.w == 0 || self.h == 0
    }

    /// Whether `(x, y)` is inside it.
    pub const fn contains(self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.right() && y >= self.y && y < self.bottom()
    }

    /// Shrink by `n` on all four sides. [`vitui_runtime::layout::rect::inset`]'s twin.
    pub const fn inset(self, n: u16) -> Cells {
        self.shrink(n, n, n, n)
    }

    /// Shrink by a different amount on each side, in the order left, top, right, bottom.
    ///
    /// **The origin moves even when the size collapses**, which is the runtime's own stated
    /// behaviour: an empty rectangle *where the inset asked for one*, rather than an empty rectangle
    /// somewhere else.
    pub const fn shrink(self, l: u16, t: u16, r: u16, b: u16) -> Cells {
        Cells {
            x: self.x.saturating_add(l),
            y: self.y.saturating_add(t),
            w: self.w.saturating_sub(l.saturating_add(r)),
            h: self.h.saturating_sub(t.saturating_add(b)),
        }
    }

    /// Cut `n` rows off the top: `(top, rest)`. [`vitui_runtime::layout::rect::split_at_v`]'s twin.
    ///
    /// **The primitive `fit` is built on.** `n` is clamped, and the remainder is positioned *after*
    /// the whole thing, so a caller that keeps splitting the rest keeps getting empty rectangles in
    /// the right place — which is what makes a chain of `fit` calls terminate correctly on a
    /// rectangle it has run out of.
    pub const fn split_at_v(self, n: u16) -> (Cells, Cells) {
        let cut = if n < self.h { n } else { self.h };
        (
            Cells {
                x: self.x,
                y: self.y,
                w: self.w,
                h: cut,
            },
            Cells {
                x: self.x,
                y: self.y + cut,
                w: self.w,
                h: self.h - cut,
            },
        )
    }

    /// Cut `n` columns off the left: `(left, rest)`.
    pub const fn split_at_h(self, n: u16) -> (Cells, Cells) {
        let cut = if n < self.w { n } else { self.w };
        (
            Cells {
                x: self.x,
                y: self.y,
                w: cut,
                h: self.h,
            },
            Cells {
                x: self.x + cut,
                y: self.y,
                w: self.w - cut,
                h: self.h,
            },
        )
    }

    /// The overlap of two rectangles, empty when they do not overlap.
    ///
    /// **What a partition gate asks of two branches**: two members of a partition intersect in
    /// nothing, and a helper that hands back a remainder overlapping what it wrote has broken §2
    /// whatever its write count says.
    pub const fn intersect(self, other: Cells) -> Cells {
        let x = if self.x > other.x { self.x } else { other.x };
        let y = if self.y > other.y { self.y } else { other.y };
        let right = if self.right() < other.right() {
            self.right()
        } else {
            other.right()
        };
        let bottom = if self.bottom() < other.bottom() {
            self.bottom()
        } else {
            other.bottom()
        };
        Cells {
            x,
            y,
            w: right.saturating_sub(x),
            h: bottom.saturating_sub(y),
        }
    }

    /// **Fill it.** One of the two verbs that needs a real `Rect`, built by this module's
    /// private `rect_in!`.
    ///
    /// A component calls this on the rectangle it *owns* and never on one it was handed by a helper
    /// — that is ADR 0026's *"the rectangle and the content are told apart"*, and a `block` calling
    /// it on the interior it is about to return is the 22 200-cell defect [`crate::frame`] measures.
    pub fn fill(self, cx: &mut Ctx<'_, '_>, cluster: &str, st: Paint) {
        if self.is_empty() {
            return;
        }
        cx.fill(rect_in!(cx, self), cluster, st);
    }

    /// **Narrow a context to it**, and run `f` inside the narrowed one.
    ///
    /// # This is the caller's verb and never a container's
    ///
    /// [`crate::frame::block`] returns its interior rather than handing it to a closure, and this
    /// method is half of why it can: a caller that wants the clip narrowed writes it, one line, at
    /// the call site. What a container may not do is narrow *on the caller's behalf*, because
    /// `CONTEXT.md`'s identity rule is that **a container that returns a rectangle preserves its
    /// children's identity and one that takes a closure renames them** — and because a narrowed
    /// context moves the origin, which puts the caller's writes in a different coordinate system
    /// from the container's and makes [`crate::counters::Tally`] unable to see the pair
    /// (`crate::frame`'s header carries the count).
    ///
    /// The closure is higher-ranked over the inner context's lifetime, which is
    /// [`Ctx::with_key`]'s own shape.
    pub fn child<'f, R>(self, cx: &mut Ctx<'f, '_>, f: impl FnOnce(&mut Ctx<'f, '_>) -> R) -> R {
        let r = rect_in!(cx, self);
        let mut inner = cx.child(r);
        f(&mut inner)
    }

    /// **Declare it as an interactive region.** The other verb that needs a real `Rect`.
    ///
    /// The id is the caller's, taken **outside every closure** — ADR 0027 — which is why this takes
    /// one rather than minting it: a `#[track_caller]` mint here would name the line in this file
    /// and collide every widget in the application on it.
    pub fn interact(self, cx: &mut Ctx<'_, '_>, id: Id, interest: Interest) -> Response {
        cx.interact(id, rect_in!(cx, self), interest)
    }

    /// **A [`Response`] that names this rectangle and declares nothing.** The fourth verb that needs
    /// a real `Rect`.
    ///
    /// # Why a pure drawer needs this at all
    ///
    /// Spec §1's rule 4 is *return `Response`, **even from a pure drawer***, and a label is the pure
    /// drawer the rule is about. The obvious spelling — `interact(cx, id, Interest::NONE)` — is a
    /// different statement: `Interest::NONE` is documented as *hit-tested and nothing more*, so it
    /// appends a hit entry, takes the id, and sits in the index between the pointer and whatever is
    /// under it. **A label that swallows a click is not a pure drawer**, and on
    /// [`crate::dense`]'s screen it would move the region count from 338 to 560 — one entry per
    /// label, one per reading, and none of them wanted.
    ///
    /// So the two are kept apart: `None` interest means *no region*, and `Some(Interest::NONE)`
    /// means *a region that asks for nothing and is still in the index*. Both are reachable through
    /// [`crate::text::TextOpts::interest`], and neither is the other's default.
    pub fn inert(self, cx: &Ctx<'_, '_>, id: Id) -> Response {
        Response::inert(id, rect_in!(cx, self))
    }

    /// **Declare the face this rectangle is to be awarded if it wins the hover.** The third verb
    /// that needs a real `Rect`.
    ///
    /// [`Ctx::hover_style`] is the runtime's *deferred hover award*: the intent is declared during
    /// the draw and applied at `end`, from the index that has just drawn, so it lands on the first
    /// frame of an overlap rather than a frame later.
    ///
    /// # It is not a component's verb, and there is one caller
    ///
    /// ADR 0026: *a restyle is free only when the component's own next draw already produces the
    /// value the restyle produced*. That is a relation between **two** statements — the face drawn
    /// and the face awarded — and a component that writes both writes them separately. So this
    /// method exists for [`crate::state::press`] and for the fixture kept beside it, and
    /// `state::tests::the_hover_award_is_declared_in_one_file_and_it_is_the_one_that_collapses_it`
    /// is what keeps it that way: a second caller is a second place the pair can disagree.
    pub fn hover_style(self, cx: &mut Ctx<'_, '_>, resp: &Response, role: Role) {
        if self.is_empty() {
            return;
        }
        cx.hover_style(resp, rect_in!(cx, self), role);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vitui_runtime::ctx::Driver;
    use vitui_runtime::layout::rect;

    /// A corpus that reaches every clamp: empty rectangles, one-cell rectangles, and shrinks larger
    /// than the thing being shrunk.
    fn corpus() -> Vec<Cells> {
        let mut out = Vec::new();
        for x in [0u16, 1, 7] {
            for y in [0u16, 1, 7] {
                for w in [0u16, 1, 3, 40] {
                    for h in [0u16, 1, 3, 10] {
                        out.push(Cells::at(x, y, w, h));
                    }
                }
            }
        }
        out
    }

    /// Compare a [`Cells`] against a runtime rectangle **without naming the rectangle's type**.
    ///
    /// A macro rather than a helper `fn` or a closure for the reason this whole module exists: both
    /// of those need the parameter written down, and `Rect` cannot be written down here. A macro
    /// expands before anything needs a name.
    macro_rules! same {
        ($ours:expr, $theirs:expr, $what:expr) => {{
            let ours: Cells = $ours;
            let theirs = $theirs;
            assert_eq!(
                (i32::from(ours.x()), i32::from(ours.y()), ours.w(), ours.h()),
                (theirs.x, theirs.y, theirs.w, theirs.h),
                "{}: `Cells` and `layout::rect` disagree",
                $what
            );
        }};
    }

    /// **The second implementation is held to the first by an equality**, which is the only thing
    /// that makes a second implementation allowed (`crate::runner`'s oracle rule, and the engine's
    /// `reference.rs` before it).
    ///
    /// Every operator, over the whole corpus, against [`vitui_runtime::layout::rect`]'s answer. The
    /// runtime's `Rect` is never *named* here — the base comes out of [`rect_in`] and everything
    /// after it is inferred, which is issue 22's own observation that inference reaches what naming
    /// cannot. The context is 200×200 so that no corpus rectangle is clamped by the bridge and the
    /// comparison is about the algebra rather than about the clip.
    #[test]
    fn the_algebra_agrees_with_the_runtimes() {
        let mut driver = Driver::headless(200, 200).expect("a sink cannot fail to attach");
        driver.frame(|cx| {
            for c in corpus() {
                let base = rect_in!(cx, c);
                for n in [0u16, 1, 2, 5, 100] {
                    same!(c.inset(n), rect::inset(base, n), "inset");
                    same!(
                        c.shrink(n, 0, 0, 0),
                        rect::shrink(base, n, 0, 0, 0),
                        "shrink l"
                    );
                    same!(
                        c.shrink(0, n, 0, 0),
                        rect::shrink(base, 0, n, 0, 0),
                        "shrink t"
                    );
                    same!(
                        c.shrink(1, n, 2, n),
                        rect::shrink(base, 1, n, 2, n),
                        "shrink all"
                    );

                    let (top, rest) = c.split_at_v(n);
                    let (rtop, rrest) = rect::split_at_v(base, n);
                    same!(top, rtop, "split_at_v top");
                    same!(rest, rrest, "split_at_v rest");

                    let (left, right) = c.split_at_h(n);
                    let (rleft, rright) = rect::split_at_h(base, n);
                    same!(left, rleft, "split_at_h left");
                    same!(right, rright, "split_at_h right");
                }
                for other in corpus() {
                    let b = rect_in!(cx, other);
                    let ours = c.intersect(other);
                    let theirs = rect::intersect(base, b);
                    // An empty intersection keeps whichever origin the clamp produced, and the two
                    // clamps need not agree on where nothing is. Both are empty or both are not,
                    // and a non-empty one agrees exactly.
                    assert_eq!(
                        ours.is_empty(),
                        theirs.w == 0 || theirs.h == 0,
                        "intersect: emptiness disagrees"
                    );
                    if !ours.is_empty() {
                        same!(ours, theirs, "intersect");
                    }
                }
            }
        });
    }

    /// **The bridge lands where the arithmetic said it would**, which is the half no algebra test
    /// can reach: `rect_in!` is what turns a `Cells` back into something a runtime verb accepts, and
    /// it is only correct if a context's own area really is origin-relative.
    #[test]
    fn the_bridge_puts_a_rectangle_where_the_cells_say() {
        let mut driver = Driver::headless(40, 12).expect("a sink cannot fail to attach");
        driver.frame(|cx| {
            for c in [
                Cells::at(0, 0, 40, 12),
                Cells::at(3, 2, 10, 4),
                Cells::at(39, 11, 1, 1),
                Cells::at(38, 10, 10, 10),
            ] {
                let r = rect_in!(cx, c);
                assert_eq!(r.x, i32::from(c.x()));
                assert_eq!(r.y, i32::from(c.y()));
                // Clamped to the context, which is what the last case is for: a `Cells` reaching
                // past the edge comes back cropped rather than wrong.
                assert_eq!(r.w, c.w().min(40 - c.x()));
                assert_eq!(r.h, c.h().min(12 - c.y()));
            }
        });
    }

    /// **A split is a partition**: the two halves cover the whole and overlap in nothing.
    ///
    /// The property every helper on this map returns its remainder under, asserted once here so the
    /// helpers can lean on it rather than restate it.
    #[test]
    fn a_split_covers_the_whole_and_overlaps_in_nothing() {
        for c in corpus() {
            for n in [0u16, 1, 2, 5, 100] {
                let (a, b) = c.split_at_v(n);
                assert_eq!(
                    a.count() + b.count(),
                    c.count(),
                    "split_at_v does not cover"
                );
                assert!(a.intersect(b).is_empty(), "split_at_v overlaps");
                let (a, b) = c.split_at_h(n);
                assert_eq!(
                    a.count() + b.count(),
                    c.count(),
                    "split_at_h does not cover"
                );
                assert!(a.intersect(b).is_empty(), "split_at_h overlaps");
            }
        }
    }
}
