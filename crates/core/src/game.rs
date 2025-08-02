use serde::{Deserialize, Serialize};

use crate::events::{Event, GameTime, ScheduledEvent};
use crate::player::Player;
use crate::statistics::DailyStats;
use crate::utils::airplanes::airplane::Airplane;
use crate::utils::airplanes::models::AirplaneStatus;
use crate::utils::coordinate::Coordinate;
use crate::utils::errors::GameError;
use crate::utils::map::Map;
use crate::utils::orders::order::MAX_DEADLINE;
use std::collections::BinaryHeap;
use std::path::{Path, PathBuf};
use std::{fs, io};

const RESTOCK_CYCLE: u64 = MAX_DEADLINE * 24;
const REPORT_INTERVAL: u64 = 24;

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
        };

        game.schedule(RESTOCK_CYCLE, Event::Restock);
        game.schedule(REPORT_INTERVAL, Event::DailyStats);

        game
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

    /// Write the entire game state to JSON to save
    pub fn save_game(&self, name: &str) -> io::Result<()> {
        let save_dir = Path::new("save_games");
        fs::create_dir_all(&save_dir)?;

        let mut path = PathBuf::from(save_dir);
        path.push(format!("{}.json", name));

        let file = fs::File::create(&path)?;
        let writer = io::BufWriter::new(file);
        serde_json::to_writer_pretty(writer, self)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
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
        let game: Game =
            serde_json::from_reader(reader).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(game)
    }

    /// Schedule `event` to occur at absolute simulation time `time`.
    pub fn schedule(&mut self, time: GameTime, event: Event) {
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
                            self.schedule(self.time + 1, Event::FlightProgress { plane });
                        } else {
                            let (airport, _) = &self.map.airports[destination];
                            let landing_fee = airport.landing_fee(airplane);
                            self.player.cash -= landing_fee;
                            self.daily_expenses += landing_fee;

                            self.arrival_times[plane] = self.time;
                            airplane.location = dest_coord;
                            airplane.status = AirplaneStatus::Parked;
                        }
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
        self.schedule(self.time + 1, Event::LoadingEvent { plane: plane_id });

        Ok(())
    }

    /// Unload all orders from the plane
    pub fn unload_all(&mut self, plane_id: usize) -> Result<(), GameError> {
        let plane = self
            .airplanes
            .iter_mut()
            .find(|p| p.id == plane_id)
            .ok_or(GameError::PlaneIdInvalid { id: plane_id })?;

        let airport_idx = self
            .map
            .airports
            .iter()
            .position(|(_, coord)| *coord == plane.location)
            .ok_or(GameError::PlaneNotAtAirport { plane_id })?;

        let airport = &mut self.map.airports[airport_idx].0;
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

    /// Unload a specific order
    pub fn unload_orders(
        &mut self,
        order_id: Vec<usize>,
        plane_id: usize,
    ) -> Result<(), GameError> {
        let plane = self
            .airplanes
            .iter_mut()
            .find(|p| p.id == plane_id)
            .ok_or(GameError::PlaneIdInvalid { id: plane_id })?;

        let airport_idx = self
            .map
            .airports
            .iter()
            .position(|(_, coord)| *coord == plane.location)
            .ok_or(GameError::PlaneNotAtAirport { plane_id })?;

        let airport = &mut self.map.airports[airport_idx].0;

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

    /// Unload a specific order
    pub fn unload_order(&mut self, order_id: usize, plane_id: usize) -> Result<(), GameError> {
        let plane = self
            .airplanes
            .iter_mut()
            .find(|p| p.id == plane_id)
            .ok_or(GameError::PlaneIdInvalid { id: plane_id })?;

        let airport_idx = self
            .map
            .airports
            .iter()
            .position(|(_, coord)| *coord == plane.location)
            .ok_or(GameError::PlaneNotAtAirport { plane_id })?;

        let airport = &mut self.map.airports[airport_idx].0;

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

    pub fn depart_plane(
        &mut self,
        plane_id: usize,
        destination_id: usize,
    ) -> Result<(), GameError> {
        let (plane, origin_idx) = {
            let plane = self
                .airplanes
                .iter_mut()
                .find(|p| p.id == plane_id)
                .ok_or(GameError::PlaneIdInvalid { id: plane_id })?;

            let origin_idx = self
                .map
                .airports
                .iter_mut()
                .find(|(_, c)| *c == plane.location)
                .ok_or(GameError::PlaneNotAtAirport { plane_id })?
                .0
                .id;

            (plane, origin_idx)
        };
        let (dest_airport, dest_coords) = &self
            .map
            .airports
            .iter()
            .find(|(a, _)| a.id == destination_id)
            .ok_or(GameError::AirportIdInvalid { id: destination_id })?;

        // charge parking
        let parked_since = self.arrival_times[plane_id];
        let parked_hours = (self.time - parked_since) as f32;
        let parking_fee = self.map.airports[origin_idx].0.parking_fee * parked_hours;
        self.player.cash -= parking_fee;
        self.daily_expenses += parking_fee;

        // consume fuel & get flight_hours
        let flight_hours = plane.consume_flight_fuel(dest_airport, dest_coords)?;
        let origin_coord = plane.location;

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

    /// Refuel a plane and charge the player. Only works if the airplne is not in transit.
    pub fn refuel_plane(&mut self, plane_id: usize) -> Result<(), GameError> {
        let plane = self
            .airplanes
            .iter_mut()
            .find(|p| p.id == plane_id)
            .ok_or(GameError::PlaneIdInvalid { id: plane_id })?;

        let airport_idx = self
            .map
            .airports
            .iter()
            .position(|(_, coord)| *coord == plane.location)
            .ok_or(GameError::PlaneNotAtAirport { plane_id })?;

        // fuel airplane
        let fueling_fee = self.map.airports[airport_idx].0.fueling_fee(plane);
        plane.refuel();

        // charge the player
        self.player.cash -= fueling_fee;
        self.daily_expenses += fueling_fee;

        // schedule fueling event
        self.schedule(self.time + 1, Event::RefuelComplete { plane: plane_id });

        Ok(())
    }
}
