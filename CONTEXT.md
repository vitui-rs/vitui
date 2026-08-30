# Context

The glossary for `vitui` — canonical vocabulary for code, comments, tickets, commits and
conversation. Definitions only. The specs and ADRs carry the arguments and the numbers.

## Layers

- **Engine** (`vitui-engine`) — everything touching the terminal: cells, surfaces, layers,
  compositing, damage, the bytes on the wire, input, and the clock deciding *when* they go out. Lays
  nothing out, knows no widget, never iterates application data, never calls the runtime.
- **Runtime** (`vitui-runtime`) — layout, identity, focus, hit-testing, routing, key maps, theming,
  overlays, the data contract. Convenience is its job, not the engine's; replaceable on the same engine.
- **Components** (`vitui-components`) — what an application author uses. A component author never names
  an engine type.
- **Draw context** — what a component is handed to draw with: a view plus what that runtime decided a
  component needs (time, a place to ask for a frame, focus, theme). The runtime's, so two runtimes offer two.
- **Component state** — what a component keeps *about* a view of data (selection, offset, expanded set),
  separate from the data so two components can show one table owning neither.
- **Identity** — the name a widget keeps between frames: parent identity, `file:line:col`, plus a key where
  one call site makes many widgets. Focus, hit-testing, interest, overlay ownership and scroll association
  key on it; a memo does not. Non-negotiable: a container returning a rectangle preserves its children's
  identity and one taking a closure renames them (`scope`/`scroll_scope` excepted); an `Id` differs between
  runs and **may never be persisted**. ADR 0013.
- **Frame state** — hit index, focus ring, overlay queue, deadline sink, key queue: **rebuilt from the draw,
  never diffed**, so a widget that did not draw cannot be clicked, focused or hovered though its cells still
  look right. Four id-keyed facts outlive it, drawing being unable to re-declare them: pointer grab, press
  origin and focus (swept when their widget stops drawing) and the click record (not swept).
- **Scene tree** — *refused, named so nobody re-derives it.* The clip stack **is** the call stack, the id path
  is the **closure** tree, the layer stack is a sorted `Vec`, and sizing takes no measure walk.

## Drawing

- **Cell** — one addressable grid position; a double-width glyph occupies two.
- **Surface** — a rectangular grid of cells. In a stack it holds cells and damage only and draws into the
  *stack's* tables, which lets one layer composite into another as a plain copy.
- **Handle table** — the grapheme interner and extended-style table a cell's handles point into. **One set per
  layer stack.** Nothing public names a handle.
- **Extended style** — a style whose colours live in a handle table rather than its `u64`. A *cost, not a state*:
  clearing both channels puts the cell back inline.
- **Sweep** — reclaiming handle-table entries nothing points at. Never inside a frame, marks no damage, and forces
  a full repaint when it renumbers, handle identity being what the mirror compares across frames. The URI table is
  not swept. Its trigger is the **high-water mark**: twice the live count, with a floor — a starting value, not a
  decision.
- **View** — a borrowed rectangle of a surface (origin, clip, content offset), narrowable, never widenable.
- **Viewport** — a surface addressed in content coordinates with only a window real. **Visibility query** — the
  engine saying which content coordinates fall inside that window, so the caller can skip the rest; it still
  iterates and measures nothing.
- **Clip region** — where a write may land; outside it writes are discarded silently and cheaply.
- **Drawing verb** — one call putting something into a surface. Span-shaped, because damage is marked once per verb.
- **Layer** — a rectangle with a z-order. A **content layer** carries its own cells; an **operator layer** has none
  and transforms what is there — necessary because a cell has no alpha.
- **Mix** — the one blend mode: a colour and how far toward it what is there moves, out of 256. Zero is the identity
  and never reaches a cell. A gradient is a fill with a varying style, not a mix.
- **Shadow** — a layer darkening what lies beneath, offset from its owner. **Lifting** is the cue that a layer sits
  above; a shadow is its commonest form.
