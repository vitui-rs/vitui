//! **Render one scene two ways and compare it cell for cell**, from a crate that cannot name the
//! engine.
//!
//! This is the primitive every hostile-axis ticket on the backlog is written against. Three of
//! spec §17's four axes were caught **only** by an equality against a reference render, and every
//! one of them made the defective build look *healthier* — the inverted scroll drew nothing at
//! **12.21 us against 62.96**, the stale tail is **71 of 80 rows** with the defective build **2.3x
//! faster marking 226x less**, and twenty wheel clicks moved the offset **0 against 16**. No counter
//! in the stack disapproved of any of them. An equality did.
//!
//! # The reference arm is not engine ticket 04's, and that is a result rather than a shortcut
//!
//! `.scratch/vitui-components-impl/issues/04` says *the second implementation is engine 04's, not a
//! new one* and asks this ticket to name what the engine's compositor cannot reach. The answer is
//! **all of it**, and there are three independent barriers, any one of which is sufficient:
//!
//! 1. **The module is private.** `crates/vitui-engine/src/lib.rs` declares `mod reference;` — not
//!    `pub mod` — so `vitui_engine::reference` is unreachable from *any* other crate, including
//!    `vitui-runtime`, which is allowed to name the engine.
//! 2. **The module is `cfg`-gated to configurations no dependent compiles.** The declaration is
//!    `#[cfg(any(test, feature = "fuzz"))]`. `cfg(test)` is set only when the engine is compiled as
//!    a test target *of itself*, and the `fuzz` feature is off by default and reachable only from
//!    `fuzz/`, which is a detached workspace. A dependency never sees the module at all.
//! 3. **This crate cannot name `vitui_engine`.** Components spec §0's constraint C6: the
//!    `[dependencies]` table is `vitui-runtime` and nothing else, gated by
//!    `line::tests::the_components_manifest_names_only_the_runtime` one crate down and by
//!    `deny.toml`'s `{ name = "vitui-engine", wrappers = ["vitui-runtime", "vitui"] }`. A
//!    `use vitui_engine::…` here is `error[E0432]` before anything runs.
//!
//! So **the components-side reference render is the rule and not the exception**, and what it is a
//! reference *for* changes with it. There is a fourth barrier standing behind the first three that
//! would survive removing all of them:
//!
//! 4. **Nothing reads a cell.** ADR 0023 — *the cell is never visible in the public API* — is a
//!    decision, and it holds against the engine's own callers as well as this crate's: no `Surface`,
//!    `View`, `Screen` or `Presented` method returns a cell, a handle or a style bit. That is
//!    [`crate::gates::REGISTER`]'s row 2, `Unreachable`, and **this module does not invert it**.
//!
//! # What is compared, then, and why it is still the gate that catches all four axes
//!
//! Two draw implementations of one scene, recorded **at the verb boundary** and compared cell for
//! cell. [`Pen`] wraps a real [`Ctx`] and records what the engine says landed:
//! `Ctx::text` and `Ctx::set` return `Written`, whose `cells` field is how many columns were
//! actually written *after clipping*, so the recording is the engine's own report of its own work
//! and not a guess about it. What the two arms differ in is the drawing logic, which is where every
//! one of the four defects lives:
//!
//! | | row 2 of the register | this module |
//! |---|---|---|
//! | what is compared | two **composited surfaces** | two **recorded draws** |
//! | who reports the extent of a write | the compositor | the engine's `Written::cells` |
//! | catches a wrong offset, a missing row, a lost column | yes | **yes** |
//! | catches a compositing defect below the verb | yes | no — the verbs are identical there |
//!
//! The second column is what components 37 buys and the third is what this ticket buys. A defect
//! that both arms commit identically is invisible to either.
//!
//! # The oracle is allowed to be slow and is not allowed to be clever
//!
//! [`reference()`] visits every cell of the rectangle, every frame, whether anything changed or not,
//! and writes it with one [`Ctx::set`]. It is engine `reference.rs`'s shape and its rule, restated
//! one layer up because the file itself is out of reach: *share no code with the fast path*.
//! [`rows_at_a_time`] is the fast path — one `Ctx::text` a row, the shape a component actually
//! draws.
//!
//! # The four defects ship as fixtures, because a gate nobody has watched fail is not a gate
//!
//! [`defective`] holds one painter per axis, each writing the defect §17 names in the words §17
//! names it. They are the evidence that the runner works: a runner validated only against a correct
//! painter reports `0 cells over 0 rows` for the same reason a broken one would.

use std::collections::BTreeSet;
use std::fmt;
use std::time::{Duration, Instant};

use vitui_runtime::ctx::Driver;
use vitui_runtime::layout::text;
use vitui_runtime::theme::CATPPUCCIN_MOCHA;
use vitui_runtime::{Ctx, Density, Paint, Role, Theme};

use crate::counters::{Allocations, Counter, Counters, Tally};

/// One cell of a recorded surface: what was written there.
///
/// The continuation half of a double-width cluster is an **empty** cluster with the head's paint,
/// which is the engine's own arrangement — a wide glyph occupies two cells and the second one is a
/// distinct thing rather than a copy. Comparing it as a copy would make a build that lost the head
/// and kept the tail compare equal on the tail.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Cell {
    /// The cluster written here, or empty for the continuation half of a wide one.
    pub cluster: String,
    /// The paint it was written with.
    pub paint: Paint,
    /// **The role a deferred hover award restyled this cell's background to**, since it was drawn.
    ///
    /// `None` after every drawing verb, because a draw writes the whole cell and leaves no restyle
    /// standing over it. `Some(role)` after [`Canvas::award`], and only where the award actually
    /// changed something — a cell already painted in `role` is left `None`, because restyling a
    /// background to the background it already carries writes nothing the terminal can see.
    ///
    /// It is a field on the cell rather than a second surface because the engine has one cell: a
    /// draw and a restyle land in the same place, and *which of the two spoke last* is the whole of
    /// what [`Canvas::repaints`] is counting.
    pub hover: Option<Role>,
}

/// A recorded surface: `w x h` cells, each either written or never touched.
///
/// **Never touched is its own value and not a blank.** A cell nobody wrote keeps whatever was
/// already there, and *what was already there is almost always right* — the whole reason
/// [`crate::counters::sentinel`] exists. Modelling it as a space would make the two arms agree on
/// exactly the cells the shrink axis is about.
/// # It is also the only instrument that can answer *how much of this frame was a change*
///
/// `marked` — spec §20's damaged-cell count — is [`crate::counters::Reading::Unreachable`] and
/// stays that way: `crates/vitui-engine/src/damage.rs` is `pub(crate)` throughout and `Presented`
/// carries no count, so nothing above the engine can read the engine's damage. What **is** knowable
/// from here is the rule that decides it: *the engine filters a write whose value equals the cell's
/// current value* ([`crate::counters::sentinel`]'s own sentence), so the measurable quantity is
/// **writes whose value differs from what is already there**. That is [`Canvas::repaints`], and it
/// is a model rather than a report — see it for exactly where the model is exact and where it is
/// conservative.
#[derive(Clone, Debug)]
pub struct Canvas {
    w: u16,
    h: u16,
    cells: Vec<Option<Cell>>,
    /// Rect whose value a verb has changed since the last [`Canvas::take_repaints`]. **Distinct
    /// cells and not events**: a chip cell that a draw changes and a restyle then changes back is
    /// one cell the frame re-damaged, which is the unit ADR 0026 prices its five instances in.
    repainted: BTreeSet<(u16, u16)>,
}

