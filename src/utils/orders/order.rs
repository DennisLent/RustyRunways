use super::cargo::CargoType;
use crate::utils::coordinate::Coordinate;
use rand::{Rng, SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

// Constants for scaling distance and deadline
pub const ALPHA: f32 = 0.5;
pub const BETA: f32 = 0.7;

// Constant for the maximum deadline (we take 14 days)
pub const MAX_DEADLINE: usize = 14;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Order {
    pub id: usize, //Global unique id
    pub name: CargoType,
    pub weight: f32,
    pub value: f32,
    pub deadline: usize,
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
    ) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        let cargo_count = CargoType::iter().count();

        // This should be fine, we know how many types we have and therefore we can just pick it out
        let cargo_type = CargoType::iter()
            .nth(rng.gen_range(0..cargo_count))
            .unwrap();

        // idk but 30 days seems to be a good max
        let deadline_day = rng.gen_range(1..=MAX_DEADLINE);
        let deadline = deadline_day * 24;

        let mut destination_id = rng.gen_range(0..num_airports);
        if destination_id == origin_airport_id {
            destination_id = (destination_id + 1) % num_airports;
        }

        let origin_coord = airport_coordinates[origin_airport_id];
        let dest_coord = airport_coordinates[destination_id];
        let (dx, dy) = (origin_coord.x - dest_coord.x, origin_coord.y - dest_coord.y);
        let distance = (dx * dx + dy * dy).sqrt();

        let weight = rng.gen_range(100.0..=20000.0);

        // Value is scaled using the cargo type, size, distance and deadline
        // More 'expensive', heavy objects that go further in a short time have a higher value
        let (min_price, max_price) = cargo_type.price_range();
        let price_per_kg = rng.gen_range(min_price..=max_price);
        let base_value = weight * price_per_kg;

        let distance_factor = 1.0 + ALPHA * (distance / 10000.0);
        let time_factor = 1.0
            + BETA * (((MAX_DEADLINE * 24) as f32 - deadline as f32) / (MAX_DEADLINE * 24) as f32);

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
