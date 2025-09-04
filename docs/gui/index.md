---
title: Graphical Interface (GUI)
---

# GUI

The GUI (`rusty_runways_gui`) is built with `eframe/egui` and uses the Core engine for all rules. See [Core](../core/index.md) for game mechanics.

## Layout

- Top header: game title, cash, current time, fleet size, and buttons for Save/Load/Menu/Exit.
- Right sidebar: stats (income/expenses/deliveries), fleet list (click to open plane), airports list (click to open airport), quick actions.
- Center: world map with airports and planes; hover for details, click to select, overlapping targets show a context popup.
- Bottom panel: full‑width game log with sticky scrolling.

## Panels & Windows

- Airport window
  - Overview: ID, location, runway, fees, fuel price.
  - Outstanding orders list.
  - Load Order(s): select single or multiple orders and a plane at this airport; load via buttons.

- Plane window
  - Overview: model, fuel, payload.
  - Manifest list.
  - Reachable airports (feasibility relative to this plane).
  - Actions: Refuel, Unload All, Maintenance.
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

