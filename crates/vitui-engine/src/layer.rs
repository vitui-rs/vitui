//! The layer stack, and compositing it into a frame.
//!
//! A layer is a rectangle positioned in the stack with a z-order. A window, a popup, a shadow and a
//! modal dim are all layers; nothing else is. Rows of a virtualised tree are not layers, which is
//! why `n` here is tens and the stack is a sorted `Vec` rather than a tree or a skip list
//! (spec §5).
//!
//! A tree was rejected because a `z` local to a parent *is* a scene, and the scene lives above the
//! engine (ADR 0002). A skip list was rejected for three reasons and each is worth keeping: **n is
//! tens**; **the dominant operation is an ordered traversal of the whole stack every frame**, which
//! is what a contiguous `Vec` is best at and a skip list worst at; and a skip list's real modern
//! advantage is lock-free concurrent ordered access, and there is no concurrency here — plus a node
//! allocation per insert and an RNG for level assignment, which is non-determinism where this
//! project wants a property a test can assert. If a `Vec` ever stops being enough the answer is
//! `BTreeMap<(z, seq), Layer>`.
//!
//! # Scope
//!
//! The whole of spec §12's `LayerStack` is here (ticket 10), and the bottom-up composite of the
//! damaged runs with the wide-glyph corruption bug closed at its third and last edge (ticket 11).
//!
//! What is **not** here is the operator layer's compositing:
//! [`add_operator`](LayerStack::add_operator) places one, orders it and lets it be moved and
//! removed, and ticket 12 is what makes a `Mix` reach a cell.

use crate::cell::{Cell, GraphemeId};
use crate::damage::Run;
use crate::exts::{ExtStyle, LinkId};
use crate::geom::Rect;
use crate::style::{Color, Style};
use crate::surface::Surface;
use crate::tables::Tables;
use crate::view::View;

/// A layer's identity, stable for the layer's life.
///
/// Opaque: no public field and no way to construct one from a number. A caller must be able to say
/// *this layer again* without being able to say *entry 7*.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct LayerId(u32);

/// The one operator: a colour, and how far toward it what is already there is moved.
///
/// **Darken is `Mix` toward black, lifting is `Mix` toward white, tint is `Mix` toward anything, and
/// a fade is `amount` moving across frames.** One mechanism instead of three blend modes, and the
/// three-item list it replaced (`Replace`, `Darken`, `Blend`) collapsed for a reason worth keeping:
/// `Replace` is not a blend mode, it is what a content layer does, and **alpha-over is rejected**
/// because a terminal cell has no alpha — blending two layers' glyphs is not a thing, one of them
/// has to win, and a compositor has no basis for choosing (spec §5).
///
/// The fields are private and [`new`](Mix::new) clamps, because `0..=256` is an invariant the
/// compositing arithmetic rests on rather than a suggestion — the same shape [`Color`] already
/// takes. `amount == 0` is the identity, and the identity never reaches a cell.
///
/// **Ticket 12 owns what a `Mix` does.** This type exists here because
/// [`add_operator`](LayerStack::add_operator) needs a signature, and the stack has to be able to
/// order, move and remove an operator layer before there is anything for it to paint.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Mix {
    toward: Color,
    amount: u16,
}

impl Mix {
    /// All the way to `toward`, leaving nothing of what was underneath.
    pub const FULL: u16 = 256;

    /// A mix `amount`/256 of the way toward `toward`, saturating at [`FULL`](Mix::FULL).
    pub const fn new(toward: Color, amount: u16) -> Mix {
        Mix {
            toward,
            amount: if amount > Mix::FULL {
                Mix::FULL
            } else {
                amount
            },
        }
    }

    /// The colour this operator moves cells toward.
    pub const fn toward(self) -> Color {
        self.toward
    }

    /// How far toward it, out of [`FULL`](Mix::FULL).
    pub const fn amount(self) -> u16 {
        self.amount
    }

    /// Whether this operator changes nothing, and is therefore skipped entirely.
    pub const fn is_identity(self) -> bool {
        self.amount == 0
    }
}

/// One layer, as the reference compositor and the gates need to see it.
///
/// `cfg(test)` because spec §12's public surface names none of it: a caller cannot read back what
/// is already on screen (ADR 0023), and an oracle is not an exception to that — it is simply
/// inside the crate.
#[cfg(test)]
pub(crate) struct LayerRef<'a> {
    pub(crate) z: i32,
    /// The tie-break among equal `z`. Handed out so the reference compositor can sort for itself.
    pub(crate) seq: u32,
    pub(crate) rect: Rect,
    pub(crate) opaque: bool,
    pub(crate) surface: &'a Surface,
}

/// What a layer is made of: cells, or a transformation of whatever is already there.
///
/// Not public, and spec §12 is where that is decided rather than here — its compositing section
/// names `LayerId` and `Mix` and no `LayerKind`. A public `Content { surface: Surface }` would also
/// be a door onto a layer's cells, which is the one ADR 0023 closes.
///
/// **The surface is boxed**, and that is a decision about the dominant operation rather than a
/// habit. §5 chose a contiguous `Vec` because *the dominant operation is an ordered traversal of the
/// whole stack every frame*, and that argument is about bytes touched per layer. A `Surface`
/// carries three tables and two `Vec`s; inline, every `Layer` would be its own cache line and a
/// scan of two hundred would be two hundred misses. Boxed, a `Layer` is forty bytes — asserted
/// below, because the number is the whole reason — and the scan reads `z`, `rect` and a tag, and
/// follows the pointer only for the layers a run actually touches.
///
/// Measured on the shipped stack (`examples/budget.rs`): the full ordered scan is **19.6 ns at
/// n = 20 and 205 ns at n = 200** — about one nanosecond a layer, and linear. §5's prototype was
/// 5.05 and 59.3 ns, a quarter of a nanosecond a layer, and forty bytes is why: 1.6 layers to a
/// cache line rather than the four a sixteen-byte record would give. It is not chased, because the
/// whole scan at n = 200 is 0.2% of a 100 µs frame and the fix would be a parallel array of the hot
/// fields — a second structure to keep in step, for a fifth of a percent.
enum Kind {
    Content { surface: Box<Surface>, opaque: bool },
    Operator(Mix),
}

struct Layer {
    id: LayerId,
    z: i32,
    /// The tie-break among equal `z`, never mutated — so raising a layer and dropping it back
    /// restores the exact original order.
    seq: u32,
    rect: Rect,
    kind: Kind,
}

impl Layer {
    /// The cells this layer paints, or `None` for an operator layer, which has none.
    fn surface(&self) -> Option<&Surface> {
        match &self.kind {
            Kind::Content { surface, .. } => Some(surface),
            Kind::Operator(_) => None,
        }
    }

    /// Whether this layer can change a composited cell at all.
    ///
    /// False for an operator whose `Mix` is the identity, which is §5's rule that `amount == 0` is
    /// skipped entirely: adding, moving and removing one damages nothing, because there is nothing
    /// it could have painted for the frame to have to undo.
    fn paints(&self) -> bool {
        match &self.kind {
            Kind::Content { .. } => true,
            Kind::Operator(mix) => !mix.is_identity(),
        }
    }

    /// Whether this layer covers every cell of `run`, opaquely — in which case nothing below it is
    /// visible there and the composite can start from it.
    fn floors(&self, run: Run) -> bool {
        let Kind::Content { opaque: true, .. } = self.kind else {
            return false;
        };
        let y = run.y as i32;
        y >= self.rect.y
            && y < self.rect.bottom()
            && self.rect.x <= run.lo as i32
            && self.rect.right() > run.hi as i32
    }
}

/// The layers, in bottom-to-top order.
///
/// There is no public constructor and no `Default`: spec §12 reaches the stack only through
/// [`Screen::layers`](crate::Screen::layers), and a derived `Default` would be a second door.
pub struct LayerStack {
    /// Sorted by `(z, seq)`. The dominant operation is an ordered traversal of the whole stack
    /// every frame, which is what a contiguous `Vec` is best at.
    layers: Vec<Layer>,
    /// **The one handle space** every surface in this stack speaks (spec §3, ADR 0011 as amended by
    /// architecture ticket 19): the grapheme interner, the extended-style table and the link table.
    /// They live here rather than in a `Surface` because that is what keeps compositing a
    /// `copy_from_slice`: with one handle space there is nothing to translate on the way across,
    /// and the per-surface arm measured 4.9x on a realistic layer and 46x on a hostile one.
    /// [`add_content_with`](LayerStack::add_content_with) is what renumbers a donated surface into
    /// them.
    tables: Tables,
    /// Rectangles of the frame that **no layer's own damage can speak for**.
    ///
    /// A layer's bitset lives in its surface, so removing a layer, moving it or reordering it
    /// damages an area that either belongs to no layer at all or belongs to layers that have not
    /// changed. Those rectangles are collected here and unioned into the frame by
    /// [`take_damage_into`](LayerStack::take_damage_into), which drains them: an exposure is a
    /// fact about one frame, not a standing property of the stack.
    ///
    /// A `Vec` that is drained rather than freed, because the steady state has no topology change
    /// in it and must allocate nothing (`tests/alloc.rs`).
    exposed: Vec<Rect>,
    next_id: u32,
    next_seq: u32,
}

impl LayerStack {
    pub(crate) fn new() -> LayerStack {
        LayerStack {
            layers: Vec::new(),
            tables: Tables::new(),
            exposed: Vec::new(),
            next_id: 0,
            next_seq: 0,
        }
    }

    /// Add a content layer at `z`, covering `rect`, and allocate its surface.
    ///
    /// `opaque` says whether every cell of the surface paints. A caller who draws only a border
    /// into an opaque layer gets a rectangle of opaque spaces that erases what is underneath —
    /// rounded corners, a tooltip with a transparent gutter and any overlay over a chart all want
    /// `opaque: false`, and pay 4.1x for the skip test (spec §5).
    ///
    /// The new layer damages its whole rectangle, because nothing beneath it has been asked to
    /// repaint what it now covers.
    pub fn add_content(&mut self, z: i32, rect: Rect, opaque: bool) -> LayerId {
        let surface = Surface::new_filled(rect.w, rect.h, ground(opaque));
        self.place(z, rect, opaque, surface)
    }

    /// Add a content layer at `z`, covering `rect`, painting cells that are already drawn.
    ///
    /// The worker's door, and the reason [`Surface`] is [`Send`]: drawing a heavy off-screen
    /// surface on another thread and donating it is legitimate traffic, and ownership transfer is
    /// the mechanism by which the app thread never waits.
    ///
    /// # The donated surface's handles are renumbered, once, here
    ///
    /// A `Surface` drawn through [`Surface::root`](Surface::root) interns into a table of its own,
    /// because that door has no engine to reach through (spec §3, architecture ticket 19). Its
    /// grapheme handles, its extended-style handles and the link ids **inside** those are therefore
    /// meaningless in this stack's handle space, and are rewritten here so that
    /// *every surface in a layer stack speaks that stack's handle space* holds by renumbering where
    /// it holds by construction for [`add_content`](LayerStack::add_content).
    ///
    /// **Nearly every donation skips the walk**: Latin, CJK, box drawing and every single-scalar
    /// emoji are their own handles and touch no table, and a cell that is neither hyperlinked nor
    /// coloured-underlined carries no extended style. A surface whose three tables are all empty
    /// moves in as it is.
    ///
    /// The renumbering marks no damage — a new layer already damages its whole rectangle — and it
    /// happens at a scene topology change, where allocation is permitted, never inside a frame.
    ///
    /// # A size mismatch is intersected, not rejected
    ///
    /// The donated surface is what there is to paint, so the layer covers `rect` intersected with
    /// it: a surface smaller than `rect` paints its own cells and no more, and one larger is
    /// clipped. That is §5's rule for a layer that hangs off an edge, applied to the other way a
    /// rectangle and a grid can disagree.
    pub fn add_content_with(
        &mut self,
        z: i32,
        rect: Rect,
        opaque: bool,
        mut surface: Surface,
    ) -> LayerId {
        self.renumber(&mut surface);
        self.place(z, rect, opaque, surface)
    }

