---
title: Custom Worlds (YAML)
---

# Custom Worlds (YAML)

RustyRunways can load fully custom worlds from a YAML file. This allows you to define airports, fees, initial orders, and gameplay pacing deterministically while still supporting randomly generated content.

## Schema

Top‑level keys:

- `version` (int): schema version; currently `1`.
- `seed` (int, optional): base seed for determinism (used for generated elements).
- `starting_cash` (float, optional, default `1_000_000.0`).
- `airports` (list, optional): explicit airport definitions. If omitted, you **must** set `num_airports`.
- `num_airports` (int, optional): number of airports to generate randomly when `airports` is empty.
- `gameplay` (object, optional): tuning knobs for restocking cadence, order behaviour, and order value scaling.

Airport fields:

- `id` (int, required when `airports` is provided): unique across all airports.
- `name` (string, required): must be unique (case‑insensitive).
- `location` (object): `{ x: float, y: float }` — bounds `[0, 10000]` each.
- `runway_length_m` (float > 0): runway length in meters.
- `fuel_price_per_l` (float > 0): $/L.
- `landing_fee_per_ton` (float >= 0): $ per ton MTOW.
- `parking_fee_per_hour` (float >= 0): $ per hour.
- `orders` (list, optional): static orders to seed the airport with. Required when order regeneration is disabled.

Manual order fields:

- `cargo` (string): any `CargoType` variant (e.g., `Food`, `Electronics`).
- `weight` (float > 0): weight in kilograms.
- `value` (float >= 0): payout in dollars.
- `deadline_hours` (int > 0): deadline window in hours.
- `destination_id` (int): airport id the cargo must reach (must exist and differ from the origin).

Gameplay tuning (defaults shown):

- `restock_cycle_hours` (int, default `336`): cadence for regeneration cycles.
- `fuel_interval_hours` (int, default `6`): cadence for dynamic fuel price adjustments.
- `orders` (object):
  - `regenerate` (bool, default `true`): whether airports restock after the initial load.
  - `generate_initial` (bool, default `true`): whether random orders are generated at time 0.
  - `max_deadline_hours` (int, default `336`): maximum deadline assigned to generated orders.
  - `min_weight` (float, default `100.0`): minimum cargo weight (kg) for generated orders.
  - `max_weight` (float, default `20000.0`): maximum cargo weight (kg) for generated orders.
  - `alpha` (float, default `0.5`): distance multiplier in the value calculation.
  - `beta` (float, default `0.7`): urgency multiplier in the value calculation.

## Examples

Tuned restocking with explicit airports:

```yaml
version: 1
seed: 42
starting_cash: 1000000.0
airports:
  - id: 0
    name: HUB
    location: { x: 1000.0, y: 1000.0 }
    runway_length_m: 3500.0
    fuel_price_per_l: 1.2
    landing_fee_per_ton: 5.0
    parking_fee_per_hour: 20.0
  - id: 1
    name: AAB
    location: { x: 3000.0, y: 2000.0 }
    runway_length_m: 2800.0
    fuel_price_per_l: 1.7
    landing_fee_per_ton: 4.5
    parking_fee_per_hour: 15.0

gameplay:
  restock_cycle_hours: 168
  fuel_interval_hours: 4
  orders:
    regenerate: true
    generate_initial: true
    max_deadline_hours: 120
    min_weight: 250.0
    max_weight: 1000.0
    alpha: 0.4
    beta: 0.9
```

Random airports with delayed restock:

```yaml
version: 1
seed: 99
starting_cash: 750000.0
num_airports: 5
gameplay:
  orders:
    regenerate: true
    generate_initial: false
```

Static manual orders (no regeneration):

```yaml
version: 1
seed: 3
starting_cash: 800000.0
airports:
  - id: 0
    name: HUB
    location: { x: 1200.0, y: 900.0 }
    runway_length_m: 3200.0
    fuel_price_per_l: 1.5
    landing_fee_per_ton: 4.5
    parking_fee_per_hour: 18.0
    orders:
      - cargo: Food
        weight: 550.0
        value: 2700.0
        deadline_hours: 48
        destination_id: 1
  - id: 1
    name: AAX
    location: { x: 3400.0, y: 2100.0 }
    runway_length_m: 2400.0
    fuel_price_per_l: 1.8
    landing_fee_per_ton: 4.0
    parking_fee_per_hour: 16.0
    orders:
      - cargo: Electronics
        weight: 300.0
        value: 4200.0
        deadline_hours: 36
        destination_id: 0

gameplay:
  orders:
    regenerate: false
    generate_initial: false
```

## Using Configs

- CLI
  - Start: `cargo run -p rusty_runways_cli -- --config examples/sample_world.yaml`
  - REPL: `LOAD CONFIG examples/sample_world_no_orders.yaml`

- GUI
  - Main menu → Start From Config → Browse → Preview → Start.

- Python
  - `GameEnv(config_path="examples/sample_world.yaml")`
  - `VectorGameEnv(4, config_path="examples/sample_world.yaml")`

## Validation & Errors

- Provide either explicit `airports` or `num_airports` (but not both).
- Duplicate airport IDs → error.
- Duplicate airport names (case‑insensitive) → error.
- Invalid coordinates (outside `[0, 10000]`) → error.
- Non‑positive runway length or fuel price → error.
- `orders.regenerate: false` requires every listed airport to provide at least one manual order.

Common issues:

- Wrong extension: ensure `.yaml` or `.yml`.
- Paths: in GUI browse for a file; in CLI/Python provide a proper relative/absolute path.

## Constraints and Future Extensions

Future schema versions may allow blending generated and explicit airports, richer economic tuning, and batch configurations for automated telemetry runs.
