#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
cd "$ROOT_DIR"

RUST_FAIL_UNDER=${RUST_FAIL_UNDER:-80}
# Default: do not fail on Python coverage unless explicitly opted in
PY_FAIL_UNDER=${PY_FAIL_UNDER:-0}

echo "[coverage] Repo: $ROOT_DIR"

# Ensure tools
if ! command -v cargo-tarpaulin >/dev/null 2>&1; then
  echo "[coverage] Installing cargo-tarpaulin"
  cargo install cargo-tarpaulin --locked
fi

python3 -m venv .venv >/dev/null 2>&1 || true
source .venv/bin/activate || true
python -m pip install --upgrade pip >/dev/null
# Python deps for tests (gym wrappers rely on numpy)
pip install --quiet maturin pytest pytest-cov numpy gymnasium

mkdir -p coverage/rust coverage/python

echo "[coverage] Rust (tarpaulin)"
# Run from the core crate to avoid workspace-level excludes/config
(
  cd crates/core
  cargo tarpaulin \
    --engine ptrace \
    --exclude-files 'crates/commands/*' \
    --exclude-files 'crates/py/*' \
    --exclude-files 'crates/cli/src/main.rs' \
    --out Xml \
    --timeout 300 \
    --fail-under "$RUST_FAIL_UNDER"
  mv cobertura.xml ../../coverage/rust/cobertura.xml 2>/dev/null || true
)

echo "[coverage] Python (pytest-cov)"
(
  cd crates/py
  maturin develop
  pytest -q \
    --cov=rusty_runways \
    --cov-branch \
    --cov-report=xml:../../coverage/python/coverage.xml \
    --cov-fail-under="$PY_FAIL_UNDER"
)

echo "[coverage] Done. Artifacts in ./coverage"
