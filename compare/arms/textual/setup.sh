#!/usr/bin/env bash
# Create this arm's virtualenv and install the pinned requirements.
# Idempotent: re-running it brings an existing .venv back to requirements.txt.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
venv="$here/.venv"

# The interpreter is part of the measurement, so it is named, not inherited
# from PATH.  Textual 8.2.8 runs on 3.14; VITUI_ARM_PYTHON overrides.
python_bin="${VITUI_ARM_PYTHON:-}"
if [ -z "$python_bin" ]; then
    for candidate in /opt/homebrew/bin/python3.14 python3.14 python3; do
        if command -v "$candidate" >/dev/null 2>&1; then
            python_bin="$(command -v "$candidate")"
            break
        fi
    done
fi
if [ -z "$python_bin" ]; then
    echo "setup.sh: no python3 found" >&2
    exit 1
fi

if [ ! -x "$venv/bin/python3" ]; then
    "$python_bin" -m venv "$venv"
fi

"$venv/bin/python3" -m pip install --quiet --upgrade pip
"$venv/bin/python3" -m pip install --quiet --requirement "$here/requirements.txt"

echo "python:  $("$venv/bin/python3" --version)"
echo "textual: $("$venv/bin/python3" -c 'import textual; print(textual.__version__)')"
echo "arm:     $here/arm.py"
