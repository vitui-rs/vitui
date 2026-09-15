#!/usr/bin/env bash
# **Every picture in this repository, regenerated — or checked.**
#
#   scripts/shots.sh            write docs/img/{components,apps}/*.svg
#   scripts/shots.sh --check    re-take them and fail if any is not what is on disk
#
# Forty-nine files: twenty-nine components, one per built row of the freeze, and twenty applications, one
# per `APPS` row that draws a frame. Each is the SVG of a frame this engine composited headlessly —
# not a photograph of a terminal, the frame itself.
#
# # Why the two halves are gated differently
#
# The components' half is a `cargo test`: the pictures are drawn by a library function, so a test can
# take one and compare it, which is exactly what a golden screen does, blessing included. The
# applications' half cannot be, because `cargo test` does not run an example and an application *is*
# an example — twenty separate binaries, each of which writes its own. So the drift check is here:
# re-take all twenty into a temporary directory and diff.
#
# A failing `--check` means a picture on disk is of a screen this workspace no longer draws. Run the
# script with no arguments, look at the diff — it is text, which is the whole reason the format is
# SVG — and commit it with whatever changed the screen.
set -euo pipefail

here=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
cd "$here"

check=0
[[ "${1:-}" == "--check" ]] && check=1

# The applications that draw a frame. `caps` is not one: it attaches, reads what the terminal
# claimed, detaches and prints, so there is no composited frame to be a picture of.
apps=$(cargo run -q -p vitui-apps --bin apps_json |
    tr ',' '\n' | sed -n 's/.*"name": *"\([a-z]*\)".*/\1/p' | grep -v '^caps$')

if [[ $check -eq 0 ]]; then
    echo "== components (29, the freeze's built rows)"
    VITUI_BLESS=1 cargo test -q -p vitui-components --test shots -- --test-threads=1

    echo "== applications"
    for app in $apps; do
        cargo run -q -p vitui-apps --example "$app" -- --shot
    done
    echo "== done; the diff is text, review it"
    exit 0
fi

echo "== components"
# No blessing: the test compares against the files on disk and says which one to look at.
cargo test -q -p vitui-components --test shots -- --test-threads=1

echo "== applications"
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT
cp -R docs/img/apps "$scratch/before"
for app in $apps; do
    cargo run -q -p vitui-apps --example "$app" -- --shot >/dev/null
done

# **One picture is excluded from the drift check and it is not excluded from the pictures.**
# `latency` draws a measured duration — `fold 119us`, `fold 122us` — so its screen is different on
# every run by design. §14's rule is that a timing is a report and never a gate, and a picture with a
# stopwatch in it is a timing wearing an equality's clothes. The file is still written and still
# reviewed like any other; what is not asserted is that two runs agree.
# `crates/vitui-apps/tests/pictures.rs` joins this exclusion to its own reason, so the two cannot
# drift apart.
if ! diff -rq --exclude=latency.svg "$scratch/before" docs/img/apps; then
    # Put the committed files back, so a failed check leaves the tree as it found it.
    cp -R "$scratch/before/." docs/img/apps
    echo
    echo "an application's picture is not the screen it draws any more."
    echo "run scripts/shots.sh, review the diff, and commit it with what changed the screen."
    exit 1
fi
echo "== every picture is the screen it is a picture of"
