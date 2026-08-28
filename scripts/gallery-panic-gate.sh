#!/usr/bin/env bash
#
# Components register row 221: **the gallery gives the terminal back before a panic prints, under a
# real pty.**
#
# The mechanism is the engine's — `crates/vitui-engine/src/shutdown.rs` installs a panic hook that
# restores first and then lets the default hook print, and it is idempotent under one atomic so a
# panic on the render thread and a panic on the app thread take the same path. Engine ticket 22 built
# it and its own gates check it over a recorder.
#
# **What this script adds is that it happens in this binary.** A restoration nobody has watched
# happen on a real terminal is a restoration nobody can trust, and there are two ways for an
# application to lose it that no in-process test can see: the process does not go through
# `Screen::drop` or the hook at all, and the epilogue is written after the backtrace — which puts the
# backtrace on a page the terminal is about to discard, at the one moment a developer needs it.
#
# So the gate is an **order**, on captured bytes, from a process that really panicked:
#
#   the leave-alt-screen sequence appears, AND it appears before the panic message.
#
# **Both directions.** A grep that finds nothing passes for any reason at all, so the epilogue must be
# present, the panic message must be present, and the epilogue must come first. A run where any of the
# three disagrees is a failure.
#
#   scripts/gallery-panic-gate.sh
#
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

# `CSI ? 1 0 4 9 l` — mode 1049 reset, which is *leave the alternate screen, restore the cursor*. It
# is the one byte sequence a human can see the absence of, and `crates/vitui-engine/src/actuate.rs`
# is where it is written.
# **The `?` is escaped, and it is the whole point of the needle.** In an ERE `\[?` is an *optional*
# literal bracket, so the pattern would reduce to `1049l` and the gate whose subject is byte-exactness
# would match any capture containing that substring — `hello 1049l world` included. `\[\?` is the two
# literals `CSI ?` this sequence really starts with.
readonly LEAVE_ALT='\[\?1049l'
# The application's own panic, which exists for this gate and for nothing else.
readonly PANIC_TEXT='the panic gate.s own panic'

readonly OUT="${TMPDIR:-/tmp}/vitui-gallery-panic.$$"
trap 'rm -f "$OUT"' EXIT

fail=0
note() { echo "GALLERY PANIC GATE FAILED: $*" >&2; fail=1; }

cargo build --example gallery -p vitui-apps

# **`CARGO_TARGET_DIR` and not a hardcoded `target/`.** With one set, a hardcoded path is a missing
# binary and this gate fails as *the pty capture is empty* — a failure whose message names the wrong
# thing, which is the shape the header above sets out to avoid.
readonly binary="${CARGO_TARGET_DIR:-$root/target}/debug/examples/gallery"
if [ ! -x "$binary" ]; then
  note "no binary at $binary after a successful build"
  exit 1
fi

# **A real pty, because `Driver::attach` refuses a pipe** — and because the whole subject is what the
# terminal is left in. `script` is the portable way to get one and its two spellings are not
# compatible: BSD (macOS) takes the command as arguments after the file, util-linux takes it after
# `-c`. Neither is a preference; a gate that guessed would be a gate that only runs on one machine.
if script -q /dev/null true </dev/null >/dev/null 2>&1; then
  script -q "$OUT" "$binary" --panic </dev/null >/dev/null 2>&1 || true
elif script -q -c true /dev/null </dev/null >/dev/null 2>&1; then
  script -q -c "$binary --panic" "$OUT" </dev/null >/dev/null 2>&1 || true
else
  note "no usable \`script\` on this machine, so the pty half was not checked at all"
  exit 1
fi

if [ ! -s "$OUT" ]; then
  note "the pty capture is empty, so nothing was checked"
  exit 1
fi

# `grep -a -b -o` gives a byte offset per match, which is what makes *before* a number rather than an
# impression. The first match of each is the one that matters: the epilogue is written once under a
# one-shot guard and the message once by the default hook.
offset_of() {
  local out status
  out="$(grep -a -b -o -m1 -E -- "$1" "$OUT")" && status=0 || status=$?
  case "$status" in
    0) printf '%s' "${out%%:*}" ;;
    1) printf '' ;;
    *) return "$status" ;;
  esac
}

alt_at="$(offset_of "$LEAVE_ALT" || true)"
panic_at="$(offset_of "$PANIC_TEXT" || true)"

if [ -z "$alt_at" ]; then
  note "the capture carries no mode-1049 reset, so the gallery left the terminal on the alternate \
screen — a shell whose scrollback is gone and whose prompt is on a page nobody can see"
fi
if [ -z "$panic_at" ]; then
  note "the capture carries no panic message, so the process did not panic and this gate checked \
nothing. \`--panic\` is the flag, and it exists for this script"
fi

if [ -n "$alt_at" ] && [ -n "$panic_at" ]; then
  if [ "$alt_at" -lt "$panic_at" ]; then
    echo "restored at byte $alt_at, panicked at byte $panic_at — the terminal came back first"
  else
    note "the panic message is at byte $panic_at and the restoration at byte $alt_at, so the \
backtrace was painted into the alternate screen and vanished with it"
  fi
fi

if [ "$fail" -ne 0 ]; then
  exit 1
fi

echo "components register 221: the gallery restores the terminal before a panic prints"
