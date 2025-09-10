# Play In Your Browser

Try a lightweight, in‑browser build of RustyRunways powered by WebAssembly. This demo runs the core simulation in WASM and the React UI in your browser — no install needed.

!!! tip "Download the Desktop App for Best Experience"
    For the best UX and performance, download the native desktop builds from the Releases page.
    
    [Download Desktop App](releases.md){ .md-button .md-button--primary }

Important limitations of the web demo:
- Saves/loads: not available in the browser version.
- Start from YAML: not available in the browser version.
- Performance: CPU/graphics are browser‑limited; long runs may be slower.
- Persistence: state resets when the page is refreshed.

These features (save/load and YAML scenarios) are available via:
- Python bindings (CLI + wrappers)
- Tauri desktop app
- Native Rust build

## Launch Demo

<iframe src="web-demo/index.html" style="width:100%; height:800px; border:1px solid var(--md-default-fg-color--lightest);"></iframe>

If the embed doesn't load, open it directly:
- Web demo: web-demo/index.html

## How it works

- The Rust core (`crates/core`) is compiled to WebAssembly via `wasm-bindgen` using a thin wrapper crate (`crates/wasm`).
- The React UI falls back to calling the WASM API when Tauri is not detected.
- We build the demo with `scripts/build_web_demo.sh`, which places a static web build under `docs/web-demo/` so MkDocs can serve it.
