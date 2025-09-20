---
title: Welcome to RustyRunways
---

# RustyRunways

RustyRunways is a logistics simulation game where you manage a growing air carrier moving both cargo and passengers: buy planes, schedule departures, balance mixed payloads, refuel, and survive deadlines plus operating costs. Time advances in hours, airports restock every 168 in‑game hours, and fuel prices fluctuate every six hours inside a bounded range. Demand heuristics ensure starter-friendly work (roughly two-thirds of visible payloads are flyable by the starter plane, route lengths start short and grow with upgrades) while larger airports seed lucrative cargo and passenger itineraries. The engine is written in Rust and exposed via a CLI, a desktop GUI, and Python bindings.

This documentation provides a high‑level overview of the project and links to in‑depth sections for the core engine, CLI usage, GUI features, and Python APIs.

## Project Layout

- `rusty_runways_core`: the simulation engine with all game rules and data structures.
- `rusty_runways_cli`: a friendly command‑line shell around the engine.
- `apps/tauri`: Tauri + React desktop app — the recommended, polished UI distributed as installers.
- `rusty_runways_gui`: a lightweight `egui` application (development client).
- `rusty_runways_py`: Python bindings with single and vectorized environments.

## Core Rules at a Glance

- **Fleet management:** You begin with one starter aircraft, $650 000 cash, and a randomly generated 12‑airport network (unless a YAML world overrides these numbers). Planes must be parked, empty, and solvent to sell or refuel. Buying additional planes requires sufficient funds and a runway long enough for the chosen model.
- **Cargo flow:** Each airport surfaces orders between 180 kg and 650 kg with deadlines capped at 96 hours. When `regenerate` is enabled (default) the network restocks every 168 hours so the player is never starved of work. Delivering before the deadline pays out immediately; missing a deadline forfeits the order value.
- **Operating costs:** Fuel updates every six hours inside the `[0.6×, 1.3×]` bounds, and standard fees (fuel buy, landing, parking, maintenance) apply whenever an action uses them. Loading, unloading, refuelling, and maintenance consume one hour each; departure consumes the full travel time calculated from aircraft speed and leg distance.
- **Progression:** The first upgrade should be reachable within the first in‑game week of focused play. Later aircraft unlock longer routes (the procedural map contains clustered hubs plus longer spokes) and higher late‑game margins without manual boosts.
- **Save/Load:** Saves capture the full simulation state, including running events and cash flow, so you can resume across CLI, GUI, or Python sessions.

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

## Working with the Python Agent

The Python bindings expose the same rules as the Rust engine. You can:

1. Instantiate `GameEnv` or `VectorGameEnv` with the default parameters or a YAML config (`config_path`), step the simulation, and issue the same commands the CLI understands via `execute()`.
2. Wrap the environment with the Gymnasium helpers (`RustyRunwaysGymEnv`, `RustyRunwaysGymVectorEnv`) to obtain standard `step`/`reset` loops and `MultiDiscrete` action spaces.
3. Run the sample heuristic agent in `benchmarks/run_benchmarks.py` to gather feasibility, margin, and upgrade statistics across many seeds. The generated plots highlight early/mid/late phase performance so you can compare code changes or YAML tweaks quickly.

Refer to [Python Bindings](python/index.md) for complete API details and to the [Benchmarks README](../benchmarks/README.md) for the automation workflow.

Use the sidebar to navigate to each section. The table of contents on each page follows the headings.