    /// Add an operator layer at `z`, covering `rect`.
    ///
    /// An operator has no cells: it is a rectangle and a transformation of whatever is already
    /// there. A shadow is one of these at a `z` just below its window, and the window occludes its
    /// own shadow by painting over it — which is why there is no region arithmetic anywhere in this
    /// file, no rectangle-minus-rectangle and no L-shapes. Modal dimming is the same trick with the
    /// scrim beneath the modal.
    ///
    /// An identity mix and a zero-area rectangle both mark no damage, because neither can change a
    /// cell.
    ///
    /// **Ticket 12 is what makes this paint.** Until then an operator layer is placed, ordered,
    /// moved and removed like any other and composites to nothing, so the picture is the one the
    /// content layers make.
    pub fn add_operator(&mut self, z: i32, rect: Rect, mix: Mix) -> LayerId {
        let id = self.insert(z, rect, Kind::Operator(mix));
        if !mix.is_identity() {
            self.expose(rect);
        }
        id
    }

    /// Remove a layer. `false` when this stack has no layer with that id.
    ///
    /// What was underneath is exposed and repainted; the layer's surface, and the cells in it, are
    /// dropped. The handle-table entries those cells named are **not** reclaimed here — that is the
    /// mark-and-compact sweep's job, and it runs where allocation is permitted rather than inside a
    /// verb (spec §3, ticket 08).
    pub fn remove(&mut self, id: LayerId) -> bool {
        let Some(at) = self.find(id) else {
            return false;
        };
        let gone = self.layers.remove(at);
        if gone.paints() {
            self.expose(gone.rect);
        }
        true
    }

    /// Move a layer to a new `z`, keeping its `seq`. `false` when this stack has no layer with
    /// that id.
    ///
    /// **`seq` is never mutated**, which is the whole of the tie-break: raising a layer to the top
    /// and dropping it back to the `z` it came from restores the exact original order, however many
    /// layers share that `z`.
    pub fn set_z(&mut self, id: LayerId, z: i32) -> bool {
        let Some(at) = self.find(id) else {
            return false;
        };
        if self.layers[at].z == z {
            return true;
        }
        let mut layer = self.layers.remove(at);
        layer.z = z;
        let (rect, paints) = (layer.rect, layer.paints());
        self.sorted_insert(layer);
        if paints {
            self.expose(rect);
        }
        true
    }

    /// Move or resize a layer. `false` when this stack has no layer with that id.
    ///
    /// A move keeps the layer's cells. **A resize does not**: the surface is reallocated at the new
    /// size and every cell of it goes back to the layer's ground, so the caller redraws. Carrying
    /// the old cells across would leave a window that has changed shape holding a picture drawn for
    /// the shape it used to be, which is worse than a blank one — and the runtime brings a
    /// rectangle and a draw for every layer every frame anyway (spec §12).
    ///
    /// The rectangle the layer came from is exposed, and the one it goes to is damaged.
    pub fn set_rect(&mut self, id: LayerId, rect: Rect) -> bool {
        let Some(at) = self.find(id) else {
            return false;
        };
        let old = self.layers[at].rect;
        let paints = self.layers[at].paints();
        match &mut self.layers[at].kind {
            Kind::Content { surface, opaque } => {
                // Against the **rectangle** the layer already had, never against the surface's own
                // size. A donated surface may be larger than the rectangle it was fitted to, and
                // comparing against the surface would reallocate on a pure move — throwing away
                // the cells this verb promises to keep, for a layer that did not change shape.
                if (rect.w, rect.h) != (old.w, old.h) {
                    **surface = Surface::new_filled(rect.w, rect.h, ground(*opaque));
                }
                surface.damage_mut().mark_all();
                self.layers[at].rect = fit(rect, surface.size());
            }
            Kind::Operator(_) => {
                self.layers[at].rect = rect;
                if paints {
                    // A content layer's new rectangle is damaged by its surface's own bitset; an
                    // operator has no surface to carry one.
                    self.expose(rect);
                }
            }
        }
        if paints {
            self.expose(old);
        }
        true
    }

