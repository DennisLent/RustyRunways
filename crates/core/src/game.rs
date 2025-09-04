use crate::config::WorldConfig;
use crate::events::{Event, GameTime, ScheduledEvent};
use crate::player::Player;
use crate::statistics::DailyStats;
use crate::utils::airplanes::airplane::Airplane;
use crate::utils::airplanes::models::AirplaneStatus;
use crate::utils::airport::Airport;
use crate::utils::coordinate::Coordinate;
use crate::utils::errors::GameError;
use crate::utils::map::Map;
use crate::utils::orders::order::MAX_DEADLINE;
use rand::{Rng, SeedableRng, rngs::StdRng};
use rusty_runways_commands::Command::*;
use rusty_runways_commands::{Command, parse_command};
use serde::{Deserialize, Serialize};
use std::collections::BinaryHeap;
use std::path::{Path, PathBuf};
use std::{fs, io};

const RESTOCK_CYCLE: u64 = MAX_DEADLINE * 24;
const REPORT_INTERVAL: u64 = 24;
const FUEL_INTERVAL: u64 = 6;

fn default_rng() -> StdRng {
    StdRng::seed_from_u64(0)
}

/// Holds all mutable world state and drives the simulation via scheduled events.
#[derive(Debug, Serialize, Deserialize)]
pub struct Game {
    /// Current simulation time (hours)
    pub time: GameTime,
    /// The world map of airports and coordinates
    pub map: Map,
    /// All airplanes in the world
    pub airplanes: Vec<Airplane>,
    /// Tracker for each plane's last arrival time
    pub arrival_times: Vec<GameTime>,
    /// The player's company (cash, fleet, deliveries)
    pub player: Player,
    /// Future events, ordered by their `time` (earliest first)
    pub events: BinaryHeap<ScheduledEvent>,
    /// Income over each day
    pub daily_income: f32,
    /// Expenses over each day
    pub daily_expenses: f32,
    /// History of all stats
    pub stats: Vec<DailyStats>,
    /// Seed used to create the RNG for deterministic behaviour
    pub seed: u64,
    /// Game-local random number generator to avoid global RNG usage
    #[serde(skip, default = "default_rng")]
    rng: StdRng,
    /// Log of messages generated during play
    #[serde(skip, default)]
    log: Vec<String>,
}

#[derive(Serialize)]
pub struct Observation {
    pub time: u64,
    pub cash: f32,
    pub airports: Vec<AirportObs>,
    pub planes: Vec<PlaneObs>,
}

#[derive(Serialize)]
pub struct AirportObs {
    pub id: usize,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub fuel_price: f32,
    pub runway_length: f32,
    pub num_orders: usize,
}

#[derive(Serialize)]
pub struct PlaneObs {
    pub id: usize,
    pub model: String,
    pub x: f32,
    pub y: f32,
    pub status: String,
    pub fuel: FuelObs,
    pub payload: PayloadObs,
    pub destination: Option<usize>,
    pub hours_remaining: Option<u64>,
}

#[derive(Serialize)]
pub struct FuelObs {
    pub current: f32,
    pub capacity: f32,
}

#[derive(Serialize)]
pub struct PayloadObs {
    pub current: f32,
    pub capacity: f32,
}

impl Game {
    /// Initialize a new game with `num_airports`, seeded randomness, and player's starting cash.
    pub fn new(seed: u64, num_airports: Option<usize>, starting_cash: f32) -> Self {
        let map = Map::generate_from_seed(seed, num_airports);

        let player = Player::new(starting_cash, &map);
        let airplanes = player.fleet.clone();
        let arrival_times = vec![0; airplanes.len()];
        let events = BinaryHeap::new();

        let mut game = Game {
            time: 0,
            map,
            airplanes,
            player,
            events,
            arrival_times,
            daily_income: 0.0,
            daily_expenses: 0.0,
            stats: Vec::new(),
            seed,
            rng: StdRng::seed_from_u64(seed),
            log: Vec::new(),
        };

        game.schedule(RESTOCK_CYCLE, Event::Restock);
        game.schedule(REPORT_INTERVAL, Event::DailyStats);
        game.schedule(FUEL_INTERVAL, Event::DynamicPricing);
        game.schedule_world_event();
        game.schedule(1, Event::MaintenanceCheck);

        game
    }

