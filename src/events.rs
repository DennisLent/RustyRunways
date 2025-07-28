use crate::utils::coordinate::Coordinate;

/// Global time unit: hours since simulation start.
pub type GameTime = u64;

/// All events that can occur in the world.
#[derive(Clone, Debug)]
pub enum Event {
    /// A plane departs from airport: charge parking duration
    FlightDeparture { plane: usize, airport: usize },
    /// Plane is loading: lasts for 1 game tick
    LoadingComplete { plane: usize, airport: usize },
    /// Flight is currently on its way
    FlightEnRoute {
        plane: usize,
        origin: usize,
        destination: usize,
    },
    /// Flight has arrived at destination airport
    FlightArrival { plane: usize, airport: usize },
    /// Plane is unloading: lasts for 1 game tick and pays the player
    UnloadingComplete { plane: usize, airport: usize },
    /// Refuels the airplane: lasts for 1 game tick and switches the airplane to parked
    RefuelComplete { plane: usize, airport: usize },
    /// An order at `airport` reaches its delivery deadline.
    OrderDeadline { airport: usize, order_index: usize },
    // TODO: add MaintenanceComplete, RefuelComplete, etc.
}

/// Wraps an `Event` with its scheduled occurrence time.
/// Implements `Ord` such that the earliest time is popped first from a max-heap.
#[derive(Clone, Debug)]
pub struct ScheduledEvent {
    pub time: GameTime,
    pub event: Event,
}

// Only compare the `time` field for equality:
impl PartialEq for ScheduledEvent {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}
impl Eq for ScheduledEvent {}

// Only compare the `time` field for ordering (reverse so min-heap behavior):
impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.time.cmp(&self.time)
    }
}
