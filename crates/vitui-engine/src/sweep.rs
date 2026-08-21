//! Eviction: the mark-and-compact sweep, and the one thing it breaks.
//!
//! # What grows, and why only one of the two tables can
//!
//! **The clusters are not the problem and the styles are, and the difference is who writes them**
//! (spec §3). A cluster handle comes from text the application supplied, so the interner is bounded
//! by the distinct clusters the application has ever drawn. An extended-style handle comes from a
//! drawing verb *and from the compositor*, because a [`Mix`](crate::Mix) over an extended cell
//! produces a style that has never existed — and an operator whose `amount` moves every frame
//! produces a new one every frame, for ever. Measured on 300×80 of hyperlinked text, 96 distinct
//! extended styles, under one operator:
//!
//! | | entries created | per frame |
//! |---|---|---|
//! | a settled modal dim, 120 frames | 96 | 0.8 |
//! | a fade in, 120 frames | 11 484 | 95.7 |
//!
//! **A static operator converges after one frame** — the table deduplicates, so every later frame
//! asks for entries that are already there, which is what `tests/alloc.rs` gates. The leak needs an
//! operator whose `amount` is changing: a 300 ms fade costs about 1 700 entries and a permanently
//! pulsing dim about 5 700 a second. **That is the case eviction exists for and the only one.**
//!
//! # The shape: mark, compact in table order, rewrite
//!
//! One marker slot per table entry; a walk of the live surfaces to set them; each table rebuilt in
//! table order; the handles rewritten. **It marks no damage**, because the cells still say the same
//! thing — the same text in the same colours, named differently.
//!
//! **Compacting in table order is what makes the cheap case cheap**: a sweep that frees nothing
//! *below* a live entry does not renumber at all, so [`Interner::compact`] and [`ExtStyles::compact`]
//! answer `None` and the rewrite pass is skipped entirely. Compacting into a free list, or sorting
//! by anything, would renumber every entry above the first hole whether or not it had to.
//!
//! It costs 82.5 µs for one screen and 897 µs for twenty layers (spec §3's prototype measured
//! 58.88 µs and 1.17 ms), and that is allowed **because it never runs on the render path** — §13's
//! cliff table lists it beside the frames that are deliberately outside the incremental budget.
//! Where it does run is [`Screen::layers`](crate::Screen::layers), which is not inside `present` and
//! *is* inside the application's frame loop; the note there is exact about the difference, and about
//! why §3 asks for the first and not the second.
//!
//! # The frame surface is one of the live surfaces
//!
//! Spec §3 says *it walks the live surfaces once*, and the composited frame is one of them: its
//! cells were copied out of the layers and carry the layer stack's handles, and only the damaged
//! runs of it are recomposited each frame. A sweep that rewrote the layers and left the frame alone
//! would leave every undamaged cell of the frame naming an entry that had moved — which no
//! instrument in this crate would report, because nothing packs an undamaged cell.
//!
//! # The URI table is not swept, and that is a decision
//!
//! Link ids are few and **an application holds handles to them**: [`LinkId`](crate::LinkId) is
//! public and [`Screen::link`](crate::Screen::link) hands one back across frames. Renumbering the
//! URI table would therefore invalidate values a caller is still holding, which is a different
//! class of failure from the one this file exists to bound — the two handle spaces the sweep does
//! touch are unreachable from outside the crate. Recorded here as a decision rather than left to be
//! read as an omission (spec §3, §15).
//!
//! # What breaks, and the general rule it comes from
//!
//! Exactly one reader compares handle identity across frames — the mirror — and the answer is one
//! flag, [`Packet::repaint`](crate::packet::Packet::repaint). The **"second reader" this was feared
//! for does not exist**, because a packet holds no handle: every handle is resolved into a side
//! table at pack time (ADR 0011), so the app thread may sweep while the render thread is inside a
//! 200 ms `write`.
//!
//! > **Nothing the render thread compares across frames may be derived from a position.**

use crate::cell::GraphemeId;
use crate::style::Style;
use crate::surface::Surface;
use crate::tables::Tables;

/// What a [`Remap`]'s map holds for an entry the sweep dropped.
///
/// No live handle can carry it. A cluster id is bounded by `LAST_CLUSTER - FIRST_CLUSTER` and an
/// extended-style handle by the twenty bits spec §3 gives it, both decades below this.
pub(crate) const DEAD: u32 = u32::MAX;

