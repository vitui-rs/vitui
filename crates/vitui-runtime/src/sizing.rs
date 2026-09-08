//! Sizing: a plain function beside a component, and the dry run kept only as the detector.
//!
//! There is no measure pass. **This module declares no trait and no type a
//! component has to implement**, and that is the whole of it: a sizing function is a *shape*.
//!
//! ```text
//! fn detail_height(data: &Detail, width: u16) -> u16
//! //               ^^^^^^^^^^^^^  ^^^^^^^^^^     ^^^
//! //               the same &data the component takes
//! //                              the dimension it is about to be given
//! //                                             integers
//! ```
//!
//! It **takes no `Ctx`**, so it cannot draw, cannot claim an identity and cannot route — which is
//! the whole of what makes it a function rather than a method on a trait. There is nothing to
//! `impl`, nothing to register and nothing to name it in: a container that sizes to its contents
//! calls the function that sits beside the component it is about to draw.
//!
//! # The stated cost of this exit is not its real cost
//!
//! Sizing a dialog to its contents is **569 ns**, and every survey container that needs sizing needs
//! it for text or for a list, both arithmetic. The real cost is **drift**: a sizing function and its
//! component are two expressions of one layout with nothing holding them together. Runtime ticket
//! 01's `detail_height` disagreed with its own component at **221 of 229 widths, including the 78
//! every number in that ticket was measured at, and three tickets did not notice.**
//!
//! Two things follow, and they are obligations rather than advice:
//!
//! 1. **The component lays itself out from the arithmetic the sizing function publishes**, so there
//!    is one expression and not two.
//! 2. **The dry run survives as the detector**, and [`check`] is it — owed by every component that
//!    publishes a sizing function. It ships here rather than in the components crate so that a
//!    component author writes one line and not a harness.
//!
//! # Why the trait died
//!
//! It was built and **does not die of cost**: same rectangles at **1.03×**, and **0 allocations
//! against 2** once it takes the caller's buffers. It dies of two things, and both are negative
//! cases here rather than sentences — [`Extent`] carries the `E0499`, and
//! `tests::a_virtualised_child_has_no_honest_measure_answer` carries the other.
//!
//! - **`E0499`.** A fit-container holds every child alive across its measure loop, so two children
//!   writing through one borrow cannot coexist. **It compiles on disjoint fields, which is why it is
//!   easy to miss** — so the pair here is a failing case *and* a passing one, and the passing one is
//!   the reason the failing one is worth writing down.
//! - **`measure` has no answer for a virtualised child.** Honest is `u16::MAX` — a rectangle nothing
//!   can draw, and **still 15× short of a million rows** — and the alternative is the parameter it
//!   was handed, which is the container asking itself.
//!
//! # Why the dry run is a detector and never the layout mechanism
//!
//! Three numbers and one design fact, and [`crate::ctx::Ctx::measured`] repeats them where somebody
//! about to reach for it will read them:
//!
//! - it is **a second whole frame**: dense screen **2.07×, +34.89 µs**, against **85.79–569.00 ns**
//!   for the sizing functions — **13.3×** for the same answer;
//! - it **measures the clip, not the content**: a 1M-row list under an 80-row clip reports 80, and
//!   the honest question (65 535 rows, all `u16` can express) costs **5.81 ms against 0.96 ns —
//!   6 051 331×**;
//! - **a frame has side effects that now happen twice**: a log tail reaches `tick = 2` where the
//!   author wrote 1, **with same-looking output**;
//! - **the measure pass needs its own `Frame`**, or it claims every id twice, doubles the hit index
//!   and drains the key queue — so **the measured world has no focus, no hover, no press and no
//!   keys**, which limits what it may be *asked*.
//!
//! The second of those is also what bounds this module's detector: it is for **bounded** content.
//! A virtualised collection's sizing function is arithmetic over a length `u16` cannot hold, so
//! there is no measured world to compare it against and no honest `measure` either. That is one
//! limit with two names, not two limits.
//!
//! # The one expensive sizing function is not a sizing question
//!
//! Column auto-fit is **17.60 µs at 1k and 17.69 ms at 1M**. It is the data contract's derived
//! aggregate, memoised at **1.73 ns — 10 231 195×**. [`auto_fit`] is the fold and it is deliberately
//! not memoised *here*: this exit keeps the fold **visible at the call site**, where a trait would
//! have hidden it inside a container.

