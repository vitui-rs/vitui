//! **The seam that lets a gate measure the shipped drawing path instead of a copy of it.**
//!
//! Spec §2's first equality is `writes == distinct cells touched`, and the two things it compares
//! come from two different places: `writes` is the engine's own report — the `cells` field of what
//! `Ctx::text` returns — and `distinct` is [`crate::counters::Tally`]'s union over the spans those
//! verbs reported. That only works if the tally **sees the verbs**. A component's shape is
//! `fn(&mut Ctx, …) -> Response` (spec §1) and nothing on this map has decided that a component
//! draws through a wrapper, so `fit` and `block` cannot take a `Tally`.
//!
//! The alternative that was not taken is a second implementation of `fit` and `block` written
//! against `Tally` for the gate to run over. That is the shape §21 spent three refinements on:
//! **a gate written against a copy of the code tests the copy.** Twelve prototypes each computed
//! §20's counters their own way inside their own binary, and `crate::counters` exists because of it.
//!
//! What ships instead is one trait with two methods and three implementations. `fit` and `block`
//! are written once, generic over it; [`Direct`] is what a component gets and allocates nothing;
//! `Tally` and [`crate::runner::Pen`] are what a gate gets. **The cells written, the order they are
//! written in, and the runtime verb that writes them are the same value in all three** — only the
//! bookkeeping beside it differs.
//!
//! # Why the trait is `run` and not `fill`
//!
//! Every span `fit` and `block` write is a run on **one row**: a border, a padding band, a label, a
//! stretch of spaces. Written with `Ctx::fill` they would be invisible to the pair, because
//! `Ctx::fill` returns `()` — there is no engine report to fold, both sides of the equality would be
//! this crate's arithmetic, and `Tally::reported` (which exists to catch exactly that) would fall
//! below `Tally::writes`. Written as runs they are all `View::text`, so **`reported == writes` holds
//! for both helpers** and the equality really is two sources compared.
//!
//! # The shipped path stages and the instrument path allocates
//!
//! [`Direct::run`] builds its run through [`Ctx::stage`] and [`Ctx::blit`], which format into the
//! frame's own reusable buffer: no allocation, which is what the standing budget *zero allocations
//! during frame composition* requires of anything a component calls. The `Tally` and `Pen`
//! implementations materialise the same string with [`str::repeat`] and hand it to their own
//! recording `text`, because their fields are private to their modules and a recording verb needs
//! the bytes. Both end at the same `View::text` with the same string, so the two paths agree on
//! every counter — and an instrument allocating is not a defect, `Pen` already allocates a `String`
//! per cell it records.

use std::fmt;

use vitui_runtime::{Ctx, Paint, Response, Role};

use crate::cells::Cells;
use crate::counters::Tally;
use crate::runner::{Award, Pen};

/// A cluster written `n` times, as a [`fmt::Display`] so it can go through [`Ctx::stage`].
///
/// The adapter is what makes the no-allocation path possible: `str::repeat` returns a `String` and
/// `format_args!` over this returns nothing at all.
struct Repeat<'a>(&'a str, u16);

impl fmt::Display for Repeat<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for _ in 0..self.1 {
            f.write_str(self.0)?;
        }
        Ok(())
    }
}

/// **What a partition helper writes through.**
///
/// Nothing outside this crate implements it: the three implementations are [`Direct`], [`Tally`]
/// and [`Pen`], and they are the three things a helper is ever asked to draw into — a real frame, a
/// counter, and a recorded surface.
///
/// # Two verbs and one declaration, and the third method is here because the gate must see it
///
/// It carried two methods while `fit` and `block` were the only helpers, and its own note said *a
/// third would be a place for a helper to write something the gate cannot see*. [`Ink::award`] is
/// the exception that note predicts rather than a breach of it: components ticket 07's
/// [`crate::state::press`] declares a **deferred hover award**, the runtime applies it as a
/// background restyle at `end`, and ADR 0026 prices the whole helper on a relation between that
/// restyle and the widget's next draw — *a restyle is free only when the component's own next draw
/// already produces the value the restyle produced*. An award the instrument cannot see is exactly
/// the write the gate is about, so it goes through the seam with the two that draw.
pub trait Ink {
    /// Write a string, and return **the engine's own column count** — how many columns landed after
    /// clipping, not how many were asked for.
    fn text(&mut self, cx: &mut Ctx<'_, '_>, x: i32, y: i32, s: &str, st: Paint) -> u16;