impl Canvas {
    /// An untouched surface.
    pub fn new(w: u16, h: u16) -> Canvas {
        Canvas {
            w,
            h,
            cells: vec![None; usize::from(w) * usize::from(h)],
            repainted: BTreeSet::new(),
        }
    }

    /// Its width.
    pub fn w(&self) -> u16 {
        self.w
    }

    /// Its height.
    pub fn h(&self) -> u16 {
        self.h
    }

    /// What is at `(x, y)`, or `None` for a cell nobody has written.
    pub fn get(&self, x: u16, y: u16) -> Option<&Cell> {
        if x >= self.w || y >= self.h {
            return None;
        }
        self.cells[usize::from(y) * usize::from(self.w) + usize::from(x)].as_ref()
    }

    /// How many cells have been written at least once.
    pub fn written(&self) -> usize {
        self.cells.iter().filter(|c| c.is_some()).count()
    }

    fn put(&mut self, x: i32, y: i32, cell: Cell) {
        let Some(at) = self.index(x, y) else {
            return;
        };
        if self.cells[at].as_ref() != Some(&cell) {
            self.repainted.insert((x as u16, y as u16));
        }
        self.cells[at] = Some(cell);
    }

    /// The flat index of `(x, y)`, or `None` when it is off the surface.
    fn index(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 {
            return None;
        }
        let (cx, cy) = (x as usize, y as usize);
        (cx < usize::from(self.w) && cy < usize::from(self.h))
            .then_some(cy * usize::from(self.w) + cx)
    }

    /// **Apply a deferred hover award to this surface**, as the runtime applies it: a background
    /// restyle over the rectangle, after the draw and before `present`.
    ///
    /// `as_painted` is `theme.paint(role)` — handed in because a [`Paint`] is opaque (ADR 0018:
    /// a component names a role and never a colour) and this crate cannot ask a paint what its
    /// background is. It is what makes the model **exact in the case that matters**: a cell already
    /// carrying `theme.paint(role)` carries `role`'s background, so restyling it to `role` writes
    /// nothing and the cell is left alone.
    ///
    /// Every other cell is recorded as changed. That is the model's one conservative direction, and
    /// it is closed at the fixture rather than assumed away: two roles can share a background while
    /// differing in foreground, and
    /// [`Theme::roles_differ_on_wire`](vitui_runtime::Theme::roles_differ_on_wire) is the question a
    /// fixture asks of the roles it is about — `crate::state`'s chip asserts it of the three faces
    /// it uses.
    pub fn award(&mut self, x: i32, y: i32, w: u16, h: u16, role: Role, as_painted: Paint) {
        for dy in 0..i32::from(h) {
            for dx in 0..i32::from(w) {
                let (cx, cy) = (x + dx, y + dy);
                let Some(at) = self.index(cx, cy) else {
                    continue;
                };
                let Some(cell) = self.cells[at].as_mut() else {
                    // A cell nobody has written has no background for the restyle to change. The
                    // engine would restyle it; what it holds is whatever was there before this
                    // surface began, which is the residue `crate::counters::sentinel` is about and
                    // which this instrument deliberately does not model.
                    continue;
                };
                if cell.paint == as_painted {
                    continue;
                }
                if cell.hover != Some(role) {
                    self.repainted.insert((cx as u16, cy as u16));
                }
                cell.hover = Some(role);
            }
        }
    }

    /// **How many distinct cells a verb has changed the value of** since the last
    /// [`Canvas::take_repaints`].
    ///
    /// # This is the honest form of `marked`, and it is a model
    ///
    /// It is not the engine's damage count and cannot become one: `damage.rs` is `pub(crate)`
    /// throughout, `Presented` carries no count, and [`crate::counters::Counters::marked`] therefore
    /// panics rather than answering `0`. What this counts is the **input** to the engine's equality
    /// filter — a write whose value differs from the resident value — over a surface this crate
    /// keeps itself, at the verb boundary, from the engine's own report of how many columns each
    /// verb landed.
    ///
    /// It is exact for the draw verbs, whose whole value ([`Cell`]) is known here. It is exact for
    /// an award onto a cell already painted in the awarded role, which is the case a correct widget
    /// produces every frame. It is **conservative** for an award onto any other cell: the model
    /// records a change, and the engine would agree unless the two roles share a background, which
    /// a fixture closes by asking `Theme::roles_differ_on_wire`.
    ///
    /// What it does not model at all is a cell nobody in this surface has written — see
    /// [`crate::counters::sentinel`] for why that is a different, and unreachable, question.
    pub fn repaints(&self) -> u64 {
        self.repainted.len() as u64
    }

    /// Read [`Canvas::repaints`] and start the next frame's count from zero.
    pub fn take_repaints(&mut self) -> u64 {
        let n = self.repaints();
        self.repainted.clear();
        n
    }

    /// One row as text, with an untouched cell spelled as a space and the trailing run trimmed.
    ///
    /// **The lossy view, and it is the right one for a content comparison across two sizes.** A row
    /// drawn at 300 columns and the same row drawn at 60 differ in every cell past column 59 and
    /// carry the same content; comparing `Option<Cell>` there would report a difference in the
    /// geometry and call it a difference in the content. [`Canvas::diff`] is the lossless one, and
    /// it is what the two-arm equality uses.
    pub fn row_text(&self, y: u16) -> String {
        let mut out = String::new();
        for x in 0..self.w {
            match self.get(x, y) {
                Some(cell) if cell.cluster.is_empty() => {}
                Some(cell) => out.push_str(&cell.cluster),
                None => out.push(' '),
            }
        }
        out.trim_end().to_string()
    }

    /// Every row as text. See [`Canvas::row_text`].
    pub fn content(&self) -> Vec<String> {
        (0..self.h).map(|y| self.row_text(y)).collect()
    }