- **Composite** — resolve the stack bottom-up into one grid, repairing every seam.
- **Exposure** — an area needing repaint though no layer's damage says so, its owner having been removed, reordered
  or moved. Recorded by the stack, folded into the frame's damage once.
- **Ground** — what an untouched cell holds: a blank if opaque, the `EMPTY` sentinel otherwise.
- **Repair** — blanking the half of a double-width pair that lost its partner. It keeps its style and returns to the
  ground, being a cell nobody asked to write.
- **Seam** — the boundary between a painted span and the cell beside it. Exactly two per paint, so repair is constant
  per span rather than a scan. A composite seam repair damages cells **outside** the layer's rectangle, and one not
  reported is a half the terminal goes on showing.
- **Frame** — one composited immutable grid, produced on the app thread and staying there.
- **Snapshot** — the hand-off to the render thread: damaged runs and their cells, nothing else.
- **Packet** — a snapshot in flight plus its buffer, **leased** from a pool and returned. Self-contained: handles are
  resolved at pack time into a side table it owns, so the app thread may sweep while a frame is written. A lease is
  never invalidated from outside; a frame a resize made wrong is discarded and the buffer returns.
- **Damage** — the region that changed. The central optimisation, and the central invariant: *frame cost is
  proportional to visible cells, never to data volume.*
- **Run** — one damaged span on one row, inclusive both ends; the only shape the serializer sees, and their order *is*
  the byte order. **The word is the engine's** — a contiguous interval of selected indices is a **`Span`**.
- **Serialize** — snapshot to bytes. The engine writes its own escape sequences; the backend is input and terminal
  mode only.
- **Mirror** — the render thread's record of what the terminal is *currently showing*, as against a frame, which is
  what the application *wants* shown. ADR 0006.
- **Unknown** — a part of the mirror that cannot be trusted (startup, resize, a renumbering sweep), written rather
  than compared — which is how a full repaint expresses itself without a mode. The rule: **never compare against what
  the mirror does not know**; the danger is the false *equality*. The unit is the **cell**, and unknown is a *value*
  (`EMPTY`), so it costs no flag and no branch.
- **Equality filter** — comparing each damaged cell against the mirror inside the run scan and emitting only what
  changed. Always on, damage area not predicting whether it pays. The same comparison the scroll region's first
  obligation makes.
- **Gap merge** — repainting between two changes rather than moving the cursor, **priced in bytes**, a cell being one
  byte of ASCII and three of braille. Also bridges two runs on one row through columns in the mirror but not the packet.
- **Scroll region** — the terminal moving a band of rows, an order of magnitude or two cheaper than repainting it, and
  emitted only after being **verified** against the mirror — never guessed, a wrong guess losing exactly the columns
  nothing can put back.
- **Band** — the rows a scroll moves and exposes. One band and one shift verified per frame. Its two **obligations**:
  every moved row already holds, in the mirror, what the frame wants where it lands; every exposed row is blank in
  every column not repainted — checked first, being smaller and the one that fails. Whole-screen bands set no margins;
  **horizontal** margins are deliberately unused.
- **Shortest** — the one cursor encoding: absolute position, absolute column, relative forward, CR, CR plus line feeds,
  **priced by digit count**. Ties go **absolute**, a relative move compounding an error — the same reason a non-ASCII
  cluster forbids the relative form for the rest of its row.
- **Session framing** — bytes holding for the whole attachment (auto-wrap off, alternate screen), against **frame
  framing** — the per-frame style reset, sync block and hyperlink close. Frame framing is written on the first cell
  that reaches the wire, so a frame the equality filter emptied says nothing at all.
- **The two SGR spellings** — the **modern** ITU-T colon form and the **legacy** xterm semicolon form, which no query
  separates. The engine emits modern; a quirk or override selects legacy. The same choice recurs in the underline colour.

## The loop

- **Screen** — the app thread's handle to the attached terminal, and the whole engine from the runtime's side.
  Dropping it gives the terminal back.
- **Frame clock** — when a composed frame becomes bytes: a ceiling in hertz enforced as a **minimum gap, not a tick**.
- **Wake** — why the thread came back: input, a posted job, a deadline, a quit. Distinct from an **event**, which is
  *what happened*.
