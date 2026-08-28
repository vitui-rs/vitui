# The component gallery: one runnable demo app carrying the whole v1 set, themes on `t`

**Resolved 2026-08-28** by `.scratch/vitui-components-impl/issues/39-o2-the-gallery.md`, ADR 0046. See
the Progress entry at the bottom for what shipped, what was found and the three figures that did not
reproduce.

## Goal

A single binary the user runs to see every shipped component at once, drive it from the
keyboard, and switch themes live with `t`. It is the library's front door: the thing that
answers "what does this look like and does it work" without reading a line of code.

**Still not built, and the reason has changed.** When this ticket was written the library was 17
lines of stubs and everything real sat in `proto-*` crates on unmerged branches; that paragraph is
gone because all three crates now ship — `vitui-components` is **28 of 29** rows of the freeze
built, on a finished engine and runtime. What is missing is not the library, it is *one screen*:
`crates/vitui-apps/examples/` holds **14 applications and every one of them is a single component's**
— `counter`, `triage`, `latency`, `ledger`, `explorer`, `reader`, `settings`, `compose`, `console`,
`theatre`, `browse`, `mixer`, `vitals`, `roster`. A component ticket ships one app, which is why
there are fourteen and no gallery. The prototype gallery is still on `prototype/gallery` and still
unmerged, and it still stands on the prototype lineage rather than on the shipped crates.

**The implementation slice exists and is unblocked**:
[`.scratch/vitui-components-impl/issues/39-o2-the-gallery.md`](../.scratch/vitui-components-impl/issues/39-o2-the-gallery.md).
Its `Blocked by:` line is 01, 10, 12, 15, 17, 19, 22, 24, 26, 28, 30, 32, 33, 34, 35 and runtime 04
— all resolved — so it is at the frontier of that backlog. **This ticket stays open until 39
closes**, and 39 says so from its own side.

## Acceptance criteria

- One binary, one screen, reachable with a single `cargo run`. Not a crate per component.
- Every component in the frozen v1 inventory appears in it, and the gallery fails to
  build — or fails a test — when the inventory gains an entry the gallery does not show.
  An inventory and a gallery that drift apart are worse than no gallery.
  **This gate is already written and is waiting for its input.** C10's freeze shipped as
  `vitui_components::INVENTORY`, twenty-nine rows a test iterates, and the drift gate is the two
  halves of **O2** — `obligations::o2_nothing_shown_is_absent_from_the_freeze(panels)` and
  `o2_everything_built_has_a_panel(panels)`. The panel list is not missing, it is a **declared empty
  seam**: `obligations::PANELS` is `&[]` with a note saying it stops being empty when ticket 39
  builds the gallery. So the plug-in point is named, and both queries are `Unmet` and watched
  panicking until it is filled. Note which way each fails: an empty gallery is `Unmet`
  over zero rather than `Met` over zero, because `Verdict::of` refuses an empty population in the
  constructor — the vacuous green this criterion exists to prevent is already refused by the
  machine, and what is missing is the screen.
  **Twenty-eight or twenty-nine is a live question, not a rounding.** `spinner` is the one unbuilt
  row and its mechanism is *a component that owns a clock* (components 42). Ticket 39's title says
  twenty-nine panels and its own criterion says one per `built` row, which is 28 today; O2's second
  equality is over `built`, so it moves on its own the day `spinner` ships. Do not paper over the
  difference — O1 and O3 both hit this and both chose `built` deliberately.
- **`t` cycles the theme live**, without restarting and without a visible rebuild pause.
- Keyboard-only navigation across the whole gallery. C11's obligations shipped as **O1–O5** in
  `vitui_components::obligations`; the key-only walkthrough is register rows 34 and 88 and is
  already green, and **O4** — the keyboard contract, thirteen components as declared data — closed
  on 2026-08-28. So the gallery inherits a keyboard that is checked rather than one it must
  establish. Two facts from O4 that decide how a gallery navigates: a focused `field` consumes every
  text-bearing key and a focused `collection` eats one into its type-ahead buffer, so **the walk keys
  and the quit key cannot be printable characters** — `Ctrl+Q` is the established spelling — and
  `Tab`/`BackTab` at all five modifier states belong to the runtime's focus walk, not to any
  component.
- The performance budget holds *in the gallery*, not only in isolated harnesses: a
  full-screen 300×80 composition under 1 ms, a typical damage-tracked frame under 100 µs,
  zero allocations during frame composition.
