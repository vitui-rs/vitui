#!/usr/bin/env bash
# The comparative suite's one entry point: build what can be built here, measure it, rewrite
# `REPORT.md`.
#
# `README.md` documents three invocations and `.github/workflows/compare.yml` uses one of them, so
# this file's interface is fixed by two files that were written before it:
#
#   ./run.sh --check                        lint the arms, build nothing else
#   ./run.sh --label "M1 Max, macOS 26.5.2" measure everything buildable, rewrite REPORT.md
#   ./run.sh --arms vitui ratatui           a subset
#
# **It reports; it does not block** — and that rule is what shapes every exit status below. A missing
# arm is not a failure of this script, because *not built here* is a fact about the run and the
# report has a cell for it; the harness refuses to be blank rather than refusing to run. What this
# script does fail on is the two things that are not about other people's software: `--check` finding
# a warning in an arm we wrote, and the harness being unable to measure what it claimed to.
#
# `FRAMES` in the environment overrides the harness default, which is how `compare.yml`'s
# `workflow_dispatch` input reaches it.
set -euo pipefail

here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$here"

check_only=0
label=""
arms=()
out=""
frames="${FRAMES:-120}"

while [ $# -gt 0 ]; do
  case "$1" in
    --check) check_only=1; shift ;;
    --label) label="${2-}"; shift 2 ;;
    --frames) frames="${2-}"; shift 2 ;;
    # Passed through to the harness. Not in `README.md`'s three lines, and it is here for the
    # footgun: `--arms vitui ratatui` otherwise rewrites the *committed* report with a two-column
    # table, which is a diff a reviewer would read as three arms having disappeared.
    --out) out="${2-}"; shift 2 ;;
    # `--arms` takes the rest of its run of non-flag words, which is the shape `README.md` shows:
    # `--arms vitui ratatui`, not `--arms=vitui --arms=ratatui`.
    --arms)
      shift
      while [ $# -gt 0 ] && [ "${1#--}" = "$1" ]; do arms+=("$1"); shift; done
      ;;
    -h|--help)
      sed -n '2,19p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
      exit 0 ;;
    *)
      echo "run.sh: unknown argument $1" >&2
      exit 64 ;;
  esac
done

python="${PYTHON:-python3}"

# ── --check: lint the arms, build nothing else ───────────────────────────────────────────────────
#
# `Cargo.toml` has no `[lints]` table and says why: a lint that failed the build here would fail a
# *report*, and a report may not be load-bearing for a gate. That is not a licence for warnings — it
# moves where they are seen, and this is where. `-D warnings` on the command line rather than in the
# manifest, so the failure lands on somebody running the linter and never on a monthly measurement.
if [ "$check_only" = 1 ]; then
  status=0
  echo "── cargo fmt"
  cargo fmt --all --check || status=1
  echo "── cargo clippy"
  cargo clippy --all-targets --all-features -- -D warnings || status=1
  echo "── python"
  # A compile and not a run: the harness's own failure mode is an arm it cannot measure, and that is
  # not something a syntax check can see.
  #
  # `compile()` rather than `compileall`, and the difference is not stylistic — `compileall` **writes
  # `__pycache__` into the working tree**, and both `__pycache__` directories in here are tracked
  # files. A linter that dirties `git status` every time it is run teaches people not to run it, and
  # this suite's whole falsifiability argument is a committed file whose diff a human reads.
  "$python" -c 'import sys
for f in sys.argv[1:]:
    compile(open(f).read(), f, "exec")' harness.py arms/textual/arm.py || status=1
  echo "── notcurses"
  # **Not a failure.** `make check-headers` exits 1 when the development files are absent, which is
  # the machine this suite was written on and is `not built here` rather than a defect. It is run for
  # its output, which names the package CI installs.
  make -C arms/notcurses check-headers || echo "   (the notcurses arm is not buildable here — see arms/notcurses/NOTES.md)"
  exit "$status"
fi

# ── build ────────────────────────────────────────────────────────────────────────────────────────
#
# Release, always, and `Cargo.toml` says why: a debug build of ours against a release build of
# somebody else's would be the most flattering possible mistake to make in the wrong direction.
echo "── cargo build --release"
cargo build --release

# The notcurses arm is C and is built by its own `Makefile`, which refuses cleanly when pkg-config
# finds nothing. `|| true`: **a missing arm is a row, not an error.** The harness discovers the
# binary is absent and writes `not built here` with the standing reason, which is the whole point of
# `Arm.why_absent` being written down rather than reconstructed from a path.
echo "── make -C arms/notcurses"
make -C arms/notcurses || true

# Textual's virtual environment is `setup.sh`'s, and it is not created here. Creating one implicitly
# would mean a measurement run reaching for the network, and the version that arrived would be
# whatever pip resolved that morning — against a suite whose entire premise is that every arm names
# an exact version. Absent, it is `not built here` with the command that fixes it.
if [ ! -x arms/textual/.venv/bin/python ]; then
  echo "── textual: no virtual environment (run arms/textual/setup.sh)"
fi

# ── measure ──────────────────────────────────────────────────────────────────────────────────────
# `if` and not `[ ... ] && cmd+=(...)`, which is the shorter spelling and is wrong under
# `set -e`: an AND-list whose left side is false *is* a failed statement, so a bare `./run.sh`
# with no label and no subset would have exited 1 before measuring anything.
cmd=("$python" harness.py --frames "$frames")
if [ -n "$label" ]; then cmd+=(--label "$label"); fi
if [ -n "$out" ]; then cmd+=(--out "$out"); fi
if [ "${#arms[@]}" -gt 0 ]; then cmd+=(--arms "${arms[@]}"); fi

echo "── ${cmd[*]}"
exec "${cmd[@]}"