use std::fmt;

use vitui_engine::Rect;

use crate::ctx::{Ctx, Driver};
use crate::layout::text;

/// The drawn extent: **how far the verbs in a body reached**, on both axes.
///
/// The two numbers are sizes rather than coordinates — `h` is one past the last row anything was
/// offered to — because that is what a sizing function returns and comparing the two is the point.
///
/// # What is recorded is what a verb **offered**, not what it managed to write
///
/// A verb that runs past its clip is clamped and discarded by the engine, and recording
/// the clamped result would make the extent agree with the clip by construction — which is the
/// failure the glossary names as *the extent going blind sideways*. So a `text` at column 30 of a
/// 20-column body reaches 30 plus its width, and the detector below sees a component drawing outside
/// what its sizing function claimed.
///
/// # It is maintained only while the frame is asked to maintain it
///
/// Measuring the display width of every verb's string is a grapheme walk a verb would not otherwise
/// do — **7% of the frame budget** — so a real frame does not do it. Today the only thing that asks
/// is [`crate::ctx::Ctx::measured`]; a scroll area over bounded content is the second caller and
/// reads it one frame late.
///
/// # The trait form's first death, which is a compile outcome
///
/// A fit-container holds every child alive across its measure loop, so two children writing through
/// one borrow cannot coexist:
///
/// ```compile_fail,E0499
/// trait Measure {
///     fn measure(&mut self, w: u16) -> u16;
/// }
///
/// struct Log {
///     lines: Vec<String>,
/// }
/// struct Tail<'a>(&'a mut Log);
/// impl Measure for Tail<'_> {
///     fn measure(&mut self, _w: u16) -> u16 {
///         self.0.lines.push(String::from("drawn"));
///         self.0.lines.len() as u16
///     }
/// }
///
/// let mut log = Log { lines: Vec::new() };
/// // A fit-container holds every child alive across its measure loop, which is what a measure
/// // phase *is*: measure all of them, then lay out, then draw all of them.
/// let mut children: Vec<Box<dyn Measure>> =
///     vec![Box::new(Tail(&mut log)), Box::new(Tail(&mut log))];
/// let total: u16 = children.iter_mut().map(|c| c.measure(20)).sum();
/// let _ = total;
/// ```
///
/// **and it compiles on disjoint fields, which is why it is easy to miss.** This one passes, and it
/// is the reason the case above is worth writing down at all — a survey built from panes over
/// separate fields never meets the error:
///
/// ```
/// trait Measure {
///     fn measure(&mut self, w: u16) -> u16;
/// }
///
/// struct App {
///     left: Vec<String>,
///     right: Vec<String>,
/// }
/// struct Pane<'a>(&'a mut Vec<String>);
/// impl Measure for Pane<'_> {
///     fn measure(&mut self, _w: u16) -> u16 {
///         self.0.len() as u16
///     }
/// }
///
/// let mut app = App { left: Vec::new(), right: Vec::new() };
/// let mut children: Vec<Box<dyn Measure>> =
///     vec![Box::new(Pane(&mut app.left)), Box::new(Pane(&mut app.right))];
/// assert_eq!(children.iter_mut().map(|c| c.measure(20)).sum::<u16>(), 0);
/// ```
///
/// **Protects:** `sizing::check`, `Agreement` — and not `Extent`, which is where this pair *sits*
/// rather than what it is *about*. The `E0499` above is the trait that was not built (the module
/// comment says so in as many words), so a rename of `check` or `Agreement` is what would leave the
/// hostile half passing for the wrong reason. The twin gate reads this line.
///
/// and the twin that names what shipped instead, by path: two children over **one** `&mut`, drawn
/// one after another, because a sizing function does not hold a child at all.
///
/// ```
/// use vitui_runtime::sizing::{self, Agreement};
///
/// // Named by path and pinned to its signature: a rename fails HERE rather than quietly turning
/// // the `compile_fail` above into a case that passes because the item vanished.
/// fn protected(widths: std::ops::RangeInclusive<u16>) -> Agreement {
///     sizing::check(widths, |_w| 1, |cx| {
///         let body = cx.theme().paint(vitui_runtime::Role::Body);
///         cx.text(0, 0, "one row", body);
///     })
/// }
/// assert!(protected(4..=40).agrees());
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Extent {
    /// How many columns were reached.
    pub w: u16,
    /// How many rows were reached.
    pub h: u16,
}

