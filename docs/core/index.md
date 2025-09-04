---
title: Core Engine
---

# Core Engine

The `rusty_runways_core` crate implements all game rules and the event‑driven simulation. It is I/O‑free and can be embedded in different frontends (CLI, GUI, Python).

## Core Concepts

- Game state (`Game`): time, player, airplanes, airports, scheduled events, and logs.
- Airports: name, coordinates, fees, runway length, dynamic fuel price, and outstanding orders.
- Airplanes: model, specs, location, status, manifest, and operational costs.
- Orders: cargo type, weight, value, deadline, origin, destination.
- Player: cash, fleet, deliveries; can buy planes subject to constraints.

## Game Rules

- Time advances in integer hours. Many actions (load, unload, refuel, maintenance) consume one hour.
- Deadlines are tracked in hours. Delivering after a deadline is considered a failure (penalties are handled in the engine’s accounting/events layer).
- Airports charge fuel by liter and fees (parking/landing) based on usage and mass.
- Airplane movement is constrained by range and runway length at both origin and destination.

## Loading, Unloading, Refueling

- Load order: `Game::load_order(order_id, plane_id)`
  - Requires the plane to be parked at an airport that holds the order.
  - Checks payload capacity; schedules a loading event (+1h).
- Unload orders
  - `unload_order(order_id, plane_id)` for a single order.
  - `unload_all(plane_id)` to empty the manifest.
  - Schedules unloading events (+1h).
- Refuel plane: `refuel_plane(plane_id)` schedules refueling (+1h) and charges the price per liter at the airport.

## Passing of Time

- `advance(hours)` progresses the simulation by the requested amount or until the next event.
- Events include arrivals, load/unload completions, maintenance, deadlines, and breakdowns.
- The engine accrues income and expenses (operating costs, purchases) as time passes and actions occur.

## World Generation (Seedable)

- A new game is created via `Game::new(seed, num_airports, starting_cash)`.
- The world uses deterministic PRNG seeding:
  - Airports (positions, names, fees, runway lengths) are generated based on the seed and `num_airports`.
  - Orders originate at airports with randomized types, weights, deadlines, and destinations.
  - The same seed produces the same world layout and initial orders.

## Maintenance

- Airplanes can be set to maintenance (`maintenance_on_airplane(plane_id)`), which takes time and can prevent breakdowns.
- Skipping routine checks increases the risk of failures (modeled by the engine), grounding planes and delaying operations.

## Fuel Prices

- Each airport has a fuel price per liter. Refueling charges depend on the quantity and local pricing.
- Strategy: refuel where cheaper, accounting for range, payload, and deadlines.

## Airplanes

All models (see `utils::airplanes::models`):

- SparrowLight – small prop
- FalconJet – light biz jet
- CometRegional – regional turbofan
- Atlas – narrow‑body jet
- TitanHeavy – wide‑body freighter
- Goliath – super‑heavy lift
- Zephyr – long‑range twin‑aisle
- Lightning – supersonic small jet

Each model exposes specs via `AirplaneModel::specs()` including:

- MTOW, cruise speed (km/h), fuel capacity (L), fuel consumption (L/h), operating cost ($/h), payload capacity (kg), purchase price, and computed minimum runway length (m).

## Landing Constraints and Derivation

Minimum runway length is derived from simplified physics based on the airplane’s cruise speed, assumed takeoff speed (~0.65·cruise), acceleration, and deceleration:

- Takeoff distance: `v² / (2a)` with `a ≈ 2.5 m/s²`.
- Landing distance: `v² / (2d)` with `d ≈ 4.0 m/s²`.
- The required runway length is the max of the two. Airports must meet this requirement to allow takeoff/landing for a model.

