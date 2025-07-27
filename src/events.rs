use crate::utils::coordinate::Coordinate;

/// Global time unit: hours since simulation start.
pub type GameTime = u64;

/// All events that can occur in the world.
/// Note: we no longer embed `time` here; it lives in `ScheduledEvent`.
#[derive(Clone, Debug)]
pub enum Event {
    /// A plane arrives at the given coordinate.
    FlightArrival {
        plane:         usize,      // index into Game.airplanes
        airport_coord: Coordinate,
    },
    /// An order at `airport` reaches its delivery deadline.
    OrderDeadline {
        airport:       usize,
        order_index:   usize,
    },
    // TODO: add MaintenanceComplete, RefuelComplete, etc.
}

/// Wraps an `Event` with its scheduled occurrence time.
/// Implements `Ord` such that the earliest time is popped first from a max-heap.
#[derive(Clone, Debug)]
pub struct ScheduledEvent {
    pub time:  GameTime,
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
        other.time.partial_cmp(&self.time)
    }
}
impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.time.cmp(&self.time)
    }
}
