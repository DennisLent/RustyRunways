---
title: Welcome to RustyRunways
---

# RustyRunways

RustyRunways is a logistics simulation game where you manage a cargo airline: buy planes, load orders, plan departures, refuel, and survive deadlines and operating costs. The engine is written in Rust and exposed via a CLI, a desktop GUI, and Python bindings.

This documentation provides a high‑level overview of the project and links to in‑depth sections for the core engine, CLI usage, GUI features, and Python APIs.

## Project Layout

- `rusty_runways_core`: the simulation engine with all game rules and data structures.
- `rusty_runways_cli`: a friendly command‑line shell around the engine.
- `apps/tauri`: Tauri + React desktop app — the recommended, polished UI distributed as installers.
- `rusty_runways_gui`: a lightweight `egui` application (development client).
- `rusty_runways_py`: Python bindings with single and vectorized environments.

## Quick Start

[▶ Play the Web Demo](demo.md){ .md-button .md-button--primary }
[⬇ Download Desktop App](releases.md){ .md-button }

- Python (pip): `pip install rusty-runways` — see Python page for Gym wrappers and extra installs
- Build: `cargo build --release`
- CLI: `cargo run -p rusty_runways_cli -- --seed 1 --n 5 --c 650000`
- GUI (download): see [Releases](releases.md) for installers (macOS universal, Windows, Linux).
- GUI (web demo): try it in your browser — no install — see [Play Online](demo.md). Note: save/load and YAML scenarios are only available in Python, Tauri desktop, or native Rust.
- GUI (from source):
  - Dev: `scripts/dev_tauri.sh`
  - Build: `cd apps/tauri/src-tauri && cargo tauri build`
- Python (local dev): `cd crates/py && maturin develop --release`

!!! note "Python wrappers and Gymnasium"
    For usage examples and how to enable the optional Gym wrappers, see the Python page: [Python Bindings](python/index.md). To install with Gym support: `pip install 'rusty-runways[gym]'`.

Use the sidebar to navigate to each section. The table of contents on each page follows the headings.