    /// A view of a layer's cells, drawing into this stack's handle space. The only way to reach
    /// them.
    ///
    /// `None` for an id this stack never minted, and for an **operator** layer, which has no cells
    /// to draw into.
    pub fn view(&mut self, id: LayerId) -> Option<View<'_>> {
        let tables = &mut self.tables;
        self.layers
            .iter_mut()
            .find(|l| l.id == id)
            .and_then(|l| match &mut l.kind {
                Kind::Content { surface, .. } => Some(surface.draw(tables)),
                Kind::Operator(_) => None,
            })
    }

    /// The topmost layer covering `(x, y)`, or `None` where there is none.
    ///
    /// A plain reverse linear scan. §5 measured 13.5 ns at n = 20 and 81.0 ns at n = 200, and a
    /// spatial index is warranted where n is thousands — there n is *widgets*, which is a runtime
    /// concept two orders of magnitude away from a stack of windows, popups, shadows and dims.
    /// The division of labour is the one `visible_rows` already established: **the engine owns the
    /// query, the runtime owns the index.**
    ///
    /// **Operator layers are not hittable.** A shadow is not a thing you click, so a click near a
    /// modal's edge cannot resolve to it.
    ///
    /// The answer is about rectangles and never about cells. A non-opaque layer is hit anywhere
    /// inside its rectangle, `EMPTY` cells included: reading cells here would make the answer
    /// depend on what a component happened to draw, which is the runtime's business and not the
    /// engine's — and **the query must return a layer and never a widget**, which is the door
    /// ADR 0002 exists to keep shut.
    pub fn topmost_at(&self, x: i32, y: i32) -> Option<LayerId> {
        self.layers
            .iter()
            .rev()
            .find(|l| {
                matches!(l.kind, Kind::Content { .. })
                    && !l.rect.is_empty()
                    && x >= l.rect.x
                    && x < l.rect.right()
                    && y >= l.rect.y
                    && y < l.rect.bottom()
            })
            .map(|l| l.id)
    }

    /// The handle space every surface in this stack speaks.
    pub(crate) fn tables(&self) -> &Tables {
        &self.tables
    }

    pub(crate) fn tables_mut(&mut self) -> &mut Tables {
        &mut self.tables
    }

    /// How many layers are in the stack.
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// Whether the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// Place a content layer, fitted to its surface, damaging the whole of it.
    fn place(&mut self, z: i32, rect: Rect, opaque: bool, mut surface: Surface) -> LayerId {
        surface.damage_mut().mark_all();
        let rect = fit(rect, surface.size());
        self.insert(
            z,
            rect,
            Kind::Content {
                surface: Box::new(surface),
                opaque,
            },
        )
    }

    /// Mint an id and a `seq`, and put the layer where `(z, seq)` says it goes.
    fn insert(&mut self, z: i32, rect: Rect, kind: Kind) -> LayerId {
        let id = LayerId(self.next_id);
        self.next_id += 1;
        let seq = self.next_seq;
        self.next_seq += 1;
        self.sorted_insert(Layer {
            id,
            z,
            seq,
            rect,
            kind,
        });
        id
    }

    fn sorted_insert(&mut self, layer: Layer) {
        let at = self
            .layers
            .partition_point(|l| (l.z, l.seq) < (layer.z, layer.seq));
        self.layers.insert(at, layer);
    }

    fn find(&self, id: LayerId) -> Option<usize> {
        self.layers.iter().position(|l| l.id == id)
    }

    /// Record a rectangle of the frame that has to be repainted although no layer's own bitset
    /// says so. Empty rectangles are dropped rather than carried.
    fn expose(&mut self, rect: Rect) {
        if !rect.is_empty() {
            self.exposed.push(rect);
        }
    }

    /// Rewrite a donated surface's handles into this stack's handle space.
    ///
    /// Two passes and not one: the tables are walked first and the cells second, so a surface of a
    /// thousand accented cells pays one re-intern per **distinct** cluster and an array lookup per
    /// cell, rather than a hash of the cluster's bytes per cell.
    ///
    /// **Links before extended styles.** An extended-style entry names a `LinkId`, and both tables
    /// deduplicate — so re-interning an entry that still carries the donor's id would dedup against
    /// the wrong key and two different links could collapse into one.
    fn renumber(&mut self, surface: &mut Surface) {
        // The skip that makes this free for nearly every donation. An empty set of tables means no
        // cell in the surface carries a handle that means anything different here.
        if surface.tables().is_empty() {
            return;
        }

        let donor = surface.tables();

        let mut links: Vec<LinkId> = Vec::with_capacity(donor.links.entries().len());
        for uri in donor.links.entries() {
            links.push(self.tables.links.mint(uri));
        }

        let mut exts: Vec<u32> = Vec::with_capacity(donor.exts.entries().len());
        for entry in donor.exts.entries() {
            let renumbered = ExtStyle {
                link: remapped(&links, entry.link),
                ..entry
            };
            exts.push(self.tables.exts.handle(renumbered));
        }

        let mut graphemes: Vec<GraphemeId> = Vec::with_capacity(donor.interner.entries().len());
        for cluster in donor.interner.entries() {
            // The same bytes that minted a handle over there mint one here, and the width — and so
            // the wide flag — is recomputed from the cluster rather than carried across, which is
            // why the handle a cell arrives with is not the handle it leaves with even in the low
            // bits. A cluster that occupies no column never reaches a cell, so it never reaches a
            // table either (`Interner::handle` answers `None` before interning one).
            graphemes.push(
                self.tables
                    .interner
                    .handle(cluster)
                    .expect("a cluster in a surface's table occupies a column"),
            );
        }

        for y in 0..surface.height() {
            for cell in surface.row_mut(y) {
                if let Some(id) = cell.grapheme.cluster_id()
                    && let Some(g) = graphemes.get(id as usize)
                {
                    cell.grapheme = *g;
                }
                if let Some(h) = cell.style.ext_handle()
                    && let Some(new) = exts.get(h as usize)
                {
                    cell.style = Style::extended(cell.style.attr_word(), *new);
                }
            }
        }

        // The surface speaks this stack's handle space now, and its own table is unreachable
        // through `LayerStack::view`. Keeping it would hold the donor's clusters alive for the life
        // of the layer.
        *surface.tables_mut() = Tables::new();
    }

    /// Move every layer's damage into the frame, in the frame's own coordinates.
    ///
    /// **`take`, not `union`, because the exposures are drained.** A layer's own bitset is left
    /// alone — `clear_damage` is what empties those, after the frame has been written — but
    /// `exposed` is consumed here, so a caller that takes damage and then throws the frame away
    /// loses it. `present` is the only caller that is not a test, and it never does that: it takes,
    /// and either finds nothing damaged (in which case the exposures were off-screen and had
    /// nothing to contribute) or composites and writes.
    ///
    /// A layer's damage is clipped to **its own rectangle** as well as to the frame. A layer cannot
    /// damage what it does not cover, and a donated surface larger than the rectangle it was given
    /// is the case that makes the difference visible.
    pub(crate) fn take_damage_into(&mut self, frame: &mut Surface) {
        debug_assert!(
            self.speaks_one_handle_space(),
            "a surface in this stack still carries a handle space of its own"
        );
        let screen = Rect::new(0, 0, frame.width(), frame.height());
        for layer in &self.layers {
            let Some(surface) = layer.surface() else {
                continue;
            };
            frame.damage_mut().union_translated(
                surface.damage(),
                layer.rect.x,
                layer.rect.y,
                screen.intersect(layer.rect),
            );
        }
        for rect in self.exposed.drain(..) {
            let r = screen.intersect(rect);
            for y in r.y..r.bottom() {
                frame.damage_mut().mark(y, r.x, r.right() - 1);
            }
        }
    }

    /// Paint the damaged runs of `frame`, bottom-up.
    ///
    /// Cost is damaged area times depth, which is the claim a flattened prefix cache would have
    /// existed to deliver — delivered by damage rectangles instead (ADR 0024).
    ///
    /// # The ground, and the floor
    ///
    /// A damaged run is **not** guaranteed to lie inside some layer's rectangle any more: `remove`,
    /// `set_z` and `set_rect` expose areas whose owner has just gone away. So the run is filled
    /// with the frame's ground before the layers paint over it — unless some opaque content layer
    /// covers the whole run, in which case nothing below it is visible and the composite starts
    /// there. The two arms are one decision: the scan for that layer is what says whether a ground
    /// fill is needed, and finding one skips both the fill and every layer beneath it.
    ///
    /// # The wide-glyph hazard, at a layer edge
    ///
    /// A content layer overwrites, so ticket 06's five repair rules move from write time to
    /// composite time (spec §5). Every paint below is a contiguous span, and a span has exactly two
    /// seams — so the fixes are **four O(1) ones per row** rather than a scan: the copied content's
    /// own halves at each end, a wide head left orphaned outside the left edge, and a continuation
    /// left orphaned outside the right.
    ///
    /// The opaque arm needs only its two seams, because the interior of a `copy_from_slice` is a
    /// span of one source row and the composite did not put those cells next to each other. The
    /// `EMPTY` arm needs a seam wherever the **skip** put two cells together: it copies cell by
    /// cell, so a skipped cell can strand a half inside the span it painted, and that costs nothing
    /// because that arm is already per-cell. A boundary between two cells that arm wrote in one
    /// pass is skipped, so both arms take the same position on what a layer hands over.
    ///
    /// A repair never writes outside the span it painted, which is `mend`'s own clamp and is what
    /// keeps a lower layer's cell from being mistaken for a bisection before the layer that owns
    /// the pair has painted its half.
    ///
    /// # One column of slop on each side, and it is not optional
    ///
    /// The painted span is the damaged run **widened by one column at each end**, and the run this
    /// answers with is what actually changed. Both halves of that are load-bearing.
    ///
    /// A repair blanks a half, so the frame stops holding what the layers painted — and the frame is
    /// the only record there is. When the layer that bisected a pair moves off it, the half outside
    /// the exposed rectangle has to come *back*, and nothing has damaged it: the layers did not
    /// change there, only the repair did. Repainting one column past the run is what restores it.
    /// The gate found this rather than the reasoning — twelve rectangles walking one column a frame
    /// across a screen of CJK.
    ///
    /// The column past *that* is only read, never written, and reading it is exactly right: a
    /// repaired frame holds a wide head at `lo - 2` **if and only if** what the layers painted at
    /// `lo - 1` is its continuation, because that is the one case the repair leaves the head alone.
    /// So the neighbour the seam is measured against carries its own answer, and the composite never
    /// has to walk a third column to ask.
    ///
    /// # The repair reaches outside the layer's rectangle
    ///
    /// The prototype ticket 07 measured did the repair and marked damage only over the layer's own
    /// columns, so the terminal kept showing the half that had just been blanked. That is why this
    /// answers with a `Run` rather than with `()`, and why a slop column is reported when — and only
    /// when — it changed: reporting it always would put two more cells on the wire for every run,
    /// which the sparse chart's four hundred of them would feel.
    ///
    /// # What arrives already broken is not this function's to mend
    ///
    /// The seams restored here are the ones **compositing creates**. A layer surface that already
    /// violates the pairing invariant composites into a frame that does too, and there is exactly
    /// one way to build one: [`View::child`](crate::View::child) may not widen its clip (spec §4),
    /// so a pair the clip bisects keeps the half outside it. That is architecture ticket 20's to
    /// decide, and mending it here would hide the case rather than answer it.
    pub(crate) fn composite_run(&self, frame: &mut Surface, run: Run) -> Run {
        let last = frame.width().saturating_sub(1);
        // The target's own ground, not a space. The frame is opaque so the two are the same cell
        // today, and taking it from the surface is what keeps that a fact rather than a coincidence
        // — a repair that punched an opaque space into a non-opaque target would be the exact hole
        // `opaque: false` exists to prevent, in a cell the caller never wrote (spec §5).
        let ground = frame.ground();
        let span = Run {
            y: run.y,
            lo: run.lo.saturating_sub(1),
            hi: run.hi.saturating_add(1).min(last),
        };
        let bounds = (span.lo, span.hi);
        let row = frame.row_mut(run.y);
        // What the two slop columns held before this run repainted them. A slop column is only
        // damage if it moved.
        let was = (row[span.lo as usize], row[span.hi as usize]);

        let floor = self.layers.iter().rposition(|l| l.floors(span));
        let from = match floor {
            Some(at) => at,
            None => {
                // The frame is opaque, so its ground is a blank rather than the `EMPTY` a
                // non-opaque layer is born as. It is a paint like any other and has the same two
                // seams: a span whose left edge lands on the continuation of a pair painted last
                // frame orphans that pair's head.
                row[span.lo as usize..=span.hi as usize].fill(Cell::BLANK);
                mend(row, span.lo as i32 - 1, bounds, ground);
                mend(row, span.hi as i32, bounds, ground);
                0
            }
        };

        for layer in &self.layers[from..] {
            // An operator layer has no cells. Ticket 12 is what gives this arm something to do.
            let Kind::Content { surface, opaque } = &layer.kind else {
                continue;
            };
            let y = run.y as i32 - layer.rect.y;
            if y < 0 || y >= layer.rect.h as i32 {
                continue;
            }
            let lo = (span.lo as i32).max(layer.rect.x);
            let hi = (span.hi as i32).min(layer.rect.right() - 1);
            if hi < lo {
                continue;
            }
            let src = surface.row(y as u16);
            let src_lo = (lo - layer.rect.x) as usize;
            let src_hi = (hi - layer.rect.x) as usize;

            if *opaque {
                row[lo as usize..=hi as usize].copy_from_slice(&src[src_lo..=src_hi]);
                mend(row, lo - 1, bounds, ground);
            } else {
                // Whether the cell to the left was written by *this* pass. A boundary between two
                // cells one layer wrote at once is that layer's own business — the same rule the
                // opaque arm gets for free, because a `copy_from_slice` cannot touch a boundary
                // inside what it copied. Only a boundary the skip created is the composite's.
                let mut wrote_left = false;
                for (i, s) in src[src_lo..=src_hi].iter().enumerate() {
                    let x = lo as usize + i;
                    let wrote = !s.grapheme.is_empty();
                    if wrote {
                        row[x] = *s;
                    }
                    if !(wrote && wrote_left) {
                        mend(row, x as i32 - 1, bounds, ground);
                    }
                    wrote_left = wrote;
                }
            }
            mend(row, hi, bounds, ground);
        }

        Run {
            y: run.y,
            lo: if row[span.lo as usize] == was.0 {
                run.lo
            } else {
                span.lo
            },
            hi: if row[span.hi as usize] == was.1 {
                run.hi
            } else {
                span.hi
            },
        }
    }

    /// Every **content** layer, in **storage order**, each carrying its own `(z, seq)`.
    ///
    /// Deliberately not "bottom-up". Storage order happens to be bottom-up because the inserts sort
    /// as they go, and handing that out as an ordering would make the reference compositor take its
    /// stacking order from the fast path — so a defect in that insert would be invisible to the
    /// gate generated from it. The oracle sorts for itself.
    ///
    /// Operator layers are absent because neither compositor paints one yet; ticket 12 is what puts
    /// them in both.
    #[cfg(test)]
    pub(crate) fn as_stored(&self) -> impl Iterator<Item = LayerRef<'_>> {
        self.layers.iter().filter_map(|l| match &l.kind {
            Kind::Content { surface, opaque } => Some(LayerRef {
                z: l.z,
                seq: l.seq,
                rect: l.rect,
                opaque: *opaque,
                surface,
            }),
            Kind::Operator(_) => None,
        })
    }

    /// Every layer's `(id, z, seq)`, bottom to top. What "the order is byte-identical" is asserted
    /// over.
    #[cfg(test)]
    pub(crate) fn order(&self) -> Vec<(LayerId, i32, u32)> {
        self.layers.iter().map(|l| (l.id, l.z, l.seq)).collect()
    }

    /// How many cells every layer's damage reports, summed.
    ///
    /// The denominator of gate #3 is the cells the verbs wrote; this is the numerator. Summed per
    /// **layer** and never over the frame's union, because two popups that overlap on screen still
    /// write into two separate surfaces and each surface's bitset is exact about its own — the
    /// union is allowed to be smaller and that is occlusion, not under-reporting.
    #[cfg(test)]
    pub(crate) fn reported_cells(&self) -> usize {
        self.layers
            .iter()
            .filter_map(Layer::surface)
            .map(Surface::damaged_cells)
            .sum()
    }

    /// Whether every surface in this stack speaks **this stack's** handle space.
    ///
    /// §5's invariant is that *every surface in one layer stack belongs to one engine*, and it is
    /// what replaces the per-cell remap the alternatives needed — 4.9x on a realistic layer and 46x
    /// on a hostile one (ADR 0011). It holds by construction rather than by checking:
    /// [`add_content`](LayerStack::add_content) mints a surface whose own tables are empty, and
    /// [`add_content_with`](LayerStack::add_content_with) renumbers a donated surface into this
    /// stack's space and empties the table it came with. So a surface still carrying a handle space
    /// of its own arrived through a door nobody has built.
    ///
    /// Asked once a frame under `debug_assert!` and never in a release build, because the frame path
    /// may not pay for an invariant construction already guarantees. The stack is tens of layers, so
    /// the debug cost is a walk of tens of pointers.
    fn speaks_one_handle_space(&self) -> bool {
        self.layers
            .iter()
            .filter_map(Layer::surface)
            .all(|s| s.tables().is_empty())
    }

    /// What a resize does to the stack: nothing but forget.
    ///
    /// A layer's rectangle and its cells are its own and do not change with the screen. What becomes
    /// meaningless is the bookkeeping about *this* frame — the exposures, which are rectangles in a
    /// coordinate space that has just been replaced, and the per-layer damage, which a frame that
    /// marks everything is about to subsume. **There is no cache to invalidate, because §5 refused
    /// the only one there would have been.**
    pub(crate) fn forget_damage(&mut self) {
        self.exposed.clear();
        self.clear_damage();
    }

    /// Clear every layer's damage. `present` owns this; nothing above the engine can reach it.
    pub(crate) fn clear_damage(&mut self) {
        for layer in &mut self.layers {
            if let Kind::Content { surface, .. } = &mut layer.kind {
                surface.damage_mut().clear();
            }
        }
    }
}

/// What an untouched cell of a content layer holds.
///
/// A non-opaque layer is born **`EMPTY`** rather than blank. The trap spec §5 names is a caller who
/// draws only a border into a popup and gets a rectangle of opaque spaces that erases the window
/// underneath; being born blank is what would spring it.
const fn ground(opaque: bool) -> Cell {
    if opaque {
        Cell::BLANK
    } else {
        Cell::new(GraphemeId::EMPTY, Style::DEFAULT)
    }
}

