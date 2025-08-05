use serde::{Deserialize, Serialize};

/// Global time unit: hours since simulation start.
pub type GameTime = u64;

/// All events that can occur in the world.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    LoadingEvent {
        plane: usize,
    },

    FlightTakeOff {
        plane: usize,
        origin: usize,
        destination: usize,
    },

    FlightProgress {
        plane: usize,
    },

    RefuelComplete {
        plane: usize,
    },

    OrderDeadline {
        airport: usize,
        order: usize,
    },

    Restock,

    DailyStats,

    // Event to adjust fuel price every 6 hours based on demand
    DynamicPricing,

    // Bigger, world-wide events that cause more fluctuation
    WorldEvent {
        airport: Option<usize>,
        factor: f32,
        duration: GameTime,
    },

    WorldEventEnd {
        airport: Option<usize>,
        factor: f32,
    },

    MaintenanceCheck,
    Maintenance {
        plane: usize,
    },
}

/// Wraps an `Event` with its scheduled occurrence time.
/// Implements `Ord` such that the earliest time is popped first from a max-heap.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
