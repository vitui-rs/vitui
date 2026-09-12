# `vitui-runtime` — architecture spec

Status: **settled**. Date: 2026-08-19. Written by runtime ticket R16, the destination of
[Map: vitui runtime architecture](map.md).

This document replaces [`architecture.md`](architecture.md), which was a proposal written so the
map's tickets had something to attack. Sixteen of its sections carried a **[D]** mark; each has now
been confirmed, narrowed or overturned by the ticket that owned it, and the proposal is kept
unedited as the record of what was argued rather than as a description of what ships. Where this
spec and `architecture.md` disagree, this spec is right and the ticket named in the margin is why.

Nothing here is invention. Every decision below arrived on a ticket with a number behind it, and the
ticket is cited so the argument can be re-read rather than re-had. The spec is done when
`/to-tickets` can slice it into implementation work without reopening a decision.

**Reading order for a newcomer:** `CONTEXT.md` (the glossary — its terms are used here without
re-definition) → this file → `docs/adr/0012`–`0021` for the decisions that are hard to reverse →
the tickets in `issues/` for the measurements.

---

## 0. What the runtime is

`vitui-runtime` is everything between the engine's `Screen` and a component function. It owns the
frame's shape, the draw context, identity, layout as a function library, hit-testing, event routing,
focus, key maps, the two-phase overlay protocol, scrolling, the data contract, theming, deadlines
and the worker handoff.

It owns **no widgets**, **no reactivity** and **no tree**.

The engine map stated the negative half — *the engine is everything that touches the terminal; the
runtime is everything above it, including the API components see* (engine ticket 12). This is the
positive half, and one sentence of it is load-bearing enough to be an ADR on its own: what the
runtime keeps between frames is **five flat structures rebuilt from the draw** and **four id-keyed
facts a widget cannot re-declare by drawing**. There is no node, no retained hierarchy and nothing
to diff. See [ADR 0012](../../docs/adr/0012-frame-state-is-rebuilt-from-the-draw.md).

---

## 1. The frame, as a sequence

Everything else in this document hangs on this order.

```
wait()                    the engine paces this; returns Wake::{Input, Posted, Deadline, Quit}
│
├─ take          worker landings → application state, before anything reads them        §17
│
├─ begin         post the input batch and split it at a routing edge                    §7
│                sample the clock once
│                swap-and-clear the five frame structures; clear the overlay queue
│                resolve, from the PREVIOUS frame's index, the two things that
│                cannot be answered during the draw: which widget is topmost
│                (a guess, demoted) and which one the wheel belongs to
│
├─ base pass     view(&mut Ctx) draws into the base layer                               §3
│
├─ overlay pass  each request in (z, seq) order: ensure its layer, run its body,
│                repeating while bodies request more, bounded at 16 rounds              §10
│
├─ end           award the press from THIS frame's index · resolve Tab from the ring
│                that has just drawn · release the focus with the grab · settle hover ·
│                sweep the four id-keyed facts · resolve scroll-into-view · fold the
│                deadline sink and the repaint flag into one wake                       §6 §8 §13
│
├─ settle        set_mouse(tracking) · set_cursor(caret) · request_wake_at(earliest)
│
└─ present()     the engine composites damaged cells, packs, paces and submits
```

Six properties of this sequence are forced rather than chosen.

1. **Routing before the draw is only what cannot be done during it.** `architecture.md` §1 made
   routing a step of its own that consumed last frame's geometry. It is not: containment is answered
   at `interact`, in current-frame coordinates, because the pointer travels down the `Ctx` with the
   same transform the view gets. What genuinely has to come from the previous frame is which widget
   is topmost — used as a demoted guess that buys in-frame feedback and is overwritten at `end` —
   and which widget the wheel belongs to, which is the one pointer channel that cannot wait, because
   the offset is read during the draw by the widget that owns it. (R01, R05, R17)
2. **The overlay pass is after the base pass**, because a component cannot open a layer mid-draw:
   `LayerStack::view` holds `&mut` of the stack for the life of the view (`E0499`, engine ticket 14
   R2). Inherited, not re-derived.
3. **The clock is sampled once per frame** and no component computes a `dt` from a nominal interval:
   120 Hz configured achieved 99.7 fps, and a fixed `dt` is 17% slow over three seconds (engine
   ticket 14 R6). A second clock exists and is not this one — an *event* carries the moment the
   input thread read it, and a double click is a threshold against that clock, never against the
   frame's. (R06)
4. **`present()` is the only exit.** Damage is marked by the drawing verbs and cleared by `present`;
   the runtime neither declares nor clears it. This is why nothing in the runtime can force a full
   repaint, which is the whole shape of §15's swap frame.
5. **A worker landing is a write to application data and belongs at the top of the view**, before
   anything reads. Taken mid-draw it costs one torn frame per landing against zero. §1 gains no step
   for this — the top-level view already holds `&mut App` — and nothing enforces the ordering,
   because a runtime that knew the tasks would be the task registry §17 refuses. (R18)
6. **`begin` is not skippable.** `IdTable` is stamped rather than cleared, and a `Frame` that has
   never been `begin`-ed carries stamp 0 over slots stamped 0, so no slot is ever "not mine" and
   `claim` walks the ring for ever. The implementation obligation: either `begin` cannot be skipped
   by construction, or `claim` fails rather than spins. The load check R02 added is the precedent,
   and a hang is worse than a failure because `cargo test` has no per-test timeout. (R15)

**The loop itself is not owned by anything yet.** See §21.

---

## 2. A component is a function

The fork `COMPONENT-HIERARCHY.md` §6 left open is closed on exit (a).

```rust
fn button(cx: &mut Ctx, area: Rect, label: &str, st: &mut ButtonState) -> Response
```

No trait, no node, no two-pass walk, no retained tree. Layout is `Rect → [Rect]`, computed by plain
functions over integers *before* anything is drawn, and the call stack stays the only tree.

`COMPONENT-HIERARCHY.md` §6 names two exits and **the second is a strawman**: it argues against a
measure pass by arguing against a *trait*. A component is already a function and a function can be
called twice, so a **dry run** into a discard surface gives intrinsic sizing with no trait, no node,
no two-pass walk and no retained tree — all three of §6's objections are false of it, and it is the
opponent exit (a) actually had to beat. It is beaten on three numbers rather than on an argument,
and it survives as a *test*. See §12. (R04)

**What this costs, stated rather than hidden.** "Size this dialog to its contents" is not automatic:
the author calls a **sizing function** (`dialog_size(msg, buttons, max) -> (u16, u16)`, 569 ns) or
states a size. And the real cost is not that call — it is that **a sizing function and its component
are two expressions of one layout with nothing holding them together**. R01's `detail_height`
disagreed with its own component at 221 of 229 widths, including the 78 every R01 number was measured
at, and three tickets did not notice. The shape that keeps them together is that the component lays
itself out *from the arithmetic the sizing function publishes*, and the detector is §20's dry-run
gate, owed by every component that publishes a sizing function. (R04)

See [ADR 0014](../../docs/adr/0014-no-measure-pass.md).

---

## 3. `Ctx` — the draw context

One type, owned by the runtime, handed to every component. It is the only thing a component sees.

```rust
pub struct Ctx<'f, 'v> { /* view + frame + env; !Send by construction */ }
```

**Two lifetimes, not one.** `architecture.md` §3 and R01 both wrote `Ctx<'f>`, and with a single
lifetime the `'f` bound on an overlay body is not a bound: `child()` shrinks it, so a body capturing
a base-pass local *compiles*, and the safety mechanism §10 rests on is gone. `'f` is the frame's,
`'v` is the borrow's. The pair is a compile outcome in both directions, and `Ctx<'f, 'v>` costs
nothing outside the runtime's own frame module — all 123 gates that existed when it landed passed
with no component signature changed. `'f` is viral into any component that opens an overlay, and
that is the price. (R07)

`Ctx` is `!Send` for free, because it holds a view which holds the engine's immobile handles. No
annotation asks for it.

**Narrowing is a reborrow**, so exactly one child context is alive at a time. That is the type
system enforcing §2's rule that a container computes all sub-rectangles first and draws children one
at a time.

### The surface

```rust
impl<'f, 'v> Ctx<'f, 'v> {
    // ── geometry ──────────────────────────────────────────────────────────────────
    fn area(&self) -> Rect;                                 // own rect, in the coordinates it draws in
    fn size(&self) -> (u16, u16);
    fn child(&mut self, r: Rect) -> Ctx<'f, '_>;            // narrows; can never widen
    fn scrolled(&mut self, dx: i32, dy: i32) -> Ctx<'f, '_>;// content-coordinate window
    fn visible_rows(&self) -> Range<i32>;                   // the virtualisation primitive
    fn visible_cols(&self) -> Range<i32>;

    // ── drawing: a component never names View, Surface or Style ───────────────────
    fn text(&mut self, x: i32, y: i32, s: &str, st: Paint) -> Written;
    fn set(&mut self, x: i32, y: i32, cluster: &str, st: Paint) -> Written;
    fn fill(&mut self, r: Rect, cluster: &str, st: Paint);
    fn restyle(&mut self, r: Rect, d: &Repaint<'_>);        // a descriptor, never a closure — §15
    fn clear(&mut self, st: Paint);
    // `link(uri) -> Link` was here and is **deleted**: engine architecture ticket 21 put the URI at
    // the drawing verb, so there is no handle to mint and `Repaint::link` carries the address.

    // ── formatting without allocating ─────────────────────────────────────────────
    fn stage(&mut self, args: fmt::Arguments<'_>) -> u16;   // width of what was staged
    fn blit(&mut self, x: i32, y: i32, st: Paint) -> Written;
    fn label(&mut self, x: i32, y: i32, args: fmt::Arguments<'_>, st: Paint) -> Written;

    // ── identity and interaction ──────────────────────────────────────────────────
    #[track_caller] fn id(&mut self) -> Id;
    fn with_key<R>(&mut self, k: u64, f: impl FnOnce(&mut Ctx<'f, '_>) -> R) -> R;
    fn with_id<R>(&mut self, id: Id, f: impl FnOnce(&mut Ctx<'f, '_>) -> R) -> R;
    #[track_caller] fn interact(&mut self, r: Rect, i: Interest) -> Response;
    fn interact_named(&mut self, id: Id, r: Rect, i: Interest) -> Response;
    fn hover_style(&mut self, resp: &Response, r: Rect, role: Role);

    // ── keyboard ──────────────────────────────────────────────────────────────────
    fn next_key(&mut self, id: Id) -> Option<Key>;          // the focused id's queue
    fn decline(&mut self, k: Key);                          // hand it back, in order
    fn key_map(&mut self, map: &KeyMap);                    // declare for the open scope
    fn focus(&mut self, id: Id);
    fn is_focused(&self, id: Id) -> bool;
    fn scope<R>(&mut self, id: Id, k: ScopeKind, f: impl FnOnce(&mut Ctx<'f, '_>) -> R) -> R;

    // ── overlays and scrolling ────────────────────────────────────────────────────
    fn overlay<F>(&mut self, owner: Id, anchor: Rect, o: OverlayOpts, body: F)
        where F: FnMut(&mut Ctx<'f, '_>) + 'f;
    fn modal_barrier_here(&mut self);
    fn scroll_scope<R>(&mut self, id: Id, view: Rect, offset: (i32, i32), max: (i32, i32),
                       f: impl FnOnce(&mut Ctx<'f, '_>) -> R) -> R;
    fn take_into_view(&mut self, id: Id) -> Option<(i32, i32)>;
    fn request_into_view(&mut self, r: Rect);

    // ── frame services ────────────────────────────────────────────────────────────
    fn now(&self) -> Instant;                               // sampled once per frame
    fn theme(&self) -> &'v Theme;
    fn theme_changed(&self) -> bool;                        // true for exactly one frame
    fn caps(&self) -> &'v Caps;
    fn caret(&mut self, x: i32, y: i32);                    // ADR 0005 sink, the theme's shape
    fn caret_with(&mut self, x: i32, y: i32, s: CursorShape);// the engine's Cursor, in full
    fn deadline(&mut self, at: Instant);                    // attributed to the call site
    fn deadline_for(&mut self, id: Id, at: Instant);        // same attribution; id is the census
    fn request_frame(&mut self);                            // == deadline(now), with a site
    fn measured<R>(&mut self, body: impl FnOnce(&mut Ctx<'f, '_>) -> R) -> (R, (u16, u16));
}
```

**Four deviations from `architecture.md` §3, each because the proposal did not compile.** (R01)

- `scratch()` and `events()` **cannot be written**: both hold a borrow of the context alive across a
  verb that needs `&mut` of it. `scratch()` becomes the `stage` / `blit` / `label` triple, which
  returns a width rather than a guard; `events()` becomes `next_key`, which returns one value per
  call, holds no borrow and has no upper bound on how many arrive — which is what ADR 0008 requires.
- `Style` becomes `Paint` throughout. A component names a role and can never construct a paint;
  see §15 and [ADR 0018](../../docs/adr/0018-a-component-names-a-role.md).
- `Response` gains `local`, without which no drag can ask where the pointer is.
- `theme()` and `caps()` are reached through `Env` behind a shared reference whose lifetime is the
  *frame's*, not the `&mut Ctx` borrow's. Through `&self` on `Ctx` they would make
  `cx.text(x, y, cx.theme().glyph(g), st)` an `E0502` and force every theme lookup in the library to
  be bound to a local first. (R01)

**Two further deviations, and both are corrections to *this* document rather than to the proposal.**
They were found by reading this spec against the engine's, which was written after the seam skeleton
R01–R20 were charted against — and the engine spec names that gap itself: *`prototypes/seam.rs` is
four tickets behind* (engine §12). Both of these sit on its delta list.

