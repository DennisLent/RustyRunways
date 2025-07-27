use crate::utils::{airport::Airport, coordinate::Coordinate};
use rand::{Rng, SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    pub num_airports: usize,
    pub airports: Vec<(Airport, Coordinate)>,
    pub seed: u64,
}

impl Map {
    /// Airports from a random seed
    /// Allows you to input a specific amount of airports or not
    /// Airports are already stocked with orders
    pub fn generate_from_seed(seed: u64, num_airports: Option<usize>) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        let num_airports = num_airports.unwrap_or_else(|| rng.gen_range(4..=10));

        let mut airport_list = Vec::with_capacity(num_airports);

        for i in 0..num_airports {
            let x: f32 = rng.gen_range(0.0..=10000.0);
            let y: f32 = rng.gen_range(0.0..=10000.0);
            let coordinates = Coordinate::new(x, y);

            let airport = Airport::generate_random(seed, i);

            airport_list.push((airport, coordinates));
        }

        let mut map = Map {
            num_airports,
            airports: airport_list,
            seed,
        };

        map.restock_airports();

        map
    }

    /// Restock the orders in the airport
    pub fn restock_airports(&mut self) {
        for (airport, _) in self.airports.iter_mut() {
            airport.generate_orders(self.seed, self.num_airports);
        }
    }

    /// Find the minimum distance between two airports.
    /// Helps us determine the starting airplane for a given map.
    pub fn min_distance(&self) -> (f32, usize) {
        let mut min_distance = f32::INFINITY;
        let mut start_index: usize = 0;

        for (airport1, coord1) in self.airports.iter() {
            for (airport2, coord2) in self.airports.iter() {
                if airport1.id != airport2.id {
                    let dx = (coord1.x - coord2.x).abs();
                    let dy = (coord1.y - coord2.y).abs();

                    let distance = (dx * dx + dy * dy).sqrt();
                    if distance < min_distance {
                        min_distance = distance;
                        start_index = airport1.id;
                    }
                }
            }
        }

        (min_distance, start_index)
    }
}
