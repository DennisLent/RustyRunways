use crate::utils::airport::Airport;
use rand::{Rng, SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    pub num_airports: usize,
    pub airports: Vec<(Airport, (usize, usize))>,
    pub seed: u64,
}

impl Map {
    pub fn generate_from_seed(seed: u64, num_airports: Option<usize>) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);

        let num_airports = num_airports.unwrap_or_else(|| rng.gen_range(4..=10));

        let mut airport_list = Vec::with_capacity(num_airports);

        for i in 0..num_airports {
            let x: usize = rng.gen_range(0..=10_000);
            let y: usize = rng.gen_range(0..=10_000);

            let airport = Airport::generate_random(seed, i);

            airport_list.push((airport, (x, y)));
        }

        Map {
            num_airports,
            airports: airport_list,
            seed,
        }
    }

    pub fn restock_airports(&mut self){
        
        for (airport, _) in self.airports.iter_mut(){
            airport.generate_orders(self.seed, self.num_airports);
        }
    }
}
