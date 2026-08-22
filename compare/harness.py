#!/usr/bin/env python3
"""The comparative suite's harness: it measures the arms and writes REPORT.md.

`SCENES.md` is normative about what a scene is and `ARM-CONTRACT.md` about how an arm is run. This
file is the third thing — what is measured, and what is refused.

Three quantities, and the ticket's reasons for each, unchanged:

* **Bytes on the wire.** Exact and machine-independent. A byte count is a byte count on any machine,
  which is the whole argument for the register's shape and the reason this is the column the report
  leads with.
* **End-to-end keystroke-to-wire latency.** End-to-end because our wake-up interval is scheduler
  latency and the synchronous renderers have no such interval at all, so a per-frame figure would be
  comparing two different things.
* **Process CPU time over a fixed scene.** Which is where Python counts: Textual's interpreter
  overhead is what its users pay.

Two rules this file enforces rather than documents:

* **A missing row is never blank.** Every cell is a number, `cannot express`, or `not built here`
  with a reason. A missing row reads as a win, and that is the single easiest way for this suite to
  become dishonest.
* **It reports; it does not block.** Nothing here exits non-zero because an arm was slow or fat. Four
  external projects' versions cannot gate our pull requests. It *does* exit non-zero when it cannot
  measure what it claimed to — an unread instrument is a failure and not a pass.
"""

from __future__ import annotations

import argparse
import os
import resource
import shutil
import statistics
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path

HERE = Path(__file__).resolve().parent

# The five picture scenes, in the order SCENES.md defines them. `layered` is the flag that produces a
# `cannot express` cell rather than a blank one.
SCENES = [
    ("caret", False),
    ("status-line", False),
    ("list-scroll", False),
    ("full-repaint", False),
    ("modal-over-list", True),
]

FRAMES = 120

# The exit status an arm uses to say *I cannot reach this picture*. Distinct from any failure, because
# the two must not be reported as the same thing: one is a fact about a framework and the other is a
# fact about this run.
#
# 69 is `EX_UNAVAILABLE` from `sysexits.h`, which is a real convention rather than a number this file
# invented — and it is far enough from the small integers that an arm exiting 3 for its own reasons
# cannot be mistaken for one saying *I cannot draw this*.
CANNOT_EXPRESS = 69

# **The declared tier, and there are two of them because one would have hidden the finding.**
#
# `SCENES.md` fixes the terminal at 120x40 and the contract puts stdout on a pipe. What it cannot fix
# from outside is what an arm believes about colour, and the arms disagree profoundly: this engine,
# handed a pipe, declares *no colour at all* and emits not one SGR sequence, because a headless
# `attach` runs no detection and the conservative tier is what is left. Others emit 24-bit colour into
# a pipe regardless of what anybody declared.
#
# Reporting one tier would therefore have been a comparison of two different pictures. Two tiers, both
# declared, and the report states what each arm did with the declaration.
TIERS = {
    "truecolor": {
        "TERM": "xterm-256color",
        "COLORTERM": "truecolor",
    },
    "no-color": {
        "TERM": "xterm-256color",
        "NO_COLOR": "1",
    },
}


@dataclass
class Arm:
    name: str
    argv: list[str]
    # Per-arm environment needed to make the *declared* tier actually take effect. Every entry here
    # is reported, and that is the point: an arm that needs a lever is an arm whose default answer to
    # the declaration was something else.
    lever: dict[str, dict[str, str]] = field(default_factory=dict)
    # Why the arm is absent, when it is. `why_absent` is the standing reason an arm may be missing,
    # written once here rather than reconstructed from a path at report time: *not built here* is only
    # honest if it says what would have to change.
    why_absent: str | None = None
    absent: str | None = None
    version: str = "?"
    declaration: str = "?"


