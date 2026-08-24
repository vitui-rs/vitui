# The comparative suite

**Register entry #27.** It is a claim about other people's software, so an absolute number cannot
check it, and §14 said as much: *the comparative suite is a report on a pinned runner, into a
committed file.* Until impl ticket 26 the entry was the one red row on the register, with the reason
`the suite has never been run`.

Three files are the suite and each answers a different question:

| | |
|---|---|
| [`SCENES.md`](SCENES.md) | **what is measured** — nine described pictures at 120×40, normative |
| [`ARM-CONTRACT.md`](ARM-CONTRACT.md) | **how an arm is run** — the invocation, the declaration it owes |
| [`harness.py`](harness.py) | **what is refused** — the three quantities, and the three kinds of non-number |

[`REPORT.md`](REPORT.md) is generated and committed. [`FINDINGS.md`](FINDINGS.md) is written by hand
and dated, because a number and what it means are two different artefacts with two different
lifetimes: the table is regenerated on every run and the reading of it is not.

```sh
compare/run.sh --check                        # lint the arms, build nothing else
compare/run.sh --label "M1 Max, macOS 26.5.2" # measure everything buildable, rewrite REPORT.md
compare/run.sh --arms vitui ratatui           # a subset
```

## Nine scenes and five arms, after runtime impl ticket 20

The suite was sliced twice. Engine impl ticket 26 built it with five scenes and four arms; runtime
impl ticket 20 owed *the runtime's scenes added to engine ticket 26's comparative suite*, and both
backlogs described the same one — same rules, same four external projects, same pinned runner, same
committed file. So there are four more scenes and one more arm, and nothing else moved.

**The four scenes** are the runtime's normative twenty (`crates/vitui-runtime/src/scenes.rs`) filtered
by the rule that already governed the first five: a scene is a described picture, and sixteen of the
twenty cannot be described without naming a mechanism or draw a picture one of the nine already
draws. `SCENES.md` names all sixteen and says which of the two applies to each, because a rejected
scene left as an absence is the same defect as a missing row.

**The fifth arm is ours**, and it is a second arm rather than a changed first one.
`arms/vitui` takes `vitui-engine`; `arms/vitui-runtime` takes the facade. Its manifest carries the
argument, and the short version is two sentences. Pointing the existing arm one layer up would have
silently redefined every row already in a committed report, which is the falsifiability mechanism
spent on a rename. And the pair buys what neither buys alone: **the runtime's cost on the wire is now
a delta between two of our own arms** — same scenes, same machine, same run — which is the only
difference in this table with exactly one known cause.

## It reports; it does not block

**Four external projects' versions cannot gate this repository's pull requests.** A suite that did
would go red when Textual cut a release, on a commit that touched nothing, and the honest response to
that would be to stop running it. So nothing here is in the four CI jobs, and
[`../.github/workflows/compare.yml`](../.github/workflows/compare.yml) is a workflow of its own on a
pinned runner.

What makes the requirement falsifiable anyway is that the report is **committed**: a worsening number
arrives as a review-visible diff. That is the same trade `fuzz/` makes — the soak is scheduled and the
committed corpus is the gate — and it is the reason this directory has no `assert` in it.

The workflow **uploads** its report rather than pushing one. A scheduled job with write access, for a
file a human has to read anyway, is the wrong shape; `soak.yml` refuses the same thing for the same
reason.

## Why `compare/` is a detached workspace

One arm depends on ratatui, which is the framework being compared against. In the workspace next door
that is a policy violation with a straight face — the engine takes crossterm plus generated UCD
tables and nothing else, ADR 0001, enforced by `../deny.toml` — and detaching means `cargo deny
check` at the repository root cannot see a single crate in here.

**Unlike `fuzz/`, this directory does not get a `deny.toml` of its own**, and the reason is on
`Cargo.toml`: `fuzz/`'s dependencies are incidental machinery, so a crate creeping into that graph is
a defect. Here the third-party dependencies *are the subject*, a policy over them could only be
satisfied by deleting an arm, and **a missing row reads as a win**. Pinning replaces it — every arm
names an exact version of what it compares against, and the report states them.

## The three kinds of non-number, which are not interchangeable

This is the rule the suite's honesty rests on, and the harness enforces it rather than documenting it:

- **`cannot express`** — the framework cannot reach the described picture, in the form the scene
  describes it. **A fact about the framework**, and the cell reads those two words rather than being
  left blank or quietly dropped from a mean.

  **The claim that notcurses cannot composite layers is withdrawn.** Impl 26 built notcurses-core
3.0.17 and demonstrated the opposite: `NCALPHA_BLEND`, resolved by the renderer over the plane stack,
lands a plane below at exactly half without the caller computing a single dimmed colour. So the
`modal-over-list` cell against notcurses is **not** a statement about what notcurses can do — the arm
refuses that scene because `SCENES.md` is normative and the correction is a map decision, and the real
picture is an opt-in scene `modal-over-list-blend` beside it. The evidence is filed against
architecture ticket 13, whose line 574 is the sentence that was wrong. **A cell claiming a competitor
cannot do something it can would have been the exact dishonesty the missing-row rule exists to
prevent, committed by that rule.**
- **`not built here`** — the arm exists and was not built on this run, with the reason. **A fact about
  the run.**
- **`FAILED`** — the arm ran and did not do what the contract says. **A defect in the arm.**

> **A missing row reads as a win, and this is the single easiest way for the suite to become
> dishonest.**

## What is measured, and why each one

- **Bytes on the wire.** Exact and machine-independent — a byte count is a byte count on any machine,
  which is the whole argument for the register's shape. Captured with stdout on a **pipe**, which is
  the declared tier, and **what each arm did with that declaration is reported**: a framework that
  ignores `NO_COLOR` is not thereby faster.
- **Keystroke-to-wire latency, end to end.** Because our wake-up interval is scheduler latency and the
  synchronous renderers have no such interval at all, so a per-frame figure would compare two
  different things. One terminal, named and pinned; the number is comparable only within it.
- **Process CPU over a fixed scene.** **Python counts** — Textual's interpreter overhead is what its
  users pay, so it is in the column rather than subtracted out of it, and its version is pinned and
  stated.

## Two declared colour tiers, not one

`SCENES.md` fixes the terminal at 120×40 and the contract puts stdout on a pipe. What it cannot fix
from outside is what an arm believes about colour, and the arms disagree profoundly — this engine,
handed a pipe, declares **no colour at all** and emits not one SGR sequence, because a headless
`attach` runs no detection and the conservative tier is what is left. Others emit 24-bit colour into a
pipe whatever anybody declared.

One tier would therefore have been a comparison of two different pictures. There are two, both
declared, and the **lever** column of the report says which arms needed telling.
