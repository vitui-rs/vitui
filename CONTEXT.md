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

**Viewport** — a surface addressed in content coordinates while only a window of it is real. What
makes drawing a million-row list affordable: writes outside the window cost nothing.

**Clip region** — the area of a surface a write is permitted to affect. Writes outside it are
discarded silently and cheaply.

**Layer** — a surface positioned in the stack with a z-order and a blend mode. A window, a popup, a
shadow and a modal dim are all layers; nothing else is.

**Blend mode** — how a layer combines with what is already beneath it. The one mechanism from which
shadows, liftings, modal dimming, gradients and fades are all built. A terminal cell has no alpha
channel, so blending resolves colours rather than compositing transparency.

**Shadow** — a layer that darkens what lies beneath it, offset from the layer it belongs to.

**Lifting** — the visual cue that a layer sits above the others; a shadow is its most common form.

**Composite** — to resolve the layer stack, bottom-up, into a single grid of cells.

**Frame** — one composited, immutable grid, handed to the render thread to be written.

**Snapshot** — the immutable hand-off of a frame from the application thread to the render thread.
The render thread sees only snapshots and never reads application state.

**Damage** — the region that changed and therefore has to be repainted. The engine's central
optimisation, and the subject of its central invariant: *frame cost is proportional to visible cells,
never to data volume.*

## Terminal

**Backend** — the seam behind which the terminal library lives. `crossterm` sits here and is not
visible in any public type.

**Capability** — something the attached terminal can do, discovered at runtime rather than assumed:
colour depth, synchronised output, the keyboard protocol level, the glyph repertoire.

**Tier** — a named combination of capabilities a scene can be rendered against. Full-colour Unicode
and `--ascii --no-color` are both tiers; so is legacy Windows conhost.

**Degradation** — rendering the same scene against a lower tier without the caller writing it twice.