    /// Initialize a game from a configuration (airports explicitly provided).
    pub fn from_config(cfg: WorldConfig) -> Result<Self, GameError> {
        if cfg.airports.is_empty() {
            return Err(GameError::InvalidConfig {
                msg: "no airports provided".into(),
            });
        }

        // validate unique ids and duplicate names
        {
            use std::collections::HashSet;
            let mut ids = HashSet::new();
            let mut names = HashSet::new();
            for a in &cfg.airports {
                if !ids.insert(a.id) {
                    return Err(GameError::InvalidConfig {
                        msg: format!("duplicate airport id {}", a.id),
                    });
                }
                let lower = a.name.to_lowercase();
                if !names.insert(lower) {
                    return Err(GameError::InvalidConfig {
                        msg: format!("duplicate airport name '{}'", a.name),
                    });
                }
            }
        }

        let mut airports_vec = Vec::with_capacity(cfg.airports.len());
        for a in &cfg.airports {
            if a.runway_length_m <= 0.0 {
                return Err(GameError::InvalidConfig {
                    msg: format!("airport {} runway_length must be > 0", a.id),
                });
            }
            if a.fuel_price_per_l <= 0.0 {
                return Err(GameError::InvalidConfig {
                    msg: format!("airport {} fuel_price_per_l must be > 0", a.id),
                });
            }
            if !(0.0..=10000.0).contains(&a.location.x) || !(0.0..=10000.0).contains(&a.location.y)
            {
                return Err(GameError::InvalidConfig {
                    msg: format!(
                        "airport {} location ({:.2},{:.2}) out of bounds [0,10000]",
                        a.id, a.location.x, a.location.y
                    ),
                });
            }
            let ap = Airport {
                id: a.id,
                name: a.name.clone(),
                runway_length: a.runway_length_m,
                fuel_price: a.fuel_price_per_l,
                landing_fee: a.landing_fee_per_ton,
                parking_fee: a.parking_fee_per_hour,
                orders: Vec::new(),
                fuel_sold: 0.0,
            };
            let coord = Coordinate::new(a.location.x, a.location.y);
            airports_vec.push((ap, coord));
        }

        let seed = cfg.seed.unwrap_or(0);
        let mut map = Map::from_airports(seed, airports_vec);
        if cfg.generate_orders {
            map.restock_airports();
        }

        let player = Player::new(cfg.starting_cash, &map);
        let airplanes = player.fleet.clone();
        let arrival_times = vec![0; airplanes.len()];
        let events = BinaryHeap::new();

        let mut game = Game {
            time: 0,
            map,
            airplanes,
            player,
            events,
            arrival_times,
            daily_income: 0.0,
            daily_expenses: 0.0,
            stats: Vec::new(),
            seed,
            rng: StdRng::seed_from_u64(seed),
            log: Vec::new(),
        };

        game.schedule(RESTOCK_CYCLE, Event::Restock);
        game.schedule(REPORT_INTERVAL, Event::DailyStats);
        game.schedule(FUEL_INTERVAL, Event::DynamicPricing);
        game.schedule_world_event();
        game.schedule(1, Event::MaintenanceCheck);

        // (no duplicate-name warnings; duplicate names are treated as errors in validation)

        Ok(game)
    }

    /// Return the seed used to initialize this game
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Drain the internal log, returning all messages collected so far
    pub fn drain_log(&mut self) -> Vec<String> {
        std::mem::take(&mut self.log)
    }

    /// Reinitialize runtime-only fields after deserializing
    pub fn reset_runtime(&mut self) {
        self.rng = StdRng::seed_from_u64(self.seed);
        self.log.clear();
    }

    fn days_and_hours(&self, total_hours: GameTime) -> String {
        let days = total_hours / 24;
        let hours = total_hours % 24;

        match (days, hours) {
            (0, h) => format!("{}h", h),
            (d, 0) => format!("{}d", d),
            (d, h) => format!("{}d {}h", d, h),
        }
    }

    fn schedule_world_event(&mut self) {
        // event every 4 to 5 days
        let next_start = self.time + self.rng.gen_range(96..=120);

        // 1/8 chance it is global
        let is_global = self.rng.gen_bool(0.125);
        let airport = if is_global {
            None
        } else {
            Some(self.rng.gen_range(0..self.map.num_airports))
        };

        // price can spike or crash
        let factor = if self.rng.gen_bool(0.5) {
            self.rng.gen_range(1.2..=1.5)
        } else {
            self.rng.gen_range(0.5..=0.8)
        };

        // lasts 12 - 72 hours
        let duration = self.rng.gen_range(24..72);
        self.schedule(
            self.time + next_start,
            Event::WorldEvent {
                airport,
                factor,
                duration,
            },
        );
    }

