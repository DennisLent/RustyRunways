# RustyRunways

[![codecov](https://codecov.io/github/DennisLent/RustyRunways/graph/badge.svg?token=NVMX1JW002)](https://codecov.io/github/DennisLent/RustyRunways)
[![Docs](https://img.shields.io/badge/docs-latest-blue.svg)](https://dennislent.github.io/RustyRunways)

RustyRunways is a logistics and airline‑ops simulation written in Rust. Manage a cargo airline: buy planes, load and deliver orders, refuel, pay fees, and meet deadlines. The engine is deterministic and event‑driven, with a CLI, an `egui` GUI, and Python bindings designed for AI/ML/RL research and training.

Keywords: reinforcement learning environment, AI/ML simulation, vectorized environments, parallel stepping, deterministic seeding, Rust game engine, Python bindings, gym‑style loop.

---

## Features

* **Reproducible world generation**: Seeded random generation of airports with unique codes, coordinates, and fees.
* **Dynamic cargo orders**: Orders with diverse cargo types, weights, values, and deadlines.
* **Multiple airplane models**: From small props to super‑heavy freighters, each with realistic specs and purchase prices.
* **Event-driven simulation**: Schedule and process flight arrivals and order deadlines efficiently.
* **Player management**: Track cash, fleet, and delivered orders; buy new airplanes.
* **Python environments**: Single and vectorized envs with optional parallel stepping (Rayon), ideal for RL pipelines.
* **Deterministic + scalable**: Control seeds for reproducibility; vectorize to scale data collection.

---

## Project Structure

The repository is organised as a Cargo workspace with multiple crates:

* **`rusty_runways_core`** – a library crate that implements the game engine. It contains all simulation logic and can be used without any I/O.
* **`rusty_runways_cli`** – a binary crate that provides the command‑line interface.
* **`rusty_runways_gui`** – a graphical interface built on top of the core crate.
* **`rusty_runways_py`** – Python bindings exposing the game for scripting or machine learning. Both single (`GameEnv`) and vectorised (`VectorGameEnv`) environments are available.

Building or testing from the workspace root acts on both crates. Individual crates can also be built or tested separately using the `-p` flag.

---

## Getting Started

1. **Clone the repository**

   ```bash
   git clone https://github.com/yourusername/RustyRunways.git
   cd RustyRunways
   ```

2. **Build**

   Build the entire workspace (core engine and CLI):

   ```bash
   cargo build --release
   ```

   Build a single crate:

   ```bash
   cargo build -p rusty_runways_core   # just the engine
   cargo build -p rusty_runways_cli    # just the CLI
   ```

3. **Run tests**

   ```bash
   cargo test
   ```

   Test an individual crate:

   ```bash
   cargo test -p rusty_runways_core
   cargo test -p rusty_runways_cli
   ```

4. **Run the CLI**

   By default the game starts with a random seed and number of airports while giving the player $1,000,000 in cash:

   ```bash
   cargo run -p rusty_runways_cli
   ```

   You can specify the configuration explicitly:

   ```bash
   cargo run -p rusty_runways_cli -- --seed 1 --n 5 --c 1000000
   ```

   The `--seed` and `--n` options must be provided together. The starting cash `--c` option is optional and defaults to `1000000`.

5. **Run the GUI**

   In order to play the game using the GUI, please run

   ```bash
   cargo run -p rusty_runways_gui
   ```

   This will launch you into the main menu and allow you to create a game or initialize a random game.

   Please mind that the GUI is still in progress...

6. **Run the Python API**

   The project also exposes Python bindings that mirror the Rust game engine and support both single and vectorised environments. To install the module locally and try it out:

   ```bash
   cd crates/py
   maturin develop --release
   ```

   Example usage from Python:

   ```python
   from rusty_runways_py import GameEnv, VectorGameEnv

   g = GameEnv(seed=1)
   g.step(1)
   print(g.time(), g.cash())

   env = VectorGameEnv(4, seed=1)
   env.step_all(1, parallel=True)
   print(env.times())
   ```

   Deterministic behaviour is controlled by seeds. `VectorGameEnv` can step environments in parallel using Rayon under the hood.

   Gymnasium‑style loop (sketch):

   ```python
   from rusty_runways_py import GameEnv

   env = GameEnv(seed=42, num_airports=6)
   for t in range(100):
       # choose an action via CLI DSL or high‑level policy
       env.execute("ADVANCE 1")
       obs = env.state_json()      # or full_state_json()
       logs = env.drain_log()
       # compute reward from obs/logs (custom to your task)
   ```

---

## Why RustyRunways for RL/AI

- Deterministic + seedable: reproduce scenarios for stable evaluation and ablations.
- Vectorized environments: `VectorGameEnv` batches N worlds; toggle parallel stepping to scale experience.
- Simple action surface: use a compact CLI‑style DSL (load/unload/refuel/depart/advance/maintenance) or build higher‑level policies.
- Rich observations: query JSON snapshots (`state_json`, `full_state_json`) for state features and logging.
- Operational constraints: range, runway length, payload, fuel prices, deadlines—great for decision‑making under constraints.
- Multi‑frontend: CLI for scripting, GUI for inspection and control, Python for training.

---

## Commands

The game can be interacted with using a Domain-specific language (DSL). This make it easier, as queries or commands can be broken down into a more manageable tokens. 

## Inspecting the world state

`SHOW AIRPORTS`

`SHOW AIRPORTS WITH ORDERS`

`SHOW AIRPORTS <airport_id>`: show full details & orders

`SHOW AIRPORTS <airport_id> WITH ORDERS`: only orders at that airport

`SHOW PLANES`: show players entire fleet

`SHOW PLANES <plane_id>`: show one plane (status, specs, manifest)

`SHOW DISTANCES <plane_id>`: shows the distances, fuel requirements and if it can land at  given airport

## Purchases

`BUY PLANE <Model> <airport_id>`: Buys and places an airplane at the given airport

## Cargo handling

`LOAD ORDER <order_id> ON <plane_id>`: Load 1 order onto the plane (takes 1 hour)

`LOAD ORDERS [<order_id>] ON <plane_id>`: Loads n orders onto the plane (takes 1 hour)

`UNLOAD ORDER <order_id> FROM <plane_id>`: Load 1 order from the plane (takes 1 hour)

`UNLOAD ORDERS [<order_id>] FROM <plane_id>`: Load 1 order from the plane (takes 1 hour)

`UNLOAD ALL FROM <plane_id>`: Unload all orders from the plane (takes 1 hour)

`REFUEL PLANE <plane_id>`: Refuel the plane (takes 1 hour)

## Dispatch & movement

`DEPART PLANE <plane_id> <destination_airport_id>`: Sends a specific airplane on its way to the destination airport

`HOLD PLANE <plane_id>`: Plane stays parked at current location

`MAINTENANCE <plane_id>`: Performs a routine maintenance check on the airplane (takes 1 hour). 

**_KEEP IN MIND_**: If you do not perform routine checks on your fleet, they can break and cause long-term disruption and grounding.

## Time control

`ADVANCE <n>`: Advances the game by n hours (ticks) or until a new event occurs

The game can also be manually progressed by 1 hour by pressing the Enter / Return Key (i.e. no input)

## Queries

`SHOW CASH`: Shows the cash reserves of the player

`SHOW TIME`: Shows the current GameTime

`SHOW STATS`: Show the stats of the game over every day

## Game

`EXIT`: As the name suggests

## Documentation

Full documentation (Core mechanics, CLI, GUI, Python) is hosted on GitHub Pages:

- Docs: https://dennislent.github.io/RustyRunways

Includes exact airplane spec tables, event/economy/error references, and end‑to‑end examples.

---
`SAVE <game_name>`: Saves the game under that name (please be careful this will override any save with the same name)

`LOAD <game_name>`: Loads the game under that name

## Next Steps

- [x] Add scheduled maintenance and breakdown events.
- [x] Ensure charging of aircrafts and cargo works.
- [ ] Dispatch & reroute flights.
- [x] Expand automated testing.
- [x] Hook up a simple GUI (Tauri or egui).
- [x] Track operating costs.
- [x] dynamic fuel prices.
- [x] Python bindings for ML.
   - [x] extract cli version
   - [x] extract gui version
   - [ ] extract python version
- [ ] Weather conditions?
- [ ] Creating a game with input file instead of random.
- [x] Handle refueling
- [x] Arrow key history for commands
- [x] Tab-completion
- [x] Helper function

Contributions welcome! Feel free to open issues or PRs for new features or improvements.