- Runs at all three repertoires and all three colour depths, so the degradation matrix is
  visible rather than asserted. C09's mechanism shipped as **ADR 0032** and components spec §16 —
  degradation is resolved at construction, one bit per `Distinction`, never branched at the draw.
  **C09's own figures do not describe the shipped palette**: the roles-collapsed and
  distinctions-lost columns come out `0 / 0 / 18` and `0 / 1 / 3` against the remembered `1 / 2 / 13
  of 78` and `0 / 1 / 2 of 10`, and the palette was deliberately *not* swapped to make the old
  numbers reappear. Read the current ones out of the runtime's `theme_numbers` and `glyph_numbers`
  reports rather than from this ticket; the gallery's job is to make the matrix visible to a human,
  which is the one thing neither report does.

## Why `t` is not a cosmetic requirement

Live theme switching is the sharpest available test of two decisions already made, which is
a reason to keep it rather than simplify it away:

- **Degradation is resolved, never branched** — one bit per `Distinction`, narrowed at
  construction (ADR 0032, components spec §16). A key that re-resolves the whole theme on a live
  frame is the test of whether "at construction" is cheap enough to redo, or whether it quietly
  means "at startup".
- **A memo's key is every input** — components spec §10, and it has gone on being forced since this
  ticket counted five: components 24 found a `field`'s wrap index keyed without the width it was
  built at, and components 31 found the rule arriving *through the door that is not a gesture* — a
  preview pane keyed by list **position** rather than by file is wrong on 100 frames of 100 after one
  re-sort, with no keystroke involved and nothing to wake a frame that would correct it. A theme
  switch that leaves a memo keyed without the theme is that defect in its most visible possible
  form, and the gallery is where it shows up as a stale-looking screen rather than as a number.

**Both predictions were confirmed on the prototype, and each is now its own ticket.** This section
argued `t` would find something; measured over the twelve-panel assembly it did:

- **six panels keep the old palette permanently** after a swap, and the rule that would prevent it
  has no caller — components 41;
- **9 956 cells of 53 280 (18.7%), in six panels of twelve**, are written by nobody on a steady
  frame — the second half of the partition rule — components 40.

Both are measured on the screen this ticket delivers, which is the argument for building it as a
gate rather than as a demo: every defect the components map found was invisible on the screen of the
ticket that owned the mechanism and visible where the components meet.

## Blocked by

**Nothing, as of 2026-08-28.** All four original blockers are cleared, and they are kept here
because what cleared each one changed what the gallery has to do:

- ~~**The engine does not exist.**~~ All three crates ship. The engine is implementation-complete
  (26 tickets, plus a 28th verification instrument production readiness added), the runtime is
  complete (21 tickets), and `vitui-components` is 28 of 29 rows built.
- ~~**Input and raw mode are unbuilt.**~~ Fourteen applications enter raw mode and read the
  keyboard, and crossterm is still invisible behind the seam (ADR 0001). What `t` needed turned out
  not to be raw mode but a loop that reads the unhandled-key window **after** its own frame — all
  five loops read it one frame too early and a source scan now keeps them honest.
- ~~**C10** — the v1 inventory must be frozen.~~ Frozen as `INVENTORY`, twenty-nine rows, and it is
  a value a test iterates rather than a table in a document. Building it contradicted four figures
  in the closed map, all four asserted as measured rather than bent to fit.
- ~~**C11**~~ — shipped as O1–O5. The gallery is no longer the *host* for those obligations; it is
  the subject of exactly one, **O2**, and the two equalities are written and waiting for a panel
  list.

What remains is scheduling, not blocking: the work is
[`.scratch/vitui-components-impl/issues/39-o2-the-gallery.md`](../.scratch/vitui-components-impl/issues/39-o2-the-gallery.md),
whose own blockers are all resolved, and components 40 and 41 are measured on the screen it builds.

## Non-goals

- Not a replacement for C11's gates. A gallery a human looks at is not a test; every defect
  found on the components map so far was invisible to a human and to every counting gate,
  and was caught only by an equality against a reference.
- Not a documentation site, not a screenshot generator.

## Progress

- Recorded 2026-08-19 from a direct user requirement: a demo app hosting the whole component
  library, with themes switching on `t`.
- 2026-08-19: a **prototype gallery** was built on `prototype/gallery` (`3fb4dba`) at the
  user's direction. `crates/proto-gallery-app`, ten panels, `n`/`p` to walk, `t`/`T` over 14
  themes, `q` to quit. Verified independently: builds, 817 tests pass at 768 distinct names,
  all ten panels non-blank, and the terminal is restored *before* a panic prints (checked
  under a real pty).
