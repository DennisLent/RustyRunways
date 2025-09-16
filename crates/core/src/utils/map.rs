use crate::utils::{
    airport::Airport,
    coordinate::Coordinate,
    orders::order::{OrderAirportInfo, OrderGenerationParams},
};
use rand::{Rng, SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    pub num_airports: usize,
    pub airports: Vec<(Airport, Coordinate)>,
    pub seed: u64,
    next_order_id: usize,
    #[serde(default)]
    pub order_params: OrderGenerationParams,
}

impl Map {
    /// Airports from a random seed.
    /// Allows you to input a specific amount of airports or not.
    /// Airports are already stocked with orders.
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
            next_order_id: 0,
            order_params: OrderGenerationParams::default(),
        };

        map.restock_airports();

        map
    }

    /// Restock the orders in the airport
    pub fn restock_airports(&mut self) {
        let airport_infos: Vec<OrderAirportInfo> = self
            .airports
            .iter()
            .map(|(airport, coord)| OrderAirportInfo {
                id: airport.id,
                runway_length: airport.runway_length,
                coordinate: *coord,
            })
            .collect();

        for (airport, _) in self.airports.iter_mut() {
            airport.generate_orders(
                self.seed,
                &airport_infos,
                &mut self.next_order_id,
                &self.order_params,
            );
        }
    }

    /// Remove all orders from every airport and reset the order id counter.
    pub fn clear_orders(&mut self) {
        for (airport, _) in self.airports.iter_mut() {
            airport.orders.clear();
        }
        self.next_order_id = 0;
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

    /// Build a map from explicit airport configs.
    pub fn from_airports(
        seed: u64,
        airports: Vec<(Airport, Coordinate)>,
        order_params: OrderGenerationParams,
        next_order_id: usize,
    ) -> Self {
        Map {
            num_airports: airports.len(),
            airports,
            seed,
            next_order_id,
            order_params,
        }
    }
}
