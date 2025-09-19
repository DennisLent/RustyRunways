---
title: Command‑Line Interface
---

# CLI

The CLI (`rusty_runways_cli`) is a REPL that wraps the core engine. It provides a simple DSL for inspecting state and issuing actions. Game rules and constraints are implemented in the [Core](../core/index.md).

## Running

```bash
cargo run -p rusty_runways_cli -- --seed 1 --n 5 --c 1000000
```

`--seed` and `--n` must be provided together; `--c` defaults to `1000000`.

## Commands and Examples

Inspecting the world state

- `SHOW AIRPORTS`
- `SHOW AIRPORTS WITH ORDERS`
- `SHOW AIRPORTS <airport_id>` — full details & orders
- `SHOW AIRPORTS <airport_id> WITH ORDERS` — orders at that airport
- `SHOW PLANES` — player’s fleet
- `SHOW PLANES <plane_id>` — one plane (status, specs, manifest)
- `SHOW DISTANCES <plane_id>` — distances, fuel requirements, landing feasibility by airport

Purchases

- `BUY PLANE <Model> <airport_id>` — buy and place an airplane at the airport
- `SELL PLANE <plane_id>` — sell a parked, empty plane for a 60% refund

Cargo handling

- `LOAD ORDER <order_id> ON <plane_id>` — load a single order (+1h)
- `LOAD ORDERS [<order_id>] ON <plane_id>` — load multiple (+1h)
- `UNLOAD ORDER <order_id> FROM <plane_id>` — unload a single order (+1h)
- `UNLOAD ORDERS [<order_id>] FROM <plane_id>` — unload selected (+1h)
- `UNLOAD ALL FROM <plane_id>` — unload all (+1h)
- `REFUEL PLANE <plane_id>` — refuel (+1h)

Dispatch & movement

- `DEPART PLANE <plane_id> <destination_airport_id>` — depart to destination
- `HOLD PLANE <plane_id>` — remain parked
- `MAINTENANCE <plane_id>` — routine maintenance (+1h)

Time control

- `ADVANCE <n>` — advance by `n` hours (or until next event)

Queries

- `SHOW CASH`
- `SHOW TIME`
- `SHOW STATS`
- `SHOW MODELS` — list all airplane models with specs table

Game

- `SAVE <game_name>` — save game
- `LOAD <game_name>` — load game
- `LOAD CONFIG <path.yaml>` — rebuild game from a custom YAML world
- `EXIT` — exit the REPL
