use crate::events::{Event, GameTime, ScheduledEvent};
use crate::player::Player;
use crate::utils::airplanes::airplane::Airplane;
use crate::utils::airplanes::models::AirplaneStatus;
use crate::utils::errors::GameError;
use crate::utils::map::Map;
use std::collections::BinaryHeap;

/// Holds all mutable world state and drives the simulation via scheduled events.
#[derive(Debug)]
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
}

impl Game {
    /// Initialize a new game with `num_airports`, seeded randomness, and player's starting cash.
    pub fn new(seed: u64, num_airports: Option<usize>, starting_cash: f32) -> Self {
        let map = Map::generate_from_seed(seed, num_airports);

        let arrival_times = Vec::new();
        let player = Player::new(starting_cash, &map);
        let airplanes = player.fleet.clone();
        let events = BinaryHeap::new();

        Game {
            time: 0,
            map,
            airplanes,
            player,
            events,
            arrival_times,
        }
    }

    /// Schedule `event` to occur at absolute simulation time `time`.
    pub fn schedule(&mut self, time: GameTime, event: Event) {
        self.events.push(ScheduledEvent { time, event });
    }

    /// Process the next scheduled event; advance `self.time`. Returns false if no events remain.
    pub fn tick_event(&mut self) -> bool {
        if let Some(scheduled) = self.events.pop() {
            // advance time
            self.time = scheduled.time;

            match scheduled.event {
                // Departure: charge parking, then load (1h)
                Event::FlightDeparture { plane, airport } => {
                    let arrival = self.arrival_times.get(plane).copied().unwrap_or(self.time);
                    let parked_hours = self.time.saturating_sub(arrival) as f32;
                    let fee_per_hr = self.map.airports[airport].0.parking_fee;
                    self.player.cash -= parked_hours * fee_per_hr;

                    self.airplanes[plane].status = AirplaneStatus::Loading;

                    // loading takes 1h
                    self.schedule(self.time + 1, Event::LoadingComplete { plane, airport });
                }

                // 2) Loaded: immediately schedule en‑route + arrival
                Event::LoadingComplete { plane, airport } => {
                    // find the order we just loaded
                    let dest = self.airplanes[plane]
                        .manifest
                        .first()
                        .expect("plane must have exactly one order")
                        .destination_id;

                    // schedule "in‑flight" status right now...
                    self.schedule(
                        self.time,
                        Event::FlightEnRoute {
                            plane,
                            origin: airport,
                            destination: dest,
                        },
                    );

                    // compute flight time (hours, rounded up)
                    let (ax, ay) = {
                        let c = self.map.airports[airport].1;
                        (c.x, c.y)
                    };
                    let (bx, by) = {
                        let c = self.map.airports[dest].1;
                        (c.x, c.y)
                    };
                    let dx = ax - bx;
                    let dy = ay - by;
                    let dist = (dx * dx + dy * dy).sqrt();
                    let speed = self.airplanes[plane].specs.cruise_speed;
                    let flight_hours = (dist / speed).ceil() as GameTime;

                    // chedule the actual arrival
                    self.schedule(
                        self.time + flight_hours,
                        Event::FlightArrival {
                            plane,
                            airport: dest,
                        },
                    );
                }

                // In‑flight: update status so you can query progress if desired
                Event::FlightEnRoute {
                    plane, destination, ..
                } => {
                    let coords = self.map.airports[destination].1;
                    self.airplanes[plane].status = AirplaneStatus::InTransit {
                        destination: coords,
                    };
                }

                // Arrival: snap location, unload (1h), record arrival_time
                Event::FlightArrival { plane, airport } => {
                    let coords = self.map.airports[airport].1;
                    let plane_ref = &mut self.airplanes[plane];
                    plane_ref.location = coords;

                    // schedule unloading in 1h
                    plane_ref.status = AirplaneStatus::Unloading;
                    self.arrival_times[plane] = self.time;
                    self.schedule(self.time + 1, Event::UnloadingComplete { plane, airport });
                }

                // Unloading complete: credit cargo value, then refuel (1h)
                Event::UnloadingComplete { plane, airport } => {
                    let plane_ref = &mut self.airplanes[plane];
                    let delivered = plane_ref.unload_all();
                    let income: f32 = delivered.iter().map(|o| o.value).sum();
                    self.player.cash += income;
                    for _ in &delivered {
                        self.player.record_delivery();
                    }

                    // schedule refuel in 1h
                    plane_ref.status = AirplaneStatus::Refueling;
                    self.schedule(self.time + 1, Event::RefuelComplete { plane, airport });
                }

                // Refuel complete: charge fuel cost, top off, mark parked
                Event::RefuelComplete { plane, airport } => {
                    let plane_ref = &mut self.airplanes[plane];
                    let specs = plane_ref.specs;
                    let used = specs.fuel_capacity - plane_ref.current_fuel;
                    let price = self.map.airports[airport].0.fuel_price;
                    self.player.cash -= used * price;

                    plane_ref.current_fuel = specs.fuel_capacity;
                    plane_ref.status = AirplaneStatus::Parked;
                    self.arrival_times[plane] = self.time;
                }

                // Order deadlines unchanged
                Event::OrderDeadline {
                    airport,
                    order_index,
                } => {
                    let (ref mut a, _) = self.map.airports[airport];
                    if order_index < a.orders.len() {
                        a.orders.remove(order_index);
                    }
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
    }

    /// Display a summary of all airports in the map, including their orders.
    /// If with_orders is true, show the orders alongside.
    pub fn list_airports(&self, with_orders: bool) {
        println!("Airports ({} total):", self.map.num_airports);
        for (airport, coord) in &self.map.airports {
            println!(
                "ID: {} | {} at ({:.2}, {:.2}) | Runway: {:.0}m | Fuel: {:.2}/L | Parking: {:.2}/hr | Landing Fee: {:.2}/ton",
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
                            "    [{}] {:?} -> {} | weight: {:.1}kg | value: ${:.2} | deadline: {} h",
                            order.id,
                            order.name,
                            self.map.airports[order.destination_id].0.name,
                            order.weight,
                            order.value,
                            order.deadline,
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
            "ID: {} | {} at ({:.2}, {:.2}) | Runway: {:.0}m | Fuel: {:.2}/L | Parking: {:.2}/hr | Landing Fee: {:.2}/ton",
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
                        "    [{}] {:?} -> {} | weight: {:.1}kg | value: ${:.2} | deadline: {} h",
                        order.id,
                        order.name,
                        self.map.airports[order.destination_id].0.name,
                        order.weight,
                        order.value,
                        order.deadline,
                    );
                }
            }
        }

        Ok(())
    }

    /// Display a summary of all airplanes in the game.
    pub fn list_airplanes(&self) {
        println!("Airplanes ({} total):", self.airplanes.len());
        for plane in &self.airplanes {
            let loc = &plane.location;
            println!(
                "ID: {} | {:?} at ({:.2}, {:.2}) | Fuel: {:.2}/{:.2} | Payload: {:.2}/{:.2} | Status: {:?}",
                plane.id,
                plane.model,
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

    /// Display a summary of a single airplane in the game.
    pub fn list_airplane(&self, plane_id: usize) -> Result<(), GameError> {
        if plane_id > (self.airplanes.len() - 1) {
            return Err(GameError::PlaneIdInvalid { id: plane_id });
        }

        let plane = &self.airplanes[plane_id];

        let loc = &plane.location;
        println!(
            "ID: {} | {:?} at ({:.2}, {:.2}) | Fuel: {:.2}/{:.2} | Payload: {:.2}/{:.2} | Status: {:?}",
            plane.id,
            plane.model,
            loc.x,
            loc.y,
            plane.current_fuel,
            plane.specs.fuel_capacity,
            plane.current_payload,
            plane.specs.payload_capacity,
            plane.status,
        );

        Ok(())
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
                self.airplanes = self.player.fleet.clone();
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Load an order if possible
    pub fn load_order(&mut self, order_id: usize, plane_id: usize) -> Result<(), GameError> {
        // Find the airplane
        let plane = self
            .airplanes
            .iter_mut()
            .find(|p| p.id == plane_id)
            .ok_or(GameError::PlaneIdInvalid { id: plane_id })?;

        // Find the associated airport
        let airport_idx = self
            .map
            .airports
            .iter()
            .position(|(_, coord)| *coord == plane.location)
            .ok_or(GameError::PlaneNotAtAirport { plane_id })?;

        let airport = &mut self.map.airports[airport_idx].0;

        airport.load_order(order_id, plane)?;

        Ok(())
    }

    pub fn depart_plane(
        &mut self,
        plane_id: usize,
        destination_id: usize,
    ) -> Result<(), GameError> {
        // Find the airplane
        let plane = self
            .airplanes
            .iter_mut()
            .find(|p| p.id == plane_id)
            .ok_or(GameError::PlaneIdInvalid { id: plane_id })?;

        // Find the associated airport
        let airport_idx = self
            .map
            .airports
            .iter()
            .position(|(a, _)| a.id == destination_id)
            .ok_or(GameError::PlaneNotAtAirport { plane_id })?;

        let (airport, airport_coordinates) = &self.map.airports[airport_idx];

        match plane.fly_to(airport, airport_coordinates) {
            Ok(()) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub fn advance(&mut self, hours: GameTime) {
        let target_time = self.time + hours;

        // Find earliest event
        if let Some(next) = self.events.peek() {
            if next.time <= target_time {
                let _ = self.tick_event();
                return;
            }
        }

        self.time = target_time;
    }

    pub fn show_cash(&self) {
        println!("${}", self.player.cash);
    }

    pub fn show_time(&self) {
        println!("{} h", self.time);
    }
}