/// Restore the pairing invariant across one boundary of a paint, **within `span`**.
///
/// `left` is the column on the left of the boundary, so `-1` is the boundary before the first
/// column and `row.len() - 1` the one after the last. A wide head on the left demands a
/// `CONTINUATION` on the right and nothing else may carry one; where the two disagree, the half
/// that has lost its partner is blanked.
///
/// One boundary at a time and left to right, which is what makes a single pass enough: blanking a
/// head cannot orphan the cell to its right, because it is blanked precisely when that cell is not
/// its continuation, and blanking a continuation cannot orphan the cell to its left for the mirror
/// image of the same reason.
///
/// # Why the clamp, and why it costs nothing
///
/// Layers paint bottom-up, so a seam is measured against a cell that is **not final** until the
/// topmost layer covering it has painted. Inside `span` that corrects itself — whoever paints last
/// mends last, over the same boundary. Outside it there is nobody to correct the guess, and the
/// guess would be made against a lower layer's cell: a base painting a space under a non-opaque
/// overlay's pair looks exactly like a bisection until the overlay's own half arrives.
///
/// Nothing is lost by declining. The column outside `span` is a column no layer repainted, and a
/// repaired frame holds a wide head there **if and only if** what the layers paint one column in is
/// its continuation — so the boundary already pairs, and there was nothing to mend.
fn mend(row: &mut [Cell], left: i32, span: (u16, u16), ground: GraphemeId) {
    let right = left + 1;
    let head = left >= 0 && row[left as usize].grapheme.is_wide_head();
    let cont = (right as usize) < row.len() && row[right as usize].grapheme.is_continuation();
    let orphan = match (head, cont) {
        (true, false) => left as u16,
        (false, true) => right as u16,
        _ => return,
    };
    if orphan < span.0 || orphan > span.1 {
        return;
    }
    // The blanked half keeps its own style and goes back to the target's **ground**, for the two
    // reasons `Row::blank` does one level down: the background is what the eye notices, and a
    // half-erased glyph that also changes colour reads as a bug even when the text is right; and
    // this is a cell nobody asked to write, so blanking it to an opaque space inside a non-opaque
    // target would erase what is underneath.
    let cell = &mut row[orphan as usize];
    *cell = Cell::new(ground, cell.style);
}

/// `rect`, shrunk to the cells a surface of `size` actually has. Placement is the rectangle's;
/// extent is the surface's, and where they disagree the smaller wins.
fn fit(rect: Rect, size: (u16, u16)) -> Rect {
    Rect::new(rect.x, rect.y, rect.w.min(size.0), rect.h.min(size.1))
}