impl Extent {
    /// Nothing was drawn.
    pub const ZERO: Extent = Extent { w: 0, h: 0 };

    /// Take in one verb's offered rectangle, given the body-relative origin of the context it was
    /// issued in.
    ///
    /// **A zero-width or zero-height rectangle reaches nothing.** `cx.text(x, y, "", st)` writes no
    /// cell, and counting its row would make an empty label as tall as a full one.
    pub(crate) fn reach(&mut self, origin: (i32, i32), r: Rect) {
        if r.w == 0 || r.h == 0 {
            return;
        }
        let right = origin.0.saturating_add(r.right()).max(0);
        let bottom = origin.1.saturating_add(r.bottom()).max(0);
        self.w = self.w.max(u16::try_from(right).unwrap_or(u16::MAX));
        self.h = self.h.max(u16::try_from(bottom).unwrap_or(u16::MAX));
    }
}

/// What a body asked the measured world for that the measured world cannot answer.
///
/// **The measured world has no focus, no hover, no press and no keys** — it gets its own `Frame`,
/// and a fresh `Frame` has none of the four. A component that reads one of them gets the same
/// default it would get from a frame where nothing was hovered and nothing was focused, which is a
/// *silent* answer to a question that had no answer.
///
/// This is what makes it loud: [`check`] carries the flags into its failure, so a disagreement
/// caused by a component sizing itself from an interaction is named as one rather than left as an
/// unexplained pair of numbers.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Consulted {
    /// The body declared an interactive region and read the response — hover, press or focus.
    pub interaction: bool,
    /// The body asked for a key.
    pub keys: bool,
}

impl Consulted {
    /// Nothing was asked.
    pub const NONE: Consulted = Consulted {
        interaction: false,
        keys: false,
    };

    /// Whether anything was.
    pub const fn any(self) -> bool {
        self.interaction || self.keys
    }

    /// The union, which is what a sweep accumulates.
    pub const fn with(self, other: Consulted) -> Consulted {
        Consulted {
            interaction: self.interaction || other.interaction,
            keys: self.keys || other.keys,
        }
    }
}

impl fmt::Display for Consulted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.interaction, self.keys) {
            (false, false) => f.write_str("nothing"),
            (true, false) => f.write_str("an interaction"),
            (false, true) => f.write_str("the keyboard"),
            (true, true) => f.write_str("an interaction and the keyboard"),
        }
    }
}

/// One body drawn into a discard surface: what it returned, how far it reached, what it asked for.
///
/// Returned by [`crate::ctx::Ctx::measured`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Measured<R> {
    /// What the body returned.
    pub value: R,
    /// How far its verbs reached.
    pub extent: Extent,
    /// What it asked the measured world for that the measured world could not answer.
    pub consulted: Consulted,
}

/// One width at which a sizing function and its component disagreed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Disagreement {
    /// The width both were given.
    pub width: u16,
    /// What the sizing function claimed.
    pub claimed: u16,
    /// How far the component actually reached.
    pub drew: u16,
}

impl fmt::Display for Disagreement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "at width {}: the sizing function claimed {} rows, the component drew {}",
            self.width, self.claimed, self.drew
        )
    }
}

/// The result of a sweep: **an equality per width**, and every width it failed at.
///
/// Every width is checked rather than stopping at the first, because *221 of 229* and *1 of 229* are
/// different defects and the count is the difference — a component that is wrong everywhere has a
/// different bug from one that is wrong at the width where a word stops fitting.
#[derive(Clone, Debug, Default)]
pub struct Agreement {
    /// How many widths were swept.
    pub widths: u32,
    /// The ones that disagreed, in sweep order.
    pub disagreements: Vec<Disagreement>,
    /// What the component asked the measured world for, over the whole sweep.
    pub consulted: Consulted,
}

impl Agreement {
    /// Whether the sizing function agreed with its component at every width.
    pub fn agrees(&self) -> bool {
        self.disagreements.is_empty()
    }