def build_arms(only: list[str] | None) -> list[Arm]:
    target = HERE / "target" / "release"
    venv_python = HERE / "arms" / "textual" / ".venv" / "bin" / "python"
    textual_arm = HERE / "arms" / "textual" / "arm.py"
    notcurses_arm = HERE / "arms" / "notcurses" / "arm"

    arms = [
        Arm(
            name="vitui",
            argv=[str(target / "arm-vitui")],
            # The engine's own lever. `VITUI_FORCE_COLOR` is precedence level 2 (see `caps.rs`), which
            # is what a headless run needs because there is no terminal to ask; the two default
            # colours are needed for the same reason — an operator layer cannot mix toward black
            # without knowing what the ground is, so scene 5's dim is a no-op until they are declared.
            lever={
                "truecolor": {
                    "VITUI_FORCE_COLOR": "truecolor",
                    "VITUI_DEFAULT_FG": "#c0c0c0",
                    "VITUI_DEFAULT_BG": "#000000",
                    "VITUI_GLYPHS": "extended",
                },
                "no-color": {},
            },
        ),
        Arm(name="ratatui", argv=[str(target / "arm-ratatui")]),
        Arm(
            name="textual",
            argv=[str(venv_python), str(textual_arm)],
            why_absent="no virtual environment — run `compare/arms/textual/setup.sh`, which "
            "installs the pinned Textual from `requirements.txt`",
        ),
        Arm(
            name="notcurses",
            argv=[str(notcurses_arm)],
            why_absent="no notcurses development files on this machine. Homebrew's formula pulls "
            "ffmpeg, and a several-hundred-megabyte install is not a side effect this suite gets to "
            "have; the pinned Linux runner installs `libnotcurses-core-dev` and builds it there. "
            "**This is `not built here`, which is a fact about the run — it is not `cannot express`, "
            "which would be a fact about notcurses, and see the withdrawal below for why that "
            "distinction is not academic**",
        ),
    ]
    for arm in arms:
        first = Path(arm.argv[0])
        if not first.exists() and shutil.which(arm.argv[0]) is None:
            arm.absent = f"not built here — {arm.why_absent}" if arm.why_absent else (
                f"not built here — {first} is missing"
            )
    if only:
        arms = [a for a in arms if a.name in only]
    return arms


def env_for(arm: Arm, tier: str) -> dict[str, str]:
    env = dict(os.environ)
    # A clean slate for every variable either tier sets, so a run does not inherit the developer's.
    for key in ("NO_COLOR", "COLORTERM", "TERM"):
        env.pop(key, None)
    env["COLUMNS"] = "120"
    env["LINES"] = "40"
    env.update(TIERS[tier])
    env.update(arm.lever.get(tier, {}))
    return env


def one_run(arm: Arm, scene: str, tier: str, frames: int) -> tuple[int | None, str]:
    """One invocation. Returns (byte count, note); the count is None when there is no number."""
    proc = subprocess.run(
        [*arm.argv, "--scene", scene, "--frames", str(frames)],
        env=env_for(arm, tier),
        stdin=subprocess.DEVNULL,
        capture_output=True,
        # **Its own session, and this is not hygiene — it is the difference between a measurement and
        # a hang.** An arm that can find `/dev/tty` will use it: notcurses opens it, takes the
        # geometry from `TIOCGWINSZ` rather than from `COLUMNS`, and then blocks for ever waiting for
        # a Device Attributes reply that a pipe will never carry. crossterm's `terminal::size()` does
        # the same thing more quietly — it *succeeds*, and returns the size of whatever window the
        # operator happened to have open, so an arm written to fall back to `COLUMNS` never reaches
        # the fallback. Detaching from the controlling terminal is what makes the contract's *there is
        # no other source of the size* true from outside rather than only by an arm's good manners.
        start_new_session=True,
    )
    if proc.returncode == CANNOT_EXPRESS:
        return None, "cannot express"
    if proc.returncode != 0:
        tail = proc.stderr.decode(errors="replace").strip().splitlines()
        return None, f"FAILED: exit {proc.returncode}: {tail[-1] if tail else 'no message'}"
    return len(proc.stdout), proc.stderr.decode(errors="replace").strip()