- **`restyle` takes a descriptor and never a closure.** `f: impl Fn(Paint) -> Paint` is not
  implementable over the shipped engine, three times over: no engine verb applies a per-cell
  function; reading a cell back is removed by name (engine ADR 0023); and an extended cell's style is
  a 52-bit handle into a table the runtime does not have — which is the engine's own ticket-05
  prototype failure, where a closure returned extended styles untouched. The engine's `Restyle`
  cannot be passed through either, because its fields are `Option<Color>` and a `Color` is what
  ADR 0018 forbids a component to name. So the runtime lowers its own role-shaped descriptor:

  ```rust
  pub struct Repaint<'a> {
      pub fg: Option<Role>,  pub bg: Option<Role>,
      pub set: u16,          pub clear: u16,        // attributes to add / remove
      pub ul: Option<Role>,  pub link: Option<Link<'a>>,
  }
  ```

  resolved against the theme inside the runtime and lowered to `engine::Restyle`. **The runtime loses
  nothing by this** — see §15, where the fourth compile outcome becomes structural.
- **`extended(base, handle: u32)` is deleted, and so is the `link(uri) -> Link` that replaced it.**
  The old verb breached two rules at once: it named a raw engine handle, where *no public signature
  names a handle* (engine §12, ADR 0023), and it returned a `Paint` from something that is not a
  `Theme`, which is ADR 0018's hole a second time. Its replacement was a mint, and a mint could not
  be written: minting needed `&mut Screen` and a `Ctx` holds a `View` borrowed from it, so the call
  that minted and the type that draws could not be held at once. It shipped on `Driver` instead, with
  the gap filed against the engine↔runtime seam.

  **Engine architecture ticket 21 closed it by deleting the mint, so the verb is unnecessary rather
  than unblocked.** A hyperlink's URI travels with the drawing verb and the engine interns it into
  the handle space it draws into; `Repaint::link` carries the URI and `lower` hands it straight to
  `engine::Restyle`. There is no handle anywhere, so there is nothing for a newtype to hide: the
  runtime's own `Link` newtype is deleted and the engine's `Link<'a>` — `None | Uri(&str)` — is
  re-exported under the same name. **No `u32` crosses the seam because none exists**, which is
  stronger than the newtype was. `Driver::link` goes with it.

  The price is one lifetime: `Repaint<'a>`, for the same reason `engine::Restyle<'a>` has one, and
  elision covers every call site because the descriptor is built inline.

**`cx.child(area)` is a convention, and that is a known hole.** A component that forgets it paints
over its siblings. Engine ticket 05's *a seam defended by convention is not defended*, reopened one
layer up, and it is recorded rather than closed. (R01)

---

## 4. Modules and crates

The workspace, as `deny.toml` enforces it:

```
vitui-engine        crossterm (no event-stream) + generated UCD tables. Nothing else.
vitui-runtime       vitui-engine and nothing else.
vitui-components    vitui-runtime, plus dependencies case by case. May not name crossterm.
vitui               the facade: engine, runtime, components.
```

Five modules were proposed as crates during the map. **Four were refused and one accepted, and the
accepted one has since been deleted** — so the shipped answer is that none of the five is a crate.
The line that decided was not size: it is whether the module has a consumer that is neither the
runtime nor a component. (R03, R07, R09, R12, R13, R14; architecture issue 24)

| Candidate | Verdict | Why |
|---|---|---|
| `vitui-layout` | refused | needs `vitui-engine` for `Rect` and ADR 0005's exports — three crates where two do the work |
| `vitui-data` | refused | escapes that objection and lands on a worse one: ~150 lines, one consumer, three crates where **one** does the work |
| `vitui-keys` | refused | the strongest case — it names no engine type at all — and loses for `data`'s reason |
| `vitui-overlay` | refused | needs `Rect`, `Style` *and* `Surface`; R03's objection with more force |
| `vitui-signals` | accepted, then **deleted** | its consumer is the application, which is the property the other four lack — and the measurement that justified it did not survive being re-taken |

**`vitui-signals` was built and is gone** (architecture issue 24). It was ~120 lines, because two of
its three parts were already in `vitui-runtime`, and it was kept out of the facade because a facade
shipping one of two shapes R13 measured identical would be picking a winner. Re-measured, the three
drivers of one screen are **41.80 ns apart — 0.21% of the slowest** — with 0 of 24 000 differing
cells, and the signal layer's whole per-frame work is 3.60 ns. That is not a mechanism worth a crate:
it is ergonomics over one hook and a cache over another, and the cache is `data::Memo`, which stays.
The numbers live in [ADR 0020](../../docs/adr/0020-reactivity-lives-above-the-runtime.md) inline,
because the crate that produced them no longer exists. (R13, R14, issue 24)

Two decisions stopped being conventions and became `cargo deny` gates: **ADR 0001**
(`wrappers = ["vitui-engine"]` — four restatements and nothing had ever checked it) and the facade
decision. `[bans] allow` is refused with a number: 38 crates. (R14)

### The module map

