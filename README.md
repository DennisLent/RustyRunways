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

## Getting Started

1. **Clone the repository**

   ```bash
   git clone https://github.com/yourusername/RustyRunways.git
   cd RustyRunways
   ```

2. **Build**

   ```bash
   cargo build --release
   ```

3. **Run tests**

   ```bash
   cargo test
   ```


## Project Structure

```text
src/
├── main.rs            # (optional) binary entrypoint
├── game.rs            # Game loop, state, and event processing
├── event.rs           # Event definitions and scheduling
├── player.rs          # Player struct and fleet management
├── utils/
│   ├── coordinate.rs  # 2D Coordinate
│   ├── map.rs         # Map and airport generation
│   ├── airport.rs     # Airport struct and order generation
│   ├── orders/        # CargoType and Order definitions
│   ├── airplanes/     # Airplane related defintions 
└── README.md          # This file
```

---

## Key Modules

### `utils/map.rs`

* **`Map::generate_from_seed(seed, n)`**: Creates `n` airports with coordinates.
* **`Map::restock_airports()`**: Populates each airport with orders.
* **`Map::min_distance()`**: Finds the shortest hop between any two airports.

### `utils/airport.rs`

* **`Airport::generate_random(seed, id)`**: Determines runway, fees, and code.
* **`Airport::generate_orders(seed, num_airports)`**: Generates orders based on airport size.

### `utils/orders/`\*\*

* **`CargoType`**: Enum of cargo (electronics, live alpacas, quantum widgets, etc.).
* **`Order::new(seed, origin, num_airports)`**: Creates deterministic orders with weight/value/deadline.

### `utils/coordinate.rs`

* **`Coordinate`**: 2D `x`, `y` with helper methods.

### `utils/airplanes/models.rs`

* **`AirplaneModel`** and **`AirplaneSpecs`**: Defines models (SparrowLight, FalconJet, TitanHeavy, etc.) with specs and purchase price.

### `utils/airplanes/airplane.rs`

* **`Airplane`**: Tracks location, fuel, payload, manifest, with methods to load, fly, refuel, and unload.

### `player.rs`

* **`Player`**: Tracks cash, fleet (Vec<Airplane>), and delivered orders. Methods to initialize with a plane that can handle the minimum hop, and to buy new planes.

### `event.rs`

* **`Event`**: FlightArrival and OrderDeadline.
* **`ScheduledEvent`**: Wraps `Event` with `time` and implements ordering by time.

### `game.rs`

* **`Game`**: Holds `time`, `map`, `airplanes`, `player`, `events`. Methods to schedule events, process them (`tick_event`), and run the simulation (`run_until`).

---

## Simulation Flow

1. **Initialization**: `Game::new(seed, num_airports, starting_cash)` builds the map, restocks orders, gives the player an initial airplane.
2. **Scheduling**: When a plane departs, schedule a `FlightArrival` at `time + flight_duration`. When creating an order, schedule its `OrderDeadline`.
3. **Event Processing**: `run_until(max_time)` repeatedly calls `tick_event()` to advance to the next event, updating plane locations, unloading cargo, crediting player cash, and removing expired orders.

---

## Next Steps

- [ ] Add scheduled maintenance and breakdown events.
- [ x ] Ensure charging of aircrafts and cargo works.
- [ ] Dispatch & reroute flights.
- [ ] Expand automated testing.
- [ ] Hook up a simple GUI or terminal map view.
- [ ] Track operating costs, depreciation, dynamic fuel prices.
- [ ] Python bindings for ML.
- [ ] Enable hangars to store airplanes and/or cargo.
- [ ] Weather conditions?
- [ ] Creating a game with input file instead of random.

Contributions welcome! Feel free to open issues or PRs for new features or improvements.
