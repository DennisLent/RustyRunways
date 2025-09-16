use super::cargo::CargoType;
use crate::{events::GameTime, utils::coordinate::Coordinate};
use rand::{Rng, SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

// Default tuning values used when no custom configuration is provided.
pub const DEFAULT_ALPHA: f32 = 0.5;
pub const DEFAULT_BETA: f32 = 0.7;
pub const DEFAULT_MAX_DEADLINE_HOURS: u64 = 14 * 24;
pub const DEFAULT_MIN_WEIGHT: f32 = 100.0;
pub const DEFAULT_MAX_WEIGHT: f32 = 20_000.0;

/// Parameters that control how random cargo orders are generated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderGenerationParams {
    pub max_deadline_hours: u64,
    pub min_weight: f32,
    pub max_weight: f32,
    pub alpha: f32,
    pub beta: f32,
}

impl Default for OrderGenerationParams {
    fn default() -> Self {
        OrderGenerationParams {
            max_deadline_hours: DEFAULT_MAX_DEADLINE_HOURS,
            min_weight: DEFAULT_MIN_WEIGHT,
            max_weight: DEFAULT_MAX_WEIGHT,
            alpha: DEFAULT_ALPHA,
            beta: DEFAULT_BETA,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Order {
    pub id: usize, // Global unique id
    pub name: CargoType,
    pub weight: f32,
    pub value: f32,
    pub deadline: GameTime,
    pub origin_id: usize,
    pub destination_id: usize,
}

impl Order {
    // prices can range from $1.00 to $8.00 per kilogram
    pub fn new(
        seed: u64,
        order_id: usize,
        origin_airport_id: usize,
        airport_coordinates: &[Coordinate],
        num_airports: usize,
        params: &OrderGenerationParams,
    ) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        let cargo_count = CargoType::iter().count();

        // This should be fine, we know how many types we have and therefore we can just pick it out
        let cargo_type = CargoType::iter()
            .nth(rng.gen_range(0..cargo_count))
            .unwrap();

        let max_deadline_hours = params.max_deadline_hours.max(1);
        let deadline = rng.gen_range(1..=max_deadline_hours);

        let mut destination_id = rng.gen_range(0..num_airports);
        if destination_id == origin_airport_id {
            destination_id = (destination_id + 1) % num_airports;
        }

        let origin_coord = airport_coordinates[origin_airport_id];
        let dest_coord = airport_coordinates[destination_id];
        let (dx, dy) = (origin_coord.x - dest_coord.x, origin_coord.y - dest_coord.y);
        let distance = (dx * dx + dy * dy).sqrt();

        let weight = rng.gen_range(params.min_weight..=params.max_weight);

        // Value is scaled using the cargo type, size, distance and deadline
        // More 'expensive', heavy objects that go further in a short time have a higher value
        let (min_price, max_price) = cargo_type.price_range();
        let price_per_kg = rng.gen_range(min_price..=max_price);
        let base_value = weight * price_per_kg;

        let distance_factor = 1.0 + params.alpha * (distance / 10000.0);
        let max_deadline_hours_f32 = max_deadline_hours as f32;
        let normalized_deadline =
            ((max_deadline_hours_f32 - deadline as f32) / max_deadline_hours_f32).clamp(0.0, 1.0);
        let time_factor = 1.0 + params.beta * normalized_deadline;

        let value = (base_value * distance_factor * time_factor).round();

        Order {
            id: order_id,
            name: cargo_type,
            weight,
            value,
            deadline,
            origin_id: origin_airport_id,
            destination_id,
        }
    }
}
