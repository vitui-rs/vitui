#!/usr/bin/env python3
"""What a tape says it records, in milliseconds, and what a GIF actually holds.

The two numbers exist because they disagreed. Every GIF in `docs/img/` was short — `commander` by
seventeen frames, `cluster` by fifty-eight, `spf` by twenty-one — and the visible symptom was a
recording that stopped in the middle of a word: `cargo build --relea`, with the cursor still on it.
A GIF that ends early is not obviously broken to the eye that made it, so it is checked rather than
watched.

    duration.py tape <tape>        what the tape records, ms
    duration.py gif <gif>          what the GIF holds, ms
    duration.py check <tape> <gif> both, and a non-zero exit if the GIF is short

`Hide` and `Show` are honoured: time inside a hidden stretch is typed but not recorded, which is how
every tape here keeps its own invocation out of frame.
"""

import re
import sys

KEYS = {
    "Down", "Up", "Left", "Right", "Tab", "Enter", "Space",
    "Insert", "Backspace", "Delete", "Escape", "PageUp", "PageDown",
}


def millis(token):
    """`900ms` or `1.5s` as a float of milliseconds, or None."""
    m = re.match(r"^([\d.]+)(ms|s)$", token or "")
    if not m:
        return None
    value = float(m.group(1))
    return value if m.group(2) == "ms" else value * 1000


def tape_millis(text):
    """The recorded duration a tape describes.

    Approximate by construction — VHS owns the real timing and adds its own settling around some
    commands — so the check that uses this allows a tolerance rather than demanding equality.
    """
    speed, recording, recorded = 100.0, True, 0.0
    for raw in text.split("\n"):
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith("Set TypingSpeed"):
            speed = millis(line.split()[-1]) or speed
            continue
        if line == "Hide":
            recording = False
            continue
        if line == "Show":
            recording = True
            continue

        spent = 0.0
        if line.startswith("Sleep"):
            spent = millis(line.split()[1]) or 0.0
        elif line.startswith("Type"):
            quoted = re.search(r'"(.*)"', line)
            spent = len(quoted.group(1)) * speed if quoted else 0.0
        else:
            m = re.match(r"^([\w+]+)(?:@([\d.]+(?:ms|s)))?(?:\s+(\d+))?$", line)
            if not m:
                continue
            name = m.group(1)
            if name not in KEYS and "+" not in name:
                continue
            each = millis(m.group(2)) if m.group(2) else speed
            spent = each * (int(m.group(3)) if m.group(3) else 1)

        if recording:
            recorded += spent
    return recorded


def gif_millis(path):
    """The sum of a GIF's frame delays, read out of the file itself."""
    data = open(path, "rb").read()
    if data[:6] not in (b"GIF87a", b"GIF89a"):
        raise SystemExit(f"{path}: not a GIF")
    total, frames, at = 0, 0, 13
    if data[10] & 0x80:                                   # global colour table
        at += 3 * (2 ** ((data[10] & 7) + 1))
    while at < len(data):
        block = data[at]
        if block == 0x3B:                                 # trailer
            break
        if block == 0x21:                                 # extension
            label, size = data[at + 1], data[at + 2]
            if label == 0xF9:                             # graphic control
                total += (data[at + 4] | (data[at + 5] << 8)) * 10
                frames += 1
            at += 3 + size
            while at < len(data) and data[at]:            # sub-blocks
                at += data[at] + 1
            at += 1
        elif block == 0x2C:                               # image descriptor
            flags = data[at + 9]
            at += 10
            if flags & 0x80:
                at += 3 * (2 ** ((flags & 7) + 1))
            at += 1                                       # LZW minimum code size
            while at < len(data) and data[at]:
                at += data[at] + 1
            at += 1
        else:
            break
    return total, frames


def main(argv):
    if len(argv) >= 3 and argv[1] == "tape":
        print(int(tape_millis(open(argv[2]).read())))
        return 0
    if len(argv) >= 3 and argv[1] == "gif":
        print(gif_millis(argv[2])[0])
        return 0
    if len(argv) >= 4 and argv[1] == "check":
        want = tape_millis(open(argv[2]).read())
        have, frames = gif_millis(argv[3])
        short = want - have
        # 5% covers what this estimate cannot model; a lost tail is far larger than that.
        ok = short <= max(400.0, want * 0.05)
        print(
            f"{argv[3]}: {frames} frames, {have / 1000:.2f}s recorded, "
            f"{want / 1000:.2f}s described by {argv[2]}"
        )
        if not ok:
            print(
                f"  SHORT BY {short / 1000:.2f}s (~{short / 41.7:.0f} frames at 24fps) — the "
                f"capture lost its tail; re-render rather than shipping this",
                file=sys.stderr,
            )
            return 1
        return 0
    print(__doc__.strip(), file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
