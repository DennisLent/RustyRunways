#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
cd "$ROOT_DIR"

echo "[check] Using repo root: $ROOT_DIR"

MODE="check"
if [[ "${1:-}" == "--fix" ]]; then
  MODE="fix"
fi

# 1) Python formatting and linting via venv
VENV_DIR=".venv"
if [[ ! -d "$VENV_DIR" ]]; then
  echo "[check] Creating Python venv at $VENV_DIR"
  python3 -m venv "$VENV_DIR"
fi

if [[ -f "$VENV_DIR/bin/activate" ]]; then
  # Unix-like
  # shellcheck source=/dev/null
  source "$VENV_DIR/bin/activate"
else
  # Windows Git Bash fallback
  # shellcheck disable=SC1091
  source "$VENV_DIR/Scripts/activate" || true
fi

python -m pip install --upgrade pip >/dev/null
pip install --quiet ruff==0.5.6 black==24.8.0

if [[ "$MODE" == "fix" ]]; then
  echo "[check] Running Ruff --fix (crates/py)"
  (
    cd crates/py
    ruff check --fix .
  )

  echo "[check] Running Black (format) (crates/py)"
  (
    cd crates/py
    black .
  )
else
  echo "[check] Running Ruff (crates/py)"
  (
    cd crates/py
    ruff check --output-format=github .
  )

  echo "[check] Running Black --check (crates/py)"
  (
    cd crates/py
    black --check .
  )
fi

# 2) Rust formatting, linting, tests
echo "[check] Running cargo fmt --all -- --check"
cargo fmt --all -- --check

echo "[check] Running cargo clippy --workspace --all-targets -- -D warnings"
cargo clippy --workspace --all-targets -- -D warnings

echo "[check] Running cargo test --workspace"
cargo test --workspace

# 3) Python dev tests (maturin develop + pytest) for crates/py
echo "[check] Installing Python test deps and running dev tests (crates/py)"
pip install --quiet maturin pytest gymnasium
(
  cd crates/py
  maturin develop
  python - <<'PY'
import sys
print("[check] Python sys.path[0]", sys.path[0])
import rusty_runways, rusty_runways_py
print("[check] Imported rusty_runways from:", getattr(rusty_runways, "__file__", "<pkg>"))
print("[check] Imported rusty_runways_py:", rusty_runways_py.__name__)
PY
  pytest tests
)

# 4) React (Tauri UI) lint/typecheck/build
echo "[check] Checking React (apps/tauri/ui)"
(
  cd apps/tauri/ui
  if [[ ! -d node_modules ]]; then
    echo "[check] Installing UI dependencies (npm ci)"
    npm ci --silent
  fi

  if [[ "$MODE" == "fix" ]]; then
    echo "[check] Running ESLint --fix (UI)"
    npx eslint . --fix || true
  else
    echo "[check] Running ESLint (UI)"
    npm run --silent lint
  fi

  echo "[check] Type-checking (tsc --noEmit)"
  npx tsc --noEmit

  echo "[check] Building UI (vite build)"
  npm run --silent build

  # Optional: run tests if present
  if grep -q '"test"' package.json; then
    echo "[check] Running UI tests"
    npm test --silent
  else
    echo "[check] No UI tests found; skipping"
  fi
)

echo "[check] All checks passed."
