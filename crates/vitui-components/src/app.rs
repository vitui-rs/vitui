//! **The one rule the correct build had to buy separately: the screen clears once.**
//!
//! Components ticket 10. Spec §2:
//!
//! > Earning the equality cost one rule of its own: **the correct build clears once, on its first
//! > frame and on a resize**, because a screen whose gaps are never painted is not the same screen.
//! > Same call, `cx.clear(theme.body)`; the difference between once and every frame is 6 662 damaged
//! > cells.
//!
//! **It is not a component and it cannot be one.** Every other rule on this map is a statement about
//! one rectangle inside one frame, which is a thing a function can obey. This one is a statement
//! about a *sequence* of frames — *once, and then never again until the size changes* — so it needs
//! a value that survives a frame, and the only such value in a runtime with no retained structure
//! (ADR 0012) is one the application owns. [`Clears`] is that value: two words, kept by the caller,
//! passed a `Ctx` once a frame.
//!
//! # Both directions are defects, and that is why the equality is what proves this
//!
//! Clearing every frame is ADR 0026's largest row — **9 024 cells** on [`crate::dense`]'s screen,
//! every frame, for a screen that is not moving. Clearing *never* is the other defect: the gaps
//! nobody paints keep whatever was there before, and the screen stops being the same screen. Neither
//! shows up on a counter of its own. What separates the three spellings is the pair of gates §2
//! states together — the naive twin's **0 of 24 000 cells differ** and the correct arm's **0
//! re-damaged** — and a build can only pass both by clearing exactly once.
//!
//! # A resize is modelled and the model is named
//!
//! A terminal resize arrives as `vitui_engine::Event::Resize`, and this crate cannot name
//! `vitui_engine` at all (constraint C6). `Driver` publishes `post_mouse` and `post_key` and no
//! resize door, so **there is no way to resize a driver from here**. What [`Clears`] keys on is the
//! size it is handed, which is [`Ctx::area`] — so a resize is stood up by carrying one `Clears`
//! across two drivers of different sizes, and the fact under test is exactly the one that matters:
//! *the size changed, so the next frame clears.* See
//! `tests::one_clear_a_size_and_never_a_third`.

use vitui_runtime::{Ctx, Role};

use crate::ink::{Direct, Ink};
use crate::text::pad_rows;

/// **The application's clear, and the whole of its state.**
///
/// Kept by the caller across frames, and asked once at the top of each. It answers *did I clear*, so
/// a gate can count the clears rather than infer them from a damage figure.
///
/// ```
/// use vitui_components::app::Clears;
/// use vitui_runtime::ctx::Driver;
///
/// let mut driver = Driver::headless(40, 10).expect("a sink attaches");
/// let mut clears = Clears::new();
/// for _ in 0..8 {
///     driver.frame(|cx| {
///         clears.frame(cx);
///     });
/// }
/// // Eight frames, one clear. The other seven are the 9 024 cells a frame that spelling costs on
/// // the dense screen.
/// assert_eq!(clears.cleared(), 1);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Clears {
    seen: Option<(u16, u16)>,
    cleared: u32,
}

impl Clears {
    /// An application that has not drawn a frame yet.
    pub const fn new() -> Clears {
        Clears {
            seen: None,
            cleared: 0,
        }
    }

    /// **Clear if this is the first frame or the size has changed.** Returns whether it did.
    ///
    /// Call it at the top of the frame, before anything is drawn. The paint is [`Role::Body`], which
    /// is §2's own `cx.clear(theme.body)`.
    pub fn frame(&mut self, cx: &mut Ctx<'_, '_>) -> bool {
        self.frame_into(&mut Direct, cx)
    }

    /// **[`Clears::frame`], writing through an [`Ink`] so a counter can see it.**
    ///
    /// # It is a run a row and not `Ctx::clear`, and that is the seam's rule rather than a deviation
    ///
    /// `Ctx::clear` is one `View::fill` over the whole context and it returns `()`. A filled cell is
    /// therefore *modelled* rather than reported, both sides of `writes == reported` become this
    /// crate's own arithmetic, and the one write on the screen a damage gate most needs to see
    /// becomes the one write no instrument can — [`crate::ink`]'s header states the argument and
    /// [`crate::dense`]'s `wash` already obeys it. The cells written are identical; what differs is
    /// that the engine reports them.
    ///
    /// The cost is `h` verbs instead of one, on a frame that happens **twice in an application's
    /// life** — its first, and each resize.
    pub fn frame_into<I: Ink>(&mut self, ink: &mut I, cx: &mut Ctx<'_, '_>) -> bool {
        let screen = cx.area();
        let size = (screen.w, screen.h);
        if self.seen == Some(size) {
            return false;
        }
        self.seen = Some(size);
        self.cleared += 1;
        let body = cx.theme().paint(Role::Body);
        pad_rows(ink, cx, screen, body);
        true
    }

    /// **How many times it has cleared.** One per size the application has been shown at.
    pub fn cleared(self) -> u32 {
        self.cleared
    }

    /// The size the last clear was taken at, or `None` before the first frame.
    pub fn size(self) -> Option<(u16, u16)> {
        self.seen
    }
}

/// **The two other spellings of the clear, kept because a gate nobody has watched fail is not a
/// gate.**
///
/// `pub` for the reason [`crate::frame::defective`] and [`crate::text::defective`] are. Both are
/// defects and they are defects in opposite directions, which is the whole shape of §2's argument:
/// one re-damages 9 024 cells a frame for ever, the other leaves the gaps carrying whatever was
/// there before.
pub mod defective {
    use super::{Clears, Ctx, Ink, Role, pad_rows};