| Module | Owns | § |
|---|---|---|
| `ctx` | `Ctx`, `Frame`, `Env`, `Scratch`, `Response`, `Interest` | §3 §6 |
| `id` | `Id`, the id stack, `IdTable`, `with_key`, `named` | §5 |
| `layout` | `Constraint`, `solve`, `Row`/`Col`, rect algebra | §11 |
| `layout::text` | wrap, width, truncate, `wrap_height`, over ADR 0005's exports | §11 |
| `sizing` | the sizing-function contract and the dry-run detector | §12 |
| `focus` | ring, `ScopeKind`, scopes, traversal, the vanish rule | §8 |
| `keys` | `Key`, `Chord`, `Mods`, `Text`, `Binding`, `KeyMap`, sequences | §9 |
| `overlay` | request queue, body queue, placement, layer lifecycle, scrim | §10 |
| `scroll` | `Scrollable`, wheel chaining, `ScrollScope`, `IntoView` | §13 |
| `route` | `Edge`, `edge_of`, `batch_len`, the one key queue behind `Ctx::next_key` | §7 |
| `data` | `Revision`, `Versioned`, `Edit`, `Memo` | §14 |
| `theme` | `Paint`, `Role`, `Roles`, `Repaint`, `Link` (the engine's, re-exported), `Theme`, `Glyph`, `Distinction`, `Density`, tiers | §15 |
| `theme::registry` | `Themes` — the standard set, import, live switching | §15 |
| `anim` | easings, `Tween`, `Spring`, phase and step helpers | §16 |
| `work` | `Slot`, `Drain`, `Task`, `Worker`, `Landing`, `Cancel` | §17 |
| `debug` | id paths, ring dump, wakeup census | **not built** — fog, §21 |

**`route` was missing from this table for four tickets** (architecture issue 21). The module is R11's,
§7 is its chapter and ADR 0016 its decision — *a frame consumes at most one routing edge, and there
are no per-id inboxes* — and it is four hundred lines with its own gates and a `Ctx` verb reaching it.
Nothing about it was undocumented; the table simply never gained the line, and the table is what a
reader checks a module against. It is added rather than the module being deleted to make the count
come out even, which is the standing precedent this repository cites elsewhere.

**`debug` is the other direction and is not a defect**, but it read identically: a row that is *fog*
and a row that is *missing* are indistinguishable to a reader, which is what cost this reading twice.
It is now marked **not built** in as many words. `crate::line::MODULES` is the value behind this
table, gated two-directionally against `lib.rs`, and it carries `Origin::Added { by, why }` for a row
the spec has not caught up with — kept, because it is the only mechanism this crate has for saying
*the code is ahead of the map*.

`architecture.md` §4 lists `app`, `hit`, `input`, `text` and `drag` as separate rows and a `Rows`
trait under `data`. Corrected: there are no per-id inboxes so `input` is not a module of its own,
`hit` is `ctx`'s, `text` is `layout`'s, `drag` is deferred to v2, `Rows` does not exist, and
`Worker::spawn` is not a verb — `Worker` is a resident thread with a one-slot inbox and `work` is a
module row of its own. (R06, R09, R18)

**Three names in this table are the engine's and are re-exported, not owned.** `GlyphSet` is
`vitui_engine::GlyphSet` (engine §10), `Slot<T>` is `vitui_engine::Slot` (engine §12) and
`CursorShape` is the third field of `vitui_engine::Cursor` (engine §5). Earlier drafts of this table
listed the first two as the runtime's, which would have shipped two types with one name across the
seam. The runtime adds nothing to any of the three; `theme` reads `GlyphSet`, `work` wraps `Slot`
beside its own `Drain`, and `ctx` forwards `CursorShape` untouched.

**The rule is wider than those three, and it is a rule rather than a list** (architecture issue 22).
The sentence above was a third true when it was written: `CursorShape` is named by
`Ctx::caret_with` and there was no `pub use vitui_engine::CursorShape` anywhere in the crate, and
sixteen further engine types were named by public signatures and reachable through nothing. A
component-facing crate may not write `vitui_engine::` — components §0's C6, enforced by `deny.toml` —
so a name it can meet in a signature and never spell is a barrier, and `vitui_engine::Rect` was one
in twenty-seven public declarations.

> **Every engine type this crate's public surface names is reachable through this crate, and so is
> every type needed to construct one that the surface accepts.**

The second clause is not decoration. `Driver::post_mouse` takes a `Mouse`, and a `Mouse` carries a
`MouseKind`, a `Buttons` and — through `MouseKind::Down`/`Up`/`Wheel` — a `Button` and a notch; none
of those four was in the inventory at all, reachable or not, which is how a barrier against `Mouse`
survived a check that only ever looked at `Mouse`. **A name a consumer can write but not build is a
barrier wearing a re-export's clothes.**

**Where.** The nine that predate the rule stay in the module that owns the concept — `theme::GlyphSet`,
`theme::Link`, `work::Slot`, `work::Wake`, `work::WakeHandle`, the four `keys` aliases. The twenty
that arrived with it sit at the crate root, which owns none of them: `Rect` alone is named by seven
modules. One arrives aliased — `vitui_engine::Wheel` is a notch direction and `scroll::Wheel` is a
*configuration*, lines and columns per click, so the root re-exports it as `Notch`. Two are here that
the surface does not name (`Restyle`, `Style`); they are re-exported rather than struck from the
inventory, because deleting a row to make a gate come out even is what `route` is the standing
precedent against.

**And it arrived on `Config` itself** (architecture issue 34). `Driver::attach(config: Config, …)`
accepts a `Config`; a `Config` carries a `Clock`, an `Output`, an `Overrides` — which carries a
`WidthSource` — and an `InputConfig`, and none of the five was in the inventory at all. That is the
*weaker* half of the finding and the consequence is the sharper one: `Config` derives `Default`, so
unlike `Mouse` a **value** could always be built and no **field** of it could be reached. The only
headless door above this crate was `Driver::headless`, whose tier is hard-coded to truecolor and
whose sink is a `Vec` moved into the engine and never returned — so a crate on the far side could
neither read a byte the engine wrote nor resolve a driver at any other tier. All five are at the
crate root and the inventory is **thirty-five**. `Driver::headless_into(w, h, sink)` was the smaller
alternative and was refused: it unblocks the caller and leaves the rule open for the next one.

The rule is gated in both directions, and the count is deliberately not asserted:
`crate::line::the_engine_names_on_the_surface_are_all_reachable`. C6 is untouched — a component's
dependency table is still `vitui-runtime` and nothing else.

**And there is a neighbouring rule about *verbs*, which this map rediscovered three times before
writing it down** (architecture issues 23, 30, 35). The rule above is about types a signature names;
this one is about methods on the `Screen` that `Driver` owns privately, where the barrier is not a
name a consumer cannot spell but a capability it cannot reach at all. `Screen::wait` was the first —
without it *no loop could be written* — `Screen::permit_slow` the second, named by a diagnostic the
engine aborts with and by no method an application could call, and `Screen::suspend`/`resume` the
third, which leave an application unable to give its terminal to `$EDITOR` or to answer the `Ctrl+Z`
that raw mode delivers as a key.

> **An engine verb an application genuinely needs is forwarded unchanged, and `Driver` is the door.**

*Unchanged* is the load-bearing word: there is no policy to add on this side in any of the three
cases, and a second one would be a second answer to a question the engine has already answered.
What *is* this crate's is the decision to forward at all — spec §21 leaves who owns the loop open,
and a forward is not a shell. **A verb the engine documents and this crate does not forward is a
finding**, and the reliable way to find one is to write an application: all three of these were
found by `vitui-apps` rather than argued.

**Not here:** widgets, reactivity, terminal I/O, a scene tree.

---

## 5. Identity

**The source is the call site.** `#[track_caller]` on `Ctx::id`, `Ctx::interact` and every
interactive component gives the caller's `file:line:col` — stable across frames, unique per call
site, and free at runtime because it is a `&'static Location`.

```
Id = fnv(parent_id, file_ptr, line, col)          // FNV-1a, and the finalizer was measured off
```

- **A loop needs a key**, because one call site produces N widgets: `cx.with_key(row.id, |cx| …)`.
- **`named()` is a `const` caller-level escape** — free, and available. `architecture.md` §5's *no
  user-supplied string, because it taxes every call site* narrows to *not mandatory*. It had escaped
  duplicate detection until a test failed. (R02)
- **A splitmix64 finalizer was built on a sound argument and measured backwards**: 1.000 against
  1.370 probes per insert, because line and column are dense and FNV maps them injectively. FNV
  ships. (R02)

### Duplicate detection is part of the design, not a debug aid

R01 shipped `Vec::contains` at 22.5 ns a widget and O(n²) growth. It is replaced by a **stamped
open-addressed table** at **1.1 ns a widget**, growing **3.95×** for 4× the widgets against
**13.20×** — chosen with the number, not the argument. The table is stamped rather than cleared,
which is what makes §1 property 6 an obligation. (R02, R15)

Collisions were checked rather than asserted: **0 in 2 880 000 enumerated ids**. The collision that
actually happens is a **merge**, at probability 1 — two widgets that legitimately share a call site.
The policy is that the first claimant wins and the second is inert, which is why §10's overlay id is
handed over rather than derived.

### The container rule, readable from a signature

**A container that returns a rectangle preserves its children's identity; one that takes a closure
renames them, and scoping has nothing to do with it.** This is the second independent force pushing
the shape R01's `E0499` already pushed, and it costs focus and scroll on every child of a
closure-taking container. (R02, joint finding with R03)

It has **exactly one exception, and it is load-bearing**: `Ctx::scope` takes a closure and must
**not** rename its children, or the frame a modal opens renames every field of the form it traps.
`scroll_scope` is the same: it scopes no identity. (R08, R17)

**The id path is the closure tree, not the draw tree.** The id stack is pushed in exactly two places
— `with_key` and `with_id` — and `Ctx::child` pushes nothing, so a rectangle-returning split leaves
its panes siblings at one depth. `architecture.md` §5's sentence assumed containers push a level;
after R01 pushed containers to the rect-returning form, most no longer can. A realistic screen's id
path is **1** at the top and **2** inside a keyed row. (R02 × R03, R06)

### What consumes an id

Focus, hit-testing, interest declaration, overlay ownership and scroll association. **Five, not
six**: a `Memo` is keyed by *where it is stored* and consumes no `Id`. `architecture.md` §5's "all
six, one mechanism" is five, and the id-keyed alternative was built — it pays a full fold every time
a tab is switched away and back. (R09)

### Three rules that survive a widget that stops drawing

Three id-keyed facts are swept when their widget stops drawing — the **pointer grab**, the **press
origin** and the **focus** — by a **281 ns** sweep, without which a stale grab swallows the pointer
for every widget still on screen. The **click record** is a fourth id-keyed cross-frame fact and
deliberately does **not** sweep. (R02, R06)

### An `Id` may never be persisted

The file pointer is an address, so an `Id` differs between two runs of the same binary. Nothing may
write one to disk, send one over a wire, or compare one against a stored value. See
[ADR 0013](../../docs/adr/0013-identity-comes-from-the-call-site.md). (R02)

### One rule for the component library, found by accident

**A `#[track_caller]` wrapper merges the widgets inside its own body.** Visible only because R02's
duplicate detection named it. (R05)

---

## 6. Hit-testing, interest and the pointer

Declaration and hit region are one call, so the runtime pays nothing for widgets that are not drawn.

```rust
let r = cx.interact(area, Interest::CLICK | Interest::HOVER | Interest::SCROLL);
```

`interact` derives the id, appends one entry to the frame's hit index in draw order, raises the
frame's tracking level to `max(current, interest.tracking())`, and returns the `Response`.

### The index carries no geometry

`architecture.md` §6 proposes `(id, layer, rect, interest)`. The shipped entry is **16 bytes against
32** and holds `(id, interest, over, scrollable)`:

- **the rect buys nothing**, because containment is decided during the draw, in the widget's own
  coordinates, from the pointer that travelled down the `Ctx`;
- **`layer` goes too**, because modality is one `Option<usize>` into a `Vec` that is already in draw
  order — an *ordering*, not a membership. `Option` and not `usize`, because `0` meant both "no
  modal" and "a modal over nothing". (R05, R07)

The map's number is **312 interactive regions** on an IDE-shaped 300×80 screen — 1.3% of the cell
ceiling, bounded by visible cells and **identical over 2 000 and 1 000 000 rows** — against **4** on
R01's screen. The spread has a mechanical trigger: a row with two targets cannot be separated by an
entry carrying no geometry, so the author declares per target. The reverse scan over all 312 is
**142.3 ns, 0.14% of the budget**. No quadtree; the engine already answers the layer question. (R05)

### Geometry is needed inside a frame and never across one

This is the rule, and it is not "no geometry". An `over` bit is computed against the pointer *as it
was when that frame drew*, so `architecture.md` §1's one-frame lag was never overlap alone: nothing
at all is hovered on the frame the pointer crosses between two widgets that do not even touch, and a
press arriving where no frame has drawn — **every press at `Buttons` tracking, since that level
sends no motion events** — reaches nothing.

Both close by **awarding at `end`, from the index that has just drawn**:

- **the press**, with `begin`'s answer demoted to a guess that buys in-frame feedback and never
  becomes a wrong click;
- **hover-as-a-style at 37.5 ns**, which also closes the z-order lag on the *first* frame of an
  overlap.

What is left one frame old is hover that changes *content or size*, and that is the whole residue.
See [ADR 0015](../../docs/adr/0015-no-geometry-crosses-a-frame.md). (R05)

### The grab, the wheel and the tracking level

A held grab is **exclusive** — without it a splitter drag lights every button it crosses — the wheel
is withheld while it is held, and a cancelled drag tells the application, which R02's sweep did not.

**One region in 312 raises the whole frame to `Motion`.** The escape is that hover interest comes
from the **theme**: a flat theme leaves the same screen at `Drag` for the same 312 entries. That is
a routing consequence of a theme switch, and §15 owns the mechanism. (R05, R10)

### `Response`

```rust
pub struct Response {
    pub id: Id,               pub rect: Rect,
    pub hovered: bool,        pub pressed: bool,      pub released: bool,
    pub clicked: bool,        pub double_clicked: bool,
    pub long_pressed: bool,   pub dragged: Option<(i32, i32)>,
    pub scrolled: (i32, i32), pub local: Option<(i32, i32)>,
    pub focused: bool,        pub focus_entered: bool, pub focus_left: bool,
    pub changed: bool,        pub mods: Mods,
}
```

- **`mods` is carried, and dropping it was a runtime omission the components map recorded as a
  terminal limit.** The engine puts `mods: Mods` on **every** pointer event (engine §9), and this
  document's `Response` had no field for it — so the components map wrote *"only `Key` carries a
  modifier byte"* three times and filed `Mode::Multi`'s pointer gestures as **keyboard-only until a
  runtime map reopens**. Nothing was ever lost on the wire; it was lost here. One byte beside a
  16-byte hit entry, and it costs the tracking level nothing, because modifiers ride an event the
  frame already receives.

- **`local` is not in §6's list and no drag works without it.** A splitter, a slider and a selection
  drag all need a position rather than a delta, and reconstructing one from `dragged` needs the
  press origin the widget was never given. (R01)
- **`double_clicked` is two comparisons against the *event's* stamp**, not the frame clock. The
  split drain would otherwise make the frame clock measure the drain rather than the user. (R06)
- **`long_pressed` costs a wakeup** — the only field on `Response` that does — because no event
  arrives while a button is held, so the runtime registers a deadline at press time or it never
  fires. It is attributed to the grab holder with the *runtime's* own call site, because blaming a
  component for it is worse than no attribution. (R06, R11)
- **`changed` is a return-value convention and the runtime never sets it.** A component with a value
  sets it before returning, because the runtime does not hold the value and precedence rule 2 says
  it may not. (R06)
- `focus_entered` / `focus_left` arrive **with** the focus, on the frame after the press, and
  validation-on-blur is the case they exist for. (R06)

**Double-click and long-press are the runtime's inference, deliberately.** The engine refuses to
synthesise (ADR 0007); the runtime is exactly where that inference is legitimate, because it is
above the honest layer and its thresholds are configuration. Defaults: click threshold 400 ms, click
slop 1 cell, long press 500 ms.

---

## 7. Event routing

`architecture.md` §6's six steps are **two orders sharing one gate**, and three of the four
mechanisms it names are one mechanism. (R06)

- Steps 1, 2 and 4 route the **pointer**; steps 2, 3, 5 and 6 route the **keyboard**.
- **Step 2's barrier bounds the pointer only.** A focused widget under a modal keeps receiving keys.
  That is §8's `Trap`, and this is its floor.
- **Bubbling, accelerators and "unhandled reach the application" are one queue**, with one byte a
  key, drained at successively outer levels.

### There are no per-id inboxes, and §1's must not be built

The literal version costs **1.17×** routing and at least one allocation a frame against zero, and
its pointer targets come from last frame's index — the stale answer R05 spent a ticket removing.
What ships is one queue and `next_key(id)`, which answers nobody but the focused id. (R06)

`next_key` written the obvious way is O(n²): an 8 000-key paste is **11.75 ms against 53.79 µs**,
fixed with one `usize`. It is a slope, and §20's growth-ratio gate is the detector. (R06)

### A frame consumes at most one routing edge

**"One event a frame" is the wrong rule.** ADR 0008 already says which events may collapse: moves,
wheel clicks and ordinary keys fold freely at **5.0 ns** an event. What costs a frame is a *routing
edge* — an event whose effect the frame reads.

The sharp half is that **which end of the batch an edge may sit at depends on when its effect is
read**. `Down`/`Up` resolve in `end` and therefore **close** a batch. R06 resolved `Tab` in `begin`,
which made it an *opening* edge — and R08 overturned exactly that by declaring the ring during the
draw, at **1.000×–1.009×** of the frame, its own noise floor. So **every routing edge is now a
closing edge**, `[Key(a), Tab]` is one frame, and nothing about focus crosses a frame.

The rule is kept even though only one of its two cases is now reachable, because the rule is what
makes the answer checkable: getting the classification backwards still misroutes a batch, and the
negative case is a gate. Sixteen edges, sixteen frames, **eight clicks, none lost**. (R06, R08)

**The defect was invisible for five tickets because at `Motion` tracking the unsplit batch works.**
R05's escape from `Motion` is what makes it reachable — a second routing consequence of one theme
switch. (R06)

### Bubbling is not a walk of the id path

A container draws before its children, so a pull API gives *capture*, not bubbling. The only moment
an ancestor can ask *after* its children is **after its body**, which only a closure-taking container
has. That was read as a third force on the container shape, pointing the other way from R02's — and
**R14 found it is not a trade at all**: taking a closure is not what renames a child, *scoping
identity* is, and they are independent. `Ctx::scope` already supplies the after-the-body moment, with
the ids inside it equal entry for entry to the ids without it. A bubbling container adds exactly one
id: its own. Bubbling costs **2.99 ns a key a level**. (R06, R14)

**Budget:** routing a realistic batch on a realistic screen is **221 ns, 0.221%**.

---

## 8. Focus

`architecture.md` §8's four things are two mechanisms, and the structural finding is not in §8 at
all. (R08)

### The focus starts as `None`, and nothing but an application changes that

**The runtime seats the focus on nobody, ever** (architecture issue 25). `Frame::focused` begins each
program as `None`, and nothing sets it: not `Ctx::interact`, not `Interest::FOCUS`, not the ring being
built during the draw. `Ctx::next_key` answers nobody but the focused id — *nothing focused, nothing
routed* — so an application with a correct key map, a declared stop and a correct drain loop receives
**nothing at all** until something seats it. Measured over two arms of one program, five `Right`
presses each: **0 against 5.**

This was found by running the first application, and the symptom is worse than silence: a press awards
the focus (§6), so the first click anywhere makes the keyboard start working, and *it only works after
I click on it* sends the user to look at their terminal emulator rather than at the program.

**An application seats it, and `Ctx` carries the question that makes that writable:**

```rust
if cx.focused().is_none() {
    cx.focus(sink);
}
```

`Ctx::focused() -> Option<Id>` is the additive half of issue 25 and the reason the obligation is
satisfiable inside the draw. Before it, `Ctx` exposed only `is_focused(id)` — a question about one id
— and `if !cx.is_focused(sink) { cx.focus(sink) }` **is a different program**: it takes the keyboard
back every frame the user has tabbed away, so `Tab` appears to do nothing. *Which widget starts with
the keyboard* is a statement about the first frame, and asking whether **anything** holds the focus is
what turns it into one. The two forms are gated apart rather than described apart —
`ctx::tests::the_guarded_seating_form_seats_once_and_the_is_focused_form_steals_it_back`.

**Two candidates were refused, and refusing them is the decision.** A runtime that focuses the first
stop when nothing holds the focus removes the failure class and buys it with an *opinion about which
widget is primary*, on a runtime whose whole design is that it has no scene tree and no such opinion —
and *first* means first in **draw order**, a layout accident rather than a statement of importance. It
also collides with the vanish rule: the focus is one of the four id-keyed facts and is swept when its
widget stops drawing, so a modal closing would hand the keyboard to whatever draws first rather than to
whatever had it before. A `Driver` opt-in only moves the argument, because its default is still a
decision. Nothing here is auto-focused, including inside a `Trap`: a modal that opens with nothing
focused inside it traps an empty ring, and seating it is the modal's own to do.

### What is in the ring, and how far one `Tab` crosses, are separate questions

No combination of the five interest bits distinguishes a tab stop from a click target — 76 tree rows
and 152 table cells are clickable and none is a stop. So **`Interest::FOCUS` is a sixth bit**, and it
costs the mouse **nothing**: `tracking() == 0`, and the dense screen stays at `Drag`.

- 312 hit entries → **67** ring entries with `FOCUS` declared;
- `ScopeKind::Group` collapses the menu bar, the toolbar and the tab strip → **43** tab stops;
- 311 / 43 = **7.2×** fewer things a keyboard walkthrough visits.

### Scopes

```rust
pub enum ScopeKind { Group, Trap, Isolated }
```

Three answers, not three degrees. A scope is a **frame-local range over this frame's ring**
(`start`, `end`, `parent`), not a fifth id-keyed cross-frame fact, and it costs **+11.6–18.3 ns**.

- **`Group`** — one tab stop for the whole range. What moves *inside* it is the component's own
  business, reached through `next_key` once the group holds the focus. See §21.
- **`Trap`** — `Tab` cannot leave, wrapping at both ends. **Its keyboard half is `next_key` itself**:
  the verb answers nobody but the focused id, so a trap that takes the focus has taken the keyboard.
  The only exposure is the frame the trap *opens* — widgets behind draw first — and it is bounded to
  **one key**, because §7's drain already ends the batch on the click that opened it.
- **`Isolated`** — `Tab` is not the ring's; a code editor inserts one. **It could not have been a
  `decline`**, because the ring consumes the key first, so it is decided in the drain from the
  previous frame's scopes.

### `dismissible` is deleted

`architecture.md` §8's flag is gone. The runtime may not close a modal — that is a write to
application state it does not hold — so it may not consume `Esc`. `Esc` is an ordinary key on both
sides of the R08/R12 line: a trap's owner receives it through §7's decline queue, and nested traps
are innermost-first for free. **R08 owns who is asked and in what order; §9 owns what a chord
means.** (R08, R12)

### The vanish rule

If the focused id disappears, focus moves to the **nearest surviving entry in the previous frame's
ring order**. This needed the previous frame's ring kept as one more swapped buffer, and it closes
R02's residue by separating two `None`s that two tickets had conflated: **the award's `None` is
intent and stands; the sweep's is absence and becomes this rule.**

Written the obvious way it is **90 002 probes and +32.2 µs a keystroke — 32% of the budget at 600
rows, quadratic from there**, found by typing into a search box. With R02's stamped table it is
**601 probes, 150×**, and a quiet frame pays **0**. (R08)

### The caret

`cx.caret(x, y)` is the sink; the runtime forwards the last write of the frame to `Screen::set_cursor`
(ADR 0005). Nothing focused means no caret. The terminal's own caret is used and **the runtime never
blinks anything itself** — engine ticket 14 measured a software caret at two wakeups a second for as
long as anything has focus.

**The shape has a path, and until this paragraph it did not.** The engine's `Cursor` is
`{ x, y, shape }` and `set_cursor` applies all three, but a sink taking `(x, y)` drops the third and
there is no other door — so a bar caret in an input beside a block caret in a list was inexpressible
above this runtime, while the component library recorded cursor shape as *supported*.
**`cx.caret_with(x, y, shape)`** carries it and `cx.caret(x, y)` keeps the common case at the theme's
default. `CursorShape` is the engine's, re-exported and not redefined — the same rule as `GlyphSet`
and `Slot<T>`, and for the same reason. The last write of the frame still wins, whichever of the two
verbs made it.

**Budget:** at most **+0.31 µs, 0.31%**.

---

## 9. Key maps

`architecture.md` §12 is rewritten rather than annotated: its `Binding` sketch is wrong in its
storage, its layout sentence is a statement about a capability rather than a design preference, and
its `KeyMap` comment hides a borrow. (R12)

### A binding stores its chords inline

`keys: &'static [Chord]` **cannot be written at a call site** — `E0716`, because rvalue promotion
does not cover a `const fn` call — so every binding list would need its own named `const`. That is a
requirement-10 failure in the one demand that exists *because* bindings are declared once and read
twice. Inline chords cost **64 bytes against 40** and buy the config-loaded map.

They also buy the ownership, which was the unexpected half: **a frame cannot borrow a map**, because
the frame outlives every draw and a modal's map is a local (`E0597`). So the bindings are copied —
and writing the copy shows that **matching needs the chords and the action and never the help**,
splitting §12's one registry into two objects with two lifetimes at **44 bytes against 64**,
**45.90 ns** a frame into a cleared-not-freed buffer, **0** allocations.

```rust
pub struct Chord { /* code + Mods; const-constructible */ }
pub struct Binding  { keys: Chords, action: ActionId, help: &'static str }   // declaration
pub struct KeyMap   { /* ordered; first match wins; declared for the open scope */ }
```

### "First match wins" is not an equality

Read as one — R06's stub and §12's literal words — it fires **33 of 33** bindings normally and
**0 of 33** the moment caps lock is on: every binding at once, with nothing on screen. Six of engine
ticket 10's eight modifier bits are *intent* and two are *state*, and matching consults
`Mods::intent()`. R04's silent drift arriving in the keyboard axis. (R12)

### Six modifiers are intent for a key, and five are intent for a character

`Mods::intent()` masks the two locks and compares the other six, and `SHIFT` is the sixth. For a
**named key** that is exactly right — `Shift+Tab` is a different binding from `Tab`, and both
`SIGNIFICANT` and `nav::step`'s *shift passes* rule depend on it. For a **character** it is wrong:
`+`, `?`, `:` and `_` cannot be typed on a US layout without shift, so the modifier is *how the
character was produced* and carries no intent of its own. An author writes the chord for the
character they mean, the user presses the only combination that makes it, and the modifier the
terminal reports makes the two unequal.

So a chord on `On::Typed` with a character code compares `TYPED_INTENT` — the five bits that are not
`SHIFT` — and everything else compares all six. That is what makes `On::Typed` differ from
`On::BaseLayout` in more than which field it reads, and it is what gives one chord four wire
spellings to be right about rather than one:

| what the terminal sends | `code` | `mods` | `text` | what matches |
|---|---|---|---|---|
| the legacy byte | `+` | — | `+` | `typed('+')`, `key('+')` |
| `CSI 43;2u` | `+` | SHIFT | `+` | `typed('+')`, `key('+').shift()` |
| `CSI 61;2;43u` | `=` | SHIFT | `+` | `typed('+')` |
| `CSI 61;2u` | `=` | SHIFT | `=` | `key('=').shift()` |

**The fourth row is not a matching defect and nothing can repair it**: the terminal reported the
unshifted key and sent no associated text, so nothing in the process knows a `+` was produced, and a
runtime that inferred one would be reading a keyboard layout §9 has refused to consult. It is an
alternate the author binds, and it is one alternate rather than the two the workaround cost.

**It is invisible on a legacy terminal**, which reports no modifier for a printable byte, which is
why it shipped — the same shape as the scroll-scope sign and the greedy wrap. Register row 43;
[ADR 0053](../../docs/adr/0053-shift-is-intent-for-a-key-and-not-for-a-character.md). (issue 28)

### Scope resolution

A map is a **range tagged with the open scope**, walked innermost-first — §8's `ScopeRec` one
mechanism over, and §7's decline order. The flat pass is wrong in the expensive direction: `Ctrl+S`
under a `Trap` fires the application's *Save* instead of the modal's, which is **a document written
behind a dialog the user has not confirmed**.

### The base layout is a capability, and the runtime reads it

§12's base-layout sentence is a statement about **kitty flag 4**. Below it, engine ticket 10's own
`code` is *inferred from `text`* — same type, same match, and the binding simply never fires. On a
Cyrillic layout: **33/33 at flag 4**, **28/33** on a terminal that falls back to Latin, **14/33** on
one that does not, with **0 wrong at every tier**. The failure is silence, not misfire, and a
US-layout test suite finds none of it.

Two sharpenings: **the whole loss is one family** — function keys, named keys and `Alt`+named are
escape sequences and are whole at every tier, while bare letters are **1 of 6** — and **which legacy
terminal we are in is unobservable**, both cases being silence on the wire.

The runtime therefore does **nothing**, and matching never consults the terminal's keyboard at all,
because a binding is not rewritten when the terminal is poor any more than a declared interest is.
See [ADR 0021](../../docs/adr/0021-the-runtime-reads-a-capability-and-never-rewrites-a-declaration.md).
(R12)

**What is readable is one boolean, not a tier, and R12's own sentence is why.** R12 wrote
`Caps::key_tier` and it cannot survive two of this project's own rules. ADR 0010 refuses
`KeyboardTier::{Legacy, Disambiguate, Full}` **by name**, with evidence — tmux forwards the `CSI u`
encoding while implementing none of the flag stack, WezTerm ships the protocol off by default — and
the engine says a tier *"would read better and would lie"*. And the three-level reading defeats
itself on the line above: **which legacy terminal we are in is unobservable**, so levels 2 and 3 are
not computable from anything the engine exports. The engine's eight flat booleans separate flag 4
present from flag 4 absent and nothing finer.

So what sits beside `truecolor` is **`Caps::base_layout_reported: bool`**, a direct projection of
`Capabilities::alternate_keys`. The 33 / 28 / 14 measurement stays as a **scene** result rather than
a readable capability, because a test rig can pin what a terminal cannot report.

*(`Caps` is `vitui_engine::Capabilities`, re-exported. This document said `Caps` throughout and never
stated the mapping.)*

### Sequences

`g g` inside one batch needs **no cross-frame state at all**. An unresolved prefix is **map-owned**
rather than id-keyed, so no sweep sees it — but it owes **one wakeup** on a screen §16 had asleep.

**How long an unresolved prefix stands is stated here, because no ticket owned it: the default is
1000 ms**, matching vim's `timeoutlen`, overridable by the application as a parameter and not as a
stored setting. It is the difference between `g g` being usable and `g` being laggy, and 1000 ms is
the value the largest existing corpus of muscle memory was trained against. The cost of the default
is exactly one wakeup per unresolved prefix, which §16's ledger attributes to the map.

### Help

Two strings and a caller-owned buffer — **0** allocations for the whole map. Its honest answer is a
refusal: it names the **base layout**, because no query in any protocol answers what key K would
print, so a help bar shows `Ctrl+S` for a keycap reading `Ы` and no tier fixes it before the first
press. Whether a component may show a binding as unavailable is not the runtime's; see §21.

**Found by pressing `Ctrl+S` into a form and six tickets old:** a text field matched on `code` alone,
so a focused field **typed the letter of every accelerator**. One comparison, and the
components-side half of this seam.

**Budget:** **135.6 ns, 0.136%** for a 33-binding map and a three-key batch.

---

## 10. Overlays

The constraint is inherited: a component cannot open a layer mid-draw (`E0499`). The protocol is
**request during the draw, satisfy after it, answer next frame**, and the two sentences
`architecture.md` §7 spends least space on are the two that decide whether it fits. (R07)

### The owner id is handed over, not derived

`cx.overlay(area, opts, body)` is broken as written. Components are `#[track_caller]` and the
attribute reaches into the body, so the derived id **is** the id the owner already claimed one line
earlier — and §5's collision policy then makes the second claimant inert and the overlay is silently
dropped. The shipped verb takes the owner:

```rust
cx.overlay(owner_id, anchor, OverlayOpts { z: Z::MENU, placement, .. }, |cx| { … });
```

The owner id keys the layer's lifecycle across frames **and roots the overlay's own id stack**. Two
failing tests were the design here: rooted at `ROOT_ID` instead, a body drawn inline and in an
overlay produced **the same four ids**. An overlay is a different *place* for identity, not only for
geometry. See [ADR 0017](../../docs/adr/0017-the-overlay-is-two-phase.md). (R07)

### The `'f` bound is the safety mechanism

A body may capture application data and `Copy` state; it may not capture a local of the base pass,
because that local is gone by the time the body runs. This works **only** with `Ctx<'f, 'v>` (§3),
and the *driver's* signature decides the bound: `impl FnOnce(&mut Ctx)` forces every body to
`'static`, while `&'f mut self` makes it "outlives this frame call". (R07)

`architecture.md` §7's "a body may not capture `&mut` state" **compiles and runs**. It is a
convention, not a bound — R01's convention-shaped hole again — and the reason to keep the convention
is that an outcome reaching its owner through the queue keeps `select` mutating its state in one
place.

### The body queue, placement and the layer lifecycle

- **The body queue**: one `Box` a body, in a `Vec` the frame call owns. **n + 1** allocations a
  frame with **n** overlay bodies — one a body, plus the queue that holds them — and **zero** with
  nothing standing, because an empty `Vec` allocates nothing. Nothing drops a body by hand: the `Box`
  does it, exactly once, whether the pass ran the body or never reached it.
  ~~One chunk, **32 bytes** at high water, **zero** allocations across a hundred frames with a
  dropdown standing. Reset rather than freed; it drops nothing itself, so a body that owns anything is
  dropped by a thunk the request carries beside it.~~ **Superseded** 2026-08-24 by
  [ADR 0034](../../docs/adr/0034-no-unsafe-above-the-engine.md), and struck through rather than
  deleted because R07 reasoned from it: erasing a closure's type into a byte buffer is this crate's
  only `unsafe`, and *no `unsafe` in any shipped crate above the engine* is worth more than *the
  overlay frame allocates nothing*. **The queue cannot keep its capacity across frames and that is
  not an oversight**: a stored body is `+ 'f`, and safe Rust cannot put a `'f`-bounded value inside
  the `Frame` that is borrowed for `'f`. §19 carries the count and the gate under it. (R07, R21)
- **Placement** is integer arithmetic in one order — place, flip, shift, clamp — verified
  **exhaustively**, 28 800 cases at **0.53 ns**. Flipping is conditional on the other side having
  more room, so a tie keeps the side that was asked for; clamping is last and never resizes.
- **Lifecycle by owner id**: a layer requested again is reused. "No reallocation" is true of a
  **move (0)** and false of a **resize (+1 surface)**.
- **A nested overlay's `z` counts from its parent's layer** (`+Z::NESTED`), or a dropdown inside a
  modal draws below the barrier that exists to protect it.
- **The layer census is not §5's sweep and cannot be**: an owner with a closed dropdown is still
  drawing.
- The second pass runs in rounds while bodies request more overlays, **bounded at 16** — a body that
  requests itself for ever is a limit, not a hang.

### A modal, and the number that changed the frame

A modal is an overlay plus a scrim (an operator layer, because a terminal cell has no alpha) plus a
`Trap` scope. An open dropdown is **+1.46 µs, 1.5%** of the frame. A modal, built naively, is
**116.62 µs — 117% of the budget** — and the whole difference is the scrim at **3.41 ns a cell** over
24 000.

That is the wrong answer, and why is the ticket's largest result. Two things fix it, and both are
obligations on everything above:

1. **The base pass draws into its own layer, with a damage-driven composite** — the engine's own
   model, which the prototype had shortcut.
2. **Every component draws its text before its padding.** A component that fills its rectangle and
   then draws into it re-damages what it drew, every frame, for identical output:
   **10 814 of 24 000 cells against 0.**

With both, the prototype measured a modal at **1.19 µs**, and with either missing 87 µs or 117 µs.
(R07)

**R 20 re-measured it against the shipped runtime and it is `+7.96 µs`** — the largest move on the
ledger, 6.7× the prototype's figure, and the reason the *headroom* half of that sentence is now
withdrawn rather than restated. The prototype subtracted its costs from a 33 µs base; the shipped
dense frame is 89.9 µs, so there is no 66.54 µs to leave. What the two obligations buy is unchanged
and is still the point: obligation 1's detector measures **1.98× and is gated at 1.5×**, and
obligation 2 — *padding drawn before text* — prints **103.3% of the budget** and is marked NOT GATED
in `examples/overlay_numbers.rs`'s own report. A dense frame with a modal standing is therefore
98–102 µs against a 100 µs budget: the one place on this map where a realistic frame reaches its
budget, and it is reported rather than hidden. See `crates/vitui-runtime/src/ledger.rs` and §19.

### Two hazards, recorded

- ~~`Mix` clears the extended-style bit, so a shadow crossing a hyperlink deletes it.~~ **Withdrawn.**
  That was measured against the engine's *prototype*, and the engine spec settles it the other way:
  *"`Mix` must go through the handle table, memoised. That is a correctness requirement — the shipped
  prototype deleted a hyperlink under a shadow — and it is cheaper afterwards, not dearer"* (engine
  §5), gated as *a `Mix` over a hyperlinked cell preserves the hyperlink*. The shadow helper
  documents **nothing** here; a rustdoc warning about a defect that cannot occur is worse than
  silence. Kept struck through rather than deleted, because R07 reasoned from it.
