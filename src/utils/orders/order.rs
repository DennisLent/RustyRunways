use super::cargo::CargoType;
use rand::{Rng, SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Order {
    pub name: CargoType,
    pub weight: f32,
    pub value: f32,
    pub deadline: usize,
    pub origin_id: usize,
    pub destination_id: usize,
}

impl Order {
    // prices can range from $1.00 to $8.00 per kilogram
    pub fn new(seed: u64, origin_airport_id: usize, num_airports: usize) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        let cargo_count = CargoType::iter().count();
        // This should be fine, we know how many types we have and therefore we can just pick it out
        let cargo_type = CargoType::iter()
            .nth(rng.gen_range(0..cargo_count))
            .unwrap();

        let weight = rng.gen_range(100.0..=20000.0);

        let (min_price, max_price) = cargo_type.price_range();
        let price_per_kg = rng.gen_range(min_price..=max_price);
        let value = weight * price_per_kg;

        // idk but 30 days seems to be a good max
        let deadline = rng.gen_range(1..=30);

        let mut destination_id = rng.gen_range(0..num_airports);
        if destination_id == origin_airport_id {
            destination_id = (destination_id + 1) % num_airports;
        }

        Order {
            name: cargo_type,
            weight,
            value,
            deadline,
            origin_id: origin_airport_id,
            destination_id,
        }
    }
}