    /// Write `cluster` `n` times from `(x, y)`, and return the engine's own column count.
    ///
    /// The verb a border run, a padding band and a stretch of trailing spaces are all one call of.
    fn run(
        &mut self,
        cx: &mut Ctx<'_, '_>,
        x: i32,
        y: i32,
        cluster: &str,
        n: u16,
        st: Paint,
    ) -> u16;

    /// **Declare the face `cells` is to be awarded if it wins the hover**, and let the instrument
    /// see it.
    ///
    /// It writes no cell of its own: the runtime applies the restyle at `end`, from the index that
    /// has just drawn. What an instrument does with it is model the restyle on its own surface, so
    /// that *the component's next draw already produces the value the restyle produced* is a number
    /// rather than a sentence.
    fn award(&mut self, cx: &mut Ctx<'_, '_>, cells: Cells, resp: &Response, role: Role);
}

/// **The implementation a component gets: draw, count nothing, allocate nothing.**
///
/// A unit struct rather than `impl Ink for ()`, so that a signature saying `Direct` says which of
/// the three paths is being taken.
#[derive(Clone, Copy, Debug, Default)]
pub struct Direct;

impl Ink for Direct {
    fn text(&mut self, cx: &mut Ctx<'_, '_>, x: i32, y: i32, s: &str, st: Paint) -> u16 {
        cx.text(x, y, s, st).cells
    }

    fn run(
        &mut self,
        cx: &mut Ctx<'_, '_>,
        x: i32,
        y: i32,
        cluster: &str,
        n: u16,
        st: Paint,
    ) -> u16 {
        if n == 0 {
            return 0;
        }
        // `stage` measures into the frame's buffer and `blit` draws what was staged: one
        // `View::text`, no allocation, and the same bytes `str::repeat` would have produced.
        let _ = cx.stage(format_args!("{}", Repeat(cluster, n)));
        cx.blit(x, y, st).cells
    }

    fn award(&mut self, cx: &mut Ctx<'_, '_>, cells: Cells, resp: &Response, role: Role) {
        cells.hover_style(cx, resp, role);
    }
}

impl Ink for Tally {
    fn text(&mut self, cx: &mut Ctx<'_, '_>, x: i32, y: i32, s: &str, st: Paint) -> u16 {
        Tally::text(self, cx, x, y, s, st)
    }

    fn run(
        &mut self,
        cx: &mut Ctx<'_, '_>,
        x: i32,
        y: i32,
        cluster: &str,
        n: u16,
        st: Paint,
    ) -> u16 {
        if n == 0 {
            return 0;
        }
        Tally::text(self, cx, x, y, &cluster.repeat(usize::from(n)), st)
    }

    /// **The declaration, and nothing folded in.** A `Tally` counts the columns a *component*
    /// wrote; the restyle is the runtime's write and it carries no per-cell value for the tally to
    /// compare against anyway. What can see it is [`Pen`], which keeps the values.
    fn award(&mut self, cx: &mut Ctx<'_, '_>, cells: Cells, resp: &Response, role: Role) {
        cells.hover_style(cx, resp, role);
    }
}

impl Ink for Pen {
    fn text(&mut self, cx: &mut Ctx<'_, '_>, x: i32, y: i32, s: &str, st: Paint) -> u16 {
        Pen::text(self, cx, x, y, s, st)
    }

    fn run(
        &mut self,
        cx: &mut Ctx<'_, '_>,
        x: i32,
        y: i32,
        cluster: &str,
        n: u16,
        st: Paint,
    ) -> u16 {
        if n == 0 {
            return 0;
        }
        Pen::text(self, cx, x, y, &cluster.repeat(usize::from(n)), st)
    }