- **The scrim's direction assumes a dark theme.** `Mix::shadow` against an engine that already ships
  `Mix::lifting` is the one structurally dark-assuming thing on this map, and `Theme::is_dark` is its
  reader. (R19, R20)

---

## 11. Layout and text measurement

Pure functions over integer rectangles. No solver state, no allocation, no floats in the result.

```rust
pub enum Constraint { Fixed(u16), Min(u16), Max(u16), Percent(u8), Ratio(u16, u16), Weight(u16) }

pub fn solve(avail: u32, spec: &[Constraint], out: &mut [Rect]) -> (usize, Fit);

Row::new().spacing(1).margin(1).split(area, [Fixed(20), Weight(1), Min(10)]) -> [Rect; 3]
Col::new().split_into(area, &constraints, &mut out) -> usize
rect::{inset, shrink, expand, intersect, clamp_to, split_at_h, split_at_v}
Align::TopRight.place(area, w, h) -> Rect;   Stack::center(area, w, h) -> Rect
```

`split` needs no turbofish: the array carries the arity. `Grid` is a convenience over `Row`/`Col`
and is not a fourth primitive. (R03)

### Largest-remainder distribution, and who actually needs it

`Percent` needs it most of all: two 50% lanes on 101 columns round to 50/50 lane by lane, which is
the exact gap §11 names. So **the proportional family is one group over one denominator**, on a
`2^40` scale **rounded up** — because rounded down, three thirds of 100 columns are 99. The
over-claim is bounded below 4×10⁻⁶ of a column at 64 lanes and a 65 535-cell band.

