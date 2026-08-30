#!/usr/bin/env bash
#
# Register entry #30, on a **real pty**: nothing this engine sends reaches the user's own page.
#
# `crate::gates::nothing_reaches_the_users_own_page` is the same property over the two functions that
# produce the bytes, and it is the gate CI runs first. It cannot run the path those bytes take:
# `Tty::open` panics under `cfg(test)`, so no test in this workspace ever puts a real terminal into
# raw mode, and *what `attach` writes to a tty, in what order* is unreachable from inside the crate.
#
# This is that path. `script(1)` gives the process a pty, so `Tty::open` succeeds, detection fires,
# and the typescript is every byte the engine wrote in the order it wrote them.
#
# # The property, and why it is *the first escape sequence* rather than the first byte
#
# The pty's line discipline echoes before the application takes raw mode — the end-of-file `script`
# forwards from its own closed standard input lands in the typescript as `^D` and two backspaces.
# Those are the **terminal's** bytes and not the engine's, and none of them is an escape sequence. So
# the anchor is the first `ESC` in the stream: everything the engine says starts there, and `?1049h`
# has to be it.
#
# Four assertions, and the fourth is what stops the middle two from being vacuous:
#
#   1. the first escape sequence in the typescript is `ESC [ ? 1 0 4 9 h`;
#   2. it appears exactly **once** — a second one on a terminal without xterm's already-on-the-
#      alternate-buffer guard saves the cursor again and gives the shell back at the wrong one;
#   3. `?1049l` appears exactly once, so the page this took is the page it gave back;
#   4. **`?7l` is in the stream**, which is `crate::actuate::negotiation` and nothing else. Without
#      it, an `attach` that failed at detection would leave a typescript holding only the batch and
#      its `Tty::drop` epilogue — where counts 2 and 3 hold whatever `Page` decides, because the
#      negotiation never ran. That is this repository's "a gate that cannot fail", and the review
#      that found it was right that nothing here proved the attach completed.
#
# **Watch it fail.** Move `?1049h` out of `crate::detect::batch` and back into
# `crate::actuate::negotiation`'s unconditional arm and count 1 breaks with the whole capability
# batch — `+q524742` and seven DECRQMs — ahead of it, naming **277 bytes**. Keep the batch's `?1049h`
# and make the negotiation's unconditional and count 2 breaks with two.
#
#   scripts/page-order-gate.sh
#
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

# `caps` and not a drawing application, for the reason production ticket 12 was filed against it: it
# attaches, reads `Capabilities::report`, detaches and prints. No frame, no render thread's bytes, no
# input — so the typescript is the prologue and the epilogue and almost nothing else, and a failure
# message is short enough to read.
readonly EXAMPLE=caps

cargo build -p vitui-apps --example "$EXAMPLE"
# **`CARGO_TARGET_DIR` and not a hardcoded `target/`**, which is `scripts/gallery-panic-gate.sh`'s
# rule and for its reason: with one set, a hardcoded path is a build that succeeded and a binary that
# is not there, which arrives as a red gate about the engine.
readonly BINARY="${CARGO_TARGET_DIR:-$root/target}/debug/examples/$EXAMPLE"
if [ ! -x "$BINARY" ]; then
  echo "PAGE ORDER GATE FAILED: no binary at $BINARY after a successful build" >&2
  exit 1
fi

# Bare `mktemp`, because `-t NAME` is two different commands: BSD appends `.XXXXXXXX` to it, GNU
# requires the template to carry at least three `X`s of its own and fails without them. This
# repository's runners are one of each, and the other four scripts here already write it this way.
typescript="$(mktemp)"
trap 'rm -f "$typescript"' EXIT

# **Standard input comes from `/dev/null`, and that is a real failure this gate had.** BSD `script`
# copies the terminal settings of *its own* standard input onto the pty it allocates, so it calls
# `tcgetattr` on whatever it inherits — and under a non-interactive parent that can be a socket, where
# the call fails with `Operation not supported on socket`, `script` exits 1 and writes a zero-byte
# typescript. The gate then reported an empty recording, which is true and useless, and did it
# intermittently, which is how a gate gets disabled. A closed standard input is also what this
# measurement wants for its own sake: `caps` reads no keys, and an inherited terminal would put the
# operator's keystrokes into the typescript beside the engine's bytes.
#
# **macOS and Linux spell the rest of it differently and the difference is not cosmetic.** BSD
# `script` takes the file first and then the command; util-linux needs `-c` and the command as one
# string. Both are asked for a raw typescript with no header.
#
# Branched on `uname` rather than on a trial run of `script`, which `scripts/gallery-panic-gate.sh`
# beside this does. Both work — that gate closes the probe's own standard input, which is exactly what
# makes a trial run reliable — and this asks the question directly instead, because *which `script` is
# installed* is a fact about the platform and a probe that allocates a pty to find it out has the
# failure mode above. This gate took the util-linux path on macOS while it was being written, on the
# strength of a probe that had failed for that reason and not for the reason it was read as.
case "$(uname -s)" in
  Darwin|*BSD*) script -q "$typescript" "$BINARY" </dev/null >/dev/null 2>&1 || true ;;
  *) script -q -e -c "$BINARY" "$typescript" </dev/null >/dev/null 2>&1 || true ;;