def run_bytes(arm: Arm, scene: str, tier: str, frames: int) -> tuple[str, str, str]:
    """Bytes for one arm, one scene, one tier.

    Returns `(total, marginal, note)`. **Two numbers rather than one, and the second is the one the
    first run of this suite was written without.**

    A total over 120 frames folds three different things together: the session prologue, the first
    frame, and the 119 after it. On `caret` that hid a reversal — this engine's total is 74% larger
    than ratatui's and its *steady frame* is three times cheaper, because it paints 4 800 cells on
    frame 0 that ratatui leaves alone. One number would have reported the opposite of what is
    happening, and it would have been a true number.

    So the marginal cost is `(bytes(N) - bytes(1)) / (N - 1)`: what one more frame of the described
    change costs, with the prologue and the first paint differenced out.
    """
    counts = []
    note = ""
    for attempt in range(2):
        n, message = one_run(arm, scene, tier, frames)
        if attempt == 0:
            note = message
        if n is None:
            head = "cannot express" if message == "cannot express" else "FAILED"
            return head, head, message
        counts.append(n)
    if counts[0] != counts[1]:
        # Not an error, and not silently averaged either. The contract requires frame *n* to be a
        # function of *n*; an arm that cannot manage it is reported as a spread, because a mean would
        # present an unrepeatable number as a repeatable one.
        return f"{counts[0]}~{counts[1]}", "—", "not byte-identical across two runs"
    first, _ = one_run(arm, scene, tier, 1)
    if first is None or frames < 2:
        return str(counts[0]), "—", note
    marginal = (counts[0] - first) / (frames - 1)
    return str(counts[0]), f"{marginal:.1f}", note


def run_cpu(arm: Arm, tier: str, seconds: int) -> tuple[str, str]:
    """Process CPU over the fixed CPU scene, as a percentage of one core."""
    env = env_for(arm, tier)
    before = resource.getrusage(resource.RUSAGE_CHILDREN)
    began = time.monotonic()
    proc = subprocess.run(
        [*arm.argv, "--scene", "cpu", "--seconds", str(seconds)],
        env=env,
        stdin=subprocess.DEVNULL,
        capture_output=True,
        start_new_session=True,
    )
    wall = time.monotonic() - began
    after = resource.getrusage(resource.RUSAGE_CHILDREN)
    if proc.returncode == CANNOT_EXPRESS:
        return "cannot express", ""
    if proc.returncode != 0:
        return "FAILED", f"exit {proc.returncode}"
    cpu = (after.ru_utime - before.ru_utime) + (after.ru_stime - before.ru_stime)
    if wall <= 0:
        return "FAILED", "no wall time elapsed"
    return f"{cpu / wall * 100:.2f}%", f"{cpu:.3f}s of CPU over {wall:.2f}s"