R01's `columns` is replaced on a measurement: **10 957 gaps in 36 503 comparable specs against 0**.
Gap-freeness is proved as contiguity by construction plus 2 406 exhaustive and 200 000 random cases.

**`Min` is only a floor; `Max` costs a fixpoint**, bounded at one round per lane, which §11 presents
as one pass. `split::<N>` needs no scratch parameter — the output buffer carries the fixpoint's
frozen bit, which is why there is no allocation. (R03)

### Clamp and discard, for a stronger reason than the engine's

Every degenerate input here is reachable **by dragging a terminal edge**, so a `Result` would mean
handling an error on every resize. `Fit { slack, overflow, rounds, gaps }` reports what was
discarded instead; the ergonomic `split` drops it. `Fit` is also where a band too small to pay for
its own furniture becomes visible instead of silent. (R03)

### Text measurement is what layout costs

A realistic screen's **32 splits and 119 lanes cost 1.76 µs, 1.8% of the frame**, flat across
terminal sizes, zero allocations. **Measuring 24 wrapped rows costs 4.41 µs — 2.5× the whole
screen.** `architecture.md` §2's deleted measure pass was for *components*; text measurement is
still a real cost and it belongs to whoever calls it.

- Everything is over ADR 0005's exports. **The runtime never ships a second copy of the UCD** —
  that is the exact failure ADR 0005 exists to prevent, and correctness costs **25.5×** against the
  wrong answer.
- The unit is the **grapheme cluster** and the measure is **display columns**.
- `wrap_height(s, w)` has a contract enforced as an equality against `wrap(s, w).count()`, and it
  breaks lines on **ASCII whitespace only**.
- **UAX #14 is a near-miss deliberately not asked of the engine.** Proper line-breaking needs
  break classes, and asking for them means the second copy of the UCD. The honest answer is the
  narrower contract above; the engine map's one open ticket stays one. (R03)

---

## 12. Sizing

Exit (a), and the mechanism is a **sizing function**: a plain function beside a component taking the
same `&data` plus the width or height it is about to be given, returning integers. It takes no draw
context, so it cannot draw, cannot claim an identity and cannot route — which is the whole of what
makes it a function rather than a method on a trait.

**`architecture.md` §2's stated cost of exit (a) is not its real cost.** Sizing a dialog to its
contents is 569 ns, and every survey container that needs sizing needs it for text or for a list,
both arithmetic. The real cost is the drift named in §2, and the detector is the dry run. (R04)

### Why the trait died, and why the dry run died

The trait form was built and **does not die of cost**: same rectangles at **1.03×**, and **0
allocations against 2** once it takes the caller's buffers. It dies of two things.

- **`E0499`** — a fit-container holds every child alive across its measure loop, so two children
  writing through one borrow cannot coexist. It *compiles* on disjoint fields, which is why it is
  easy to miss.
- **`measure` has no answer for a virtualised child.** Honest is `u16::MAX` — a rectangle nothing
  can draw, and still 15× short of 1M rows — and the alternative is the parameter it was handed.

The dry run dies of three numbers and one design fact:

- it is **a second whole frame**: dense screen **2.07×, +34.89 µs**, against **85.79–569.00 ns** for
  the sizing functions — **13.3×** for the same answer;
- it **measures the clip, not the content**: a 1M-row list under an 80-row clip reports 80, and the
  honest question (65 535 rows, all `u16` can express) costs **5.81 ms against 0.96 ns —
  6 051 331×**;
- **a frame has side effects that now happen twice**: a log tail reaches `tick = 2` where the author
  wrote 1, with same-looking output;
- **the measure pass needs its own `Frame`**, or it claims every id twice, doubles the hit index and
  drains §7's key queue — so the measured world has no focus, no hover, no press and no keys, which
  limits what it may be *asked*.

`Ctx::measured` exists and is that mechanism, scoped to the one legitimate use: **the test that
keeps a sizing function honest**. Never the layout mechanism. (R04)

### The one expensive sizing function is not a sizing question

Column auto-fit is **17.60 µs at 1k and 17.69 ms at 1M**. It is §14's derived aggregate, memoised at
**1.73 ns — 10 231 195×**. Exit (a) keeps the fold visible at the call site, where exit (b) would
have hidden it inside a container. (R04)

---

## 13. Scrolling

Two mechanisms, not one, and conflating them is the most expensive mistake available above this
runtime.

| | `scroll_area` | virtualised collection |
|---|---|---|
| what moves | a content offset the application owns | an index range the caller draws |
| cost | ∝ **content** | ∝ **visible window** |
| needs | a content size, or the drawn extent | a length and a row height |
| use for | a form, a panel, a document page | rows, logs, files, processes |

Measured in one round: the list is **0.997–1.003×** from 1k to 1M and the area is **887–889×**, so
the wrong pairing is **7 025–7 076× — 2.03 ms against 0.29 µs, twenty times the frame budget**. The
*right* pairing — a full-screen area with a virtualised list inside it over the same million rows —
is **7.65–7.80 µs**. (R17)

### `Scrollable` is four directions, not two axes

`architecture.md` §13's chaining clause is correct and the declaration under it defeated the clause.
`can_scroll` as a pair of *axis* bools makes a list at its bottom answer "yes, vertically" because it
can still go up: **twenty clicks into a five-row list holding twenty rows moved the enclosing area by
0**. Four direction bits matched on the wheel's sign differ from the axis matcher on **16 of 64**
enumerated cases and never more loosely. Free: **−0.033 to +0.190 µs** over 312 entries.

A component that publishes a scrollable region **owes the pair per axis**, computed from its clamped
offset. (R17)

### Wheel chaining

The innermost scrollable under the pointer that can still move *the way the wheel is going* consumes
the click; otherwise it passes outward. **This is the one pointer channel that cannot be resolved at
`end`**, because the offset is read during the draw by the widget that owns it. The residue is one
click, at each end stop and on an area's first frame, and it is a documented property rather than a
bug to be fixed later. (R17)

### Scroll-into-view is one frame, and it costs §8 a sentence

`architecture.md` §13 records the request during the draw and applies it next frame. It resolves in
**`end`**, from the ring that has just drawn, so the *next* frame is already scrolled.

That needs **the ring to carry a content-coordinate rectangle**, which overturns §8's "the ring
carries no geometry" and nothing beside it — because the rule was never "no geometry" but *geometry
is needed inside a frame and never across one* (§6), and this rect is read at `end`, exactly where
the press award already is. What crosses the frame boundary is **`IntoView`, 16 bytes, naming no
`Rect`**, and that is the ninth doctest pair.

Three decisions were failing tests first:

- it fires **only for a keyboard-driven focus move** — a press already proves the widget was on
  screen, and an unconditional pull is `list`'s old bug;
