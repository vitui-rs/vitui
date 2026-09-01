#!/usr/bin/env bash
#
# Register entry #31, on a **real pty**: a suspension does not vacate standard input.
#
# `Screen::suspend` writes the epilogue, joins the render thread and leaves raw mode. It does **not**
# stop the `vitui-pty` thread, which is an unconditional `loop { stdin.read(..) }` that safe Rust
# cannot cancel — so for the whole of a suspension this process is still one of the readers of the
# user's terminal. ADR 0052 prices the four ways it could be made cancellable and refuses all four;
# `Screen::suspend`'s first paragraph is where a caller meets the consequence, and `Screen::resume`'s
# discard is what the engine does about the half that is decidable.
#
# **Four documents claimed it and nothing watched it**, which is production ticket 13 — the sentence
# was in `suspend`'s doc from the day the pair was built and was read as a caveat rather than as a
# refusal, and an arm that cannot work was written against it, reviewed, and removed.
#
# # Why this cannot be a `cargo test`
#
# `crate::detect::Tty::open` panics under `cfg(test)`, so no test in this workspace ever spawns the
# reader. Every keystroke a headless gate sees was put into the queue by `Screen::inject`, which is
# on this side of the reader and cannot say whether the reader exists — which is why
# `crate::gates::a_resume_delivers_no_keystroke_from_the_suspension_and_every_resize` gates *what
# happens to the bytes* and is silent about where they came from. `script(1)` is the terminal here
# and this file is the hand that types, exactly as `scripts/page-order-gate.sh` is for entry #30.
#
# # It is a tripwire, and its polarity is deliberate
#
# This gate asserts a **refusal**. Make the read cancellable and it goes red — and that is what it is
# for: the doc paragraph, the discard in `resume`, ADR 0052's section and production ticket 13's
# answer all have to move in the same edit, and a red gate is what puts them in front of whoever
# makes the change. Nothing here is a bug to be fixed by making this script pass.
#
# # Watch it fail
#
# Count the reads in `crate::detect::Tty::open`'s thread and `break` at **two**, which is a reader
# that lives exactly as long as this run's detection and control arm and then stops. Measured on
# 2026-09-01: `control=ok`, `during=timeout`, and the message below. Stopping it at *one* read fails
# the control arm instead and says nothing about the subject — which is what the control arm is for.
#
#   scripts/suspend-reader-gate.sh
#
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

readonly EXAMPLE=suspend_reader

# **`TERM` is pinned, so that every run takes the same path through `attach`.** One value of it takes
# a different one: `attach` asks a `TERM=dumb` terminal nothing at all — level 4 of spec §10's
# precedence overrules every answer detection could get — so the capability batch never goes out and
# the session this gate suspends was built without a detection round trip. A CI container leaves
# `TERM` unset and an operator's shell can hold anything, and *which shape of session the reader
# belongs to* is not a thing this gate should discover by accident. The value is otherwise inert
# here: nothing in `quirks.rs` matches on it alone, and the harness answers only DA1, so every
# capability stays at its conservative default either way.
export TERM=xterm-256color

cargo build -p vitui-engine --example "$EXAMPLE"
# `CARGO_TARGET_DIR` and not a hardcoded `target/`, which is the rule the other two pty gates here
# already keep: with one set, a hardcoded path is a build that succeeded and a binary that is not
# there, arriving as a red gate about the engine.
readonly BINARY="${CARGO_TARGET_DIR:-$root/target}/debug/examples/$EXAMPLE"
if [ ! -x "$BINARY" ]; then
  echo "SUSPEND READER GATE FAILED: no binary at $BINARY after a successful build" >&2
  exit 1
fi

# Bare `mktemp`, because `-t NAME` is two different commands on the two platforms this runs on. The
# verdict file is a *name* rather than a file: the example writes it last, after the `Screen` is
# dropped, so its absence is an instrument that never finished and its presence is a terminal that
# was given back.
typescript="$(mktemp)"
verdict="$(mktemp)"
rm -f "$verdict"
trap 'rm -f "$typescript" "$verdict"' EXIT

# How long to wait for a marker before giving up on the whole run. The example's own patience is
# fifteen seconds per arm, so this is above the sum: whichever side is stuck, the example is the one
# that should say so, because it knows which arm it was in.
readonly DEADLINE=40