    /// Fail the test, naming the count, the first width and what the component asked for.
    ///
    /// **The count comes first** because it is the part that says which defect this is, and the
    /// consulted line is here because a component that sized itself from a hover would otherwise
    /// disagree for a reason nothing on the screen explains.
    #[track_caller]
    pub fn assert(&self) {
        if self.agrees() {
            return;
        }
        let first = self
            .disagreements
            .first()
            .expect("a disagreement exists, or `agrees` would have returned");
        let mut message = format!(
            "the sizing function disagreed with its component at {} of {} widths\n         first {first}",
            self.disagreements.len(),
            self.widths,
        );
        if self.consulted.any() {
            message.push_str(&format!(
                "\n         the component consulted {} in the measured world, where there is \
                 none of either — a component that sizes itself from an interaction cannot be \
                 checked this way",
                self.consulted
            ));
        }
        panic!("{message}");
    }
}

/// **The detector: a sizing function against its component, over a width sweep.**
///
/// This is the gate every component that publishes a sizing function owes. It draws the component
/// into a discard surface of **exactly the claimed size** and compares the drawn extent with the
/// claim, at every width.
///
/// # Examples
///
/// ```
/// use vitui_runtime::layout::text;
/// use vitui_runtime::sizing;
///
/// // The sizing function: the same `&data`, the width it is about to be given, integers out.
/// fn paragraph_height(data: &str, width: u16) -> u16 {
///     u16::try_from(text::wrap_height(data, width)).unwrap_or(u16::MAX)
/// }
///
/// // The component, laid out from the arithmetic its sizing function publishes.
/// fn paragraph(cx: &mut vitui_runtime::Ctx<'_, '_>, data: &str) {
///     let body = cx.theme().paint(vitui_runtime::Role::Body);
///     for (row, line) in text::wrap(data, cx.area().w).enumerate() {
///         cx.text(0, row as i32, line, body);
///     }
/// }
///
/// let data = "the detector is one line in a component's own test module";
/// sizing::check(
///     8..=60,
///     |w| paragraph_height(data, w),
///     |cx| paragraph(cx, data),
/// )
/// .assert();
/// ```
///
/// # Exactly the claimed size, and not one row of slack
///
/// Both directions of drift have to fail, and a generous surface only catches one of them. A
/// component that draws **too little** — a claim with a border in it the component forgot to draw —
/// leaves the extent short of the claim, and a component that draws **too much** runs past the clip
/// and is recorded anyway, because [`Extent`] records what a verb offered rather than what it
/// wrote. A surface sized to the claim makes both an equality against the same number.
///
/// # What it cannot check
///
/// A **virtualised** collection, and by construction rather than by omission: its sizing function is
/// `len × row_height` over a length `u16` cannot hold, so there is no discard surface to draw it
/// into. That is the same limit as the trait form's second death and it is not a coincidence — see
/// this module's documentation.
///
/// A component that **sizes itself from an interaction** is the other one, and it is reported rather
/// than silently wrong: see [`Consulted`].
pub fn check<S, C>(widths: impl IntoIterator<Item = u16>, mut claim: S, mut draw: C) -> Agreement
where
    S: FnMut(u16) -> u16,
    C: FnMut(&mut Ctx<'_, '_>),
{
    let sweep: Vec<u16> = widths.into_iter().collect();
    let mut out = Agreement::default();
    // **A driver of its own, so a caller writes one line.** The measured world needs an `Env` — a
    // component reads the theme — and an `Env` is reachable only from inside a frame. The screen is
    // 1×1 because nothing is drawn on it: every verb here goes into the discard surface.
    let mut driver = Driver::headless(1, 1).expect("a sink cannot fail to attach");
    driver.frame(|cx| {
        for &w in &sweep {
            let claimed = claim(w);
            let measured = cx.measured(w, claimed, |inner: &mut Ctx<'_, '_>| draw(inner));
            out.widths += 1;
            out.consulted = out.consulted.with(measured.consulted);
            if measured.extent.h != claimed {
                out.disagreements.push(Disagreement {
                    width: w,
                    claimed,
                    drew: measured.extent.h,
                });
            }
        }
    });
    out
}