- **Owed frame** — a frame the pacing gate deferred, not dropped; it arrives as a deadline, because what deferred it
  was the clock. Nothing is owed when nothing was damaged.
- **Time anchor** — a moment stored so a moving value is a *function of the frame's `now`*. Distinct from an
  **overlay's anchor**, a rectangle a popup is placed against.
- **Phase** — where something moving has got to, as a value rather than a moment. **Stored state may be a time anchor,
  never a phase**: a phase accumulated from a nominal interval is wrong by more the longer it runs.
- **Frame's `now`** — the moment a frame was sampled at, once. A component reads it and never the machine's clock: one
  sampling its own is invisible to a pinned clock and disagrees with everything else on screen.
- **Event** — a key, mouse action, paste, resize or focus change; owned outright, carrying when it was read, saying
  nothing about which widget it concerns.
- **Intent** — what decides whether an event may be discarded. Presses, releases, wheel turns, keystrokes and resizes
  never are; intermediate pointer positions collapse. ADR 0008.
- **Base key** — which key was pressed, as against what it printed; separate fields, one alone making `Ctrl+Shift+5`
  on a non-US layout unbindable.
- **Key text** — what a keystroke produced, as a grapheme cluster, stored inline. Too long is reported as **nothing at
  all**: half a cluster is a different cluster.
- **Read boundary** — where one `read` ended; meaningful in one place only, a lone trailing escape byte being Escape.
- **Type-ahead** — bytes typed before the program was ready, set aside during detection and handed over ahead of the
  channel, being older.
- **Unrecognised sequence** — dropped, but counted with the last kept whole, silent discard being the defect class that
  costs a day.
- **Interest** — what a component declares it wants, stated during the draw with its region. Declaring nothing costs
  nothing, and a component outside the visible area declares nothing by not drawing.
- **Tracking level** — how much the terminal reports about the pointer: nothing, buttons, drag, motion. Totally
  ordered, so several needs combine by taking the highest.
- **Focus ring** — the widgets that can hold the keyboard, in `Tab` order, rebuilt every frame by those declaring
  focusability. Declared, never derived: clickable is not a tab stop. Entries carry their rectangle in content
  coordinates. A **tab stop** is a position `Tab` can land on — a scope may collapse a range onto one, so a menu bar
  of seven is seven entries and one stop.
- **Focus scope** — a ring range changing what `Tab` does: a *group* is one stop, a *trap* cannot be left, an
  *isolated* scope keeps `Tab` for the focused widget. Frame-local, and the one closure-taking construct that does
  **not** rename its children.
- **Base layout** — where a key physically is, as opposed to what it printed. **Base layout reported** — whether the
  terminal says so: **one boolean, never a ladder**, read and never edited. A binding is not rewritten because the
  terminal is poor; it simply does not fire.
- **Chord** — a key plus modifiers, naming either a base-layout position or a printed character — and which is part of
  the chord. Caps and num lock are keyboard *state* and never part of one.
- **Binding** — chords, an action, and the help text naming it, declared once so it can be routed and rendered.
  **Key map** — an ordered set, first match wins, consulted after the focused widget and its scopes decline; the
  innermost scope holding the focus answers first.
- **Handoff slot** — a one-value drop point from a worker, takeable only without waiting, so a background result is
  something the app thread *finds*.

## Layout and sizing

- **Sizing function** — a plain function beside a component answering how large it wants to be: the same `&data` plus
  the extent it is about to be given. It takes no draw context, so it cannot draw, claim an identity or route.
- **Measure pass** — *deliberately absent*: either a trait every component implements or a second frame, both built
  and priced.
- **Dry run** — drawing into a discard surface and reading how far the verbs reached. Not the layout mechanism; kept
  as the **test** that a sizing function still agrees with its component.
- **Drawn extent** — the largest content coordinate any verb touched. The coordinate recorded is the one a verb
  **offered**, not what it managed to write, and it is maintained only when asked for.

## Scrolling

