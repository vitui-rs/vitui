# The component gallery: one runnable demo app carrying the whole v1 set, themes on `t`

## Goal

A single binary the user runs to see every shipped component at once, drive it from the
keyboard, and switch themes live with `t`. It is the library's front door: the thing that
answers "what does this look like and does it work" without reading a line of code.

This does not exist and is not currently produced by anything on the components map. The
sixteen map tickets decide *architecture*; none of them delivers a runnable artifact. The
eight prototype crates built so far (`proto-c02-app` … `proto-c09-app`, each on its own
`prototype/*` branch) are **batch measurement harnesses** — they render a surface, print
counts and exit. None has an event loop, none enters raw mode, and none is merged.

## Acceptance criteria

- One binary, one screen, reachable with a single `cargo run`. Not a crate per component.
- Every component in C10's frozen v1 inventory appears in it, and the gallery fails to
  build — or fails a test — when the inventory gains an entry the gallery does not show.
  An inventory and a gallery that drift apart are worse than no gallery.
- **`t` cycles the theme live**, without restarting and without a visible rebuild pause.
- Keyboard-only navigation across the whole gallery, since key-only walkthroughs are
  already a verification obligation on C11.
- The performance budget holds *in the gallery*, not only in isolated harnesses: a
  full-screen 300×80 composition under 1 ms, a typical damage-tracked frame under 100 µs,
  zero allocations during frame composition.
- Runs at all three repertoires and all three colour depths, so the degradation matrix is
  visible rather than asserted. C09 measured that `Danger`, `Warn` and `Ok` all quantise to
  bright white at the bottom tier — the gallery is where that becomes obvious to a human.

## Why `t` is not a cosmetic requirement

Live theme switching is the sharpest available test of two decisions already made, which is
a reason to keep it rather than simplify it away:

- **C09 decided degradation is resolved, never branched** — one bit per `Distinction`,
  narrowed at construction. A key that re-resolves the whole theme on a live frame is the
  test of whether "at construction" is cheap enough to redo, or whether it quietly means
  "at startup".
- **A memo's key is every input** — forced five times independently now (R10's theme, C04's
  sort, C05's expansion, C06's wrap, and R20's rule extended to paints *or glyphs*). A
  theme switch that leaves a memo keyed without the theme is that defect in its most
  visible possible form, and the gallery is where it shows up as a stale-looking screen
  rather than as a number.

## Blocked by

- **The engine does not exist.** `vitui-engine`, `vitui-runtime` and `vitui-components`
  hold 17 lines of Rust between them; everything real is in `proto-*` crates on unmerged
  branches. The gallery needs the shipped library, not the prototypes.
- **Input and raw mode are unbuilt.** crossterm is used for input and terminal mode only
  and must stay invisible behind the engine seam (ADR 0001). No prototype has exercised
  that path; `t` is the first thing that requires it.
- **C10** — the v1 inventory must be frozen before the gallery can claim to carry it.
  C10 is blocked only on C08 at the time of writing.
- **C11** — the gallery is the natural host for the key-only walkthroughs and the
  degradation-matrix screens C11 already owes.

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
- **This ticket is NOT satisfied and stays open.** The gallery stands on the prototype
  lineage, not on the shipped library: `vitui-engine/src/lib.rs` is still a stub and
  `seam.rs` is still `todo!()` throughout. The acceptance criteria that remain unmet are the
  library itself, C10's full 29-row inventory (the gallery carries the built tier only), the
  drift gate between inventory and gallery, the performance budget measured in-gallery, and
  the three-repertoire x three-depth matrix.
- Both obligations O1 and O2 are mandatory, and the frozen inventory is 29 panels, not the
  ten built here.
