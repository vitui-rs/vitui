# Context

The glossary for `vitui`. Terms here are the project's canonical vocabulary — use them in code, in
tickets, in commit messages and in conversation. No implementation details live in this file.

## Layers of the system

**Engine** — `vitui-engine`. Everything that touches the terminal: cells, surfaces, layers,
compositing, damage, the bytes on the wire, input, and the frame clock that decides *when* those
bytes go out. It does not lay anything out, does not know what a widget is, never iterates
application data, and never calls the runtime. "The engine only draws" is the short form; this is the
form that survives an argument about input.

**Runtime** — `vitui-runtime`. Everything above the engine: the scene tree, layout, reactivity,
focus, hit-testing, event routing — and the API components are written against. Convenience is the
runtime's responsibility, not the engine's. Replaceable in principle: a different runtime should be
able to sit on the same engine, and a TEA-style one and a signal-based one both do.

**Components** — `vitui-components`. The library of things an application author uses directly:
windows, panels, charts, lists, trees, forms, pickers. A component author never names an engine type.

**Draw context** — what a component is handed when it is asked to draw: the runtime's own type,
carrying a view together with whatever else that runtime decided a component needs — the frame's
time, a place to ask for another frame, focus, theme. It belongs to the runtime, which is why two
runtimes can offer two different ones over the same engine.

**Component state** — what a component keeps *about* a view of data: a selection, a scroll position,
an expanded set. Named as a separate thing from the data itself because it is passed alongside it and
never owned together with it — that is what lets two components show one table at the same moment,
neither owning it and neither needing a mutable borrow of it.

## Drawing

**Cell** — one addressable position in the terminal grid, holding what is drawn there and how it is
styled. A double-width glyph occupies two cells.

**Surface** — a rectangular grid of cells that can be drawn into. The engine's central primitive.
Usually a layer owns one; a caller constructs one directly only to draw somewhere off-screen.

**View** — a borrowed rectangle of a surface: an origin, a clip region and a content offset, with no
cells of its own. A view can be narrowed into a child view and can never be widened. It is what the
runtime wraps in a draw context; a component reaches it through that, not directly.

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

**Run** — one damaged span on one row, inclusive at both ends. The unit damage is reported in and the
only shape the serializer ever sees: runs arrive in row order, ascending by column within a row, and
that order *is* the order bytes are written in.

**Serialize** — to turn a snapshot into the bytes that go to the terminal. The engine writes its own
escape sequences; the backend crate is used for input and terminal mode, never for output.

**Mirror** — the render thread's record of what the terminal is currently showing: one grid of cells,
updated as bytes are emitted. Distinct from a frame, which is what the application *wants* shown.
Named a mirror rather than a shadow because a shadow is already a kind of layer. It is what lets the
serializer skip a cell the frame rewrote without changing, and what lets a scroll be proved before it
is emitted. See `docs/adr/0006`.

**Unknown row** — a row of the mirror that cannot be trusted: at startup, and after a resize. An
unknown row is written whole rather than compared, which is how a full repaint expresses itself
without a separate mode.

**Scroll region** — the terminal's own ability to move a band of rows, addressed as top and bottom
margins plus a count. Cheaper than repainting the band by two orders of magnitude, applicable only
when moving *every* column of the band produces what the frame asked for, and therefore emitted only
after that has been checked against the mirror.

## The loop

**Screen** — the app thread's handle to the attached terminal, and the whole of the engine from the
runtime's side: the layer stack, the composited grid, the frame clock, the wake source and the event
queue behind one name. Obtained by attaching, and dropping it gives the terminal back.

**Frame clock** — the engine's decision about *when* a composed frame becomes bytes. Expressed as a
ceiling in hertz and enforced as a **minimum gap, not a tick**: the first change after a quiet period
goes out at once, and everything arriving inside the gap is folded into a single later frame. The
engine holds it, so how often a runtime offers frames is not what decides how often they are shown.

**Wake** — why the app thread came back to life: input arrived, a background job posted, a registered
deadline passed, or the program was asked to quit. Distinct from an **event**, which is *what
happened*; a wake is only the reason for looking.

**Event** — what happened: a key, a mouse action, a paste, a resize, focus arriving or leaving.
Distinct from a wake, which is only the reason the thread looked. An event is owned outright, carries
the moment the input thread read it, and says nothing about which widget it concerns.