- **Scroll area** — a viewport over larger content, moved by an offset the *application* owns. Cost is proportional to
  the content: right for a form, wrong for a million rows.
- **Virtualised collection** — a viewport over an indexed source where the caller draws only the visible range; cost is
  proportional to the window. A different mechanism, not a faster one, and picking the wrong one of the two is the
  single most expensive mistake available above this runtime.
- **Scrollable** — where a widget can still move, as four directions rather than two axes, a wheel click being one
  direction.
- **Wheel chaining** — the innermost scrollable under the pointer that can still move *the way the wheel is going*
  consumes the click. Resolved from the previous frame's index — the one pointer channel that cannot wait for the end
  of the frame. Residue: one click, at each end stop and on an area's first frame.
- **Scroll-into-view** — bringing a newly focused entry into its area's viewport, resolved at the end of the frame that
  drew, for keyboard-driven moves only, a press proving the widget was on screen. What crosses the frame boundary is an
  offset, never a rectangle; it has no meaning inside a virtualised collection.

## Overlays

- **Overlay** — a layer requested during a draw and drawn after it. The request names an owner, an anchor and a body;
  the body runs after every base-pass draw context is dropped, and its outcome reaches its owner the frame after.
- **Owner id** — the identity an overlay keys on, *handed to* the request rather than derived at it, `#[track_caller]`
  reaching into the body. It roots the overlay's own id stack, making an overlay a different *place* for identity.
- **Overlay body queue** — one `Box` a body in a `Vec` the frame call owns; **n** bodies cost **n + 1** allocations. It
  replaced the frame arena, the runtime's only `unsafe` (ADR 0034), and cannot keep capacity across frames.
- **Placement** — place, flip, shift, clamp, in that order. A tie keeps the side asked for; clamping never resizes.
- **Scrim** — the darkening under a modal: an operator layer, proportional to the screen and not to the overlay, which
  makes it the expensive half.
- **Modal barrier** — the hit-index position below which nothing receives the pointer. An **ordering, not a
  membership**, which forces a nested overlay's z-order to count from its parent's layer.

## Data

- **Revision** — a number naming a version of application data and the key a memo is stored under, from **one
  process-global counter**. Bumped once per edit, never per frame or per row.
- **Versioned** — a wrapper whose only path to `&mut` is a guard; `DerefMut` is deliberately absent, and that absence is
  the whole mechanism.
- **Edit** — the write guard: the `Revision` moves when it *drops*, which is what makes a bump unforgettable. Three
  deliberate prices — exclusivity while held, a bump for an edit that changed nothing, one revision for the whole value.
- **Memo** — a cached result beside the `Revision` it was computed at, keyed by **where it is stored**, so it consumes
  no `Id` and is not swept. The whole of the reactivity here, and it is not reactivity: it is a cache with a key.
- **Reactivity layer** — a TEA pump or signal graph, **above** the runtime and never inside it. **Re-running the view is
  the propagation**, a frame drawing less than the screen *declaring* less than the screen; and a layer wanting to know
  what changed must **diff**, the guard bumping on drop rather than on movement.
- **Rows** — *refused.* The runtime's would-be only trait, removed by building it; the O(1)-access guarantee it was
  wanted for is prose in both shapes and carried by neither.

## Threads

- **App thread** — whichever thread the engine was attached on: owns application state, produces and submits frames,
  and is the one whose freezing a user sees.
- **Capability token** — a zero-sized proof of being on the app thread that cannot move off it; containing types inherit
  the immobility. **Internal** — no signature takes one. ADR 0003.
- **Handle pair** — one shared primitive as two types, so each thread holds only its own verbs; an unreachable method
  needs no enforcement. **Internal**. ADR 0003.
- **Frame budget overrun** — an iteration longer than one frame interval, waking to submitting. Its cause is irrelevant
  to its effect: a slow computation and a blocking read give the same frozen screen.
- **Permitted iteration** — an iteration declared slow in advance, with a reason. Its cost comes off the iteration, not
  the detector.
