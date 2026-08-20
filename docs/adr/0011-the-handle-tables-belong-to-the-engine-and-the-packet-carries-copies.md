---
status: accepted
date: 2026-08-17
---

# The handle tables belong to the engine, and the packet carries copies

A cell holds two handles: a `u32` grapheme id and, when bit 63 of the style word is set, a 52-bit
handle to the colours that did not fit inline. Both point into tables that live **once per engine**,
not once per surface and not once per process. Drawing verbs reach them through the draw context; a
`Surface` does not hold them and no public signature names one.

> **Amended 2026-08-20 — see the Amendment below.** "Once per engine" is once per **layer stack**, and
> a surface that is not in one carries a local, usually empty, interner that compositing never reads.
> The decision below and every number in it stand unchanged.

When a frame is packed, every handle it carries is **resolved into a side table the packet owns** —
cluster bytes into the packet's arena, extended styles into the packet's own list, hyperlink URIs
into the same arena — each keyed by the handle rather than written into the cell. The cell is copied
byte for byte. The render thread therefore never holds a handle into an engine table, and the app
thread may mutate, grow or sweep those tables while a frame is being written.

## Why

**One table per engine, because a per-surface one costs the compositor its memcpy.** Ticket 06 blits
a content layer with `copy_from_slice`; a per-surface table means every cell has to be translated on
the way across, which is measured at 4.9× on a realistic layer and 46× on one where every style is
extended. Memoising the translation does not buy the memcpy back. With one handle space there is
nothing to translate, so the question disappears rather than being optimised.

**Keyed rather than written into the cell, because the mirror compares packed cells.** The render
thread keeps a mirror of what the terminal is showing (ADR 0006) and filters against it. If the
packed form of a cell depends on *where in the packet* it landed — which is what an arena offset
stored in the cell is — then an unchanged cell compares unequal whenever the frame's damage changes
shape, and the filter re-emits it. Measured at 284× in bytes on five steady frames of a page with
one cluster cell in a hundred. Keying by the handle makes a packed cell byte-identical to the surface
cell, which makes the filter exact.

The general rule, which any future change to the packet has to pass: **nothing the render thread
compares across frames may be derived from a position.**

**Copies rather than a shared table, because eviction needs a second reader not to exist.** Sharing
an append-only table behind an `Arc` was available and is refused: it would either put a lock in the
app thread's write path or leave the render thread reading a table the app thread is growing, and it
would make every eviction scheme account for handles sitting inside a submitted-but-unwritten frame.
A packet that carries copies has no such reader. The one thing that still crosses frames is handle
*identity*, which only the mirror compares — so a sweep that renumbers sets one flag on the next
packet and that frame repaints in full.

## Consequences

- Compositing stays a copy: 6.16 µs for a full screen against 30.04 µs for the translated form.
- The pack step gets faster as a side effect — 1.6×–2.6× — because the run copy is a
  `copy_from_slice` again rather than a per-cell store loop.
- The four reserved bytes of ticket 04's cell stay free. The collision that was expected between the
  cluster arena and the extended-style handle does not arise: an extended style already owns 52 bits
  of its own word.
- Eviction is a mark-and-compact sweep over the live surfaces, run where allocation is already
  permitted, costing 58.88 µs for one screen and 1.17 ms for twenty layers, and costing one full
  repaint when it renumbers.
- Every side-table round trip in the engine — `restyle`, `Mix`, pack — is memoised on the style word
  rather than taken per cell, because a run is contiguous and shares a `u64`. Without that,
  `restyle` over a hyperlinked screen is 289 µs instead of 75, and a full-screen operator is 647 µs
  instead of 200.
- A content-derived key would make the packet independent of handle identity too, removing the
  repaint. It is built and refused: it buys exactness with a collision probability, and it spends the
  app thread's budget to save the render thread's.

Evidence: `.scratch/vitui-engine-architecture/issues/16-handle-tables.md`, prototype branch
`prototype/16-handle-tables`.

## Amendment, 2026-08-20: the surface that is not in a stack

Found while implementing the ticket this ADR names —
`.scratch/vitui-engine-architecture/issues/19-the-standalone-surface-and-the-interner.md`.

**"A `Surface` does not hold them" was true of every surface this ADR was reasoned about, and there is
one it was not.** `Surface::new` and `Surface::root` are public, and a `View` obtained that way has no
engine to reach through. The gap is not a routing problem: a worker drawing an off-screen surface
cannot hold `&mut` to the app thread's interner, and that is the borrow checker rather than a
convention, so the case needs a second table, a lock in the app thread's write path, or deletion.

The amendment, in three sentences:

- The tables live in the **layer stack**, which `attach` mints and of which there is one per `Screen`.
  Every layer surface speaks that stack's handle space, and compositing is the `copy_from_slice` this
  ADR bought.
- A surface outside a stack carries an interner of its own, empty until a multi-scalar cluster is
  written through `Surface::root` — which Latin, CJK, box drawing and single-scalar emoji never do —
  and an empty one holds no allocation.
- `add_content_with` renumbers a donated surface's handles into the stack's table **once, at
  donation**, and skips entirely when there are none.

**Nothing in the decision above changes, because the remap this ADR refused is a different one.** The
4.9× and 46× were measured on a translation that runs per composite — per frame, per layer, for the
life of the layer. A donation renumbering runs once, at a scene topology change, where allocation is
already permitted and where the eviction sweep already lives. Its cost is *owed rather than measured*,
and the layer-stack ticket pays it.

The invariant is restated in the words that survive a donated surface: ***every surface in a layer
stack speaks that stack's handle space*** — by construction for `add_content`, by renumbering for
`add_content_with`.