- `scroll_scope` **scopes no identity** (§5's rule applied);
- `to_content` resets at the area boundary.

**A virtualised collection is one tab stop.** The ring is built from what drew, so a row outside the
window has no entry and no rectangle: mapping a selection index to an offset is the container's job
and a different mechanism. (R17)

### Two axes were one axis written twice

Three defects deep: `scroll_area` used opposite senses nine lines apart, `scroll_area_auto` had no
horizontal branch, and the watermark was marked with the **clipped** width beside an unclipped `y`.
The fix for the third is the near-miss worth carrying into every future frame-local accounting field:
written unconditionally it costs **+7 µs, 7% of the budget, for a field that is off by default**.
**A frame that is not measuring must not maintain the measurement.** (R17)

### Sticky headers, and what stays out

A sticky header is a **rect split**, not a second scroll area — a second area would be a second entry
in the wheel chain, competing with the body it heads. **No momentum and no smooth-scroll physics**: a
terminal delivers discrete wheel clicks, and an animated deceleration would hold the frame clock at
its ceiling after every flick. A configurable lines-per-click is the whole model.

---

## 14. The data contract

**There is no trait.** `Revision`, `Versioned<T>` and `Memo<T>` ship as plain types, and the case
that was supposed to justify a trait is the one that removes it. See
[ADR 0019](../../docs/adr/0019-the-data-contract-is-three-types-and-no-trait.md). (R09)

`architecture.md` §14's two methods both fail when run:

- `len()` is already an argument to every collection and becomes a **second source of truth** the
  moment a view is filtered;
- `revision()` is a promise to *report* a number, and what makes a bump unforgettable is `Drop` on a
  guard — a wrapper, not a trait.

§14's own sentence about the cells — *closures cover every shape a trait would* — is equally true of
the length and of the revision. The only trait shape that really would carry O(1), one handing out a
slice, **cannot be implemented for the struct-of-vectors every R01 number rests on** (`E0308`).

The load-bearing assumption is carried by neither shape. One row drawer accepts indexed, chunked and
linked sources with the same 80 verbs and the same hit index, at **19.77 µs, 320.85 µs and 78.40 ms**
— and the *reasonable-looking* chunked source is already **321% of the whole frame budget**. O(1)
indexed access is prose in both shapes; §20's scene list is what keeps it honest.

### `Revision`

```rust
pub struct Revision(u64);          // from ONE process-global counter
```

- **The counter must be process-global and §14 does not say so**: two per-value counters both at
  revision 1 make a data swap invisible to every memo keyed on them. Found by building the wrong
  one.
- **Two revisions are compared with `==` and never with `<`.** A revision answers *is this
  different*, never *is this newer*. It becomes load-bearing the moment a worker lands (§17), and
  the first `<` between two revisions turns a landing into a silent wrong answer.
- A **content-derived** revision was built and refused at **2.59 ms a frame**.
- Under any crate line the global counter is safe: two copies of the runtime both hand out revision
  1 and can never meet, because `Revision` is nominal (`E0308`). (R09, R14, R18)

### `Versioned<T>` and `Edit`

Reading goes through `Deref` and stays shared; `DerefMut` is deliberately absent, and that absence
is the whole mechanism. The wrapper costs **1.005%** at the call site, and its three prices are each
a gate rather than an oversight: the data is **exclusive while the guard is held** (`E0502`), the
bump happens **on drop rather than on change**, and **one revision covers the whole `T`**.

### `Memo<T>`

A cached result beside the `Revision` it was computed at. The chart's fold is **84.91 µs a frame,
85% of the budget, against a 1.28 ns memo hit — 66 301×**.

**A memo is keyed by where it is stored, consumes no `Id`, and must not join §5's sweep.** The
id-keyed alternative was built and pays a full fold every time a tab is switched away and back.

**§15's promise is narrowed to two thirds.** `Memo` skips the computation and damage skips the
output, but **nothing skips the composition**: a frame in which nothing changed still costs
**23.58 µs, 23.6% of the budget**.

**Budget:** the whole contract adds **≈16 ns to a realistic frame — 0.016%** — and is simultaneously
the largest risk on the map, because the cost it governs is the caller's.

---

## 15. Theme

`architecture.md` §10's three bullets are two decisions and one sentence that is false in the
direction that matters. (R10)

### "No literals" is a type, not a lint

```rust
pub struct Paint(Style);           // field is pub(super); only a Theme hands one out
pub struct Roles([Style; 13]);     // sixteen colours in, the pairing inside the crate
pub enum Role { /* 13 */ }
```

`Paint` costs **1.002×** and **8 bytes against 8**, with **zero** component signature changes across
all 56 gates that existed when it landed.

**The fourth compile outcome is now structural rather than a doctest, and that is stronger than what
R20 shipped.** R20 counted four outcomes and named `restyle` as the one that would have been missed:
six verbs took a style as a parameter and were closed, the seventh took a *function over styles* and
could return one it invented. §3 replaces that closure with the `Repaint` descriptor — because the
closure was not implementable over the engine at all — and the consequence is that **no verb on `Ctx`
returns a `Paint`.** There is no longer a hole for the fourth outcome to guard. It stays in the
corpus as a *regression* case, in the closure's shape, so that reintroducing the verb reintroduces
the gate.

That fourth outcome kills §10's own role list: **a role names a paint, not a colour.** A component
handed `surface` and `on_surface` must pair them, and pairing is `Style::pack` — the literal the same
section forbids three bullets earlier. See
[ADR 0018](../../docs/adr/0018-a-component-names-a-role.md).

**The door was open again before R20 started.** R19's `Theme::imported(&'static [Style; 13], …)` plus
a public `Style::pack` plus `Box::leak` is three lines to an arbitrary `Paint`, and it compiled from
outside the crate. The door is now **`Roles`** — sixteen colours in, the pairing inside the crate —
and it is gated in both directions. (R20)

### `Theme::custom`, and what the gate is actually protecting

R20 stated the property as *no route to an arbitrary `Paint`*, and the components map then spent a
verb this document never defined — `Theme::custom` — in four sections and two gate constructions: a
chart's series colours, a picture's per-cell colour, a QR code, and the sentinel probe that the
components gate register is measured against. Read literally, R20's gate makes all of them fail to
compile.

**The literal reading is wrong about which property matters, and the components map found the hole
by using it.** What ADR 0018 exists to prevent is a component minting a *palette* — because a palette
is what `resolve(tier)` narrows, and a component that forges one bypasses the whole degradation
model, silently, in the direction that always looks fine on the author's terminal. A single colour is
not a palette. A chart's fifth series, a photograph's pixel and a test sentinel are **not roles**, no
theme can promise them, and forcing them through `Roles` would mean inventing thirteen role names
for things that have no semantics.

So the constructor exists and it belongs to the **theme**, not to the component:

```rust
impl Theme { pub fn custom(&self, fg: Rgb, bg: Rgb) -> Paint; }   // reachable only via cx.theme()
```

`&self` is the whole mechanism. A `Paint` still cannot exist without a live `&Theme`, so it is still
impossible to construct one before the theme has resolved, to carry one across a swap, or to build a
`Roles` the registry did not make. **The gate narrows from "no route to an arbitrary `Paint`" to "no
route to an arbitrary `Roles`, and no `Paint` without a live `&Theme`"** — which is what R20's
argument actually establishes, and what its own `Box::leak` negative case actually closed.

Two obligations follow and both land above this crate. A `Paint` from `custom` **carries no tier
guarantee**: `resolve(tier)` cannot narrow what it never saw, so a component using it owes its own
branch on `caps()` — the same obligation ADR 0009 puts on a braille chart, one axis over. And
**`custom` is a per-call cost, not a lookup**: the components map measures 24 000 calls a frame for a
full-screen picture against 4 for a QR code, which is a census a component is expected to publish
rather than a number this crate can bound.

### Quantisation collapses pairs, and the palette bit described the wrong palette

§10's engine claim is **true and was checked against `serializer.rs` rather than assumed**:
quantisation happens at serialise time and no engine change is proposed. Its next four words are
false of every *pair*: **1 / 2 / 13 of 78 role pairs are indistinguishable at True / C256 / C16**,
and the pair that collapses first is `Face` / `FaceHover`, both landing on index 59 at 256 colours.

So §6's escape from `Motion` was resting on a bit describing the **authored** palette rather than the
one the terminal shows: on a 256-colour terminal the frame paid `Motion` — every pointer move a
wakeup and a whole **32.96 µs** frame — for a highlight nobody can see.

`Theme::resolve(tier)` joins the two sentences into one mechanism at **4.16 ns, once**, and drops the
dense screen from tracking **3** to **2** with the same 312 regions and nothing written by the author.
**The runtime does not edit a declared interest** (ADR 0007, one layer up): the component reads
`theme.hover_interest()`, so a component that ignores it is visibly wrong rather than invisibly
corrected. (R10)

### Glyphs are half a lookup and half a branch

A border is a lookup; a chart is a **branch**, because the sub-rows per cell change — **8 / 2 / 1**
for braille / blocks / ASCII — and so does *the number of samples asked of the data*. No table can
carry that; ADR 0009's "a component branches" is the mechanism. `GlyphSet` is ADR 0010's ladder at
full width and `Ord` is the point: a declared axis is downward-closed.

**The lookup half is the theme's table, and until this paragraph this document shipped only the
branch half.** The omission has a count behind it: with nowhere to put a fallback table, **nine
crates of twelve grew a private `mod missing`** — six glyph literals and a `match` on `GlyphSet`,
byte-identical in all nine — and `GlyphSet::` reached **24 occurrences** across four component
crates. Nothing caught it, because **a blank fallback moves every counter the wrong way**: against a
complete table it is no slower, writes 77 fewer cells, marks the same damage and allocates the same.
Absence is cheaper than presence at every level of the stack, and the only detector is the rendered
surface — 135 cells and 77 of 80 rows at Extended, **545 cells and 80 of 80 rows** at ASCII. So the
theme owns the table:

```rust
pub enum Glyph { /* … */ }                     // Glyph::ALL — one row per entry
impl Theme {
    pub fn glyph(&self, g: Glyph) -> &'static str;   // 0.661 ns
    pub fn with_glyphs(self, set: GlyphSet) -> Theme;// declared, never detected
}
```

> **A `Glyph` is a lookup with a spelling at every level, every spelling exactly one cell, and no
> spelling blank.** Anything failing either rule is not a `Glyph` — it is a **branch**, and the
> paragraph above is where a branch belongs.

**Both halves of that rule are the runtime's invariant**, gated over `Glyph::ALL × GlyphSet` as two
counts that are both zero. **The entry list is not the runtime's.** The runtime ships the mechanism
and whatever entries it needs itself; the component library declares the demand set and owns its
size, because the demand is a property of what is drawn and not of what a theme is.

`Theme::glyph` returning `&'static str` **cannot be closed by a type the way `Paint` was**, because
text is legitimately a component's own content and there is nothing to forbid. What replaces the type
is a count, and it belongs to the library rather than here: `GlyphSet::` occurrences in
`vitui-components` == 0.

**`with_glyphs` is the operator's promise arriving, not a capability question.** A repertoire is
declared and never detected (ADR 0010), so nothing in `theme` goes near crossterm and no public
signature moves (ADR 0001).

### `resolve` narrows more than colour

**`hover_distinct` was the first entry of a set, not a special case.** A declared `Distinction` is
narrowed to **one bit at construction**, from the palette as it arrives at the terminal and the
repertoire as it was declared; a component then branches on a bool and names neither axis:

```rust
impl Theme {
    pub fn shows(&self, d: Distinction) -> bool;                  // 0.524 ns, resolved once
    pub fn roles_differ_on_wire(&self, a: Role, b: Role) -> bool;
    pub const fn colours_differ_on_wire(&self, a: Rgb, b: Rgb) -> bool;   // issue 34
}
```

`shows(Hover)` **is** `hover_distinct` and `shows(Fade)` **is** `fade_is_showable`. They stay **two
answers** — 338 themes can show a hover and not a fade — and the rustdoc obligation on the pair
stands: a component reading only the first pays wakeups it cannot show. Deciding per draw instead
costs **8.22 ns against 0.524 — 15.7×** on cell-identical screens, so the choice is cost and
vocabulary, never correctness.

**`roles_differ_on_wire` is `resolve`'s pair count exposed as a question a component may ask**, and
not only as a gate this document runs. At sixteen colours `(Danger, Warn)` and `(Warn, Ok)` are both
**false**, so a traffic light derived from roles alone arrives monochrome — *5 594 cells differing by
style and 0 by cluster* is the signature of a colour-only distinction dying. Its rustdoc carries the
obligation that answers it: **a distinction survives the whole matrix iff it is carried on both
axes**, so a component reading `false` owes a second axis — a glyph, a rule, a position — and never a
darker colour.

**`colours_differ_on_wire` is that same question asked of two colours** (architecture issue 34), and
it is here because the obligation `Theme::custom` states in its own documentation had no verb behind
it. `custom` says the caller *owes a branch on the terminal's own capabilities* — the two colours it
picked may be one colour on the wire — and the only question the surface had was over the thirteen
**roles**, which a `custom` cell is outside by construction. The first screen in the workspace whose
every cell is `custom` reached it through a contrivance, authoring a whole theme per colour pair at
773 ns a call, and filed the friction; the answer is one `const` line over the quantiser key. **The
verb is not by itself the saving**: a caller that writes `Theme::default().resolve(tier)` inside the
call has authored the same thirteen-role theme and measures 811 ns, so the theme has to be hoisted
out to the tier it belongs to — 57 ns once it is. **It
publishes no index and cannot**: *do these two look the same* is a fact a component may act on,
*which index this landed on* is not (ADR 0007). The obligation on a `false` is the one above,
verbatim.

**`Density` stays because it changes rectangles**, not because it is comfortable.

### A theme is data, and data has a price

The obvious `Vec<Style>` palette allocates on the frame a swap is visible, so a theme is **heap-free**:
**0 allocations across two swaps and two full frames**. `Theme` **owns** its thirteen styles — the
`&'static` was the whole of the hot-reload question — at **+0.32 ns** a lookup and **+0.015 µs** a
frame. A swap costs **14.30 ns** and **0** allocations. (R10, R20)

**The theme is a second memo input §14 does not cover.** A data-keyed memo answers a theme swap with
the previous theme's output. The fix is a pair key at **+0.312 ns, 1.247×**, and it works only
because the theme draws from §14's own process-global counter — a per-theme counter reintroduces
exactly the two-values-both-at-1 defect §14 refused.

**Read as "every memo", the pair key is itself a defect**: it costs §14's fold per memo on every swap
for values that are bit-identical afterwards — **221 / 399 µs** at two and four memos. The rule is
narrow: **a memo carries the theme in its key exactly when its value is made of paints**, and §20's
detector checks both directions. (R20)

### The standard set

The spine is **base16**, and the mapping is one `const fn` — **a pick rather than a rename**, because
it walks the ramp by what the quantiser can still separate. Over the whole corpus of **338 schemes**
it collapses **0.08 / 0.26 / 10.87** of 78 pairs at True / C256 / C16 against the stub's 1 / 2 / 13,
and **`Face`/`FaceHover` collapses at C256 in 0 of 338** against 73 under the naive slot-name mapping.

base16 *names* three ramp steps and the corpus does not oblige — **92 / 73 / 61 of 338** schemes put
`base00`/`base01`, `base01`/`base02`, `base02`/`base03` inside one C256 bucket — so four roles that
share `base00` as a background are **picked against what is already spent**, and where a scheme has
nothing left the step is **derived** from its own two ends. That is what ADR 0007 forbids the engine
and permits an offline `const fn`.

Import is `const`, so a theme is `.rodata`: **104 B** the palette, **160 B** with attribution, forty
themes **6.4 KB**. The exception list is **4 schemes of 338**, three of them authored collisions.
(R19)

**Three findings nobody asked for, and all three constrain components:**

- **The C16 pair count is a lower bound.** Through ten real terminal profiles the corpus mean goes
  **10.87 → 16.36**, because five of the ten spell an index twice. Nothing in the process can read the
  operator palette (ADR 0007), so a count taken at sixteen colours is a bound, not a measurement.
- **`fade_is_showable` is a different capability from `hover_distinct`.** 338 themes can show a hover
  state at C256 and **165** can show the animation into it, so a component reading only the first
  pays §16's 19 wakeups for a switch in 173 of 338.
- **Contrast is a check the pair count structurally cannot make.** Pairing by luminance per
  background — rather than nailing `base05` to every face — takes `FaceActive` below 3:1 from
  **218 of 338** to **55**.

**§4 of the proposal inverts twice**: a light theme collapses **fewer** pairs at C16 (3 against 13)
and carries more than twice the low-contrast roles, so every degradation number on this map was taken
on the harder variant. Transparency is opt-in and is a **degraded animation contract**:
`Body → Selection` moves through **32** background values opaque and **2** transparent. (R19)

### Live switching

The registry is a **runtime type held as application state**. It cannot be a field of whatever holds
the frame services — `E0502`, because a picker reads the set inside the frame while the loop writes
it between frames — and it earns its place on one line: a hand-rolled set forgets `resolve(tier)`,
and the failure of forgetting is a **tracking level** rather than a colour. The pick reaches the swap
**one frame later** and cannot do better. A re-detected tier rebuilds **one** theme (16.75 ns), not
the set.

**The swap frame is not the thing to be afraid of.** It is **41.92 µs** against a **33.20 µs** steady
one — a steady frame plus a full repaint, **20 060 damaged cells against 0** — and it needs no
permission to exceed the budget, because a swap frame is a cliff by construction. What *does* exceed
it are two silent mistakes: the pair key read as "every memo" above, and a **theme-dependent fold
over the whole dataset** rather than the visible window, at **6.63 / 13.27 ms** — the map's
O(visible) invariant broken on the one frame nobody was measuring, and **46.97 µs** when bounded.

**Nothing can force the repaint.** Damage is marked at write time, so a partial-drawing application
leaves **19 909 of 24 000** cells carrying the previous palette while marking 20, and marking the
rest would re-send stale bytes. The runtime therefore does what ADR 0007 has the engine do one layer
down and **tells the truth**: `Ctx::theme_changed()`, true for exactly one frame, stale count **0**
when consulted.