- **Stall** — an iteration that has not come back at all, as against one that came back late: a different claim, watcher
  and sanction. **Observer** — the debug-only thread watching for one; not a second overrun detector, and it may not
  panic (wrong stack), so its sanction is restore, print, abort.
- **One-slot outbox** — where a worker leaves a result: non-blocking take, no blocking twin, newest supersedes, and what
  it displaced goes back to the worker.
- **Question** — what a background job is asked, identified by a key the app thread computes. Two requests with one key
  are one question, so asking is idempotent and a component may ask every frame — the only moment immediate mode has.
- **Generation** — a monotonic number minted when a question is asked and carried back with the answer; what makes a
  landing rejectable. Deliberately **not** a `Revision`, which answers *is this different*, never *newer*.
- **Landing** — an answer arriving. A write to application data, belonging at the top of the view; taken mid-draw it
  tears the frame between two widgets reading one field.
- **Resident worker** — a background thread with a **one-slot inbox**, asked questions rather than handed functions.
  Inbox, outbox and frame mailbox share one rule — *the newest supersedes*.
- **Cooperative cancel** — a flag a job agrees to poll. There is no other kind: a thread cannot be killed.

## Terminal

- **Backend** — the seam the terminal library lives behind. `crossterm` sits here, invisible in every public type.
- **Negotiation** — the sequences sent once at startup deciding what the terminal will ever report. Not detection, and
  not symmetric: keyboard flags cost nothing at rest and are unconditional; mouse, focus and paste turn an idle
  application into a woken one and are asked for only when declared.
- **Restoration** — negotiation backwards plus the alt screen, in the order taken. **Not a method anybody calls**: an
  idempotent function behind one atomic, reached from the panic hook and from `Screen`'s `Drop`. A mode not taken may
  not be given back.
- **Actuator** — a call changing what the terminal *is* rather than what it shows: `set_mouse` and `set_cursor`, both
  **recording rather than writing**, and free in the strong sense — an unchanged value causes no frame.
- **Caret** — the terminal's own cursor, placed after the frame's last write. The only caret there is; a software one
  costs two wakeups a second. Position, shape and visibility are three separate deltas.
- **Capability** — something the terminal can do, and how it is known decides the answer's shape. *Detected* by query →
  independent booleans, the world not sorting. *Declared* by the operator (the repertoire, which no query can ask) → may
  be an ordered ladder, a promise being downward-closed. *Inferred* (OSC 8) → must be **correctable by a declaration**.
  ADR 0010.
- **Repertoire** — the characters the operator promises their font shows: ASCII, Unicode with box drawing and blocks, or
  everything including braille and emoji (`GlyphSet`). A component branches on it rather than the engine substituting
  behind its back.
- **Glyph** — a character drawn for structure rather than content: **a lookup with a spelling at every repertoire level,
  every spelling exactly one cell, and no spelling blank.** Anything failing either rule is a *branch*, not a glyph. The
  table is the **theme's**; a component names no repertoire. Absence is not representable, and that is the point.
- **Distinction** — a difference the user is meant to see, narrowed by the theme to **one bit at construction** from the
  palette as it arrives and the repertoire as declared. **It survives the matrix iff carried on both axes**: told two
  roles do not differ on the wire, a component owes a glyph, a rule or a position, never a darker colour.
- **Role** — what a component asks for instead of a colour. **A role names a paint, not a colour** — hence thirteen and
  not twenty-six, pairing being the style literal the no-literals rule forbids. Two roles are the unit degradation is
  measured in, quantisation collapsing *pairs*.
- **Paint** — a resolved style, obtainable only from a `Theme`, its inner value private; a paint *is* a style once made,
  so the role a cell was painted with cannot be read back (ADR 0018). `Theme::custom` needs a live `&Theme`: what a
  component may never mint is a **palette**. A paint made that way carries no tier guarantee and owes its own branch on
  `caps()`.
- **Application palette** — the colours an application ships and a `Theme` is made of: thirteen roles, each a whole paint.
  **Theme registry** — the set a program offers plus which is current, held as *application* state.
