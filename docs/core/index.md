---
title: Core Engine
---

# Core Engine

The `rusty_runways_core` crate implements all game rules and the event‑driven simulation. It is I/O‑free and can be embedded in different frontends (CLI, GUI, Python).

## Core Concepts

- Game state (`Game`): time, player, airplanes, airports, scheduled events, and logs.
- Airports: name, coordinates, fees, runway length, dynamic fuel price, and outstanding orders.
- Airplanes: model, specs, location, status, manifest, and operational costs.
- Orders: either cargo (type, weight) or passenger groups (count), plus value, deadline, origin, destination.
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
- Dynamic pricing nudges prices up or down by the configured elasticity each `fuel_interval_hours`, clamped between the configured minimum and maximum multipliers of the airport's base price.
- Strategy: refuel where cheaper, accounting for range, payload, and deadlines.

## Airplanes

All models (see `utils::airplanes::models`):

- SparrowLight – short-range combi prop
- FalconJet – light biz jet (passenger)
- CometRegional – regional turbofan (passenger)
- Atlas – narrow-body combi
- TitanHeavy – wide-body freighter (cargo)
- Goliath – super-heavy freighter (cargo)
- Zephyr – long-range twin-aisle (passenger)
- Lightning – supersonic small jet (passenger)
- BisonFreighter – medium cargo hauler
- TrailblazerCombi – high-capacity combi aircraft

Each model exposes specs via `AirplaneModel::specs()` including:

- MTOW, cruise speed (km/h), fuel capacity (L), fuel consumption (L/h), operating cost ($/h), cargo payload capacity (kg), passenger capacity (seats), model role (cargo/passenger/mixed), purchase price, and computed minimum runway length (m).
- Players may sell a parked, empty airplane back to the market for 60% of its purchase price.

### Custom Airplane Catalog (YAML)

Scenarios can replace or extend the built‑in airplane catalog via the world YAML. Add an `airplanes` block at the top level:

```
airplanes:
  strategy: replace   # or: add (default)
  models:
    - name: WorkshopCombi
      mtow: 15000.0
      cruise_speed: 520.0
      fuel_capacity: 3200.0
      fuel_consumption: 260.0
      operating_cost: 950.0
      payload_capacity: 3200.0
      passenger_capacity: 24
      purchase_price: 780000.0
      min_runway_length: 1200.0
      role: Mixed        # Cargo | Passenger | Mixed
```

- strategy=replace uses only the declared models. strategy=add merges them with defaults.
- All fields are required. Validation enforces positive values and role‑specific capacities:
  - Cargo requires payload_capacity > 0
  - Passenger requires passenger_capacity > 0
  - Mixed requires both > 0
 
Games started from this YAML will list these models in the CLI, Python, and Tauri UI and allow buying them by name.

## Landing Constraints and Derivation

Minimum runway length is derived from simplified physics based on the airplane’s cruise speed, assumed takeoff speed (~0.65·cruise), acceleration, and deceleration:

- Takeoff distance: `v² / (2a)` with `a ≈ 2.5 m/s²`.
- Landing distance: `v² / (2d)` with `d ≈ 4.0 m/s²`.
- The required runway length is the max of the two. Airports must meet this requirement to allow takeoff/landing for a model.
