#!/usr/bin/env bash
# Render a VHS tape to the GIF its `Output` line names.
#
#     docs/tapes/render.sh docs/tapes/commander.tape
#
# **Why this is not just `vhs <tape>`.** VHS has two halves: it drives a headless Chrome to
# screenshot the terminal frame by frame, then calls ffmpeg to assemble those frames. On this
# machine — macOS 26, vhs 0.12.0, ffmpeg 9.0.1 — the first half works and the second does nothing at
# all: VHS prints `Creating <file>.gif...`, exits 0, writes no file, and never spawns ffmpeg
# (verified with a logging wrapper first on PATH, which the dependency check finds and the assembly
# never calls). Its own `vhs new` demo tape fails identically, so this is the toolchain and not the
# tape. The frames are real and correct, so this script performs the assembly ffmpeg would have
# done. The tape stays the source of truth, and the day `vhs <tape>` writes a file again this script
# is deleted.
#
# # The tail this script used to lose
#
# **The first version of this script watched VHS's own temporary directory and hard-linked frames as
# they appeared.** That is a race and it lost, every time: VHS removes that directory on the way out,
# so whatever it wrote in its last moments was gone before the watcher's next poll. All three
# committed GIFs were short — `commander` by seventeen frames, `cluster` by fifty-eight, `spf` by
# twenty-one — and the way it showed was a recording that stops in the middle of a word, `cargo build
# --relea` with the cursor still sitting on it. Nothing failed; the file was simply shorter than the
# tape, which is not something the eye that made it will catch.
#
# There is no watcher now. **VHS writes the frames itself**, into a directory named by a second
# `Output` line, and a directory VHS was told to write is a directory VHS does not delete. This
# script adds that line to a copy of the tape rather than to the tape, so the tape on disk stays the
# thing a reader can run by hand.
#
# And because a lost tail is invisible, the assembly is checked rather than trusted: the frame
# numbering must have no gap, both sequences must be the same length, and the finished GIF's own
# frame delays must add up to roughly what the tape describes. `duration.py` is that last check and
# is runnable on its own over a committed GIF.
set -uo pipefail

tape="${1:?usage: render.sh <tape>}"
here="$(cd "$(dirname "$0")" && pwd)"
out=$(grep -m1 '^Output ' "$tape" | awk '{print $2}')
fps=$(grep -m1 '^Set Framerate ' "$tape" | awk '{print $3}')
fps="${fps:-50}"

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
frames="$work/frames"
mkdir -p "$frames"

# The tape, plus one line telling VHS where to leave the frames. Appended to a copy: the tape a
# person runs by hand should not carry this script's scratch path.
derived="$work/$(basename "$tape")"
cp "$tape" "$derived"
printf '\nOutput %s\n' "$frames/" >> "$derived"

vhs "$derived" >/dev/null 2>&1

shopt -s nullglob
text=("$frames"/frame-text-*.png)
cursor=("$frames"/frame-cursor-*.png)
shopt -u nullglob

if [ "${#text[@]}" -eq 0 ]; then
  echo "render.sh: VHS wrote no frames into $frames" >&2
  echo "render.sh: check that this VHS understands a directory as an Output target" >&2
  exit 1
fi
if [ "${#text[@]}" -ne "${#cursor[@]}" ]; then
  echo "render.sh: ${#text[@]} text frames and ${#cursor[@]} cursor frames — the pair is uneven" >&2
  exit 1
fi

# `image2` stops at the first missing number without saying so, which would truncate the GIF exactly
# the way the watcher used to. Establish that the run is contiguous before handing it to ffmpeg.
first=$(basename "${text[0]}"  | sed 's/[^0-9]*\([0-9]*\)\.png/\1/')
last=$(basename  "${text[-1]}" | sed 's/[^0-9]*\([0-9]*\)\.png/\1/')
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
