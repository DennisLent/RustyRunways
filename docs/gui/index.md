---
title: Graphical Interface (GUI)
---

# GUI

RustyRunways ships with two GUIs:

- Tauri + React desktop app (recommended): a polished UI that bundles the web frontend with a native shell via Tauri. It looks nicer, supports full-screen map, and is available as installers for macOS, Windows, and Linux. See Downloads: [Releases](../releases.md).
- Web Demo: run the same UI in your browser via WebAssembly. Great for a quick try — no install. [Play Online](../demo.md){ .md-button }
- egui desktop app: a lightweight Rust/egui client primarily used during early development. It remains available under `crates/gui` but is not distributed as prebuilt binaries.

Both UIs use the same Core engine and commands. See [Core](../core/index.md) for the game mechanics.

## Layout

- Top header: game title, cash, current time, fleet size, and buttons for Save/Load/Menu/Exit.
- Right sidebar: stats (income/expenses/deliveries), fleet list (click to open plane), airports list (click to open airport), quick actions.
- Center: world map with airports and planes; hover for details, click to select, overlapping targets show a context popup.
- Bottom panel: full‑width game log with sticky scrolling.

## Start From Config

- Main menu includes a “Start From Config” section with:
  - Path input and a Browse file picker (YAML/YML) via native dialog.
  - Preview window listing all parsed airports and fees.
  - Start launches a new game using the YAML config.

## Panels & Windows

- Airport window
  - Overview: ID, location, runway, fees, fuel price.
  - Outstanding orders list.
  - Load Order(s): select single or multiple orders and a plane at this airport; load via buttons.

- Plane window
  - Overview: model, fuel, payload.
  - Manifest list.
  - Reachable airports (feasibility relative to this plane).
  - Actions: Refuel, Unload All, Maintenance, Sell (parked & empty only).
  - Load Order(s):
    - Filters: destination and min/max weight.
    - Single‑select and multi‑select order lists with detailed labels.
  - Dispatch: destination dropdown and Depart button.

## Buying Planes

- Click “Buy new plane” next to Fleet Overview.
- In the dialog:
  - Select a model; specs are displayed (price, payload, cruise, fuel, burn, operating cost, runway requirement).
  - Select the starting airport; runway suitability is annotated.
  - Balance indicator shows price, cash, and affordability; purchase is disabled unless cash and runway constraints are satisfied.

## Responsiveness & UX

- Resizable panels and windows; stable sizes for Save/Load and info windows.
- Multi‑order selection and filters improve throughput when dispatching multiple orders.
- Log panel spans full width and sticks to the latest message.

## Getting the desktop app

- Download installers from the [Releases](../releases.md) page. We publish cross‑platform builds for every tagged release. On macOS we ship a universal binary that runs on both Apple Silicon and Intel.
- Developers can run the app from source:
  - Dev mode: `scripts/dev_tauri.sh` (starts Vite + Tauri window)
  - Build: `cd apps/tauri/src-tauri && cargo tauri build`

Limitations in the web demo:
- Save/load and YAML scenarios are not available in the browser. Use the Python wrappers, Tauri desktop app, or native Rust builds for those workflows.
