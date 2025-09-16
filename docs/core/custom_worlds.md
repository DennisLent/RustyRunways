---
title: Custom Worlds (YAML)
---

# Custom Worlds (YAML)

RustyRunways can load fully custom worlds from a YAML file. This allows you to define airports, fees, and initial conditions deterministically, while still choosing whether to auto‑generate cargo orders.

## Schema

Top‑level keys:

- `version` (int): schema version; currently `1`.
- `seed` (int, optional): base seed for determinism (used for generated elements).
- `starting_cash` (float, optional, default `1_000_000.0`).
- `generate_orders` (bool, optional, default `true`): if true, airports are auto‑stocked with orders.
- `gameplay` (object, optional): tuning knobs for restocking cadence and order generation.
- `airports` (list, required): list of airport definitions.

Airport fields:

- `id` (int, required): unique across all airports.
- `name` (string, required): must be unique (case‑insensitive).
- `location` (object): `{ x: float, y: float }` — bounds `[0, 10000]` each.
- `runway_length_m` (float > 0): runway length in meters.
- `fuel_price_per_l` (float > 0): $/L.
- `landing_fee_per_ton` (float >= 0): $ per ton MTOW.
- `parking_fee_per_hour` (float >= 0): $ per hour.

Gameplay tuning:

- `restock_cycle_hours` (int, optional, default `336`): how often airports restock orders.
- `fuel_interval_hours` (int, optional, default `6`): cadence for dynamic fuel price adjustments.
- `orders` (object, optional): order generation parameters.

`orders` fields:

- `max_deadline_hours` (int, optional, default `336`): maximum delivery deadline assigned to generated orders.
- `min_weight` (float, optional, default `100.0`): minimum cargo weight in kilograms.
- `max_weight` (float, optional, default `20000.0`): maximum cargo weight in kilograms.
- `alpha` (float, optional, default `0.5`): distance multiplier when valuing orders.
- `beta` (float, optional, default `0.7`): urgency multiplier when valuing orders.

## Examples

Generate orders (default):

```yaml
version: 1
seed: 42
starting_cash: 1000000.0
generate_orders: true
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
    max_deadline_hours: 120
    min_weight: 250.0
    max_weight: 1000.0
    alpha: 0.4
    beta: 0.9
```

No initial orders:

```yaml
version: 1
seed: 7
starting_cash: 1500000.0
generate_orders: false
airports:
  - id: 0
    name: AAA
    location: { x: 500.0, y: 2500.0 }
    runway_length_m: 3000.0
    fuel_price_per_l: 1.0
    landing_fee_per_ton: 4.0
    parking_fee_per_hour: 18.0
  - id: 1
    name: AAB
    location: { x: 4500.0, y: 1500.0 }
    runway_length_m: 2400.0
    fuel_price_per_l: 1.9
    landing_fee_per_ton: 6.0
    parking_fee_per_hour: 25.0
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

- Duplicate airport IDs → error.
- Duplicate airport names (case‑insensitive) → error.
- Invalid coordinates (outside `[0, 10000]`) → error.
- Non‑positive runway length or fuel price → error.

Common issues:

- Wrong extension: ensure `.yaml` or `.yml`.
- Paths: in GUI browse for a file; in CLI/Python provide a proper relative/absolute path.

## Constraints and Future Extensions

This first version expects all airports to be explicit. A future mode will allow constraints to be specified (e.g., number of airports, fee ranges) and auto‑fill missing airports using the seed.