/// The link id a donated one becomes.
///
/// # An id the donor's table does not name is passed through, not cleared
///
/// [`Screen::link`](crate::Screen::link) is the **only** public mint, and it mints into the layer
/// stack's table. So the only link id a caller can put on a standalone [`Surface`] is one that
/// already belongs to this stack's handle space, and every publicly reachable donation arrives with
/// an **empty** donor link table and ids that are already right. Renumbering them would map them
/// onto whatever the donor happened to hold, and clearing them would **delete a hyperlink
/// silently** — which is the one failure this whole area exists to prevent: the prototype spec §5
/// records deleted a hyperlink under a shadow, and `Style::with_fg_bg` was removed for the same
/// thing.
///
/// The case the two rules disagree about — a surface carrying ids from **both** tables — is not
/// reachable from the public API, because there is no second mint for one of them to come from.
/// That is a gap in spec §3/§4/§12 rather than a decision this file may take, and it is filed as
/// [architecture ticket 21](../../../.scratch/vitui-engine-architecture/issues/21-a-hyperlink-on-a-standalone-surface-has-no-mint.md).
fn remapped(links: &[LinkId], id: LinkId) -> LinkId {
    let Some(i) = id.index() else {
        return LinkId::NONE;
    };
    match links.get(i) {
        Some(mapped) => *mapped,
        None => {
            // The undecidable case, made loud in debug rather than left to be discovered: a donor
            // that minted links of its own **and** a cell naming an id past the end of them. There
            // is no public way to build one today, and if there ever is, pass-through stops being
            // obviously right and ticket 21 has to have answered first.
            debug_assert!(
                links.is_empty(),
                "a donated surface carries link ids from two handle spaces at once"
            );
            id
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::restyle::Restyle;
    use crate::style::Style;
    use crate::testing::{Harness, assert_pairing_holds};
    use vitui_bench::Bench;

    /// One whole frame — take the damage, composite every run it reported, clear — with those runs
    /// handed back for the tests that are about *which* cells were repainted.
    fn repaint(stack: &mut LayerStack, frame: &mut Surface) -> Vec<Run> {
        let runs = runs_of(stack, frame);
        for r in &runs {
            stack.composite_run(frame, *r);
        }
        frame.damage_mut().clear();
        stack.clear_damage();
        runs
    }

    /// The same, for the tests that are about the picture rather than the runs.
    fn composite(stack: &mut LayerStack, frame: &mut Surface) {
        repaint(stack, frame);
    }

    fn glyphs(s: &Surface, y: u16) -> String {
        s.row(y)
            .iter()
            .map(|c| c.grapheme.as_scalar().unwrap_or('?'))
            .collect()
    }

    fn runs_of(stack: &mut LayerStack, frame: &mut Surface) -> Vec<Run> {
        stack.take_damage_into(frame);
        let mut runs = Vec::new();
        frame.damage().for_each_run(|r| runs.push(r));
        runs
    }

    #[test]
    fn a_layer_is_forty_bytes_so_that_the_ordered_scan_stays_cheap() {
        // The reason `Kind` boxes the surface. A `Surface` inline would put every layer on its own
        // cache line, and §5 chose a contiguous `Vec` precisely because the dominant operation is
        // a traversal of the whole stack. The number is asserted rather than argued because it is
        // the only thing the decision rests on, and adding one `Vec` to `Layer` would undo it
        // silently.
        assert_eq!(size_of::<Layer>(), 40);
        assert!(
            size_of::<Surface>() > 4 * size_of::<Layer>(),
            "if a surface ever became small enough to inline, this decision is worth re-taking"
        );
    }

    #[test]
    fn a_new_stack_is_empty() {
        let stack = LayerStack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn a_content_layer_paints_into_the_frame() {
        let mut stack = LayerStack::new();
        let id = stack.add_content(0, Rect::new(1, 1, 4, 1), true);
        stack.view(id).unwrap().text(0, 0, "abcd", Style::new());
        let mut frame = Surface::new(8, 3);
        composite(&mut stack, &mut frame);
        assert_eq!(glyphs(&frame, 1), " abcd   ");
    }

    #[test]
    fn a_new_layer_damages_its_whole_rectangle() {
        let mut stack = LayerStack::new();
        stack.add_content(0, Rect::new(2, 1, 3, 2), true);
        let mut frame = Surface::new(8, 4);
        assert_eq!(
            runs_of(&mut stack, &mut frame),
            vec![Run { y: 1, lo: 2, hi: 4 }, Run { y: 2, lo: 2, hi: 4 }]
        );
    }

    #[test]
    fn a_higher_layer_paints_over_a_lower_one() {
        let mut stack = LayerStack::new();
        let low = stack.add_content(0, Rect::new(0, 0, 8, 1), true);
        let high = stack.add_content(1, Rect::new(2, 0, 3, 1), true);
        stack
            .view(low)
            .unwrap()
            .fill(Rect::new(0, 0, 8, 1), ".", Style::new());
        stack
            .view(high)
            .unwrap()
            .fill(Rect::new(0, 0, 3, 1), "#", Style::new());
        let mut frame = Surface::new(8, 1);
        composite(&mut stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), "..###...");
    }

    #[test]
    fn layers_added_out_of_z_order_still_paint_in_z_order() {
        let mut stack = LayerStack::new();
        let high = stack.add_content(5, Rect::new(0, 0, 4, 1), true);
        let low = stack.add_content(-5, Rect::new(0, 0, 4, 1), true);
        stack
            .view(high)
            .unwrap()
            .fill(Rect::new(0, 0, 4, 1), "#", Style::new());
        stack
            .view(low)
            .unwrap()
            .fill(Rect::new(0, 0, 4, 1), ".", Style::new());
        let mut frame = Surface::new(4, 1);
        composite(&mut stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), "####");
    }

    #[test]
    fn two_layers_with_identical_z_paint_in_insertion_order() {
        let mut stack = LayerStack::new();
        let first = stack.add_content(0, Rect::new(0, 0, 4, 1), true);
        let second = stack.add_content(0, Rect::new(0, 0, 4, 1), true);
        stack
            .view(first)
            .unwrap()
            .fill(Rect::new(0, 0, 4, 1), "1", Style::new());
        stack
            .view(second)
            .unwrap()
            .fill(Rect::new(0, 0, 4, 1), "2", Style::new());
        let mut frame = Surface::new(4, 1);
        composite(&mut stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), "2222");
    }

    #[test]
    fn a_layer_hanging_off_an_edge_is_intersected_not_rejected() {
        let mut stack = LayerStack::new();
        let id = stack.add_content(0, Rect::new(-2, -1, 4, 3), true);
        stack
            .view(id)
            .unwrap()
            .fill(Rect::new(0, 0, 4, 3), "#", Style::new());
        let mut frame = Surface::new(4, 2);
        composite(&mut stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), "##  ");
        assert_eq!(glyphs(&frame, 1), "##  ");
    }

    #[test]
    fn a_layer_larger_than_the_frame_is_intersected_not_rejected() {
        // The other half of §5's first degenerate case: too big rather than out to the left.
        let mut stack = LayerStack::new();
        let id = stack.add_content(0, Rect::new(0, 0, 400, 400), true);
        stack
            .view(id)
            .unwrap()
            .fill(Rect::new(0, 0, 400, 400), "#", Style::new());
        let mut frame = Surface::new(3, 2);
        composite(&mut stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), "###");
        assert_eq!(glyphs(&frame, 1), "###");
    }

    #[test]
    fn a_shadow_falling_off_the_edge_is_the_same_intersection() {
        // An operator hanging off two edges at once. It paints nothing until ticket 12; what is
        // asserted here is that placing it is not a panic and that it damages only what is on
        // screen.
        let mut stack = LayerStack::new();
        stack.add_operator(
            -1,
            Rect::new(-2, 1, 4, 40),
            Mix::new(Color::rgb(0, 0, 0), 128),
        );
        let mut frame = Surface::new(4, 3);
        assert_eq!(
            runs_of(&mut stack, &mut frame),
            vec![Run { y: 1, lo: 0, hi: 1 }, Run { y: 2, lo: 0, hi: 1 }]
        );
    }

    #[test]
    fn a_zero_area_layer_is_skipped() {
        let mut stack = LayerStack::new();
        let id = stack.add_content(0, Rect::new(1, 1, 0, 4), true);
        let op = stack.add_operator(1, Rect::new(1, 1, 4, 0), Mix::new(Color::DEFAULT, 256));
        assert_eq!(stack.len(), 2);
        let mut frame = Surface::new(8, 4);
        assert!(runs_of(&mut stack, &mut frame).is_empty());
        assert_eq!(stack.topmost_at(1, 1), None, "it covers no cell to be hit");
        // And it is still a layer: reachable, drawable-into and removable.
        assert!(stack.view(id).is_some());
        assert!(stack.remove(op));
    }

    #[test]
    fn a_non_opaque_layer_lets_what_is_under_it_show_through() {
        let mut stack = LayerStack::new();
        let low = stack.add_content(0, Rect::new(0, 0, 4, 1), true);
        let high = stack.add_content(1, Rect::new(0, 0, 4, 1), false);
        stack
            .view(low)
            .unwrap()
            .fill(Rect::new(0, 0, 4, 1), ".", Style::new());
        // Only the middle two cells of the upper layer are written; the rest stay EMPTY.
        stack.view(high).unwrap().text(1, 0, "ab", Style::new());
        let mut frame = Surface::new(4, 1);
        composite(&mut stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), ".ab.");
    }

    #[test]
    fn an_opaque_layer_drawn_only_at_its_border_erases_what_is_under_it() {
        // The other half of the same decision: `opaque` is a promise the caller makes, and this is
        // what it costs when it is made carelessly. Spec §5 names it as the real degenerate case.
        let mut stack = LayerStack::new();
        let low = stack.add_content(0, Rect::new(0, 0, 4, 1), true);
        let high = stack.add_content(1, Rect::new(0, 0, 4, 1), true);
        stack
            .view(low)
            .unwrap()
            .fill(Rect::new(0, 0, 4, 1), ".", Style::new());
        stack.view(high).unwrap().text(1, 0, "ab", Style::new());
        let mut frame = Surface::new(4, 1);
        composite(&mut stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), " ab ");
    }

    #[test]
    fn view_of_a_layer_this_stack_did_not_mint_is_none() {
        let mut ours = LayerStack::new();
        let mut theirs = LayerStack::new();
        let stranger = theirs.add_content(0, Rect::new(0, 0, 1, 1), true);
        assert!(ours.view(stranger).is_none());
        let id = ours.add_content(0, Rect::new(0, 0, 1, 1), true);
        assert!(ours.view(id).is_some());
    }

    #[test]
    fn view_of_an_operator_layer_is_none() {
        // `view(id)` is the only way to a layer's cells, and an operator layer has none.
        let mut stack = LayerStack::new();
        let op = stack.add_operator(0, Rect::new(0, 0, 4, 1), Mix::new(Color::DEFAULT, 64));
        assert!(stack.view(op).is_none());
    }

    #[test]
    fn two_layers_standing_apart_leave_the_gap_between_them_undamaged() {
        // The reason a ground fill costs nothing on an ordinary frame: no damaged run reaches past
        // the layers that caused it, so the gap is never composited at all.
        let mut stack = LayerStack::new();
        let left = stack.add_content(0, Rect::new(0, 0, 5, 1), true);
        let right = stack.add_content(0, Rect::new(100, 0, 5, 1), true);
        stack
            .view(left)
            .unwrap()
            .fill(Rect::new(0, 0, 5, 1), "L", Style::new());
        stack
            .view(right)
            .unwrap()
            .fill(Rect::new(0, 0, 5, 1), "R", Style::new());

        let mut frame = Surface::new(300, 1);
        let runs = runs_of(&mut stack, &mut frame);
        assert_eq!(
            runs,
            vec![
                Run { y: 0, lo: 0, hi: 4 },
                Run {
                    y: 0,
                    lo: 100,
                    hi: 104
                }
            ],
            "per-row spans would merge these into one run 105 cells wide"
        );

        for r in runs {
            stack.composite_run(&mut frame, r);
        }
        assert!(
            frame.row(0)[5..100].iter().all(|c| *c == Cell::BLANK),
            "the gap was never damaged, so it was never composited"
        );
    }

    #[test]
    fn clearing_damage_leaves_the_cells_alone() {
        let mut stack = LayerStack::new();
        let id = stack.add_content(0, Rect::new(0, 0, 4, 1), true);
        stack
            .view(id)
            .unwrap()
            .fill(Rect::new(0, 0, 4, 1), "#", Style::new());
        stack.clear_damage();
        let mut frame = Surface::new(4, 1);
        assert!(runs_of(&mut stack, &mut frame).is_empty());
    }

    // --- removing, reordering and moving -------------------------------------------------------

    #[test]
    fn removing_a_layer_exposes_what_was_under_it() {
        let mut stack = LayerStack::new();
        let low = stack.add_content(0, Rect::new(0, 0, 6, 1), true);
        let high = stack.add_content(1, Rect::new(2, 0, 2, 1), true);
        stack
            .view(low)
            .unwrap()
            .fill(Rect::new(0, 0, 6, 1), ".", Style::new());
        stack
            .view(high)
            .unwrap()
            .fill(Rect::new(0, 0, 2, 1), "#", Style::new());
        let mut frame = Surface::new(6, 1);
        composite(&mut stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), "..##..");

        assert!(stack.remove(high));
        assert_eq!(stack.len(), 1);
        composite(&mut stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), "......");
    }

    #[test]
    fn removing_the_only_layer_leaves_the_frames_own_ground() {
        // The fill `composite_run` did not have before this ticket: nothing is under the removed
        // layer, so the exposed run has no layer to paint it and the frame's blank is the answer.
        let mut stack = LayerStack::new();
        let id = stack.add_content(0, Rect::new(1, 0, 3, 1), true);
        stack
            .view(id)
            .unwrap()
            .fill(Rect::new(0, 0, 3, 1), "#", Style::new());
        let mut frame = Surface::new(5, 1);
        composite(&mut stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), " ### ");

        assert!(stack.remove(id));
        composite(&mut stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), "     ");
    }

    #[test]
    fn removing_a_layer_this_stack_did_not_mint_is_false() {
        let mut ours = LayerStack::new();
        let mut theirs = LayerStack::new();
        let stranger = theirs.add_content(0, Rect::new(0, 0, 1, 1), true);
        assert!(!ours.remove(stranger));
        assert!(!ours.set_z(stranger, 3));
        assert!(!ours.set_rect(stranger, Rect::new(0, 0, 1, 1)));
    }

    #[test]
    fn raising_a_layer_and_dropping_it_back_restores_the_exact_original_order() {
        // The gate `seq` exists for. Three layers share a `z`, so a `seq` that was rewritten on the
        // way up could only put the raised one back at the end.
        let mut stack = LayerStack::new();
        let a = stack.add_content(0, Rect::new(0, 0, 4, 1), true);
        let b = stack.add_content(0, Rect::new(0, 0, 4, 1), true);
        let c = stack.add_content(0, Rect::new(0, 0, 4, 1), true);
        let before = stack.order();
        assert_eq!(
            before.iter().map(|e| e.0).collect::<Vec<_>>(),
            vec![a, b, c]
        );

        assert!(stack.set_z(a, 100));
        assert_eq!(
            stack.order().iter().map(|e| e.0).collect::<Vec<_>>(),
            vec![b, c, a],
            "raised to the top"
        );

        assert!(stack.set_z(a, 0));
        assert_eq!(
            stack.order(),
            before,
            "byte-identical, because `seq` never moved"
        );
    }

    #[test]
    fn two_layers_with_identical_z_stay_stable_under_any_later_reordering() {
        // §5's third degenerate case. `b` goes up and comes back; `c` goes down and comes back;
        // the order is the one insertion gave.
        let mut stack = LayerStack::new();
        let a = stack.add_content(0, Rect::new(0, 0, 4, 1), true);
        let b = stack.add_content(0, Rect::new(0, 0, 4, 1), true);
        let c = stack.add_content(0, Rect::new(0, 0, 4, 1), true);
        let before = stack.order();
        for (id, z) in [(b, 7), (c, -7), (b, 0), (c, 0)] {
            assert!(stack.set_z(id, z));
        }
        assert_eq!(stack.order(), before);
        assert_eq!(
            stack.order().iter().map(|e| e.0).collect::<Vec<_>>(),
            vec![a, b, c]
        );
    }

    #[test]
    fn setting_z_repaints_the_layers_rectangle() {
        let mut stack = LayerStack::new();
        let low = stack.add_content(0, Rect::new(0, 0, 4, 1), true);
        let high = stack.add_content(1, Rect::new(1, 0, 2, 1), true);
        stack
            .view(low)
            .unwrap()
            .fill(Rect::new(0, 0, 4, 1), ".", Style::new());
        stack
            .view(high)
            .unwrap()
            .fill(Rect::new(0, 0, 2, 1), "#", Style::new());
        let mut frame = Surface::new(4, 1);
        composite(&mut stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), ".##.");

        assert!(stack.set_z(high, -1));
        assert_eq!(
            repaint(&mut stack, &mut frame),
            vec![Run { y: 0, lo: 1, hi: 2 }],
            "only the moved layer's own rectangle"
        );
        assert_eq!(glyphs(&frame, 0), "....", "it went under");
    }

    #[test]
    fn setting_z_to_the_z_it_already_has_damages_nothing() {
        let mut stack = LayerStack::new();
        let id = stack.add_content(3, Rect::new(0, 0, 4, 1), true);
        let mut frame = Surface::new(4, 1);
        composite(&mut stack, &mut frame);
        assert!(stack.set_z(id, 3));
        assert!(runs_of(&mut stack, &mut frame).is_empty());
    }

    #[test]
    fn moving_a_layer_keeps_its_cells_and_repaints_both_rectangles() {
        let mut stack = LayerStack::new();
        let id = stack.add_content(0, Rect::new(0, 0, 2, 1), true);
        stack.view(id).unwrap().text(0, 0, "ab", Style::new());
        let mut frame = Surface::new(6, 1);
        composite(&mut stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), "ab    ");

        assert!(stack.set_rect(id, Rect::new(4, 0, 2, 1)));
        assert_eq!(
            repaint(&mut stack, &mut frame),
            vec![Run { y: 0, lo: 0, hi: 1 }, Run { y: 0, lo: 4, hi: 5 }],
            "the rectangle it left and the one it arrived at"
        );
        assert_eq!(glyphs(&frame, 0), "    ab", "the cells travelled with it");
    }

    #[test]
    fn resizing_a_layer_starts_it_again_from_its_ground() {
        // A resized window redraws. Carrying the old cells across would leave a picture drawn for
        // the shape the layer used to be.
        let mut stack = LayerStack::new();
        let id = stack.add_content(0, Rect::new(0, 0, 4, 1), true);
        stack.view(id).unwrap().text(0, 0, "abcd", Style::new());
        let mut frame = Surface::new(6, 1);
        composite(&mut stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), "abcd  ");

        assert!(stack.set_rect(id, Rect::new(0, 0, 6, 1)));
        composite(&mut stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), "      ");
        stack.view(id).unwrap().text(0, 0, "xyzxyz", Style::new());
        composite(&mut stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), "xyzxyz");
    }

    #[test]
    fn moving_a_donated_layer_fitted_smaller_than_its_surface_keeps_its_cells() {
        // The review's find. The layer's rectangle is 2 wide and its donated surface is 6, so a
        // `set_rect` that asked the *surface* whether the size changed would reallocate on a pure
        // move and throw away the cells `set_rect` promises to keep.
        let mut off = Surface::new(6, 1);
        off.root().text(0, 0, "abcdef", Style::new());
        let mut stack = LayerStack::new();
        let id = stack.add_content_with(0, Rect::new(0, 0, 2, 1), true, off);
        let mut frame = Surface::new(8, 1);
        composite(&mut stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), "ab      ");

        assert!(stack.set_rect(id, Rect::new(5, 0, 2, 1)));
        composite(&mut stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), "     ab ", "the cells travelled with it");
    }

    #[test]
    fn moving_an_operator_layer_repaints_both_rectangles() {
        let mut stack = LayerStack::new();
        let low = stack.add_content(0, Rect::new(0, 0, 8, 1), true);
        stack
            .view(low)
            .unwrap()
            .fill(Rect::new(0, 0, 8, 1), ".", Style::new());
        let op = stack.add_operator(1, Rect::new(0, 0, 2, 1), Mix::new(Color::DEFAULT, 128));
        let mut frame = Surface::new(8, 1);
        composite(&mut stack, &mut frame);

        assert!(stack.set_rect(op, Rect::new(6, 0, 2, 1)));
        assert_eq!(
            runs_of(&mut stack, &mut frame),
            vec![Run { y: 0, lo: 0, hi: 1 }, Run { y: 0, lo: 6, hi: 7 }]
        );
    }

    #[test]
    fn an_identity_operator_marks_no_damage() {
        // §5: `amount == 0` is skipped entirely and the identity never reaches a cell.
        let mut stack = LayerStack::new();
        let op = stack.add_operator(0, Rect::new(0, 0, 4, 1), Mix::new(Color::rgb(0, 0, 0), 0));
        let mut frame = Surface::new(4, 1);
        assert!(runs_of(&mut stack, &mut frame).is_empty());
        assert!(stack.set_rect(op, Rect::new(1, 0, 2, 1)));
        assert!(runs_of(&mut stack, &mut frame).is_empty());
    }

    #[test]
    fn a_mix_saturates_rather_than_wrapping() {
        assert_eq!(Mix::new(Color::DEFAULT, 60_000).amount(), Mix::FULL);
        assert!(Mix::new(Color::DEFAULT, 0).is_identity());
        assert!(!Mix::new(Color::DEFAULT, 1).is_identity());
        assert_eq!(
            Mix::new(Color::rgb(1, 2, 3), 8).toward(),
            Color::rgb(1, 2, 3)
        );
    }

    /// Gate #1's shape, applied to the verbs that have no scene.
    ///
    /// Composite the whole stack the slow obvious way before a topology change and again after it;
    /// every cell that differs must lie inside a run the damage structure reported, and the
    /// damage-tracked frame must equal the oracle **everywhere**, including outside every run.
    /// Nothing in that sentence mentions `exposed`, which is why it would still catch the ground
    /// fill being wrong after the mechanism behind it is replaced.
    ///
    /// The twelve scenes are a normative list and none of them removes a layer, so this drives the
    /// same oracle from here rather than adding a thirteenth (spec §14).
    fn agrees_with_the_oracle(
        stack: &mut LayerStack,
        frame: &mut Surface,
        what: &str,
        change: impl FnOnce(&mut LayerStack),
    ) {
        let (w, h) = frame.size();
        let before = crate::reference::composite(stack, w, h);
        change(stack);
        let after = crate::reference::composite(stack, w, h);

        let runs = repaint(stack, frame);

        for (x, y) in crate::reference::differences(&before, &after) {
            assert!(
                runs.iter().any(|r| r.y == y && r.lo <= x && x <= r.hi),
                "{what}: ({x}, {y}) changed and no run reported it"
            );
        }
        for y in 0..h {
            for x in 0..w {
                assert_eq!(
                    frame.row(y)[x as usize],
                    after.row(y)[x as usize],
                    "{what}: the damage-tracked frame and the oracle disagree at ({x}, {y})"
                );
            }
        }
    }

    #[test]
    fn every_topology_change_agrees_with_the_reference_compositor() {
        const W: u16 = 12;
        const H: u16 = 4;
        let mut stack = LayerStack::new();
        let mut frame = Surface::new(W, H);

        let back = stack.add_content(0, Rect::new(0, 0, W, H), true);
        stack
            .view(back)
            .unwrap()
            .fill(Rect::new(0, 0, W, H), ".", Style::new());
        let popup = stack.add_content(2, Rect::new(2, 1, 5, 2), true);
        stack
            .view(popup)
            .unwrap()
            .fill(Rect::new(0, 0, 5, 2), "#", Style::new());
        let float = stack.add_content(1, Rect::new(8, 0, 3, 3), false);
        stack.view(float).unwrap().text(0, 0, "xyz", Style::new());
        composite(&mut stack, &mut frame);

        agrees_with_the_oracle(&mut stack, &mut frame, "raise the popup", |s| {
            assert!(s.set_z(popup, 9));
        });
        agrees_with_the_oracle(&mut stack, &mut frame, "drop it back", |s| {
            assert!(s.set_z(popup, 2));
        });
        agrees_with_the_oracle(&mut stack, &mut frame, "move the float", |s| {
            assert!(s.set_rect(float, Rect::new(1, 2, 3, 2)));
        });
        agrees_with_the_oracle(&mut stack, &mut frame, "redraw the float", |s| {
            s.view(float).unwrap().text(0, 0, "pq", Style::new());
        });
        agrees_with_the_oracle(&mut stack, &mut frame, "add a layer over it", |s| {
            let over = s.add_content(3, Rect::new(0, 0, 4, 1), true);
            s.view(over)
                .unwrap()
                .fill(Rect::new(0, 0, 4, 1), "=", Style::new());
        });
        agrees_with_the_oracle(&mut stack, &mut frame, "remove the popup", |s| {
            assert!(s.remove(popup));
        });
        // The step the ground fill exists for: nothing is left under the float.
        agrees_with_the_oracle(&mut stack, &mut frame, "remove the background", |s| {
            assert!(s.remove(back));
        });
        agrees_with_the_oracle(&mut stack, &mut frame, "empty the stack", |s| {
            assert!(s.remove(float));
        });
        assert_eq!(
            stack.len(),
            1,
            "the layer added mid-sequence is still there"
        );
    }

    // --- the point query -----------------------------------------------------------------------

    #[test]
    fn topmost_at_answers_with_the_layer_on_top() {
        let mut stack = LayerStack::new();
        let low = stack.add_content(0, Rect::new(0, 0, 8, 2), true);
        let high = stack.add_content(1, Rect::new(2, 0, 3, 1), true);
        assert_eq!(stack.topmost_at(3, 0), Some(high));
        assert_eq!(stack.topmost_at(3, 1), Some(low), "below the popup");
        assert_eq!(stack.topmost_at(0, 0), Some(low));
        assert_eq!(stack.topmost_at(9, 0), None, "outside every rectangle");
        assert_eq!(stack.topmost_at(-1, 0), None);
    }

    #[test]
    fn topmost_at_never_returns_an_operator_layer_at_any_z() {
        // A shadow is not a thing you click, so a click near a modal's edge cannot resolve to it.
        // Asserted at three `z`s — under the content, between two content layers, and above
        // everything — because "not hittable" must not be an accident of ordering.
        let mut stack = LayerStack::new();
        let window = stack.add_content(0, Rect::new(0, 0, 8, 1), true);
        let popup = stack.add_content(5, Rect::new(2, 0, 2, 1), true);
        for z in [-100, 3, 100] {
            let shadow =
                stack.add_operator(z, Rect::new(0, 0, 8, 1), Mix::new(Color::DEFAULT, 128));
            for x in 0..8 {
                let hit = stack.topmost_at(x, 0);
                assert_ne!(
                    hit,
                    Some(shadow),
                    "an operator answered at x = {x}, z = {z}"
                );
                let expected = if (2..4).contains(&x) { popup } else { window };
                assert_eq!(hit, Some(expected));
            }
            assert!(stack.remove(shadow));
        }
    }

    #[test]
    fn topmost_at_over_an_empty_stack_is_none() {
        assert_eq!(LayerStack::new().topmost_at(0, 0), None);
    }

    #[test]
    fn topmost_at_follows_a_layer_that_was_raised() {
        let mut stack = LayerStack::new();
        let a = stack.add_content(0, Rect::new(0, 0, 4, 1), true);
        let b = stack.add_content(0, Rect::new(0, 0, 4, 1), true);
        assert_eq!(stack.topmost_at(1, 0), Some(b));
        assert!(stack.set_z(a, 1));
        assert_eq!(stack.topmost_at(1, 0), Some(a));
    }

    #[test]
    fn topmost_at_answers_over_a_non_opaque_layers_untouched_cells() {
        // The query is about rectangles. A popup with a transparent gutter is still the layer at
        // that point; which of its cells were written is the runtime's business.
        let mut stack = LayerStack::new();
        stack.add_content(0, Rect::new(0, 0, 4, 1), true);
        let overlay = stack.add_content(1, Rect::new(0, 0, 4, 1), false);
        assert_eq!(stack.topmost_at(3, 0), Some(overlay));
    }

    // --- donation ------------------------------------------------------------------------------

    fn donated(surface: Surface) -> (LayerStack, LayerId) {
        let mut stack = LayerStack::new();
        let (w, h) = surface.size();
        let id = stack.add_content_with(0, Rect::new(0, 0, w, h), true, surface);
        (stack, id)
    }

    #[test]
    fn a_donated_surface_of_plain_text_moves_in_as_it_is() {
        let mut off = Surface::new(4, 1);
        off.root().text(0, 0, "ab漢", Style::new());
        assert!(
            off.tables().is_empty(),
            "Latin and CJK are their own handles and touch no table"
        );

        let (mut stack, _) = donated(off);
        let mut frame = Surface::new(4, 1);
        composite(&mut stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), "ab漢?", "the continuation reads as `?`");
        assert!(stack.tables().is_empty(), "and nothing was interned here");
    }

    #[test]
    fn a_cluster_donated_and_composited_reaches_the_frame_as_the_same_cluster() {
        // The equality the renumbering exists for. The handle the cell arrives with is **not** the
        // handle it leaves with: the donor's table has one entry and this stack's has two, so a
        // pass that forgot to renumber would resolve the cell against the wrong row.
        let family = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";
        let mut off = Surface::new(4, 1);
        off.root().text(0, 0, family, Style::new());
        let donated_handle = off.row(0)[0].grapheme;

        let mut stack = LayerStack::new();
        // Something else is already in this stack's table, so id 0 over there is not id 0 here.
        let other = stack.add_content(0, Rect::new(0, 0, 2, 1), true);
        stack
            .view(other)
            .unwrap()
            .text(0, 0, "e\u{301}", Style::new());
        let id = stack.add_content_with(1, Rect::new(0, 0, 4, 1), true, off);

        let mut frame = Surface::new(4, 1);
        composite(&mut stack, &mut frame);
        let landed = frame.row(0)[0].grapheme;
        assert_ne!(landed, donated_handle, "it was renumbered");
        assert_eq!(
            stack.tables().interner.resolve(landed).as_deref(),
            Some(family)
        );
        assert!(stack.view(id).is_some());
    }

    #[test]
    fn a_hyperlinked_cell_donated_and_composited_resolves_to_the_same_uri() {
        // The link id inside the extended-style entry has to be renumbered *before* the entry is
        // re-interned, or the new table dedups against the wrong key.
        const URI: &str = "https://example.com/donated";
        let mut off = Surface::new(2, 1);
        {
            let link = off.tables_mut().link(URI);
            let mut view = off.root();
            view.text(0, 0, "ab", Style::new());
            view.restyle(
                Rect::new(0, 0, 2, 1),
                &Restyle {
                    link: Some(link),
                    ..Default::default()
                },
            );
        }

        let donated_handle = off.row(0)[0]
            .style
            .ext_handle()
            .expect("extended over there");
        let donated_link = off
            .tables()
            .exts
            .get(donated_handle)
            .expect("interned over there")
            .link;

        let mut stack = LayerStack::new();
        // Two URIs and an extended style this stack minted first, so **neither** the donor's link
        // id nor its extended-style handle is this stack's — which is what makes the assertions
        // below about renumbering rather than about two tables happening to agree.
        stack.tables_mut().link("https://example.com/a");
        stack.tables_mut().link("https://example.com/b");
        let seeded = stack.add_content(0, Rect::new(0, 0, 2, 1), true);
        stack.view(seeded).unwrap().text(0, 0, "xy", Style::new());
        stack.view(seeded).unwrap().restyle(
            Rect::new(0, 0, 2, 1),
            &Restyle {
                ul: Some(Color::rgb(1, 1, 1)),
                ..Default::default()
            },
        );
        stack.add_content_with(1, Rect::new(0, 0, 2, 1), true, off);

        let mut frame = Surface::new(2, 1);
        composite(&mut stack, &mut frame);
        let handle = frame.row(0)[0]
            .style
            .ext_handle()
            .expect("the cell is extended");
        let entry = stack.tables().exts.get(handle).expect("interned here");
        assert_ne!(handle, donated_handle, "the extended handle was renumbered");
        assert_ne!(entry.link, donated_link, "and so was the link id inside it");
        assert_eq!(
            stack.tables().links.uri(entry.link),
            Some(URI),
            "and it still names the same URI, which is the whole point"
        );
    }

    #[test]
    fn a_screen_minted_link_survives_a_donation_it_was_not_minted_for() {
        // The only publicly reachable hyperlink-plus-donation flow, because `Screen::link` is the
        // only mint: the id is already in this stack's space and arrives on a surface whose own
        // link table is empty. Clearing it here would delete a hyperlink silently, which is the
        // defect this area exists to prevent. See `remapped` and architecture ticket 21.
        const URI: &str = "https://example.com/minted-by-the-screen";
        let mut stack = LayerStack::new();
        let link = stack.tables_mut().link(URI);

        let mut off = Surface::new(2, 1);
        {
            let mut view = off.root();
            view.text(0, 0, "ab", Style::new());
            view.restyle(
                Rect::new(0, 0, 2, 1),
                &Restyle {
                    link: Some(link),
                    ..Default::default()
                },
            );
        }
        assert!(off.tables().links.is_empty(), "no second mint exists");

        stack.add_content_with(0, Rect::new(0, 0, 2, 1), true, off);
        let mut frame = Surface::new(2, 1);
        composite(&mut stack, &mut frame);
        let handle = frame.row(0)[0]
            .style
            .ext_handle()
            .expect("the cell is extended");
        let entry = stack.tables().exts.get(handle).expect("interned here");
        assert_eq!(stack.tables().links.uri(entry.link), Some(URI));
    }

    #[test]
    fn a_donated_underline_colour_survives_without_a_link() {
        // The extended half that carries no link at all: `remapped` must leave `NONE` alone rather
        // than sending it through the map.
        let mut off = Surface::new(2, 1);
        off.root().text(0, 0, "ab", Style::new());
        off.root().restyle(
            Rect::new(0, 0, 2, 1),
            &Restyle {
                ul: Some(Color::rgb(9, 8, 7)),
                ..Default::default()
            },
        );

        let (stack, _) = donated(off);
        let handle = stack.as_stored().next().unwrap().surface.row(0)[0]
            .style
            .ext_handle()
            .expect("the cell is extended");
        let entry = stack.tables().exts.get(handle).expect("interned here");
        assert_eq!(entry.ul, Color::rgb(9, 8, 7));
        assert_eq!(entry.link, LinkId::NONE);
    }

    #[test]
    fn a_donated_surface_hands_its_own_tables_back_empty() {
        let mut off = Surface::new(2, 1);
        off.root().text(0, 0, "e\u{301}", Style::new());
        assert!(!off.tables().is_empty());

        let (stack, _) = donated(off);
        let surface = stack.as_stored().next().unwrap().surface;
        assert!(
            surface.tables().is_empty(),
            "the surface speaks the stack's handle space now"
        );
    }

    #[test]
    fn two_donations_of_the_same_cluster_land_on_one_entry() {
        let cluster = "a\u{308}";
        let mut stack = LayerStack::new();
        for x in [0, 2] {
            let mut off = Surface::new(2, 1);
            off.root().text(0, 0, cluster, Style::new());
            stack.add_content_with(0, Rect::new(x, 0, 2, 1), true, off);
        }
        assert_eq!(
            stack.tables().interner.entries().len(),
            1,
            "the second donation deduplicated"
        );
    }

    #[test]
    fn a_donated_surface_smaller_than_its_rectangle_paints_only_its_own_cells() {
        let mut off = Surface::new(2, 1);
        off.root().text(0, 0, "ab", Style::new());
        let mut stack = LayerStack::new();
        stack.add_content_with(0, Rect::new(0, 0, 6, 1), true, off);
        let mut frame = Surface::new(6, 1);
        composite(&mut stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), "ab    ");
        assert_eq!(stack.topmost_at(4, 0), None, "the rectangle shrank with it");
    }

    #[test]
    fn a_donated_surface_larger_than_its_rectangle_is_clipped() {
        let mut off = Surface::new(6, 2);
        off.root().text(0, 0, "abcdef", Style::new());
        off.root().text(0, 1, "ghijkl", Style::new());
        let mut stack = LayerStack::new();
        stack.add_content_with(0, Rect::new(0, 0, 3, 1), true, off);
        let mut frame = Surface::new(6, 2);
        composite(&mut stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), "abc   ");
        assert_eq!(
            glyphs(&frame, 1),
            "      ",
            "the second row is not the layer's"
        );
    }

    // --- ticket 11: the wide-glyph hazard at a layer edge -----------------------------------

    #[test]
    fn a_layer_hanging_off_the_left_edge_leaves_no_bare_continuation() {
        let mut h = Harness::new(12, 1);
        let id = h
            .screen
            .layers()
            .add_content(0, Rect::new(-1, 0, 12, 1), true);
        h.screen
            .layers()
            .view(id)
            .unwrap()
            .text(0, 0, "漢字", Style::new());
        h.present();
        assert_pairing_holds(h.screen.frame());
    }

    #[test]
    fn a_non_opaque_overlay_half_covering_a_pair_leaves_no_two_adjacent_heads() {
        let mut h = Harness::new(6, 1);
        let low = h
            .screen
            .layers()
            .add_content(0, Rect::new(0, 0, 4, 1), true);
        let high = h
            .screen
            .layers()
            .add_content(1, Rect::new(1, 0, 2, 1), false);
        h.screen
            .layers()
            .view(low)
            .unwrap()
            .text(0, 0, "漢字", Style::new());
        h.screen
            .layers()
            .view(high)
            .unwrap()
            .text(0, 0, "漢", Style::new());
        h.present();
        assert_pairing_holds(h.screen.frame());
    }

    /// The bug ticket 07 found in the prototype's `blit`, kept as a test rather than as a sentence.
    ///
    /// The repair was right and the damage was wrong: the blanked half lay one column outside the
    /// layer's rectangle, the damage covered the rectangle only, and the terminal went on showing
    /// the half that had just been erased — for as long as nothing else wrote there, which on an
    /// idle screen is for ever. `Harness::present` is what makes the second half of that assertable:
    /// it replays the bytes and compares the terminal's screen against the frame, so a repair
    /// nobody packed fails here rather than in a screenshot.
    #[test]
    fn a_repair_outside_the_layers_rectangle_is_damaged_and_reaches_the_terminal() {
        let mut h = Harness::new(8, 1);
        let base = h
            .screen
            .layers()
            .add_content(0, Rect::new(0, 0, 8, 1), true);
        h.screen
            .layers()
            .view(base)
            .unwrap()
            .text(4, 0, "漢", Style::new());
        h.present();

        // One column wide, over the pair's **second** half only. Everything this layer knows about
        // is column 5.
        let over = h
            .screen
            .layers()
            .add_content(1, Rect::new(5, 0, 1, 1), true);
        h.screen
            .layers()
            .view(over)
            .unwrap()
            .text(0, 0, "x", Style::new());
        h.present();

        assert_eq!(
            glyphs(h.screen.frame(), 0),
            "     x  ",
            "the orphaned head at column 4 is blanked"
        );
        assert_eq!(
            h.screen.runs(),
            [Run { y: 0, lo: 4, hi: 5 }],
            "the damaged run reaches column 4, which no layer covers"
        );
        assert_pairing_holds(h.screen.frame());
    }

    /// The other direction: the layer moves off the pair and the half it had blanked comes back.
    ///
    /// Nothing has damaged that column — the layers painted exactly what they painted last frame —
    /// so only the composite's own slop can restore it. This is the case the twelve-bisecting-layers
    /// gate found; it is written out small here because a gate that fails over a screen of CJK says
    /// very little about why.
    #[test]
    fn a_blanked_half_comes_back_when_the_layer_that_bisected_it_moves_away() {
        let mut h = Harness::new(8, 1);
        let base = h
            .screen
            .layers()
            .add_content(0, Rect::new(0, 0, 8, 1), true);
        h.screen
            .layers()
            .view(base)
            .unwrap()
            .text(4, 0, "漢", Style::new());
        let over = h
            .screen
            .layers()
            .add_content(1, Rect::new(5, 0, 1, 1), true);
        h.screen
            .layers()
            .view(over)
            .unwrap()
            .text(0, 0, "x", Style::new());
        h.present();
        assert_eq!(glyphs(h.screen.frame(), 0), "     x  ");

        h.screen.layers().set_rect(over, Rect::new(7, 0, 1, 1));
        h.present();
        assert_eq!(
            glyphs(h.screen.frame(), 0),
            "    漢? x",
            "column 4 is the head again, and no layer damaged it"
        );
        assert_pairing_holds(h.screen.frame());
    }

    /// A repair may not reach past the columns the composite repainted.
    ///
    /// The layers are painted bottom-up, so a seam is measured against a cell that is not the final
    /// one until the topmost layer covering it has painted. Inside the composited span that
    /// resolves itself — whoever paints last mends last. **Outside it there is nobody to correct
    /// the guess**, and the guess is made against a lower layer's cell.
    ///
    /// Here the base paints a space over column 4, the seam at (3, 4) looks orphaned, and the
    /// non-opaque overlay that owns the pair has not painted its half yet. A repair that reached
    /// column 3 would erase half a glyph nothing damaged, and nothing downstream would repaint it —
    /// the run does not cover it, so the blanking would be invisible to the terminal and permanent
    /// on the screen.
    #[test]
    fn a_repair_does_not_reach_past_the_columns_the_composite_repainted() {
        let mut h = Harness::new(10, 1);
        let base = h
            .screen
            .layers()
            .add_content(0, Rect::new(0, 0, 10, 1), true);
        h.screen
            .layers()
            .view(base)
            .unwrap()
            .fill(Rect::new(0, 0, 10, 1), ".", Style::new());
        // Non-opaque, and its pair starts one column left of the span the run below composites —
        // so the overlay's own half arrives *after* the base has painted over the other one.
        let over = h
            .screen
            .layers()
            .add_content(1, Rect::new(3, 0, 4, 1), false);
        h.screen
            .layers()
            .view(over)
            .unwrap()
            .text(0, 0, "漢", Style::new());
        h.present();
        assert_eq!(glyphs(h.screen.frame(), 0), "...漢?.....");

        // Damage column 5 only. The span is 4..=6, and column 3 is outside it.
        let mark = h
            .screen
            .layers()
            .add_content(2, Rect::new(5, 0, 1, 1), true);
        h.screen
            .layers()
            .view(mark)
            .unwrap()
            .text(0, 0, "X", Style::new());
        h.present();
        assert_eq!(
            glyphs(h.screen.frame(), 0),
            "...漢?X....",
            "the pair at columns 3 and 4 belongs to a layer the run never asked about"
        );
        assert_pairing_holds(h.screen.frame());
    }

    /// Two runs a repair made touch are one run.
    ///
    /// The gap between them is one column, and the right-hand run widens into it to blank a head its
    /// own paint orphaned. Left as two runs it would cost a cursor move between adjacent cells and
    /// break §14's gate #2, which reads *two runs that touch are one run*. They cannot **overlap**,
    /// and the reason is worth keeping: a run widens right only by blanking an orphaned
    /// `CONTINUATION` and left only by blanking an orphaned wide head, and one column cannot be
    /// both.
    #[test]
    fn two_runs_a_repair_made_touch_are_folded_into_one() {
        let mut h = Harness::new(12, 1);
        let base = h
            .screen
            .layers()
            .add_content(0, Rect::new(0, 0, 12, 1), true);
        h.screen
            .layers()
            .view(base)
            .unwrap()
            .text(4, 0, "漢", Style::new());
        h.present();

        // Column 3 and column 5, with column 4 — the pair's head — undamaged between them.
        for x in [3, 5] {
            let id = h
                .screen
                .layers()
                .add_content(1, Rect::new(x, 0, 1, 1), true);
            h.screen
                .layers()
                .view(id)
                .unwrap()
                .text(0, 0, "x", Style::new());
        }
        h.present();

        assert_eq!(
            h.screen.runs(),
            [Run { y: 0, lo: 3, hi: 5 }],
            "the right-hand run widened onto column 4 and the two became one"
        );
        assert_eq!(glyphs(h.screen.frame(), 0), "   x x      ");
        assert_pairing_holds(h.screen.frame());
    }

    /// A pair that arrives broken stays broken, and the oracle has to agree about that.
    ///
    /// `View::child` may not widen its clip (spec §4), so a pair the clip bisects keeps the half
    /// outside it and the layer's own surface holds a bare `CONTINUATION`. Whether that is right is
    /// architecture ticket 20's question and not this file's — but **the two compositors must take
    /// the same position on it**, or gate #1's equality is false for a program nobody has written
    /// yet and the failure points at the compositor instead of at the open question.
    ///
    /// The rule both of them follow: a boundary between two columns **one layer painted at once**
    /// is that layer's own business. The composite repairs the seams it creates and nothing else.
    #[test]
    fn a_pair_a_child_clip_bisected_composites_as_it_is_and_the_oracle_says_so_too() {
        let mut stack = LayerStack::new();
        let id = stack.add_content(0, Rect::new(0, 0, 8, 1), true);
        let mut view = stack.view(id).unwrap();
        view.text(3, 0, "漢", Style::new());
        // The child's clip ends at column 3, so rule 2 declines to blank the continuation at 4.
        view.child(Rect::new(0, 0, 4, 1))
            .text(3, 0, "x", Style::new());

        let mut frame = Surface::new(8, 1);
        composite(&mut stack, &mut frame);
        let oracle = crate::reference::composite(&stack, 8, 1);
        for x in 0..8 {
            assert_eq!(
                frame.row(0)[x],
                oracle.row(0)[x],
                "the damage-tracked frame and the reference compositor disagree at ({x}, 0)"
            );
        }
    }

    /// The same, through the `EMPTY` skip, which is where the two arms could disagree.
    ///
    /// The opaque arm is a `copy_from_slice` and cannot touch a boundary inside what it copied. The
    /// non-opaque arm walks cell by cell and could, so it has to be told not to: a boundary between
    /// two cells this layer wrote in one pass is the layer's, and only a boundary the skip created
    /// is the composite's.
    #[test]
    fn a_bisected_pair_inside_a_non_opaque_layer_is_left_alone_by_both_compositors() {
        let mut stack = LayerStack::new();
        let base = stack.add_content(0, Rect::new(0, 0, 8, 1), true);
        stack
            .view(base)
            .unwrap()
            .fill(Rect::new(0, 0, 8, 1), ".", Style::new());
        let over = stack.add_content(1, Rect::new(0, 0, 8, 1), false);
        let mut view = stack.view(over).unwrap();
        view.text(3, 0, "漢", Style::new());
        view.child(Rect::new(0, 0, 4, 1))
            .text(3, 0, "x", Style::new());

        let mut frame = Surface::new(8, 1);
        composite(&mut stack, &mut frame);
        let oracle = crate::reference::composite(&stack, 8, 1);
        for x in 0..8 {
            assert_eq!(
                frame.row(0)[x],
                oracle.row(0)[x],
                "the damage-tracked frame and the reference compositor disagree at ({x}, 0)"
            );
        }
    }

    #[test]
    fn a_layers_damage_translates_into_the_frame_and_is_clipped_not_dropped() {
        // Spec §6's two operations, at the level that has layers rather than bitsets: translate
        // into the frame's coordinates, and clip a layer that hangs off an edge.
        let mut stack = LayerStack::new();
        let off = stack.add_content(0, Rect::new(-4, -1, 10, 3), true);
        stack
            .view(off)
            .unwrap()
            .fill(Rect::new(0, 0, 10, 3), "#", Style::new());
        let mut frame = Surface::new(20, 4);
        let runs = runs_of(&mut stack, &mut frame);
        assert_eq!(
            runs,
            vec![Run { y: 0, lo: 0, hi: 5 }, Run { y: 1, lo: 0, hi: 5 }],
            "row -1 is dropped, the columns left of zero are clipped, and the layer is not"
        );
    }

    #[test]
    fn two_popups_a_hundred_columns_apart_are_180_cells_and_not_280() {
        // The case that chose the bitset over per-row spans (spec §6), driven through the stack
        // rather than through `RowBits` directly: the gap between two layers survives the union.
        let mut stack = LayerStack::new();
        for x in [0, 190] {
            let id = stack.add_content(0, Rect::new(x, 0, 90, 1), true);
            stack
                .view(id)
                .unwrap()
                .fill(Rect::new(0, 0, 90, 1), "#", Style::new());
        }
        let mut frame = Surface::new(300, 1);
        let emitted: usize = runs_of(&mut stack, &mut frame)
            .iter()
            .map(|r| r.len())
            .sum();
        assert_eq!(emitted, 180, "per-row spans would emit 280");
    }

    /// Ticket 11's report: what a composite costs, by damaged area and by stack depth.
    ///
    /// **A report, not a gate.** §14's rule is that a timing is a gate only at cliff granularity
    /// with the headroom written next to the number, and none of these is near one — the budget is
    /// 1 ms for a full screen and 100 µs for a typical damage-tracked frame, and both are gated
    /// where they belong, in `examples/budget.rs` over the twelve scenes.
    ///
    /// Per cell of the table and never summed, for the reason the register keeps everything per
    /// scene: the 27x scroll-detector regression the map found was visible only that way.
    ///
    /// # Two shapes, because depth means two different things
    ///
    /// The matrix is spec §5's own arrangement — a full-screen opaque base under staggered 90x14
    /// popups — and it is the shape in which depth costs anything: a popup covers part of a
    /// full-width run, so every layer is visited.
    ///
    /// The four `stacked` cases are the shape §5's **content-layer column** was measured in, layers
    /// that each cover the whole screen, and they are here because that column and this compositor
    /// disagree by design. §5 recorded 6.28 / 20.2 / 133.5 / 372.3 µs at depths 1, 3, 20 and 50 —
    /// linear, one full copy per layer. This one starts from the topmost **opaque layer that floors
    /// the run** and never looks below it, so fifty full-screen layers cost what one does. The
    /// column is not reproduced; it is the number the floor removed.
    ///
    /// **The operator column is not here and cannot be**: §5's 107.3 µs is the *popups with their
    /// shadows* figure, 78.2 µs of it is the operator layer, and ticket 12 is what makes a `Mix`
    /// reach a cell. Reporting a content-only number under that heading would be a report that
    /// quietly measured something else.
    ///
    /// ```text
    /// cargo test --release -p vitui-engine composite_costs -- --nocapture
    /// ```
    #[test]
    fn composite_costs_by_damaged_area_and_depth() {
        const W: u16 = 300;
        const H: u16 = 80;
        const DEPTHS: [usize; 4] = [1, 3, 20, 50];
        /// One cell, one row, one popup, the whole screen — §5's four damaged areas.
        const AREAS: [(&str, u16, u16); 4] = [
            ("one-cell", 1, 1),
            ("one-row", W, 1),
            ("one-popup", 90, 14),
            ("whole-screen", W, H),
        ];

        /// A full-screen opaque base under `depth - 1` layers, drawn so no cell is a sentinel the
        /// copy could shortcut. `stacked` makes those layers cover the whole screen too, which is
        /// what puts an opaque floor over every run.
        fn stack_of(depth: usize, stacked: bool) -> LayerStack {
            let mut stack = LayerStack::new();
            let base = stack.add_content(0, Rect::new(0, 0, W, H), true);
            stack
                .view(base)
                .unwrap()
                .fill(Rect::new(0, 0, W, H), ".", Style::new());
            for i in 1..depth as i32 {
                let rect = if stacked {
                    Rect::new(0, 0, W, H)
                } else {
                    Rect::new(
                        (i * 11) % (W as i32 - 92),
                        (i * 3) % (H as i32 - 15),
                        90,
                        14,
                    )
                };
                let id = stack.add_content(i, rect, true);
                stack
                    .view(id)
                    .unwrap()
                    .fill(Rect::new(0, 0, rect.w, rect.h), "#", Style::new());
            }
            stack
        }

        let mut plan: Vec<(String, u16, u16, usize, bool)> = Vec::new();
        for (area, aw, ah) in AREAS {
            for depth in DEPTHS {
                plan.push((format!("{area}/depth-{depth}"), aw, ah, depth, false));
            }
        }
        for depth in DEPTHS {
            plan.push((format!("stacked/depth-{depth}"), W, H, depth, true));
        }

        let mut bench = Bench::new(20);
        for (name, aw, ah, depth, stacked) in &plan {
            let stack = stack_of(*depth, *stacked);
            let mut frame = Surface::new(W, H);
            let runs: Vec<Run> = (0..*ah)
                .map(|y| Run {
                    y: y + (H - ah) / 2,
                    lo: (W - aw) / 2,
                    hi: (W - aw) / 2 + aw - 1,
                })
                .collect();
            // Fewer iterations for the expensive cells, so a round stays a round rather than a
            // coffee break: the reported duration is per iteration either way.
            let iters = if *aw as u32 * *ah as u32 > 10_000 {
                20
            } else {
                200
            };
            bench = bench.case(name, iters, move || {
                for r in &runs {
                    std::hint::black_box(stack.composite_run(&mut frame, *r));
                }
            });
        }
        let report = bench.run();

        if cfg!(debug_assertions) {
            println!(
                "these numbers are a debug build and are not comparable to the figures below; \
                 rerun with --release"
            );
        }
        println!(
            "composite cost, minimum of 20 rounds:\n{report}\n\
             spec §5 recorded, for content layers covering the whole screen at depths \
             1 / 3 / 20 / 50: 6.28 / 20.2 / 133.5 / 372.3 us — which is what the `stacked` rows \
             cost before an opaque floor was allowed to end the walk. And at 20 popups with their \
             shadows — 40 layers, half of them operators ticket 12 has yet to build — \
             171 ns for one cell, 2.02 us for one row, 16.7 us for one popup, 100.7 us for the \
             whole screen"
        );
    }

    #[test]
    fn the_union_across_twenty_layers_is_exact() {
        // Twenty layers, each one column wide, one gap column apart. Every gap has to survive, and
        // an `OR` is the only union that keeps all nineteen of them.
        let mut stack = LayerStack::new();
        for i in 0..20 {
            let id = stack.add_content(i, Rect::new(i * 2, 0, 1, 1), true);
            stack
                .view(id)
                .unwrap()
                .fill(Rect::new(0, 0, 1, 1), "#", Style::new());
        }
        let mut frame = Surface::new(64, 1);
        let runs = runs_of(&mut stack, &mut frame);
        assert_eq!(runs.len(), 20, "nineteen gaps, twenty runs");
        assert_eq!(runs.iter().map(|r| r.len()).sum::<usize>(), 20);
    }
}
