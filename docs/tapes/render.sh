#!/usr/bin/env bash
# Render a VHS tape to the GIF its `Output` line names.
#
#     docs/tapes/render.sh docs/tapes/commander.tape
#
# **Why this is not just `vhs <tape>`.** VHS has two halves: it drives a headless Chrome to
# screenshot the terminal frame by frame into a temporary directory, then calls ffmpeg to assemble
# those frames. On this machine — macOS 26, vhs 0.12.0, ffmpeg 9.0.1 — the first half works and the
# second does nothing at all: VHS prints `Creating <file>.gif...`, exits 0, writes no file, and never
# spawns ffmpeg (verified with a logging wrapper first on PATH, which the dependency check finds and
# the assembly never calls). Its own `vhs new` demo tape fails identically, so this is the toolchain
# and not the tape.
#
# The frames are real and correct, so this script runs VHS for the capture and performs the assembly
# ffmpeg would have done. **The frames have to be taken while VHS is still running**, because it
# removes its temporary directory on the way out — so a watcher hard-links each frame as it appears,
# and a hard link survives the original being deleted. Nothing here changes what is recorded: the
# tape is the source of truth, and the day `vhs <tape>` writes a file again this script is deleted.
set -uo pipefail

tape="${1:?usage: render.sh <tape>}"
out=$(grep -m1 '^Output ' "$tape" | awk '{print $2}')
fps=$(grep -m1 '^Set Framerate ' "$tape" | awk '{print $3}')
fps="${fps:-50}"

collect=$(mktemp -d)
trap 'rm -rf "$collect"' EXIT

# Hard-link every frame that appears, until VHS exits. `ln` and not `cp`: the frames are large and
# numerous, and a link is what makes the copy survive VHS's own cleanup.
watch() {
  while :; do
    for d in "${TMPDIR%/}"/vhs*; do
      [ -d "$d" ] || continue
      for f in "$d"/frame-*.png; do
        [ -e "$f" ] || continue
        b=$(basename "$f")
        [ -e "$collect/$b" ] || ln "$f" "$collect/$b" 2>/dev/null
      done
    done
    sleep 0.2
  done
}
watch & watcher=$!

vhs "$tape" >/dev/null 2>&1
sleep 0.5
kill "$watcher" 2>/dev/null
wait "$watcher" 2>/dev/null

frames=$(ls "$collect"/frame-text-*.png 2>/dev/null | wc -l | tr -d ' ')
[ "$frames" -gt 0 ] || { echo "render.sh: VHS captured no frames for $tape" >&2; exit 1; }
echo "render.sh: $frames frames at ${fps}fps"

mkdir -p "$(dirname "$out")"
ffmpeg -y -loglevel error \
  -r "$fps" -start_number 1 -i "$collect/frame-text-%05d.png" \
  -r "$fps" -start_number 1 -i "$collect/frame-cursor-%05d.png" \
  -filter_complex "[0][1]overlay,split[a][b];[a]palettegen=stats_mode=diff[p];[b][p]paletteuse=dither=bayer:bayer_scale=3" \
  "$out" || { echo "render.sh: ffmpeg could not assemble $out" >&2; exit 1; }
echo "render.sh: wrote $out ($(du -h "$out" | cut -f1))"