- 2026-08-19: **this ticket is NOT satisfied and stays open.** The gallery stands on the prototype
  lineage, not on the shipped library: `vitui-engine/src/lib.rs` is a stub and `seam.rs` is
  `todo!()` throughout *as of this entry* — both were built out later, see 2026-08-28. The
  acceptance criteria that remain unmet are the library itself, C10's full 29-row inventory (the
  gallery carries the built tier only), the drift gate between inventory and gallery, the
  performance budget measured in-gallery, and the three-repertoire x three-depth matrix.
- 2026-08-19: both obligations O1 and O2 are mandatory, and the frozen inventory is 29 panels, not
  the ten built here. (O1 closed on its own, components 36; O2 is still this ticket's.)
- 2026-08-28 — **premises refreshed; the deliverable is unchanged and the ticket stays open.** The
  Goal's second paragraph, all four blockers and the C09/C10/C11 citations described a workspace that
  no longer exists — 17 lines of stubs, eight unmerged `proto-*` crates, an unfrozen inventory. What
  replaced each is recorded above. The acceptance criteria themselves were **not** relaxed: still one
  binary, still one screen, still the whole freeze, still `t` live, still the budget measured
  in-gallery, still three repertoires by three depths.
  Three things are genuinely different from the ticket as written. **The drift gate no longer has to
  be invented** — it is O2's two equalities, already written against a panel list nobody supplies,
  and `Verdict::of` already refuses the vacuous green over an empty gallery. **The gallery is a gate
  with two defects already measured on it**, six panels keeping a stale palette and 9 956 unwritten
  cells of 53 280, now components 41 and 40. And **the panel count is 28 or 29 depending on
  `spinner`**, which O2's second equality handles by being written over `built`.
  Not resolved, and deliberately so: `tickets/002` is the requirement and components 39 is the
  implementation slice, which is unblocked and at that backlog's frontier.

- 2026-08-28 — **resolved.** `cargo run -p vitui-apps --example gallery` is the binary; the screen,
  the panel table and the twenty-eight drawings are `vitui_components::gallery`'s, because spec §21
  names two defects to be measured *on the assembled gallery* and both are components tickets whose
  gate is `cargo test` — *a screen only an application can draw is a screen no gate can measure*.
  O2 is green in both halves; components register 218 → **223 rows, 208 evaluated**. ADR 0046.
  Every acceptance criterion above is met, and three of them are met differently from how they were
  written:
  - **Twenty-eight panels, not twenty-nine.** `spinner` is the one unbuilt row and O2's second
    equality is over `built`, so the population moves on its own the day it ships. The ticket already
    said not to paper over the difference.
  - **The wrapper list did not have to move.** The gallery names no crossterm at all —
    `Driver::attach` owns raw mode, the alternate screen, the input and the restoration — so
    `deny.toml`'s `{ name = "crossterm", wrappers = ["vitui-engine"] }` is unchanged, and a gate
    asserts that it still reads exactly that because the gallery is the reason.
  - **`t` is bound beside `Ctrl+T`, and that is O4's finding rather than a compromise.** A focused
    `field` consumes every text-bearing key; this screen has a `field`, a `form`, three collections
    and a picker on it, all one `Tab` away.

  **The three figures that did not reproduce**, all asserted as measured:
  - *`Danger`, `Warn` and `Ok` all quantise to bright white at sixteen colours* is true of **8 of the
    14** shipped schemes and of **14 of 14** at `ColorDepth::None` — the right claim about the wrong
    rung.
  - *`Theme::…resolve()` including all ten distinctions is 291 ns* — what `t` presses is ~**70 ns**,
    and `with_glyphs` before `resolve` is ~**550 ns**, because a declared repertoire is a real input
    to the ten distinction bits.
  - The degradation matrix's colour axis **is not observable on the screen at all**: `Theme::resolve`
    returns the same `Paint` for all thirteen roles at all four depths, because quantisation is the
    engine's and happens before the mirror. The repertoire axis is 58 / 82 / 110 clusters read off
    the surface; the colour axis is 66 / 9 / 0 collapsed role pairs and 3 / 2 / 0 distinctions lost,
    read off the theme. The paint count reads ten in all nine cells and is printed beside them.

  **The budget holds in the gallery**: ~390 µs at 300×80 against the 1 ms full-screen class, ~79 µs at
  100×30 against the 100 µs typical frame, `writes == distinct` and `merges == 0` at every size, and
  **zero allocations on every page** — which found two preview drawers spelling their row labels with
  `format!`, 4 a frame on page three and zero on every other page.

  **What is deliberately not done**: register rows 7 and 8 stay red. This ticket's own two predictions
  are components 40 and 41, and the screen they are measured on now exists with both numbers printed
  — 10 252 unwritten cells of 24 000 at 300×80, and a swap that keeps 0 of 13 748 paints, 64.6% of
  clusters under a rung change and 99.9% under a tier change.
