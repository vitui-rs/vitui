#!/usr/bin/env bash
#
# Register entry #17, the process half: **an idle application costs nothing at all.**
#
# The count half is a unit test — `an_idle_application_parks_once_and_wakes_for_nothing` — and it
# proves the app thread parks once and comes back zero times. What a unit test cannot say is
# `0.00 user 0.00 sys`, because that is a claim about a *process*: libtest's harness is in the same
# one, and so is cargo if you go through `cargo run`. So this builds first and then runs the binary
# directly, with `/usr/bin/time` around it and nothing else.
#
# **It is a two-runner job, not a matrix one.** `/usr/bin/time` is two different programs with two
# different flags: BSD's `-l` on macOS and GNU's `-v` on Linux. Both report the two numbers this
# gate is about, under different labels, and the parsing below is per-flavour rather than shared.
#
# Thirty seconds is a floor rather than a preference. At three seconds the difference between a
# parked application and a 120 Hz ticker is below the tool's resolution — 0.00 either way. At thirty
# it is zero against 3 600 wakeups.
#
#   scripts/idle-gate.sh [seconds]
#
set -euo pipefail

seconds="${1:-30}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
binary="$root/target/release/examples/idle"

# The ceiling on what an idle process may spend. Not zero: the exit path prints two lines and the
# process has to start, and a runner that is swapping can charge a few milliseconds of system time
# for the address-space teardown. A ticker would spend orders more.
readonly MAX_USER_SECONDS="0.10"
readonly MAX_SYS_SECONDS="0.10"
# Two for the watchdog — one sleep, one quit — plus the app thread's park, the render thread's park,
# and whatever the loader does on the way in. A 120 Hz ticker over thirty seconds would put 3 600
# here, so the bound is three orders of magnitude away from what it is guarding against.
readonly MAX_VOLUNTARY_SWITCHES=60

cd "$root"
echo "building the idle probe (release, so the numbers are the shipped ones)"
cargo build --release --example idle -p vitui-engine

if [ ! -x "$binary" ]; then
  echo "no binary at $binary after a successful build" >&2
  exit 1
fi

measured="$(mktemp)"
trap 'rm -f "$measured"' EXIT

# Which `time` is this. `-l` is BSD's and `-v` is GNU's, and each one errors on the other's flag.
if /usr/bin/time -l true >/dev/null 2>&1; then
  flavour=bsd
  flag=-l
elif /usr/bin/time -v true >/dev/null 2>&1; then
  flavour=gnu
  flag=-v
else
  echo "/usr/bin/time understands neither -l nor -v: this gate needs one of them" >&2
  exit 1
fi

echo "measuring ${seconds}s of idle with /usr/bin/time $flag ($flavour)"
# `time` writes its report to stderr and the probe writes its own lines to stdout, so both are kept
# and the report is the one that is parsed.
/usr/bin/time $flag "$binary" "$seconds" 2>"$measured"
cat "$measured"

fail=0
note() { echo "IDLE GATE FAILED: $*" >&2; fail=1; }

# `awk` rather than `bc`: the runner images have one and not always the other.
over() { awk -v a="$1" -v b="$2" 'BEGIN { exit (a > b) ? 0 : 1 }'; }

case "$flavour" in
  bsd)
    # `        30.00 real         0.00 user         0.00 sys`
    user="$(awk '/ real / { for (i = 1; i <= NF; i++) if ($i == "user") print $(i - 1) }' "$measured" | head -1)"
    sys="$(awk '/ real / { for (i = 1; i <= NF; i++) if ($i == "sys") print $(i - 1) }' "$measured" | head -1)"
    switches="$(awk '/voluntary context switches/ { print $1 }' "$measured" | head -1)"
    ;;
  gnu)
    user="$(awk -F': ' '/User time \(seconds\)/ { print $2 }' "$measured" | head -1)"
    sys="$(awk -F': ' '/System time \(seconds\)/ { print $2 }' "$measured" | head -1)"
    switches="$(awk -F': ' '/Voluntary context switches/ { print $2 }' "$measured" | head -1)"
    ;;
esac

# An unparsed field is a failure and not a pass. A gate that cannot read its own instrument is a gate
# that cannot fail, which §14 names exactly: a flaky test wearing a budget's clothes.
[ -n "${user:-}" ] || note "could not read user time out of the report"
[ -n "${sys:-}" ] || note "could not read system time out of the report"
[ -n "${switches:-}" ] || note "could not read the voluntary context switch count"

if [ -n "${user:-}" ] && over "$user" "$MAX_USER_SECONDS"; then
  note "user time ${user}s over ${seconds}s of idle, against a ceiling of ${MAX_USER_SECONDS}s"
fi
if [ -n "${sys:-}" ] && over "$sys" "$MAX_SYS_SECONDS"; then
  note "system time ${sys}s over ${seconds}s of idle, against a ceiling of ${MAX_SYS_SECONDS}s"
fi
if [ -n "${switches:-}" ] && [ "$switches" -gt "$MAX_VOLUNTARY_SWITCHES" ]; then
  note "$switches voluntary context switches, against a ceiling of $MAX_VOLUNTARY_SWITCHES — \
something is waking the app thread"
fi

if [ "$fail" -ne 0 ]; then
  exit 1
fi

echo "idle over ${seconds}s: ${user}s user, ${sys}s sys, ${switches} voluntary context switches"
echo "a 120 Hz ticker would have put $((120 * seconds)) in the last number"
