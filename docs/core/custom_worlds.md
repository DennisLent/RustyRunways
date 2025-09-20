---
title: Custom Worlds (YAML)
---

# Custom Worlds (YAML)

RustyRunways can load fully custom worlds from a YAML file. This allows you to define airports, fees, initial orders, and gameplay pacing deterministically while still supporting randomly generated content. When airports are generated, they are placed in deterministic clusters so each map starts with local routes for small aircraft and longer hops for bigger planes.

## Schema

Top‑level keys:

- `version` (int): schema version; currently `1`.
- `seed` (int, optional): base seed for determinism (used for generated elements).
- `starting_cash` (float, optional, default `650_000.0`).
- `num_airports` (int, optional): number of airports to generate automatically when `airports` is omitted.
- `airports` (list, optional): explicit or partially specified airport definitions.
- `gameplay` (object, optional): tuning knobs for restocking cadence, fuel price behaviour, and order generation.

Airport fields (everything except `id`/`name` optional):

- `id` (int): unique across all airports.
- `name` (string): must be unique (case‑insensitive).
- `location` (object, optional): `{ x: float, y: float }` — bounds `[0, 10000]` each. When omitted a location is generated based on the seed (airports are laid out in clusters to guarantee local routes).
- `runway_length_m` (float > 0, optional): runway length in meters (generated deterministically when missing).
- `fuel_price_per_l` (float > 0, optional): $/L (generated when missing).
- `landing_fee_per_ton` (float >= 0, optional): $ per ton MTOW (generated when missing).
- `parking_fee_per_hour` (float >= 0, optional): $ per hour (generated when missing).
- `orders` (list, optional): static orders to seed the airport with. Required when order regeneration is disabled.

Manual order fields:

- `cargo` (string): any `CargoType` variant (e.g., `Food`, `Electronics`).
- `weight` (float > 0): weight in kilograms.
- `value` (float >= 0): payout in dollars.
- `deadline_hours` (int > 0): deadline window in hours.
- `destination_id` (int): airport id the cargo must reach (must exist and differ from the origin).

Gameplay tuning (defaults shown):

- `restock_cycle_hours` (int, default `168`): cadence for regeneration cycles.
- `fuel_interval_hours` (int, default `6`): cadence for dynamic fuel price adjustments.
- `fuel` (object):
  - `elasticity` (float, default `0.04`): fractional step applied when prices move up or down.
  - `min_price_multiplier` (float, default `0.6`): floor expressed as a multiple of each airport's base price.
  - `max_price_multiplier` (float, default `1.3`): ceiling expressed as a multiple of each airport's base price.
- `orders` (object):
  - `regenerate` (bool, default `true`): whether airports restock after the initial load.
  - `generate_initial` (bool, default `true`): whether random orders are generated at time 0.
  - `max_deadline_hours` (int, default `96`): maximum deadline assigned to generated orders.
  - `min_weight` (float, default `180.0`): minimum cargo weight (kg) for generated orders.
  - `max_weight` (float, default `650.0`): maximum cargo weight (kg) for generated orders.
  - `alpha` (float, default `0.12`): distance multiplier in the value calculation.
  - `beta` (float, default `0.55`): urgency multiplier in the value calculation.

## Examples

Tuned restocking with explicit airports:

```yaml
version: 1
seed: 42
starting_cash: 650000.0
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
  fuel:
    elasticity: 0.04
    min_price_multiplier: 0.6
    max_price_multiplier: 1.3
  orders:
    regenerate: true
    generate_initial: true
    max_deadline_hours: 96
    min_weight: 180.0
    max_weight: 650.0
    alpha: 0.12
    beta: 0.55
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

Minimal airport definitions (locations and fees generated from the seed):

```yaml
version: 1
seed: 12
starting_cash: 600000.0
airports:
  - id: 0
    name: GATEWAY
  - id: 1
    name: SPOKE
    fuel_price_per_l: 1.6
    runway_length_m: 2400.0
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

- Provide either explicit `airports` or `num_airports` (minimal airport entries are allowed; missing fields are generated).
- Duplicate airport IDs → error.
- Duplicate airport names (case‑insensitive) → error.
- Invalid coordinates (outside `[0, 10000]`) → error.
- Non‑positive runway length or fuel price → error.
- Fuel tuning: `elasticity` must be in `(0,1)`, `min_price_multiplier > 0`, and `max_price_multiplier >= min_price_multiplier` (typically > 1).
- `orders.regenerate: false` requires every listed airport to provide at least one manual order.

Common issues:

- Wrong extension: ensure `.yaml` or `.yml`.
- Paths: in GUI browse for a file; in CLI/Python provide a proper relative/absolute path.

## Constraints and Future Extensions

Future schema versions may allow blending generated and explicit airports, richer economic tuning, and batch configurations for automated telemetry runs.