/// The renumbering one table's liveness markers imply.
pub(crate) struct Remap {
    /// The new id for each old id, [`DEAD`] where the entry was dropped.
    pub(crate) map: Vec<u32>,
    /// Whether any **survivor's** id changed, which is the only reason to rewrite a cell.
    pub(crate) renumbered: bool,
}

/// The renumbering `live` implies, or `None` when every entry is live and there is nothing to do.
///
/// Shared by the two tables that are swept, because the arithmetic is the whole of the "table
/// order" decision and having it once is what keeps the two from drifting apart.
pub(crate) fn remap(live: &[bool]) -> Option<Remap> {
    let mut map = vec![DEAD; live.len()];
    let mut next = 0u32;
    let mut renumbered = false;
    for (old, &alive) in live.iter().enumerate() {
        if !alive {
            continue;
        }
        map[old] = next;
        renumbered |= next as usize != old;
        next += 1;
    }
    if next as usize == live.len() {
        return None;
    }
    Some(Remap { map, renumbered })
}

/// What one sweep did, in the counts spec §14 asks a gate to be made of.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) struct Swept {
    /// Whether any live handle moved, and therefore whether the mirror has to be told.
    ///
    /// **False when nothing below a live entry was freed** — register entry #11's second half.
    pub(crate) renumbered: bool,
    /// Clusters still pointed at by a cell.
    pub(crate) live_clusters: usize,
    /// Extended styles still pointed at by a cell.
    pub(crate) live_exts: usize,
    /// Cluster entries reclaimed.
    pub(crate) freed_clusters: usize,
    /// Extended-style entries reclaimed.
    pub(crate) freed_exts: usize,
}

/// Reclaim every entry of `tables` that no cell of `surfaces` points at.
///
/// Two passes over the cells and not one, for the same reason `add_content_with`'s donation walk
/// takes two: the mark pass is an array store per cell and the rewrite pass an array load, where a
/// single fused pass would have to decide a handle's fate before it had seen the whole surface.
///
/// The second pass is skipped outright when nothing renumbered, which is the common case for a
/// table that grew from the top and lost its top.
pub(crate) fn sweep(tables: &mut Tables, surfaces: &mut [&mut Surface]) -> Swept {
    let mut clusters_live = vec![false; tables.interner.len()];
    let mut exts_live = vec![false; tables.exts.len()];

    for surface in surfaces.iter() {
        for y in 0..surface.height() {
            for cell in surface.row(y) {
                if let Some(id) = cell.grapheme.cluster_id()
                    && let Some(slot) = clusters_live.get_mut(id as usize)
                {
                    *slot = true;
                }
                if let Some(handle) = cell.style.ext_handle()
                    && let Some(slot) = exts_live.get_mut(handle as usize)
                {
                    *slot = true;
                }
            }
        }
    }

    let swept = Swept {
        renumbered: false,
        live_clusters: clusters_live.iter().filter(|&&l| l).count(),
        live_exts: exts_live.iter().filter(|&&l| l).count(),
        freed_clusters: clusters_live.iter().filter(|&&l| !l).count(),
        freed_exts: exts_live.iter().filter(|&&l| !l).count(),
    };

    let clusters = tables.interner.compact(&clusters_live);
    let exts = tables.exts.compact(&exts_live);
    if clusters.is_none() && exts.is_none() {
        return swept;
    }

    for surface in surfaces.iter_mut() {
        for y in 0..surface.height() {
            for cell in surface.row_mut(y) {
                if let Some(map) = &clusters
                    && let Some(id) = cell.grapheme.cluster_id()
                    && let Some(&new) = map.get(id as usize)
                {
                    debug_assert_ne!(new, DEAD, "a live cell names a cluster the sweep dropped");
                    cell.grapheme = GraphemeId::cluster(new, cell.grapheme.is_wide_head());
                }
                if let Some(map) = &exts
                    && let Some(handle) = cell.style.ext_handle()
                    && let Some(&new) = map.get(handle as usize)
                {
                    debug_assert_ne!(
                        new, DEAD,
                        "a live cell names an extended style the sweep dropped"
                    );
                    cell.style = Style::extended(cell.style.attr_word(), new);
                }
            }
        }
    }

    Swept {
        renumbered: true,
        ..swept
    }
}