# Wait for something to appear in the typescript. **Polling a file and not sleeping a guess** — the
# two keystrokes this gate is about are typed in response to a marker the example printed, so there
# is no window to get wrong and no arm that is fast on a laptop and slow in a container. The one
# thing written without waiting is the DA1 reply below, and it is written *first* precisely so that
# it has no deadline to be late for.
#
# `-a`, because the typescript is a terminal recording and `grep` would otherwise answer *binary file
# matches* on some of these and nothing on others depending on where the escape sequences fell.
#
# Every byte of stdout in this function goes to the pty, so it must produce none: `grep -q` is silent
# and the `sleep` returns nothing. That is not a style note — a stray `echo` here is a keystroke.
wait_for() {
  local needle="$1" waited=0
  while [ "$waited" -lt "$((DEADLINE * 10))" ]; do
    if grep -qa -- "$needle" "$typescript" 2>/dev/null; then
      return 0
    fi
    sleep 0.1
    waited=$((waited + 1))
  done
  return 1
}

# The hand that types, on the write end of the pipe `script` reads as its standard input.
#
# It must **not** close that pipe while the example is still running: an end-of-file on standard
# input is `read` returning `Ok(0)`, which is the channel closing, which is a `Wake::Quit` — the
# engine's answer to a terminal that went away. So the last thing it does is wait for the verdict.
feed() {
  # **First it has to be a terminal, and `script(1)` is not one.** A pty is a pair of descriptors,
  # not an emulator: nothing on the far side answers a query, so `detect` reads nothing inside its
  # ceiling and `attach` fails with `AttachError::NoAnswer` — no session, nothing to suspend.
  # (`scripts/page-order-gate.sh` never meets this because it closes standard input, and `Tty::open`
  # asks about *both* ends: a non-tty stdin means no detection at all.)
  #
  # So the harness answers the sentinel. DA1 is the only reply that matters — `crate::detect::csi`
  # treats a `c` that does not start with `>` as the end of the exchange — and every other question
  # stays at its conservative default, which is exactly right for a terminal that is not one.
  #
  # **Written first and unprompted, and that is what keeps this off a clock.** The obvious shape is
  # to wait for the batch in the typescript and then reply, and it is a race: `detect::CEILING` is
  # 250 ms, it is an *idle* deadline that nothing resets because nothing else ever arrives, and a
  # poll loop that forks a `grep` has to fit inside it — on a loaded shared runner that is a gate
  # reporting `attach=failed` about the engine because the harness was late. Writing the reply before
  # anything asks for it has no deadline in it at all, and every ordering works: land before raw
  # mode and the line discipline holds it until the newline below releases it into the same input
  # queue; land after raw mode and before the batch and the reader has it waiting when `detect`
  # first reads; land after the batch and it is the ordinary case. The stray newline is handed back
  # as type-ahead and reaches the application as one `Enter`, which nothing here looks at.
  printf '\033[?1;2c\n' 2>/dev/null || true
  wait_for '\[MARK\] armed' || return 0
  printf 'a' 2>/dev/null || true
  wait_for '\[MARK\] suspended' || return 0
  # **The newline is not decoration.** `suspend` left raw mode, so the pty's line discipline is
  # canonical again and holds a bare `b` in the kernel's line buffer until one arrives. A gate that
  # typed `b` alone would report `during=timeout` — the reader stopped — about a terminal that had
  # simply not been told the line was over.
  printf 'b\n' 2>/dev/null || true
  local waited=0
  while [ "$waited" -lt "$((DEADLINE * 10))" ] && [ ! -f "$verdict" ]; do
    sleep 0.1
    waited=$((waited + 1))
  done
}

