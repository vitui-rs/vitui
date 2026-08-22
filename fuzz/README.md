# `fuzz/` — two targets, a committed corpus, and a soak

Spec §14, one sentence, and everything here is a consequence of it:

> **Every crash is minimised and committed as an ordinary unit test, and the committed corpus
> replayed as an ordinary test is the gate. The fuzzer itself is a scheduled soak.**

A fuzzer cannot be a pull-request gate. At sixty seconds it finds nothing and reports green, which is
a gate that cannot fail. At an hour it is a flaky test that goes red on a commit unrelated to
whatever it finally reached, and the next person to see it red reruns it. So the polarity inverts:
**the corpus in this directory is the gate**, replayed by
`crates/vitui-engine/src/fuzz.rs`'s `the_committed_corpus_replays_as_an_ordinary_test` under an
ordinary `cargo test`, on stable, in every job that runs the suite.

The fuzzer's job is to *grow* that corpus.

## The harness is in the engine, not here

`fuzz_targets/*.rs` is four lines each. Everything — the byte-string decoder, the oracle, the caps
on how much one input may do — is `vitui_engine::fuzz`, behind a non-default `fuzz` feature and
`#[doc(hidden)]`.

That is not tidiness. The gate and the soak have to be **the same code**: if the decoding lived here,
the replay test could only reimplement it, and *a check that is weaker than the gate is not a check*.
A corpus file would then be replayed through a second oracle that agrees with the first for exactly
as long as somebody keeps the two in step.

The engine's own `crate::audit` holds the door shut from the other side —
`the_fuzz_door_is_behind_a_feature_and_hidden` asserts the feature exists, is off by default, and is
hidden from rustdoc, so nothing an ordinary dependent compiles can reach any of it.

## The two targets

**`draw_sequence`** — a byte string is a *program*: two bytes of screen size, then verbs against a
layer stack. Every `present` is a checkpoint, and at a checkpoint the frame is asserted against
`crate::reference`, the naive compositor that visits one cell at a time with no damage and no runs.
Two halves, and they are gate #1's own:

1. every cell the frame's verbs changed lies inside a run the damage structure reported, and
2. the damage-tracked frame equals the reference **everywhere**, including outside every damaged run.

This is where §14 says gate #1 is *generated rather than hand-written*. A hand-written expectation
about damage is written by whoever wrote the damage and agrees with it for the same reason.

**`input_bytes`** — a byte string is a terminal talking, and this is the one surface in the engine
that parses input the engine did not produce. §14 concedes the oracle is weaker: no panic, every byte
consumed, no unbounded growth. What each of those means as an assertion is in `crate::fuzz`, and the
middle one is spelled out at length there because its literal reading is a statement about a `for`
loop rather than about the parser.

## Running it

`cargo-fuzz` needs nightly and `libfuzzer-sys`:

```sh
rustup toolchain install nightly
cargo install cargo-fuzz --locked

# One target, fifteen minutes, starting from the committed corpus.
RUSTUP_TOOLCHAIN=nightly cargo fuzz run draw_sequence -- -max_total_time=900
RUSTUP_TOOLCHAIN=nightly cargo fuzz run input_bytes   -- -max_total_time=900
```

`RUSTUP_TOOLCHAIN=nightly` rather than `cargo +nightly fuzz`: `cargo-fuzz` shells out to `cargo`
again for the inner build, and the `+toolchain` argument does not survive that hop — the inner build
resolves to the default toolchain and fails on `-Zsanitizer=address`. The environment variable does
survive.

New inputs land in `corpus/<target>/` under a SHA-1 name. **They are not what gets committed** — see
*What the committed corpus is* below. A crashing input lands in `artifacts/<target>/`, which is
`.gitignore`d, because a crash does not stay there:

```sh
# Minimise it first — libFuzzer's own minimiser, and it is good at it.
RUSTUP_TOOLCHAIN=nightly cargo fuzz tmin draw_sequence artifacts/draw_sequence/crash-…
```

Then:

1. Add a `#[test]` to `crates/vitui-engine/src/fuzz.rs`'s `mod tests`, **named for the defect and not
   for the byte string**, whose body is one call with the minimised input inline. The tests there are
   the record of what has been found; the section header says so.
2. Fix the defect.
3. Commit the minimised input to `corpus/<target>/` under a name that says what it is, so the fuzzer
   keeps that shape in its pool.

The unit test is the regression gate and the corpus entry is the seed. They are not redundant: the
test names one input for ever, and the corpus entry is what the *next* soak mutates.

## What the committed corpus is, and what it deliberately is not

