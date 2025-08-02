# RustyRunways

[![codecov](https://codecov.io/github/DennisLent/RustyRunways/graph/badge.svg?token=NVMX1JW002)](https://codecov.io/github/DennisLent/RustyRunways)

RustyRunways is a small logistics simulation game written in Rust. You manage an airline company, buying and operating airplanes to transport cargo orders between randomly generated airports. The simulation is driven by an event-based system, allowing flights and deadlines to occur at precise times.

---

## Features

* **Reproducible world generation**: Seeded random generation of airports with unique codes, coordinates, and fees.
* **Dynamic cargo orders**: Orders with diverse cargo types, weights, values, and deadlines.
* **Multiple airplane models**: From small props to super‑heavy freighters, each with realistic specs and purchase prices.
* **Event-driven simulation**: Schedule and process flight arrivals and order deadlines efficiently.
* **Player management**: Track cash, fleet, and delivered orders; buy new airplanes.

---

## Project Structure

The repository is organised as a Cargo workspace with two main crates:

* **`rusty_runways_core`** – a library crate that implements the game engine. It contains all simulation logic and can be used without any I/O.
* **`rusty_runways_cli`** – a binary crate that provides the command‑line interface. It depends on the core crate and offers the interactive gameplay loop.

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

## Time control

`ADVANCE <n>`: Advances the game by n hours (ticks) or until a new event occurs

The game can also be manually progressed by 1 hour by pressing the Enter / Return Key (i.e. no input)

## Queries

`SHOW CASH`: Shows the cash reserves of the player

`SHOW TIME`: Shows the current GameTime

`SHOW STATS`: Show the stats of the game over every day

## Game

`EXIT`: As the name suggests
`SAVE <game_name>`: Saves the game under that name (please be careful this will override any save with the same name)
`LOAD <game_name>`: Loads the game under that name

## Next Steps

- [ ] Add scheduled maintenance and breakdown events.
- [x] Ensure charging of aircrafts and cargo works.
- [ ] Dispatch & reroute flights.
- [x] Expand automated testing.
- [ ] Hook up a simple GUI (Tauri or egui).
- [x] Track operating costs.
- [ ] dynamic fuel prices.
- [ ] Python bindings for ML.
   - [ ] extract cli version
   - [ ] extract gui version
   - [ ] extract python version
- [ ] Weather conditions?
- [ ] Creating a game with input file instead of random.
- [x] Handle refueling
- [x] Arrow key history for commands
- [x] Tab-completion
- [x] Helper function

Contributions welcome! Feel free to open issues or PRs for new features or improvements.