esac

if [ ! -s "$typescript" ]; then
  echo "PAGE ORDER GATE FAILED: the typescript is empty, so nothing was recorded and nothing was \
checked. That is this gate failing to be an instrument rather than the engine failing a property — \
every defect it can find puts more bytes on the wire, never none" >&2
  exit 1
fi

# **`od` into `awk`, and not `python3`.** The `rust:*` images CI runs on do not carry python, which
# `scripts/lint-rung-gate.sh` already records in as many words: *a gate that cannot run in CI is not a
# gate.* The subject here is bytes and an offset, so the file is turned into one unsigned byte per
# line and the search is a state machine over numbers — which is exact, needs no locale, and cannot be
# confused by an `ESC` in the middle of a record the way a line-oriented tool would be. `-v` so runs
# of identical bytes are not elided into `*`, which would silently move every offset after one.
od -An -v -tu1 "$typescript" | tr -s ' ' '\n' | grep -v '^$' | LC_ALL=C awk '
  { byte[++n] = $1 + 0 }

  # Does the pattern in p[] start at byte i?
  function at(i, p, len,    k) {
    for (k = 0; k < len; k++) if (byte[i + k] != p[k + 1]) return 0
    return 1
  }
  # The bytes from i, rendered so a failure message does not reprogram the reader own terminal.
  function shown(i, howmany,    k, v, s) {
    s = ""
    for (k = i; k < i + howmany && k <= n; k++) {
      v = byte[k]
      s = s ((v >= 32 && v < 127) ? sprintf("%c", v) : sprintf("\\x%02x", v))
    }
    return s
  }

  END {
    split("27 91 63 49 48 52 57 104", enter, " ")   # ESC [ ? 1 0 4 9 h
    split("27 91 63 49 48 52 57 108", leave, " ")   # ESC [ ? 1 0 4 9 l
    split("27 91 63 55 108", nowrap, " ")           # ESC [ ? 7 l — the negotiation, and only it

    first = 0
    for (i = 1; i <= n; i++) if (byte[i] == 27) { first = i; break }
    if (first == 0) {
      print "PAGE ORDER GATE FAILED: the typescript holds no escape sequence at all, so `attach` \
never reached a terminal: " shown(1, 60) > "/dev/stderr"
      exit 1
    }

    entered = 0; left = 0; negotiated = 0; where = 0
    for (i = 1; i <= n; i++) {
      if (at(i, enter, 8)) { entered++; if (where == 0) where = i }
      if (at(i, leave, 8)) left++
      if (at(i, nowrap, 5)) negotiated++
    }

    if (!at(first, enter, 8)) {
      print "PAGE ORDER GATE FAILED: the first escape sequence on the wire is not `?1049h`, so " \
        (where > 0 ? where - first : n - first + 1) " bytes reached the user own page ahead of it: " \
        shown(first, 60) > "/dev/stderr"
      exit 1
    }
    if (entered != 1) {
      print "PAGE ORDER GATE FAILED: the alternate screen was entered " entered " times rather than \
once. A second `?1049h` saves the cursor again on a terminal without xterm guard, and the shell \
comes back at the alternate screen origin" > "/dev/stderr"
      exit 1
    }
    if (left != 1) {
      print "PAGE ORDER GATE FAILED: the alternate screen was left " left " times rather than once, \
so the page this session took is not the page it gave back" > "/dev/stderr"
      exit 1
    }
    if (negotiated < 1) {
      print "PAGE ORDER GATE FAILED: `?7l` is not in the typescript, so `actuate::negotiation` never \
ran and this attach did not get past detection. The two counts above are then true of the capability \
batch alone and say nothing about the property" > "/dev/stderr"
      exit 1
    }

    print "the first escape sequence is `?1049h`, at byte " (first - 1) " of the typescript"
    print "entered once, left once, negotiated, over " n " bytes on a real pty"
  }
'

echo "register #30: nothing this engine sends reaches the user's own page"