In flight across a swap: the grab survives, the tracking level moves **3 → 2** with hit entries
unchanged at 312, and a tween **snaps** — same progress, new endpoints — while a fade that stops
being showable goes from **19 wakeups to 2**. (R20)

---

## 16. Time, animation and deadlines

**There is no animation object.** A `Tween<T>` is **48 bytes**, `Copy` and heap-free; the runtime
holds nothing, and a hundred animating frames of the dense screen with four animations live allocate
**0**. (R11)

### Attribution is the call site, not the id

`architecture.md` §9's "attributes each to the **id** that asked" is wrong twice. The id stack is the
closure tree, so twelve animated chips give **`ROOT_ID` 12 of 12** unkeyed and the *key scope* 0 of
12 keyed — **never the widget**. And an `Id` is a hash of a file pointer §5 forbids persisting.

What works is the **call site**: free from R01's viral `#[track_caller]`, viral in the right
direction because it names the *application's* line rather than the library's, at **0.987×** and 40
bytes against 16. (R11)

### The defect: the runtime had two wakeup sinks and flushed one

`architecture.md` §1's settle flushes the earliest deadline and nothing else, while §7's split-batch
drain sets `repaint`. Unfolded, `[Tab, Key(a)]` leaves a key queued, `settle` returns `None`, the
loop blocks and **the key is never routed**. The pointer case survives only *by accident*, on the
long press's deadline arriving for an unrelated reason, which is why five tickets did not see it.

Folded: **`request_frame()` is `deadline(now)` with a call site**, and the lazy offender joins the
gate it was invisible to. (R11)

### The gate is four integers and a pointer compare

Driven the way §1's loop is actually driven: a quiet dense screen runs **one frame and then blocks
for ever**, a 300 ms fade runs **19**, a caret **1.9 a second**, a spinning component **60.1**. What
separates an animation from a spin is the **streak**, not the total, and `WakeLedger::runaway(n)`
names the offending line.

The ledger is **unconditional** — **1.51–1.61 ns a frame**, of both signs against a whole frame — so
there is no `debug_assertions` guard and no feature flag, and the threshold is a parameter to
`runaway(n)` rather than a stored setting. (R11, R15)

### Colour is a verb on the theme

`impl Lerp for Paint` and `Paint::derive` both fail (`E0423`, `E0624`), so a colour tween is
`Theme::mix(a, b, t)` — **7.58 ns**, zero allocations over 24 000 cells — and a component still names
two roles and never a colour.

**The tier decides whether an animation is one at all.** `Face → FaceHover` has **30 / 1 / 1**
distinct values on the wire at True / C256 / C16, so the same 300 ms fade costs **19 frames at every
tier ungated and 19 / 2 / 2 gated** — §15's `resolve(tier)` removing wakeups by reading a capability,
one axis over.

ADR 0007 arrives in the colour axis: `Color::DEFAULT` has no value to fade towards, so
`Body → Selection` takes **2** background values across a whole fade and **32** foreground ones. **A
pair can be half animatable, and no role list can say so.** (R11)

### The spring is wrong in both halves of §9's phrase

It has **no duration** and needs **four fields** (`start, x0, v0, target`).

- Dropping the velocity on a retarget is **3.17 cells and 141 cells/s** apart one frame later.
- A per-frame integrator is **3.95 / 5.50 cells** off the closed form at a steady / jittered cadence,
  against **0** by construction — so a spring may compute no `dt` at all.
- What lets the screen sleep is a **threshold, not a duration**: half a cell → 32 frames, 1e-6 → 79,
  zero → **never**.

**Drift is the runtime's and compounds with the run**: phase from `(now − started)` is **1.0000**
against an accumulated `dt`'s **0.9222**, and a spinner asked at `now + per` takes **353 steps in
30 s** where an anchored one takes **375**. Every helper is a closed form over `(now, start,
duration)`. (R11)

### The price of motion is the whole story

A fading chip is **114 ns**, a spinner 61, a caret 59 — and **the frame each asks for is 34.17 µs, of
which 99.7% is redrawing a screen that did not change**. That is §14's "nothing skips the
composition" in the time axis. Sixty a second is **2.05 ms/s, 0.2% of a core**. (R11)

**Budget:** `settle` is **3.62 ns, 0.0036%**; the animations are below the ±0.5 µs noise floor.

---

## 17. Async work

Engine ticket 18's primitive is right and `architecture.md` §4's noun for it is wrong: **a slot keeps
the newest *landing* and nothing keeps the newest *question*.** Twenty rows of arrow key down a
directory whose files get smaller end with the pane on **file 0** while the selection is on **19** —
**19 of 20** frames wrong — and it stays wrong for as long as you care to run it, because there is no
further post and therefore no further frame. (R18)

### The generation

What closes it is **8 bytes**: one `u64` minted on the app thread at request time and carried back
beside the answer. It **cannot be a `Revision`** (§14), and that constraint runs both ways.

**The verb that mints it must be idempotent in the question**, and that is not an optimisation:
immediate mode has no mount, so a component can only ask every frame, and **1000 frames on one
selection are 1 job against 1000** — the multiplier is the frame rate. It is also what makes `&self`
safe, which is what keeps precedence rule 2.

### The worker is a noun

`std::thread::spawn` is **9.63–12.74 µs** against **73–83 ns** to put a question in a resident
worker's one-slot inbox — **121–174×** — and a question replaced before anyone looks at it costs
nothing at all. Three mailboxes now hold one thing each, and they share one rule: *the newest
supersedes, because nobody wants the older one.*

**Cancellation is a bracket, not a promise**: 1 unit of 210 when the job has not started, 210 of 210
when it never polls. On real threads it is a crossover — at 1 ms between selections **5.3–7.2×** the
necessary work against **20.0×**, and at 10 ms and 30 ms **20.0× in every column**. *Cancellation
saves nothing while the decode is faster than the user.*

### A pending job is not a fifth cross-frame fact

The registry that would make it one was built and has **no correct setting**: join §5's sweep and a
pane that misses one frame restarts its decode; skip it and the entry outlives the widget. The grab,
the press origin and the focus are *interaction* state; a decode is **commissioned work whose
lifetime is the question's**. So the slot lives in application state and a component holds
`&Task<T>`.

### `Slot` and `Drain`

`Slot<T>` is the **wrong primitive for a directory read**: a delta payload loses **9 000 of 10 000**
rows silently, and a cumulative one copies **50.5×** the elements a queue does. `Drain<T>` ships
beside it as a second accessor, and the pair is documented rather than left to preference.

Requirement 11 holds in the case it did not mention: sixty frames with a job in flight ask for **0**
wakeups against **60** for the polling shape — whose streak of 60 fires §16's runaway detector on a
screen doing nothing — and twenty posts while the app thread is busy cost **1** wake. Free:
**−0.18 µs** on a 300×80 browser, **0** allocations in a steady frame.

**A verification rule falls out, and §20 carries it:** the alloc probe counts a worker thread's
allocations and calls them the frame's. (R18)

---

## 18. Reactivity is above this crate

`architecture.md` §15 holds, **narrowed and strengthened**, and the sentence carrying it is the one
§15 spends least space on. (R13)

> **Re-running the view is the propagation** — not as a convenience of immediate mode, but because a
> frame that draws less than the whole screen *declares* less than the whole screen.

The same screen written three ways — a direct application, a TEA pump with a pure reducer, and a
signal graph — produces **0 of 24 000 differing cells** on a runtime none of them modified, and the
modification was checkable rather than asserted: the reactivity module `#[path]`s the runtime as its
*child*, so the diff over every prototype module is **0 lines**. **The instrument is gone and the
numbers are kept**: architecture issue 24 deleted `vitui-signals`, so ADR 0020 carries the figures in
its own body rather than pointing at a crate.

**The hooks are four names, not three**, because shipping `Slot<T>` without the generation beside it
ships §17's defect:

1. `request_frame()`
2. `Wake::Posted` + `Slot<T>` + `Task<T>`
3. deadlines
4. — and `Memo`, which is what a graph can actually skip; `Computed<T>` turns out to be `Memo` with
   the key filled in. **The 132× this line used to quote is on the wrong denominator**: it is one
   derived value at one fold length (31.8× at 600 rows, 248.1× at 4 800), and at the screen's own 600
   rows the memo saves 44.79 ns against a 19 558 ns frame — **0.23%**.

**`request_frame()` is on `Ctx`, which TEA cannot reach**: `update` runs after the view by definition
and every `Ctx` is gone by then. It costs no runtime change and has two available fixes, both the
*loop's* — and there is no loop to put them in (§21).

**The expected barrier was the type system and there is none.** A stored `Box<dyn Fn(&mut Ctx)>`
compiles, runs and passes. The real barrier is a count: `Frame::begin` rebuilds the hit index, the
ring, the overlay queue and the deadline sink **from the draw**, so a frame that redraws one region
declares **1 hit entry against 312 and 0 tab stops against 43** while **23 993 of 24 000 cells stay
correct** — and the loss is silent for exactly one frame, because routing uses the previous frame's
index. **Fine-grained reactivity here is not expensive; it is a request to revert §6.**

Both layers converge, from opposite directions, on **copy-and-diff**, because a component writes its
state during the draw and the revision guard bumps on drop rather than on change. What makes the diff
free is that component state is small, owned and comparable — **C8, not the runtime** — so a
component shipping state that is not `Copy + PartialEq` narrows what can be built above the runtime
without the runtime changing. That is the components map's to state and cannot be enforced here.

One asymmetry belongs to the shapes rather than to the runtime: TEA shows a new value **one frame
later**. The signal layer measured alone is **3.60 ns**.

See [ADR 0020](../../docs/adr/0020-reactivity-lives-above-the-runtime.md).

---

## 19. The performance budget

Inherited from the engine map and enforced with the runtime's work included: **full-screen 300×80
composition < 1 ms; typical damage-tracked frame < 100 µs; zero allocations during frame composition,
with one stated exception.** A screen with no animation produces zero wakeups.

### The exception, and it is a decision rather than a slip

**A frame with n overlay bodies standing allocates n + 1 times**: one `Box` a body, plus the one `Vec`
that holds them — **2** for a dropdown, **5** for a menu + submenu + modal + tooltip, and **0** for
every frame with nothing standing, because an empty `Vec` allocates nothing. It replaces a bump arena
that allocated nothing and needed `unsafe` to hold a closure with its type erased, and the trade — *no
`unsafe` in any shipped crate above the engine* over *the overlay frame allocates nothing* — is
[ADR 0034](../../docs/adr/0034-no-unsafe-above-the-engine.md). **The queue cannot keep its capacity
across frames**, because a stored body is `+ 'f` and safe Rust cannot put a `'f`-bounded value inside
the `Frame` that is borrowed for `'f`; that is the whole of the `+ 1`.

**The gate is the marginal count**, which is the figure that stays true as the queue grows: one more
overlay standing is exactly one more allocation a frame,
`tests/alloc.rs::one_standing_overlay_costs_one_allocation_a_frame`, with the absolute pair asserted
beside it. Nothing else on the frame path allocates: there is no arena left in this crate to reset,
which is the point. (R21)

### The headroom ledger

**The ledger is `crates/vitui-runtime/src/ledger.rs`, and this section is no longer a copy of it.**
(R 20) Every gated or reported number in the crate has exactly one home there, most importantly the
frame budget itself — which this section states, and which was `const FRAME_NS: f64 = 100_000.0`
written out twelve times across `examples/` until R 20 gave it one.

**What R 20 found is that the base was never the shipped frame.** Every row of the table this
section used to carry came from the architecture map's prototype, and so did the figure they were
subtracted from: *measured directly at 32.85–33.05 µs, giving ≈67 µs*. The shipped dense frame is
**89.75–89.92 µs** — three quiet runs an arm, the two arms indistinguishable — so the headroom is
**1.11×**, not 67 µs, and the old total was arithmetic against a frame that does not exist.

That is not a regression, and the distinction is the reason this section now points at a file
instead of restating it. **R 17 had already measured it** — *"the dense 300×80 frame is 91–94 µs on
both arms and both settings"* — and recorded in the same breath that R14's magnitude *"is not
reproducible here"*. The number was found, written into a ticket, and never reached this page. A
figure with no single home can be discovered twice and fixed neither time.

**The rows do not sum to the frame, and that is the finding rather than a caveat.** 90.3% of the
dense frame is the engine serialising 7 488 damaged cells; this crate's own declared work is
**8.71 µs** of it. The engine's ledger totals its costs against a budget because on the app thread
every row *is* its own work — here the same arithmetic charges the runtime for the engine, so
`ledger::table()` prints both shares and the gate's failure message names them. A bare assertion at
100 µs is a runtime gate that fires when the engine regresses.

Of the twelve cost rows, **seven reproduce with a different number, one holds exactly, one is zero
by construction, and three are still the prototype's** — `Machine::Prototype`, counted by a test so
the number is visible rather than discovered, because the shipped reports print those rows'
absolute cost but not the delta. The row that moved most is a modal, **1.19 µs → 7.96 µs**, which
puts a dense frame with a modal standing at 98–102 µs against a 100 µs budget. The row that held is
the crate line: nothing with ThinLTO.

**The three frames deliberately outside the budget are `Kind::Cliff`, and all three moved:**

| Frame | Then | Now | Why it is allowed, and what catches it |
|---|---|---|---|
| an unchanged frame (R 06) | 23.58 µs | **13.16 µs** | nothing skips the composition; `Ctx::theme_changed()`, entry 26 |
| an animating frame (R 06) | 34.17 µs | **20.28 µs** | **99.92%** of it is redrawing an unchanged screen, up from 99.7% — the conclusion strengthened when the number fell; `anim::WakeLedger`, entry 29 |
| a theme swap (R 05) | 41.92 µs | **not comparable** | the shipped fixture is a full-screen fill rather than the dense IDE screen, so the microsecond columns are about different screens. What survived is the shape of the cliff, restated in **bytes**, where the fixture cannot blur it: a steady frame is **0 on the wire** and a swap frame is **26 272**. Its detector is the memo pair key |