    /// Cell for cell against another surface of the same size.
    ///
    /// # Panics
    ///
    /// Panics on a size mismatch. Two surfaces of different sizes have no cell-for-cell comparison
    /// to make, and returning *every cell differs* would be a number rather than an answer — the
    /// two-size question is [`at_two_sizes`] and it is about content, not cells.
    pub fn diff(&self, other: &Canvas) -> Diff {
        assert_eq!(
            (self.w, self.h),
            (other.w, other.h),
            "two surfaces of different sizes have no cell-for-cell comparison"
        );
        let mut cells = 0usize;
        let mut rows = 0usize;
        let mut first = None;
        for y in 0..self.h {
            let mut row_differs = false;
            for x in 0..self.w {
                if self.get(x, y) != other.get(x, y) {
                    cells += 1;
                    row_differs = true;
                    if first.is_none() {
                        first = Some((x, y));
                    }
                }
            }
            if row_differs {
                rows += 1;
            }
        }
        Diff {
            cells,
            rows,
            first,
            over: (self.w, self.h),
        }
    }
}

/// **`n` cells over `m` rows** — the form every defect on the map was legible in.
///
/// The row count is not derivable from the cell count and carries the shape of the failure: 6 662
/// cells over 80 rows is a whole screen wrong, 6 662 over 9 is a band, and *71 of 80 rows* is how
/// the stale tail was actually stated. A count of cells alone reports both as one number.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Diff {
    /// How many cells the two arms disagree about.
    pub cells: usize,
    /// How many rows carry at least one of them.
    pub rows: usize,
    /// The first disagreement in reading order, which is what a reader looks at next.
    pub first: Option<(u16, u16)>,
    /// The surface both arms were drawn on.
    pub over: (u16, u16),
}

impl Diff {
    /// Whether the two arms agree everywhere.
    pub fn clean(self) -> bool {
        self.cells == 0
    }

    /// Fail with the counts and the first disagreement.
    ///
    /// # Panics
    ///
    /// Panics when the two arms disagree anywhere.
    #[track_caller]
    pub fn assert_clean(self, scene: &str) {
        assert!(self.clean(), "{scene}: {self}");
    }
}

impl fmt::Display for Diff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} cells over {} rows of {}x{}",
            self.cells, self.rows, self.over.0, self.over.1
        )?;
        if let Some((x, y)) = self.first {
            write!(f, ", first at ({x}, {y})")?;
        }
        Ok(())
    }
}

/// A pen that draws through a real [`Ctx`] and records what the engine says landed.
///
/// # The recording is the engine's report and the placement is ours
///
/// `Ctx::text` and `Ctx::set` return `Written`, whose `cells` field is how many columns were written
/// **after clipping**, so the extent of every write comes from the engine. Where the columns are is
/// this crate's arithmetic: the verb started at `x` and ran for `cells` columns. That is exact for a
/// verb that starts inside its context and wrong by the discarded prefix for one that starts left of
/// the clip — ADR 0022's clamp-and-discard, and [`Tally`] states the same caveat for the same reason.
/// A fixture drawing outside its own rectangle is already failing spec §2's partition rule, which is
/// what the tally beside the canvas is measuring.
#[derive(Debug)]
pub struct Pen {
    canvas: Canvas,
    tally: Tally,
    awards: Vec<Award>,
}

/// **One deferred hover award a frame declared**, as the instrument sees it.
///
/// `Ctx::hover_style` declares an intent and `Driver::frame` applies it *after the draw and before
/// `present`*, which is what makes it land in the same frame. A model that applied it at the
/// declaration would have the restyle happen before the widget's own cells were written, and would
/// then score the widget's draw as undoing it — the opposite of the relation ADR 0026 states. So an
/// award is recorded here and applied by [`Pen::end_frame`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Award {
    /// Its left edge, in the coordinates the verbs were called in.
    pub x: i32,
    /// Its top edge.
    pub y: i32,
    /// Its width.
    pub w: u16,
    /// Its height.
    pub h: u16,
    /// The role the background is restyled to.
    pub role: Role,
    /// `theme.paint(role)`, carried because a [`Paint`] is opaque. See [`Canvas::award`].
    pub painted: Paint,
    /// **Whether the runtime would actually apply it this frame** — `Frame::hover_to_apply` picks
    /// exactly one rectangle, the one belonging to the id the award named as hovered.
    pub applied: bool,
}

impl Pen {
    /// A pen over an untouched surface.
    pub fn new(w: u16, h: u16) -> Pen {
        Pen::over(Canvas::new(w, h))
    }

    /// **A pen over a surface a previous frame left behind**, which is what makes a steady-state
    /// measurement possible at all.
    ///
    /// A `Pen::new` per frame reports every cell as a first paint, for ever: *re-damage* is a
    /// relation between two frames and there is nothing to relate it to. The tally starts empty —
    /// `writes`, `distinct` and `verbs` are per-frame counters and always have been — and the
    /// canvas does not, because the resident value is exactly the thing that carries over.
    pub fn over(canvas: Canvas) -> Pen {
        Pen {
            canvas,
            tally: Tally::new(),
            awards: Vec::new(),
        }
    }

    /// Take the surface back, to hand to the next frame's [`Pen::over`].
    pub fn into_canvas(self) -> Canvas {
        self.canvas
    }

    /// Record a deferred hover award. See [`Award`] and [`crate::ink::Ink::award`].
    pub fn declare(&mut self, award: Award) {
        self.awards.push(award);
    }

    /// The awards this frame declared, in declaration order.
    pub fn awards(&self) -> &[Award] {
        &self.awards
    }

    /// **Apply this frame's awards, where `Driver::frame` applies them** — after the draw and
    /// before `present`.
    ///
    /// A harness that forgets this measures a widget that declared an award and never got one,
    /// which scores every arm as free. `crate::state::resting` is the one caller, and
    /// `state::tests::the_award_is_applied_after_the_draw_and_not_before` is what watches the
    /// ordering matter.
    pub fn end_frame(&mut self) {
        for award in std::mem::take(&mut self.awards) {
            if !award.applied {
                continue;
            }
            self.canvas.award(
                award.x,
                award.y,
                award.w,
                award.h,
                award.role,
                award.painted,
            );
        }
    }

    /// Draw a string, record where it landed, and return the engine's own column count.
    pub fn text(&mut self, cx: &mut Ctx<'_, '_>, x: i32, y: i32, s: &str, st: Paint) -> u16 {
        let columns = self.tally.text(cx, x, y, s, st);
        self.record(x, y, s, columns, st);
        columns
    }

    /// Draw one cluster, record it, and return the engine's own column count.
    ///
    /// The verb the reference render is written on: one cell at a time, no runs.
    pub fn set(&mut self, cx: &mut Ctx<'_, '_>, x: i32, y: i32, cluster: &str, st: Paint) -> u16 {
        let columns = self.tally.set(cx, x, y, cluster, st);
        self.record(x, y, cluster, columns, st);
        columns
    }

    fn record(&mut self, x: i32, y: i32, s: &str, columns: u16, st: Paint) {
        for (col, width, cluster) in clusters(s, columns) {
            self.canvas.put(
                x + i32::from(col),
                y,
                Cell {
                    cluster: cluster.to_string(),
                    paint: st,
                    hover: None,
                },
            );
            for tail in 1..width {
                self.canvas.put(
                    x + i32::from(col) + i32::from(tail),
                    y,
                    Cell {
                        cluster: String::new(),
                        paint: st,
                        hover: None,
                    },
                );
            }
        }
    }

