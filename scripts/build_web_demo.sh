#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
cd "$ROOT_DIR"

echo "[web-demo] Building WASM core (wasm-pack)"
if ! command -v wasm-pack >/dev/null 2>&1; then
  cargo install wasm-pack --locked
fi
wasm-pack build crates/wasm --release --target web --out-dir apps/tauri/ui/public/rr_wasm

echo "[web-demo] Building React UI into docs/web-demo (relative base)"
pushd apps/tauri/ui >/dev/null
NODE_OPTIONS="--max-old-space-size=4096" npx vite build --base ./ --outDir ../../../docs/web-demo
popd >/dev/null

echo "[web-demo] Done. Open docs/web-demo/index.html to try the demo."