    /// **Declare it, and apply it to the recorded surface the way the runtime applies it.**
    ///
    /// `Frame::hover_to_apply` restyles exactly one rectangle a frame — the one belonging to the id
    /// the award named as hovered — so the model applies it where `Response::hovered` is the fact
    /// the fixture is standing up. That is the one place this instrument stands in for the runtime,
    /// and it stood in for it because **the pointer could not be driven from this crate at all**:
    /// `Driver::post_mouse` takes a `vitui_engine::Mouse`, which was
    /// `EngineName { name: "Mouse", reachable_as: None }` in `crates/vitui-runtime/src/line.rs`,
    /// and `Driver::plant` reaches the grab, the focus and the click and not the pointer.
    ///
    /// **Runtime architecture issue 22 lifted it.** The stand-in is kept — this instrument models
    /// the runtime's restyle and does not need a gesture to do it — but it is a choice now rather
    /// than the only arrangement available.
    fn award(&mut self, cx: &mut Ctx<'_, '_>, cells: Cells, resp: &Response, role: Role) {
        let painted = cx.theme().paint(role);
        cells.hover_style(cx, resp, role);
        self.declare(Award {
            x: i32::from(cells.x()),
            y: i32::from(cells.y()),
            w: cells.w(),
            h: cells.h(),
            role,
            painted,
            applied: resp.hovered,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vitui_runtime::Role;
    use vitui_runtime::ctx::Driver;

    /// **The three implementations write the same cells**, which is the claim the whole seam rests
    /// on: if `Direct` and `Tally` disagreed, the gate would be measuring a path a component does
    /// not take.
    ///
    /// The comparison is `Pen`'s recorded surface against itself under `Direct`, and the only thing
    /// that can be compared for `Direct` is what the engine reported — so the assertion is on the
    /// column count, which is the number `writes` is made of.
    #[test]
    fn the_staged_run_and_the_repeated_one_write_the_same_columns() {
        let mut driver = Driver::headless(20, 3).expect("a sink cannot fail to attach");
        driver.frame(|cx| {
            let paint = cx.theme().paint(Role::Body);
            let mut tally = Tally::new();
            let mut pen = Pen::new(20, 3);
            for (cluster, n) in [(" ", 0u16), (" ", 1), ("-", 19), ("-", 25), ("ab", 4)] {
                let direct = Direct.run(cx, 0, 0, cluster, n, paint);
                let tallied = Ink::run(&mut tally, cx, 0, 1, cluster, n, paint);
                let penned = Ink::run(&mut pen, cx, 0, 2, cluster, n, paint);
                assert_eq!(
                    (direct, tallied),
                    (direct, direct),
                    "`Direct` and `Tally` disagree on `{cluster}` x {n}"
                );
                assert_eq!(
                    (direct, penned),
                    (direct, direct),
                    "`Direct` and `Pen` disagree on `{cluster}` x {n}"
                );
            }
            // The clip is the engine's, not ours: 25 dashes into 20 columns is 20.
            assert_eq!(Direct.run(cx, 0, 0, "-", 25, paint), 20);
        });
    }

    /// **A run of nothing is not a verb**, which is what keeps `verbs` honest: a helper whose
    /// padding happens to be zero cells wide must not report a call.
    #[test]
    fn a_zero_length_run_writes_nothing_and_counts_nothing() {
        let mut driver = Driver::headless(20, 3).expect("a sink cannot fail to attach");
        driver.frame(|cx| {
            let paint = cx.theme().paint(Role::Body);
            let mut tally = Tally::new();
            assert_eq!(Ink::run(&mut tally, cx, 0, 0, " ", 0, paint), 0);
            assert_eq!(tally.verbs(), 0);
            assert_eq!(tally.writes(), 0);
            assert_eq!(tally.distinct(), 0);
        });
    }
}