    /// What has been drawn.
    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    /// The three counters no frame structure holds — `writes`, `distinct` and `verbs`.
    pub fn tally(&self) -> &Tally {
        &self.tally
    }
}

/// The clusters of `s` that fit in `limit` columns, as `(column, width, cluster)`.
///
/// Built from [`text::truncate`] rather than from a cluster iterator, because the runtime exposes
/// the former and not the latter — and it is the better source anyway: truncation is the **engine's
/// own** width table reached through the runtime's re-export, so the oracle and the fast path
/// segment a string the same way and a disagreement between them is never about UAX #29.
fn clusters(s: &str, limit: u16) -> Vec<(u16, u16, &str)> {
    let mut out = Vec::new();
    let mut prev_len = 0usize;
    let mut prev_col = 0u16;
    for col in 1..=limit {
        let len = text::truncate(s, col).len();
        if len > prev_len {
            out.push((prev_col, col - prev_col, &s[prev_len..len]));
            prev_len = len;
            prev_col = col;
        }
    }
    out
}

/// The content a scene stands up, inside **a rectangle that does not move**.
///
/// # The rectangle is a field and the content is a field, and that is the whole point
///
/// > *The surface after a shrink equals a freshly built one*, written against a terminal resize,
/// > passes on all twelve panels — because `Gallery::resize` allocates a new `Surface` and the
/// > residue has nowhere to survive. **That spelling tests the resize path and not the defect**,
/// > which is content shrinking inside a rectangle that does not move. (§21)
///
/// [`Fixture::shrunk_to`] changes `rows` and leaves `w` and `h` alone; [`Fixture::resized`] changes
/// `w` and `h`. Both exist and they are not the same gesture — the second is the spelling §21 says
/// may stand beside the first and may not stand instead of it.
///
/// # It is small on purpose
///
/// §21's scenes are stated at a million rows and 5 475 600 pairs. Those magnitudes live on
/// [`crate::scenes::Scene`], which is a value describing a screen; a fixture is what a test actually
/// stands up, and the reference arm is `w * h` calls to `Ctx::set` a frame. The scene says how big
/// the screen was; the fixture says how big this run is.
#[derive(Clone, Debug)]
pub struct Fixture {
    rows: Vec<String>,
    offset: usize,
    w: u16,
    h: u16,
}

impl Fixture {
    /// A fixture of `rows` generated lines in a `w x h` rectangle, scrolled to the top.
    ///
    /// The lines are ASCII and single-width. A fixture with a wide cluster in it is a fixture about
    /// UAX #29, and this module's private `clusters` helper documents why that is not the question
    /// this runner asks.
    ///
    /// # No two rows share a column, and that is what makes the cell count mean something
    ///
    /// Column `c` of row `i` is `(5i + c) mod 26` as a letter, so two rows agree at one column only
    /// if they agree at **all** of them — `i ≡ j (mod 26)`. A wrong row therefore costs exactly `w`
    /// cells, and *n cells over m rows* reads as `m * w`.
    ///
    /// A fixture whose rows share a prefix — `row 000123 of content`, the obvious spelling —
    /// reports a **smaller** cell count for the same wrong row, because the prefix matches whatever
    /// row was drawn. That is a fact about the fixture and not about the defect, and it is the same
    /// mistake in miniature as the twelve prototypes reporting `allocs / n`: the number goes down
    /// and nothing is better.
    pub fn lines(w: u16, h: u16, rows: usize) -> Fixture {
        let width = usize::from(w).max(64);
        Fixture::of(
            w,
            h,
            (0..rows)
                .map(|i| {
                    (0..width)
                        .map(|c| char::from(b'a' + ((i * 5 + c) % 26) as u8))
                        .collect()
                })
                .collect(),
        )
    }

    /// A fixture over content the caller wrote.
    pub fn of(w: u16, h: u16, rows: Vec<String>) -> Fixture {
        Fixture {
            rows,
            offset: 0,
            w,
            h,
        }
    }

    /// The same content, viewed from a different offset. The `scrolled` and `wheeled` gesture.
    pub fn scrolled_to(&self, offset: usize) -> Fixture {
        Fixture {
            offset,
            ..self.clone()
        }
    }

    /// **Less content, in the same rectangle.** The `shrunk` gesture, in §21's own spelling.
    pub fn shrunk_to(&self, rows: usize) -> Fixture {
        let mut next = self.clone();
        next.rows.truncate(rows);
        next
    }

    /// The same content, in a different rectangle. The resize spelling, which may stand **beside**
    /// [`Fixture::shrunk_to`] and not instead of it.
    pub fn resized(&self, w: u16, h: u16) -> Fixture {
        Fixture {
            w,
            h,
            ..self.clone()
        }
    }

    /// The rectangle it is drawn into.
    pub fn size(&self) -> (u16, u16) {
        (self.w, self.h)
    }

    /// How many rows of content it holds.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Whether it holds no content at all — a legitimate state, and the one the stale tail is about.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// The first visible row.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// The content of screen row `y`, padded and truncated to the rectangle's width.
    ///
    /// A row past the end of the content is blank, which is **the correct answer and the one the
    /// stale tail gets wrong**: a component that stops drawing when it runs out of rows leaves the
    /// previous frame's on screen.
    pub fn row_text(&self, y: u16) -> String {
        let line = self
            .rows
            .get(self.offset + usize::from(y))
            .map(String::as_str)
            .unwrap_or("");
        let line = text::truncate(line, self.w);
        let mut out = String::from(line);
        for _ in text::width(line)..self.w {
            out.push(' ');
        }
        out
    }

    /// The cluster the reference expects at `(x, y)`.
    pub fn cell(&self, x: u16, y: u16) -> String {
        let row = self.row_text(y);
        match clusters(&row, self.w).into_iter().find(|(c, _, _)| *c == x) {
            Some((_, _, cluster)) => cluster.to_string(),
            None => " ".to_string(),
        }
    }
}

/// What a scene's content is drawn by. Two of these and one [`Fixture`] make an equality.
pub type Painter = fn(&mut Pen, &mut Ctx<'_, '_>, &Fixture);

/// **The reference render: one cell at a time, no runs, no offsets it did not compute itself.**
///
/// Engine `reference.rs`'s shape and its rule — *the oracle is allowed to be slow and is not allowed
/// to be clever* — restated here because that file is unreachable from this crate for the three
/// independent reasons in this module's header. It shares no line with [`rows_at_a_time`]: it asks
/// of every cell, independently, *what belongs here*, where the fast path asks of every row *what
/// string goes on it*.
pub fn reference(pen: &mut Pen, cx: &mut Ctx<'_, '_>, fx: &Fixture) {
    let body = cx.theme().paint(Role::Body);
    let (w, h) = fx.size();
    for y in 0..h {
        for x in 0..w {
            let cluster = fx.cell(x, y);
            pen.set(cx, i32::from(x), i32::from(y), &cluster, body);
        }
    }
}

