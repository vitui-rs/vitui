#!/usr/bin/env bash
# Render a VHS tape to the GIF its `Output` line names.
#
#     docs/tapes/render.sh docs/tapes/commander.tape
#
# **Why this is not just `vhs <tape>`.** VHS has two halves: it drives a headless Chrome to
# screenshot the terminal frame by frame, then calls ffmpeg to assemble those frames. On this
# machine — macOS 26, vhs 0.12.0, ffmpeg 9.0.1 — the first half works and the second writes nothing
# at all: VHS prints `Creating <file>.gif...`, exits 0 and leaves no file behind. **The cause is a
# cancelled context, and it is in VHS rather than in ffmpeg or in the tape.** `Evaluate` calls
# `teardown()` — whose whole job is `cancel()` on the context the recorder runs under — and then
# hands that same context to `Render`, which builds every conversion as an
# `exec.CommandContext(ctx, "ffmpeg", …)`. A command started on a cancelled context never runs, so
# `CombinedOutput` comes back with no output and the error `context canceled`, which VHS logs as the
# empty string. That is why a logging `ffmpeg` first on PATH is never called: no ffmpeg process is
# ever spawned. Its own `vhs new` demo tape fails identically. The frames are real and correct, so
# this script performs the assembly ffmpeg would have done, from the frames VHS itself writes out.
# The day `vhs <tape>` writes a file again this script is deleted.
#
# # The tail this script used to lose, and the one line that cost it
#
# **The first version of this script watched VHS's own temporary directory and hard-linked frames as
# they appeared.** That is a race and it lost, every time: VHS removes that directory on the way out,
# so whatever it wrote in its last moments was gone before the watcher's next poll. All three
# committed GIFs were short — `commander` by seventeen frames, `cluster` by fifty-eight, `spf` by
# twenty-one — and the way it showed was a recording that stops in the middle of a word, `cargo build
# --relea` with the cursor still sitting on it. Nothing failed; the file was simply shorter than the
# tape, which is not something the eye that made it will catch.
#
# **The second version asked VHS for the frames and got none**, which is worse than the race and was
# committed unrun: a second `Output` line naming a directory is exactly the right mechanism —
# `Evaluate`'s deferred block does `os.Rename(<temp dir>, <that directory>)` before it cleans up —
# and `os.Rename` **fails onto a directory that already exists**. The script created it first, so
# every render came back with an empty capture. The directory is now named and not created, which is
# the whole fix, and the tape's copy quotes the path: VHS's parser reads an unquoted absolute path
# as a sequence of commands and reports `Invalid command: var`.
#
# There is no watcher and no race now: the rename moves the complete set or it moves nothing. The
# checks below stay anyway, because a lost tail is invisible — the frame numbering must have no gap,
# both sequences must be the same length, and the finished GIF's own frame delays must add up to
# roughly what the tape describes. `duration.py` is that last check and is runnable on its own over a
# committed GIF.
set -uo pipefail

tape="${1:?usage: render.sh <tape>}"
here="$(cd "$(dirname "$0")" && pwd)"
out=$(grep -m1 '^Output ' "$tape" | awk '{print $2}')
fps=$(grep -m1 '^Set Framerate ' "$tape" | awk '{print $3}')
fps="${fps:-50}"

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
# Named, never created: VHS renames its own temporary directory onto this path, and `os.Rename`
# fails onto a directory that exists.
frames="$work/frames"

# The tape, plus one line telling VHS where to leave the frames. Appended to a copy: the tape a
# person runs by hand should not carry this script's scratch path.
derived="$work/$(basename "$tape")"
cp "$tape" "$derived"
printf '\nOutput "%s/"\n' "$frames" >> "$derived"

vhs "$derived" >/dev/null 2>&1

shopt -s nullglob
text=("$frames"/frame-text-*.png)
cursor=("$frames"/frame-cursor-*.png)
shopt -u nullglob

if [ "${#text[@]}" -eq 0 ]; then
  echo "render.sh: VHS wrote no frames into $frames" >&2
  echo "render.sh: the rename VHS does at the end lands nothing if $frames already exists" >&2
  exit 1
fi
if [ "${#text[@]}" -ne "${#cursor[@]}" ]; then
  echo "render.sh: ${#text[@]} text frames and ${#cursor[@]} cursor frames — the pair is uneven" >&2
  exit 1
fi

# `image2` stops at the first missing number without saying so, which would truncate the GIF exactly
# the way the watcher used to. Establish that the run is contiguous before handing it to ffmpeg.
first=$(basename "${text[0]}"  | sed 's/[^0-9]*\([0-9]*\)\.png/\1/')
# `${text[-1]}` is a bash 4 spelling and macOS ships bash 3.2 as `/bin/bash`, where it is a
# syntax error rather than a wrong answer — which is how the first render after this script was
# rewritten failed on a machine that had the frames in hand.
last=$(basename  "${text[${#text[@]}-1]}" | sed 's/[^0-9]*\([0-9]*\)\.png/\1/')
span=$((10#$last - 10#$first + 1))
if [ "$span" -ne "${#text[@]}" ]; then
  echo "render.sh: frames $first..$last is $span numbers but only ${#text[@]} files — a gap would" >&2
  echo "render.sh: truncate the GIF silently at the first one" >&2
  exit 1
fi
echo "render.sh: ${#text[@]} frames ($first..$last) at ${fps}fps"

mkdir -p "$(dirname "$out")"
ffmpeg -y -loglevel error \
  -r "$fps" -start_number "$first" -i "$frames/frame-text-%05d.png" \
  -r "$fps" -start_number "$first" -i "$frames/frame-cursor-%05d.png" \
  -filter_complex "[0][1]overlay,split[a][b];[a]palettegen=stats_mode=diff[p];[b][p]paletteuse=dither=bayer:bayer_scale=3" \
  "$out" || { echo "render.sh: ffmpeg could not assemble $out" >&2; exit 1; }

echo "render.sh: wrote $out ($(du -h "$out" | cut -f1))"

# The check that would have caught the tail. Advisory only where python3 is missing — a machine
# without it can still render, it just cannot prove the render is whole.
if command -v python3 >/dev/null; then
  python3 "$here/duration.py" check "$tape" "$out" || exit 1
else
  echo "render.sh: no python3, so the duration of $out is unverified" >&2
fi
