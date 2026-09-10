#!/usr/bin/env bash
#
# **Register entry #25, measured: 60 fps steady state against 5% of a core.**
#
# Until impl 26 this entry was arithmetic in `examples/budget.rs` — one animated frame's app-thread
# cost times sixty — and it read 0.0029%. The arithmetic was not wrong; it was about the wrong
# subject. It contained no render thread, no serializer, no write, and none of the sixty condvar
# round trips a second that pacing a real animation costs. The entry says *of a core*, and a core is
# a property of a process.
#
# So this is entry #17's shape a second time: `examples/steady.rs` asserts the **count** — that sixty
# frames a second actually happened, because a percentage measured over four frames is not a
# measurement of a steady state — and this script reads the **timing** out of `/usr/bin/time` and
# prints it. A report, per §14, and the count underneath it is what stops the report being vacuous.
#
# **Two-runner, not matrix**, for the same reason as `idle-gate.sh`: `/usr/bin/time` is BSD's with
# `-l` on macOS and GNU's with `-v` on Linux.
#
# Thirty seconds by default rather than ten, and the reason is resolution rather than rigour:
# `/usr/bin/time` reports hundredths, so a ten-second run cannot resolve better than 0.10% of a core
# and the measured figure is at that floor. At thirty seconds the floor is 0.033%, which is below the
# number rather than equal to it.
#
#   scripts/steady-report.sh [seconds]
#
set -euo pipefail

seconds="${1:-30}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
binary="$root/target/release/examples/steady"

# The budget entry #25 names. A **report**, so this script does not exit non-zero on it — see the
# note at the bottom, which says what would have to be true for that to change.
readonly BUDGET_PERCENT="5"

cd "$root"
echo "building the steady-state probe (release, so the numbers are the shipped ones)"
cargo build --release --example steady -p vitui-engine

if [ ! -x "$binary" ]; then
  echo "no binary at $binary after a successful build" >&2
  exit 1
fi

measured="$(mktemp)"
trap 'rm -f "$measured"' EXIT

if /usr/bin/time -l true >/dev/null 2>&1; then
  flavour=bsd
  flag=-l
elif /usr/bin/time -v true >/dev/null 2>&1; then
  flavour=gnu
  flag=-v
else
  echo "/usr/bin/time understands neither -l nor -v: this report needs one of them" >&2
  exit 1
fi

echo "measuring ${seconds}s at 60 Hz with /usr/bin/time $flag ($flavour)"
# The probe's own two lines go to stdout and `time`'s report to stderr, so `frames=` is read from one
# and the CPU figures from the other.
probe="$(mktemp)"
trap 'rm -f "$measured" "$probe"' EXIT
# **The probe's stderr is `time`'s file, so a panic lands where only awk looks.** On 2026-09-10 a
# hosted macOS runner could not sustain 60 Hz, `examples/steady.rs` asserted its frame-count floor,
# and this line exited 101 under `set -e` — before `cat "$probe"`, with the message sitting in
# `$measured` until the EXIT trap deleted it. The run reported an exit code and no reason. An
# instrument that hides why it failed is the same defect as one that cannot fail, so the failure
# path prints what it captured.
# `|| status=$?` and not `if ! ...`, where `$?` would be the negation's and always 0.
status=0
/usr/bin/time $flag "$binary" "$seconds" >"$probe" 2>"$measured" || status=$?
if [ "$status" -ne 0 ]; then
  echo "STEADY REPORT FAILED: the probe exited $status. What it and \`time\` wrote:" >&2
  cat "$probe" >&2
  cat "$measured" >&2
  exit "$status"
fi
cat "$probe"

case "$flavour" in
  bsd)
    real="$(awk '/ real / { for (i = 1; i <= NF; i++) if ($i == "real") print $(i - 1) }' "$measured" | head -1)"
    user="$(awk '/ real / { for (i = 1; i <= NF; i++) if ($i == "user") print $(i - 1) }' "$measured" | head -1)"
    sys="$(awk '/ real / { for (i = 1; i <= NF; i++) if ($i == "sys") print $(i - 1) }' "$measured" | head -1)"
    switches="$(awk '/voluntary context switches/ { print $1 }' "$measured" | head -1)"
    ;;
  gnu)
    real="$(awk -F': ' '/Elapsed \(wall clock\) time/ { print $2 }' "$measured" | head -1 | awk -F: '{ if (NF == 2) print $1 * 60 + $2; else print $1 }')"
    user="$(awk -F': ' '/User time \(seconds\)/ { print $2 }' "$measured" | head -1)"
    sys="$(awk -F': ' '/System time \(seconds\)/ { print $2 }' "$measured" | head -1)"
    switches="$(awk -F': ' '/Voluntary context switches/ { print $2 }' "$measured" | head -1)"
    ;;
esac

frames="$(awk -F'[= ]' '/^frames=/ { print $2 }' "$probe" | head -1)"

# **An unread instrument is a failure and not a pass**, even in a report: a report whose number is
# empty is indistinguishable from one whose number is zero, and the second is the answer somebody
# would like to hear.
for field in real user sys switches frames; do
  if [ -z "${!field:-}" ]; then
    echo "STEADY REPORT FAILED: could not read \`$field\` out of the instruments" >&2
    exit 1
  fi
done

percent="$(awk -v u="$user" -v s="$sys" -v r="$real" 'BEGIN { printf "%.3f", (u + s) / r * 100 }')"
headroom="$(awk -v p="$percent" -v b="$BUDGET_PERCENT" 'BEGIN { if (p > 0) printf "%.1f", b / p; else print "inf" }')"
per_frame="$(awk -v u="$user" -v s="$sys" -v f="$frames" 'BEGIN { printf "%.1f", (u + s) / f * 1e6 }')"
resolution="$(awk -v r="$real" 'BEGIN { printf "%.3f", 0.01 / r * 100 }')"

cat <<REPORT

report #25  60 fps steady state, MEASURED
            ${frames} frames over ${real}s · ${user}s user + ${sys}s sys · ${switches} voluntary context switches
            ${percent}% of one core against a ${BUDGET_PERCENT}% budget — ${headroom}x of headroom
            ${per_frame} us of process CPU a frame, all threads, wire included
            instrument floor at this window is ${resolution}% (/usr/bin/time reports hundredths)

            **A report, and it does not fail this script.** §14: a timing is a report and a gate only
            at cliff granularity. The count underneath it is the gate, and it is inside
            examples/steady.rs — a run that painted a tenth of the frames aborts there rather than
            reporting a tenth of the CPU here. What would make this a gate is a cliff: 5% is 50x the
            measured figure, so a threshold anywhere near the budget would be a gate that cannot
            fail, and one near the measurement would be a stopwatch on a shared runner.
REPORT
