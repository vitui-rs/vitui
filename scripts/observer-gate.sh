#!/usr/bin/env bash
#
# Register entry #18: **no observer thread is linked into a release binary.**
#
# Spec §11's fifth rung is the only one that costs a wakeup. Measured over thirty seconds, a process
# with no observer takes 10 context switches, a 100 ms poll takes 369 and a 10 ms poll 2 792 — CPU is
# not the objection, wakeups are, and standing requirement 11 names that property literally. So the
# observer is `cfg(debug_assertions)` and requirement 11 stays a **release-build** property.
#
# That makes the entry a *compile outcome*, and this script is how a compile outcome about absent code
# is checked: the observer's own sanction message is a string literal in the only function that
# prints it, so the literal is in the binary if and only if the code is. `strings` is not consulted
# for elegance — it is consulted because a symbol name can be inlined away and a message the process
# has to be able to print cannot.
#
# **Both directions, and the second one is the point.** A grep that finds nothing passes for any
# reason at all, including the message having been reworded, the example failing to link the engine,
# or this script pointing at a path that no longer exists. So the release binary must NOT contain it
# and the debug binary MUST, and a run where either half disagrees is a failure.
#
#   scripts/observer-gate.sh
#
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

# A phrase from `crate::perf::stall_report`, which is `cfg(debug_assertions)` along with everything
# that calls it. Long enough not to collide with anything else in the binary, and short enough to
# survive the message being re-wrapped.
readonly MARKER="has not come back"

# `examples/idle` rather than a test binary: it is an ordinary application that attaches on
# `Clock::System`, which is the configuration the observer exists for, and it is already built by the
# idle gate beside this one.
readonly EXAMPLE=idle

fail=0
note() { echo "OBSERVER GATE FAILED: $*" >&2; fail=1; }

# `strings` is not on every minimal image; `grep -a` on the whole binary does the same job here and is
# in coreutils. `-c` rather than `-q` so the count is in the log when a half disagrees.
#
# **`|| true` was here and was a hole.** `grep` exits 1 for *no match* and 2 for *could not read the
# file*, and swallowing both made an unreadable binary look like a clean release: the substitution came
# back empty, `[ "$found" -eq 0 ]` errored, the branch was taken as false, and the gate printed
# "the observer is linked in ( match(es))" and passed having checked nothing. Which is precisely the
# failure the header above sets out to prevent, arriving through the error handling rather than through
# the grep. So exit 1 is a real zero and exit 2 is fatal.
occurrences() {
  local out status
  out="$(grep -a -c -- "$MARKER" "$1")" && status=0 || status=$?
  case "$status" in
    0) printf '%s' "$out" ;;
    1) printf '0' ;;
    *) return "$status" ;;
  esac
}

for profile in debug release; do
  case "$profile" in
    debug) cargo build --example "$EXAMPLE" -p vitui-engine ;;
    release) cargo build --release --example "$EXAMPLE" -p vitui-engine ;;
  esac
  binary="$root/target/$profile/examples/$EXAMPLE"
  if [ ! -x "$binary" ]; then
    note "no binary at $binary after a successful $profile build"
    continue
  fi
  if ! found="$(occurrences "$binary")"; then
    note "grep could not read $binary, so neither half of this gate was checked"
    continue
  fi
  # A non-numeric result would make every comparison below silently false. There is no path left that
  # produces one, and this is what says so rather than assuming it.
  case "$found" in
    ''|*[!0-9]*)
      note "the match count for $binary came back as \"$found\", which is not a number"
      continue
      ;;
  esac
  case "$profile" in
    debug)
      if [ "$found" -eq 0 ]; then
        note "the debug binary does not contain \"$MARKER\" — either the observer is gone from \
every profile, or the message was reworded and this gate is now checking nothing"
      else
        echo "debug:   the observer is linked in ($found match(es) for \"$MARKER\")"
      fi
      ;;
    release)
      if [ "$found" -ne 0 ]; then
        note "the release binary contains \"$MARKER\" ($found match(es)) — an observer thread is \
linked into a release build, and requirement 11's zero wakeups is no longer a release property"
      else
        echo "release: the observer is absent, and so are its words"
      fi
      ;;
  esac
done

if [ "$fail" -ne 0 ]; then
  exit 1
fi

echo "register #18: no observer thread is linked into a release binary"