**Every file in `corpus/` is named for what it is, and there is nothing else in there.** Two kinds:
hand-written seeds for shapes that are hard to reach by luck, and every input that has ever found
something — a defect, or a disagreement that turned out to be an open question.

A twenty-minute soak of `draw_sequence` grows the pool to about three thousand files; `cargo fuzz
cmin` reduces that to about thirteen hundred, and renames all of them to SHA-1. **That pool is a soak
artefact and is not committed**, and the decision is deliberate rather than lazy:

- Thirteen hundred files nobody can explain is not something this repository commits. Every other
  fixture here — the twelve scenes, the goldens, the negative corpus — is named for the property it
  carries, and a directory of hashes would be the one exception.
- It buys less than it looks. Measured on the first run: **794 new units in 8 619 executions**, which
  is under a second. libFuzzer re-derives that pool from these seeds almost immediately, so what the
  hashes carry is a head start of seconds on a fifteen-minute run.
- `cmin` renames what it keeps, so committing the pool means *losing* the named seeds — measured, on
  this repository, while writing this file.

The soak workflow uploads its grown corpus as an artifact on every run, which is where to look for it
when a run finds something worth keeping. What comes back into `corpus/` comes back with a name.

One consequence worth knowing locally: **do not run `cargo test` while a soak is writing the
corpus.** The replay test reads the directory and libFuzzer rewrites it, and the test will fail
reading a file that has just been renamed. That is the test being strict about a file it was told
exists, which is the right behaviour for a gate and an annoyance for exactly this one workflow.

## Writing a seed by hand

The committed seeds are named for what they encode, and the decoder they are read by is documented in
`crate::fuzz` — the arithmetic is deliberately all `%`, so a byte's meaning is local and a seed can be
written by hand. The short version, for `draw_sequence`:

| bytes | meaning |
|---|---|
| 0 | width, `4 + b % 13` |
| 1 | height, `2 + b % 5` |
| then | one opcode byte, `b % 12`, and its operands |

Opcodes: 0 add content, 1 add operator, 2 add shadow, 3 remove, 4 set z, 5 set rect, 6–10 a drawing
verb (`b % 5`: text, fill, set, restyle, clipped-and-scrolled child), 11 present. A coordinate on an
axis `span` long is `b % (span + 8) - 4`, so four columns outside each end are reachable — ADR 0022 is
clamp-and-discard and the wide-glyph hazard lives at an edge, so a generator that only produced
coordinates inside the screen would test neither.

`input_bytes` is simpler: byte 0 picks the paste ceiling out of five, byte 1 the read size the stream
is chunked into, and the rest is the wire.

## `fuzz/` is its own workspace, and that is a loophole

`cargo-fuzz` needs `libfuzzer-sys`, and the engine's dependency policy is *crossterm plus generated
UCD tables and nothing else* (ADR 0001, enforced by the root `deny.toml`). Detaching the workspace is
what keeps that policy honest — and it also means the root `cargo deny check` **cannot see anything in
here**. §14 names it: *that is a loophole, not a permission*.

So this directory has its own `deny.toml` and its own `cargo deny` invocation, wired into the `deny`
job of both `.gitlab-ci.yml` and `.github/workflows/ci.yml`. **`Cargo.lock` is committed** — unlike
`examples/app-template`'s, which is `.gitignore`d — and the difference is that gate: `yanked = "deny"`
and the advisory check are claims about resolved versions, so a graph re-resolved on every run is a
gate whose subject moves under it. The policy is the root's, and the delta
is one license: `libfuzzer-sys` is `(MIT OR Apache-2.0) AND NCSA`, because it vendors LLVM's own
libFuzzer sources. That allowance is scoped to this file, so a `libfuzzer-sys` that somehow appeared
in the engine's graph would still be rejected at the root.

## What has actually been run, and what has not

The soak is `.github/workflows/soak.yml`, weekly and on demand, **and this repository has no hosted
CI** — the same caveat `.gitlab-ci.yml` opens with. It has never been watched go green, and a job
nobody has watched go green is not a gate. That is survivable here in a way it would not be for a
gate, and it is exactly why the corpus is the gate instead.

What *was* run, at ticket 25, on macOS/arm64 with nightly and `cargo-fuzz 0.13.2`: both targets, from
the committed seeds, for the times recorded in the ticket. `draw_sequence` found **two live defects**
in the operator's reach at the frame's edge — both fixed, both committed as named unit tests — and
**three instances of an open architecture question** (arch 20), which are in the corpus and are
excused by one allowance that names the ticket. `input_bytes` found nothing over **10 518 149 executions in
1 201 seconds** — 24 741 new coverage units at 8 757 executions a second.
