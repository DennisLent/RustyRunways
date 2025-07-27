use std::collections::BinaryHeap;
use crate::events::{Event, ScheduledEvent, GameTime};
use crate::player::Player;
use crate::utils::airplanes::airplane::Airplane;
use crate::utils::map::Map;

/// Holds all mutable world state and drives the simulation via scheduled events.
#[derive(Debug)]
pub struct Game {
    /// Current simulation time (hours)
    pub time: GameTime,
    /// The world map of airports and coordinates
    pub map: Map,
    /// All airplanes in the world
    pub airplanes: Vec<Airplane>,
    /// The player's company (cash, fleet, deliveries)
    pub player: Player,
    /// Future events, ordered by their `time` (earliest first)
    pub events: BinaryHeap<ScheduledEvent>,
}

impl Game {
    /// Initialize a new game with `num_airports`, seeded randomness, and player's starting cash.
    pub fn new(seed: u64, num_airports: Option<usize>, starting_cash: f32) -> Self {

        let mut map = Map::generate_from_seed(seed, num_airports);
        map.restock_airports();

        let airplanes = Vec::new();
        let player    = Player::new(starting_cash, &map);
        let events    = BinaryHeap::new();

        Game { time: 0, map, airplanes, player, events }
    }

    /// Schedule `event` to occur at absolute simulation time `time`.
    pub fn schedule(&mut self, time: GameTime, event: Event) {
        self.events.push(ScheduledEvent { time, event });
    }

    /// Process the next scheduled event; advance `self.time`. Returns false if no events remain.
    pub fn tick_event(&mut self) -> bool {
        if let Some(scheduled) = self.events.pop() {
            self.time = scheduled.time;
            match scheduled.event {
                Event::FlightArrival { plane, airport_coord } => {
                    let plane = &mut self.airplanes[plane];
                    plane.location = airport_coord;
                    let delivered = plane.unload_all();
                    let income: f32 = delivered.iter().map(|o| o.value).sum();
                    self.player.cash += income;
                    self.player.record_delivery();
                }
                Event::OrderDeadline { airport, order_index } => {
                    let (ref mut ap, _) = self.map.airports[airport];
                    if order_index < ap.orders.len() {
                        ap.orders.remove(order_index);
                        // log missed delivery?
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
}
