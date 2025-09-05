RustyRunways Tauri + React UI

This directory contains a Tauri desktop shell and a React (Vite) web UI.

Structure:
- src-tauri: Tauri Rust crate (desktop shell)
- ui: React + Vite frontend used by Tauri

Dev (requires Node, npm, Rust):
- cd apps/tauri/ui && npm install && npm run dev
- In another terminal: cd apps/tauri/src-tauri && cargo tauri dev

Notes:
- The React dev server runs on port 5173 (see `ui/vite.config.ts`). If dev doesn’t hot-reload in Tauri, ensure the server is reachable from `http://localhost:5173`.

Build:
- cd apps/tauri/ui && npm run build
- cd apps/tauri/src-tauri && cargo tauri build

The Tauri config points to ../ui/dist for production and http://localhost:5173 for dev.
