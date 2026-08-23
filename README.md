# vitui

A Rust TUI library: layered terminal rendering with damage tracking, built so that **frame cost is
proportional to visible cells, never to data volume**.

Three crates behind one facade. `vitui-engine` is everything that touches the terminal — cells,
surfaces, layers, compositing, damage, input, and the bytes on the wire. `vitui-runtime` is
everything above it — layout, identity, focus, hit-testing, routing, key maps, theming, overlays and
the data contract. `vitui-components` is the widget library. An application that wants only fast
layered output can depend on `vitui-engine` alone and never meet a layout type.

## Status

**Nothing is published. The version is `0.0.0` and there is no stability promise before 0.x.** Read
this section before the numbers below it.

| crate | state |
|---|---|
| `vitui-engine` | **implementation-complete.** All 26 implementation tickets resolved, all 28 verification-register entries wired, none pinned red. A production-readiness backlog of 9 tickets is open. |
| `vitui-runtime` | **9 of 20 tickets in.** `data`, `layout`, `layout::text`, `theme`, `keys`, `ctx`, `id`, `route` and the hit index exist. Focus, overlays, scrolling, sizing, async work, animation, the standard theme set and the verification ledger do not. |
| `vitui-components` | **empty.** One `lib.rs` with module documentation and no components. Out of scope for the current effort; it gets its own architecture map afterwards. |
| `vitui` | facade re-export of the three. |

Three things a prospective user should know, stated here rather than discovered:

- **One terminal emulator has been asked, once.** Until 2026-08-23 no instrument here had ever
  compared the engine's bytes against a real emulator's screen: the round-trip suite, the reference
  compositor and the terminal model they are checked against all live inside the crate, so a case
  where the model and the serializer are wrong *in the same direction* was invisible to every gate.
  `conform/` is the missing fourth party, and Ghostty 1.3.1 agrees with the engine on all eleven
  attribute bits. That is one scene on one emulator on one machine. The width questions are not
  answerable by a screen dump at all and are still open.
- **Windows has never been run.** `.github/workflows/ci.yml` declares a three-OS matrix and no hosted
  CI has been watched go green. Every green run behind the numbers below is a shared local GitLab on
  one machine: linux/arm64, one OS, one architecture.
- **One architecture question is still open**: the clip-edge pair, `.scratch/vitui-engine-architecture/issues/20`.
  Its central unknown is what a terminal does with a bisected double-width glyph, and both emulators
  measured re-serialise their grid as text with no padding cell — so no screen dump can settle it. The
  default SGR spelling (23) and a hyperlink on a standalone surface (21) were closed on 2026-08-22.

## The frame, as a sequence

```text
Engine::new(Config) -> attach() -> (Screen, WakeHandle)
  |                                 raw mode, the capability queries, the prologue -- and only
  |                                 then the render thread, where no concurrency existed yet
  |
  +- layers()                add layers; view(id) to draw into one
  |   +- the verbs           text - fill - restyle, marking damage as they write
  |
  +- set_mouse(level)        what the frame's components asked the pointer for, as a `max`
  +- set_cursor(caret)       where the caret goes, applied after the frame's last write
  |
  +- present() -> Presented  the app thread:
                               lease a packet, or fold this frame into the next one
                               composite the damaged rectangles bottom-up
                               pack runs and their cells into the packet
                               submit, clear damage, return
                             the render thread:
                               take the packet, serialise against the mirror,
                               one write into the sink, give the packet back
```

Three threads and one synchronisation primitive: a `Mutex<Shared>` with two condvars, and a pool of
exactly two packets. There is no backpressure, because dropping an intermediate frame is implemented
as *never composing it* — the app thread composites only after the renderer signals it is free.
`Config::clock` selects a deterministic single-thread mode in which both halves run on the calling
thread before `present` returns, same mailbox, same packet, same bytes; that mode is public API
rather than test scaffolding.

**The engine does not own the loop.** It hands out the verbs and `present`, and the runtime drives.
Damage is marked by the verbs and cleared by `present`, and neither is reachable from outside — which
is also why nothing above the engine can force a full repaint.

## The decisions that constrain everything

These are architectural, not stylistic. Each cost a session and has an ADR or a measurement behind
it in `docs/adr/`.

- **The engine lays nothing out.** Callers bring rectangles; no layout concept may enter through the
  `Surface` API (ADR 0002).
- **The engine never iterates application data.** It offers clipping and offset viewports; culling is
  the caller's job. Checked at both ends: a component drawing against `visible_rows()` costs 54 µs
  flat at 1 000, 100 000 and 1 000 000 rows, and a 1k-row list and a 1M-row list showing the same 80
  rows produce identical damage, identical run counts and identical emitted cells.
- **crossterm is invisible.** Input and terminal mode only, never output, and it appears in no public
  signature (ADR 0001).
- **Damage is marked at write time, not derived by diffing.** Prior art measured ratatui's
  full-buffer diff at ~170 µs on 300×80 — already over the budget for a whole frame.
- **A cell holds an interned grapheme-cluster handle, not a `char`**, forced by UAX #29. 16 bytes,
  align 4, no padding hole.
- **Dependency policy is enforced, not reviewed.** The engine takes `crossterm` plus
  build-script-generated UCD tables; the runtime takes nothing at all. `deny.toml` carries it, so a
  crate reaching for an async runtime meets `error[banned]` rather than a code review.
- **No `unsafe` in the engine** (`#![forbid(unsafe_code)]`), no traits on the public surface, no
  executor, no thread pool, and no way to read a cell back off the screen.

## Performance

The budget is a set of CI gates, not aspirations:

