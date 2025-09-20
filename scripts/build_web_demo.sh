#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")"/.. && pwd)"
cd "$ROOT_DIR"

OUT_DIR="docs/web-demo"
WASM_OUT_DIR="apps/tauri/ui/public/rr_wasm"

echo "[web-demo] Cleaning output directory"
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"
rm -rf "$WASM_OUT_DIR"
mkdir -p "$WASM_OUT_DIR"

echo "[web-demo] Building WASM core (wasm-pack)"
if ! command -v wasm-pack >/dev/null 2>&1; then
  cargo install wasm-pack --locked
fi
pushd crates/wasm >/dev/null
wasm-pack build --release --target web --out-dir ../../$WASM_OUT_DIR
popd >/dev/null

echo "[web-demo] Building React UI into docs/web-demo (GitHub Pages base)"
pushd apps/tauri/ui >/dev/null
NODE_OPTIONS="--max-old-space-size=4096" npx vite build --base /RustyRunways/web-demo/ --outDir ../../../$OUT_DIR
popd >/dev/null

echo "[web-demo] Copying wasm package"
mkdir -p "$OUT_DIR/rr_wasm"
cp -R "$WASM_OUT_DIR"/. "$OUT_DIR/rr_wasm/"

echo "[web-demo] Done. Open docs/web-demo/index.html to try the demo."