/// Everything one cell says, with every handle already resolved.
///
/// The oracle register entry #11 is made of: *after a sweep every live cell resolves to the same
/// channels it resolved to before.* A cell is compared through this rather than by its bytes,
/// because a sweep is **allowed** to change its bytes and forbidden to change what they mean — a
/// comparison of `Cell` values would fail on every correct sweep and a comparison of rendered text
/// alone would miss the four channels an extended style hides behind one handle.
#[cfg(test)]
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) struct Channels {
    /// The text the grapheme handle names, or `None` for `EMPTY` and `CONTINUATION`.
    text: Option<String>,
    /// The attribute bits, which no handle is involved in.
    attrs: u64,
    fg: crate::style::Color,
    bg: crate::style::Color,
    /// The underline's own colour. `Color::DEFAULT` on an inline cell, which has none.
    ul: crate::style::Color,
    /// The URI the hyperlink names, resolved through the link table rather than compared as an id —
    /// **the link table is not swept, so its ids do not move**, and resolving anyway is what would
    /// catch it if a future change swept it.
    link: Option<String>,
}

/// Every cell of `surface`, resolved, appended to `into`.
#[cfg(test)]
pub(crate) fn channels_of(surface: &Surface, tables: &Tables, into: &mut Vec<Channels>) {
    use crate::style::Color;

    for y in 0..surface.height() {
        for cell in surface.row(y) {
            let text = tables.interner.resolve(cell.grapheme);
            let attrs = cell.style.attr_word();
            let resolved = match cell.style.ext_handle() {
                Some(handle) => tables.exts.get(handle).map(|e| {
                    (
                        e.fg,
                        e.bg,
                        e.ul,
                        tables.links.uri(e.link).map(str::to_owned),
                    )
                }),
                None => Some((
                    cell.style.foreground(),
                    cell.style.background(),
                    Color::DEFAULT,
                    None,
                )),
            };
            // `None` is a handle the table does not name, which is a defect rather than a channel —
            // carried through as a distinguishable value so the assertion reports it instead of
            // panicking inside the oracle.
            let (fg, bg, ul, link) = resolved.unwrap_or((
                Color::DEFAULT,
                Color::DEFAULT,
                Color::DEFAULT,
                Some("<a handle this table does not name>".to_owned()),
            ));
            into.push(Channels {
                text,
                attrs,
                fg,
                bg,
                ul,
                link,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::Rect;
    use crate::scenes::{H, Scene, W, hyperlinked_page};
    use crate::style::{Color, Style};
    use crate::testing::{pinned_truecolor, screen_without_a_round_trip};

    /// §14's twelfth scene, with the two numbers the row was put on the list for.
    ///
    /// **A report, not a gate.** The shape of the growth is gated — `crate::gates`'
    /// `a_fading_operator_mints_per_distinct_style_and_never_per_cell` is a count and
    /// `tests/alloc.rs`'s settled arm is a zero — and these are the two absolute numbers spec §3
    /// measured, re-taken on the shipped mechanism at §13's full screen:
    ///
    /// | | entries created | per frame |
    /// |---|---|---|
    /// | a settled modal dim, 120 frames | 96 | 0.8 |
    /// | a fade in, 120 frames | 11 484 | 95.7 |
    ///
    /// It lives here rather than in `examples/budget.rs` for the reason ticket 04 moved the scene
    /// definitions one file across: **an example has only the public API**, and a count of table
    /// entries is not on it — ADR 0023 keeps cells off the public surface and the tables are one step
    /// behind the cells. The example owns the two *timings* of the same scene, which is what an
    /// example can take.
    #[test]
    fn the_hyperlinked_page_under_an_animating_operator_grows_only_while_it_animates() {
        const FRAMES: u32 = 120;
        let distinct = crate::scenes::HyperlinkedPageUnderAnOperator::distinct_styles();

        // **Entries created, not table length.** The sweep fires inside this loop — that is the
        // whole point of it — so a length delta would measure the sweep keeping up rather than the
        // growth it is keeping up with.
        let mut counted = Vec::new();
        let mut swept = Vec::new();
        for fading in [false, true] {
            let mut scene = hyperlinked_page();
            let mut screen = screen_without_a_round_trip(W, H, scene.overrides());
            scene.build(&mut screen);
            // Counted from **before** the first present, because the mixed results the first
            // composite mints are 96 of the 96 spec §3 attributes to the settled arm.
            let start = screen.extended_styles_minted();
            for t in 1..=FRAMES {
                if fading {
                    scene.step(&mut screen, t);
                } else {
                    scene.step_settled(&mut screen, t);
                }
                screen.present();
            }
            counted.push((screen.extended_styles_minted() - start) as usize);
            swept.push((screen.sweeps(), screen.table_lengths().1));
        }
        let (settled, fading) = (counted[0], counted[1]);

        println!(
            "\nthe hyperlinked page under an animating operator, {W}x{H}, {distinct} distinct \
             extended styles:\n  \
             a settled modal dim, {FRAMES} frames: {settled:>7} entries created, \
             {:>5.1} per frame, {} sweeps, {} entries held after\n  \
             a fade in,           {FRAMES} frames: {fading:>7} entries created, \
             {:>5.1} per frame, {} sweeps, {} entries held after\n  \
             spec §3 measured 96 / 0.8 and 11 484 / 95.7. **The last two columns are this \
             ticket's half**: the fading\n  \
             arm creates two orders of magnitude more entries than the settled one and ends holding \
             a bounded\n  \
             number of them rather than all of them — bounded by the high-water policy and not by \
             the frame\n  \
             count, which is what makes it a leak that was closed rather than one that got slower. \
             Report, not a gate.",
            settled as f64 / FRAMES as f64,
            swept[0].0,
            swept[0].1,
            fading as f64 / FRAMES as f64,
            swept[1].0,
            swept[1].1,
        );

        // The shape, which *is* asserted: a settled operator converges and a fading one does not.
        // Both bounds are orders of magnitude wide, because the numbers above are the report and a
        // report may not be load-bearing for a gate.
        assert!(
            settled <= distinct,
            "a settled operator created {settled} entries over {FRAMES} frames; it converges after \
             one"
        );
        assert!(
            fading >= distinct * (FRAMES as usize / 2),
            "a fading operator created only {fading} entries over {FRAMES} frames, so this report \
             is measuring something that is not a fade"
        );
    }

    /// What the sweep costs, at one screen and at twenty layers.
    ///
    /// **A report, not a gate**, and the one number on it that is load-bearing is not a duration:
    /// spec §13 lists the sweep among the frames *deliberately outside the incremental budget* —
    /// 1.17 ms at twenty layers — and what makes that legal is that it never runs on the render
    /// path, which `crate::gates::the_sweep_never_runs_inside_present` gates as a count.
    ///
    /// # The fixture has to leave something dead, and the first draft did not
    ///
    /// A sweep over a table with nothing dead in it takes the early return at the top of
    /// [`sweep`] — `remap` answers `None` for both tables and neither the compaction nor the
    /// rewrite pass runs at all. The first draft of this report built twenty layers of plain `'m'`
    /// with one link each, swept, and reported *the mark pass alone* as though it were spec §3's
    /// number. It is the same shape as the two vacuous operator gates this ticket found one file
    /// over, and the fix is the same one: **prove the mechanism ran before reporting what it cost**,
    /// which is why the timing loop asserts `renumbered` and a non-zero free count on every sample.
    ///
    /// So the fixture draws each layer twice — a distinct cluster and a distinct link, then a
    /// *second* distinct cluster and link — which leaves the odd-numbered entries of both tables
    /// dead and interleaved with the live ones. That is the expensive case by construction: every
    /// survivor moves, so the rewrite pass runs over every cell of every surface.
    ///
    /// # Why this does not use `vitui_bench::Bench`
    ///
    /// A sweep is not repeatable on its own fixture: the second one finds nothing dead and takes the
    /// early return, so a closure timed forty times would report one real sample and thirty-nine
    /// empty ones — the same defect one level up. `Bench` has no per-sample setup hook, so the
    /// minimum-of-N discipline is kept by hand over a fixture rebuilt untimed between samples.
    ///
    /// Spec §3's numbers are 58.88 µs for one screen and 1.17 ms for twenty layers. The ratio is the
    /// shape to read and it is not twenty: the work is two passes over the cells of every live
    /// surface and **the frame is one of the surfaces**, so the one-layer arm walks two screens and
    /// the twenty-layer arm walks twenty-one, and 21 / 2 is 10.5.
    ///
    /// ```text
    /// cargo test --release -p vitui-engine the_sweep_costs -- --nocapture
    /// ```
    #[test]
    fn the_sweep_costs_what_spec_3_recorded() {
        /// A screen of `layers` full-screen content layers whose handle tables are **half dead**.
        ///
        /// Each layer is drawn twice: a cluster in the first column of every row and a hyperlink
        /// over the whole of it, then a second cluster and a second hyperlink. The first of each
        /// pair is then pointed at by nothing, and the two pairs interleave down both tables, so
        /// every survivor moves.
        fn staged(layers: u16) -> crate::engine::Screen {
            let mut screen = screen_without_a_round_trip(W, H, pinned_truecolor());
            let row: String = std::iter::repeat_n('m', W as usize).collect();
            let ink = Style::new().fg(Color::indexed(15)).bg(Color::indexed(17));
            let ids: Vec<_> = (0..layers)
                .map(|i| {
                    screen
                        .layers()
                        // Opaque only at the bottom: the layers above have to stay visible in the
                        // frame, or the frame would name one layer's handles and the sweep would
                        // walk nineteen surfaces to find the same one entry.
                        .add_content(i as i32, Rect::new(0, 0, W, H), i == 0)
                })
                .collect();
            // `pass` 0 mints the entries that will die and `pass` 1 the ones that will live.
            for pass in 0..2u32 {
                for (i, &id) in ids.iter().enumerate() {
                    let mark = char::from_u32('\u{0300}' as u32 + pass).expect("a combining mark");
                    let base = char::from_u32('a' as u32 + i as u32 % 26).expect("a letter");
                    let cluster = format!("{base}{mark}");
                    let link =
                        screen.link(&format!("https://example.com/vitui#layer{i}pass{pass}"));
                    let mut view = screen.layers().view(id).expect("just added");
                    for y in 0..H as i32 {
                        view.text(0, y, &row, ink);
                        view.text(0, y, &cluster, ink);
                    }
                    view.restyle(
                        Rect::new(0, 0, W, H),
                        &crate::restyle::Restyle {
                            link: Some(link),
                            ..Default::default()
                        },
                    );
                }
                screen.present();
            }
            screen
        }

        /// The fastest of `rounds` sweeps, each on a fixture built fresh and untimed.
        fn fastest_sweep(layers: u16, rounds: u32) -> std::time::Duration {
            (0..rounds)
                .map(|_| {
                    let mut screen = staged(layers);
                    let at = std::time::Instant::now();
                    let swept = screen.sweep_now();
                    let took = at.elapsed();
                    // Prove the mechanism ran before reporting what it cost.
                    assert!(swept.renumbered, "the fixture left nothing to renumber");
                    assert_eq!(swept.freed_exts, layers as usize);
                    assert_eq!(swept.freed_clusters, layers as usize);
                    took
                })
                .min()
                .expect("at least one round")
        }

        // Ten rounds and not forty: every sample rebuilds and presents a whole stack, so the
        // fixture costs orders of magnitude more than the thing being timed and forty of them at
        // twenty layers is a minute of a debug build for a report.
        const ROUNDS: u32 = 10;
        let single = fastest_sweep(1, ROUNDS).as_secs_f64() * 1e6;
        let stacked = fastest_sweep(20, ROUNDS).as_secs_f64() * 1e6;
        println!(
            "\nthe handle-table sweep, fastest of {ROUNDS}, each on a fixture built fresh:\n  \
             one screen     {single:>9.2} us   spec §3 measured   58.88 us\n  \
             twenty layers  {stacked:>9.2} us   spec §3 measured 1 170 us\n  \
             twenty / one = {:.2}x, against 10.5x of surfaces walked: two screens (one layer plus \
             the frame)\n  \
             against twenty-one. Linear in surfaces is the shape, and the frame being one of them \
             is why the\n  \
             divisor is two rather than one. Report, not a gate: what is gated is that it never \
             runs on the\n  \
             render path.",
            stacked / single,
        );
    }

    #[test]
    fn a_table_with_nothing_dead_in_it_is_not_remapped_at_all() {
        assert!(remap(&[true, true, true]).is_none());
        assert!(remap(&[]).is_none());
    }

    #[test]
    fn freeing_only_above_the_last_live_entry_frees_without_renumbering() {
        // Spec §3's cheap case, and the half of register entry #11 that is about `renumbered`.
        let r = remap(&[true, true, false, false]).expect("two entries were freed");
        assert!(!r.renumbered);
        assert_eq!(r.map, vec![0, 1, DEAD, DEAD]);
    }

    #[test]
    fn freeing_below_a_live_entry_renumbers_it() {
        let r = remap(&[false, true, false, true]).expect("two entries were freed");
        assert!(r.renumbered);
        assert_eq!(r.map, vec![DEAD, 0, DEAD, 1]);
    }

    #[test]
    fn a_table_with_nothing_live_in_it_empties_without_renumbering_anything() {
        let r = remap(&[false, false]).expect("both entries were freed");
        assert!(!r.renumbered, "there is no survivor to have moved");
        assert_eq!(r.map, vec![DEAD, DEAD]);
    }
}
