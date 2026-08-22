#!/usr/bin/env bash
#
# Spec §11's **lint rung**, and the control that makes it mean something.
#
# The rung is a `clippy.toml` fragment, and the finding it is built on is that **a library cannot ship
# this lint to its users**: clippy reads its configuration from the crate being linted, and there is no
# stable mechanism for a dependency to inject lints downstream. Verified with a control rather than
# assumed — the file in the dependency produces no diagnostic, and the same code with the file in the
# application produces the warning.
#
# So what ships is a ready fragment, in `examples/app-template/clippy.toml` and quoted in that
# directory's README, **wired into vitui's own CI — which protects vitui and not its users.** This
# script is that wiring, and it runs in both directions:
#
#   1. the template lints **clean**, so the fragment is compatible with an application that does
#      everything right; and
#   2. a copy of the template with one `std::thread::sleep` in it lints **red**, so a clean result in
#      (1) is not the fragment being inert, the attribute having been deleted, or `clippy.toml` having
#      been moved out from under the crate.
#
# Direction (2) is the one worth having. Measured on this repository's own history: a
# `RUSTFLAGS`-flipping "must fail to compile" step passed with the negative cases deleted.
#
#   scripts/lint-rung-gate.sh
#
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
template="$root/examples/app-template"

# **Inside the repository's own `target/`, which is the directory CI caches.** The template is a
# separate workspace, so left alone it would build `vitui-engine` and `crossterm` from scratch into
# `examples/app-template/target` on every run, in a directory no cache key covers. Sharing a target
# directory across workspaces is safe — cargo namespaces every unit by a hash of how it was built —
# and it is what makes this gate cost seconds rather than a cold compile of the engine twice.
export CARGO_TARGET_DIR="$root/target/lint-rung"

# What the control adds, and the diagnostic it must produce. Anything on the fragment's list would
# do; `sleep` is the one spec §11 quotes.
readonly OFFENCE='fn _the_control() { std::thread::sleep(std::time::Duration::from_millis(1)); }'
readonly DIAGNOSTIC='disallowed method `std::thread::sleep`'

[ -f "$template/clippy.toml" ] || {
  echo "LINT RUNG GATE FAILED: no clippy.toml at $template — the fragment is what is being gated" >&2
  exit 1
}

echo "1/2  the template must lint clean"
(cd "$template" && cargo clippy --all-targets)

echo "2/2  the control must lint red"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
# A copy rather than an edit in place, so a failure here cannot leave the template dirty and so this
# is safe to run on a machine somebody is working on. Three files by name rather than `cp -R`, which
# would drag in the template's own `target/` if anybody had built it.
mkdir -p "$work/probe/src"
cp "$template/clippy.toml" "$work/probe/clippy.toml"
cp "$template/src/main.rs" "$work/probe/src/main.rs"
# Cargo will not resolve `../../crates/vitui-engine` from a temporary directory, so the path is
# rewritten to an absolute one. Everything else about the crate — `clippy.toml` included — is
# byte-identical to the template, which is what makes this a control and not a second experiment.
#
# `sed` writing to a new file rather than `sed -i`, whose in-place flag takes an argument on BSD and
# none on GNU, and this repository's runners are one of each. Not `python3` either: the `rust:*`
# images do not carry it, and a gate that cannot run in CI is not a gate.
sed "s|path = \"../../crates/vitui-engine\"|path = \"$root/crates/vitui-engine\"|" \
  "$template/Cargo.toml" > "$work/probe/Cargo.toml"
grep -q "$root/crates/vitui-engine" "$work/probe/Cargo.toml" || {
  echo "LINT RUNG GATE FAILED: the dependency path was not rewritten, so the control would fail to \
resolve rather than fail to lint" >&2
  exit 1
}
printf '\n%s\n' "$OFFENCE" >> "$work/probe/src/main.rs"

if out="$(cd "$work/probe" && cargo clippy --all-targets 2>&1)"; then
  echo "$out"
  echo "LINT RUNG GATE FAILED: the control linted clean. The fragment is not reaching the crate — \
either \`#![warn(clippy::disallowed_methods)]\` is gone from src/main.rs, or clippy.toml is not \
being read, or the list no longer names std::thread::sleep" >&2
  exit 1
fi

if ! printf '%s' "$out" | grep -qF -- "$DIAGNOSTIC"; then
  echo "$out"
  echo "LINT RUNG GATE FAILED: the control failed for some other reason than the lint. It must fail \
with \"$DIAGNOSTIC\" and nothing else, or this gate is measuring a compile error" >&2
  exit 1
fi

echo "     the control failed with: $DIAGNOSTIC"
echo "spec §11's lint rung: the fragment is clean on the template and red on the control"
echo "it protects vitui and not vitui's users — see examples/app-template/README.md"
