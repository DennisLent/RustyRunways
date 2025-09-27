use crate::utils::{
    airport::Airport,
    coordinate::Coordinate,
    orders::{DemandGenerationParams, order::OrderAirportInfo},
};
use rand::{Rng, SeedableRng, rngs::StdRng, seq::SliceRandom};
use serde::{Deserialize, Serialize};
use std::f32::consts::TAU;

/// A procedurally generated world map with airports and demand parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    pub num_airports: usize,
    pub airports: Vec<(Airport, Coordinate)>,
    pub seed: u64,
    next_order_id: usize,
    #[serde(default)]
    pub demand_params: DemandGenerationParams,
}

impl Map {
    fn clustered_coordinates(seed: u64, count: usize) -> Vec<Coordinate> {
        if count == 0 {
            return Vec::new();
        }

        let mut rng = StdRng::seed_from_u64(seed.wrapping_mul(31).wrapping_add(17));
        let cluster_count = count.clamp(1, (count as f32 / 4.0).ceil() as usize).max(1);
        let cluster_count = cluster_count.min(count);

        let mut centers: Vec<Coordinate> = Vec::with_capacity(cluster_count);
        let min_separation = 2_000.0_f32;
        for _ in 0..cluster_count {
            let mut attempts = 0;
            loop {
                attempts += 1;
                let x = rng.gen_range(800.0..=9_200.0);
                let y = rng.gen_range(800.0..=9_200.0);
                let candidate = Coordinate::new(x, y);
                if centers
                    .iter()
                    .all(|c| ((c.x - x).powi(2) + (c.y - y).powi(2)).sqrt() >= min_separation)
                    || attempts > 20
                {
                    centers.push(candidate);
                    break;
                }
            }
        }

        let mut assignments: Vec<usize> = (0..count)
            .map(|_| rng.gen_range(0..cluster_count))
            .collect();
        assignments.shuffle(&mut rng);

        let mut coords = Vec::with_capacity(count);
        for cluster_idx in assignments {
            let center = centers[cluster_idx];
            let radius = rng.gen_range(350.0..=1_200.0);
            let angle = rng.gen_range(0.0..TAU);
            let distance = radius * rng.gen_range(0.0_f32..=1.0_f32).sqrt();
            let mut x = center.x + distance * angle.cos();
            let mut y = center.y + distance * angle.sin();
            x = x.clamp(0.0, 10_000.0);
            y = y.clamp(0.0, 10_000.0);
            coords.push(Coordinate::new(x, y));
        }

        coords
    }

    /// Generate clustered coordinates for a given count using the seed.
    ///
    /// Parameters
    /// - `seed`: RNG seed.
    /// - `count`: Number of coordinates.
    ///
    /// Returns
    /// - `Vec<Coordinate>`: Deterministic pseudo-random coordinates.
    pub fn generate_clustered_coordinates(seed: u64, count: usize) -> Vec<Coordinate> {
        Self::clustered_coordinates(seed, count)
    }

    /// Generate airports and orders from a random seed.
    ///
    /// Parameters
    /// - `seed`: RNG seed.
    /// - `num_airports`: Number of airports (default when `None`).
    ///
    /// Returns
    /// - `Map`: New map with initial orders stocked.
    pub fn generate_from_seed(seed: u64, num_airports: Option<usize>) -> Self {
        let num_airports = num_airports.unwrap_or(12);

        let coordinates = Self::clustered_coordinates(seed, num_airports);
        let mut airport_list = Vec::with_capacity(num_airports);

        for (i, coordinate) in coordinates.into_iter().enumerate() {
            let airport = Airport::generate_random(seed, i);
            airport_list.push((airport, coordinate));
        }

        let mut map = Map {
            num_airports,
            airports: airport_list,
            seed,
            next_order_id: 0,
            demand_params: DemandGenerationParams::default(),
        };

        map.restock_airports();

        map
    }

    /// Restock all airports with new orders using current demand parameters.
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
                &self.demand_params,
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

    /// Find the minimum distance between two airports and the index of one endpoint.
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
    ///
    /// Parameters
    /// - `seed`: RNG seed for future procedures.
    /// - `airports`: Airport definitions and their coordinates.
    /// - `demand_params`: Demand generation parameters.
    /// - `next_order_id`: Starting order counter.
    ///
    /// Returns
    /// - `Map`: Constructed map using provided airports.
    pub fn from_airports(
        seed: u64,
        airports: Vec<(Airport, Coordinate)>,
        demand_params: DemandGenerationParams,
        next_order_id: usize,
    ) -> Self {
        let mut map = Map {
            num_airports: airports.len(),
            airports,
            seed,
            next_order_id,
            demand_params,
        };

        for (airport, _) in map.airports.iter_mut() {
            airport.ensure_base_fuel_price();
        }

        map
    }
}