| | budget |
|---|---|
| full-screen composition, 300×80 (≈24 000 cells) | < 1 ms |
| typical damage-tracked frame | < 100 µs |
| steady-state 60 fps animation | < 5% of one core |
| allocations during frame composition | zero |
| a genuinely idle application | zero CPU |

**Measured on an Apple M1 Max (aarch64), release, on a 300×80 grid** — criterion medians or
minimum-of-forty round-robin samples. Everything the app thread pays for on the worst *realistic*
frame:

| step | cost |
|---|---|
| component drawing, five hostile components | ≈45 µs |
| compositing, worst measured full screen at 40 layers | 107.3 µs |
| packing, full screen with the equality filter's shadow | ≈53 µs |
| damage marking and scanning, worst frame | ≈7 µs |
| the mailbox critical section | 47 ns |
| the overrun detector | 42.6 ns |
| **total** | **≈205 µs against 1 ms** |

Roughly 5× of headroom, and that is what keeps parallel compositing out of scope rather than a claim
that it would not help. Idle, over thirty seconds: `30.01 s real, 0.00 user, 0.00 sys, 0 voluntary
context switches`, where a 120 Hz ticker would have woken 3 600 times. At 60 frames a second over
thirty seconds: 0.133% of a core against the 5% budget.

A timing is a gate only at cliff granularity, with the headroom written next to the number.
Nineteen of the twenty-seven gated properties are counts, ratios, equalities or compile outcomes
instead — a gate tuned to a measurement is a flaky test that gets disabled within a month.

### Against other libraries

`compare/` runs the same five described scenes against ratatui and Textual. **It reports; it does not
gate** — four external projects' versions cannot decide whether this repository's changes land.
Machine: Apple M1 Max, macOS 26.5.2, rustc 1.97.1, python 3.14.6; 120×40, 120 frames per scene.

Marginal bytes for one more frame at the `truecolor` tier — `(bytes(120) - bytes(1)) / 119`, with the
session prologue and first paint differenced out. This is the table to read; the per-scene totals in
`compare/REPORT.md` fold a one-off and a steady state together and invert at least one row.

| scene | vitui | ratatui | textual |
|---|---|---|---|
| `caret` | 9.5 | 28.5 | 25.0 |
| `status-line` | 22.4 | 46.5 | 47.5 |
| `list-scroll` | 572.3 | 704.7 | 5124.0 |
| `full-repaint` | 96408.0 | 106216.0 | 111036.0 |
| `modal-over-list` | 19.0 | 36.0 | 17.0 |

| | vitui | ratatui | textual |
|---|---|---|---|
| CPU, caret at 60 Hz | 0.27% of a core | 0.75% | 10.48% |
| keystroke to wire, µs p50 / p99 | 46 / 590 | 490 / 8695 | 1986 / 17069 |

The shape is what the architecture predicts — the advantage is largest where the change is smallest,
and `full-repaint` at 1.10× compares nothing but the encoder. Four caveats belong with these numbers
and `compare/FINDINGS.md` argues each one at length:

- **`list-scroll` at 1.2× is not a win and should be.** The scene as written moves the highlight with
  the list, so every frame changes forty rows of content and the scroll region — worth 30× to 85× on
  a steady scroll — is unreachable for both arms. A better version of that scene exists and is not
  this one.
- **The latency figure is not the number a user would feel.** The harness reads stdin on the app
  thread and calls `present` inline, so the sample never crosses the mailbox. A real keystroke pays a
  4.58 µs hop from the input thread that these 46 µs do not include.
- **Textual wins `modal-over-list`,** and one row of five is not a rounding error.
- **notcurses — the honest ceiling — is not in the table.** It was not built on this machine, which
  is a fact about the run and not about notcurses. A missing row reads as a win, which is the single
  easiest way for this suite to become dishonest.

Our own first frame writes all 4 800 cells of a screen the alternate-screen switch had already
blanked — about 4 900 bytes, once per session. Not fixed: it is a decision about what the engine may
assume of a terminal.

## Building

Requires **Rust 1.88** (`rust-version` in `Cargo.toml`), verified by the `msrv` job in
`.gitlab-ci.yml` rather than declared. Edition 2024.

```bash
cargo build --workspace
cargo test --workspace -- --test-threads=1   # serialised: the allocation gates count globally
cargo clippy --workspace --all-targets
cargo clippy -p vitui-engine --all-targets --features fuzz
cargo fmt --all
cargo deny check
cargo run --release --example budget -p vitui-engine   # the register and the ledger, runnable
```

Warnings are build failures locally exactly as in CI — `[workspace.lints.rust] warnings = "deny"`,
which unlike a committed `RUSTFLAGS` cannot be switched off from a shell.

`fuzz/` holds two libFuzzer targets and is a detached workspace with its own `deny.toml`, because
`cargo-fuzz` needs nightly and `libfuzzer-sys` and the engine's dependency policy will not have
them. The committed corpus replayed by `cargo test` is the gate; the fuzzers themselves are a soak.

## Documents

The architecture is written down before it is built, and every decision cites the ticket that
measured it.

- `CONTEXT.md` — the glossary. Its terms are used everywhere else without re-definition.
- `docs/adr/` — the decisions that are hard to reverse and surprising without context.
- `.scratch/vitui-engine-architecture/spec.md` — the engine's settled architecture.
- `.scratch/vitui-runtime-architecture/spec.md` — the runtime's.
- `compare/README.md`, `compare/SCENES.md` — what the comparative scenes are and how they are read.
- `fuzz/README.md` — the whole soak procedure.

## Licence

Dual-licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option. Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in the work shall be dual-licensed as above, without any additional terms or conditions.
