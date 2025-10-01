#!/usr/bin/env bash
set -euo pipefail

# Run the RustyRunways Tauri desktop app in dev mode.
# - Uses Tauri's beforeDevCommand to start Vite on http://localhost:5173
# - Launches `cargo tauri dev` for the desktop shell

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TAURI_DIR="$ROOT_DIR/apps/tauri/src-tauri"

# Port management (default 5173) and optional auto-kill
PORT=${PORT:-5173}
FORCE_KILL=0

# Parse args: [-f|--force] to auto-kill port holder, [-p|--port N] to change port
while [[ $# -gt 0 ]]; do
  case "$1" in
    -f|--force)
      FORCE_KILL=1; shift ;;
    -p|--port)
      PORT="$2"; shift 2 ;;
    *)
      echo "[tauri-dev] Unknown argument: $1" >&2
      echo "Usage: $0 [-f|--force] [-p|--port <port>]" >&2
      exit 1 ;;
  esac
done

list_port_procs() {
  if command -v lsof >/dev/null 2>&1; then
    lsof -iTCP:"$PORT" -sTCP:LISTEN -Pn || true
  else
    ss -ltnp 2>/dev/null | grep ":$PORT\\>" || true
  fi
}

port_pids() {
  if command -v lsof >/dev/null 2>&1; then
    lsof -tiTCP:"$PORT" -sTCP:LISTEN || true
  else
    ss -ltnp 2>/dev/null | awk -v p=":$PORT" '$4 ~ p { if (match($6, /pid=([0-9]+)/, m)) print m[1] }' | sort -u
  fi
}

echo "[tauri-dev] Checking port $PORT availability..."
if list_port_procs | grep -q ":$PORT\|LISTEN"; then
  echo "[tauri-dev] Port $PORT is in use by:"
  list_port_procs
  RESP="n"
  if [[ "$FORCE_KILL" -eq 1 ]]; then
    RESP="y"
  else
    read -r -p "[tauri-dev] Kill process(es) on port $PORT? [y/N] " RESP || RESP="n"
  fi
  case "${RESP,,}" in
    y|yes)
      PIDS=$(port_pids)
      if [[ -z "$PIDS" ]]; then
        echo "[tauri-dev] Could not resolve PIDs; aborting."
        exit 1
      fi
      echo "[tauri-dev] Sending SIGTERM to: $PIDS"
      kill $PIDS || true
      for i in {1..20}; do
        sleep 0.25
        if ! list_port_procs | grep -q ":$PORT\|LISTEN"; then
          break
        fi
      done
      if list_port_procs | grep -q ":$PORT\|LISTEN"; then
        echo "[tauri-dev] Forcing SIGKILL..."
        kill -9 $PIDS || true
      fi
      ;;
    *)
      echo "[tauri-dev] Aborting. Free the port or use -f to force."
      exit 1 ;;
  esac
fi
PORT=${PORT:-5173}

echo "[tauri-dev] Checking port $PORT availability..."
if command -v lsof >/dev/null 2>&1; then
  if lsof -iTCP:"$PORT" -sTCP:LISTEN -Pn >/dev/null 2>&1; then
    echo "[tauri-dev] Port $PORT is in use. Please stop the other process:"
    lsof -iTCP:"$PORT" -sTCP:LISTEN -Pn || true
    echo "[tauri-dev] Tip: kill with 'kill -9 <PID>' or change the port consistently in:"
    echo "  - apps/tauri/ui/vite.config.ts (server.port)"
    echo "  - apps/tauri/src-tauri/tauri.conf.json (build.devUrl and beforeDevCommand --port)"
    exit 1
  fi
else
  if ss -ltnp 2>/dev/null | grep -q ":$PORT\>"; then
    echo "[tauri-dev] Port $PORT is in use (detected via ss). Please free it or update configs."
    ss -ltnp | grep ":$PORT\>" || true
    exit 1
  fi
fi
echo "[tauri-dev] Ensuring Tauri icons exist..."
# Generate platform icons once (needed for Windows/macOS builds; harmless elsewhere)
ICONS_DIR="$ROOT_DIR/apps/tauri/src-tauri/icons"
if [ ! -f "$ICONS_DIR/icon.icns" ] || [ ! -f "$ICONS_DIR/icon.ico" ]; then
  mkdir -p "$ICONS_DIR"
  if command -v npx >/dev/null 2>&1; then
    echo "[tauri-dev] Generating icons via @tauri-apps/cli"
    npx --yes @tauri-apps/cli@2 icon -o "$ICONS_DIR" "$ROOT_DIR/docs/assets/rusty_runways.png" || true
  else
    echo "[tauri-dev] npx not found; skipping icon generation"
  fi
fi

echo "[tauri-dev] Launching UI dev server (vite) on port $PORT..."
# Ensure UI deps on first run
if [ ! -d "$ROOT_DIR/apps/tauri/ui/node_modules" ]; then
  echo "[tauri-dev] Installing UI dependencies (first run)..."
  npm --prefix "$ROOT_DIR/apps/tauri/ui" install --silent
fi

# Start Vite dev server in background
"$(command -v npm)" --prefix "$ROOT_DIR/apps/tauri/ui" run dev -- --host localhost --port "$PORT" --strictPort &
UI_PID=$!
echo "[tauri-dev] UI dev started (pid=$UI_PID). Waiting for port $PORT..."

# Wait until port is listening (max ~15s)
for i in {1..60}; do
  if command -v lsof >/dev/null 2>&1; then
    if lsof -iTCP:"$PORT" -sTCP:LISTEN -Pn >/dev/null 2>&1; then break; fi
  else
    if ss -ltnp 2>/dev/null | grep -q ":$PORT\\>"; then break; fi
  fi
  sleep 0.25
done

echo "[tauri-dev] Launching Tauri desktop shell..."
cd "$TAURI_DIR"
if cargo tauri -V >/dev/null 2>&1; then
  cargo tauri dev || true
else
  echo "[tauri-dev] cargo-tauri not found. Trying npx @tauri-apps/cli@2 dev ..."
  npx @tauri-apps/cli@2 dev || true
fi

echo "[tauri-dev] Shutting down UI dev (pid=$UI_PID)"
kill "$UI_PID" >/dev/null 2>&1 || true
