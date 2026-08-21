# Context

The glossary for `vitui`. Terms here are the project's canonical vocabulary — use them in code, in
tickets, in commit messages and in conversation. No implementation details live in this file.

## Layers of the system

**Engine** — `vitui-engine`. Everything that touches the terminal: cells, surfaces, layers,
compositing, damage, the bytes on the wire, input, and the frame clock that decides *when* those
bytes go out. It does not lay anything out, does not know what a widget is, never iterates
application data, and never calls the runtime. "The engine only draws" is the short form; this is the
form that survives an argument about input.

**Runtime** — `vitui-runtime`. Everything above the engine: layout, identity, focus, hit-testing,
event routing, key maps, theming, overlays and the data contract — and the API components are
written against. Convenience is the runtime's responsibility, not the engine's. Replaceable in
principle: a different runtime should be able to sit on the same engine, and a TEA-style one and a
signal-based one both do — built twice on the frozen seam, and twice again on the runtime itself,
with the same screen coming out cell for cell.

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

**Identity** — the name a widget keeps between frames, derived from the call site it is written at:
its parent's identity, its `file:line:col`, and a key where one call site produces many widgets. It
is what focus, hit-testing, interest, overlay ownership and scroll association are keyed on — five
things, not six, because a memo is keyed by where it is *stored* instead. Two properties are not
negotiable. A container that returns a rectangle preserves its children's identity and one that takes
a closure renames them, with `scope` and `scroll_scope` the deliberate exceptions. And **an `Id`
differs between two runs of the same binary and may never be persisted**: the file pointer is an
address, so nothing may write one to disk, send one over a wire, or compare one against a stored
value. See `docs/adr/0013`.

**Frame state** — what the runtime keeps for the length of one draw and rebuilds on the next: the
hit index, the focus ring, the overlay request queue, the deadline sink and the key queue. Five flat
structures, not one and not a tree, and **rebuilt from the draw rather than diffed against the last
one** — which is why a widget that did not draw cannot be clicked, focused or hovered even though
its cells still look right. Four id-keyed facts deliberately outlive it, because they are the ones a
widget cannot re-declare by drawing: the pointer grab, the press origin and the focus, which a sweep
releases when their widget stops drawing, and the click record, which is not swept.

**Scene tree** — *considered and refused*, and it was in this file for longer than any other refused
term. There is no tree of nodes anywhere in `vitui`, and four separate answers each removed one:
the clip stack **is** the call stack, so the draw tree is never a value; the id path is the
**closure** tree and not the draw tree, so a rectangle-returning split leaves its panes siblings at
one depth; the layer stack is a sorted `Vec` and a window inside a window is two entries with
different z-order; and intrinsic sizing takes no measure walk, so nothing needs a node to hang a
cached size on. What the runtime owns instead is *frame state* above. Named here so nobody
re-derives it — and because the term survived nine tickets that had already made it false.

## Drawing

**Cell** — one addressable position in the terminal grid, holding what is drawn there and how it is
styled. A double-width glyph occupies two cells.

**Surface** — a rectangular grid of cells that can be drawn into. The engine's central primitive.
Usually a layer owns one; a caller constructs one directly only to draw somewhere off-screen. A
surface in a layer stack holds cells and damage and nothing else — it draws into the *stack's*
handle tables, which is what lets one layer be composited into another as a plain copy. A surface
outside a stack has no stack to reach, so it carries a table of its own; see **Handle table**.

**Handle table** — engine-owned state that a cell's handles point into: the grapheme interner for
multi-scalar clusters, and the extended-style table for the colours that did not fit in the style
word. There is **one set per layer stack**, so every surface in one speaks one handle space. Nothing
public names a handle; drawing verbs reach the tables through the draw context.

A surface drawn through `Surface::root` is not in a stack and interns into a table of its own, which
stays empty for every string of Latin, CJK, box drawing and single-scalar emoji. `add_content_with`
renumbers a donated surface's handles into the stack's, once, at donation.

**Extended style** — a style whose colours live in a handle table rather than in its `u64`. It is
what an underline colour or a hyperlink costs, and it is a *cost, not a state*: clearing both
channels puts the cell back inline. Under 1% of cells in ordinary text.