/// **The fast path: one `Ctx::text` a row**, which is the shape a component actually draws.
///
/// It writes the whole width of every row of the rectangle, including the rows past the end of the
/// content — which is what [`defective::stale_tail`] does not.
pub fn rows_at_a_time(pen: &mut Pen, cx: &mut Ctx<'_, '_>, fx: &Fixture) {
    let body = cx.theme().paint(Role::Body);
    let (_, h) = fx.size();
    for y in 0..h {
        let line = fx.row_text(y);
        pen.text(cx, 0, i32::from(y), &line, body);
    }
}

/// **One painter per hostile axis, each writing the defect §17 names.**
///
/// A runner validated only against a correct painter reports `0 cells over 0 rows` for the same
/// reason a broken one would, and §21 has that finding three times over from the other direction: a
/// CI job nobody has watched go green is not a gate, whatever it checks. These are what the runner
/// has been watched catching.
///
/// They are `pub` deliberately. The engine ships an obviously-correct-and-far-too-slow compositor
/// for the same kind of reason: an instrument crate's fixtures are part of the instrument.
pub mod defective {
    use super::{Fixture, Pen};
    use vitui_runtime::{Ctx, Role};

    /// **The inverted scroll sign: 12.21 us / 3 058 writes against 62.96 / 20 418, and *faster*.**
    ///
    /// `offset - y` where the content is at `offset + y`. Found three times independently, and every
    /// counter in the stack approved of it: it draws nothing, so it is fast and marks almost nothing.
    pub fn inverted_scroll(pen: &mut Pen, cx: &mut Ctx<'_, '_>, fx: &Fixture) {
        let body = cx.theme().paint(Role::Body);
        let (w, h) = fx.size();
        let inverted = fx.scrolled_to(0);
        for y in 0..h {
            let source = fx.offset().wrapping_sub(usize::from(y));
            let line = if source < fx.len() {
                inverted.scrolled_to(source).row_text(0)
            } else {
                " ".repeat(usize::from(w))
            };
            pen.text(cx, 0, i32::from(y), &line, body);
        }
    }

    /// **The stale tail: 71 of 80 rows, the defective build 2.3x faster marking 226x less.**
    ///
    /// It stops when it runs out of content instead of clearing the rest of its rectangle, so what
    /// stays on screen is the previous frame — spec §2's *no cell never*, from the direction no
    /// counter watches. Visible only across two frames, which is why [`super::play`] takes a
    /// sequence of steps and does not clear the surface between them.
    pub fn stale_tail(pen: &mut Pen, cx: &mut Ctx<'_, '_>, fx: &Fixture) {
        let body = cx.theme().paint(Role::Body);
        let (_, h) = fx.size();
        let drawable = fx.len().saturating_sub(fx.offset()).min(usize::from(h));
        for y in 0..drawable as u16 {
            let line = fx.row_text(y);
            pen.text(cx, 0, i32::from(y), &line, body);
        }
    }

    /// **Twenty wheel clicks move the offset 0 against 16**, in four resolved tickets' code.
    ///
    /// It draws from the top whatever the offset says, so every wheel gesture is a no-op and the
    /// screen is identical on every frame — which is exactly what a damage counter rewards.
    pub fn unmoved_wheel(pen: &mut Pen, cx: &mut Ctx<'_, '_>, fx: &Fixture) {
        super::rows_at_a_time(pen, cx, &fx.scrolled_to(0));
    }

    /// **Green at 300x80 and red at 60x20** — §13's overlap, the one axis on which a component's
    /// construction changes rather than its contents.
    ///
    /// Below twenty columns it reserves two columns for an affordance and never writes them, so the
    /// last two columns of every row keep whatever was there. At 300 columns the branch is not taken
    /// and the arm is identical to [`super::rows_at_a_time`].
    pub fn narrow_drops_a_column(pen: &mut Pen, cx: &mut Ctx<'_, '_>, fx: &Fixture) {
        let body = cx.theme().paint(Role::Body);
        let (w, h) = fx.size();
        let drawn = if w < 20 { w.saturating_sub(2) } else { w };
        for y in 0..h {
            let line = fx.row_text(y);
            let line = vitui_runtime::layout::text::truncate(&line, drawn);
            pen.text(cx, 0, i32::from(y), line, body);
        }
    }
}

/// One played scene: the surface it drew, what it cost, and the driver it drew through.
pub struct Run {
    pen: Pen,
    driver: Driver,
    elapsed: Duration,
    frames: u32,
}

impl Run {
    /// What was drawn.
    pub fn canvas(&self) -> &Canvas {
        self.pen.canvas()
    }

    /// The draw-side counters.
    pub fn tally(&self) -> &Tally {
        self.pen.tally()
    }

    /// How long the frames took. **A report and never a gate** — R15's rule, inherited unchanged.
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// How many frames were played.
    pub fn frames(&self) -> u32 {
        self.frames
    }

    /// The nine per-frame counters, of which this crate can read eight.
    ///
    /// The allocation total is the caller's, because [`vitui_alloc_probe`] is a dev-dependency and a
    /// library cannot install a global allocator on a consumer's behalf. There is no default: a
    /// figure defaulted to zero is a counter that prints `0` when it means *nobody counted*, which
    /// is the shape [`crate::counters::Reading`] exists to end.
    ///
    /// [`vitui_alloc_probe`]: https://docs.rs/vitui-alloc-probe
    pub fn counters(&self, allocations: Allocations) -> Counters {
        Counters::of(&self.driver, self.pen.tally(), allocations)
    }

    /// The one-line metric row §21 reports every scene in.
    pub fn row(&self, scene: &'static str, allocations: Allocations) -> MetricRow {
        MetricRow {
            scene,
            elapsed: self.elapsed,
            frames: self.frames,
            counters: self.counters(allocations),
            allocations,
        }
    }
}

/// **`us / marked / writes / verbs / regions / stops / allocations`, in one line.**
///
/// So that a later ticket's numbers are comparable with §21's without re-deriving the format.
///
/// # `marked` prints `unreachable`, and printing `0` there would be the defect this file is about
///
/// The damaged-cell count is not on the engine's *public* surface for anyone —
/// `crates/vitui-engine/src/damage.rs` is `pub(crate)` from top to bottom and `Presented` carries
/// `submitted`, `coalesced` and `discarded_for_resize` and no count of cells. So
/// [`Counters::marked`] is [`crate::counters::Reading::Unreachable`] and this row prints the word.
///
/// A `0` there would be worse than a gap in exactly the way §21's first refinement describes: *a
/// threshold on the wrong side of the question is not a weak gate, it is a green one*. Two of the
/// four axes were established by a build that **marked less** — the stale tail marks 226x less than
/// the correct one — so a report showing `marked=0` next to a healthy-looking microsecond figure is
/// the defective build's own self-portrait.
///
/// The microsecond figure is a **report**. R15's rule is inherited unchanged: *a gate is a count, a
/// ratio, an equality or a compile outcome; a timing is a report* — and the reason is on this page
/// too, because three of the four axes were faster.
#[derive(Clone, Copy, Debug)]
pub struct MetricRow {
    scene: &'static str,
    elapsed: Duration,
    frames: u32,
    counters: Counters,
    allocations: Allocations,
}

