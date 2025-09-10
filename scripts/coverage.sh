#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
cd "$ROOT_DIR"

# Per-language local thresholds (do not enforce globally here)
RUST_FAIL_UNDER=${RUST_FAIL_UNDER:-80}
# Default: do not fail on Python coverage unless explicitly opted in
PY_FAIL_UNDER=${PY_FAIL_UNDER:-0}
# Global project threshold for local runs (0 disables)
TOTAL_FAIL_UNDER=${TOTAL_FAIL_UNDER:-0}

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
pip install --quiet maturin pytest pytest-cov numpy gymnasium coverage-lcov

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
    --out Lcov \
    --output-dir ../../coverage/rust \
    --timeout 300 \
    --fail-under "$RUST_FAIL_UNDER"
  # Collect Cobertura report regardless of where tarpaulin wrote it
  mkdir -p ../../coverage/rust
  if [ -f cobertura.xml ]; then mv cobertura.xml ../../coverage/rust/cobertura.xml; fi
  if [ -f ../../cobertura.xml ]; then mv ../../cobertura.xml ../../coverage/rust/cobertura.xml; fi
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
  # Export Python coverage to LCOV for unified totals
  python -m coverage lcov -o ../../coverage/python/lcov.info || true
)

# Optional: Frontend (React) coverage if Node + tests are present
echo "[coverage] Frontend (React)"
if [ -d apps/tauri/ui ] && command -v node >/dev/null 2>&1; then
  (
    cd apps/tauri/ui
    if node -e "const p=require('./package.json');process.exit(p.scripts&&p.scripts.test?0:1)"; then
      echo "[coverage] Installing npm deps and running tests with coverage"
      if ! npm ci; then
        echo "[coverage] npm ci failed (lockfile out of sync). Falling back to npm install..."
        npm install --no-audit --no-fund || { echo "[coverage] Frontend deps install failed; skipping frontend coverage"; exit 0; }
      fi
      # Prefer vitest if present
      if node -e "const p=require('./package.json');process.exit((p.devDependencies&&p.devDependencies.vitest)||(p.dependencies&&p.dependencies.vitest)?0:1)"; then
        npx vitest run --coverage --reporter=dot || npx vitest run --coverage
      else
        npm test -- --coverage --watchAll=false || true
      fi
      mkdir -p ../../../coverage/frontend
      if [ -f coverage/lcov.info ]; then
        cp coverage/lcov.info ../../../coverage/frontend/lcov.info
      fi
    else
      echo "[coverage] No frontend test script; skipping"
    fi
  )
else
  echo "[coverage] Frontend directory or Node not found; skipping"
fi

# Compute combined total coverage mirroring Codecov project status
echo "[coverage] Calculating unified project coverage (LCOV/Cobertura)"
python - "$TOTAL_FAIL_UNDER" << 'PY'
import os, sys, xml.etree.ElementTree as ET

def parse_lcov(path):
    total = 0
    covered = 0
    with open(path, 'r', encoding='utf-8', errors='ignore') as f:
        for line in f:
            if line.startswith('DA:'):
                try:
                    _, cnt = line.strip()[3:].split(',')
                    cnt = int(cnt)
                except Exception:
                    continue
                total += 1
                if cnt > 0:
                    covered += 1
    return covered, total

def parse_cobertura(path):
    try:
        root = ET.parse(path).getroot()
        lv = root.attrib.get('lines-valid')
        lc = root.attrib.get('lines-covered')
        if lv is not None and lc is not None:
            return int(lc), int(lv)
        # Fallback: sum lines
        covered = 0
        total = 0
        for line in root.findall('.//line'):
            total += 1
            if int(line.attrib.get('hits', '0')) > 0:
                covered += 1
        return covered, total
    except Exception:
        return 0, 0

INCLUDE_FRONTEND = os.environ.get('INCLUDE_FRONTEND_IN_TOTAL', '1').lower() in ('1','true','yes')

sources = [
    ('Rust', 'coverage/rust/lcov.info', 'coverage/rust/cobertura.xml'),
    ('Python', 'coverage/python/lcov.info', None),
]
if INCLUDE_FRONTEND:
    sources.append(('Frontend', 'coverage/frontend/lcov.info', None))

parts = []
for name, lcov_path, cobre_path in sources:
    c = t = 0
    if lcov_path and os.path.exists(lcov_path):
        c, t = parse_lcov(lcov_path)
    elif cobre_path and os.path.exists(cobre_path):
        c, t = parse_cobertura(cobre_path)
    if t > 0:
        parts.append((name, c, t))

total_cov = sum(c for _, c, _ in parts)
total_lines = sum(t for _, _, t in parts)

def fmt(pct):
    return f"{pct:.2f}%"

print('[coverage] Parts:')
for name, c, t in parts:
    pct = (c / t * 100.0) if t else 0.0
    print(f"  - {name}: {c}/{t} = {fmt(pct)}")

overall = (total_cov / total_lines * 100.0) if total_lines else 0.0
print(f"[coverage] TOTAL: {total_cov}/{total_lines} = {fmt(overall)}")

try:
    fail_under = float(sys.argv[1]) if len(sys.argv) > 1 else 0.0
except Exception:
    fail_under = 0.0
if fail_under > 0 and overall < fail_under:
    print(f"[coverage] ERROR: total coverage {fmt(overall)} is below threshold {fail_under}%")
    sys.exit(2)
PY

echo "[coverage] Done. Artifacts in ./coverage"