A cliff filed as a cost is a category error: it is a whole frame of another shape, not a marginal
cost on the dense one, and summing two of them charged the runtime 42 µs of work it does not do.
`Kind::Cliff` exists because the ledger made that mistake and its own test refused it.

### The release profile

**`codegen-units = 1` is removed; `lto = "thin"` stays.** R14 measured the setting and left the
decision here. On its prototype's dense screen: monolith **33.0 µs** with `codegen-units = 1`
against **28.7 µs** at 16, while the four-crate build is **28.7** and **28.4** — four crates are four
units whatever the setting says, so it is moot on the shipped shape and costs **4.3 µs** on the one
shape where it is not (a single-crate build, which an application embedding the runtime can easily
produce). It buys nothing this map could find. ThinLTO is what actually earns the inlining back.

**Those four figures are the prototype's and are kept as the record of what was argued, not as a
description of the shipped frame** (R 20). The shipped dense screen is 89.75–89.92 µs on both arms
and both settings, and at that size the boundary question is invisible: 92% of the frame is the
engine serialising damaged cells, so a boundary worth 0.7 µs is 0.8% of it and inside the run-to-run
spread. R 17 measured the line on the small workload instead — 32 splits and 119 lanes of pure
runtime arithmetic — where a cross-crate call that stopped being inlined would be unmissable, and
found **−0.07 µs with ThinLTO and +0.11 µs with LTO off**. Two workloads, not one.

---

## 20. Verification

The register is engine ticket 13's split, with three refinements the runtime had to make. **A gate is
a count, a ratio, an equality or a compile outcome; a timing is a report** — except for one shape,
below. (R15)

1. **A gate is an equality only when the number is a property of the mechanism.** `walk.len() == 43`
   is legitimate: 43 is a property of the dense screen and the scene list is normative. `t_true == 1`
   is not: 1 was a property of R01's stub palette, and R19 replaced the palette. A number that
   belongs to the *data* must be a relation (`t_16 > t_256`, `quadratic / linear >= 100`) or it
   becomes a gate that is edited rather than fixed.
2. **A report may not be load-bearing for a gate.** Five negative cases were kept honest only by a
   `size_of` line in a benchmark, which nobody had decided and `cargo test` compiled by accident.
3. **A positive twin must name the protected item by path.** The runtime's pairs are *behavioural*,
   and a behavioural twin exercises a mechanism without naming it — so renaming the protected item
   leaves every binary green while the must-fail half fails identically, `E0308` having quietly
   become `E0599`. **11 of 57** cases were written that way, including the one R10 calls the case
   that would have been missed.

**The corpus was never being run.** 55 compile outcomes in 15 files behind six `cfg` names, and **0**
evaluated by any CI command. They are now **paired rustdoc doctests** across the real crate line: a
```` ```compile_fail ```` block and an ordinary block naming the same items by path, differing in
exactly the hostile line. Eight were ported and **two refused** — `tab_target` and `hash_loc` would
each cost a `#[doc(hidden)] pub`, so their honest form is a **count** (zero restricted-visibility
items on the component-facing surface) rather than a compile outcome. No `RUSTFLAGS`, no second job,
no `trybuild`.

**The growth ratio is the detector, and the budget is a floor.** R01's quadratic duplicate scan costs
the dense screen **1.19×** and walks straight through a 100 µs gate, while the same defect at
200 → 800 widgets is **9.18–9.32×** against the shipped **3.95–3.97×**. Most quadratics on this map
have been slopes, not cliffs.

**One test command, serialised:** `cargo test --workspace -- --test-threads=1` — **9.36 s** in-binary,
**10.9 s** wall. Serialised because the alloc probe's counter is process-global. Every CI job carries
`timeout-minutes`, because `cargo test` has no per-test timeout and §1 property 6 is a hang rather
than a failure. `criterion` is **not** the harness: it reaches `futures-core` through
plotters → web-sys → js-sys, which `deny.toml` bans by name. `crates/vitui-bench` replaces it.

### The gates

| Gate | Kind | Owner |
|---|---|---|
| a sizing function against its component, over a width sweep | equality per width | R04 — owed by every component that publishes one |
| a keyboard walkthrough visits every tab stop exactly once | count (43, deduped) | R08 |
| the vanish rule's probe count | count + ratio (601 / 90 002, ≥ 100×) | R08 |
| role pairs indistinguishable at each tier | relation per tier | R10, R19 |
| `Face`/`FaceHover` distinct at C256, every shipped theme | universal | R19 |
| every shipped theme names an author | universal | R19 |
| a memo's value moves with the theme ⟺ the theme is in its key | count, both directions | R20 |
| zero allocations in a steady frame | count | all — needs `--test-threads=1` |
| the id table does not grow in a steady frame | count (`grows == 0`) | R02 |
| a worker cannot reach the app thread's half | compile outcome | R18 |
| the negative cases | compile outcome, paired | all |
| dense frame under the budget | timing at the budget | R15 |
| growth ratio, 200 → 800 widgets | ratio (≤ 6×) | R15 |
| a `scroll_area` against a `list` over 1M rows | ratio | R17 |
| bindings reachable per (layout, tier); the lock-key sweep | counts | R12 |

**Reports, committed and gating nothing:** every `examples/*_numbers.rs`, the headroom ledger above,
the comparative suite against ratatui / Textual / notcurses, and the idle-cost measurement.

### The scene list is normative

A scene is removed only by a ticket naming the property it can no longer distinguish.

| Scene | What it decided |
|---|---|
| the dense IDE screen, 300×80 | 312 interactive regions, 43 tab stops, ~33 µs a frame |
| a 1M-row virtualised list | the data-volume invariant, 15 876×, flat 0.987× from 1k to 1M |
| 200 → 800 keyed widgets | duplicate detection's slope: 3.95× against 9.26× |
| a modal over the dense screen | the scrim: 116.62 → 1.19 µs |
| a chart folding 2 000 rows a frame | the memo: 84.91 µs against 1.28 ns |
| a search box filtering 600 keyed rows | the vanish rule: 90 002 probes against 601 |
| an 8 000-key paste | `next_key`'s O(n²): 11.75 ms against 53.79 µs |
| a 300 ms fade at each colour tier | 19 / 2 / 2 wakeups gated, 19 at every tier ungated |
| a list at its end inside a scroll area | wheel chaining: 0 clicks reach the area under an axis matcher |
| three nested scroll areas | the chain stops at the innermost that can move |
| a 40-field form in a 6-row viewport | scroll-into-view: 1 frame resolved in `end`, 2 in the draw |
| 1M rows through a `scroll_area` and through a `list` | C21 as a ratio: 887–889× against 0.997–1.003× |
| a file browser with a preview pane, 20 selections | staleness: 19 of 20 frames wrong |
| a decode superseded before it finishes | cancel is a bracket: 1 of 210 against 210 of 210 |
| a directory read streamed in 100 batches | `Slot` of deltas loses 9 000 of 10 000 rows |
| sixty frames with a job in flight | 0 wakeups against 60 polled |
| an imported theme flat at the tier under test | `hover_interest` is `NONE`, the level is `Drag` |
| a Cyrillic layout at three key tiers | 33 / 28 / 14 of 33 reachable, 0 wrong |
| a chunked data source | 320.85 µs, 321% of the budget — the trap through the storage |
| a theme swap over the dense screen | 41.92 µs, 20 060 damaged cells against 0 |

**A gate has an attribution window.** `vitui-alloc-probe` counts `alloc` calls, not `alloc` calls *by
the app thread*, so a window opened around a frame while a worker is decoding attributes the worker's
growth to the frame. **An allocation gate with a background job in it runs on a deterministic
spawner, or joins before it measures** — and the same reasoning is why §17's behavioural gates run on
a queueing spawner whose completion order the test chooses: handing the test that order turns a race
into a count. (R18)

---

## 21. What this spec does not decide

Each of these is fog with a named shape, not an oversight.

- **Whether the runtime ships an application shell (`Ui::run`) or only the pieces.** The wakeup gate
  is only meaningful against a loop, and the loop is the piece nothing owns: §16's numbers were
  produced against a loop written in a test harness. §18 sharpens it — `request_frame()` is on `Ctx`,
  so a reactivity layer whose update runs after the frame body cannot reach it, and the workaround is
  the *loop's*. R14 adds the one fact the split contributes: the rig both halves of its comparison
  run on names nothing that is not `pub`, so the loop can live in an application, in the runtime or
  in a crate of its own without any of the three needing a new item. **Issue 23 makes the first of
  those three writable**: `Driver::wait` forwards `Screen::wait` and `Driver::wake` hands out the
  handle `attach` used to drop, because until then no crate above the engine could park at all and
  `Worker::hire` was public with nothing able to call it. It ships no shell and decides nothing here.
- **Drag & drop payloads across components.** Deferred to v2 deliberately.
- **A debug inspector's surface** beyond "it prints id paths". Three constraints are known: the ring
  and its stops are readable (`tab_walk`, `stop_ids`) and nothing may write them; and **the role a
  cell was painted with is not recoverable from the cell**, because a `Paint` is a `Style` and the
  role is gone by then — an inspector that wants it needs the theme to record, which is a cost on
  every verb.
- **Whether a component may ship state that is not `Copy + PartialEq`.** A components-map obligation
  (C8's own ticket); it cannot be enforced from here, but a component that breaks it narrows what can
  be built above the runtime.
- **What a `Group` moves with internally** — arrows, `Home`/`End`, type-ahead. A component contract
  rather than a runtime one; it graduates on the components map.
- **Whether a help bar may show a binding as unavailable.** Below kitty flag 4 the runtime cannot
  know whether a chord is reachable, only that it might not be, so the honest surface is the tier and
  not a per-binding flag. Whether a component should say so at all is the components map's.
- **The engine↔runtime crate boundary, built rather than counted.** Every ticket from R01 on compiled
  the engine *inside* the runtime's binary. What stands in for the compile is a count — **0
  `pub(crate)`, `pub(super)` or `pub(in …)` items** in the frozen seam — which says the leak class
  cannot exist there and is not the same as having built it. Cheap for whoever writes the first real
  `vitui-runtime`.
- **Multi-window / multi-screen.** The engine is alt-screen only, so this is not a runtime question
  yet.

Out of scope entirely: anything that touches the terminal (the engine's map), widgets (the components
map), and a stylesheet language, bidi and screen-reader accessibility (non-goals, with reasons
recorded in the components map's requirements).

---

## Appendix A — ADRs this spec rests on

Inherited from the engine map: **0001** crossterm behind a seam · **0002** layout lives outside the
engine (amended by R14) · **0005** the engine owns the caret and exports its text tables · **0007**
the input model is honest about the terminal · **0008** intent is never dropped, position is ·
**0009** the engine degrades presentation, never content · **0010** detected axes are flat, declared
axes are ordered · **0011** the handle tables belong to the engine.

Added by this map:

| ADR | Decision | § |
|---|---|---|
| [0012](../../docs/adr/0012-frame-state-is-rebuilt-from-the-draw.md) | Frame state is rebuilt from the draw | §0 §18 |
| [0013](../../docs/adr/0013-identity-comes-from-the-call-site.md) | Identity comes from the call site and may never be persisted | §5 |
| [0014](../../docs/adr/0014-no-measure-pass.md) | There is no measure pass | §2 §12 |
| [0015](../../docs/adr/0015-no-geometry-crosses-a-frame.md) | No geometry crosses a frame | §6 §13 |
| [0016](../../docs/adr/0016-a-frame-consumes-one-routing-edge.md) | A frame consumes at most one routing edge | §7 |
| [0017](../../docs/adr/0017-the-overlay-is-two-phase.md) | The overlay is two-phase and its owner id is handed over | §10 |
| [0018](../../docs/adr/0018-a-component-names-a-role.md) | A component names a role and can never construct a paint | §15 |
| [0019](../../docs/adr/0019-the-data-contract-is-three-types-and-no-trait.md) | The data contract is three types and no trait | §14 |
| [0020](../../docs/adr/0020-reactivity-lives-above-the-runtime.md) | Reactivity lives above the runtime | §18 |
| [0021](../../docs/adr/0021-the-runtime-reads-a-capability-and-never-rewrites-a-declaration.md) | The runtime reads a capability and never rewrites a declaration | §9 §15 |

## Appendix B — where each decision was taken

| § | Ticket |
|---|---|
| §1 §2 §3 | [R01 signature and `Ctx`](issues/01-component-signature-and-ctx.md), [R04 intrinsic sizing](issues/04-intrinsic-sizing.md) |
| §5 | [R02 identity](issues/02-identity.md) |
| §6 | [R05 hit index](issues/05-hit-index.md) |
| §7 | [R06 routing](issues/06-routing.md) |
| §8 | [R08 focus](issues/08-focus.md) |
| §9 | [R12 key maps](issues/12-key-maps.md) |
| §10 | [R07 overlays](issues/07-overlays.md) |
| §11 | [R03 layout](issues/03-layout.md) |
| §12 | [R04 intrinsic sizing](issues/04-intrinsic-sizing.md) |
| §13 | [R17 scrolling](issues/17-scrolling.md) |
| §14 | [R09 data contract](issues/09-data-contract.md) |
| §15 | [R10 theme](issues/10-theme.md), [R19 standard set](issues/19-standard-theme-set.md), [R20 live switching](issues/20-live-theme-switching.md) |
| §16 | [R11 deadlines and animation](issues/11-deadlines-and-animation.md) |
| §17 | [R18 async work](issues/18-async-work.md) |
| §18 | [R13 reactivity stays out](issues/13-reactivity-stays-out.md) |
| §4 §19 | [R14 crate split](issues/14-crate-split.md) |
| §20 | [R15 verification strategy](issues/15-verification-strategy.md) |