- **Swap frame** — the frame the theme changes on: a steady frame plus a full repaint, and where every memo keyed on the
  theme misses at once — which is why a memo carries the theme in its key only when its value is made of paints.
- **Operator palette** — the sixteen ANSI colours the *terminal* is configured with, plus its defaults. No application may
  assume it and nothing in the process can read it, so a role resolved to a palette index has an unknowable distance from
  another (ADR 0007) and a pair count at sixteen colours is a lower bound.
- **Quirk** — a correction applied *after* detection, for a terminal that answers correctly and misbehaves anyway.
- **Override** — a capability **pinned** by the caller, in either direction; where `--ascii` and `--no-color` land.
  Precedence: explicit API, `VITUI_*`, `NO_COLOR`, `TERM=dumb` or non-tty, quirks, detection, conservative defaults.
  **An axis is overridable iff nothing measured it, or the engine's own output depends on it and the person at the
  terminal knows the value.** `Capabilities` is what someone above acts on; `Overrides` is what someone below is told.
  ADR 0010.
- **Degradation** — one scene against a weaker terminal without writing it twice. The engine degrades *presentation*
  silently at serialise time and never *content*: a glyph is emitted unchanged. A component that cannot express itself at
  a repertoire builds something different. ADR 0009.
- **Quantise** — resolving a colour to the nearest the terminal can show, inside the run scan and *before* the mirror
  comparison — so the mirror holds what the terminal was told.

## Verification

- **Gate** — a check that fails a build: a **count**, **ratio**, **equality** or **compile outcome**, never a timing except
  the cliff shape below. A number asserted nowhere is not a gate, however often it is quoted.
- **Report** — a committed, read measurement that gates nothing (every `examples/*_numbers.rs`), and may not be
  load-bearing for a gate.
- **Consumer gate** — a gate whose subject is a program written *against* the surface and built by CI: every application in
  `crates/vitui-apps/examples/`. They catch a different class, because **a gate exercises a component where its author put
  it and an application puts it somewhere else**. O7 makes *every component has one* a query.
- **Path join** — matching a name through the module it is declared in, so `keys::text` and `text::text` are different
  pairs. A bare-name scan here has been wrong at least twice.
- **Pair** — a ```` ```compile_fail ```` block and an ordinary block in one rustdoc, differing in exactly the hostile line.
  Neither half is a gate alone: deleting the hostile line is caught by the first, renaming the protected item only by the
  second, and **only if the second names it by path**. The error code is documentation, unenforced on stable.
- **Cliff** — a regression as a large absolute cost on one scene, caught by a timing gate **sitting at the budget**, never
  at the current measurement.
- **Slope** — a regression in how cost grows with widget count, caught by a **growth ratio** across two sizes and by
  nothing else; most quadratics on this map have been slopes.
- **Work counter** — steps taken **inside** a visit, as against the counters of what a frame puts on the wire. Every output
  counter is blind to work producing no output. `painted` beside `touched` — the pair, never either alone.
- **Per-input ceiling** — `fixed + per_input * n`, gated **beside** a slope, not instead: a slope is blind to a constant and
  a ceiling is met by a large enough one. A component whose cost is its visible window has no `n` in its ceiling at all.
- **Attribution window** — the span an assertion is taken over, and *whose* work it describes. The allocation probe is
  process-global, so a window overlapping a worker attributes the worker's growth to the frame.
- **Scene list** — the fixed set of screens every gate and report is measured against; part of the gate, not an appendix,
  since scenes scoring identically on every candidate validate the wrong design while reporting success. A scene is removed
  only by a ticket naming the property it can no longer distinguish.
- **Round trip** — the primary instrument: composite, serialise, replay through the terminal model, assert the replayed
  screen equals the frame. It stores nothing — and cannot see a defect the serializer and the model share.
- **Golden frame** — the residue the round trip cannot reach: the composited **picture**, as two fixed-width planes (glyph
  and style) over a legend, with a row gutter and a header naming scene, frame, size and tier. Blessed with
  `VITUI_BLESS=1`; the review is the git diff. Never a golden **byte string** — the encoding is the part allowed to change.
