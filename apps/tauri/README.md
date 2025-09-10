RustyRunways Tauri + React UI

This directory contains a Tauri desktop shell and a React (Vite) web UI.

Structure:
- src-tauri: Tauri Rust crate (desktop shell)
- ui: React + Vite frontend used by Tauri

Run (Desktop)
- Install deps once: `cd apps/tauri/ui && npm install`
- Ensure Tauri CLI v2 is installed (cargo plugin): `cargo install tauri-cli --locked --version ^2`
- Quick start: `scripts/dev_tauri.sh` (from repo root)
  - Starts the UI dev server on http://localhost:5173 and launches the desktop app.
- Manual (two terminals):
  - Terminal A: `cd apps/tauri/ui && npm run dev`
  - Terminal B: `cd apps/tauri/src-tauri && cargo tauri dev`

Note
- Do not open the UI in a regular browser; it relies on Tauri APIs only available in the desktop window.

Troubleshooting
- Error: `window.__TAURI_IPC__ is not a function`
  - Cause: UI is running in a regular browser, not inside the Tauri desktop webview.
  - Fix: Run the desktop app with `cd apps/tauri/src-tauri && cargo tauri dev`.
- First run is slow: Rust crates (including Tauri 2) compile on first build; subsequent runs are much faster thanks to cargo’s incremental builds. Use `sccache` to speed up rebuilds.
- Dev server port: The Tauri config expects `http://localhost:5173` (see `src-tauri/tauri.conf.json`). If you change the Vite port, update `devUrl` accordingly.

Build
- Production UI build: `cd apps/tauri/ui && npm run build`
- Desktop bundle: `cd apps/tauri/src-tauri && cargo tauri build`

Config
- `src-tauri/tauri.conf.json` uses `build.devUrl` = `http://localhost:5173` in dev and `build.frontendDist` = `../ui/dist` for production.