# **macOS and Linux spell `script` differently**, and the branch is on `uname` rather than on a trial
# run for the reason `scripts/page-order-gate.sh` records: which `script` is installed is a fact about
# the platform, and a probe that allocates a pty to find it out has its own failure mode. BSD takes
# the file first and then the command; util-linux needs `-c` and the command as one string.
#
# **`-F` / `-f` is load-bearing here in a way it is not for `scripts/page-order-gate.sh`**, and it
# cost an hour. That gate reads the typescript *after* `script` exits, where every buffer is flushed
# anyway; this one reads it while the session is running, and BSD `script`'s default flush interval
# is **thirty seconds**. Without the flag the file is empty for the whole of a run that takes under a
# second, every `wait_for` times out, nothing is ever typed, and the failure that comes back is
# `attach=failed` — a message about the engine, produced by a recorder that had not written anything
# down yet.
#
# **A watchdog around the whole thing, where the platform has one.** `feed` gives up after
# `DEADLINE` and the example bounds each of its two arms with `PATIENCE`, and between them they still
# do not cover `attach` or the render-thread join inside `drop(screen)` — so a wedge there is a gate
# that hangs, which this repository's own note on `retry` calls worse than one that fails. `timeout`
# is coreutils and is on the CI image; macOS has it only as `gtimeout` from Homebrew, and where
# neither exists this runs unbounded and says so rather than pretending otherwise.
watchdog=""
for candidate in timeout gtimeout; do
  if command -v "$candidate" >/dev/null 2>&1; then
    watchdog="$candidate 90"
    break
  fi
done
if [ -z "$watchdog" ]; then
  echo "note: no timeout(1) or gtimeout(1) on this machine, so a wedged example hangs this gate \
rather than failing it" >&2
fi

case "$(uname -s)" in
  Darwin|*BSD*)
    feed | $watchdog script -q -F "$typescript" "$BINARY" "$verdict" >/dev/null 2>&1 || true
    ;;
  *)
    # **Quoted inside the string, because `-c` takes one shell command and not an argv.** Both paths
    # are ours — a `mktemp` name and a cargo target directory — so a space in `TMPDIR` or
    # `CARGO_TARGET_DIR` would otherwise split into the wrong argv and the failure would arrive as
    # *the example wrote no verdict*, which reads as the engine's fault. A path containing a single
    # quote is out of reach and out of scope.
    feed | $watchdog script -q -e -f -c "'$BINARY' '$verdict'" "$typescript" >/dev/null 2>&1 || true
    ;;
esac

if [ ! -s "$typescript" ]; then
  echo "SUSPEND READER GATE FAILED: the typescript is empty, so no pty was allocated and nothing \
was checked. That is this gate failing to be an instrument rather than the engine failing a \
property" >&2
  exit 1
fi

if [ ! -f "$verdict" ]; then
  echo "SUSPEND READER GATE FAILED: the example wrote no verdict, so it never reached the end of \
its own protocol. The first 2 000 bytes of the typescript are the only evidence left:" >&2
  # **Every byte that is not printable becomes a dot**, which is `scripts/page-order-gate.sh`'s rule
  # and for its reason: this is a terminal recording, and echoing one raw into a failure message
  # reprograms the terminal of whoever is reading the failure.
  head -c 2000 "$typescript" | LC_ALL=C tr -c '\11\12\40-\176' '.' >&2
  echo >&2
  exit 1
fi

value() { sed -n "s/^$1=//p" "$verdict"; }

attach="$(value attach)"
control="$(value control)"
during="$(value during)"

if [ "$attach" != "ok" ]; then
  echo "SUSPEND READER GATE FAILED: the engine did not attach to the pty, so there was no session \
to suspend: $(value why)" >&2
  exit 1
fi

# **The control arm, and it is what stops the subject from being vacuous.** Without it a pty that
# delivers no keystroke at all — a `script` that never allocated a terminal, a feeder whose pipe was
# closed — reports exactly what a stopped reader reports.
if [ "$control" != "ok" ]; then
  echo "SUSPEND READER GATE FAILED: no keystroke reached the application before the suspension, so \
this run says nothing about the reader. The control arm is the instrument checking itself: the pty, \
the feeder and the parser all have to work for a plain 'a' to arrive" >&2
  exit 1
fi

if [ "$during" != "ok" ]; then
  echo "SUSPEND READER GATE FAILED: nothing typed during the suspension reached the application, so \
the reader stopped ($during). That is a change to a documented refusal and not a defect this gate \
found: ADR 0052's 'The reader does not stop', engine spec section 7, \`Screen::suspend\`'s first \
paragraph, \`Queue::drop_what_the_user_typed\` and register entry #31 all state it, and \
\`Screen::resume\`'s discard exists only because of it. Move them in the same edit" >&2
  exit 1
fi

echo "a keystroke before the suspension arrived, and so did one during it, on a real pty"
echo "register #31: a suspension does not vacate standard input"