impl MetricRow {
    /// The scene this row is about.
    pub fn scene(&self) -> &'static str {
        self.scene
    }

    /// The counters behind the line.
    pub fn counters(&self) -> &Counters {
        &self.counters
    }

    /// The seven columns, without the scene's name in front of them.
    ///
    /// [`crate::scenes::report`] puts §21's own numbering and wording there instead, so that a scene
    /// the report can only name lines up under the same headings as one it can measure.
    pub fn columns(&self) -> String {
        let micros = self.elapsed.as_secs_f64() * 1e6 / f64::from(self.frames.max(1));
        let word = |c: Counter| match self.counters.get(c).measured() {
            Some(n) => n.to_string(),
            None => "unreachable".to_string(),
        };
        format!(
            "{:>10.2} us  marked={:<11}  writes={:<7}  verbs={:<6}  regions={:<5}  stops={:<5}  \
             allocations={}/{}",
            micros,
            word(Counter::Marked),
            word(Counter::Writes),
            word(Counter::Verbs),
            word(Counter::Regions),
            word(Counter::TabStops),
            self.allocations.total(),
            self.allocations.frames(),
        )
    }

    /// The line, in §21's column order, with the scene's name in front of it.
    pub fn line(&self) -> String {
        format!("{:<52}  {}", self.scene, self.columns())
    }
}

/// The heading the metric rows line up under.
pub fn metric_heading() -> String {
    format!(
        "{:<52}  {:>13}  {:<18}  {:<14}  {:<12}  {:<13}  {:<11}  {}",
        "scene", "us/frame", "marked", "writes", "verbs", "regions", "stops", "allocations"
    )
}

/// **Play a scene: one frame a step, into a surface that is not cleared between them.**
///
/// The surface persisting across steps is the mechanism, not an optimisation. The stale tail is a
/// defect *of the second frame given the first*, and a runner that started each step from a blank
/// surface would score it clean — the same way `Gallery::resize` scores clean by allocating a new
/// `Surface`, which is §21's own reason for not banking that gate.
///
/// # Panics
///
/// Panics on an empty step list, and on a step whose rectangle is not the surface's. A scene played
/// over no steps draws nothing and compares equal to anything, which is
/// [`crate::obligations::Verdict::of`]'s vacuity refusal arriving one file over. A step that changes
/// its own rectangle is a resize, and [`at_two_sizes`] is where that question is asked.
pub fn play(paint: Painter, steps: &[Fixture]) -> Run {
    play_at(Density::default(), paint, steps)
}

/// **A headless driver at a chosen density**, which is the one thing `Driver::headless` does not
/// take an argument for.
///
/// The palette, the glyph repertoire and the resolved colour tier are the driver's own answers, read
/// off the theme it built and handed to the one that replaces it. **The tier is never named** —
/// `ColorDepth` is `reachable_as: None` in `crates/vitui-runtime/src/line.rs`, so a crate whose
/// dependency list is `vitui-runtime` and nothing else can hold one and cannot write one down.
pub fn driver_at(w: u16, h: u16, density: Density) -> Driver {
    let mut driver = Driver::headless(w, h).expect("a sink cannot fail to attach");
    let base = *driver.env().theme();
    driver
        .set_theme(Theme::authored(&CATPPUCCIN_MOCHA, base.glyphs(), density).resolve(base.tier()));
    driver
}

/// **[`play`], at a chosen density.**
///
/// Density is theme data and it changes rectangles (spec §3), so a scene measured at one density is
/// a measurement of one screen and not of the construction. `Driver::headless` builds
/// `Theme::default()`, which is `Density::default()`; this rebuilds the same palette at the density
/// asked for and keeps everything else — the glyph repertoire and the resolved colour tier — as the
/// driver already answered them. `Density::default()` therefore reaches exactly the theme `play`
/// used to build, which `tests::the_default_density_rebuilds_the_drivers_own_theme` asserts.
///
/// # Panics
///
/// [`play`]'s two, unchanged.
pub fn play_at(density: Density, paint: Painter, steps: &[Fixture]) -> Run {
    assert!(
        !steps.is_empty(),
        "a scene played over no steps draws nothing and compares equal to anything"
    );
    let (w, h) = steps[0].size();
    for step in steps {
        assert_eq!(
            step.size(),
            (w, h),
            "the rectangle does not move within a play. A step that resizes it is `at_two_sizes`"
        );
    }

    let mut driver = driver_at(w, h, density);
    let mut pen = Pen::new(w, h);
    let started = Instant::now();
    for step in steps {
        driver.frame(|cx| paint(&mut pen, cx, step));
    }
    let elapsed = started.elapsed();
    Run {
        pen,
        driver,
        elapsed,
        frames: steps.len() as u32,
    }
}

/// **Render a scene two ways and compare the surfaces cell for cell.**
///
/// The primitive. Returns *n cells over m rows*, which is the form every defect on the map was
/// legible in.
pub fn compare(a: Painter, b: Painter, steps: &[Fixture]) -> Diff {
    compare_at(Density::default(), a, b, steps)
}

/// **[`compare`], at a chosen density.**
///
/// Density changes rectangles (spec §3), so two arms compared at two densities are not being
/// compared at all. One argument, threaded through both plays, is what stops that being possible to
/// write by accident.
pub fn compare_at(density: Density, a: Painter, b: Painter, steps: &[Fixture]) -> Diff {
    play_at(density, a, steps)
        .canvas()
        .diff(play_at(density, b, steps).canvas())
}

/// The result of rendering one scene at two sizes.
///
/// Two numbers and not one: *how many rows were compared* is what stops the answer being vacuous.
/// **A build that merely drew less would score 0 differing rows by drawing nothing**, and three
/// separate defects on the map did exactly that.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SizePair {
    /// How many rows exist at both sizes and were therefore compared.
    pub rows_compared: usize,
    /// How many of them carry different content.
    pub rows_differing: usize,
    /// Rect written at the wide size.
    pub wide_written: usize,
    /// Rect written at the narrow size.
    pub narrow_written: usize,
}

