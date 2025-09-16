#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
VENV_DIR="$ROOT_DIR/benchmarks/.venv"
PYTHON_BIN=${PYTHON:-python3}

mkdir -p "$ROOT_DIR/benchmarks"

if [ ! -d "$VENV_DIR" ]; then
  echo "[benchmarks] Creating virtual environment in $VENV_DIR"
  "$PYTHON_BIN" -m venv "$VENV_DIR"
fi

source "$VENV_DIR/bin/activate"

pip install --upgrade pip
pip install -e "$ROOT_DIR/crates/py"
pip install tabulate tqdm matplotlib

echo "[benchmarks] Environment ready. Activate with 'source benchmarks/.venv/bin/activate'"
