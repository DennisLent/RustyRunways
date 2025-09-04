---
title: Welcome to RustyRunways
---

# RustyRunways

RustyRunways is a logistics simulation game where you manage a cargo airline: buy planes, load orders, plan departures, refuel, and survive deadlines and operating costs. The engine is written in Rust and exposed via a CLI, a GUI, and Python bindings.

This documentation provides a high‑level overview of the project and links to in‑depth sections for the core engine, CLI usage, GUI features, and Python APIs.

## Project Layout

- `rusty_runways_core`: the simulation engine with all game rules and data structures.
- `rusty_runways_cli`: a friendly command‑line shell around the engine.
- `rusty_runways_gui`: an interactive `egui` application for playing the game.
- `rusty_runways_py`: Python bindings with single and vectorized environments.

## Quick Start

- Python (pip): `pip install rusty-runways` — see Python page for Gym wrappers and extra installs
- Build: `cargo build --release`
- CLI: `cargo run -p rusty_runways_cli -- --seed 1 --n 5 --c 1000000`
- GUI: `cargo run -p rusty_runways_gui`
- Python (local dev): `cd crates/py && maturin develop --release`

!!! note "Python wrappers and Gymnasium"
    For usage examples and how to enable the optional Gym wrappers, see the Python page: [Python Bindings](python/index.md). To install with Gym support: `pip install 'rusty-runways[gym]'`.

Use the sidebar to navigate to each section. The table of contents on each page follows the headings.