/// **Column auto-fit: the one expensive sizing function, and it is not a sizing question.**
///
/// The widest cell in a column, which is what a table's *fit to contents* means. It is a fold over
/// the whole column — **17.60 µs at 1k rows and 17.69 ms at 1M** — and the answer is not to make the
/// fold cheaper. It is the data contract's derived aggregate: memoised against the data's revision
/// it is **1.73 ns**, a ratio of **10 231 195×**, and it recomputes when the data changes rather
/// than when a frame runs.
///
/// # The fold is visible at the call site, and that is the decision
///
/// This function does not memoise, and the memo does not live in a container. A measure pass would
/// have hidden this fold inside one — a container calling `measure` on a column has no idea it just
/// walked a million rows — and the whole of exit (a) is that the caller can see what it is asking
/// for.
///
/// # Examples
///
/// ```
/// use vitui_runtime::data::{Memo, Versioned};
/// use vitui_runtime::sizing;
///
/// #[derive(Default)]
/// struct TableState {
///     /// The fold, parked where the application can see it.
///     name_column: Memo<u16>,
/// }
///
/// let rows = Versioned::new(vec![String::from("alice"), String::from("bartholomew")]);
/// let mut state = TableState::default();
///
/// // The call site: a revision, and a closure that folds. Two frames, one fold.
/// for _frame in 0..2 {
///     let width = *state
///         .name_column
///         .get(rows.revision(), || sizing::auto_fit(rows.iter().map(String::as_str)));
///     assert_eq!(width, 11);
/// }
/// assert_eq!(state.name_column.recomputes, 1);
///
/// // And it recomputes when the data changes, which is the only thing that should move it.
/// let mut rows = rows;
/// rows.edit().push(String::from("christopher-robin"));
/// let width = *state
///     .name_column
///     .get(rows.revision(), || sizing::auto_fit(rows.iter().map(String::as_str)));
/// assert_eq!((width, state.name_column.recomputes), (17, 2));
/// ```
///
/// # It measures columns, not bytes
///
/// Every cell goes through the runtime's text measurement, so a CJK cell is two columns per cluster
/// and a family emoji is two for the whole thing. `len()` would fit the column to the wrong number
/// on every non-ASCII table.
pub fn auto_fit<'a>(cells: impl IntoIterator<Item = &'a str>) -> u16 {
    cells.into_iter().map(text::width).max().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::time::Instant;

    use vitui_engine::{Buttons, Key, KeyCode, KeyKind, KeyText, Mods, Mouse, MouseKind, Rect};

    use super::*;
    use crate::theme::Role;
    use crate::{Id, Interest};

    fn moved(x: u16, y: u16) -> Mouse {
        Mouse {
            x,
            y,
            kind: MouseKind::Move,
            buttons: Buttons::NONE,
            mods: Mods::NONE,
            at: Instant::now(),
        }
    }

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: Mods::NONE,
            kind: KeyKind::Press,
            text: KeyText::EMPTY,
            at: Instant::now(),
        }
    }

    /// A paragraph and its sizing function, laid out from one arithmetic: the fixture the honest
    /// half of these tests uses.
    fn paragraph_height(data: &str, width: u16) -> u16 {
        u16::try_from(text::wrap_height(data, width)).unwrap_or(u16::MAX)
    }

    fn paragraph(cx: &mut Ctx<'_, '_>, data: &str) {
        let body = cx.theme().paint(Role::Body);
        for (row, line) in text::wrap(data, cx.area().w).enumerate() {
            cx.text(0, i32::try_from(row).unwrap_or(i32::MAX), line, body);
        }
    }

    const PROSE: &str = "The real cost is drift: a sizing function and its component are two \
                         expressions of one layout with nothing holding them together";

    #[test]
    fn a_sizing_function_laid_out_from_one_arithmetic_agrees_at_every_width() {
        check(
            4..=90,
            |w| paragraph_height(PROSE, w),
            |cx| paragraph(cx, PROSE),
        )
        .assert();
    }

    /// **The defect this ticket exists for, reproduced.** The sizing function counts wrapped lines;
    /// the component wraps at a width one narrower because it draws a gutter it never told anybody
    /// about. Both are reasonable-looking, both are *nearly* right, and nothing but this check can
    /// tell them apart.
    #[test]
    fn the_detector_finds_a_component_that_drifted_from_its_sizing_function() {
        let found = check(
            4..=90,
            |w| paragraph_height(PROSE, w),
            |cx| {
                let body = cx.theme().paint(Role::Body);
                let gutter = cx.area().w.saturating_sub(1);
                for (row, line) in text::wrap(PROSE, gutter).enumerate() {
                    cx.text(1, i32::try_from(row).unwrap_or(i32::MAX), line, body);
                }
            },
        );
        assert!(
            !found.agrees(),
            "a one-column gutter changes the wrap, and the detector is what notices"
        );
        // A relation and not a count: how many widths a gutter moves is a property of this prose,
        // not of the mechanism.
        //
        // **The divisor moved from four to eight because the subject was repaired, not because the
        // gate was loosened.** `layout::text::wrap` had a defect — a row that exactly filled its
        // band fell through to the overhang branch and was cut at the *first* space — and it fired
        // at one width of a pair and not the other, so the detector was counting it as drift. With
        // the wrap fixed the same prose and the same one-column gutter disagree at **15 of 87**
        // widths against **43 of 87**: twenty-eight of the forty-three were the wrap and not the
        // gutter. The claim is unchanged — *broad rather than a single boundary width* — and 15
        // against a floor of 11 is the headroom.
        assert!(
            found.disagreements.len() * 8 > found.widths as usize,
            "the drift is broad rather than a single boundary width: {} of {}",
            found.disagreements.len(),
            found.widths
        );
    }

    /// The other direction, which a generous discard surface would not have caught: the sizing
    /// function claims a row the component never draws.
    #[test]
    fn the_detector_finds_a_sizing_function_that_claims_a_row_nobody_draws() {
        let found = check(
            10..=40,
            |w| paragraph_height(PROSE, w) + 1,
            |cx| paragraph(cx, PROSE),
        );
        assert_eq!(
            found.disagreements.len(),
            found.widths as usize,
            "a claim one row too tall is wrong at every width"
        );
    }

    /// **The extent records what a verb offered, not what it managed to write.** Without this the
    /// discard surface would agree with itself: everything is clipped to the claim, so everything
    /// fits.
    #[test]
    fn the_extent_records_what_a_verb_offered_and_not_what_it_wrote() {
        let mut driver = Driver::headless(4, 4).expect("sink");
        driver.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            let measured = cx.measured(10, 2, |inner| {
                inner.text(0, 5, "five rows below the top of a two-row surface", body);
            });
            assert_eq!(measured.extent.h, 6);
        });
    }

    #[test]
    fn an_empty_string_reaches_nothing() {
        let mut driver = Driver::headless(4, 4).expect("sink");
        driver.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            let measured = cx.measured(10, 3, |inner| {
                inner.text(0, 0, "", body);
            });
            assert_eq!(measured.extent, Extent::ZERO);
        });
    }

    /// The extent is in the **body's** coordinates, which is what makes a component drawn through
    /// two containers measurable at all.
    #[test]
    fn a_nested_child_reaches_in_the_bodys_own_coordinates() {
        let mut driver = Driver::headless(4, 4).expect("sink");
        driver.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            let measured = cx.measured(20, 8, |inner| {
                let mut outer = inner.child(Rect::new(2, 1, 16, 6));
                let mut nested = outer.child(Rect::new(3, 2, 10, 4));
                nested.text(0, 0, "abc", body);
            });
            assert_eq!(measured.extent, Extent { w: 8, h: 4 });
        });
    }

    /// A scrolled context offsets the content, so the extent follows the content and not the cells.
    ///
    /// # The figure moved, and the reason is a sign this test was pinning against the view
    ///
    /// It read **2**, which is content row 4 *minus* the 3 — a scroll-*position* reading of
    /// `Ctx::scrolled`'s argument. `View::at` is `(x + origin.0, y + origin.1)` and
    /// `View::scrolled` **adds**, so the cell this verb writes lands at root row 7 and not at root
    /// row 1; the extent was measuring one of those and the surface the other. A components gate
    /// made the two origins take the translation with the same sign, and the figure is **8** — the
    /// row the verb actually reached, plus one.
    ///
    /// Both directions are here now, because the whole defect was that only one was ever written
    /// down: `scrolled(0, 3)` pushes the content **down** and `scrolled(0, -3)` pulls it up, which
    /// is the sense `Ctx::scroll_scope` negates an offset into.
    #[test]
    fn a_scrolled_context_reaches_where_the_content_is() {
        let mut driver = Driver::headless(4, 4).expect("sink");
        driver.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            let down = cx.measured(20, 8, |inner| {
                let mut scrolled = inner.scrolled(0, 3);
                scrolled.text(0, 4, "one row", body);
            });
            assert_eq!(
                down.extent.h, 8,
                "content row 4 pushed down three lands at root row 7"
            );

            let up = cx.measured(20, 8, |inner| {
                let mut scrolled = inner.scrolled(0, -3);
                scrolled.text(0, 4, "one row", body);
            });
            assert_eq!(up.extent.h, 2, "and pulled up three it lands at root row 1");
        });
    }

    /// **A real frame does not maintain the extent**, because measuring the width of every verb is
    /// 7% of the frame budget and no frame is asked to pay it.
    #[test]
    fn a_real_frame_maintains_no_extent() {
        let mut driver = Driver::headless(8, 4).expect("sink");
        driver.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            cx.text(0, 0, "drawn", body);
        });
        assert_eq!(driver.inspect().extent(), None);
    }

    // ── the measured world's four absences, planted in the real frame and absent in the measured
    //    one. Each of these is *asserted* rather than assumed, because the default a component gets
    //    for one of them looks exactly like an honest answer. ──────────────────────────────────────

    #[test]
    fn the_measured_world_has_no_focus() {
        let widget = Id::named("field");
        let mut driver = Driver::headless(8, 4).expect("sink");
        driver.plant(None, Some(widget), None);
        driver.frame(|cx| {
            let real = cx.interact(widget, Rect::new(0, 0, 4, 1), Interest::FOCUS);
            assert!(real.focused, "the real frame has the focus planted on it");
            let measured = cx.measured(8, 2, |inner| {
                inner
                    .interact(widget, Rect::new(0, 0, 4, 1), Interest::FOCUS)
                    .focused
            });
            assert!(!measured.value, "the measured world has no focus");
            assert!(measured.consulted.interaction);
        });
    }

    #[test]
    fn the_measured_world_has_no_hover() {
        let widget = Id::named("button");
        let mut driver = Driver::headless(8, 4).expect("sink");
        // Two frames: the hover guess is resolved in `begin` from the previous frame's index, so a
        // widget is hovered on the frame after the one that declared it under the pointer.
        for _ in 0..2 {
            driver.post_mouse(moved(1, 0));
            driver.frame(|cx| {
                cx.interact(widget, Rect::new(0, 0, 4, 1), Interest::HOVER);
            });
        }
        driver.frame(|cx| {
            let real = cx.interact(widget, Rect::new(0, 0, 4, 1), Interest::HOVER);
            assert!(real.hovered, "the real frame has the pointer over it");
            let measured = cx.measured(8, 2, |inner| {
                inner
                    .interact(widget, Rect::new(0, 0, 4, 1), Interest::HOVER)
                    .hovered
            });
            assert!(!measured.value, "the measured world has no pointer at all");
        });
    }

    #[test]
    fn the_measured_world_has_no_press() {
        let widget = Id::named("button");
        let mut driver = Driver::headless(8, 4).expect("sink");
        driver.plant(Some(widget), None, None);
        driver.frame(|cx| {
            let real = cx.interact(widget, Rect::new(0, 0, 4, 1), Interest::CLICK);
            assert!(real.pressed, "the real frame holds the grab");
            let measured = cx.measured(8, 2, |inner| {
                inner
                    .interact(widget, Rect::new(0, 0, 4, 1), Interest::CLICK)
                    .pressed
            });
            assert!(!measured.value, "the measured world holds no grab");
        });
    }

    /// **And the key queue is not drained**, which is the half that would have been silent: a
    /// measured world sharing the frame would eat the keystroke the real widget was about to read.
    #[test]
    fn the_measured_world_has_no_keys_and_drains_none() {
        let widget = Id::named("field");
        let mut driver = Driver::headless(8, 4).expect("sink");
        driver.plant(None, Some(widget), None);
        driver.post_key(key(KeyCode::Char('a')));
        driver.frame(|cx| {
            let measured = cx.measured(8, 2, |inner| inner.next_key(widget));
            assert!(measured.value.is_none(), "the measured world has no keys");
            assert!(measured.consulted.keys);
            // The real frame still has it, in the real queue, for the real widget.
            assert!(
                cx.next_key(widget).is_some(),
                "the measured world drained the queue the real widget reads"
            );
        });
    }

    /// **Its own `Frame`, and these are the three counts that say so**: the id table, the hit index
    /// and the tab ring all belong to the real frame and a measured draw leaves every one of them
    /// where it was.
    #[test]
    fn the_measured_world_claims_its_ids_in_a_table_of_its_own() {
        let widget = Id::named("row");
        let mut driver = Driver::headless(8, 4).expect("sink");
        driver.frame(|cx| {
            cx.interact(widget, Rect::new(0, 0, 4, 1), Interest::FOCUS);
            // The same id, claimed again — and drawn, so that the body demonstrably ran.
            let measured = cx.measured(8, 2, |inner| {
                let body = inner.theme().paint(Role::Body);
                inner.interact(widget, Rect::new(0, 0, 4, 1), Interest::FOCUS);
                inner.text(0, 0, "row", body);
            });
            assert_eq!(measured.extent, Extent { w: 3, h: 1 });
        });
        // Three counts, read after the frame because they are the frame's rather than the draw's:
        // one id, one hit entry, one tab stop — and not two of any of them.
        let frame = driver.inspect();
        assert_eq!(
            (frame.ids().live(), frame.hits().len(), frame.ring().len()),
            (1, 1, 1)
        );
    }

    /// **A frame has side effects, and a second frame has them twice.** The output looks the same,
    /// which is the whole reason this is a test and not a note.
    #[test]
    fn a_body_run_twice_has_its_side_effects_twice() {
        let ticks = Cell::new(0u32);
        let mut driver = Driver::headless(8, 4).expect("sink");
        driver.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            let tail = |cx: &mut Ctx<'_, '_>| {
                ticks.set(ticks.get() + 1);
                cx.text(0, 0, "tick", body);
            };
            tail(cx);
            let _ = cx.measured(8, 2, tail);
        });
        assert_eq!(
            ticks.get(),
            2,
            "the author wrote one tick and the dry run made it two"
        );
    }

    /// **The dry run measures the clip, not the content.** A virtualised list draws what is visible
    /// and the extent reports the clip — so a dry run cannot size it, and the row count is the
    /// evidence: the same body over a thousand times more data touches the same rows.
    #[test]
    fn the_dry_run_measures_the_clip_and_not_the_content() {
        let touched = Cell::new(0u32);
        let mut driver = Driver::headless(8, 4).expect("sink");
        driver.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            // A virtualised list: it iterates the reachable rows and never the source.
            let list = |cx: &mut Ctx<'_, '_>, rows: u32| {
                for row in cx.visible_rows() {
                    if u32::try_from(row).unwrap_or(u32::MAX) >= rows {
                        break;
                    }
                    touched.set(touched.get() + 1);
                    cx.text(0, row, "row", body);
                }
            };
            let thousand = cx.measured(8, 80, |inner| list(inner, 1_000));
            let after_thousand = touched.get();
            let million = cx.measured(8, 80, |inner| list(inner, 1_000_000));
            assert_eq!(
                (thousand.extent.h, million.extent.h),
                (80, 80),
                "the clip, whatever the source holds — a million rows still report eighty"
            );
            assert_eq!(
                touched.get() - after_thousand,
                after_thousand,
                "frame cost is proportional to visible rows and never to data volume"
            );
        });
    }

    /// **The trait form's second death.** `measure` has to return the type a rectangle is made of,
    /// the honest answer for a virtualised child is *all of it*, and all of it does not fit.
    #[test]
    fn a_virtualised_child_has_no_honest_measure_answer() {
        const ROWS: u32 = 1_000_000;
        assert!(
            u16::try_from(ROWS).is_err(),
            "the honest answer is not expressible"
        );
        assert_eq!(
            ROWS / u32::from(u16::MAX),
            15,
            "and `u16::MAX` — a rectangle nothing can draw — is still 15x short of it"
        );
    }

    #[test]
    fn auto_fit_measures_columns_and_not_bytes() {
        assert_eq!(auto_fit(["ab", "漢字", "abcd"]), 4);
        assert_eq!(auto_fit(std::iter::empty()), 0);
    }

    /// The same rule the data module keeps, for the same reason: **this is a claim about absent
    /// code, and absent code is only visible where it would have been written.** A test that
    /// constructed something instead would pass for any reason at all, including the trait existing
    /// and being unused.
    #[test]
    fn the_module_declares_no_trait() {
        let source = include_str!("sizing.rs");
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
            "spec §12 says a sizing function is a shape and not a type, and this module \
             declares {declarations:?}"
        );
    }
}
