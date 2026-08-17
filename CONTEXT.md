# Context

The glossary for `vitui`. Terms here are the project's canonical vocabulary — use them in code, in
tickets, in commit messages and in conversation. No implementation details live in this file.

## Layers of the system

**Engine** — `vitui-engine`. Turns drawing calls into bytes on the terminal, as fast as possible. It
owns cells, surfaces, layers, compositing, damage and the frame writer. It does not lay anything out,
does not know what a widget is, and never iterates application data.

**Runtime** — `vitui-runtime`. Everything between the engine and a component: the scene tree, layout,
reactivity, focus, hit-testing and event routing. Replaceable in principle — a different runtime
should be able to sit on the same engine.

**Components** — `vitui-components`. The library of things an application author uses directly:
windows, panels, charts, lists, trees, forms, pickers.

## Drawing

**Cell** — one addressable position in the terminal grid, holding what is drawn there and how it is
styled. A double-width glyph occupies two cells.

**Surface** — a rectangular grid of cells that can be drawn into. The engine's central primitive and
the type component authors touch most.

**View** — a borrowed rectangle of a surface: an origin, a clip region and a content offset, with no
cells of its own. What a component is handed when it is asked to draw. A view can be narrowed into a
child view and can never be widened.

**Viewport** — a surface addressed in content coordinates while only a window of it is real. Part of
what makes drawing a million-row list affordable; the other part is the visibility query, because a
discarded write is cheap rather than free and a million cheap writes are not affordable.

**Visibility query** — the engine telling a caller which content coordinates currently fall inside a
view's window, so the caller can skip the rest. The engine still iterates nothing and measures
nothing: it answers about coordinates the caller already chose.

**Clip region** — the area of a surface a write is permitted to affect. Writes outside it are
discarded silently and cheaply.

**Drawing verb** — one call that puts something into a surface. Verbs are span-shaped: they carry a
run of cells, not a single cell, because damage is marked once per verb.

**Layer** — a rectangle positioned in the stack with a z-order. A window, a popup, a shadow and a
modal dim are all layers; nothing else is. A layer is one of two kinds.

**Content layer** — a layer that carries its own cells, in a surface of its own, and is painted over
whatever lies beneath it. A window and a popup are content layers.

**Operator layer** — a layer with no cells of its own: a rectangle and a transformation applied to
whatever is already there. A shadow and a modal dim are operator layers. The distinction matters
because a terminal cell has no alpha channel, so an effect with no content of its own cannot be
expressed as content that happens to be transparent.

**Blend mode** — how a layer combines with what is already beneath it. The one mechanism from which
shadows, liftings, modal dimming, gradients and fades are all built. Blending resolves colours
rather than compositing transparency, and resolving them means turning a palette index or the
terminal's default colour into concrete channels first.

**Shadow** — a layer that darkens what lies beneath it, offset from the layer it belongs to.

**Lifting** — the visual cue that a layer sits above the others; a shadow is its most common form.

**Composite** — to resolve the layer stack, bottom-up, into a single grid of cells.

**Frame** — one composited, immutable grid. It is produced on the application thread and stays
there; what reaches the render thread is a snapshot of the part of it that changed.

**Snapshot** — the immutable hand-off from the application thread to the render thread: the damaged
runs of a frame together with the cells inside them, and nothing else. The render thread sees only
snapshots, never reads application state, and holds no grid of its own.

**Packet** — a snapshot in flight, together with the buffer carrying it. Packets are leased from a
pool and returned to it, so a steady stream of frames allocates nothing.

**Lease** — taking a buffer from the pool to draw or to fill. A lease is never invalidated from
outside; if what it produced has become wrong — the terminal resized under it — the frame is
discarded and the buffer returns to the pool.

**Damage** — the region that changed and therefore has to be repainted. The engine's central
optimisation, and the subject of its central invariant: *frame cost is proportional to visible cells,
never to data volume.*

## Threads

**App thread** — the thread that owns application state, produces frames and submits them. Not
necessarily the process's first thread: it is whichever thread the engine was attached on. The thread
whose freezing is visible to a user, and therefore the one the whole enforcement vocabulary below
exists to protect.

**Capability token** — a zero-sized value that is proof of being on the app thread, and cannot be
moved off it. It is not a permission the holder was granted so much as a fact about where the holder
is running; types that contain one inherit the same immobility.

**Handle pair** — one shared primitive presented as two types, so that each thread holds only the
verbs it is allowed to use. The app thread's half cannot leave it; the other half can do nothing the
app thread's half is responsible for. Preferred over a runtime check or a documented rule, because an
unreachable method needs no enforcement.

**Frame budget overrun** — an app-thread iteration that took longer than one frame interval, measured
from waking to submitting. Named as a distinct thing because its cause is irrelevant to its effect: a
slow pure computation and a blocking read produce the same frozen screen.

**Permitted iteration** — an iteration declared in advance to be legitimately slow, with a reason
recorded. Cold start reads configuration; that is not the defect the overrun detector hunts, and
saying so explicitly is what keeps the detector strict everywhere else.

## Terminal

**Backend** — the seam behind which the terminal library lives. `crossterm` sits here and is not
visible in any public type.

**Capability** — something the attached terminal can do, discovered at runtime rather than assumed:
colour depth, synchronised output, the keyboard protocol level, the glyph repertoire.

**Tier** — a named combination of capabilities a scene can be rendered against. Full-colour Unicode
and `--ascii --no-color` are both tiers; so is legacy Windows conhost.

**Degradation** — rendering the same scene against a lower tier without the caller writing it twice.