**Intent** — the property that decides whether an event may be discarded. A press, a release, a wheel
turn, a keystroke, a resize express something the user meant and are never dropped; an intermediate
pointer position expresses only where the pointer was on the way, and consecutive ones collapse. Named
because *input is never dropped* is not a rule the engine can keep, and this one it can. See
`docs/adr/0008`.

**Interest** — what a component declares it wants to receive, stated during the draw with the region
it applies to. It belongs to the runtime; what reaches the engine is only the combined tracking level
the frame turned out to need. A component that declares nothing costs nothing, and one drawn outside
the visible area declares nothing by not being drawn.

**Tracking level** — how much the terminal is asked to report about the pointer: nothing, buttons
only, buttons and drag, or every movement. The levels are totally ordered, each containing the one
below, which is what lets several components' needs combine by taking the highest.

**Base layout** — where a key physically is, as opposed to what it printed. A shortcut is expressed
against the base layout and text against what was produced, because on a non-US layout the two
disagree and binding to either alone loses one of them.

**Handoff slot** — a one-value drop point from a worker thread to the app thread. It can only be
taken from without waiting, which is why a background result reaches the app thread as something it
finds rather than something it waits for.

## Threads

**App thread** — the thread that owns application state, produces frames and submits them. Not
necessarily the process's first thread: it is whichever thread the engine was attached on. The thread
whose freezing is visible to a user, and therefore the one the whole enforcement vocabulary below
exists to protect.

**Capability token** — a zero-sized value that is proof of being on the app thread, and cannot be
moved off it. It is not a permission the holder was granted so much as a fact about where the holder
is running; types that contain one inherit the same immobility. **Internal**: no signature takes one,
and it is named here because it is what makes the drawing types immovable, not because anyone passes
it. See `docs/adr/0003`.

**Handle pair** — one shared primitive presented as two types, so that each thread holds only the
verbs it is allowed to use. The app thread's half cannot leave it; the other half can do nothing the
app thread's half is responsible for. Preferred over a runtime check or a documented rule, because an
unreachable method needs no enforcement. **Internal** in the same sense: the app-thread halves live
inside the screen and only the posting half is ever handed out. See `docs/adr/0003`.

**Frame budget overrun** — an app-thread iteration that took longer than one frame interval, measured
from waking to submitting. Named as a distinct thing because its cause is irrelevant to its effect: a
slow pure computation and a blocking read produce the same frozen screen.

**Permitted iteration** — an iteration declared in advance to be legitimately slow, with a reason
recorded. Cold start reads configuration; that is not the defect the overrun detector hunts, and
saying so explicitly is what keeps the detector strict everywhere else.

## Terminal

**Backend** — the seam behind which the terminal library lives. `crossterm` sits here and is not
visible in any public type.

**Capability** — something the attached terminal can do. A capability is either *detected* by querying
the live pty — colour depth, synchronised output, the keyboard protocol flags — or *declared* by the
operator, which is the only way the glyph repertoire can be known, since no query asks whether a font
contains a character. The distinction decides the shape of the answer: a detected axis is exposed as
independent booleans, because the world does not sort; a declared axis may be an ordered ladder,
because a promise is downward-closed by whoever makes it. See `docs/adr/0010`.

**Repertoire** — the set of characters the operator promises their font can show: ASCII only, Unicode
with box drawing and block elements, or everything including braille and emoji. Declared, never
detected, and a component branches on it rather than the engine substituting behind its back.

**Quirk** — a correction applied *after* detection, for a terminal that answers a query correctly and
then misbehaves anyway. Where the per-terminal facts live: legacy SGR on ConPTY, and which text
attributes actually work.

**Override** — a capability pinned or lowered by the caller instead of being detected. Overrides are
where `--ascii` and `--no-color` land, and they sit at the top of one stated precedence order: the
explicit API, then `VITUI_*` environment variables, then `NO_COLOR`, then `TERM=dumb` or a non-tty,
then the quirk table, then detection, then conservative defaults.

**Degradation** — rendering the same scene against a weaker terminal without the caller writing it
twice. The engine degrades *presentation* — colour is quantised, an unsupported attribute is dropped,
both silently at serialise time — and never *content*: a glyph is emitted unchanged, because the
character carries the information itself and there is no meaning-preserving substitute for it. A
component that cannot express itself at a given repertoire builds something different instead. See
`docs/adr/0009`.

**Quantise** — to resolve a colour into the nearest one the terminal can show. Happens on the render
thread, inside the run scan, *before* the comparison with the mirror — so the mirror holds what the
terminal was told, and the equality filter is exact with respect to the wire.