    /// **`cx.clear(body)` at the top of every frame.** ADR 0026's largest row.
    pub fn every_frame<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>) -> bool {
        let screen = cx.area();
        let body = cx.theme().paint(Role::Body);
        pad_rows(ink, cx, screen, body);
        true
    }

    /// **A clear that never happens.** The other defect, and the one no damage counter can see: it
    /// re-damages nothing at all, and it is caught only by the equality against a screen that did
    /// clear.
    pub fn never<I: Ink>(_: &mut I, _: &mut Ctx<'_, '_>) -> bool {
        false
    }

    /// **A `Clears` that keys on nothing**, so it clears once and then never again — including
    /// across the resize it is supposed to notice. The third spelling, and the one that looks
    /// correct on every steady-frame gate there is.
    pub fn once_and_deaf<I: Ink>(state: &mut Clears, ink: &mut I, cx: &mut Ctx<'_, '_>) -> bool {
        if state.cleared() > 0 {
            return false;
        }
        state.frame_into(ink, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::counters::Tally;
    use crate::runner::driver_at;
    use vitui_runtime::Density;

    /// The size a clear is measured at here. Small on purpose — the magnitude that matters is
    /// [`crate::dense`]'s, and this file is about *how many times*.
    const W: u16 = 40;
    /// See [`W`].
    const H: u16 = 10;

    /// A tally over `frames` frames of one `Clears`, at one size.
    fn played(state: &mut Clears, w: u16, h: u16, frames: u32) -> Tally {
        let mut driver = driver_at(w, h, Density::Compact);
        let mut tally = Tally::new();
        for _ in 0..frames {
            driver.frame(|cx| {
                state.frame_into(&mut tally, cx);
            });
        }
        tally
    }

    /// **Criterion 5: one clear a size, and never a third.**
    ///
    /// The whole rule as a count. Sixteen frames at one size and sixteen at another is **two**
    /// clears; the every-frame spelling is thirty-two.
    #[test]
    fn one_clear_a_size_and_never_a_third() {
        let mut clears = Clears::new();
        let first = played(&mut clears, W, H, 16);
        assert_eq!(clears.cleared(), 1, "sixteen frames, one clear");
        assert_eq!(
            first.writes(),
            u64::from(W) * u64::from(H),
            "and the one clear covered the screen"
        );
        assert_eq!(clears.size(), Some((W, H)));

        // The resize, modelled as this module's header names it: the size changed, so the next
        // frame clears — and the fifteen after it do not.
        let second = played(&mut clears, W / 2, H, 16);
        assert_eq!(clears.cleared(), 2, "one more, and one only");
        assert_eq!(second.writes(), u64::from(W / 2) * u64::from(H));
        assert_eq!(clears.size(), Some((W / 2, H)));

        // And a third size, so *two* is not a constant this happens to reach.
        played(&mut clears, W, H * 2, 3);
        assert_eq!(clears.cleared(), 3);
    }

    /// **The three spellings, and each one watched being what it is.**
    ///
    /// `once_and_deaf` is the one worth reading twice: it passes every steady-frame damage gate on
    /// this map and it is still wrong, because the frame it fails on is the one nothing here can
    /// post.
    #[test]
    fn the_every_frame_spelling_clears_every_frame_and_the_deaf_one_misses_the_resize() {
        let mut driver = driver_at(W, H, Density::Compact);
        let mut tally = Tally::new();
        let mut cleared = 0u32;
        for _ in 0..8 {
            driver.frame(|cx| {
                cleared += u32::from(defective::every_frame(&mut tally, cx));
            });
        }
        assert_eq!(cleared, 8, "eight frames, eight clears");
        assert_eq!(tally.writes(), 8 * u64::from(W) * u64::from(H));
        assert_eq!(
            tally.writes(),
            tally.distinct() * 8,
            "and every cell of the screen written eight times, which is the re-damage"
        );

        // Never: no verb at all, and no counter on this map disapproves.
        let mut none = Tally::new();
        driver.frame(|cx| {
            assert!(!defective::never(&mut none, cx));
        });
        assert_eq!(none.writes(), 0);

        // Deaf: right at one size, silent at the next.
        let mut deaf = Clears::new();
        let mut probe = Tally::new();
        let mut small = driver_at(W, H, Density::Compact);
        small.frame(|cx| {
            assert!(defective::once_and_deaf(&mut deaf, &mut probe, cx));
        });
        let mut wide = driver_at(W * 2, H, Density::Compact);
        wide.frame(|cx| {
            assert!(
                !defective::once_and_deaf(&mut deaf, &mut probe, cx),
                "the deaf spelling noticed a resize, so it is no longer the defect it is kept as"
            );
        });
        // The correct one does notice, through the same two drivers.
        let mut honest = Clears::new();
        let mut small = driver_at(W, H, Density::Compact);
        small.frame(|cx| {
            assert!(honest.frame_into(&mut probe, cx));
        });
        let mut wide = driver_at(W * 2, H, Density::Compact);
        wide.frame(|cx| {
            assert!(honest.frame_into(&mut probe, cx));
        });
        assert_eq!(honest.cleared(), 2);
    }

    /// **The clear writes each cell exactly once**, which is the partition rule on the one write
    /// that covers the whole screen.
    #[test]
    fn the_clear_is_a_partition_of_the_screen() {
        let mut clears = Clears::new();
        let tally = played(&mut clears, W, H, 1);
        assert_eq!(tally.writes(), tally.distinct());
        assert_eq!(tally.distinct(), u64::from(W) * u64::from(H));
        assert_eq!(
            tally.reported(),
            tally.writes(),
            "a write the engine did not report, which is what `Ctx::clear` would have cost"
        );
        assert_eq!(tally.verbs(), u64::from(H), "one run a row");
    }
}