**Sweep** — reclaiming handle-table entries nothing points at, by marking from the live surfaces and
compacting. Runs where allocation is already permitted, never inside a frame, and marks no damage —
the cells still say the same thing. When it renumbers, the next packet repaints in full, because
handle identity is the one thing the mirror compares across frames.

The composited frame is one of the live surfaces: its cells were copied out of the layers and name
the same tables, and only its damaged runs are recomposited. The **URI table is not swept**, because
an application holds link ids across frames.

**High-water mark** — the table size at which the next sweep is due, and the trigger for one. It is
twice the live count at the last sweep, with a floor — *a starting value and not a decision*: the
mechanism is measured and nothing measured discriminates between candidate policies.

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

**Blend mode** — how a layer combines with what is already beneath it. Blending resolves colours
rather than compositing transparency, and resolving them means turning a palette index or the
terminal's default colour into concrete channels first. **There is only one, and it is called
[Mix](#mix)** — the historic list of three (`Replace`, `Darken`, `Blend`) collapsed to two
mechanisms, because `Replace` is not a blend mode but what a content layer does, and alpha-over is
not expressible on a cell that has no alpha.

**Mix** — the one operator: a colour, and how far toward it what is already there is moved, out of
256. Darkening is a mix toward black, lifting is a mix toward white, a tint is a mix toward anything,
and a fade is the amount moving across frames. An amount of zero is the identity and never reaches a
cell. A gradient is **not** a mix: it is a fill with a varying style, and the compositor never sees
one.

**Shadow** — a layer that darkens what lies beneath it, offset from the layer it belongs to.

**Lifting** — the visual cue that a layer sits above the others; a shadow is its most common form.

**Composite** — to resolve the layer stack, bottom-up, into a single grid of cells, repairing every
[seam](#seam) it creates.

**Exposure** — an area of the frame that has to be repainted although no layer's own damage says so,
because the layer that owned it was removed, reordered or moved. Damage lives in a layer's surface,
so an area whose owner has gone away has nothing to speak for it; exposures are recorded by the
stack and folded into the frame's damage once, at the start of the composite.

**Ground** — what an untouched cell holds. A blank for an opaque surface, the `EMPTY` sentinel for a
non-opaque one. A composited run falls back to the frame's ground where no layer covers it, which is
what an exposure over bare screen resolves to.

**Repair** — blanking the half of a double-width pair that has lost its partner, so that a
continuation never appears without a wide head to its left and a wide head is always followed by one.
Overwriting half a pair is what corrupts a terminal's own idea of where the columns are, and the
repair is what forecloses it. A repaired half keeps its own style and goes back to the surface's
[ground](#ground), not to an opaque space — it is a cell nobody asked to write.

**Seam** — the boundary between a span that was just painted and the cell beside it. Every paint has
exactly two, which is why a repair is a constant cost per span rather than a scan: a drawing verb
repairs the seams of what it wrote, and a composite repairs the seams of what each layer copied. A
repair at a composite seam damages cells **outside** the layer's own rectangle, and a repair that is
not reported is a half the terminal goes on showing.

**Frame** — one composited, immutable grid. It is produced on the application thread and stays
there; what reaches the render thread is a snapshot of the part of it that changed.

**Snapshot** — the immutable hand-off from the application thread to the render thread: the damaged
runs of a frame together with the cells inside them, and nothing else. The render thread sees only
snapshots, never reads application state, and holds no grid of its own.

**Packet** — a snapshot in flight, together with the buffer carrying it. Packets are leased from a
pool and returned to it, so a steady stream of frames allocates nothing. A packet is
**self-contained**: every handle its cells carry is resolved at pack time into a side table the
packet owns, so the render thread never reads a handle table and the app thread may grow or sweep one
while a frame is being written.

**Lease** — taking a buffer from the pool to draw or to fill. A lease is never invalidated from
outside; if what it produced has become wrong — the terminal resized under it — the frame is
discarded and the buffer returns to the pool.

**Damage** — the region that changed and therefore has to be repainted. The engine's central
optimisation, and the subject of its central invariant: *frame cost is proportional to visible cells,
never to data volume.*

**Run** — one damaged span on one row, inclusive at both ends. The unit damage is reported in and the
only shape the serializer ever sees: runs arrive in row order, ascending by column within a row, and
that order *is* the order bytes are written in. **The word is the engine's and is not available for
anything else** — a contiguous interval of selected indices in a collection is a **`Span`**, never a
run, and the components map's "run list" is a span list. Two meanings of one word, both carrying
measurements, is the collision this glossary exists to prevent.

**Serialize** — to turn a snapshot into the bytes that go to the terminal. The engine writes its own
escape sequences; the backend crate is used for input and terminal mode, never for output.

**Mirror** — the render thread's record of what the terminal is currently showing: one grid of cells,
updated as bytes are emitted. Distinct from a frame, which is what the application *wants* shown.
Named a mirror rather than a shadow because a shadow is already a kind of layer. It is what lets the
serializer skip a cell the frame rewrote without changing, and what lets a scroll be proved before it
is emitted. See `docs/adr/0006`.

**Unknown** — a part of the mirror that cannot be trusted: at startup, after a resize, and after a
**sweep** renumbered a handle table. What is unknown is written rather than compared, which is how a
full repaint expresses itself without a separate mode. A cell becomes known when the serializer emits
it, and nothing else makes it known.

The rule that carries the property is one sentence: **never compare against what the mirror does not
know.** The danger is not the false inequality, which costs bytes; it is the false equality, which
leaves the terminal showing the wrong text.

The unit is the **cell**, and it was the row until it was measured: a row became known only when one
frame wrote every column of it, and at three hundred columns almost nothing does — over spec §14's
twelve scenes that left eleven of them with every row unknown for ever and the **equality filter**
worth nothing at all. It costs no flag and no branch, because unknown is a *value*: the `EMPTY`
sentinel, which no composited frame cell can hold, so a frame cell never compares equal to one. *Row*
survives as a question asked of the cells, and the **scroll region** is what asks it.

**Equality filter** — comparing each damaged cell against the **mirror** inside the run scan, and
emitting only what actually changed. Always on: damage area does not predict whether it pays, and the
intuition is inverted — a full-screen change damages 24 000 cells and buys nothing, a cleared-row list
scroll damages the same 24 000 and buys 37x.

**Gap merge** — repainting the cells between two changes rather than moving the cursor over them,
**priced in bytes** because that is the unit the wire is measured in: the clusters' UTF-8 lengths plus
a floor for a style change inside, against the digit-counted cheapest encoding of the move it would
avoid. A cell-counted threshold is in the wrong unit and no value of one is right — a cell is one byte
of ASCII and three of braille, so six cells is six bytes on one row and eighteen on the next. The same
rule bridges two **runs** on one row through the columns between them, which are not in the **packet**
but are in the mirror.

**Scroll region** — the terminal's own ability to move a band of rows, addressed as top and bottom
margins plus a count. Cheaper than repainting the band by two orders of magnitude, applicable only
when moving *every* column of the band produces what the frame asked for, and therefore emitted only
after that has been checked against the mirror.

**Shortest** — the cursor encoding the engine ships, and the only one: absolute positioning, absolute
column, relative forward, carriage return, and carriage return plus line feeds, **priced by digit
count** and the cheapest taken. There is no table and no per-move search over content overwrite.
Ties go to the **absolute** encoding, because a relative move compounds an error and an absolute one
cannot — which is the same reason a run that has emitted a non-ASCII cluster forbids the relative form
for the rest of its row.

**Session framing** — bytes that hold for the whole attachment rather than for one frame, written once
on entry and given back on leaving: auto-wrap off, and the alternate screen. Distinct from **frame
framing**, which is the style reset every frame opens with, the synchronised-output block it may be
wrapped in, and the hyperlink it must close — a frame is self-contained, a session is not. Frame
framing is written on the first cell that actually reaches the wire, so **a frame the equality filter
emptied says nothing at all** rather than spending twenty bytes announcing it.

**The two SGR spellings** — one colour, two encodings that no capability query separates. The
**modern** one is ITU-T T.416's, with colons and an empty colour-space id; the **legacy** one is
xterm's older semicolon form. The engine emits the modern one and a **quirk** or an override selects
the legacy one for a terminal that mis-parses it. Which is the default is a compatibility decision and
not a bandwidth one — the modern form is one byte longer per parameterised colour — and the disagreement
runs one parameter along into the underline colour, where the same two spellings exist and are selected
separately.

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

**Focus ring** — the widgets that can hold the keyboard, in the order `Tab` visits them. Rebuilt
every frame during the draw, by the widgets that declare focusability, so it is always the ring that
just drew rather than a description of the previous frame. Focusability is declared and never
derived: being clickable is not being a tab stop, and wanting keys is not either. An entry carries
its rectangle in the enclosing scroll area's content coordinates — read at the end of the same frame
and never after it — which is what lets a `Tab` onto a row below the fold scroll it into view.

**Tab stop** — a ring position `Tab` can land on. Not the same as a ring entry: a scope may collapse
a whole range onto one stop, so a menu bar of seven is seven entries and one stop. The stop count,
not the ring length, is what decides whether a keyboard walkthrough of a screen is usable.

**Focus scope** — a range of the ring, opened around a body, that changes what `Tab` does with it.
A *group* is one stop for the whole range; a *trap* is a range `Tab` cannot leave; an *isolated*
scope keeps the tab key for the focused widget, which is how a code editor inserts one. Frame-local:
a scope is a pair of ring positions and nothing about it survives the frame. The one closure-taking
construct that does **not** rename its children, because a trap appears around a form that is
already on screen and renaming it there loses the focus at exactly the wrong moment.

**Base layout** — where a key physically is, as opposed to what it printed. A shortcut is expressed
against the base layout and text against what was produced, because on a non-US layout the two
disagree and binding to either alone loses one of them. **Knowable only where the terminal reports
it**: below that, what arrives in its place is the keycap, and nothing distinguishes the two.

**Base layout reported** — whether the terminal says which physical key was pressed, rather than only
what it printed. **One boolean, never a ladder.** It decides whether a chord means what it says, and
it is read and never edited: a binding is not rewritten because the terminal is poor, it simply does
not fire. *This entry replaces "Key tier", which was an ordered ladder over a detected axis and
therefore refused twice over — by `docs/adr/0010` on principle, and by measurement, since which
legacy terminal we are in is unobservable and only flag-4-present is separable from flag-4-absent.*

**Chord** — a key together with the modifiers held with it, as a thing an application binds an
action to. It names either a base-layout position or a printed character, and which of the two is
part of the chord rather than a matter of taste: `Ctrl+S` means a place, a bare `y` on a yes/no
prompt means a letter, and on a non-US layout no answer serves both. Caps lock and num lock are
keyboard *state* and can never be part of one.

**Binding** — a chord or a few interchangeable chords, an action, and the help text that names it,
declared once so it can be both routed and rendered. What routes and what is rendered are not the
same object: routing needs the chords and the action, help needs the words.

**Key map** — an ordered set of bindings, first match wins, consulted after the focused widget and
its enclosing scopes have declined. A map is declared for a scope, and the innermost scope holding
the focus answers first — so a dialog's own bindings beat the application's while the dialog is up.

**Handoff slot** — a one-value drop point from a worker thread to the app thread. It can only be
taken from without waiting, which is why a background result reaches the app thread as something it
finds rather than something it waits for.

## Layout and sizing

**Sizing function** — a plain function beside a component that answers how large it wants to be:
the same `&data` the component takes, plus the width or height it is about to be given, returning
integers. It takes no draw context, so it cannot draw, cannot claim an identity and cannot route —
which is the whole of what makes it a function rather than a method on a trait. It is how a
container sizes to its contents without any measure pass existing.

**Measure pass** — calling a component in a mode that produces a size instead of cells, so a
container can lay out around the answer. Deliberately absent: it is either a trait a component must
implement, or a second execution of the frame, and both were built and priced. What replaces it is
a sizing function beside the component, checked against it.

**Dry run** — drawing a component into a discard surface and reading how far its verbs reached. Not
the layout mechanism — it is a second whole frame, it reports the clip rather than the content for
anything virtualised, and it repeats every side effect the frame has. It is kept as the **test**
that a sizing function still agrees with the component beside it, which is a thing no compiler
checks and which drifted silently for three tickets.

**Drawn extent** — the largest content coordinate any drawing verb touched inside a body, on both
axes. What a dry run reads, and what a scroll area over bounded content uses instead of a declared
content size. It is one frame old when a scroll area uses it, and same-frame when a test uses it.
The coordinate recorded is the one a verb **offered**, not the one it managed to write — marking the
clipped width is what made the extent blind sideways — and it is maintained only while the frame is
asked to maintain it, because measuring the width of every verb costs 7% of the frame budget.

## Scrolling

**Scroll area** — a viewport over content larger than itself, moved by an offset the *application*
owns. Cost is proportional to the content, because the body draws as if everything were visible and
the clip rejects the rest; that is what makes it right for a form and wrong for a million rows.

**Virtualised collection** — a viewport over an indexed source, where the caller draws only the rows
the visible range admits. Cost is proportional to the window. It is a different mechanism from a
scroll area and not a faster one, and choosing the wrong one of the two is the single most expensive
mistake available above this runtime.

**Scrollable** — where a widget can still move, as four directions rather than two axes. It is
declared per direction because a wheel click is one direction: a collection at its bottom that
reports "the vertical axis is movable" consumes every downward click and the area around it never
sees one.

**Wheel chaining** — the innermost scrollable under the pointer that can still move *the way the
wheel is going* consumes the click; otherwise it passes outward. Resolved from the previous frame's
index, and it is the one pointer channel that cannot be resolved at the end of the frame instead,
because the offset is read during the draw by the widget that owns it. The residue is one click, at
each end stop and on an area's first frame.

**Scroll-into-view** — bringing a newly focused entry inside its enclosing area's viewport. Resolved
at the end of the frame that drew, from the focus ring's content-coordinate rectangle, so the next
frame is already scrolled; it fires only for a keyboard-driven focus move, because a press proves
the widget was on screen and an unconditional pull fights the wheel. What crosses the frame boundary
is an offset, never a rectangle. It has no meaning inside a virtualised collection: a row that did
not draw is not in the ring.

## Overlays

**Overlay** — a layer requested during a draw and drawn after it, because a component cannot open a
layer mid-draw. The request names an owner, an anchor and a body; the body runs in a second pass,
after every base-pass draw context has been dropped, and its outcome reaches its owner on the frame
after. A dropdown, a menu, a tooltip and a modal are overlays; anything that draws inline is not.

**Owner id** — the identity an overlay is keyed on, *handed to* the request rather than derived at
it. Derivation cannot work: a component carries `#[track_caller]` and the attribute reaches into the
body, so a derived id is the id the owner already claimed one line earlier. The owner id keys the
layer's lifecycle across frames and roots the overlay's own id stack — which is what makes an
overlay a different *place* for identity and not only for geometry.

**Frame arena** — the bump region an overlay body lives in for the length of one frame. Reset rather
than freed, so a steady stream of frames allocates nothing; it drops nothing itself, so a body that
owns anything is dropped by a thunk the request carries beside it.

**Placement** — where an overlay's rectangle lands against its anchor, as integer arithmetic in one
order: place, flip, shift, clamp. Flipping is conditional on the other side having more room, so a
tie keeps the side that was asked for, and clamping is last and never resizes.

**Scrim** — the darkening under a modal. An operator layer, because a terminal cell has no alpha: it
transforms what is already there rather than covering it. Proportional to the screen it darkens and
not to the overlay it belongs to, which is why it is the expensive half of a modal.

**Modal barrier** — the position in a frame's hit index below which nothing receives the pointer. It
is an **ordering, not a membership**: everything past it is inside the modal's scope by position
alone, and no entry carries a scope of its own. That is what forces a nested overlay's z-order to
count from its parent's layer rather than from its own band — a dropdown inside a modal, sorted into
the band its own kind belongs to, would land below the barrier that exists to protect it.

## Data

**Revision** — a number that names a version of some application data, and the key a memoised result
is stored under. It comes from **one process-global counter**, not one per value: two values with
their own counters both stand at revision 1, and every memo keyed on them is blind to a swap between
them. Bumped once per edit, never per frame and never per row.

**Versioned** — a wrapper around application data whose only path to `&mut` is a guard. Reading goes
through `Deref` and stays shared, so any number of components may read one table in a frame; writing
does not, because `DerefMut` is deliberately absent, and that absence is the whole mechanism rather
than an omission.

**Edit** — the write guard `Versioned` hands out, and the thing that makes a bump unforgettable: the
`Revision` moves when the guard *drops*, not when the value changes. Three prices, each deliberate —
the data is exclusive for as long as the guard is held, an edit that changed nothing still bumps, and
one revision covers the whole value, so touching one column of a table invalidates memos of the
columns that did not move.

**Memo** — a cached result beside the `Revision` it was computed at: it recomputes when the revision
it was given differs from the one it holds, and returns what it has otherwise. Keyed by **where it is
stored** — a field of the owner's own state — so it consumes no `Id`, is not swept when its widget
stops drawing, and survives a tab being switched away and back for nothing. It is the whole of the
reactivity this runtime contains, and it is not reactivity: it is a cache with a key.

**Reactivity layer** — a TEA pump, a signal graph, or whatever else an application puts between its
state and the draw. It lives **above** the runtime and never inside it: what the runtime offers is
the frame loop, `request_frame()`, deadlines and `Wake::Posted`, and what a layer does with them is
its own business. Two consequences are worth naming because they are not obvious. **Re-running the
view is the propagation** — not as a convenience of immediate mode, but because a frame that draws
less than the whole screen *declares* less than the whole screen: the hit index, the focus ring, the
overlay queue and the deadline sink are rebuilt from the draw, so a widget that did not draw cannot
be clicked, focused or hovered, while every one of its cells still looks correct. And a layer that
wants to know whether something changed must **diff**, because the revision guard bumps when it
drops rather than when the value moves; the diff is free only because component state is small,
owned and comparable.

**Rows** — *considered and refused.* The proposal had the runtime's first and only trait, with `len`
and `revision` on it. Both were removed by building them: a length is already an argument to every
collection, and under a trait a filtered view has to invent a second one; a revision is a value, and
what makes it unforgettable is `Drop` on a guard, which is a wrapper rather than a trait. The
guarantee a trait was wanted for — O(1) indexed access — is prose in both shapes and is carried by
neither. Named here so nobody re-derives it.

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

**Question** — the thing a background job is asked, identified by a key the app thread computes. Not
the job and not the answer: two requests carrying the same key are one question, so the verb that
asks is idempotent and a component may call it unconditionally on every frame. Immediate mode has no
mount, so *every frame* is the only moment a component has.

**Generation** — a monotonic number minted on the app thread when a question is asked, and carried
back beside the answer. What makes a landing rejectable: an answer whose generation is not the newest
is an answer to a question nobody is asking. Deliberately **not** a `Revision` — a revision is
compared with `==` and answers *is this different*, never *is this newer*.

**Landing** — an answer arriving from a worker: the payload plus its generation. A landing is a write
to application data and belongs at the top of the view, before anything reads. Taken mid-draw it
tears the frame between two widgets that read the same field.

**Resident worker** — a background thread with a **one-slot inbox**, asked questions rather than
handed functions. A question replaced in the inbox before the worker looks at it costs nothing: no
thread, no started job, no decision to stop. The counterpart of the one-slot outbox and of the frame
mailbox, and the three share one rule — *the newest supersedes, because nobody wants the older one*.

**Cooperative cancel** — a flag a job agrees to poll. There is no other kind: a thread cannot be
killed, so a job that never looks is not cancellable and no signature says so. Worth having and worth
not over-trusting: it saves nothing at all while the work finishes faster than the user moves.

## Terminal

**Backend** — the seam behind which the terminal library lives. `crossterm` sits here and is not
visible in any public type.

**Capability** — something the attached terminal can do. A capability is *detected* by querying the live
pty — colour depth, synchronised output, the keyboard protocol flags — *declared* by the operator, which
is the only way the glyph repertoire can be known, since no query asks whether a font contains a
character — or **inferred**, which is the engine guessing on the world's behalf and is the case OSC 8
hyperlinks fall into, because no query asks that either and nobody was asked to promise it. The
distinction decides the shape of the answer: a detected axis is exposed as independent booleans, because
the world does not sort; a declared axis may be an ordered ladder, because a promise is downward-closed
by whoever makes it; and an inferred axis must be **correctable by a declaration**, because there is no
second query to ask more carefully. See `docs/adr/0010`.

**Repertoire** — the set of characters the operator promises their font can show: ASCII only, Unicode
with box drawing and block elements, or everything including braille and emoji. Declared, never
detected, and a component branches on it rather than the engine substituting behind its back. The
type is `GlyphSet`, with the three levels `docs/adr/0010` names.

**Glyph** — a character a component draws for structure rather than as content: an arrow, a box
corner, a tee, a line, an ellipsis. **A glyph is a lookup with a spelling at every repertoire level,
every spelling exactly one cell, and no spelling blank.** Anything failing either rule is not a
glyph, it is a *branch* — the sub-cell ladders are the case, because the number of samples asked of
the data changes with the rung and no table can carry that. The table is the **theme's**, reached
through `Theme::glyph`; a component names no repertoire. Absence is not representable, and that is
the point: a blank fallback is no slower, writes fewer cells and marks the same damage, so every
counter approves of it and only the rendered surface does not.

**Distinction** — a difference a component intends the user to see — a hover, a fade, a threshold —
narrowed by the theme to **one bit at construction**, from the palette as it arrives at the terminal
and the repertoire as it was declared. A component branches on the bool and names neither axis.
**A distinction survives the whole matrix iff it is carried on both axes**: a component told that two
roles do not differ on the wire owes a second axis — a glyph, a rule, a position — and never a darker
colour.

**Role** — what a component asks a theme for instead of asking for a colour. **A role names a paint,
not a colour**, and that is the whole of why there are thirteen of them rather than twenty-six: a
component handed a foreground and a background separately would have to pair them, and pairing is the
style literal the no-literals rule exists to forbid. Two roles are the unit degradation is measured
in, because quantisation collapses *pairs* — a role that survives a tier alone tells you nothing.

**Paint** — a resolved style, obtainable only from a `Theme`. A newtype whose inner value is private
to the theme, so a component can name a role and can never construct one; the six drawing verbs that
take a style are closed by their signature, and the seventh, `restyle`, is closed by taking a
descriptor rather than a function that could return a style it invented. A paint *is* a style once it
is made, which is why the role a cell was painted with cannot be read back from the cell. See
`docs/adr/0018`.

The one constructor that takes colours, `Theme::custom(&self, fg, bg)`, is on the **theme** and needs
a live `&Theme`, which is what keeps the rule intact: the thing a component may never mint is a
**palette**, because a palette is what `resolve(tier)` narrows. A single colour that no role can
promise — a chart's fifth series, a photograph's pixel, a test sentinel — is not a palette. A paint
made that way **carries no tier guarantee** and its component owes its own branch on `caps()`.

**Application palette** — the colours an application ships and a `Theme` is made of: thirteen roles,
each a whole paint. A base16 YAML scheme or an opencode theme JSON is one of these. It is authored
against widgets, it travels with the program, and every colour in it is a real colour.

**Theme registry** — the set of application palettes a program offers its users, plus which one is
current. A runtime type held as *application* state: the type resolves each theme against the
detected colour tier so the call cannot be forgotten, and the value is written between frames like
any other state, because a component may not write to `Env` mid-frame. A picker reads it as ordinary
data.

**Swap frame** — the one frame on which the current theme changes. It is a steady frame plus a full
repaint, because every cell whose style moved is a cell that changed; it is also the frame on which
every memo keyed on the theme misses at once, which is why a memo carries the theme in its key only
when its value is made of paints.

**Operator palette** — the sixteen ANSI colours the *terminal* is configured with, plus its own
default foreground and background. An `.itermcolors` plist or a terminal profile is one of these. It
decides what `indexed(n)` and `Color::DEFAULT` mean on the machine the frame lands on, no application
may assume it, and nothing in the process can read it — which is why a role resolved to a palette
index is a role whose distance from another role is unknowable from inside (`docs/adr/0007`), and why
a pair count taken at sixteen colours is a lower bound rather than a measurement.

**Quirk** — a correction applied *after* detection, for a terminal that answers a query correctly and
then misbehaves anyway. Where the per-terminal facts live: legacy SGR on ConPTY, and which text
attributes actually work.

**Override** — a capability **pinned** by the caller instead of being detected, in either direction: a
pin may raise as well as lower, which is what makes a truecolor headless tier reachable. Overrides are
where `--ascii` and `--no-color` land, and they sit at the top of one stated precedence order: the
explicit API, then `VITUI_*` environment variables, then `NO_COLOR`, then `TERM=dumb` or a non-tty,
then the quirk table, then detection, then conservative defaults. Every field has a `VITUI_*` twin,
because level 2 exists to be the same lever without a new release.

**Which axes are overridable** is a rule and not a list, and it is not the same set as the *capabilities*
a caller can read: **an axis is overridable iff nothing measured it — nothing can, or the engine inferred
it — or the engine's own output depends on it and the value is one the person at the terminal knows.**
`Capabilities` is what someone above can act on; `Overrides` is what someone below can be told. Two
questions, two field sets, and the input axes are on neither side of the second: a declaration cannot
make an event arrive. See `docs/adr/0010`.

**Degradation** — rendering the same scene against a weaker terminal without the caller writing it
twice. The engine degrades *presentation* — colour is quantised, an unsupported attribute is dropped,
both silently at serialise time — and never *content*: a glyph is emitted unchanged, because the
character carries the information itself and there is no meaning-preserving substitute for it. A
component that cannot express itself at a given repertoire builds something different instead. See
`docs/adr/0009`.

**Quantise** — to resolve a colour into the nearest one the terminal can show. Happens on the render
thread, inside the run scan, *before* the comparison with the mirror — so the mirror holds what the
terminal was told, and the equality filter is exact with respect to the wire.

## Verification

**Gate** — a check that fails a build. A gate is a **count**, a **ratio**, an **equality** or a
**compile outcome**, never a timing — except for one shape, below. A number that appears in a report
and is asserted nowhere is not a gate, however often it is quoted.

**Report** — a measurement that is committed and read, and gates nothing. Every
`examples/*_numbers.rs` is one. A report may not be load-bearing for a gate: five of the runtime's
negative cases were, for a while, kept honest only by a `size_of` line in a benchmark, which nobody
had decided and `cargo test` compiled by accident.

**Pair** — how a compile outcome is written: a ```` ```compile_fail ```` block and an ordinary block
in the same rustdoc, differing in exactly the hostile line. Neither half is a gate alone. Deleting the
hostile line is caught by the first half; renaming the item it protects is caught only by the second,
and **only if the second names the item by path** — a positive half that merely exercises the
mechanism survives the rename, and eleven of the runtime's fifty-seven cases were written that way.
The error-code annotation is documentation: it is not enforced on stable.

**Cliff** — a regression that shows up as a large absolute cost on one scene: an un-memoised fold, a
paste that went quadratic, a second whole frame. Caught by a **timing gate sitting at the budget**,
never at the current measurement.

**Slope** — a regression in how a cost grows with the widget count. Caught by a **growth ratio**
across two sizes of one scene, and by nothing else: a quadratic duplicate scan costs the dense screen
1.19x and walks straight through a 100 µs gate, while the same defect at 200 → 800 widgets is 9.2x
against 3.96x. Most quadratics on this map have been slopes.

**Attribution window** — the span an allocation or timing assertion is taken over, and *whose* work
it can be trusted to describe. The allocation probe is process-global: it counts allocations, not
allocations by the app thread, so a window overlapping a worker attributes the worker's growth to the
frame. A gate with a background job in it therefore runs on a deterministic spawner, or joins before
it measures.

**Scene list** — the fixed set of screens every gate and report is measured against. It is part of the
gate rather than an appendix: three scenes that score identically on every candidate validate the
wrong design while reporting success. A scene is removed only by a ticket naming the property it can
no longer distinguish.

**Round trip** — the primary instrument: composite a frame, serialise it, replay the bytes through the
terminal model, assert the replayed screen equals the frame. It stores nothing, so there is nothing to
review and nothing to maintain — and it cannot see a defect the serializer and the terminal model
share, because they are then wrong in the same direction.

**Golden frame** — the residue the round trip cannot reach: the composited **picture**, one file per
frame, as two fixed-width planes — a glyph plane and a style plane — over a legend, with a row-number
gutter and a header naming scene, frame, size and tier. Two planes because a cell's cluster and its
style word change independently and one rendering can make only one of them diff legibly.
Regenerated with `VITUI_BLESS=1`, and the review is the git diff. Never a golden **byte string**: the
encoding is the part that is allowed to change.