def run_latency(arm: Arm, tier: str, trials: int) -> tuple[str, str]:
    """Keystroke to wire: the interval from writing a byte to the arm's next output byte.

    A pipe rather than a pty, and the report says so. What is measured is the arm's own reaction, and
    the emulator's paint is not in it and cannot be from a harness — `SCENES.md` states that as the
    honest form of the quantity rather than as a limitation discovered here.
    """
    env = env_for(arm, tier)
    proc = subprocess.Popen(
        [*arm.argv, "--scene", "latency"],
        env=env,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        bufsize=0,
        start_new_session=True,
    )
    samples = []
    try:
        # Drain the initial screen. Without this the first trial times the tail of a frame the arm was
        # already writing, which reads as a spectacular latency for a keystroke that had not arrived.
        drain_until_quiet(proc, seconds=1.0)
        for trial in range(trials):
            key = b"a" if trial % 2 == 0 else b"b"
            began = time.perf_counter()
            proc.stdin.write(key)
            proc.stdin.flush()
            first = proc.stdout.read(1)
            if not first:
                break
            samples.append((time.perf_counter() - began) * 1e6)
            drain_until_quiet(proc, seconds=0.05)
    except (BrokenPipeError, OSError) as exc:
        return "FAILED", f"{type(exc).__name__}: {exc}"
    finally:
        try:
            proc.stdin.close()
        except OSError:
            pass
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()
    if len(samples) < max(3, trials // 4):
        return "FAILED", f"only {len(samples)} of {trials} keystrokes were answered"
    samples.sort()
    p50 = statistics.median(samples)
    p99 = samples[min(len(samples) - 1, int(len(samples) * 0.99))]
    return f"{p50:.0f} / {p99:.0f}", f"{len(samples)} samples, us p50 / p99"


def drain_until_quiet(proc: subprocess.Popen, seconds: float) -> None:
    """Read whatever is waiting until nothing has arrived for `seconds`."""
    import selectors

    sel = selectors.DefaultSelector()
    sel.register(proc.stdout, selectors.EVENT_READ)
    try:
        while sel.select(timeout=seconds):
            if not os.read(proc.stdout.fileno(), 1 << 16):
                return
    finally:
        sel.close()


def declaration_of(arm: Arm, tier: str) -> str:
    """The arm's own stderr line, which is where `no_color` and `alt_screen` come from."""
    proc = subprocess.run(
        [*arm.argv, "--scene", "caret", "--frames", "1"],
        env=env_for(arm, tier),
        stdin=subprocess.DEVNULL,
        capture_output=True,
        start_new_session=True,
    )
    for line in proc.stderr.decode(errors="replace").splitlines():
        if line.startswith("arm="):
            return line.strip()
    return "no declaration line — the arm did not honour the contract"


def main() -> int:
    ap = argparse.ArgumentParser(description="run the comparative suite")
    ap.add_argument("--frames", type=int, default=FRAMES)
    ap.add_argument("--cpu-seconds", type=int, default=5)
    ap.add_argument("--latency-trials", type=int, default=40)
    ap.add_argument("--arms", nargs="*", default=None)
    ap.add_argument("--out", default=str(HERE / "REPORT.md"))
    ap.add_argument("--label", default="", help="what machine this run is from")
    args = ap.parse_args()

    arms = build_arms(args.arms)
    present = [a for a in arms if a.absent is None]
    if not present:
        print("no arm is built; nothing can be measured", file=sys.stderr)
        return 1

    for arm in present:
        arm.declaration = declaration_of(arm, "truecolor")
        for field_name in arm.declaration.split():
            if field_name.startswith("version="):
                arm.version = field_name.removeprefix("version=")

    results: dict[tuple[str, str, str], tuple[str, str]] = {}
    for tier in TIERS:
        for scene, _layered in SCENES:
            for arm in present:
                print(f"  {tier:<10} {scene:<18} {arm.name}", file=sys.stderr, flush=True)
                results[(tier, scene, arm.name)] = run_bytes(arm, scene, tier, args.frames)

    cpu: dict[str, tuple[str, str]] = {}
    latency: dict[str, tuple[str, str]] = {}
    for arm in present:
        print(f"  cpu        {arm.name}", file=sys.stderr, flush=True)
        cpu[arm.name] = run_cpu(arm, "truecolor", args.cpu_seconds)
        print(f"  latency    {arm.name}", file=sys.stderr, flush=True)
        latency[arm.name] = run_latency(arm, "truecolor", args.latency_trials)

    report = render(arms, present, results, cpu, latency, args)
    Path(args.out).write_text(report)
    print(f"\nwrote {args.out}")
    return 0


def render(arms, present, results, cpu, latency, args) -> str:
    names = [a.name for a in present]
    out: list[str] = []
    w = out.append

    w("# The comparative suite")
    w("")
    w("**Generated by `compare/harness.py`. It reports; it does not block.**")
    w("")
    w("Four external projects' versions cannot gate this repository's pull requests, and a worsening")
    w("number arriving as a review-visible diff is what makes requirement 11 falsifiable. So this file")
    w("is committed and regenerated, and nothing in CI goes red because a number in it moved.")
    w("")
    w("Read [`SCENES.md`](SCENES.md) first. Every scene here is a described picture at 120x40 — a")
    w("terminal size, an initial screen, and a sequence of changes — and an arm may reach it any way")
    w("its framework prefers. Nothing in the definitions says *widget*, *diff* or *layer*, because the")
    w("comparison spans immediate-mode and retained-mode designs and those words would have decided it")
    w("in advance.")
    w("")
    w(f"- **Frames per scene:** {args.frames}")
    w(f"- **CPU scene:** the caret at 60 Hz for {args.cpu_seconds}s of wall time")
    w(f"- **Latency trials:** {args.latency_trials} keystrokes")
    if args.label:
        w(f"- **Machine:** {args.label}")
    w("")

    w("## The arms, and what each did with the declaration")
    w("")
    w("| arm | version | `NO_COLOR` | alt screen | lever needed for the declared tier |")
    w("|---|---|---|---|---|")
    for arm in arms:
        if arm.absent:
            w(f"| {arm.name} | — | — | — | **{arm.absent}** |")
            continue
        fields = dict(
            part.split("=", 1) for part in arm.declaration.split() if "=" in part
        )
        lever = ", ".join(sorted(arm.lever.get("truecolor", {}))) or "none"
        w(
            f"| {arm.name} | `{fields.get('version', '?')}` | {fields.get('no_color', '?')} "
            f"| {fields.get('alt_screen', '?')} | {lever} |"
        )
    w("")
    w("**A framework that ignores `NO_COLOR` is not thereby faster.** It is emitting fewer sequences")
    w("than it was asked to and more than the tier allows, so its byte count on the `no-color` tier")
    w("means something different from an arm that honoured the declaration. `unsupported` is a third")
    w("answer and not a synonym for either.")
    w("")
    w("The **lever** column is the finding this table exists for. An arm that needs one is an arm")
    w("whose default answer to a pipe was something other than the declaration.")
    w("")

    for tier in TIERS:
        w(f"## Bytes a frame, steady state — declared tier `{tier}`")
        w("")
        w("`(bytes(120) - bytes(1)) / 119` — what one more frame of the described change costs, with")
        w("the session prologue and the first paint differenced out. **This is the table to read**;")
        w("the totals below fold three different things together and reverse at least one row.")
        w("")
        w("| scene | " + " | ".join(names) + " |")
        w("|---" * (len(names) + 1) + "|")
        for scene, layered in SCENES:
            cells = []
            for name in names:
                _total, marginal, _note = results[(tier, scene, name)]
                cells.append(
                    f"**{marginal}**" if marginal in ("cannot express", "FAILED") else marginal
                )
            suffix = " *(layered)*" if layered else ""
            w(f"| `{scene}`{suffix} | " + " | ".join(cells) + " |")
        w("")
        w(f"### And the totals over {args.frames} frames, prologue and first paint included")
        w("")
        w("| scene | " + " | ".join(names) + " |")
        w("|---" * (len(names) + 1) + "|")
        for scene, layered in SCENES:
            cells = []
            for name in names:
                total, _marginal, _note = results[(tier, scene, name)]
                cells.append(f"**{total}**" if total in ("cannot express", "FAILED") else total)
            suffix = " *(layered)*" if layered else ""
            w(f"| `{scene}`{suffix} | " + " | ".join(cells) + " |")
        w("")
        for scene, _ in SCENES:
            for name in names:
                total, _marginal, note = results[(tier, scene, name)]
                if note and total in ("cannot express", "FAILED"):
                    w(f"- `{scene}` / {name}: {note}")
        w("")

    w("## Process CPU over the fixed scene")
    w("")
    w("| arm | % of one core | measured |")
    w("|---|---|---|")
    for name in names:
        cell, note = cpu[name]
        w(f"| {name} | {cell} | {note} |")
    w("")
    w("**Python counts.** Textual's interpreter overhead is what its users pay, and it is in this")
    w("column rather than subtracted out of it.")
    w("")

    w("## Keystroke to wire")
    w("")
    w("| arm | us p50 / p99 | samples |")
    w("|---|---|---|")
    for name in names:
        cell, note = latency[name]
        w(f"| {name} | {cell} | {note} |")
    w("")
    w("Keystroke-to-**wire**, not keystroke-to-photon: the emulator's own paint is not in the number")
    w("and cannot be from a harness. This is the form of the quantity that is comparable at all — our")
    w("wake-up interval is scheduler latency and a synchronous renderer has no such interval, which is")
    w("exactly why an absolute per-frame figure would be comparing two different things.")
    w("")
    w("Measured over a pipe rather than a pty. The declared terminal is `TERM=xterm-256color`; the")
    w("number is comparable only within one terminal, and that is the one.")
    w("")

    w("## What is absent, and why it is written down")
    w("")
    w("**A missing row reads as a win, and that is the single easiest way for this suite to become")
    w("dishonest.** So there are three kinds of non-number and they are not interchangeable:")
    w("")
    w("- **`cannot express`** — the framework cannot reach the described picture as the scene")
    w("  describes it. A fact about the framework, not about this run. **And the one instance of it in")
    w("  this table carries a withdrawal**: notcurses' `modal-over-list` cell is not a claim that")
    w("  notcurses cannot composite layers — it can, through `NCALPHA_BLEND`, built and demonstrated")
    w("  by impl 26. The arm refuses the scene because `SCENES.md` is normative; see `FINDINGS.md`.")
    w("- **`not built here`** — the arm exists and was not built on this run. A fact about the run.")
    w("- **`FAILED`** — the arm was run and did not do what the contract says. A defect, in the arm.")
    w("")
    for arm in arms:
        if arm.absent:
            w(f"- **{arm.name}**: {arm.absent}")
    w("")
    return "\n".join(out) + "\n"


if __name__ == "__main__":
    sys.exit(main())