    /// Write the entire game state to JSON to save
    pub fn save_game(&self, name: &str) -> io::Result<()> {
        let save_dir = Path::new("save_games");
        fs::create_dir_all(save_dir)?;

        let mut path = PathBuf::from(save_dir);
        path.push(format!("{}.json", name));

        let file = fs::File::create(&path)?;
        let writer = io::BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self).map_err(io::Error::other)
    }

    /// Load a game from JSON
    pub fn load_game(name: &str) -> io::Result<Self> {
        let mut path = PathBuf::from("save_games");
        path.push(format!("{}.json", name));

        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Save file '{}' not found", path.display()),
            ));
        }

        let file = fs::File::open(&path)?;
        let reader = io::BufReader::new(file);
        let game: Game = serde_json::from_reader(reader).map_err(io::Error::other)?;
        Ok(game)
    }

    /// Schedule `event` to occur at absolute simulation time `time`.
    fn schedule(&mut self, time: GameTime, event: Event) {
        self.events.push(ScheduledEvent { time, event });
    }

    /// Show current player cash
    pub fn show_cash(&self) {
        println!("${}", self.player.cash);
    }

    /// Show current time
    pub fn show_time(&self) {
        println!("{}", self.days_and_hours(self.time));
    }

    /// Shows the lifetime stats
    pub fn show_stats(&self) {
        let headers = ["Day", "Income", "Expense", "End Cash", "Fleet", "Delivered"];

        //get max width per column
        let mut col_widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
        let mut rows: Vec<Vec<String>> = Vec::with_capacity(self.stats.len());

        for s in &self.stats {
            let row = vec![
                s.day.to_string(),
                format!("{:.2}", s.income),
                format!("{:.2}", s.expenses),
                format!("{:.2}", s.net_cash),
                s.fleet_size.to_string(),
                s.total_deliveries.to_string(),
            ];

            for (i, cell) in row.iter().enumerate() {
                col_widths[i] = col_widths[i].max(cell.len());
            }
            rows.push(row);
        }

        for (i, header) in headers.iter().enumerate() {
            if i > 0 {
                print!(" | ");
            }
            // left-align
            print!("{:<width$}", header, width = col_widths[i]);
        }
        println!();

        // Separator
        let total_width: usize = col_widths.iter().sum::<usize>() + (3 * (headers.len() - 1));
        println!("{}", "-".repeat(total_width));

        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i > 0 {
                    print!(" | ");
                }

                // right-align
                print!("{:>width$}", cell, width = col_widths[i]);
            }
            println!();
        }
    }

    /// Process the next scheduled event; advance `self.time`. Returns false if no events remain.
    pub fn tick_event(&mut self) -> bool {
        if let Some(scheduled) = self.events.pop() {
            // advance time
            self.time = scheduled.time;

            match scheduled.event {
                // Restock every 14 days
                Event::Restock => {
                    self.map.restock_airports();
                    self.schedule(self.time + RESTOCK_CYCLE, Event::Restock);
                }

                // Finished loading, therefore we need to update the status
                Event::LoadingEvent { plane } => {
                    self.airplanes[plane].status = AirplaneStatus::Parked;
                }

                // Update the progress of the flight
                Event::FlightProgress { plane } => {
                    // buffer for events
                    let mut to_schedule: Vec<(GameTime, Event)> = Vec::new();

                    {
                        let airplane = &mut self.airplanes[plane];

                        if let AirplaneStatus::InTransit {
                            hours_remaining,
                            destination,
                            origin,
                            total_hours,
                        } = airplane.status
                        {
                            let dest_coord = self.map.airports[destination].1;
                            let hours_elapsed = total_hours - hours_remaining + 1;
                            let fraction = (hours_elapsed as f32) / (total_hours as f32);

                            airplane.location = Coordinate {
                                x: origin.x + (dest_coord.x - origin.x) * fraction,
                                y: origin.y + (dest_coord.y - origin.y) * fraction,
                            };

                            if hours_remaining > 1 {
                                airplane.status = AirplaneStatus::InTransit {
                                    hours_remaining: hours_remaining - 1,
                                    destination,
                                    origin,
                                    total_hours,
                                };

                                // still in transit
                                to_schedule.push((self.time + 1, Event::FlightProgress { plane }));
                            } else {
                                // landing
                                let (airport, _) = &self.map.airports[destination];
                                let landing_fee = airport.landing_fee(airplane);
                                self.player.cash -= landing_fee;
                                self.daily_expenses += landing_fee;

                                self.arrival_times[plane] = self.time;
                                airplane.location = self.map.airports[destination].1;

                                if airplane.needs_maintenance {
                                    airplane.status = AirplaneStatus::Broken;
                                    to_schedule.push((self.time + 8, Event::Maintenance { plane }));
                                } else {
                                    airplane.status = AirplaneStatus::Parked;
                                }
                            }
                        }
                    }

                    // Schedule new events
                    for (when, ev) in to_schedule {
                        self.schedule(when, ev);
                    }
                }

                Event::RefuelComplete { plane } => {
                    self.airplanes[plane].status = AirplaneStatus::Parked;
                }

                Event::DailyStats => {
                    let day = self.time / 24;
                    self.stats.push(DailyStats {
                        day,
                        income: self.daily_income,
                        expenses: self.daily_expenses,
                        net_cash: self.player.cash,
                        fleet_size: self.player.fleet_size,
                        total_deliveries: self.player.orders_delivered,
                    });

                    //reset
                    self.daily_expenses = 0.0;
                    self.daily_expenses = 0.0;

                    self.schedule(self.time + REPORT_INTERVAL, Event::DailyStats);
                }

                Event::DynamicPricing => {
                    // Adjust prices across the board
                    for (airport, _) in self.map.airports.iter_mut() {
                        airport.adjust_fuel_price();
                    }

                    // Schedule next
                    self.schedule(self.time + FUEL_INTERVAL, Event::DynamicPricing);
                }

                Event::WorldEvent {
                    airport,
                    factor,
                    duration,
                } => {
                    match airport {
                        Some(airport_id) => {
                            self.map.airports[airport_id].0.fuel_price *= factor;
                            let name = &self.map.airports[airport_id].0.name;

                            let pct = (factor - 1.0) * 100.0;
                            println!(
                                "Fuel price spike of +{:.0}% at {} for {}h!",
                                pct, name, duration
                            );
                        }
                        None => {
                            for (airport, _) in &mut self.map.airports {
                                airport.fuel_price *= factor
                            }

                            let pct = (factor - 1.0) * 100.0;
                            println!("Global fuel price spike of +{:.0}% for {}h!", pct, duration);
                        }
                    }

                    let event_end = self.time + duration;
                    self.schedule(event_end, Event::WorldEventEnd { airport, factor });
                }

                // Reset world event
                Event::WorldEventEnd { airport, factor } => {
                    match airport {
                        Some(airport_id) => {
                            self.map.airports[airport_id].0.fuel_price /= factor;

                            let name = &self.map.airports[airport_id].0.name;
                            let pct = (factor - 1.0) * 100.0;
                            println!("Fuel price spike of +{:.0}% at {} has ended.", pct, name);
                        }
                        None => {
                            for (airport, _) in &mut self.map.airports {
                                airport.fuel_price /= factor
                            }
                            let pct = (factor - 1.0) * 100.0;
                            println!("Global fuel price spike of +{:.0}% has ended.", pct);
                        }
                    }

                    // schedule the next event
                    self.schedule_world_event();
                }

                Event::MaintenanceCheck => {
                    // Collect vec of planes that are broken:
                    let mut just_broke = Vec::new();

                    for (idx, airplane) in self.airplanes.iter_mut().enumerate() {
                        if airplane.status != AirplaneStatus::Maintenance {
                            airplane.add_hours_since_maintenance();
                            let p_fail = airplane.risk_of_failure();
                            if self.rng.gen_bool(p_fail as f64) {
                                airplane.needs_maintenance = true;
                                if matches!(
                                    airplane.status,
                                    AirplaneStatus::Parked
                                        | AirplaneStatus::Loading
                                        | AirplaneStatus::Unloading
                                        | AirplaneStatus::Refueling
                                ) {
                                    airplane.status = AirplaneStatus::Broken;
                                    just_broke.push(idx);
                                }
                            }
                        }
                    }

                    // Schedule broken events
                    for idx in just_broke {
                        self.schedule(self.time + 8, Event::Maintenance { plane: idx });
                    }

                    // next check
                    self.schedule(self.time + 1, Event::MaintenanceCheck);
                }

                Event::Maintenance { plane } => {
                    let airplane = &mut self.airplanes[plane];
                    airplane.status = AirplaneStatus::Parked;
                    airplane.hours_since_maintenance = 0;
                    airplane.needs_maintenance = false;
                }

                _ => {
                    println!("Not implemented!")
                }
            }

            true
        } else {
            false
        }
    }

    /// Run the simulation until `max_time` or until there are no more events.
    pub fn run_until(&mut self, max_time: GameTime) {
        while self.time < max_time && self.tick_event() {}

        //if no events, just jump to time step
        if self.time < max_time {
            self.time = max_time;
        }
    }

    pub fn advance(&mut self, hours: GameTime) {
        let target = self.time + hours;

        // Keep processing events in time order until we're past `target`
        while let Some(ev) = self.events.peek() {
            if ev.time <= target {
                self.tick_event();
            } else {
                break;
            }
        }

        // Finally bump the clock
        self.time = target;
    }

    /// Display a summary of all airports in the map, including their orders.
    /// If with_orders is true, show the orders alongside.
    pub fn list_airports(&self, with_orders: bool) {
        println!("Airports ({} total):", self.map.num_airports);
        for (airport, coord) in &self.map.airports {
            println!(
                "ID: {} | {} at ({:.2}, {:.2}) | Runway: {:.0}m | Fuel: ${:.2}/L | Parking: ${:.2}/hr | Landing Fee: ${:.2}/ton",
                airport.id,
                airport.name,
                coord.x,
                coord.y,
                airport.runway_length,
                airport.fuel_price,
                airport.parking_fee,
                airport.landing_fee,
            );
            if with_orders {
                if airport.orders.is_empty() {
                    println!("  No pending orders.");
                } else {
                    println!("  Orders:");
                    for order in &airport.orders {
                        println!(
                            "    [{}] {:?} -> {} | weight: {:.1}kg | value: ${:.2} | deadline: {} | destination: {}",
                            order.id,
                            order.name,
                            self.map.airports[order.destination_id].0.name,
                            order.weight,
                            order.value,
                            order.deadline,
                            order.destination_id
                        );
                    }
                }
            }
        }
    }

    /// Display a summary of a single airport in the map, including its orders.
    /// If with_orders is true, show the orders alongside.
    pub fn list_airport(&self, airport_id: usize, with_orders: bool) -> Result<(), GameError> {
        if airport_id > (self.map.num_airports - 1) {
            return Err(GameError::AirportIdInvalid { id: airport_id });
        }

        let (airport, coord) = &self.map.airports[airport_id];
        println!(
            "ID: {} | {} at ({:.2}, {:.2}) | Runway: {:.0}m | Fuel: ${:.2}/L | Parking: ${:.2}/hr | Landing Fee: ${:.2}/ton",
            airport.id,
            airport.name,
            coord.x,
            coord.y,
            airport.runway_length,
            airport.fuel_price,
            airport.parking_fee,
            airport.landing_fee,
        );
        if with_orders {
            if airport.orders.is_empty() {
                println!("  No pending orders.");
            } else {
                println!("  Orders:");
                for order in &airport.orders {
                    println!(
                        "    [{}] {:?} -> {} | weight: {:.1}kg | value: ${:.2} | deadline: {} | destination: {}",
                        order.id,
                        order.name,
                        self.map.airports[order.destination_id].0.name,
                        order.weight,
                        order.value,
                        self.days_and_hours(order.deadline),
                        order.destination_id
                    );
                }
            }
        }

        Ok(())
    }

    fn find_associated_airport(&self, location: &Coordinate) -> Result<String, GameError> {
        let airport = match self.map.airports.iter().find(|(_, c)| c == location) {
            Some((airport, _)) => airport,
            _ => {
                return Err(GameError::AirportLocationInvalid {
                    location: *location,
                });
            }
        };

        Ok(airport.name.clone())
    }

    /// Locate a plane and the index of the airport where it is currently parked.
    ///
    /// Returns [`GameError::PlaneIdInvalid`] if no plane with `plane_id` exists or
    /// [`GameError::PlaneNotAtAirport`] if the plane is not located at any airport.
    fn plane_and_airport_idx(&self, plane_id: usize) -> Result<(usize, usize), GameError> {
        let plane_index = self
            .airplanes
            .iter()
            .position(|p| p.id == plane_id)
            .ok_or(GameError::PlaneIdInvalid { id: plane_id })?;

        let location = self.airplanes[plane_index].location;
        let airport_idx = self
            .map
            .airports
            .iter()
            .position(|(_, coord)| *coord == location)
            .ok_or(GameError::PlaneNotAtAirport { plane_id })?;

        Ok((plane_index, airport_idx))
    }

    /// Display a summary of all airplanes in the game.
    pub fn list_airplanes(&self) -> Result<(), GameError> {
        println!("Airplanes ({} total):", self.airplanes.len());
        for plane in &self.airplanes {
            if let AirplaneStatus::InTransit {
                hours_remaining,
                destination,
                ..
            } = plane.status
            {
                let dest_name = &self.map.airports[destination].0.name;
                println!(
                    "ID: {} | {:?} en-route to airport {} | Location: ({:.2}, {:.2}) | Fuel: {:.2}/{:.2}L | Payload: {:.2}/{:.2}kg | Status: InTransit - arrival in {}",
                    plane.id,
                    plane.model,
                    dest_name,
                    plane.location.x,
                    plane.location.y,
                    plane.current_fuel,
                    plane.specs.fuel_capacity,
                    plane.current_payload,
                    plane.specs.payload_capacity,
                    self.days_and_hours(hours_remaining)
                );
            } else {
                let loc = &plane.location;
                let airport_name = self.find_associated_airport(loc)?;
                println!(
                    "ID: {} | {:?} at airport {} ({:.2}, {:.2}) | Fuel: {:.2}/{:.2}L | Payload: {:.2}/{:.2}kg | Status: {:?}",
                    plane.id,
                    plane.model,
                    airport_name,
                    loc.x,
                    loc.y,
                    plane.current_fuel,
                    plane.specs.fuel_capacity,
                    plane.current_payload,
                    plane.specs.payload_capacity,
                    plane.status,
                );
            }
        }

        Ok(())
    }

    /// Display a summary of a single airplane in the game.
    pub fn list_airplane(&self, plane_id: usize) -> Result<(), GameError> {
        if plane_id > (self.airplanes.len() - 1) {
            return Err(GameError::PlaneIdInvalid { id: plane_id });
        }

        let plane = &self.airplanes[plane_id];

        if let AirplaneStatus::InTransit {
            hours_remaining,
            destination,
            ..
        } = plane.status
        {
            let dest_name = &self.map.airports[destination].0.name;
            println!(
                "ID: {} | {:?} en-route to airport {} | Location: ({:.2}, {:.2}) | Fuel: {:.2}/{:.2}L | Payload: {:.2}/{:.2}kg | Status: InTransit - arrival in {}",
                plane.id,
                plane.model,
                dest_name,
                plane.location.x,
                plane.location.y,
                plane.current_fuel,
                plane.specs.fuel_capacity,
                plane.current_payload,
                plane.specs.payload_capacity,
                self.days_and_hours(hours_remaining)
            );

            Ok(())
        } else {
            let loc = &plane.location;
            let airport_name = self.find_associated_airport(loc)?;
            println!(
                "ID: {} | {:?} at airport {} ({:.2}, {:.2}) | Fuel: {:.2}/{:.2}L | Payload: {:.2}/{:.2}kg | Status: {:?}",
                plane.id,
                plane.model,
                airport_name,
                loc.x,
                loc.y,
                plane.current_fuel,
                plane.specs.fuel_capacity,
                plane.current_payload,
                plane.specs.payload_capacity,
                plane.status,
            );
            if !plane.manifest.is_empty() {
                println!("  Manifest:");
                for order in plane.manifest.clone() {
                    println!(
                        "    [{}] {:?} -> {} | weight: {:.1}kg | value: ${:.2} | deadline: {} | destination: {}",
                        order.id,
                        order.name,
                        self.map.airports[order.destination_id].0.name,
                        order.weight,
                        order.value,
                        order.deadline,
                        order.destination_id
                    );
                }
            }

            Ok(())
        }
    }

    pub fn show_distances(&self, plane_id: usize) -> Result<(), GameError> {
        if plane_id > (self.airplanes.len() - 1) {
            return Err(GameError::PlaneIdInvalid { id: plane_id });
        }

        let plane = &self.airplanes[plane_id];

        // If plane is in transit, dont't calc
        if let AirplaneStatus::InTransit { .. } = plane.status {
            println!("Plane currently in transit");
            Ok(())
        } else {
            for (airport, coordinate) in &self.map.airports {
                let distance = plane.distance_to(coordinate);

                let can_land = plane.can_fly_to(airport, coordinate).is_ok();

                println!(
                    "ID: {} | {} at ({:.2}, {:.2}) | Runway: {:.0}m | Distance to: {:.2}km | Can land: {:?}",
                    airport.id,
                    airport.name,
                    coordinate.x,
                    coordinate.y,
                    airport.runway_length,
                    distance,
                    can_land
                );
            }
            Ok(())
        }
    }

    /// Buy an airplane is possible
    pub fn buy_plane(&mut self, model: &String, airport_id: usize) -> Result<(), GameError> {
        // Get copy of home coordinate
        let home_coord = {
            let (_airport, coord) = &self.map.airports[airport_id];
            *coord
        };

        // Borrow airport as mut
        let airport_ref = &mut self.map.airports[airport_id].0;

        match self.player.buy_plane(model, airport_ref, &home_coord) {
            Ok(_) => {
                // update expenses (can safely unwrap becuse else this wouldn't be ok)
                let new_plane = self.airplanes.last().unwrap();
                let buying_price = new_plane.specs.purchase_price;
                self.daily_expenses += buying_price;

                // Buy plane, update fleet and update arrival times
                self.airplanes = self.player.fleet.clone();
                self.arrival_times.push(self.time);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Load an order if possible.
    ///
    /// Returns [`GameError::PlaneIdInvalid`] if the plane doesn't exist or
    /// [`GameError::PlaneNotAtAirport`] if the plane isn't parked at an airport.
    pub fn load_order(&mut self, order_id: usize, plane_id: usize) -> Result<(), GameError> {
        let (plane_idx, airport_idx) = self.plane_and_airport_idx(plane_id)?;
        let plane = &mut self.airplanes[plane_idx];
        let airport = &mut self.map.airports[airport_idx].0;

        airport.load_order(order_id, plane)?;
        self.schedule(self.time + 1, Event::LoadingEvent { plane: plane_id });

        Ok(())
    }

    /// Unload all orders from the plane.
    ///
    /// Returns [`GameError::PlaneIdInvalid`] if the plane doesn't exist or
    /// [`GameError::PlaneNotAtAirport`] if the plane isn't parked at an airport.
    pub fn unload_all(&mut self, plane_id: usize) -> Result<(), GameError> {
        let (plane_idx, airport_idx) = self.plane_and_airport_idx(plane_id)?;

        let airport = &mut self.map.airports[airport_idx].0;
        let plane = &mut self.airplanes[plane_idx];
        let mut deliveries = plane.unload_all();

        // Check deliveries
        for delivery in deliveries.drain(..) {
            // reached the destination and before deadline
            if delivery.destination_id == airport.id {
                if delivery.deadline != 0 {
                    println!("Successfully delivered order {}", delivery.id);
                    self.player.cash += delivery.value;
                    self.daily_income += delivery.value;
                    self.player.record_delivery();
                } else {
                    println!("Order {}: Deadline expired", delivery.id)
                }
            }
            // not the destination so it goes into the stock at the airport
            else {
                println!(
                    "Order {} being stored at airport {}",
                    delivery.id, airport.id
                );
                airport.orders.push(delivery);
            }
        }

        self.schedule(self.time + 1, Event::LoadingEvent { plane: plane_id });

        Ok(())
    }

    /// Unload a list of orders from a plane.
    ///
    /// Returns [`GameError::PlaneIdInvalid`] if the plane doesn't exist or
    /// [`GameError::PlaneNotAtAirport`] if the plane isn't parked at an airport.
    pub fn unload_orders(
        &mut self,
        order_id: Vec<usize>,
        plane_id: usize,
    ) -> Result<(), GameError> {
        let (plane_idx, airport_idx) = self.plane_and_airport_idx(plane_id)?;

        let airport = &mut self.map.airports[airport_idx].0;
        let plane = &mut self.airplanes[plane_idx];

        for order in order_id {
            let delivery = plane.unload_order(order)?;

            if delivery.destination_id == airport.id {
                if delivery.deadline != 0 {
                    println!("Successfully delivered order {}", delivery.id);
                    self.player.cash += delivery.value;
                    self.daily_income += delivery.value;
                    self.player.record_delivery();
                } else {
                    println!("Order {}: Deadline expired", delivery.id)
                }
            }
            // not the destination so it goes into the stock at the airport
            else {
                println!(
                    "Order {} being stored at airport {}",
                    delivery.id, airport.id
                );
                airport.orders.push(delivery);
            }
        }
        self.schedule(self.time + 1, Event::LoadingEvent { plane: plane_id });

        Ok(())
    }

    /// Unload a specific order from a plane.
    ///
    /// Returns [`GameError::PlaneIdInvalid`] if the plane doesn't exist or
    /// [`GameError::PlaneNotAtAirport`] if the plane isn't parked at an airport.
    pub fn unload_order(&mut self, order_id: usize, plane_id: usize) -> Result<(), GameError> {
        let (plane_idx, airport_idx) = self.plane_and_airport_idx(plane_id)?;

        let airport = &mut self.map.airports[airport_idx].0;
        let plane = &mut self.airplanes[plane_idx];

        let delivery = plane.unload_order(order_id)?;

        if delivery.destination_id == airport.id {
            if delivery.deadline != 0 {
                println!("Successfully delivered order {}", delivery.id);
                self.player.cash += delivery.value;
                self.daily_income += delivery.value;
                self.player.record_delivery();
            } else {
                println!("Order {}: Deadline expired", delivery.id)
            }
        }
        // not the destination so it goes into the stock at the airport
        else {
            println!(
                "Order {} being stored at airport {}",
                delivery.id, airport.id
            );
            airport.orders.push(delivery);
        }

        self.schedule(self.time + 1, Event::LoadingEvent { plane: plane_id });

        Ok(())
    }

    /// Depart a plane to another airport.
    ///
    /// Returns [`GameError::PlaneIdInvalid`] if the plane doesn't exist,
    /// [`GameError::PlaneNotAtAirport`] if the plane isn't parked at an airport, or
    /// [`GameError::AirportIdInvalid`] if the destination airport doesn't exist.
    pub fn depart_plane(
        &mut self,
        plane_id: usize,
        destination_id: usize,
    ) -> Result<(), GameError> {
        let (plane_idx, origin_idx) = self.plane_and_airport_idx(plane_id)?;
        let plane = &mut self.airplanes[plane_idx];
        let (dest_airport, dest_coords) = &self
            .map
            .airports
            .iter()
            .find(|(a, _)| a.id == destination_id)
            .ok_or(GameError::AirportIdInvalid { id: destination_id })?;

        // consume fuel & get flight_hours
        // check before if we can get there, else we don't charge
        let flight_hours = plane.consume_flight_fuel(dest_airport, dest_coords)?;
        let origin_coord = plane.location;

        // charge parking
        let parked_since = self.arrival_times[plane_id];
        let parked_hours = (self.time - parked_since) as f32;
        let parking_fee = self.map.airports[origin_idx].0.parking_fee * parked_hours;
        self.player.cash -= parking_fee;
        self.daily_expenses += parking_fee;

        // set the status (no location change here!)
        plane.status = AirplaneStatus::InTransit {
            hours_remaining: flight_hours,
            destination: destination_id,
            origin: origin_coord,
            total_hours: flight_hours,
        };

        // kick off the first hourly tick
        self.schedule(self.time + 1, Event::FlightProgress { plane: plane_id });

        Ok(())
    }

    /// Refuel a plane and charge the player. Only works if the airplane is not in transit.
    ///
    /// Returns [`GameError::PlaneIdInvalid`] if the plane doesn't exist or
    /// [`GameError::PlaneNotAtAirport`] if the plane isn't parked at an airport.
    pub fn refuel_plane(&mut self, plane_id: usize) -> Result<(), GameError> {
        let (plane_idx, airport_idx) = self.plane_and_airport_idx(plane_id)?;
        let plane = &mut self.airplanes[plane_idx];

        // fuel airplane and log liters for dynamic pricing
        let fueling_fee = self.map.airports[airport_idx].0.fueling_fee(plane);
        self.map.airports[airport_idx].0.fuel_supply(plane);
        plane.refuel();

        // charge the player
        self.player.cash -= fueling_fee;
        self.daily_expenses += fueling_fee;

        // schedule fueling event
        self.schedule(self.time + 1, Event::RefuelComplete { plane: plane_id });

        Ok(())
    }

    /// Perform maintenance on airplane
    pub fn maintenance_on_airplane(&mut self, plane_id: usize) -> Result<(), GameError> {
        let airplane = &mut self.airplanes[plane_id];

        // cannot perform maintenance when not at an airport
        if matches!(
            airplane.status,
            AirplaneStatus::InTransit {
                hours_remaining: _,
                destination: _,
                origin: _,
                total_hours: _
            }
        ) {
            return Err(GameError::PlaneNotAtAirport { plane_id });
        }

        airplane.maintenance();
        self.schedule(self.time + 1, Event::Maintenance { plane: plane_id });
        Ok(())
    }

    pub fn execute_str(&mut self, line: &str) -> Result<(), GameError> {
        let cmd =
            parse_command(line).map_err(|e| GameError::InvalidCommand { msg: e.to_string() })?;
        self.execute(cmd)
    }

    pub fn execute(&mut self, cmd: Command) -> Result<(), GameError> {
        match cmd {
            ShowAirports { .. }
            | ShowAirport { .. }
            | ShowAirplanes
            | ShowAirplane { .. }
            | ShowDistances { .. }
            | ShowCash
            | ShowTime
            | ShowStats
            | ShowModels
            | LoadConfig { .. }
            | Exit => Ok(()),
            BuyPlane { model, airport } => self.buy_plane(&model, airport),
            LoadOrder { order, plane } => self.load_order(order, plane),
            LoadOrders { orders, plane } => {
                for o in orders {
                    self.load_order(o, plane)?;
                }
                Ok(())
            }
            UnloadOrder { order, plane } => self.unload_order(order, plane),
            UnloadOrders { orders, plane } => {
                for o in orders {
                    self.unload_order(o, plane)?;
                }
                Ok(())
            }
            UnloadAll { plane } => self.unload_all(plane),
            Refuel { plane } => self.refuel_plane(plane),
            DepartPlane { plane, dest } => self.depart_plane(plane, dest),
            HoldPlane { .. } => Ok(()),
            Advance { hours } => {
                self.advance(hours);
                Ok(())
            }
            SaveGame { name } => self
                .save_game(&name)
                .map_err(|e| GameError::InvalidCommand { msg: e.to_string() }),
            LoadGame { name } => {
                *self = Game::load_game(&name)
                    .map_err(|e| GameError::InvalidCommand { msg: e.to_string() })?;
                Ok(())
            }
            Maintenance { plane_id } => self.maintenance_on_airplane(plane_id),
        }
    }

    pub fn observe(&self) -> Observation {
        let airports = self
            .map
            .airports
            .iter()
            .map(|(airport, coord)| AirportObs {
                id: airport.id,
                name: airport.name.clone(),
                x: coord.x,
                y: coord.y,
                fuel_price: airport.fuel_price,
                runway_length: airport.runway_length,
                num_orders: airport.orders.len(),
            })
            .collect();

        let planes = self
            .airplanes
            .iter()
            .map(|plane| {
                let (destination, hours_remaining) = match plane.status {
                    AirplaneStatus::InTransit {
                        destination,
                        hours_remaining,
                        ..
                    } => (Some(destination), Some(hours_remaining)),
                    _ => (None, None),
                };
                PlaneObs {
                    id: plane.id,
                    model: format!("{:?}", plane.model),
                    x: plane.location.x,
                    y: plane.location.y,
                    status: format!("{:?}", plane.status),
                    fuel: FuelObs {
                        current: plane.current_fuel,
                        capacity: plane.specs.fuel_capacity,
                    },
                    payload: PayloadObs {
                        current: plane.current_payload,
                        capacity: plane.specs.payload_capacity,
                    },
                    destination,
                    hours_remaining,
                }
            })
            .collect();

        Observation {
            time: self.time,
            cash: self.player.cash,
            airports,
            planes,
        }
    }

    // ************************
    // ******* GUI APIs *******
    // ************************

    pub fn get_cash(&self) -> f32 {
        self.player.cash
    }

    pub fn get_time(&self) -> String {
        self.days_and_hours(self.time)
    }

    pub fn airports(&self) -> &[(Airport, Coordinate)] {
        &self.map.airports
    }

    pub fn planes(&self) -> &Vec<Airplane> {
        &self.airplanes
    }
}