impl SizePair {
    /// Fail unless the content survived the narrowing **and both sizes drew something**.
    ///
    /// # Panics
    ///
    /// Panics when a row's content differs, and panics when either size wrote no cells at all. The
    /// second half is the one that matters: an arm that draws nothing agrees with itself about every
    /// row it did not draw.
    #[track_caller]
    pub fn assert_content_equal(self, scene: &str) {
        assert!(
            self.wide_written > 0 && self.narrow_written > 0,
            "{scene}: a size wrote nothing — {} cells wide, {} narrow. A build that merely drew \
             less scores 0 differing rows",
            self.wide_written,
            self.narrow_written
        );
        assert_eq!(
            self.rows_differing, 0,
            "{scene}: {} of {} rows differ between the two sizes",
            self.rows_differing, self.rows_compared
        );
    }
}

/// **Render a scene at two sizes and compare the content**, which is the axis §13 is red on.
///
/// Not a cell-for-cell equality: a row drawn at 300 columns and the same row at 60 differ in every
/// cell past column 59 and carry the same content. What is compared is [`Canvas::row_text`] at the
/// narrow size against the wide one truncated to the narrow width, over the rows both sizes have —
/// so a construction that changes with the width is a differing row and a rectangle that changes
/// with the width is not.
///
/// # Panics
///
/// Panics when the narrow size is not narrower than the wide one in both dimensions or the same.
pub fn at_two_sizes(
    fx: &Fixture,
    wide: (u16, u16),
    narrow: (u16, u16),
    paint: Painter,
) -> SizePair {
    assert!(
        narrow.0 <= wide.0 && narrow.1 <= wide.1,
        "the narrow size is not narrower"
    );
    let big = play(paint, &[fx.resized(wide.0, wide.1)]);
    let small = play(paint, &[fx.resized(narrow.0, narrow.1)]);
    let big_rows = big.canvas().content();
    let small_rows = small.canvas().content();

    let rows_compared = usize::from(narrow.1);
    let mut rows_differing = 0usize;
    for y in 0..rows_compared {
        let expected = text::truncate(&big_rows[y], narrow.0).trim_end();
        if small_rows[y].as_str() != expected {
            rows_differing += 1;
        }
    }
    SizePair {
        rows_compared,
        rows_differing,
        wide_written: big.canvas().written(),
        narrow_written: small.canvas().written(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The two correct arms agree cell for cell**, which is what makes every disagreement below
    /// evidence about the painter rather than about the runner.
    ///
    /// §21's scene 2, in its own words: *the same screen drawn naive and correct* — 0 of 24 000 cells
    /// apart. This is the same equality over a fixture small enough for a per-cell oracle.
    #[test]
    fn the_same_screen_drawn_both_ways_is_equal_cell_for_cell() {
        let fx = Fixture::lines(40, 12, 200).scrolled_to(7);
        let diff = compare(reference, rows_at_a_time, std::slice::from_ref(&fx));
        assert_eq!((diff.cells, diff.rows), (0, 0), "{diff}");
        assert!(diff.clean());

        // And it is not clean by drawing nothing: every cell of the rectangle is written.
        let run = play(reference, &[fx]);
        assert_eq!(run.canvas().written(), 40 * 12);

        // **And across a shrink**, which is the sequence a single frame cannot ask about. This is
        // the arm that fails when the fast path stops at the end of its content: the equality is
        // over `rows_at_a_time` itself, so a stale tail injected into the shipped painter is caught
        // here rather than only by the fixture in `defective`.
        let full = Fixture::lines(40, 80, 200);
        let after = compare(
            reference,
            rows_at_a_time,
            &[full.clone(), full.shrunk_to(9)],
        );
        assert!(
            after.clean(),
            "the fast path left something standing after the content shrank: {after}"
        );
    }

    /// **The inverted scroll sign: it draws nothing, and it is faster.**
    ///
    /// The runner's report is *n cells over m rows*. Both halves are asserted, because the count of
    /// rows is what says the failure is the whole screen and not a band.
    #[test]
    fn the_runner_catches_an_inverted_scroll_sign() {
        let fx = Fixture::lines(40, 12, 200).scrolled_to(60);
        let diff = compare(
            reference,
            defective::inverted_scroll,
            std::slice::from_ref(&fx),
        );
        assert_eq!(
            (diff.cells, diff.rows),
            (440, 11),
            "eleven of twelve rows wrong: {diff}"
        );
        assert_eq!(diff.first, Some((0, 1)));

        // **And every counter approves of it**, which is why the equality is the only detector.
        let correct = play(rows_at_a_time, std::slice::from_ref(&fx));
        let broken = play(defective::inverted_scroll, &[fx]);
        assert_eq!(correct.tally().verbs(), broken.tally().verbs());
        assert!(
            broken.tally().distinct() <= correct.tally().distinct(),
            "the defective build touches no more cells than the correct one"
        );
    }

    /// **The stale tail: 71 of 80 rows, and it needs two frames to be visible at all.**
    ///
    /// §21's spelling of the shrink axis: *content shrinking inside a rectangle that does not move*.
    /// The rectangle here is 40x80 on both frames and only the content changes, which is the
    /// distinction the terminal-resize spelling cannot make.
    #[test]
    fn the_runner_catches_a_stale_tail_after_content_shrinks_inside_a_rectangle_that_does_not_move()
    {
        let full = Fixture::lines(40, 80, 200);
        let shrunk = full.shrunk_to(9);
        assert_eq!(full.size(), shrunk.size(), "the rectangle does not move");

        let diff = compare(
            reference,
            defective::stale_tail,
            &[full.clone(), shrunk.clone()],
        );
        assert_eq!(diff.rows, 71, "71 of 80 rows, which is §21's own number");
        assert_eq!(diff.cells, 71 * 40);
        assert_eq!(diff.first, Some((0, 9)));

        // **The defective build looks healthier**, and this is the half that makes the equality
        // load-bearing. Measured on the frame the defect is in — the shrink — because the frame
        // before it is identical in both builds and averaging the two is what hides the ratio:
        // 3 200 writes against 360, nearly nine to one, and the build that leaves 71 rows of the
        // previous screen standing is the one that looks cheap.
        let correct = play(rows_at_a_time, std::slice::from_ref(&shrunk));
        let broken = play(defective::stale_tail, std::slice::from_ref(&shrunk));
        assert_eq!(
            (correct.tally().writes(), broken.tally().writes()),
            (3200, 360)
        );
        assert!(broken.tally().writes() * 8 < correct.tally().writes());

        // And the resize spelling, which passes — the reason §21 refuses to bank it. A fresh
        // rectangle has nowhere for the residue to survive, so the same painter is clean.
        let resized = compare(
            reference,
            defective::stale_tail,
            &[Fixture::lines(40, 9, 9).resized(40, 9)],
        );
        assert!(
            resized.clean(),
            "the resize spelling scores the same defect clean: {resized}"
        );
    }

    /// **Twenty wheel clicks move the offset 0 against 16.**
    ///
    /// The gesture is a sequence of steps, which is why [`play`] takes one.
    #[test]
    fn the_runner_catches_twenty_wheel_clicks_that_move_nothing() {
        let base = Fixture::lines(40, 12, 400);
        let clicks: Vec<Fixture> = (0..=20).map(|c| base.scrolled_to(c * 3)).collect();

        let diff = compare(reference, defective::unmoved_wheel, &clicks);
        assert_eq!(
            (diff.cells, diff.rows),
            (480, 12),
            "twenty clicks later the screen is still the one at offset 0: {diff}"
        );

        // The correct painter over the same twenty clicks is clean, which is the other direction.
        assert!(compare(reference, rows_at_a_time, &clicks).clean());
    }

    /// **Green at 300x80 and red at 60x20** — the narrow axis, which is a content equality across
    /// two sizes rather than a cell equality at one.
    #[test]
    fn a_construction_that_changes_with_the_width_is_green_wide_and_red_narrow() {
        let fx = Fixture::lines(60, 20, 200);

        // The correct painter: the content survives the narrowing.
        at_two_sizes(&fx, (60, 20), (16, 8), rows_at_a_time).assert_content_equal("rows_at_a_time");

        // The defective one is identical at the wide size and drops two columns at the narrow one.
        let wide = compare(
            reference,
            defective::narrow_drops_a_column,
            &[fx.resized(60, 20)],
        );
        assert!(wide.clean(), "green at the wide size: {wide}");
        let narrow = compare(
            reference,
            defective::narrow_drops_a_column,
            &[fx.resized(16, 8)],
        );
        assert_eq!(
            (narrow.cells, narrow.rows),
            (16, 8),
            "red at 16x8: {narrow}"
        );

        let pair = at_two_sizes(&fx, (60, 20), (16, 8), defective::narrow_drops_a_column);
        assert_eq!(pair.rows_differing, 8);
        assert_eq!(pair.rows_compared, 8);
    }

    /// **A size that wrote nothing fails loudly**, which is what keeps the two-size equality from
    /// being satisfied by drawing less.
    #[test]
    #[should_panic(expected = "a size wrote nothing")]
    fn a_two_size_comparison_over_a_blank_arm_is_refused() {
        fn draws_nothing(_: &mut Pen, _: &mut Ctx<'_, '_>, _: &Fixture) {}
        at_two_sizes(
            &Fixture::lines(40, 10, 50),
            (40, 10),
            (20, 5),
            draws_nothing,
        )
        .assert_content_equal("a painter that draws nothing");
    }

    /// **A scene played over no steps is refused in the constructor.** The vacuity rule, arriving
    /// where the vacuous answer is *the two arms agree everywhere*.
    #[test]
    #[should_panic(expected = "compares equal to anything")]
    fn a_scene_played_over_no_steps_is_refused() {
        let _ = play(reference, &[]);
    }

    /// **A never-written cell is not a blank**, which is the distinction the stale tail lives in.
    #[test]
    fn an_untouched_cell_is_its_own_value_and_not_a_space() {
        let fx = Fixture::of(6, 2, vec!["ab".to_string()]);
        let run = play(rows_at_a_time, &[fx]);
        let canvas = run.canvas();
        assert_eq!(canvas.get(0, 0).map(|c| c.cluster.as_str()), Some("a"));
        assert_eq!(
            canvas.get(3, 0).map(|c| c.cluster.as_str()),
            Some(" "),
            "the row was padded, so this cell was written with a space"
        );
        assert_eq!(canvas.written(), 12);

        // A painter that stops early leaves the rest of the surface `None`, and `None != Some(" ")`.
        let short = play(
            defective::stale_tail,
            &[Fixture::of(6, 2, vec!["ab".into()])],
        );
        assert_eq!(short.canvas().get(0, 1), None);
        assert_eq!(short.canvas().written(), 6);
    }

    /// **The metric row prints `marked=unreachable` and never `marked=0`.**
    ///
    /// §21's first refinement, as a string: *a threshold on the wrong side of the question is not a
    /// weak gate, it is a green one*. Two of the four axes marked **less** than the correct build,
    /// so a `0` in this column beside a healthy microsecond figure is the defective build's own
    /// self-portrait.
    #[test]
    fn the_metric_row_says_marked_is_unreachable_rather_than_zero() {
        let run = play(rows_at_a_time, &[Fixture::lines(40, 10, 100)]);
        let row = run.row("a scrolled collection", Allocations::over(1, 0));
        let line = row.line();
        assert!(line.contains("marked=unreachable"), "{line}");
        assert!(!line.contains("marked=0"), "{line}");
        for column in [
            "us",
            "marked=",
            "writes=",
            "verbs=",
            "regions=",
            "stops=",
            "allocations=",
        ] {
            assert!(
                line.contains(column),
                "the row is missing `{column}`: {line}"
            );
        }
        assert_eq!(row.counters().reachable(), 8);
        assert_eq!(
            metric_heading().split_whitespace().count(),
            8,
            "one heading a column"
        );
    }

    /// **A wide cluster occupies two cells and the second is not a copy of the first.**
    ///
    /// The engine's own arrangement. A continuation modelled as a copy would let a build that lost
    /// the head and kept the tail compare equal on the tail.
    #[test]
    fn a_wide_cluster_records_a_head_and_a_continuation() {
        let found = clusters("a\u{4e00}b", 4);
        assert_eq!(
            found,
            vec![(0, 1, "a"), (1, 2, "\u{4e00}"), (3, 1, "b")],
            "the middle cluster is two columns wide"
        );

        let mut driver = Driver::headless(6, 1).expect("a sink cannot fail to attach");
        let mut pen = Pen::new(6, 1);
        driver.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            pen.text(cx, 0, 0, "a\u{4e00}b", body);
        });
        assert_eq!(
            pen.canvas().get(1, 0).map(|c| c.cluster.as_str()),
            Some("\u{4e00}")
        );
        assert_eq!(
            pen.canvas().get(2, 0).map(|c| c.cluster.as_str()),
            Some(""),
            "the continuation half"
        );
        assert_eq!(
            pen.canvas().get(3, 0).map(|c| c.cluster.as_str()),
            Some("b")
        );
    }

    /// **A play does not clear the surface between steps, and a step may not move the rectangle.**
    ///
    /// Both halves of the mechanism the shrink axis rests on, asserted where they are decided rather
    /// than inferred from the test that uses them.
    #[test]
    fn a_play_keeps_one_surface_across_its_steps_and_refuses_a_moving_rectangle() {
        let fx = Fixture::lines(10, 4, 40);
        let run = play(defective::stale_tail, &[fx.clone(), fx.shrunk_to(1)]);
        assert_eq!(
            run.canvas().written(),
            40,
            "the first frame's rows survived the second"
        );
        assert_eq!(run.frames(), 2);
        assert!(run.elapsed() > Duration::ZERO);

        let moved = std::panic::catch_unwind(|| play(reference, &[fx.clone(), fx.resized(9, 4)]));
        assert!(moved.is_err(), "a step that resizes is refused");
    }
}
